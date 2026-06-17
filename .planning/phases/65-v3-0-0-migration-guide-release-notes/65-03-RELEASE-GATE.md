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

## Part 3 — v2 sample crate smoke-test

### Sub-step A — Create crate
Created `/tmp/ga_v2_smoke` with `cargo new --lib`.

### Sub-step B — Write v2 `src/lib.rs`
Exercised top 3 breaking patterns:
1. `SmokeChrom` implementing v2 `ChromosomeT` with `dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()` inline
2. `use genetic_algorithms::reporter::SimpleReporter` + `.with_reporter(Box::new(SimpleReporter::new(10)))`
3. `SmokeSelection` implementing v2 `SelectionOperator` with 3-param `select()` returning `Vec<(usize, usize)>`

### Sub-step C — Baseline build against v2.4.0
Exit code: 0
Output (tail):
```
warning: use of deprecated method `genetic_algorithms::ga::Ga::<U>::with_reporter`: use with_observer() instead.
Finished `dev` profile [optimized + debuginfo] target(s) in 0.07s
```

### Sub-step D — Switch to in-progress v3 (path dependency)
Exit code: NON-ZERO (expected)
Captured v3 errors:

#### Captured v3 error: E0432 (Reporter import)
```
error[E0432]: unresolved import `genetic_algorithms::reporter`
 --> src/lib.rs:3:25
  |
3 | use genetic_algorithms::reporter::SimpleReporter;
  |                         ^^^^^^^^ could not find `reporter` in `genetic_algorithms`
```

#### Captured v3 error: E0407 (ChromosomeT method removal)
```
error[E0407]: method `dna` is not a member of trait `ChromosomeT`
error[E0407]: method `dna_mut` is not a member of trait `ChromosomeT`
error[E0407]: method `set_dna` is not a member of trait `ChromosomeT`
error[E0407]: method `set_fitness_fn` is not a member of trait `ChromosomeT`
```

#### Captured v3 error: E0277 (LinearChromosome bound)
```
error[E0277]: the trait bound `SmokeChrom: LinearChromosome` is not satisfied
  --> src/lib.rs:24:23
   |
24 | impl ValueMutable for SmokeChrom {}
   |                       ^^^^^^^^^^ unsatisfied trait bound
```

#### Captured v3 error: E0050 (SelectionOperator parameter count)
```
error[E0050]: method `select` has 4 parameters but the declaration in trait `select` has 5
  --> src/lib.rs:50:9
   |
50 | /         &self,
51 | |         _chromosomes: &[U],
52 | |         _couples: usize,
53 | |         _threads: usize,
   | |_______________________^ expected 5 parameters, found 4
   |
   = note: `select` from trait: `fn(&Self, &[U], usize, usize, usize) -> Vec<Vec<usize>>`
```

### Sub-step E — Reconciliation

| Pattern | Captured Error | MIGRATION.md | Outcome | MIGRATION.md Change |
|---------|---------------|-------------|---------|-------------------|
| Reporter removal | E0432: `unresolved import genetic_algorithms::reporter` | E0432: `unresolved import genetic_algorithms::reporter` | MATCHES VERBATIM | No |
| ChromosomeT split | E0277: `LinearChromosome not satisfied` + E0407 at impl site | E0277: `LinearChromosome not satisfied` at call site | MATCHES (code, different call site) | No |
| SelectionOperator::select | E0050: `has 4 parameters but the declaration in trait select has 5` | E0053: `incompatible type for trait` | MISMATCH — code differs | YES: updated E0053 → E0050 |

**MIGRATION.md fix applied:** SelectionOperator compiler error updated from `error[E0053]` (incompatible type) to `error[E0050]` (wrong parameter count) with the actual captured rustc output.

### Sub-step F — Apply MIGRATION.md fixes and re-build
Exit code: 0
Output (tail):
```
warning: struct `SmokeGene` is never constructed
warning: struct `SmokeChrom` is never constructed
warning: struct `SmokeSelection` is never constructed
Finished `dev` profile [optimized + debuginfo] target(s) in 0.09s
```

### Sub-step G — Cleanup
Smoke crate directory removed: `test ! -d "${TMPDIR:-/tmp}/ga_v2_smoke"` exits 0.

### Reconciliation summary

| Pattern | Captured error code | MIGRATION.md outcome | MIGRATION.md change required |
|---------|--------------------|--------------------|-----------------------------|
| ChromosomeT + LinearChromosome split | E0277 / E0407 | MATCHES (code, paraphrased — different call site) | No |
| Reporter removal | E0432 | MATCHES VERBATIM | No |
| SelectionOperator::select | E0050 | MISMATCH (fixed) | Yes — E0053 → E0050 |
