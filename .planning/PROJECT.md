# genetic_algorithms

## What This Is

A modular, concurrent Genetic Algorithms library for Rust. Provides composable operators (selection, crossover, mutation, survivor), multi-threaded execution via `rayon`, Island Model GA, NSGA-II multi-objective optimization, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA, adaptive GA mode, elitism/stopping criteria, population diversity tracking, a full `GaObserver<U>` trait system, constraint handling, Hall of Fame, warm start, Adaptive Operator Selection (AOS), memetic algorithm framework, and standard benchmark suites (ZDT, DTLZ). Also provides four alternative metaheuristic engines: Differential Evolution (5 strategies + JADE/L-SHADE), Scatter Search, Cellular GA (2D toroidal grid, 4 neighborhoods), and ALPS (age-layered populations). Published on crates.io as `genetic_algorithms` with ten runnable examples.

## Core Value

Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library.

## Current Milestone: v3.0.0 — Advanced Representations, Alternative Strategies & Architecture Simplification

**Goal:** Use the major semver break to simplify library architecture and usability, introduce new genotypes and alternative strategies, and add advanced chromosome representations.

**Target features:**
- Architecture audit and API simplification — reduce boilerplate, clean up types that grew organically across v2.x
- Unified `Strategy` trait abstracting over GA, HillClimb, and Permutate (#177)
- HillClimb strategy: Stochastic and SteepestAscent variants (#172)
- Permutate strategy: exhaustive enumeration for small search spaces (#173)
- `Unique<T>` genotype for permutation problems (TSP, scheduling) (#174)
- `MultiRange<T>` genotype: per-gene independent ranges and mutation (#175)
- `MultiUnique<T>` genotype: multiple independent permutation groups (#176)
- Lexicase Selection: multi-case fitness evaluation (#220)
- Multi-parent crossover operators: UNDX, SPX, PCX (#221)
- Self-adaptive mutation: strategy parameters co-evolving within the chromosome (#222)
- Tree Chromosome for Genetic Programming (#223)
- Variable-length chromosomes (#224)

## Last Milestone: v2.4.0 — Observer Integration, New Operators, Advanced Multi-Objective & Framework Extensions (Shipped 2026-05-18)

**Shipped:** GaObserver hooks wired into all 4 alt-metaheuristic engines; 7 new operators (Clearing selection, Deterministic Crowding, Edge Recombination, DE crossover/mutation, Cauchy/Lévy Flight/Uniform mutation); NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA multi-objective engines; multi-objective quality indicators (Hypervolume, GD, IGD, Spread); constraint handling, Hall of Fame, warm start, AOS, memetic algorithm framework; standard benchmark suites (ZDT, DTLZ, single-objective); full documentation refactor; WASM (wasm32-unknown-unknown) support.

## Previous Milestone: v2.3.0 — Alternative Metaheuristics & Population Models (Shipped 2026-04-27)

Restructured src/ non-breakingly and shipped 4 independent optimization engines: DE (5 strategies + JADE/L-SHADE), Scatter Search, Cellular GA (4 neighborhoods, sync/async), ALPS (3 age schemes, cross-layer mating). 58 files, 3,361 LOC added. Observer hooks not yet wired into the 4 new engines (deferred to next milestone).

## Previous Milestone: v2.2.1 — Performance Optimizations (Shipped 2026-04-23)

Eliminated unnecessary heap allocations, reduced algorithmic complexity, and improved concurrency across the GA engine. All 24 requirements closed. No public API changes.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Core GA engine with configurable operators — v1.0
- ✓ Binary and Range<T> genotypes — v1.0
- ✓ Selection: Random, RouletteWheel, SUS, Tournament, Rank, Boltzmann, Truncation — v2.0
- ✓ Crossover: Cycle, MultiPoint, Uniform, SinglePoint, Order, SBX, BlendAlpha, PMX, Arithmetic — v2.0
- ✓ Mutation: Swap, Inversion, Scramble, Value, BitFlip, Creep, Gaussian, Polynomial, NonUniform, Insertion — v2.0
- ✓ Survivor: Fitness, Age, μ+λ, μ,λ — v2.0
- ✓ Island Model GA with migration topologies — v2.0
- ✓ NSGA-II multi-objective optimization — v2.0
- ✓ Island + NSGA-II hybrid — v2.0
- ✓ Fitness sharing / niching — v2.0
- ✓ Elitism support — v2.0
- ✓ Compound stopping criteria (stagnation, convergence, time limit) — v2.0
- ✓ Structured error handling (GaError) — v2.0
- ✓ Serde support (feature flag) — v2.0
- ✓ Checkpoint save/load — v2.0
- ✓ Seedable RNG — v2.0
- ✓ Adaptive GA (dynamic crossover/mutation probabilities) — v2.0
- ✓ Per-generation statistics (GenerationStats) — v2.0
- ✓ Rayon-based parallelism — v2.0
- ✓ Population diversity metric (`GenerationStats.diversity`) wired into extension trigger and dynamic mutation — v2.1.0
- ✓ `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets, compatible with all operators — v2.1.0
- ✓ `Reporter<U>` trait with `on_start`, `on_generation_complete`, `on_new_best`, `on_finish` hooks; zero overhead when unset — v2.1.0
- ✓ `visualization` feature flag: `plot_fitness`, `plot_diversity`, `plot_histogram` (PNG/SVG via plotters) — v2.1.0
- ✓ Six runnable examples: `rastrigin`, `feature_selection`, `niching`, `nsga2_zdt1`, `island_model`, `job_scheduling` — v2.1.0
- ✓ README `## Examples` table documenting all 10 examples with `cargo run` commands — v2.1.0
- ✓ `GaObserver<U>` trait with 12 lifecycle/operator/event hooks, zero overhead via `Option<Arc<dyn GaObserver>>` — v2.2.0
- ✓ `LogObserver` reproducing all pre-v2.2.0 `log!()` output; all hardcoded log calls removed from `ga.rs` — v2.2.0
- ✓ `TracingObserver` behind `observer-tracing` feature flag — structured `tracing` spans per generation — v2.2.0
- ✓ `IslandGaObserver<U>` and `Nsga2Observer<U>` sub-traits wired into Island GA and NSGA-II run loops — v2.2.0
- ✓ `CompositeObserver<U>` fan-out + `AllObserver` blanket impl — v2.2.0
- ✓ `MetricsObserver` behind `observer-metrics` feature flag — 11 per-generation metrics via `metrics` facade — v2.2.0
- ✓ Zero-allocation crossover and mutation: parent clones deferred/eliminated; PMX and OX upgraded to O(n) position maps — v2.2.1
- ✓ Rank/Boltzmann selection upgraded to O(log n) binary search; fitness values collected once per generation — v2.2.1
- ✓ Fitness sharing O(n) on-the-fly (eliminates O(n²) distance matrix); elite reinsertion and mass genesis use `select_nth_unstable_by()` — v2.2.1
- ✓ RNG atomics relaxed to Acquire/Relaxed; extension regrow parallelized with rayon — v2.2.1
- ✓ `Range` gene uses `Arc<[(T,T)]>` shared slice; `Copy`-specialized `value()`; `MassDeduplication` uses incremental `DefaultHasher` — v2.2.1
- ✓ `GenerationStats` moved (not cloned); truncation and island migration use O(n) partitioning; `Arc` migrant sharing — v2.2.1
- ✓ Non-breaking `src/` restructure into `engines/`, `types/`, `observe/` groups via `#[path]` re-exports in lib.rs — v2.3.0
- ✓ Differential Evolution engine: `DeGene` trait, 5 mutation strategies, binomial/exponential crossover, JADE/L-SHADE adaptive variants; 11 tests, benchmark — v2.3.0
- ✓ Scatter Search engine: diversification, reference set, combination, optional local search; 7 tests, benchmark — v2.3.0
- ✓ Cellular GA engine: 2D toroidal grid, 4 neighborhood types (VonNeumann/Moore/CompactR2/Linear), sync/async update; 10 tests, benchmark — v2.3.0
- ✓ ALPS engine: age-layered population, 3 age schemes (Linear/Fibonacci/Polynomial), cross-layer mating, periodic injection; 11 tests, benchmark — v2.3.0
- ✓ GaObserver hooks wired into DeEngine, ScatterEngine, CellularEngine, AlpsEngine — v2.4.0
- ✓ Clearing selection operator (#196); Deterministic Crowding survivor (#197) — v2.4.0
- ✓ Edge Recombination crossover (#198); DE crossover/mutation for standard GA (#199) — v2.4.0
- ✓ Cauchy mutation (#200), Lévy Flight mutation (#201), Uniform mutation (#202) — v2.4.0
- ✓ NSGA-III (#203), MOEA/D (#204), SPEA2 (#205), SMS-EMOA/IBEA (#206), quality indicators (#207) — v2.4.0
- ✓ Constraint handling, Hall of Fame, warm start, AOS, memetic algorithm framework (#212–#219) — v2.4.0
- ✓ ZDT, DTLZ, single-objective benchmark suites — v2.4.0
- ✓ WASM (wasm32-unknown-unknown) support — v2.4.0
- ✓ `MultiCaseFitness: ChromosomeT` opt-in trait: `case_fitness() -> &[f64]`, `set_case_fitness(Vec<f64>)` — v3.0.0 Phase 50
- ✓ `LexicaseSelection`: shuffles test cases per event, filters case-by-case to elites, syncs scalar fitness to mean — v3.0.0 Phase 50
- ✓ `EpsilonLexicaseSelection`: fixed or dynamic MAD epsilon per case; `SelectionConfiguration::epsilon` with `0.0` = dynamic MAD sentinel — v3.0.0 Phase 50

### Active

<!-- Current scope. Building toward these. -->

- [ ] Architecture audit: full review of public API, traits, builders, enums, and module structure — v3.0.0
- [ ] API simplification: reduce boilerplate for common-case usage, clean up types that grew organically across v2.x — v3.0.0
- [ ] Remove deprecated `Reporter<U>` trait (soft-deprecated since v2.2.0) — v3.0.0
- [ ] Unified `Strategy` trait abstracting GA, HillClimb, and Permutate under one interface (#177) — v3.0.0
- [ ] HillClimb strategy: Stochastic and SteepestAscent variants (#172) — v3.0.0
- [ ] Permutate strategy: exhaustive enumeration for small search spaces (#173) — v3.0.0
- [ ] `Unique<T>` genotype for permutation problems (TSP, scheduling) (#174) — v3.0.0
- [ ] `MultiRange<T>` genotype: per-gene independent ranges and mutation behavior (#175) — v3.0.0
- [ ] `MultiUnique<T>` genotype: multiple independent permutation groups (#176) — v3.0.0
- [ ] Multi-parent crossover operators: UNDX, SPX, PCX (#221) — v3.0.0
- [ ] Self-adaptive mutation: strategy parameters co-evolving within the chromosome (#222) — v3.0.0
- [ ] Tree Chromosome for Genetic Programming: breaks `dna() -> &[Gene]` linear assumption (#223) — v3.0.0
- [ ] Variable-length chromosomes: most architecturally disruptive change (#224) — v3.0.0

### Future

<!-- Validated direction, not yet scheduled. -->

(none currently — v3.0.0 closes all known planned feature gaps)

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- GUI/interactive visualization — library generates static PNG/SVG charts; interactive dashboards are users' concern
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick
- Per-gene hooks in observer — too granular, unacceptable overhead in hot loops
- DE-vs-GA head-to-head benchmark — deferred from v2.4.0; not a user-facing feature

## Context

- Library is published on crates.io; v3.0.0 is a major bump — breaking changes are intentional and expected
- v2.4.0 shipped: ~20,000+ LOC Rust, 10 runnable examples, full observer system, 5 multi-objective engines, 4 alt-metaheuristic engines, framework extensions, benchmark suites, WASM support
- `Reporter<U>` (v2.1.0) is soft-deprecated since v2.2.0 — will be removed in v3.0.0
- `GaObserver<U>` is the canonical lifecycle hook system; all new engines use it
- Feature flags: `serde`, `observer-tracing`, `observer-metrics`, `visualization`, `benchmarks`
- GitHub milestones #4 (Alternative strategies) and #11 (Advanced Representations) define v3.0.0 scope
- Architecture simplification is a first-class goal — v3.0.0 is the only opportunity for breaking ergonomic fixes

## Constraints

- **Breaking changes allowed**: v3.0.0 is a major version — intentional API breaks are in scope
- **Zero overhead**: `Option::None` branch when no observer — no allocations, no measurements
- **Feature flags**: `observer-tracing` and `observer-metrics` off by default
- **Rust edition**: 2021, MSRV 1.81.0
- **Thread safety**: All observers and chromosomes must be `Send + Sync` (used across rayon threads)
- **WASM compatibility**: All new features must compile for `wasm32-unknown-unknown`; gate `Instant::now()` and `par_iter()` with `#[cfg]`

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| fitness_std_dev as diversity metric | Simple, allocation-free, one pass over fitness values | ✓ Good — wired cleanly into extension and dynamic mutation |
| Stats computed once per generation, then passed to subsystems | Eliminates duplicate computation; diversity is authoritative | ✓ Good — removed compute_cardinality call from ga.rs |
| `Reporter<U>` as `Option<Box<dyn Reporter<U> + Send>>` | Simpler than Arc for single-owner; Box is fine for non-shared | ✓ Good — zero overhead confirmed via if-let guard |
| plotters 0.3.7 for visualization | Widely used, pure Rust, no C deps, good PNG/SVG support | ✓ Good — compiles cleanly behind feature flag |
| RangeChromosome<i32> for job_scheduling permutation | `list_random_initialization(..., Some(false))` achieves unique IDs | ⚠ Revisit — ListChromosome would be more semantic |
| Facade pattern (log, tracing, metrics crates) | Users choose their own backends; library stays agnostic | ✓ Good — confirmed working in v2.2.0 |
| Observer via `Option<Arc<dyn GaObserver<U>>>` | Zero cost when unused, shared across rayon threads | ✓ Good — zero overhead confirmed, `Send+Sync` enforced at compile time |
| Default no-op methods on GaObserver hooks | Forward-compatible: new events don't break existing observers | ✓ Good — all observers implement only what they need |
| `AllObserver<U>` blanket impl | Any type implementing all three observer traits gets AllObserver automatically | ✓ Good — TracingObserver composability gap closed via empty impls |
| Extension block before on_generation_end | Extension is part of the generation, not a post-generation side effect | ✓ Good — restores pre-v2.2.0 hook ordering semantics |
| Combined elapsed for operator timing (EXT-01 deferred) | Single timing block covers crossover+mutation+fitness; per-operator separation is v2.3+ | — Pending (EXT-01) |
| `#[path]` re-exports in lib.rs for directory restructure | Zero downstream breakage without semver bump; all existing `use genetic_algorithms::...` paths preserved | ✓ Good — no breakage confirmed |
| `mod.rs` directory form for restructured modules | Flat `.rs` file breaks submodule resolution when subdirs exist; directory mod is correct Rust idiom | ✓ Good — deviation from original plan was necessary and correct |
| `DeGene` trait extending `GeneT` | DE requires f64 arithmetic; generic `GeneT` doesn't provide it; trait extension keeps engine generic without polluting core | ✓ Good — clean extension point |
| `ValueMutable` bound for CellularEngine | In-place mutation operators require it; consistent with ga.rs pattern | ✓ Good — same pattern as existing engine |
| 20% cross-layer mating probability in ALPS | Balances exploration vs exploitation without full inter-layer panmixia | ✓ Good — keeps layers meaningful |
| `injection_interval = 0` disables ALPS injection | Clean opt-out with zero overhead; no special-case enum needed | ✓ Good — consistent with 0-means-disabled pattern |
| Observer not wired in new engines (v2.3.0 deferral) | Implementations were fast-tracked via direct commits; observer integration requires additional plan+execute cycle | ⚠ Revisit — v2.4.0 should wire observer into all 4 new engines |
| `Arc<[(T,T)]>` for Range gene ranges | Constructed once, read-many — Arc eliminates per-gene heap allocation; serde rc feature enables deserialization | ✓ Good — zero-copy sharing across crossover/mutation |
| `Copy` bound on Range gene impls | All concrete Range users (f64, i32) satisfy Copy; enables value-return without clone in hot accessor | ✓ Good — backward compatible (Copy implies Clone) |
| On-the-fly niching (apply_fitness_sharing_with_dna) | Old O(n²) matrix left intact (no breaking change); new function replaces it in ga.rs loop only | ✓ Good — safe incremental opt, old API preserved |
| `Acquire`/`Relaxed` RNG atomics | Acquire on SEED.load pairs with Release on set_seed — correct visibility; COUNTER only needs monotonic uniqueness | ✓ Good — minimal ordering, correct under Rust memory model |
| `Arc<Vec<U>>` for island migrant sharing | Neighbors auto-deref through Arc; single allocation shared across topology — no per-neighbor clone | ✓ Good — measurable reduction for high-connectivity topologies |

## Context

- Library is published on crates.io; backward compatibility matters
- v2.3.0 shipped: src/ restructured into engines/, types/, observe/ (non-breaking); 4 new alternative engines; ~17,000 LOC Rust (estimated), 10 runnable examples
- `Reporter<U>` (v2.1.0) coexists with `GaObserver<U>` (v2.2.0) — soft-deprecated but not removed
- All observer traits use default no-op methods for forward compatibility
- Feature flags keep optional dependencies (`tracing`, `metrics`) out of default builds
- New engines (DE, Scatter, Cellular, ALPS) do NOT yet have `GaObserver` hooks — next milestone priority
- GitHub milestones: #1–#9 shipped; next milestone candidates: Observer for new engines, New Operators (#196–#202), Advanced Multi-Objective (#203–#207)

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-06-15 after Phase 68 — Build/perf M2 dependency hygiene complete: env_logger removed from [dependencies], LogLevel/with_logs() removed, logging feature gate added (default-on), crate::log_*! macro family introduced, CI feature matrix extended, logger-history.md intel file created. 8/8 must-haves verified, SC-6 gap closed.*
