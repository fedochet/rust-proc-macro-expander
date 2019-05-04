#![feature(proc_macro_internals)]
#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
extern crate proc_macro_expander;

use std::io::Read;

use proc_macro_expander::macro_expansion::{ExpansionResults, ExpansionTask};

fn read_stdin() -> String {
    let mut buff = String::new();
    std::io::stdin()
        .read_to_string(&mut buff)
        .expect("Cannot read from stdin!");

    buff
}

fn main() {
    let input = read_stdin();
    let expansion_tasks: Vec<ExpansionTask> =
        serde_json::from_str(&input).expect(&format!("Cannot parse '{}'", &input));

    let results: Vec<ExpansionResults> = expansion_tasks
        .iter()
        .map(|task| proc_macro_expander::expand_task(&task))
        .collect();

    println!(
        "{}",
        &serde_json::to_string(&results).expect("Cannot serialize results!")
    );
}
