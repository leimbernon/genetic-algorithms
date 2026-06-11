# Architecture

**Analysis Date:** 2026-03-20

## Pattern Overview

**Overall:** Generic trait-driven genetic algorithm library with pluggable operators and runtime dispatch.

**Key Characteristics:**
- Trait-based abstractions over chromosome and gene types enable zero-cost specialization for any DNA representation
- Enum + factory-function pattern for all genetic operators (selection, crossover, mutation, survivor, extension)
- Parallel fitness evaluation via rayon with optional LRU caching for redundant DNA
- Three orchestration modes: single-population (`Ga`), multi-population island model (`IslandGa`), and multi-objective (`Nsga2Ga`)
- Builder pattern for fluent configuration through trait composition

## Layers

**Abstraction Layer (Traits):** `src/traits/` - Core traits: `ChromosomeT`, `GeneT`, operator traits, configuration traits
**Chromosome & Gene Layer:** `src/chromosomes/` and `src/genotypes/` - Binary and Range<T> implementations
**Operator Layer:** `src/operations/` - Selection, crossover, mutation, survivor, and extension implementations
**Population & Statistics Layer:** `src/population.rs`, `src/stats.rs` - Population management and per-generation metrics
**Fitness Layer:** `src/fitness/` - Fitness function wrappers, LRU caching, utility functions
**Initialization Layer:** `src/initializers/` - Default initializers for various chromosome types
**Configuration Layer:** `src/configuration.rs` - Aggregated GA configuration structures
**Orchestration Layer:** `src/ga.rs`, `src/island/`, `src/nsga2/` - Main GA orchestrators
**Utility Layers:** Error handling (`src/error.rs`), validation (`src/validators/`), RNG (`src/rng.rs`), checkpointing, niching, extensions

## Data Flow

**Initialization:** User creates `Ga::new()` → chains builder methods → `.build()` validates and caches fitness → `.run()` triggers lazy initialization with parallel fitness evaluation

**Per-Generation Cycle:** Selection → Crossover/Mutation (parallel) → Population merge → Elitism → Survivor selection → Fitness calculation → Adaptive GA adjustments → Extension strategy → Niching → Best update → Statistics → Checkpointing → Callback → Stopping check

**Island Model:** N independent populations evolve in parallel with periodic migration between neighbors based on MigrationTopology

**NSGA-II:** Multi-objective with non-dominated sorting and crowding distance for Pareto front maintenance

## Key Abstractions

**ChromosomeT:** DNA container with `Cow<[Gene]>` for zero-copy operations, implements `dna()`, `dna_mut()`, `set_dna()`
**GeneT:** Minimal trait with `id()`, `set_id()`, and required bounds `Default + Clone + Sync + Send`
**SelectionOperator:** `select(&[U], couples, threads) -> Vec<(usize, usize)>` - parent pair selection
**CrossoverOperator:** `crossover(&U, &U) -> Result<Vec<U>, GaError>` - genetic recombination
**MutationOperator:** `mutate(&mut U, step, sigma) -> Result<(), GaError>` - perturbation in-place
**SurvivorOperator:** `select_survivors(&mut Vec<U>, size, limits) -> Result<(), GaError>` - population pruning
**ExtensionOperator:** `apply_extension(&mut Vec<U>, size, problem, config) -> Result<(), GaError>` - diversity rescue
**InitializationFn & FitnessFn:** User closures wrapped in Arc for thread-safe sharing

## Entry Points

**Single-Objective:** `Ga<U>::new()` → builder chain → `.run()` in `src/ga.rs`
**Island Model:** `IslandGa<U>::new()` → `.run()` in `src/island/mod.rs`
**Multi-Objective:** `Nsga2Ga<U>::new()` → `.with_objective_fns()` → `.run()` in `src/nsga2/mod.rs`

## Error Handling

All operations return `Result<T, GaError>` - single enum covering ConfigurationError, ValidationError, CrossoverError, MutationError, InitializationError, SelectionError, InvalidIslandConfiguration, InvalidNsga2Configuration, InvalidNichingConfiguration, MigrationError, CheckpointError

## Cross-Cutting Concerns

**Logging:** `log` crate with `LogLevel` configuration, uses `info!(target="ga_events", ...)`
**Validation:** Centralized in `src/validators/` factory function checking population, operators, alleles, stopping criteria
**Parallelization:** rayon `.par_iter()` for fitness, offspring, and island evolution
**Observability:** Callback system in `run_with_callback()` with early termination via `ControlFlow::Break()`

---

*Architecture analysis: 2026-03-20*
