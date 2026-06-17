# Release Gate — Phase 65 / Plan 65-03

## Pre-flight

| Check | Value |
|-------|-------|
| `cargo --version` | cargo 1.94.1 (29ea6fb6a 2026-03-24) |
| `rustc --version` | rustc 1.94.1 (e408947bf 2026-03-25) |
| `build.rs` exists | YES (`-rw-r--r--@ 231 bytes`) |
| `wasm32-unknown-unknown` target installed | YES |

## Part 1 — CI Matrix

### `cargo test`
Exit code: 0
Output (head):
```
running 1661 tests
...
```
Output (tail):
```
test result: ok. 267 passed; 0 failed; 29 ignored; 0 measured; 0 filtered out; finished in 27.96s
```

### `cargo test --features serde`
Exit code: 0
Output (head):
```
Compiling genetic_algorithms v3.0.0 (/Users/luis/RustroverProjects/genetic-algorithms)
...
```
Output (tail):
```
test result: ok. 267 passed; 0 failed; 29 ignored; 0 measured; 0 filtered out
```

### `cargo clippy --all-targets -- -D warnings`
Exit code: 0
Output (head):
```
Checking genetic_algorithms v3.0.0
...
```
Output (tail):
```
Finished `dev` profile [optimized + debuginfo] target(s) in 3.48s
```
Warning count: 0

### `cargo doc --no-deps --all-features`
Exit code: 0
Warning grep (`grep -c '^warning:'`): 0
Output (tail):
```
Generated /Users/luis/RustroverProjects/genetic-algorithms/target/doc/genetic_algorithms/index.html
Finished `dev` profile [optimized + debuginfo] target(s) in 1.53s
```

### `cargo check --target wasm32-unknown-unknown --no-default-features --features logging`
Exit code: 0
Output (tail):
```
Finished `dev` profile [optimized + debuginfo] target(s) in 1.58s
```

## Part 2 — cargo publish --dry-run

Exit code: 0
Output:
```
Updating crates.io index
Packaging genetic_algorithms v3.0.0
Packaged 455 files, 4.3MiB (829.1KiB compressed)
Verifying genetic_algorithms v3.0.0
Finished `dev` profile [optimized + debuginfo] target(s) in 3.57s
Uploading genetic_algorithms v3.0.0
warning: aborting upload due to dry run
```
Note: Dry-run upload aborted as expected — `cargo publish` would succeed.
