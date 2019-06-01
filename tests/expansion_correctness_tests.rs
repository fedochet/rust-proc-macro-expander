extern crate proc_macro_expander;
extern crate tempfile;
#[macro_use]
extern crate assert_matches;

use proc_macro_expander::macro_expansion::{ExpansionTask, ExpansionResult};

use std::fs::{canonicalize, create_dir, File};
use std::{io, fs};
use std::io::Write;
use std::io::ErrorKind;
use std::path::{PathBuf, Path};
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

fn setup_project_with_derives(root_dir: &Path) -> io::Result<()> {
    let mut cargo_toml = File::create(root_dir.join("Cargo.toml"))?;
    write!(
        &mut cargo_toml,
        "{}",
        r#"
[package]
name = "test_proc_macro"
version = "0.1.0"

[dependencies]
serde_derive = "1.0.0"
getset = "0.0.7"
derive_builder = "0.7.1"
    "#
    )?;

    create_dir(root_dir.join("src"))?;
    let mut main_file = File::create(root_dir.join("src").join("main.rs"))?;
    write!(
        &mut main_file,
        "{}",
        r#"
fn main() {}
    "#
    )?;

    Ok(())
}

fn setup_proc_macro_project(root_dir: &Path) -> io::Result<()> {
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


#[cfg(not(target_os = "windows"))]
static DYLIB_NAME_PREFIX: &str = "lib";

#[cfg(target_os = "windows")]
static DYLIB_NAME_PREFIX: &str = "";

fn compile_proc_macro(dir: &Path, proc_macro_name: &str) -> io::Result<PathBuf> {
    Command::new("cargo")
        .current_dir(dir)
        .arg("+nightly")
        .arg("build")
        .arg("-p").arg(proc_macro_name)
        .status()?;

    let buf = dir
        .join("target")
        .join("debug")
        .join(format!("{}{}{}", DYLIB_NAME_PREFIX, proc_macro_name, DYLIB_NAME_EXTENSION));

    if buf.is_file() {
        Ok(canonicalize(buf)?)
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
        &serde_json::to_string(&vec![&task])?
    )?;

    result.wait()?;

    let results: Vec<ExpansionResult> = serde_json::from_reader(result.stdout.unwrap())?;

    // FIXME this is terrible
    Ok(results.into_iter().nth(0).expect(
        &format!("Expansion results for task {:?} are empty!", &task)
    ))
}

#[test]
fn test_simple_bang_proc_macros() -> io::Result<()> {
    let tmp_dir = TempDir::new().expect("Cannot create temp dir");
    setup_proc_macro_project(&tmp_dir.path()).expect("Cannot setup test project");
    let proc_macro_dyn_lib = compile_proc_macro(&tmp_dir.path(), "test_proc_macro")
        .expect("Cannot find proc macro!");

    {
        let id_macro_task = ExpansionTask {
            libs: vec![proc_macro_dyn_lib.clone()],
            macro_body: "struct S {}".to_string(),
            macro_name: "id_macro".to_string(),
            attributes: None,
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
            macro_name: "make_answer_macro".to_string(),
            attributes: None,
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

#[test]
fn test_proc_macro_libraries() {
    let tmp_dir = TempDir::new().expect("Cannot create temp dir");
    setup_project_with_derives(&tmp_dir.path()).expect("Cannot setup test project");
    let getset_lib = compile_proc_macro(&tmp_dir.path(), "getset")
        .expect("Cannot find proc macro!");

    {
        let expansion_task = ExpansionTask {
            libs: vec![getset_lib.clone()],
            macro_body: "struct S { #[set] y: i32 }".to_string(),
            macro_name: "Setters".to_string(),
            attributes: None,
        };

        let expansion_result = perform_expansion(expansion_task).expect(
            "Cannot perform expansion for 'Setters'"
        );

        assert_matches!(
            expansion_result,
            ExpansionResult::Success { ref expansion }
            if expansion.contains("fn set_y")
        );
    }
}

