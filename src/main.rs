#![feature(proc_macro_internals)]
#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
extern crate clap;
extern crate dylib;
extern crate goblin;
extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;
extern crate syntax;
extern crate syntax_pos;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{App, Arg};
use dylib::DynamicLibrary;
use goblin::mach::Mach;
use goblin::Object;
use quote::ToTokens;

use proc_macro::bridge::client::ProcMacro;
use proc_macro::bridge::server::SameThread;

mod rustc_server;

static NEW_REGISTRAR_SYMBOL: &str = "__rustc_proc_macro_decls_";
static _OLD_REGISTRAR_SYMBOL: &str = "__rustc_derive_registrar_";

pub const EXEC_STRATEGY: SameThread = SameThread;

fn read_bytes(file: &PathBuf) -> Option<Vec<u8>> {
    let mut fd = File::open(file).ok()?;
    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer).ok()?;

    Some(buffer)
}

fn get_symbols_from_lib(file: &PathBuf) -> Option<Vec<String>> {
    let buffer = read_bytes(file)?;
    let object = Object::parse(&buffer).ok()?;

    return match object {
        Object::Elf(elf) => {
            let symbols = elf.dynstrtab.to_vec().ok()?;
            let names = symbols.iter().map(|s| s.to_string()).collect();

            Some(names)
        }

        Object::PE(_) => {
            // TODO: handle windows dlls
            None
        }

        Object::Mach(mach) => {
            match mach {
                Mach::Binary(binary) => {
                    let exports = binary.exports().ok()?;
                    let names = exports.iter().map(|s| s.name.clone()).collect();

                    Some(names)
                }

                Mach::Fat(_) => None
            }
        }

        Object::Archive(_) | Object::Unknown(_) => None,
    };
}

fn is_derive_registrar_symbol(symbol: &str) -> bool {
    symbol.contains(NEW_REGISTRAR_SYMBOL)
}

fn find_registrar_symbol(file: &PathBuf) -> Option<String> {
    let symbols = get_symbols_from_lib(file)?;

    symbols.iter()
        .find(|s| is_derive_registrar_symbol(s))
        .map(|s| s.clone())
}

fn get_proc_macros(file: &PathBuf) -> Result<&'static &'static [ProcMacro], String> {
    let symbol_name = find_registrar_symbol(file).ok_or(
        format!("Cannot find registrar symbol in file {:?}", file)
    )?;

    let lib = DynamicLibrary::open(Some(file))?;

    let registrar = unsafe {
        let symbol = lib.symbol(&symbol_name)?;
        std::mem::transmute::<*mut u8, &&[ProcMacro]>(symbol)
    };

    std::mem::forget(lib); // let library live for the rest of the execution

    Ok(registrar)
}

fn parse_string(code: &str) -> Option<proc_macro2::TokenStream> {
    let parsed_file = syn::parse_file(code).ok()?;

    Some(parsed_file.into_token_stream())
}

struct ExpansionArgs {
    libs: Vec<PathBuf>,
    derives: Option<Vec<String>>,
}

struct Expander {
    derives: Vec<ProcMacro>
}

impl Expander {
    fn new(libs: &Vec<PathBuf>) -> Result<Expander, String> {
        let mut derives: Vec<ProcMacro> = vec![];

        for lib in libs {
            let macros = get_proc_macros(lib)?;
            derives.extend(macros.iter());
        }

        Ok(Expander { derives })
    }

    fn expand(&self, code: &str, trait_to_expand: &str) -> Option<String> {
        for derive in &self.derives {
            if let ProcMacro::CustomDerive { trait_name, client, .. } = derive {
                if *trait_name == trait_to_expand {
                    let token_stream = parse_string(code).expect(
                        &format!("Error while parsing this code: '{}'", code)
                    );
                    let res = client.run(&EXEC_STRATEGY, rustc_server::Rustc::default(), token_stream);

                    return res.ok().map(|token_stream| token_stream.to_string());
                }
            }
        }

        None
    }

    fn expand_for_all_derives(&self, code: &str) -> Vec<String> {
        let mut result = vec![];

        for d in &self.derives {
            if let ProcMacro::CustomDerive { client, .. } = d {
                let token_stream = parse_string(code).expect(
                    &format!("Error while parsing this code: '{}'", code)
                );

                let res = client.run(&EXEC_STRATEGY, rustc_server::Rustc::default(), token_stream);

                if let Ok(res) = res {
                    result.push(res.to_string())
                }
            }
        }

        result
    }
}

fn parse_args() -> ExpansionArgs {
    let matches = App::new("proc_macro_expander")
        .version("1.0")
        .about("Expands procedural macros")
        .arg(Arg::with_name("libs")
            .short("l")
            .long("libs")
            .value_name("LIBFILE")
            .multiple(true)
            .required(true)
            .help("Compiled proc macro libraries")
            .takes_value(true))
        .arg(Arg::with_name("derives")
            .short("d")
            .long("derives")
            .value_name("TRAIT")
            .multiple(true)
            .takes_value(true)
            .help("Traits for which expansions should be performed"))
        .get_matches();

    let libs = matches.values_of("libs").expect("Cannot expand without specified --libs!");
    let libs = libs.map(|lib| PathBuf::from(lib)).collect();

    let derives = match matches.values_of("derives") {
        Some(derives) => Some(derives.map(|derive| derive.to_string()).collect()),
        None => None
    };

    ExpansionArgs { libs, derives }
}

fn read_stdin() -> String {
    let mut buff = String::new();
    std::io::stdin().read_to_string(&mut buff).expect("Cannot read from stdin!");

    buff
}

fn main() {
    let args = parse_args();
    let expander = Expander::new(&args.libs).expect(
        &format!("Cannot perform expansion wit those libs: {:?}", &args.libs)
    );

    let code_to_expand = read_stdin();

    if let Some(derives) = args.derives {
        for derive in derives {
            let expansion = expander.expand(&code_to_expand, &derive).expect(
                &format!("Cannot perform expansion for {}!", derive)
            );

            println!("{}", expansion);
        }
    } else {
        for expansion in expander.expand_for_all_derives(&code_to_expand) {
            println!("{}", expansion)
        }
    }
}
