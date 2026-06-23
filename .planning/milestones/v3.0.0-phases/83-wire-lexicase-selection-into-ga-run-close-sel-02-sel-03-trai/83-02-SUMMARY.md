---
phase: 83-wire-lexicase-selection-into-ga-run-close-sel-02-sel-03-trai
plan: "02"
subsystem: engines/lexicase
tags: [lexicase, integration-tests, sel-02, sel-03, traits-01]
status: complete

dependency_graph:
  requires: ["83-01"]
  provides: ["lexicase-integration-tests"]
  affects: ["tests/engines/lexicase", "tests/test_engines.rs"]

tech_stack:
  added: []
  patterns: ["GA builder + run_lexicase()", "VectorFitness trait test coverage", "with_fitness_fn trigger pattern for fixture initialization"]

key_files:
  created:
    - tests/engines/lexicase/test_ga_run_lexicase.rs
  modified:
    - tests/test_engines.rs

decisions:
  - "Provided with_fitness_fn(|_| 0.0) to trigger calculate_fitness() during initialization — build_one_chromosome only calls calculate_fitness when a fitness_fn is attached; without it fitness_values stay empty and factory_lexicase rejects the population"
  - "Used SelectionConfig and StoppingConfig traits in imports — these builder methods live behind trait bounds not auto-imported in test scope"
  - "Annotated fv as VectorFitness::fitness_values(c) for explicit trait dispatch to resolve type inference ambiguity in the mean-sync loop"

metrics:
  duration: "407 seconds (~7 minutes)"
  completed: "2026-06-23T19:37:24Z"
  tasks_completed: 2
  files_changed: 2

requirements: [SEL-02, SEL-03, TRAITS-01]
---

# Phase 83 Plan 02: Lexicase GA-Run Integration Tests Summary

End-to-end integration tests proving `Ga::<MultiCaseChromosome>::run_lexicase()` completes for both `Selection::Lexicase` and `Selection::EpsilonLexicase`, with the standard `run()` guard and D-04 mean-sync invariant verified.

## Objective

Add the missing integration tests that exercise the full Phase 83 lexicase wiring: `run_lexicase()`, `EpsilonLexicase`, the `run()` error guard naming `run_lexicase`, the D-04 scalar-fitness mean-sync, and population diversity preservation.

## Tasks Completed

### T01 — Create the lexicase GA-run integration test file

Created `tests/engines/lexicase/test_ga_run_lexicase.rs` with five tests:

| Test Function | Requirement | What It Proves |
|---|---|---|
| `test_ga_run_lexicase_completes` | SEL-02 | `run_lexicase()` with `Selection::Lexicase` returns `Ok` with non-empty population |
| `test_ga_run_epsilon_lexicase_completes` | SEL-03 | `run_lexicase()` with `Selection::EpsilonLexicase` (epsilon=0.5) completes |
| `test_run_lexicase_on_non_vector_fitness_returns_error` | SEL-02 | `run()` with `Selection::Lexicase` returns `Err(ConfigurationError)` mentioning `run_lexicase` |
| `test_lexicase_mean_sync_in_run` | TRAITS-01/D-04 | After 1-generation run, every chromosome's `fitness()` equals mean of `fitness_values()` |
| `test_run_lexicase_diversity` | SEL-02 | Final population contains >= 2 distinct `fitness_values()` profiles (diversity preserved) |

**Commit:** `ce63635`

### T02 — Register the lexicase test module and run the suite

Added `mod lexicase { mod test_ga_run_lexicase; }` inside the `mod engines { ... }` block in `tests/test_engines.rs`, placed after the `mod island { ... }` block.

All gates passed:
- `cargo test test_ga_run_lexicase`: 5 passed, 0 failed
- `cargo test`: 1582 passed, 6 ignored
- `cargo test --features serde`: 1627 passed, 6 ignored
- `cargo clippy --all-targets -- -D warnings`: clean
- `cargo check --target wasm32-unknown-unknown`: success

**Commit:** `815fce4`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] fitness_values empty on initial population**
- **Found during:** T01 first run
- **Issue:** `MultiCaseChromosome::calculate_fitness()` was never called during initialization because `build_one_chromosome` only calls `calculate_fitness()` when a `fitness_fn` is provided. With default `fitness = 0.0` (not NaN), `population.fitness_calculation()` also skips calling it. Result: `factory_lexicase` rejected all chromosomes with `"fitness_values() is empty"`.
- **Fix:** Added `.with_fitness_fn(|_dna: &[Gene]| 0.0)` to each builder in the test file. This triggers `calculate_fitness()` during initialization. The closure's return value is ignored — `MultiCaseChromosome::calculate_fitness()` computes `fitness_values` and `fitness` from gene IDs.
- **Files modified:** `tests/engines/lexicase/test_ga_run_lexicase.rs`

**2. [Rule 1 - Bug] Missing trait imports for builder methods**
- **Found during:** T01 first compile
- **Issue:** `with_selection_method()` and `with_max_generations()` require `SelectionConfig` and `StoppingConfig` traits in scope; these are not transitively imported via `ConfigurationT`.
- **Fix:** Added `SelectionConfig` and `StoppingConfig` to the `use genetic_algorithms::traits::...` import.
- **Files modified:** `tests/engines/lexicase/test_ga_run_lexicase.rs`

**3. [Rule 1 - Bug] Type inference failure in fitness_values loop**
- **Found during:** T01 first compile
- **Issue:** `c.fitness_values()` in a loop over `&MultiCaseChromosome` triggered `E0282` because the compiler couldn't infer the dispatch target.
- **Fix:** Changed to `VectorFitness::fitness_values(c)` for explicit trait dispatch.
- **Files modified:** `tests/engines/lexicase/test_ga_run_lexicase.rs`

## Known Stubs

None.

## Threat Flags

None — test-only changes; no new network surface, auth paths, or schema changes.

## Self-Check: PASSED

- `tests/engines/lexicase/test_ga_run_lexicase.rs` — FOUND
- `tests/test_engines.rs` (mod lexicase) — FOUND
- Commit `ce63635` — FOUND (`git log --oneline | grep ce63635`)
- Commit `815fce4` — FOUND
- `cargo test test_ga_run_lexicase` — 5 passed
