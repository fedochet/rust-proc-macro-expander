# rust-proc-macro-expander

This utility is capable of calling compiled Rust custom derive dynamic libraries on arbitrary code.

**IMPORTANT**: compiler API, used in this utility, is not in stable or even nightly `rustc` build.
In order to use it, you have to build [this PR](https://github.com/rust-lang/rust/pull/49219) 
locally, and use it to compile this project.

## Usage

**IMPORTANT**: should be built with `RUSTFLAGS='--cfg procmacro2_semver_exempt'` as stated [here](https://github.com/alexcrichton/proc-macro2#unstable-features).