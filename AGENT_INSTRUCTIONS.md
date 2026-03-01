# Instructions for Programming Agents

> This document defines the rules and conventions that **every AI agent** must follow
> when contributing code to this Rust genetic algorithms library.

---

## 1. Project Structure

```
src/
├── lib.rs                     # Entry point. Re-exports all public modules.
├── traits/                    # Core traits (GeneT, ChromosomeT, ConfigurationT)
│   ├── gene.rs
│   ├── chromosome.rs
│   └── configuration.rs
├── genotypes/                 # Gene implementations (Binary, Range<T>)
│   ├── binary.rs
│   └── range.rs
├── chromosomes/               # Chromosome implementations (Binary, Range<T>)
│   ├── binary.rs
│   └── range.rs
├── operations/                # Genetic operators
│   ├── selection/             # Parent selection (Random, RouletteWheel, SUS, Tournament)
│   ├── crossover/             # Crossover (Cycle, MultiPoint, Uniform)
│   ├── mutation/              # Mutation (Swap, Inversion, Scramble, Value)
│   └── survivor/              # Survivor selection (Fitness, Age)
├── initializers/              # Population initialization functions
│   ├── binary_initializer.rs
│   ├── range_initializer.rs
│   └── generic_initializer.rs
├── configuration.rs           # Configuration structs (GaConfiguration, LimitConfiguration, etc.)
├── population.rs              # Population management (parallel fitness, best_chromosome, AGA)
├── ga.rs                      # Main GA orchestrator (builder pattern, evolutionary cycle)
├── fitness/                   # Fitness functions and wrapper
│   ├── fitness_fn_wrapper.rs
│   └── count_true.rs
└── validators/                # Configuration validators
    ├── validator_factory.rs
    └── generic_validator.rs

tests/                         # Integration tests
├── structures.rs              # Test Gene and Chromosome (used by all tests)
├── test_ga.rs
├── test_operations.rs
├── test_chromosomes.rs
├── test_fitness.rs
├── test_initializers.rs
├── test_population.rs
├── chromosomes/
├── fitness/
└── operations/

benches/                       # Criterion benchmarks
examples/                      # Runnable examples (knapsack, n-queens)
```

### Main Structure Rule

- **Every new module must follow the existing directory pattern exactly.**
- If you add a new crossover operator, it goes in `src/operations/crossover/<name>.rs`.
- If you add a new gene type, it goes in `src/genotypes/<name>.rs`.
- If you add a new chromosome type, it goes in `src/chromosomes/<name>.rs`.
- Parent modules (`operations.rs`, `genotypes.rs`, `chromosomes.rs`, etc.) must be updated with the corresponding `pub mod` and `pub use`.

---

## 2. Code Conventions

### 2.1 Rust Edition and Minimum Version
- **Edition**: 2021
- **MSRV**: 1.81.0
- Do not use unstable features.

### 2.2 Style
- Follow standard `rustfmt`. Run `cargo fmt` before any commit.
- Run `cargo clippy` and resolve all warnings before finalizing.
- Document **everything** public with doc-comments (`///`), including `# Arguments`, `# Returns`, `# Panics`, and `# Examples` where applicable.
- Use `log::debug!`, `log::trace!`, `log::info!` for logging, following existing targets (`ga_events`, `selection_events`, `crossover_events`, `mutation_events`, `survivor_events`, `population_events`).

### 2.3 Traits and Generics
- New gene types **must** implement `GeneT` (defined in `src/traits/gene.rs`).
- New chromosome types **must** implement `ChromosomeT` (defined in `src/traits/chromosome.rs`).
- Use `Cow<[Gene]>` in `set_dna` to avoid unnecessary clones (established pattern).
- Use `Arc` for functions shared across threads (like `FitnessFnWrapper`).

### 2.4 Operator Enums
- Each new operator must add a variant to the corresponding enum in `src/operations.rs`:
  - `Selection`, `Crossover`, `Mutation`, `Survivor`
- Update the `factory` function of the corresponding module to dispatch the new variant.
- If the operator needs additional configuration parameters, add them to the appropriate configuration struct in `src/configuration.rs` (e.g., `CrossoverConfiguration`, `MutationConfiguration`).
- Add the corresponding builder method in `ConfigurationT` (trait) and in `GaConfiguration` (impl).

### 2.5 Error Handling
- Operator functions currently use `panic!` for validation errors.
- If migrating to `Result<T, GaError>`, follow that pattern in all new code.
- **Never** use `unwrap()` in new code without a comment justifying why it is safe.

### 2.6 Parallelism
- The project uses `std::thread` + `sync_channel` + `Arc<Mutex<>>` for parallelism.
- If migrating to `rayon`, use `par_iter`/`par_chunks` following the new pattern.
- Every structure shared across threads must be `Send + Sync`.

---

## 3. Tests: Mandatory Rules

### 3.1 Golden Rule
> **No change may break an existing test.**
> Before considering any modification complete, run `cargo test` and verify
> that **all** tests pass (currently 42 tests + 7 doc-tests).

### 3.2 Mandatory Coverage
- **All new code must have unit and/or integration tests.**
- Minimum tests required per type of change:

| Type of change | Required tests |
|---|---|
| New selection operator | ≥ 2 tests in `tests/operations/test_selection.rs` |
| New crossover operator | ≥ 2 tests in `tests/operations/test_crossover.rs` (or new file `test_crossover_<name>.rs`) |
| New mutation operator | ≥ 2 tests in `tests/operations/test_mutation.rs` (or new file) |
| New survivor operator | ≥ 2 tests in `tests/operations/test_survivor.rs` |
| New gene type | ≥ 3 tests in `tests/` (creation, get/set, integration with ChromosomeT) |
| New chromosome type | ≥ 4 tests in `tests/chromosomes/` (new, set\_dna, fitness, phenotype) |
| New initializer | ≥ 2 tests in `tests/test_initializers.rs` |
| Change in `ga.rs` | ≥ 1 integration test in `tests/test_ga.rs` |
| New module/feature | Own test file + full GA integration |

### 3.3 Test Structure
- Integration tests go in `tests/`.
- Use the test structures defined in `tests/structures.rs` (`Gene`, `Chromosome`) for generic tests.
- For concrete type tests (Binary, Range), use the types from `src/genotypes/` and `src/chromosomes/` directly.
- Name tests descriptively: `test_<module>_<expected_behavior>`.
- Each test function must be self-contained and not depend on execution order.

### 3.4 How to Write a Test

```rust
#[test]
fn test_<operator>_<case>() {
    // 1. ARRANGE: Create the input data
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }];
    let mut chromosome = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // 2. ACT: Execute the operation
    operation(&mut chromosome);

    // 3. ASSERT: Verify the result
    assert_eq!(chromosome.get_dna().len(), 3); // DNA size does not change
    assert_ne!(chromosome.get_dna(), &original_dna); // DNA was modified
}
```

### 3.5 Tests for Stochastic Operations
- For random operations (selection, mutation, crossover), verify **invariants**:
  - DNA size is preserved.
  - Resulting genes come from the parents (for crossover).
  - No out-of-range indices are produced.
- **Do not** verify exact values that depend on the RNG.
- If you need to test a specific path, consider using a fixed seed or mocking randomness.

### 3.6 Doc-Tests
- Every public type must have at least one example in its doc-comment that compiles and passes.
- Doc-tests are verified with `cargo test --doc`.

### 3.7 Post-Change Validation
Always run in this order:
```bash
cargo fmt --check         # Formatting
cargo clippy              # Lints
cargo test                # All tests
cargo test --doc          # Doc-tests
cargo bench --no-run      # Verify benchmarks compile
```

---

## 4. Benchmarks

- Benchmarks use **Criterion** and are located in `benches/`.
- If you add a new operator, add at least **one benchmark** in the corresponding file
  (`benches/crossover.rs`, `benches/mutation.rs`, `benches/selection.rs`, `benches/survivor.rs`).
- Use existing benchmark groups with sizes `genes_10`, `genes_100`, `genes_1000`.
- **Do not** break existing benchmarks.

---

## 5. Examples

- Runnable examples go in `examples/`.
- If you add significant functionality, consider adding or updating an example.
- Examples must be self-contained and demonstrate end-to-end usage with `Ga::new()`.

---

## 6. Dependencies

- Current dependencies: `rand`, `num_cpus`, `log`, `env_logger`.
- Dev-dependencies: `criterion`, `pprof`.
- **Do not** add dependencies without justification. Prefer the standard library when possible.
- If a new dependency is optional, use **feature flags** in `Cargo.toml`.

---

## 7. Workflow for Adding a New Operator

Example: adding a new crossover method called "OrderCrossover":

1. **Create the file**: `src/operations/crossover/order.rs`
2. **Implement the function**:
   ```rust
   pub fn order<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Option<Vec<U>> { ... }
   ```
3. **Register in the module**: Add `pub mod order;` and `pub use self::order::order;` in `src/operations/crossover.rs`
4. **Add enum variant**: `Order` in `src/operations.rs` → enum `Crossover`
5. **Update factory**: Add `Crossover::Order => { order(parent_1, parent_2) }` in `src/operations/crossover.rs`
6. **Update configuration** (if parameters needed): Add fields in `CrossoverConfiguration` and builder methods
7. **Write tests**: Minimum 2 tests in `tests/operations/test_crossover.rs`
8. **Write benchmark**: Add group in `benches/crossover.rs`
9. **Run full validation**: `cargo fmt && cargo clippy && cargo test && cargo bench --no-run`

---

## 8. Workflow for Adding a New Gene/Chromosome Type

1. **Create genotype**: `src/genotypes/<name>.rs` implementing `GeneT`
2. **Create chromosome**: `src/chromosomes/<name>.rs` implementing `ChromosomeT`
3. **Register modules**: Update `src/genotypes.rs` and `src/chromosomes.rs`
4. **Create initializer** (if applicable): `src/initializers/<name>_initializer.rs`
5. **Write tests**: Minimum 3 tests for the genotype, 4 for the chromosome, 2 for the initializer
6. **Create example**: `examples/<use_case>.rs`
7. **Validate**: `cargo fmt && cargo clippy && cargo test`

---

## 9. Final Checklist

Before considering any change complete, verify:

- [ ] `cargo fmt --check` passes without errors
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo test` → **all** tests pass (0 failures)
- [ ] `cargo test --doc` → all doc-tests pass
- [ ] `cargo bench --no-run` → benchmarks compile
- [ ] New code has doc-comments (`///`)
- [ ] New code has tests (see coverage table §3.2)
- [ ] Enums updated if an operator was added
- [ ] Factory updated if an operator was added
- [ ] Parent modules updated (`pub mod` + `pub use`)
- [ ] No unnecessary dependencies were added

