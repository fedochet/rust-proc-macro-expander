# rust-proc-macro-expander

This utility is capable of calling compiled Rust custom derive dynamic libraries on arbitrary code.

**IMPORTANT**: compiler API, used in this utility, is not in stable `rustc` build.
Use nightly rustc version to build it.

## Usage

**IMPORTANT**: should be built with `RUSTFLAGS='--cfg procmacro2_semver_exempt'` as stated [here](https://github.com/alexcrichton/proc-macro2#unstable-features).