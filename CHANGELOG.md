# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-03-01

### Added
- `GaError` enum for structured error handling across all operations.
- All factories and operators now return `Result<T, GaError>` instead of panicking.
- Parallelism via `rayon` replacing manual thread management.
- `Mutation::Value` variant for numeric range chromosomes (`Range<T>`).
- `Mutation::BitFlip` variant for binary chromosomes (flips a random gene's boolean value).
- `Mutation::Creep` — small uniform perturbation mutation for `Range<T>` chromosomes (configurable step size).
- `Mutation::Gaussian` — normal distribution perturbation mutation for `Range<T>` chromosomes (configurable sigma).
- `Crossover::SinglePoint` operator (splits parents at one random point, swaps tails).
- `Crossover::Order` operator (OX — preserves relative gene order; essential for permutation/TSP problems).
- `Crossover::Sbx` — Simulated Binary Crossover for continuous optimization with `Range<T>` chromosomes (configurable eta).
- `Crossover::BlendAlpha` — BLX-α blend crossover for continuous optimization with `Range<T>` chromosomes (configurable alpha).
- `Selection::Rank` — rank-based selection that assigns probability proportional to fitness rank, avoiding dominance by very fit individuals.
- Compound stopping criteria: `StoppingCriteria` with stagnation detection (N generations without improvement), convergence threshold (fitness std dev), and time limit (wall-clock seconds).
- New `TerminationCause` variants: `StagnationReached`, `ConvergenceReached`, `TimeLimitReached`.
- Elitism support: `with_elitism(n)` preserves the top N individuals across generations.
- `stats` module with `GenerationStats` for per-generation tracking (best/worst/avg fitness, std dev).
- `error` module (`src/error.rs`) with `GaError` variants: `ConfigurationError`, `ValidationError`, `CrossoverError`, `MutationError`, `InitializationError`, `SelectionError`.
- Builder methods: `with_sbx_eta()`, `with_blend_alpha()`, `with_mutation_step()`, `with_mutation_sigma()`, `with_stopping_criteria()`.
- `AGENT_INSTRUCTIONS.md` with coding guidelines for contributors and agents.
- `CONTRIBUTING.md` with development workflow and conventions.

### Changed
- Parent pairs returned as `Vec<(usize, usize)>` instead of `HashMap<usize, usize>` (deterministic ordering, no duplicate key collisions).
- Mutation factory uses `ValueMutable` trait with default swap fallback instead of `Any`/`downcast`.
- `ValueMutable` trait extended with `creep_mutate()` and `gaussian_mutate()` methods (default falls back to swap).
- `CrossoverConfiguration` extended with `sbx_eta` and `blend_alpha` fields.
- `MutationConfiguration` extended with `step` and `sigma` fields.
- Mutation factory supports `factory_with_params()` for passing step/sigma to Creep/Gaussian mutations.
- All thread-based parallelism replaced with `rayon` (`par_iter`, `into_par_iter`).
- `GeneT::get_id()` now returns `i32` directly (was wrapped).
- `ChromosomeT::set_dna` uses `Cow<'a, [Gene]>` to avoid redundant copies.
- Selection functions accept `&[U]` instead of `&Vec<U>`.
- Internal configuration structs use `#[derive(Default)]` where applicable.

### Removed
- `Any`/`downcast` usage in mutation factory.
- Manual thread spawning and join handles (replaced by `rayon`).
- `pprof` profiler integration from benchmarks (version incompatibility with Criterion 0.7).
- Obsolete examples using old `set_dna(&[Gene])` signatures.

### Fixed
- Eliminated all `clippy` warnings across library, tests, and benchmarks.
- Resolved `cargo fmt` trailing whitespace issues.
- Fixed `if_same_then_else` in validator factory.
- Fixed `clone_on_copy` in range initializer.
- Replaced `assert_eq!` with `assert!` for boolean comparisons in tests.

