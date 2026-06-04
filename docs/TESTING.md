<!-- generated-by: gsd-doc-writer -->
# Testing

This document describes the test framework, how to run tests, how to write new tests, and how
testing is integrated into CI for the `genetic_algorithms` crate.

## Test Framework and Setup

The project uses Rust's built-in test harness (`cargo test`) with no external test framework.
Benchmarks use [Criterion](https://github.com/bheisler/criterion.rs) (`criterion = "0.8.2"`,
declared as a dev-dependency).

No special setup is required before running tests beyond installing the Rust toolchain at the
minimum supported version (`rust-version = "1.81.0"`) and running `cargo build`.

Some test files depend on optional feature flags:

| Feature flag | Effect on tests |
|---|---|
| `serde` | Enables `tests/observe/test_checkpoint.rs`, `tests/observe/test_serde.rs`, and serde-gated blocks in `tests/types/chromosomes/test_list.rs` and `tests/types/genotypes/test_list.rs` |
| `observer-tracing` | Required for `tests/observe/observer/test_tracing_observer.rs` |
| `observer-metrics` | Required for `tests/observe/observer/test_metrics_observer.rs` and the `metrics_observer` benchmark |
| `visualization` | Required for `tests/observe/visualization/test_visualization.rs` |

## Running Tests

**Full test suite (default features):**

```bash
cargo test
```

**Full test suite including serde tests:**

```bash
cargo test --features serde
```

**Full test suite with all features enabled:**

```bash
cargo test --all-features
```

**Run a specific test file or test name:**

```bash
# Run all tests whose name contains "selection"
cargo test selection

# Run a single named test
cargo test test_tournament_selection

# Run all GP tests
cargo test --test gp

# Run all variable-length tests
cargo test --test test_variable_length
```

### Notable integration test suites

Two integration suites at the top level of `tests/` cover the v3.0.0 features end-to-end and double as runnable usage references (no standalone `examples/` for these yet):

| Test file | Coverage |
|---|---|
| [`tests/gp.rs`](../tests/gp.rs) | Genetic Programming engine — custom `GpNode` impls, ramped half-and-half initialization, subtree/point/hoist mutation, subtree crossover with bloat retry, `GpGa` end-to-end runs, `MathNode` / `BoolNode` built-ins, serde checkpointing of tree chromosomes |
| [`tests/test_variable_length.rs`](../tests/test_variable_length.rs) | Variable-length chromosomes — `ChromosomeLength::{Fixed, Variable}` enforcement, `Mutation::Insertion` / `Mutation::Deletion` operators, `Crossover::VariableLength(Trim/Pad)` alignment, parsimony pressure via `with_length_penalty`, variable-length-aware extension regrowth |

**Run benchmarks (Criterion):**

```bash
cargo bench
```

Available benchmark targets: `selection`, `crossover`, `mutation`, `survivor`, `ga_run`,
`nsga2`, `island_ga`, `de`, `scatter`, `alps`, `cellular`, `metrics_observer` (requires
`--features observer-metrics`).

## Writing New Tests

**File location:** All tests live in `tests/`. Do not add inline `#[cfg(test)] mod tests`
blocks inside implementation files.

**Naming convention:** Test files are named `test_<subject>.rs` (e.g.,
`test_crossover_single_point.rs`, `test_selection_boltzmann.rs`).

**Directory layout:**

```
tests/
  structures.rs                        # Shared Gene / Chromosome fixtures (re-used across suites)
  test_engines.rs                      # Module file wiring engine sub-tests
  test_error.rs                        # GaError enum tests
  test_extension.rs                    # Extension module tests
  test_fitness.rs                      # Fitness module tests
  test_initializers.rs                 # Initializer tests
  test_niching.rs                      # Niching module tests
  test_observe.rs                      # Module file wiring observe sub-tests
  test_operations.rs                   # Module file wiring operator sub-tests
  test_population.rs                   # Population container tests
  test_rng.rs                          # RNG tests
  test_stats.rs                        # Statistics tests
  test_types.rs                        # Module file wiring type sub-tests
  test_validators.rs                   # Validator tests
  engines/
    test_ga.rs                         # Core Ga engine tests
    test_examples.rs                   # Example-based integration tests
    alps/test_alps.rs
    cellular/test_cellular.rs
    de/test_de.rs
    scatter/test_scatter.rs
    island/
      test_island.rs  test_island_configuration.rs
      test_island_migration.rs  test_island_nsga2.rs  test_island_topology.rs
    nsga2/
      test_crowding_distance.rs  test_non_dominated_sort.rs
      test_nsga2.rs  test_nsga2_configuration.rs  test_pareto.rs
  operations/
    test_selection.rs  test_selection_boltzmann.rs
    test_selection_rank.rs  test_selection_truncation.rs
    test_crossover.rs  test_crossover_arithmetic.rs  test_crossover_blend_alpha.rs
    test_crossover_clone.rs  test_crossover_order.rs  test_crossover_pmx.rs
    test_crossover_rejuvenate.rs  test_crossover_sbx.rs
    test_crossover_single_point.rs  test_crossover_uniform.rs
    test_mutation.rs  test_mutation_bit_flip.rs  test_mutation_creep_gaussian.rs
    test_mutation_dynamic.rs  test_mutation_insertion.rs  test_mutation_list_value.rs
    test_mutation_non_uniform.rs  test_mutation_polynomial.rs  test_mutation_range_value.rs
    test_survivor.rs  test_survivor_mu_comma_lambda.rs  test_survivor_mu_plus_lambda.rs
  observe/
    test_checkpoint.rs  test_serde.rs
    observer/
      test_observer.rs  test_composite_observer.rs  test_metrics_observer.rs
      test_tracing_observer.rs  test_sub_trait_observers.rs  test_observer_reexports.rs
    reporter/test_reporter.rs
    visualization/test_visualization.rs
  extension/
    test_extension.rs  test_extension_configuration.rs
  fitness/
    test_cache.rs  test_count_true.rs  test_fitness_fn_wrapper.rs
  initializers/test_initializers.rs
  niching/
    test_niching_configuration.rs  test_niching_distance.rs  test_niching_sharing.rs
  types/
    chromosomes/test_binary.rs  test_list.rs  test_range.rs
    genotypes/test_list.rs
  validators/
    test_generic_validator.rs  test_validator_factory.rs
```

**Shared fixtures:** Import the shared `Gene` and `Chromosome` types from `tests/structures.rs`
rather than redeclaring them. The module is exposed as `mod structures;` in the top-level module
files (`test_engines.rs`, `test_operations.rs`).

```rust
// Inside tests/engines/test_ga.rs
use crate::structures::{Chromosome, Gene};
```

**Feature-gated tests:** Wrap tests that require optional features with the appropriate
`#[cfg(feature = "...")]` attribute:

```rust
#[cfg(feature = "serde")]
#[test]
fn test_checkpoint_roundtrip() { … }
```

For files where every test requires a feature, apply the gate at the file level with
`#![cfg(feature = "...")]` on the first line (as used by `test_checkpoint.rs`,
`test_tracing_observer.rs`, `test_metrics_observer.rs`, and `test_visualization.rs`).

## Coverage Requirements

No numeric coverage thresholds are configured (no `.nycrc`, `tarpaulin.toml`, or coverage
section in `Cargo.toml`). Source files annotate benchmark helpers with
`#[cfg(not(tarpaulin_include))]` to exclude them from coverage measurement when
[cargo-tarpaulin](https://github.com/xd009642/tarpaulin) is used locally, but tarpaulin is
not a mandatory CI gate.

## CI Integration

Tests are executed in two separate GitHub Actions workflows on every pull request targeting
`main`:

**`rust-unit-tests.yml` — Unit Tests** (`build` job)
- Trigger: pull request to `main`
- Runner: `ubuntu-latest`
- Steps: `cargo build --verbose` then `cargo test --verbose`
- Note: runs with default features only; serde, observer, and visualization feature flags are
  not tested in CI automatically.

**`rust-clippy.yml` — Lint Check** (`clippy_check` job)
- Trigger: pull request (all branches)
- Runner: `ubuntu-latest`
- Steps: `cargo clippy --all-targets --all-features --message-format=json`, results uploaded
  as a SARIF report to the GitHub Security tab via `github/codeql-action/upload-sarif`.

PRs must pass both workflows before merging. Additionally, the project convention (see
`CLAUDE.md`) requires that the following commands all succeed locally before submitting a PR:

```bash
cargo test
cargo test --features serde
cargo clippy
cargo doc --no-deps
```
