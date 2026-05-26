# genetic_algorithms — Claude Code Project Guide

## Project Overview

Rust library for evolutionary computation: single-objective (`Ga<U>`), multi-objective (NSGA-II/III, MOEA/D, SPEA2, SMS-EMOA, IBEA), island model, Differential Evolution, Scatter Search, Cellular GA, ALPS, and Genetic Programming (`GpGa<N>`). Generic over chromosome/gene types via traits. Published on crates.io as `genetic_algorithms`.

## Architecture

### Core Abstractions

- **`ChromosomeT`** (`src/traits/chromosome.rs`) — Core trait. DNA as `&[Self::Gene]`, scalar fitness (`f64`), age tracking. Uses `Cow<[Gene]>` for zero-copy DNA operations.
- **`TreeChromosome`** (`src/engines/gp/chromosome.rs`) — Supertrait of `ChromosomeT` for tree chromosomes. Provides `tree()`, `tree_mut()`, `depth()`, `node_count()`. Implemented by `GpChromosome<N>`.
- **`GeneT`** (`src/traits/gene.rs`) — Minimal gene trait: `id() -> i32`, `new()`, `default()`.
- **Operator traits** (`src/traits/operators.rs`):
  - `SelectionOperator::select(&[U], couples, threads) -> Vec<(usize, usize)>`
  - `CrossoverOperator::crossover(&U, &U) -> Result<Vec<U>, GaError>`
  - `MutationOperator::mutate(&mut U, step, sigma) -> Result<(), GaError>`
  - `SurvivorOperator::select_survivors(&mut Vec<U>, pop_size, limits) -> Result<(), GaError>`
  - `ExtensionOperator::apply_extension(&mut Vec<U>, pop_size, problem, config) -> Result<(), GaError>`
- **`ConfigurationT`** (`src/traits/configuration.rs`) — Fluent builder pattern combining all sub-config traits.

### Operator Dispatch

Operators use **enum + factory function** pattern (runtime dispatch):
- `Selection` enum → `selection::factory()`
- `Crossover` enum → `crossover::factory()`
- `Mutation` enum → `mutation::factory_with_params()`
- `Survivor` enum → `survivor::factory()`
- `Extension` enum → `extension::factory()`

### Execution Flow (`src/ga.rs`)

Per-generation cycle:
1. Selection → parent pairs
2. Crossover + Mutation (parallel via rayon) → offspring
3. Population merge (parents + offspring)
4. Elitism (extract elite → survivor selection → reinsert elite)
5. Adaptive GA parameter updates (if enabled)
6. Extension strategy (if diversity < threshold)
7. Niching / fitness sharing (if enabled)
8. Best chromosome update
9. Statistics collection
10. Checkpointing (if enabled)
11. Callback invocation (if configured)
12. Stopping criteria check

### Module Map

| Module | Purpose |
|--------|---------|
| `src/engines/ga.rs` | Single-population GA orchestrator |
| `src/engines/island/` | Island model (multi-population + migration) |
| `src/engines/nsga2/` | NSGA-II multi-objective optimization |
| `src/engines/gp/` | Genetic Programming engine (`GpGa<N>`, `GpChromosome<N>`, `GpNode`, operators) |
| `src/operations/` | All operators (selection, crossover, mutation, survivor, extension) |
| `src/operations/crossover/variable_length.rs` | Variable-length crossover with AlignmentStrategy |
| `src/operations/mutation/length_mutation.rs` | `Insertion` / `Deletion` variable-length mutations |
| `src/operations/survivor/parsimony.rs` | Parsimony pressure fitness adjustment |
| `src/types/chromosomes/` | Built-in chromosome types (Binary, Range<T>, ChromosomeLength) |
| `src/types/genotypes/` | Built-in gene types (Binary, Range<T>) |
| `src/traits/` | Core trait definitions |
| `src/configuration.rs` | All configuration structs |
| `src/population.rs` | Population container |
| `src/niching/` | Fitness sharing |
| `src/extension/` | Extension configuration |
| `src/stats.rs` | Per-generation statistics |
| `src/observe/checkpoint.rs` | Serialization (serde feature) |
| `src/fitness/` | Fitness function wrapper |
| `src/rng.rs` | RNG with optional seeding |
| `src/validators/` | Configuration validation |
| `src/error.rs` | GaError enum |

## Development Workflow

### Branching Strategy

**CRITICAL**: Never branch from `main` directly for features/fixes.

```
main
 └── milestone/<milestone-name>     ← create from main
      ├── feat/<issue-number>       ← create from milestone branch
      └── fix/<issue-number>        ← create from milestone branch
```

1. Check if the milestone branch exists. If not, create it from `main`.
2. **CRITICAL — Cargo version bump**: The very first commit on every new milestone branch must update the `version` field in `Cargo.toml` to match the milestone's version. No phase work is merged before this bump exists.
3. Create `feat/<issue>` or `fix/<issue>` from the milestone branch.
4. PR targets the milestone branch, not `main`.

### GitHub Authentication

The `GITHUB_TOKEN` env var may be invalid. Always use:
```bash
GITHUB_TOKEN= gh <command>
```
This forces `gh` to use the keyring-stored `leimbernon` credentials.

### Testing

```bash
cargo test                        # All tests
cargo test --features serde       # Including serde tests
cargo clippy                      # Lint check
cargo doc --no-deps               # Doc generation
cargo bench                       # Benchmarks (criterion)
```

All PRs must pass: `cargo test`, `cargo test --features serde`, `cargo clippy`, zero rustdoc warnings.

### Feature Flags

| Flag | Dependencies | Purpose |
|------|-------------|---------|
| `serde` | serde, serde_json | Checkpoint serialization |

Future flags planned: `observer-tracing`, `observer-metrics`, `benchmarks`.

## Tool Use Discipline

Do not re-read files already read in the current session unless explicitly asked to. Work from context already loaded — minimize tool calls.

## Important Constraints

### No Breaking Changes (default policy)

Prefer non-breaking additions:
- New enum variants, new builder methods, new traits, new modules
- `Option<T>` fields with `None` default
- Feature flags for optional dependencies

When breaking changes are unavoidable, use parallel traits/wrappers (see issues labeled `breaking-change`).

### WASM Compatibility (mandatory)

Every new feature **must compile for `wasm32-unknown-unknown`**. This target has no threads and no `std::time`. Apply these rules at implementation time, not as a retrofit:

- **`std::time::Instant` / `SystemTime`** — never call `.now()` or `.elapsed()` unconditionally. Gate the call: `#[cfg(not(target_arch = "wasm32"))]`. The type annotation (`Option<Instant>`) can remain un-gated; only the instantiation must be gated.
- **`rayon` parallelism** — never call `.par_iter()` unconditionally. Duplicate only the iterator expression behind cfg gates; keep the closure body shared:
  ```rust
  #[cfg(not(target_arch = "wasm32"))]
  let results: Vec<_> = items.par_iter().map(|x| process(x)).collect();
  #[cfg(target_arch = "wasm32")]
  let results: Vec<_> = items.iter().map(|x| process(x)).collect();
  ```
- **Verify during implementation** — run `cargo check --target wasm32-unknown-unknown` before considering a feature complete. CI enforces this via `.github/workflows/wasm-check.yml`, but catching it locally is cheaper.
- **New dependencies** — check that any new crate either supports `wasm32-unknown-unknown` or is gated behind a non-wasm cfg.

### Performance Awareness

Known performance patterns to maintain:
- Parallel fitness evaluation via rayon
- `Cow<[Gene]>` for zero-copy DNA in `set_dna()`
- In-place mutation via `dna_mut()` / `set_gene()` (prefer over `dna().to_vec()`)
- `select_nth_unstable_by()` over full sort when finding top-k
- Pre-allocate Vecs with `with_capacity()` in hot paths

### Observability (critical initiative)

All changes to the GA execution flow must preserve observability hooks. The `GaObserver` trait (issue #182) is the foundation — never remove or bypass observer notification points. This initiative must survive all refactors.

## Active Milestones

| Milestone | Focus | Key Issues |
|-----------|-------|------------|
| Improve usability | Current active milestone | Backlog features (#166-#179) |
| Observability & Traceability | GaObserver trait system | #182-#186 |
| Performance Optimizations | Reduce clones, improve algorithms | #187-#192 |
| New Operators | Selection, crossover, mutation additions | #196-#202 |
| Advanced Multi-Objective | NSGA-III, MOEA/D, SPEA2 | #203-#207 |
| Alt. Metaheuristics & Pop. Models | DE, Scatter Search, Cellular GA, ALPS | #208-#211 |
| Framework Extensions | Constraints, memetic, warm start, AOS, benchmarks | #212-#219 |
| Advanced Representations (Breaking) | Lexicase, multi-parent, GP, variable-length | #220-#224 |

## Code Style

- Follow existing patterns: enum + factory for new operators
- Use `target` in log macros: `info!(target="ga_events", ...)`
- Operator implementations go in their own file under `src/operations/<type>/`
- Configuration structs in `src/configuration.rs` (or `<module>/configuration.rs` for sub-modules)
- Builder methods via traits in `src/traits/configuration.rs`
- Errors as `GaError` enum variants
- Tests in the same file (`#[cfg(test)] mod tests`) or in `tests/`
