# Programming Instructions for Agents

> Mandatory guide for any agent programming in this project.

---

## 1. Project Structure

```
src/
├── lib.rs                    # Entry point, re-exports all public modules
├── error.rs                  # GaError enum — every error returned as Result
├── ga.rs                     # Main orchestrator (Ga<U>)
├── population.rs             # Population management
├── configuration.rs          # Configuration structs (GaConfiguration, etc.)
├── traits/                   # Base traits (GeneT, ChromosomeT, ConfigurationT)
│   ├── gene.rs
│   ├── chromosome.rs
│   └── configuration.rs
├── genotypes/                # Concrete GeneT implementations
│   ├── binary.rs
│   └── range.rs
├── chromosomes/              # Concrete ChromosomeT implementations
│   ├── binary.rs
│   └── range.rs
├── fitness/                  # Fitness helpers (FitnessFnWrapper, count_true)
│   ├── count_true.rs
│   └── fitness_fn_wrapper.rs
├── initializers/             # DNA initialization functions
│   ├── binary_initializer.rs
│   ├── generic_initializer.rs
│   └── range_initializer.rs
├── operations/               # Genetic operators
│   ├── crossover/            # Cycle, Multipoint, Uniform
│   ├── mutation/             # Swap, Inversion, Scramble, Value
│   ├── selection/            # Random, FitnessProportionate, Tournament
│   └── survivor/             # Age, Fitness
└── validators/               # Configuration validators
    ├── generic_validator.rs
    └── validator_factory.rs
```

**Rule:** Each new operator goes in its own file within the corresponding subdirectory (`crossover/`, `mutation/`, etc.), is re-exported from the parent `mod.rs`, and a variant is added to the corresponding enum in `operations.rs`.

---

## 2. Code Conventions

### 2.1 Error Handling
- **NEVER** use `panic!` in library code. Use `Result<T, GaError>`.
- Operators (crossover, mutation, etc.) return `Result`.
- Tests use `.unwrap()` on `Result` values.
- Initializers keep `expect()` because they are behind `Fn() -> Vec<Gene>` closures.

### 2.2 Parallelism
- Use **rayon** (`par_iter`, `into_par_iter`) for all parallel operations.
- **DO NOT** use `std::thread::spawn`, `Arc<Mutex<>>`, or `sync_channel` manually.
- The `number_of_threads` parameter in the configuration is kept for backward compatibility but rayon manages the pool automatically.

### 2.3 Data Types
- Parent pairs from selection are `Vec<(usize, usize)>`, **not** `HashMap<usize, usize>`.
- DNA is passed with `Cow<[Gene]>` to avoid unnecessary copies.
- For value mutation on `Range<T>`, implement the `ValueMutable` trait.

### 2.4 Traits and Generics
- Every operator must be generic over `U: ChromosomeT`.
- For operations that need `Send + Sync + 'static + Clone`, add those bounds.
- New traits go in `src/traits/` with their own file.

### 2.5 Style
- Document public functions with `///` doc-comments.
- Use `log` crate (`debug!`, `trace!`, `info!`) with descriptive targets.
- Imports: `use crate::...` for internal modules, `use xxx::prelude::*` only for rayon.

---

## 3. Tests

### 3.1 Test Structure
```
tests/
├── structures.rs             # Shared test Gene and Chromosome
├── test_chromosomes.rs       # Chromosome tests (mod that includes subdirectory)
├── test_fitness.rs           # Fitness tests
├── test_ga.rs                # GA orchestrator tests
├── test_initializers.rs      # Initializer tests
├── test_operations.rs        # Operation tests (mod that includes subdirectory)
├── test_population.rs        # Population tests
├── chromosomes/              # Unit tests for chromosomes
├── fitness/                  # Unit tests for fitness
└── operations/               # Unit tests for each operator
```

### 3.2 Test Rules
1. **Every change must have tests.** No exceptions.
2. **No existing test may break.** Run `cargo test` before each commit.
3. Stochastic tests (mutation, selection) use **retry loops** to avoid flakiness:
   ```rust
   let mut changed = false;
   for _ in 0..10 {
       // ... stochastic operation ...
       if result_changed { changed = true; break; }
   }
   assert!(changed, "descriptive message");
   ```
4. New operator tests go in `tests/operations/test_<operator>.rs` and are registered in `tests/test_operations.rs`.
5. New chromosome tests go in `tests/chromosomes/test_<type>.rs` and are registered in `tests/test_chromosomes.rs`.

### 3.3 Running Tests
```bash
cargo test              # All tests
cargo test test_ga      # Only GA tests
cargo test -- --nocapture  # With visible output
```

---

## 4. Adding a New Operator (Example: new crossover)

1. Create `src/operations/crossover/my_crossover.rs`
2. Implement: `pub fn my_crossover<U: ChromosomeT>(...) -> Result<Vec<U>, GaError>`
3. Add variant `MyCrossover` to the `Crossover` enum in `src/operations.rs`
4. Register in the factory: `src/operations/crossover.rs` → `match` arm
5. Re-export: `pub use self::my_crossover::my_crossover;` in `src/operations/crossover.rs`
6. Create test: `tests/operations/test_crossover_my.rs`
7. Register test in `tests/test_operations.rs`: `mod test_crossover_my;`
8. Run `cargo test` — everything must pass

---

## 5. Dependencies

| Crate | Version | Usage |
|-------|---------|-------|
| `rand` | 0.9 | Random number generation |
| `rayon` | 1.10 | Parallelism (par_iter) |
| `log` | 0.4 | Structured logging |
| `env_logger` | 0.11 | Logging backend |

**Dev-dependencies:** `criterion` (benchmarks), `pprof` (profiling).

---

## 6. Pre-Commit Checklist

- [ ] `cargo test` — all tests pass
- [ ] `cargo build` — no warnings
- [ ] New code has unit tests
- [ ] Public functions have doc-comments
- [ ] No `panic!` in library code (only `Result<T, GaError>`)
- [ ] No manual `thread::spawn` (use rayon)

