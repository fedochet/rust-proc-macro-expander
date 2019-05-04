#[macro_use]
extern crate serde_json;
extern crate proc_macro_expander;
extern crate tempfile;
#[macro_use]
extern crate assert_matches;

use proc_macro_expander::macro_expansion::{ExpansionTask, ExpansionResults, ExpansionResult};

use std::fs::{canonicalize, create_dir, DirEntry, File};
use std::io;
use std::io::Write;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn proc_macro_expander_exe() -> io::Result<PathBuf> {
    let mut test_exe = std::env::current_exe()?;

    test_exe.pop();
    if test_exe.ends_with("deps") {
        test_exe.pop();
    }
    test_exe.push("proc_macro_expander");

    Ok(test_exe)
}

fn setup_proc_macro_project(root_dir: &PathBuf) -> io::Result<()> {
    let mut cargo_toml = File::create(root_dir.join("Cargo.toml"))?;
    write!(
        &mut cargo_toml,
        "{}",
        r#"
[package]
name = "test_proc_macro"
version = "0.1.0"

[lib]
proc-macro = true

[dependencies]
    "#
    )?;

    create_dir(root_dir.join("src"))?;
    let mut lib_file = File::create(root_dir.join("src").join("lib.rs"))?;
    write!(
        &mut lib_file,
        "{}",
        r#"
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn id_macro(input: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
    "#
    )?;

    Ok(())
}

fn compile_proc_macro(dir: &PathBuf) -> io::Result<PathBuf> {
    Command::new("cargo")
        .current_dir(dir)
        .arg("+stable")
        .arg("build")
        .status()?;

    // FIXME change for windows

    let buf = dir
        .join("target")
        .join("debug")
        .join("libtest_proc_macro.so");
    if buf.is_file() {
        Ok(buf)
    } else {
        Err(io::Error::from(ErrorKind::NotFound))
    }
}

#[test]
fn foo() -> io::Result<()> {
    let tmp_dir = TempDir::new()?;
    setup_proc_macro_project(&tmp_dir.path().to_path_buf())?;
    let proc_macro_dyn_lib = compile_proc_macro(&tmp_dir.path().to_path_buf())?;

    let expander = proc_macro_expander_exe()?;

    let mut result = Command::new(expander)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let task = ExpansionTask {
        libs: vec![proc_macro_dyn_lib],
        macro_body: "".to_string(),
        attributes: None,
        macro_names: vec!["id_macro".to_string()],
    };

    write!(
        result.stdin.as_mut().unwrap(),
        "{}",
        &serde_json::to_string(&vec![task])?
    )?;

    result.wait()?;

    let results: Vec<ExpansionResults> = serde_json::from_reader(result.stdout.unwrap())?;
    let expected_success = &results[0].results[0];

    assert_matches!(
        expected_success,
        ExpansionResult::Success { expansion } if expansion.contains("answer")
    );

    Ok(())
}
