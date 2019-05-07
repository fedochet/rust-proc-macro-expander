extern crate proc_macro_expander;
extern crate tempfile;
#[macro_use]
extern crate assert_matches;

use proc_macro_expander::macro_expansion::{ExpansionTask, ExpansionResults, ExpansionResult};

use std::fs::{canonicalize, create_dir, File};
use std::{io, fs};
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
    input
}

#[proc_macro]
pub fn make_answer_macro(input: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
    "#
    )?;

    Ok(())
}

#[cfg(target_os = "linux")]
static DYLIB_NAME_EXTENSION: &str = ".so";

#[cfg(target_os = "macos")]
static DYLIB_NAME_EXTENSION: &str = ".dylib";

#[cfg(target_os = "windows")]
static DYLIB_NAME_EXTENSION: &str = ".dll";

fn compile_proc_macro(dir: &PathBuf) -> io::Result<PathBuf> {
    Command::new("cargo")
        .current_dir(dir)
        .arg("+stable")
        .arg("build")
        .status()?;

    // FIXME change for windows

    for entry in fs::read_dir(dir.join("target").join("debug"))? {
        println!("{:?}", entry?.path())
    }

    let buf = dir
        .join("target")
        .join("debug")
        .join(format!("libtest_proc_macro{}", DYLIB_NAME_EXTENSION));

    if buf.is_file() {
        Ok(buf)
    } else {
        Err(io::Error::from(ErrorKind::NotFound))
    }
}

fn perform_expansion(task: ExpansionTask) -> io::Result<ExpansionResult> {
    let expander = proc_macro_expander_exe()?;

    let mut result = Command::new(expander)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    write!(
        result.stdin.as_mut().unwrap(),
        "{}",
        &serde_json::to_string(&vec![task])?
    )?;

    result.wait()?;

    let results: Vec<ExpansionResults> = serde_json::from_reader(result.stdout.unwrap())?;

    // FIXME this is terrible
    Ok(results.into_iter().nth(0).unwrap().results.into_iter().nth(0).unwrap())
}

#[test]
fn test_simple_bang_proc_macros() -> io::Result<()> {
    let tmp_dir = TempDir::new().expect("Cannot create temp dir");
    setup_proc_macro_project(&tmp_dir.path().to_path_buf()).expect("Cannot setup test project");
    let proc_macro_dyn_lib = compile_proc_macro(&tmp_dir.path().to_path_buf())
        .and_then(|p| canonicalize(p))
        .expect("Cannot find proc macro!");

    {
        let id_macro_task = ExpansionTask {
            libs: vec![proc_macro_dyn_lib.clone()],
            macro_body: "struct S {}".to_string(),
            attributes: None,
            macro_names: vec!["id_macro".to_string()],
        };

        let id_macro_expansion = perform_expansion(id_macro_task).expect(
            "Cannot perform expansion for 'id_macro'"
        );

        assert_matches!(
            id_macro_expansion,
            ExpansionResult::Success { ref expansion } if expansion.contains("struct S")
        );
    }

    {
        let make_answer_macro_task = ExpansionTask {
            libs: vec![proc_macro_dyn_lib.clone()],
            macro_body: "".to_string(),
            attributes: None,
            macro_names: vec!["make_answer_macro".to_string()],
        };

        let make_answer_macro_expansion = perform_expansion(make_answer_macro_task).expect(
            "Cannot perform expansion for 'make_answer_macro'"
        );

        assert_matches!(
            make_answer_macro_expansion,
            ExpansionResult::Success { ref expansion } if expansion.contains("fn answer")
        );
    }


    Ok(())
}
