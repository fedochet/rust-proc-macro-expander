#![feature(proc_macro_internals)]
#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
extern crate proc_macro;
extern crate dylib;
extern crate syn;
extern crate quote;
extern crate goblin;

extern crate syntax;
extern crate syntax_pos;
extern crate proc_macro2;

use std::path::PathBuf;
use std::fs;

mod rustc_server;

use proc_macro::bridge::server::Diagnostic;

use proc_macro::Spacing;
use proc_macro::LineColumn;

// internals
use proc_macro::TokenStream;
use dylib::DynamicLibrary;

use quote::ToTokens;

use proc_macro::bridge::{server, TokenTree};
use proc_macro::bridge::client::ProcMacro;

use goblin::mach::Mach;

static NEW_REGISTRAR_SYMBOL: &str = "__rustc_proc_macro_decls_";
static _OLD_REGISTRAR_SYMBOL: &str = "__rustc_derive_registrar_";

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
        },

        Object::PE(pe) => {
            // TODO: handle windows dlls
            None
        },

        Object::Mach(mach) => {
            match mach {
                Mach::Binary(binary) => {
                    let exports = binary.exports().ok()?;
                    let names = exports.iter().map(|s| s.name.clone()).collect();

                    Some(names)
                }

                Mach::Fat(fat) => None
            }
        },

        Object::Archive(_) | Object::Unknown(_) => None,
    }
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

fn get_proc_macros(file: &PathBuf) -> Option<&&[ProcMacro]> {
    let symbol_name = find_registrar_symbol(file)?;
    let lib = DynamicLibrary::open(Some(file)).ok()?;

    unsafe {
        let symbol = lib.symbol(&symbol_name).ok()?;
        let registrar = std::mem::transmute::<*mut u8, &&[ProcMacro]>(symbol);
        std::mem::forget(lib); // let library live for the rest of the execution

        Some(registrar)
    }
}

fn list_files(path: &str) -> Vec<PathBuf> {
    if let Ok(paths) = fs::read_dir(path) {
        return paths.into_iter()
            .filter_map(|res| res.ok())
            .map(|file| file.path())
            .collect();
    }

    vec![]
}

use std::io;
use std::fs::File;
use std::io::Read;
use goblin::{error, Object};
use goblin::elf::sym::{Symtab, Sym};

fn main() {
    let fixed = env!("CARGO_PKG_NAME").replace("-", "_");

    let paths = list_files("./another_so_files");

    for path in &paths {
        println!("{:?}", path.file_name());

        if let Some(function) = get_proc_macros(path) {
            println!("Can parse this")
        } else {
            println!("Nope")
        }
    }

//        if let Some(file_name) = path.to_str() {
//            if file_name.contains(&fixed) {
//                continue
//            }
//        }
//
//        unsafe {
//            if let Some(proc_macros) = get_plugin_registrar_fun(path) {
////                let mut registry = MyRegistrar {};
//                for a in proc_macros.iter() {
//                    match a {
//                        ProcMacro::CustomDerive { trait_name, attributes, client } => {
//
//                            let s = syn::parse_file(&input_code).unwrap();
//
//                            println!("// Calling {} expander!", trait_name);
//
//                            let t = s.into_token_stream();
//
//                            let res = client.run(rustc_server::Rustc {}, t);
//
//                            if let Ok(res) = res {
//                                println!("{}", res);
//                            }
//                        }
//                        _ => {}
//                    }
//                }
//            }
//        }
//    }
}
