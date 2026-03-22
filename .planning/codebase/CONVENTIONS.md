# Code Conventions

**Focus:** Code style, naming, patterns, error handling
**Analyzed:** 2026-03-20

## Language & Runtime

- **Language:** Rust (edition 2021)
- **MSRV:** Stable Rust (no nightly features)
- **Formatting:** `rustfmt` (standard Rust formatting)
- **Linting:** `clippy` with no warnings allowed in CI

## Naming Conventions

| Construct | Convention | Example |
|-----------|-----------|---------|
| Types/Traits | PascalCase | `ChromosomeT`, `GaError`, `SelectionOperator` |
| Functions/methods | snake_case | `crossover()`, `set_dna()`, `factory_with_params()` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_POPULATION_SIZE` |
| Modules | snake_case | `operations`, `crossover`, `selection` |
| Trait suffixes | `T` suffix for domain traits | `ChromosomeT`, `GeneT`, `ConfigurationT` |
| Operator traits | `*Operator` suffix | `CrossoverOperator`, `SelectionOperator` |
| Factory functions | `factory()` / `factory_with_params()` | consistent across all operators |

## Module Organization

- Each operator type has its own directory: `src/operations/<type>/`
- Operator implementations go in their own file: `src/operations/crossover/clone.rs`
- Public re-exports in `src/operations/<type>.rs` (the module file)
- Sub-modules (island, nsga2) have their own `configuration.rs`
- `src/configuration.rs` holds main GA configuration structs

## Code Patterns

### Enum + Factory Dispatch
All operators use runtime dispatch via enum + factory pattern:
```rust
// 1. Enum variant in e.g. src/operations/crossover.rs
pub enum Crossover { Clone, SinglePoint, Uniform, ... }

// 2. Factory function
pub fn factory<U: ChromosomeT>(crossover: &Crossover) -> Box<dyn CrossoverOperator<U>>

// 3. Trait impl in dedicated file
impl<U: ChromosomeT> CrossoverOperator<U> for CloneCrossover { ... }
```

### Builder / Fluent Config
Configuration uses fluent builder methods via traits:
```rust
ga.with_population_size(100)
  .with_crossover(Crossover::SinglePoint)
  .with_mutation(Mutation::BitFlip)
```

### Error Handling
- All fallible operations return `Result<T, GaError>`
- Error variants carry descriptive `String` messages
- Use `?` operator throughout; no `unwrap()` in library code (only tests/examples)
- `GaError` is `Clone + PartialEq` for testing

### Zero-Copy DNA
- `Cow<[Gene]>` used in `set_dna()` for zero-copy DNA operations
- In-place mutation via `dna_mut()` / `set_gene()` preferred over `dna().to_vec()`

### Logging
- Use `log` crate with structured targets:
  ```rust
  info!(target="ga_events", ...);
  debug!(target="crossover_events", method="clone"; "...");
  ```
- Target naming: `<module>_events` pattern

### Parallelism
- `rayon` for parallel fitness evaluation and crossover
- Thread safety via `Send + Sync` bounds on chromosome types

## Documentation

- All public items have rustdoc comments (`///`)
- Module-level docs use `//!`
- Doc examples in `# Examples` sections where applicable
- `cargo doc --no-deps` must produce zero warnings

## Feature Flags

- `serde` feature gates checkpoint serialization
- Feature-gated items use `#[cfg(feature = "serde")]`
- Derive macros behind feature: `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`

---
*Mapped: 2026-03-20*
