# Testing Patterns

**Focus:** Test framework, structure, mocking, coverage
**Analyzed:** 2026-03-20

## Framework

- **Unit/integration tests:** Built-in Rust `#[test]` + `cargo test`
- **Benchmarks:** `criterion` crate (`cargo bench`)
- **No mocking framework** — tests use real implementations

## Test Structure

### File Organization
```
tests/
├── test_ga.rs                  # Core GA orchestration
├── test_operations.rs          # Operator module (re-exports sub-tests)
│   operations/
│   ├── test_crossover.rs
│   ├── test_crossover_clone.rs
│   ├── test_crossover_single_point.rs
│   ├── test_crossover_uniform.rs
│   ├── test_crossover_order.rs
│   ├── test_crossover_pmx.rs
│   ├── test_crossover_arithmetic.rs
│   ├── test_crossover_blend_alpha.rs
│   ├── test_crossover_sbx.rs
│   ├── test_mutation.rs
│   ├── test_mutation_bit_flip.rs
│   ├── test_mutation_creep_gaussian.rs
│   ├── test_mutation_dynamic.rs
│   ├── test_mutation_insertion.rs
│   ├── test_mutation_non_uniform.rs
│   ├── test_mutation_polynomial.rs
│   ├── test_mutation_range_value.rs
│   ├── test_selection.rs
│   ├── test_selection_boltzmann.rs
│   ├── test_selection_rank.rs
│   ├── test_selection_truncation.rs
│   ├── test_survivor.rs
│   ├── test_survivor_mu_comma_lambda.rs
│   └── test_survivor_mu_plus_lambda.rs
├── test_chromosomes.rs         # Built-in chromosome types
├── test_island.rs              # Island model
├── test_nsga2.rs               # NSGA-II multi-objective
├── test_niching.rs             # Fitness sharing
├── test_fitness.rs             # Fitness function wrapper
├── test_fitness_cache.rs       # LRU fitness cache
├── test_stats.rs               # Statistics collection
├── test_validators.rs          # Configuration validation
├── test_population.rs          # Population container
├── test_rng.rs                 # RNG seeding
├── test_checkpoint.rs          # Checkpoint save/load
├── test_serde.rs               # Serde feature (feature-gated)
├── test_error.rs               # Error types
├── test_extension.rs           # Extension operators
├── test_extension_configuration.rs
└── test_initializers.rs        # Population initialization
```

### In-file Unit Tests
Small, focused unit tests live inline in source files:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_something() { ... }
}
```

## Testing Patterns

### Test Helper Chromosomes
Tests define minimal `ChromosomeT` implementations inline or in `tests/structures.rs`:
```rust
struct TestChromosome { dna: Vec<BinaryGene>, fitness: f64 }
impl ChromosomeT for TestChromosome { ... }
```

### Error Testing
```rust
assert!(result.is_err());
assert_eq!(result.unwrap_err(), GaError::CrossoverError("...".to_string()));
```

### Deterministic Testing
Use seeded RNG for reproducible results:
```rust
let config = GaConfiguration::default().with_seed(42);
```

### Feature-Gated Tests
Serde/checkpoint tests run with `cargo test --features serde`:
```rust
#[cfg(feature = "serde")]
#[test]
fn test_checkpoint_roundtrip() { ... }
```

## CI Requirements

All PRs must pass:
1. `cargo test` — all tests
2. `cargo test --features serde` — including serde tests
3. `cargo clippy` — zero lint warnings
4. `cargo doc --no-deps` — zero rustdoc warnings

## Coverage Gaps

- Island model migration integration tests sparse
- NSGA-II crowding distance with degenerate Pareto fronts
- Checkpoint/restore round-trip not tested for all operator combinations
- Edge cases: empty chromosomes, single-gene chromosomes in permutation crossovers

---
*Mapped: 2026-03-20*
