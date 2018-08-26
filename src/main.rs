#![feature(proc_macro_internals)]
#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
#![proc_macro]
extern crate proc_macro;
extern crate dylib;
extern crate elf;
extern crate syn;
extern crate quote;
extern crate syntax_pos;
extern crate syntax;
extern crate rustc_data_structures;
extern crate goblin;

use std::env;
use std::path::PathBuf;
use std::fs;
use rustc_data_structures::sync::Lrc;

mod rustc_server;

use proc_macro::bridge::server::Diagnostic;
use proc_macro::bridge::server::Span;

use proc_macro::Delimiter;
use proc_macro::Spacing;
use proc_macro::LineColumn;

use syntax::diagnostics::plugin::Level;

// internals
use proc_macro::TokenStream;
use dylib::DynamicLibrary;

use elf::File as ElfFile;
use elf::types::Symbol;

use quote::ToTokens;

use proc_macro::bridge::{server, TokenTree};
use proc_macro::bridge::client::ProcMacro;

static DERIVE_REGISTRAR_SYMBOL: &str = "__rustc_proc_macro_decls_";

fn open_elf_file(path: &PathBuf) -> Option<ElfFile> {
    elf::File::open_path(path).ok()
}

fn get_registrar_function(file: &ElfFile) -> Option<String> {
    let text_scn = file.get_section(".symtab")?;
    let sections = file.get_symbols(text_scn).ok()?;

    sections.iter()
        .find(|s| s.name.contains(DERIVE_REGISTRAR_SYMBOL))
        .map(|s| s.name.to_string())

}

fn find_registrar_function(file: &PathBuf) -> Option<String> {
    let elf_file = open_elf_file(file)?;
    let function = get_registrar_function(&elf_file);
    function
}

unsafe fn get_plugin_registrar_fun(file: &PathBuf) -> Option<&&[ProcMacro]> {
    let symbol_name = find_registrar_function(file)?;
    let lib = DynamicLibrary::open(Some(file)).ok()?;

    let symbol = lib.symbol(&symbol_name).ok()?;
    let registrar = std::mem::transmute::<*mut u8, &&[ProcMacro]>(symbol);
    std::mem::forget(lib);

    Some(registrar)
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

    let mut input_code = String::new();

//    io::stdin().read_line(&mut input_code);

    for path in &paths {

        let mut fd = File::open(path).unwrap();
        let mut buffer = Vec::new();
        fd.read_to_end(&mut buffer).unwrap();

        match Object::parse(&buffer).unwrap() {
//            Object::Elf(elf) => {
//                let tab = elf.dynstrtab.to_vec();
//                for sym in tab.iter() {
//                    println!("{:?}", sym);
//                }
//            },
//            Object::Mach(mach) => {
//                println!("{:?}", mach);
//            },
//            Object::PE(pe) => {
//                println!("pe: {:#?}", &pe);
//            },
//            _ => {}
            Object::Elf(elf) => {
                println!("elf:");
            },
            Object::PE(pe) => {
                println!("pe");
            },
            Object::Mach(mach) => {
                match mach {

                    goblin::mach::Mach::Fat(fat) => { println!("Fat mach") }

                    goblin::mach::Mach::Binary(binary) => {
                        for e in mach.exports().unwrap() {
                            if e.name.contains("rustc") {
                                println!("{}", e.name)
                            }
                        }
                    }
                }
            },
            Object::Archive(archive) => {
                println!("archive");
            },
            Object::Unknown(magic) => { println!("unknown magic: {:#x}", magic) }
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
