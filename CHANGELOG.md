# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`parallel` feature (default-on)**. Disable via `default-features = false` to shed rayon + crossbeam dependencies for embedded / wasm-only builds. All rayon call-sites are gated under `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`; sequential fallbacks are provided under `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]`. Phase 69 Action #3.

### Changed

- Bench harness migrated from criterion to divan (Phase 69 Action #7). All 13 bench files (`selection`, `crossover`, `mutation`, `survivor`, `ga_run`, `nsga2`, `island_ga`, `de`, `scatter`, `alps`, `cellular`, `rastrigin`, `metrics_observer`) ported to divan 0.1.21. criterion removed from `[dev-dependencies]`. No public-API change. (Phase 69 / Plan 69-01)
- `src/engines/ga.rs` (3342 lines) split into 11 submodules under `src/engines/ga/`: `mod.rs` (orchestrator), `lifecycle.rs`, `generation.rs`, `adaptive.rs`, `aos.rs`, `extension.rs`, `cache.rs`, `batch.rs`, `stats.rs`, `observer.rs`, `stopping.rs`. No API change; zero semantic change verified via cargo expand symbol diff and 1661 passing tests. Phase 69 Action #4. See `.planning/intel/ga-internals.md` for per-submodule responsibilities and visibility rules.

### Removed

---

## [2.4.0] - 2026-05-18

### Added
- **Standard benchmark functions** (`benchmarks` feature): comprehensive suite for evaluating optimization algorithms. Single-objective: `Ackley`, `Rastrigin`, `Sphere`. Bi-objective ZDT suite: `ZDT1`–`ZDT6`. Many-objective DTLZ suite: `DTLZ1`–`DTLZ7`. All functions implement the `BenchmarkFn` trait with unified `evaluate(&[f64]) -> Vec<f64>` interface, metadata bounds, and known optimum values. Gated behind the `benchmarks` feature flag — no impact on default builds.
- **Memetic algorithm framework** (`LocalSearch` operator): hill-climbing local search refinement that improves individual solutions after mutation. Configurable via `LocalSearchConfiguration` with `HillClimbingConfig` (step_size, max_iterations). Supports `Lamarckian` (genotype updated) and `Baldwinian` (fitness only) modes. Application strategies: `AllOffspring`, `BestN`, `Probabilistic`, `EveryNGenerations`. Integrated into the `Ga` generation loop via `.with_local_search()` builder method. New `LocalSearchOperator` trait for custom implementations.
- New example: `memetic_rastrigin.rs` demonstrating local search on the Rastrigin benchmark.
- **Comprehensive documentation refactor**: all 11 engines documented to "ficha técnica" standard with Description, When to Use, parameter tables, compilable examples, and cross-references. All 19 example files have inline doc comments (`/*!` / `//!` blocks). Crate-level `src/lib.rs` expanded to 242 `//!` lines covering all engines, feature flags, and decision guidance. 17 new `docs/` guide files (per-engine guides + concept guides). `docs/examples.md` and `docs/engines.md` rewritten with current API. Zero `cargo doc --no-deps` warnings.

---

## [2.3.0] - 2026-04-27

### Added
- **`GaObserver<U>` trait** (`observer` module): telemetry-agnostic observability hook with 12 lifecycle callbacks covering generation start/end, operator results (selection, crossover, mutation, survivor), extension events, new-best events, and run start/finish. All methods have default no-op implementations for forward compatibility.
- **`ExtensionEvent` struct**: carries the extension strategy name, generation number, and diversity value at the moment an extension was triggered — passed to `on_extension`.
- **`NoopObserver`**: zero-cost `GaObserver` implementation for when no observer is needed.
- **`LogObserver`**: drop-in replacement for the previous hardcoded `log!()` calls. Implements all 12 `GaObserver` hooks (and both sub-trait extensions) with identical log output. Zero behavior change for existing setups.
- **`TracingObserver`** (`observer-tracing` feature): structured OpenTelemetry-compatible spans and events via the `tracing` crate. Each generation opens a span; operator results and special events become child spans/events. Gated behind the `observer-tracing` feature flag — no impact on default builds.
- **`IslandGaObserver<U>` sub-trait**: extends `GaObserver<U>` with Island-GA-specific hooks (`on_migration`, `on_island_generation_complete`). `LogObserver` implements this trait.
- **`Nsga2Observer<U>` sub-trait**: extends `GaObserver<U>` with NSGA-II-specific hooks (`on_front_assigned`, `on_crowding_distance_computed`). `LogObserver` implements this trait.
- **`AllObserver<U>` supertrait**: blanket supertrait combining `GaObserver<U>`, `IslandGaObserver<U>`, and `Nsga2Observer<U>` for observers that span all GA modes.
- **`CompositeObserver<U>`**: fan-out observer that forwards every hook call to a list of `Arc<dyn AllObserver<U>>` observers. Constructed with a fluent `add()` builder. Enables combining `LogObserver` + `TracingObserver` + `MetricsObserver` without custom glue code.
- **`MetricsObserver`** (`observer-metrics` feature): records per-generation gauges (`best_fitness`, `worst_fitness`, `avg_fitness`, `diversity`), histograms (crossover offspring count, mutation events), and counters (new-best events, extension triggers) via the `metrics` crate facade. Gated behind the `observer-metrics` feature flag — compatible with any `metrics`-compatible backend (Prometheus, StatsD, etc.).
- `Extension::as_str()` method: human-readable name for each extension strategy, used by `LogObserver` and `MetricsObserver`.
- Re-exports: `GaObserver`, `AllObserver`, `CompositeObserver`, `NoopObserver`, `LogObserver` available from the crate root.

### Changed
- `Ga<U>` now accepts an optional `Arc<dyn GaObserver<U>>` via `.with_observer(obs)`. All 12 hook call sites are wired into the execution loop; zero overhead when no observer is set (`Option::None` branch, no allocations).
- `IslandGa<U>` wired with `IslandGaObserver<U>` hooks at migration and per-island generation boundaries.
- `Nsga2Ga<U>` wired with `Nsga2Observer<U>` hooks at front-assignment and crowding-distance steps.
- `log!()` calls removed from `ga.rs` execution paths — all logging now routes through `LogObserver`.

### Deprecated
- `Reporter<U>` trait and `.with_reporter()` builder: superseded by `GaObserver<U>` + `LogObserver`. `Reporter<U>` remains functional for this release; it will be removed in a future version.

---

## [2.2.0] - 2026-03-22

### Added
- **Extension strategies for population diversity control**: optional diversity-rescue mechanisms that trigger when fitness standard deviation drops below a configurable threshold. Four strategies: `MassExtinction`, `MassGenesis`, `MassDegeneration`, `MassDeduplication`.
- `ExtensionOperator` trait for custom implementations.
- Builder methods: `with_extension_method()`, `with_extension_diversity_threshold()`, `with_extension_survival_rate()`, `with_extension_mutation_rounds()`, `with_extension_elite_count()`.
- **Population diversity metric**: `GenerationStats` now exposes a `diversity` field (fitness standard deviation) updated every generation. Extension strategies and adaptive mutation use it internally; users can read it in callbacks and reporters.
- **List genotype**: `genotypes::List<T>` and `chromosomes::ListChromosome<T>` for problems over finite symbolic alphabets (colors, directions, categories, etc.). Works with all existing operators. Includes `list_random_initialization` and `list_random_initialization_without_repetitions` initializers, plus a new `Mutation::ListValue` operator.
- **Reporter trait**: attach lifecycle observers to `Ga` via `.with_reporter(Box::new(r))`. Hooks: `on_start`, `on_generation_complete(&GenerationStats)`, `on_new_best(generation, chromosome)`, `on_finish(TerminationCause, &[GenerationStats])`. Zero overhead when no reporter is configured. Built-in: `NoopReporter`, `SimpleReporter` (stdout every N gens), `DurationReporter` (wall-clock timing summary).
- **Visualization** (`visualization` feature): generate PNG or SVG charts from run statistics — `plot_fitness`, `plot_diversity`, `plot_histogram`. Format detected from path extension. Powered by `plotters`; absent from the binary unless the feature is enabled.

---

## [2.1.0] - Unreleased

### Added
- **Extension strategies for population diversity control**: optional diversity-rescue mechanisms that trigger when fitness standard deviation drops below a configurable threshold. Four strategies: `MassExtinction`, `MassGenesis`, `MassDegeneration`, `MassDeduplication`.
- `ExtensionOperator` trait for custom implementations.
- Builder methods: `with_extension_method()`, `with_extension_diversity_threshold()`, `with_extension_survival_rate()`, `with_extension_mutation_rounds()`, `with_extension_elite_count()`.
- **Population diversity metric**: `GenerationStats` now exposes a `diversity` field (fitness standard deviation) updated every generation. Extension strategies and adaptive mutation use it internally; users can read it in callbacks and reporters.
- **List genotype**: `genotypes::List<T>` and `chromosomes::ListChromosome<T>` for problems over finite symbolic alphabets (colors, directions, categories, etc.). Works with all existing operators. Includes `list_random_initialization` and `list_random_initialization_without_repetitions` initializers, plus a new `Mutation::ListValue` operator.
- **Reporter trait**: attach lifecycle observers to `Ga` via `.with_reporter(Box::new(r))`. Hooks: `on_start`, `on_generation_complete(&GenerationStats)`, `on_new_best(generation, chromosome)`, `on_finish(TerminationCause, &[GenerationStats])`. Zero overhead when no reporter is configured. Built-in: `NoopReporter`, `SimpleReporter` (stdout every N gens), `DurationReporter` (wall-clock timing summary).
- **Visualization** (`visualization` feature): generate PNG or SVG charts from run statistics — `plot_fitness`, `plot_diversity`, `plot_histogram`. Format detected from path extension. Powered by `plotters`; absent from the binary unless the feature is enabled.

---

## [2.0.0] - 2026-03-01

### Added
- **Island Model GA** (`IslandGa`): evolve multiple sub-populations in parallel with configurable migration policies and topologies (ring, fully-connected, custom).
- **NSGA-II multi-objective optimizer** (`Nsga2Ga`): non-dominated sorting, crowding distance, per-objective direction (min/max), and constraint handling with constrained tournament selection.
- **Island + NSGA-II hybrid** (`IslandNsga2Ga`): orchestrator combining island model with NSGA-II for distributed multi-objective optimization.
- **Fitness sharing / niching**: maintain population diversity by penalizing similar individuals.
- **Elitism support**: `with_elitism(n)` preserves the top N individuals across generations.
- **Early-stop callbacks**: `ControlFlow`-based callbacks with per-generation `GenerationStats` (best/worst/avg fitness, std dev).
- **Compound stopping criteria** (`StoppingCriteria`): stagnation detection (N generations without improvement), convergence threshold (fitness std dev), and time limit (wall-clock seconds). New `TerminationCause` variants: `StagnationReached`, `ConvergenceReached`, `TimeLimitReached`.
- **Serde support** (feature flag `serde`): opt-in serialization/deserialization for all core types.
- **Checkpoint save/load**: save and restore GA progress to/from disk via the `serde` feature.
- **Seedable RNG**: reproducible GA runs with user-supplied random seeds.
- **New selection operators**: `Selection::Rank`, `Selection::Boltzmann`, `Selection::Truncation`.
- **New crossover operators**: `Crossover::SinglePoint`, `Crossover::Order` (OX), `Crossover::Sbx` (Simulated Binary), `Crossover::BlendAlpha` (BLX-α), `Crossover::Pmx` (Partially Mapped), `Crossover::Arithmetic`.
- **New mutation operators**: `Mutation::Value` (for `Range<T>`), `Mutation::BitFlip` (for binary chromosomes), `Mutation::Creep` (uniform perturbation), `Mutation::Gaussian` (normal perturbation), `Mutation::Polynomial`, `Mutation::NonUniform`, `Mutation::Insertion`.
- **New survivor strategies**: μ+λ and μ,λ survivor selection.
- `GaError` enum for structured error handling — all factories and operators return `Result<T, GaError>` instead of panicking.
- `stats` module with `GenerationStats` for per-generation tracking.
- `error` module (`src/error.rs`) with variants: `ConfigurationError`, `ValidationError`, `CrossoverError`, `MutationError`, `InitializationError`, `SelectionError`.
- `build()` validation step on `Ga`, `IslandGa`, `Nsga2Ga` builders.
- Standard trait implementations (`Debug`, `Clone`, `Default`, `PartialEq`) on core types.
- `dna_mut()` on `ChromosomeT` for zero-copy in-place gene edits.
- Operator traits (`SelectionOperator`, `CrossoverOperator`, `MutationOperator`) for extensibility.
- Builder methods: `with_sbx_eta()`, `with_blend_alpha()`, `with_mutation_step()`, `with_mutation_sigma()`, `with_stopping_criteria()`.
- OneMax binary example ("Hello World" of genetic algorithms).
- Comprehensive benchmark suite: hot-path benchmarks for `Ga::run`, NSGA-II, and Island GA.
- Extensive test coverage: unit tests for zero-coverage modules, edge-case tests for all operators, integration tests for NSGA-II, Island GA, AGA, niching, and Range chromosomes.
- `AGENT_INSTRUCTIONS.md` and `CONTRIBUTING.md`.

### Changed
- **BREAKING**: All thread-based parallelism replaced with `rayon` (`par_iter`, `into_par_iter`). Manual thread spawning removed.
- **BREAKING**: Parent pairs returned as `Vec<(usize, usize)>` instead of `HashMap<usize, usize>` (deterministic ordering, no duplicate key collisions).
- **BREAKING**: Mutation factory uses `ValueMutable` trait with default swap fallback instead of `Any`/`downcast`.
- **BREAKING**: `get_` prefix removed from accessor methods (e.g., `get_fitness()` → `fitness()`).
- **BREAKING**: Non-negative `i32` fields changed to `usize`.
- **BREAKING**: Builder pattern unified to consuming `self -> Self`.
- **BREAKING**: `ConfigurationT` broken into focused sub-traits.
- `ValueMutable` trait extended with `creep_mutate()` and `gaussian_mutate()` methods.
- `GeneT::get_id()` now returns `i32` directly (was wrapped).
- `ChromosomeT::set_dna` uses `Cow<'a, [Gene]>` to avoid redundant copies.
- Selection functions accept `&[U]` instead of `&Vec<U>`.
- Internal configuration structs use `#[derive(Default)]` where applicable.

### Removed
- `Any`/`downcast` usage in mutation factory.
- Manual thread spawning and join handles (replaced by `rayon`).
- `pprof` profiler integration from benchmarks (version incompatibility with Criterion 0.7+).

### Fixed
- Critical correctness bugs in selection, crossover, mutation, and fitness evaluation.
- Eliminated all panics and undefined behavior — safety hardening across the library.
- Eliminated all `clippy` warnings across library, tests, and benchmarks.
- SBX and BLX-α crossover enabled through enum dispatch via `Any` downcasting.
- Warnings added when `ValueMutable` defaults fall back to swap mutation.
- Flaky CI tests fixed (concurrency-safe RNG, increased time limits).

### Performance
- Parallelize NSGA-II objective evaluation with `rayon`.
- Parallelize island evolution with `rayon` `par_iter_mut`.
- Eliminate objectives cloning in NSGA-II by using slice references.
- Eliminate full population clone in best-chromosome update.
- Eliminate per-chromosome clone in fitness calculation.
- Use `select_nth_unstable` to clone only elite individuals in `extract_elite`.
- Use `HashSet` for segment membership in order crossover.
- Use `HashMap` for gene ID lookup in cycle crossover.
- Use `HashSet` for `unique_gene_ids` and O(N) `same_dna_length`.
- Use `truncate`/`drain` instead of loop `remove` in survivor selection.
- Use Fisher-Yates partial shuffle in random selection.

---

## [1.6.0] - 2024-10-30

### Added
- Benchmark suite using Criterion with `pprof` profiling for selection, crossover, mutation, and survivor operations.

### Fixed
- Cycle crossover bug: incorrect DNA generation for offspring during cycle detection ([#94](https://github.com/leimbernon/rust_genetic_algorithms/issues/94)).
- Inversion mutation crash: `None` reference error when accessing gene indices ([#95](https://github.com/leimbernon/rust_genetic_algorithms/issues/95)).
- Roulette wheel selection: simplified and corrected `total_fitness` calculation to avoid incorrect probability distribution ([#93](https://github.com/leimbernon/rust_genetic_algorithms/issues/93)).

### Changed
- Simplified test folder structure.
- Downgraded minimum Rust version for GitHub Actions compatibility.

---

## [1.5.0] - 2024-08-30

### Added
- **Progress callback**: register a callback function that is invoked after each generation, receiving the current population and generation number. Enables real-time monitoring and custom logging.
- Termination cause reported in the callback.
- Unit tests for the callback mechanism.

---

## [1.4.2] - 2024-08-27

### Fixed
- Version metadata correction (no functional changes).

---

## [1.4.1] - 2024-08-27

### Fixed
- Setters returning `&mut self` for builder chaining in save-progress configuration.

---

## [1.4.0] - 2024-08-24

### Added
- **Save/load GA progress**: new configuration parameters to persist genetic algorithm state (population, generation, configuration) to disk and resume from a checkpoint. Enables long-running experiments to survive interruptions.
- New dependencies for serialization support.

### Changed
- Removed `Copy` derive from configuration structs to support non-Copy fields.

---

## [1.3.2] - 2023-11-22

### Fixed
- Bug where the number of couples was not set correctly, causing a panic when the configuration omitted `number_of_couples` ([#75](https://github.com/leimbernon/rust_genetic_algorithms/issues/75)).

---

## [1.3.1] - 2023-11-18

### Fixed
- Packages and security dependencies updated.
- Documentation improvements.

---

## [1.3.0] - 2023-11-17

### Added
- **Configuration builder pattern**: fluent API to configure GA with chained method calls (`with_selection()`, `with_crossover()`, `with_mutation()`, `with_limits()`). Replaces verbose struct initialization.
- `ConfigurationT` trait for GA configuration, enabling custom configuration implementations.
- Configuration trait integrated into the `Ga` structure.

### Changed
- Configuration is now set within the `Ga` builder chain instead of being constructed separately and passed in.
- Trait renaming: added `T` suffix for consistency (e.g., `Configuration` → `ConfigurationT`).
- Deleted CircleCI badge (all CI now runs on GitHub Actions).

---

## [1.2.0] - 2023-11-04

### Added
- **Population initialization within GA**: the GA can now initialize the population automatically from configuration, removing the need to manually create and pass a population.
- Parameter to set initial population directly if already constructed.
- Condition checker updated to validate initialization parameters.

### Changed
- If population is provided, initialization step is skipped.
- Documentation simplified and updated.

---

## [1.1.0] - 2023-10-12

### Added
- **Adaptive Genetic Algorithm (AGA)**: crossover and mutation probabilities automatically adjust based on population fitness statistics.
  - Adaptive crossover probability: adjusts based on individual fitness relative to population average and maximum fitness.
  - Adaptive mutation probability: same adaptive mechanism applied to mutation rates.
  - New AGA-specific configuration parameters (`probability_max`, `probability_min`).
- **Random population initialization**: functions to initialize a population with random DNA, with or without repeated alleles. Supports multithreading.
- Allele uniqueness validation: panic if unique alleles are required but fewer alleles than DNA length are provided ([#51](https://github.com/leimbernon/rust_genetic_algorithms/issues/51)).
- Panic with descriptive message when fitness target is not set for fixed-fitness problems ([#50](https://github.com/leimbernon/rust_genetic_algorithms/issues/50)).
- Custom `LogLevel` enum replacing direct use of `log::LevelFilter` in configuration ([#49](https://github.com/leimbernon/rust_genetic_algorithms/issues/49)).
- `Gene::set_id()` function for setting gene IDs after construction.
- Average and maximum fitness tracked on the population without extra iteration.
- Condition checkers for AGA crossover and mutation configuration.
- Scramble mutation fix for edge cases.

### Fixed
- Fitness proportionate selection: corrected probability calculation.
- Scramble mutation: off-by-one error in gene range.
- `add_individuals` function: error when adding individuals to a population ([#57](https://github.com/leimbernon/rust_genetic_algorithms/issues/57)).

### Changed
- Functions renamed: removed "multithread" from function names where single-thread and multithread variants were consolidated.

---

## [1.0.1] - 2023-09-05

### Fixed
- Info-level log messages reduced — generation-level detail moved to debug logs to avoid flooding the console.

---

## [1.0.0] - 2023-09-05

### Added
- **Logging system**: log traces at all levels (error, warn, info, debug, trace) across all GA operations — selection, crossover, mutation, survivor, and main GA loop.
- Log level configurable via `GaConfiguration` (uses `env_logger`).
- `kv_unstable` feature for structured log key-value pairs.
- `SECURITY.md` security policy.

### Changed
- **BREAKING**: Getters no longer return references for `Copy` types (`get_id`, `get_fitness`, `get_age` return values directly).
- **BREAKING**: `get_dna()` returns `&[Gene]` (slice) instead of `Vec<Gene>`.
- **BREAKING**: `get_dna_mut()` replaced by `set_dna()` and `set_gene()` for controlled mutation.
- **BREAKING**: `GeneT::get_id()` made optional (default implementation provided).
- **BREAKING**: `Gene` is now a type alias used inside `Genotype`, making the genotype generic over the gene type.
- `new_gene()` method made optional with a default implementation.
- `GeneT::new_gene()` provides a default implementation.
- Condition that didn't match requirements fixed ([#23](https://github.com/leimbernon/rust_genetic_algorithms/issues/23)).
- Duplicated lines removed from README.
- Clippy results uploaded to GitHub via CI.
- Technical debt addressed: lint fixes and test improvements.

---

## [0.9.2] - 2023-08-22

### Fixed
- Technical debt: code quality improvements and lint fixes.

---

## [0.9.1] - 2023-08-22

### Fixed
- Technical debt: Rust code quality improvements and lint compliance.

---

## [0.9.0] - 2023-08-22

### Added
- **Operator probabilities**: crossover, mutation, and selection operations now support configurable probability values, allowing fine-grained control over how often each operator is applied.
- Selection method probability configuration.
- README updated with probability configuration documentation.

### Changed
- GA configuration simplified to include probabilities inline.

---

## [0.8.4] - 2023-07-26

### Changed
- CI/CD improvements: added unit testing badge, set up `rust-unit-tests.yml` workflow.
- Documentation improved.

---

## [0.8.3] - 2023-07-26

### Fixed
- Minor documentation fixes.

---

## [0.8.2] - 2023-07-26

### Changed
- Added Rust syntax highlighting to code documentation.
- Version number updated.

---

## [0.8.1] - 2023-07-26

### Changed
- Documentation improvements and version number fix.

---

## [0.8.0] - 2023-07-25

### Added
- **Best individual by generation**: new function `get_best_individual_by_generation()` to retrieve the best-performing individual at any given generation. Useful for tracking convergence history.
- New configuration field to support per-generation tracking.

### Changed
- Renamed `get_best_fitness_by_generation` to `get_best_individual_by_generation` (returns full individual, not just fitness).

---

## [0.7.3] - 2023-07-25

### Changed
- README improvements and version number update.

---

## [0.7.2] - 2023-07-25

### Changed
- Code style fixes.
- Added `rust-clippy.yml` GitHub Actions workflow for automated linting.

---

## [0.7.1] - 2023-01-07

### Fixed
- Cleanup of unused functions.

---

## [0.7.0] - 2023-01-07

### Added
- **Full multithreading for crossover operations**: crossover now runs in parallel across parent pairs, completing the multithreading coverage for all major GA operations.

---

## [0.6.0] - 2023-01-02

### Added
- **Multithreaded mutation**: mutation operations now execute in parallel using threads.

---

## [0.5.1] - 2022-12-26

### Fixed
- Tournament selection multithreading error: race condition fixed when running tournament selection across multiple threads.

---

## [0.5.0] - 2022-12-26

### Added
- **Multithreading support**: fitness calculation, parent selection, and tournament selection now run in parallel using threads. First major performance feature.
- Documentation for multithreading usage.

---

## [0.4.0] - 2022-12-25

### Added
- **Tournament selection** operator.
- **Run-based API** (`ga.run()`): simplified entry point that handles the full GA loop (selection → crossover → mutation → survivor → repeat).
- **Configuration struct**: centralized GA configuration with all parameters in one place.
- **Fitness target termination**: GA stops when a target fitness value is reached.
- **Fitness distance calculation**: measure how far current best is from the target.
- `LICENSE` file (Apache-2.0).

### Changed
- Major refactoring of configuration management — parameters moved into a dedicated configuration module.
- Naming conventions standardized.

---

## [0.3.0] - 2022-12-24

### Added
- **Uniform crossover** operator.
- **Scramble mutation** operator.
- **Stochastic universal sampling** selection operator.

### Changed
- README updated with new operators documentation.

---

## [0.2.1] - 2022-12-18

### Changed
- README styling improvements.

---

## [0.2.0] - 2022-12-18

### Added
- **Roulette wheel selection** (fitness proportionate selection) operator.
- **Multipoint crossover** operator (configurable number of crossover points).
- **Inversion mutation** operator.
- **Age-based selection** operator.

---

## [0.1.1] - 2022-12-04

### Changed
- README improvements with additional information about the library.

---

## [0.1.0] - 2022-12-04

### Added
- Initial release of the `genetic_algorithms` library.
- Core genetic algorithm loop: selection → crossover → mutation → survivor selection → repeat.
- **Gene and Genotype traits**: generic abstractions for representing individuals with typed DNA.
- **Cycle crossover** operator.
- **Swap mutation** operator.
- **Random selection** operator.
- **Fitness-based survivor selection** operator.
- **Population** struct with support for adding/removing individuals.
- Phenotype (fitness) calculation.
- Maximization and minimization problem support.
- Unit tests for all implemented operators.

---

[2.4.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/2.3.0...HEAD
[2.3.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/2.2.1...2.3.0
[2.2.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/2.2.0...2.2.1
[2.2.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/2.1.0...2.2.0
[2.1.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/2.0.0...2.1.0
[2.0.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.6.0...2.0.0
[1.6.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.5.0...1.6.0
[1.5.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.4.2...1.5.0
[1.4.2]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.4.1...1.4.2
[1.4.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.4.0...1.4.1
[1.4.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.3.2...1.4.0
[1.3.2]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.3.1...1.3.2
[1.3.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.3.0...1.3.1
[1.3.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.2.0...1.3.0
[1.2.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.1.0...1.2.0
[1.1.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.0.1...1.1.0
[1.0.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.9.2...1.0.0
[0.9.2]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.9.1...0.9.2
[0.9.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.9.0...0.9.1
[0.9.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.8.4...0.9.0
[0.8.4]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.8.3...0.8.4
[0.8.3]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.8.2...0.8.3
[0.8.2]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.8.1...0.8.2
[0.8.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.7.3...0.8.0
[0.7.3]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.7.2...0.7.3
[0.7.2]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.7.1...0.7.2
[0.7.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.5.1...0.6.0
[0.5.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/leimbernon/rust_genetic_algorithms/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/leimbernon/rust_genetic_algorithms/releases/tag/0.1.0
