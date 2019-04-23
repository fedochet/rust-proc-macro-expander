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
#[macro_use]
extern crate serde_derive;
extern crate serde;

use serde::{Serialize, Deserialize};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{App, Arg};
use dylib::DynamicLibrary;
use goblin::mach::Mach;
use goblin::Object;

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
    syn::parse_str(code).ok()
}

struct ExpansionArgs {
    libs: Vec<PathBuf>,
}

struct Expander {
    derives: Vec<(ProcMacro)>
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

    fn expand(&self, code: &str, macro_to_expand: &str) -> Result<String, proc_macro::bridge::PanicMessage> {
        let token_stream = parse_string(code).expect(
            &format!("Error while parsing this code: '{}'", code)
        );

        for derive in &self.derives {
            match derive {
                ProcMacro::CustomDerive { trait_name, client, .. }
                if *trait_name == macro_to_expand => {
                    let res = client.run(&EXEC_STRATEGY, rustc_server::Rustc::default(), token_stream);

                    return res.map(|token_stream| token_stream.to_string());
                }

                ProcMacro::Bang { name, client }
                if *name == macro_to_expand => {
                    let res = client.run(&EXEC_STRATEGY, rustc_server::Rustc::default(), token_stream);

                    return res.map(|token_stream| token_stream.to_string());
                }

                ProcMacro::Attr { name, client }
                if *name == macro_to_expand => {
                    // fixme attr macro needs two inputs
                    let res = client.run(&EXEC_STRATEGY, rustc_server::Rustc::default(), proc_macro2::TokenStream::new(), token_stream);

                    return res.map(|token_stream| token_stream.to_string());
                }

                _ => { continue; }
            }
        }

        Err(proc_macro::bridge::PanicMessage::String("Nothing to expand".to_string()))
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
        .get_matches();

    let libs = matches.values_of("libs").expect("Cannot expand without specified --libs!");
    let libs = libs.map(|lib| PathBuf::from(lib)).collect();

    ExpansionArgs { libs }
}

fn read_stdin() -> String {
    let mut buff = String::new();
    std::io::stdin().read_to_string(&mut buff).expect("Cannot read from stdin!");

    buff
}

#[derive(Deserialize)]
struct ExpansionTask {
    /// Argument of macro call.
    ///
    /// In custom derive that would be a struct or enum; in attribute-like macro - underlying
    /// item; in function-like macro - the macro body.
    macro_body: String,

    /// Names of macros to expand.
    ///
    /// In custom derive those are names of derived traits (`Serialize`, `Getters`, etc.). In
    /// attribute-like and functiona-like macros - single name of macro itself (`show_streams`).
    macro_names: Vec<String>,

    /// Possible attributes for the attribute-like macros.
    attributes: Option<String>,
}

#[derive(Serialize)]
struct ExpansionResults {
    results: Vec<ExpansionResult>
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ExpansionResult {
    #[serde(rename = "success")]
    Success { expansion: String },
    #[serde(rename = "error")]
    Error { reason: String },
}

fn main() {
    let args = parse_args();
    let expander = Expander::new(&args.libs).expect(
        &format!("Cannot perform expansion wit those libs: {:?}", &args.libs)
    );

    let input = read_stdin();
    let expansion_tasks: Vec<ExpansionTask> = serde_json::from_str(&input).expect(
        &format!("Cannot parse '{}'", &input)
    );

    let mut results = vec![];

    for task in expansion_tasks {
        let mut task_results = vec![];

        for derive in &task.macro_names {
            match expander.expand(&task.macro_body, &derive) {
                Ok(expansion) => task_results.push(ExpansionResult::Success { expansion }),

                Err(msg) => {
                    let reason = format!("Cannot perform expansion for {}: error {:?}!", derive, msg.as_str());
                    task_results.push(ExpansionResult::Error { reason })
                }
            }
        }

        results.push(ExpansionResults { results: task_results })
    }

    println!("{}", &serde_json::to_string(&results).expect("Cannot serialize results!"));
}
