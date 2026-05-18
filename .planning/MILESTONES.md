# Milestones

## v2.4.0 — Observer Integration, New Operators, Advanced Multi-Objective & Framework Extensions (Shipped: 2026-05-18)

Wired GaObserver into all 4 alt-metaheuristic engines, expanded the operator library, added 5 multi-objective engines, multi-objective quality indicators, full framework extensions suite, standard benchmark functions, WASM support, and a documentation refactor.

**Phases:** 30–46 (17 phases, 55 plans) | **Timeline:** 2026-04-27 → 2026-05-18 | **Files:** ~370 changed

**Key accomplishments:**

- GaObserver lifecycle hooks wired into DeEngine, ScatterEngine, CellularEngine, AlpsEngine
- 7 new operators: Clearing selection, Deterministic Crowding survivor, Edge Recombination crossover, DE crossover/mutation for standard GA, Cauchy/Lévy Flight/Uniform mutation
- NSGA-III (reference-point based), MOEA/D (decomposition), SPEA2 (strength Pareto), SMS-EMOA (steady-state HV), IBEA (indicator-based) multi-objective engines
- Multi-objective quality indicators: Hypervolume (WFG), Generational Distance, IGD, Spread
- Framework Extensions: constraint handling (penalty/repair/decoder), Hall of Fame, warm start / population seeding, Adaptive Operator Selection (AOS), memetic algorithm framework
- Standard benchmark suites: ZDT1–6, DTLZ1–7, classic single-objective (Sphere, Rastrigin, Ackley, etc.)
- WASM (wasm32-unknown-unknown) support — all time/thread APIs gated with `#[cfg(not(target_arch = "wasm32"))]`
- Full rustdoc documentation refactor across all public items

---

## v2.3.0 — Alternative Metaheuristics & Population Models (Shipped: 2026-04-27)

Restructured `src/` non-breakingly and shipped four independent optimization engines, each with unit tests and criterion benchmarks.

**Phases:** 25–29 (5 phases, 8 plans) | **Timeline:** 2026-04-26 → 2026-04-27 (2 days) | **Commits:** 5 | **Files:** 58 changed, 3,361 lines added

**Key accomplishments:**

- Non-breaking `src/` restructure: `engines/`, `types/`, `observe/` groups via `#[path]` lib.rs re-exports — all existing `use genetic_algorithms::...` paths preserved, zero semver bump needed
- Differential Evolution engine: `DeGene` trait + 5 mutation strategies (Rand/1, Best/1, CurrentToBest/1, Rand/2, Best/2), binomial/exponential crossover, JADE self-adaptive F/CR, L-SHADE history-memory F/CR; 11 integration tests
- Scatter Search engine: diversification phase (quality + diversity split), reference set management, linear-interpolation combination, optional hill-climbing local search; 7 integration tests
- Cellular GA engine: 2D toroidal grid with 4 neighborhood topologies (VonNeumann 4, Moore 8, CompactR2 24, Linear 2), synchronous and asynchronous update modes, greedy local replacement; 10 integration tests
- ALPS engine: age-layered population with Linear/Fibonacci/Polynomial age schemes, 20% cross-layer mating with adjacent elder, periodic layer-0 injection; 11 integration tests

**Known tech debt at close:** GaObserver lifecycle hooks not wired into any of the 4 new engines (deferred); DE-vs-GA head-to-head benchmark not added.

---

## v2.2.1 — Performance Optimizations (Shipped: 2026-04-23)

Eliminated unnecessary heap allocations, reduced algorithmic complexity, and improved concurrency across the GA engine — all internal changes with no public API impact.

**Phases:** 19–24 (6 phases, 13 plans) | **Timeline:** 2026-03-30 → 2026-04-05 | **Commits:** 21 perf/refactor

**Key accomplishments:**

- Eliminated redundant parent clones in crossover hot path; five numeric mutation operators use `set_gene()` instead of `dna().to_vec()` — zero Vec allocation per mutation call
- PMX crossover replaced O(n²) linear position scan with O(n) `HashMap` position map; OX similarly uses O(n) `HashSet` membership
- Rank and Boltzmann selection use `partition_point()` binary search (O(log n)); fitness values collected once per generation and shared across extension, niching, and stats
- Fitness sharing computes distance on-the-fly — eliminates O(n²) distance matrix allocation per generation
- Elite reinsertion and mass genesis both use `select_nth_unstable_by()` O(n) instead of O(n log n) sort
- RNG atomic ordering relaxed from `SeqCst` to `Acquire`/`Relaxed`; extension population regrow parallelized via rayon
- `Range` genes share `Arc<[(T,T)]>` slice per chromosome; `value()` for `Copy` types returns by value; `MassDeduplication` uses incremental `DefaultHasher`
- `GenerationStats` moved (not cloned) into stats history; island migration uses `select_nth_unstable_by()` and `Arc`-shared migrant vectors

---

## v2.1.0 — New Examples (Shipped: 2026-03-22)

Added `GenerationStats.diversity`, `ListChromosome<T>` genotype, `Reporter<U>` lifecycle trait, and a `visualization` feature flag — then demonstrated the whole library with six runnable examples covering every major GA mode.

**Phases:** 6–12 (7 phases, 15 plans) | **Timeline:** 2026-03-20 → 2026-03-22 | **Commits:** ~103

**Key accomplishments:**

- Added `diversity: f64` to `GenerationStats` (fitness std-dev); wired into extension trigger and dynamic mutation
- Introduced `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets, integrating with all existing operators
- Shipped `Reporter<U>` trait with `on_start`, `on_generation_complete`, `on_new_best`, `on_finish` hooks; zero overhead when unset
- Added `visualization` feature flag with `plot_fitness`, `plot_diversity`, and `plot_histogram` (PNG/SVG via plotters)
- Added six self-contained examples: `rastrigin`, `feature_selection`, `niching`, `nsga2_zdt1`, `island_model`, `job_scheduling`
- Updated README with `## Examples` table documenting all 10 examples with exact `cargo run` commands

**Known gaps (deferred):** Reporter/Visualization not demonstrated in examples; ListChromosome has no dedicated example. See `.planning/milestones/v2.1.0-MILESTONE-AUDIT.md`.

---

## v2.0.0 — Restructuring & Optimisation (Completed 2026-03-01)

Major rewrite: Island GA, NSGA-II, structured errors, rayon parallelism, serde support, new operators, elitism, stopping criteria, adaptive GA, checkpoint support.

**Phases:** Pre-GSD (no phase tracking)

---
