# rust-proc-macro-expander

This utility is capable of calling compiled Rust custom derive dynamic libraries on arbitrary code.

**IMPORTANT**: compiler API, used in this utility, is not in stable `rustc` build.
Use nightly rustc version to build it.

## Usage

**IMPORTANT**: should be built with `RUSTFLAGS='--cfg procmacro2_semver_exempt'` as stated [here](https://github.com/alexcrichton/proc-macro2#unstable-features).

Expander launches as CLI tool and accepts json array of expansion tasks from stdin. 

Assuming you have `expansion_task.json` in current directory, 
and compiled procedural macro `id_macro` somewhere:
 
```json
[
  {
    "macro_body": "struct S {}", 
    "macro_names": [ "id_macro" ],
    "libs": [ "path/to/libid_macro.so" ]
  }
]
```

you can launch proc_macro_expander like this: 

```
> cat expansion_task.json | ./proc_macro_expander

[ {"results": [ {"type": "success", "expansion": "struct S { }"} ] } ]
```

## Testing

You can launch tests with this command: 

```
> RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo +nightly-2019-04-01 test
```

Current stable and nighly builds are having incompatible ABIs due to [this PR](https://github.com/rust-lang/rust/pull/59820). 
That is why `nightly-2019-04-01` is used. As soon as changes from this PR make it into the stable branch, tests should be 
able to run on current `nightly`.
