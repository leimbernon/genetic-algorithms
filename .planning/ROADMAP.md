# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- ✅ **v2.1.0 — New Examples** — Phases 10-12 (shipped 2026-03-22)
- ✅ **v2.2.0 — Observability & Traceability** — Phases 13-18 (shipped 2026-03-28)
- ✅ **v2.2.1 — Performance Optimizations** — Phases 19-24 (shipped 2026-04-23)
- ✅ **v2.3.0 — Alternative Metaheuristics & Population Models** — Phases 25-29 (shipped 2026-04-27)
- 🚧 **v2.4.0 — Observer Integration & New Operators + Advanced Multi-Objective** — Phases 30-45 (in progress)

## Phases

<details>
<summary>✅ v2.1 — Improve Usability, partial (Phases 1-5) — SHIPPED 2026-03-20</summary>

Phases 1-5 predate GSD tracking. Issues closed: #165, #166, #167, #168, #169.

- [x] Extension strategies (MassExtinction, MassGenesis, MassDegeneration, MassDeduplication)
- [x] Dynamic mutation probability based on population cardinality
- [x] Clone crossover strategy
- [x] Rejuvenate crossover operator
- [x] LRU fitness cache

</details>

<details>
<summary>✅ v2.2 — Improve Usability, completion (Phases 6-9) — SHIPPED 2026-03-21</summary>

Issues closed: #170, #171, #178, #179.

- [x] **Phase 6: Diversity Estimation** — `GenerationStats.diversity` wired into extension trigger and dynamic mutation (completed 2026-03-20)
- [x] **Phase 7: List Genotype** — `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets (completed 2026-03-21)
- [x] **Phase 8: Reporter Trait** — `Reporter<U>` with 4 lifecycle hooks, `SimpleReporter`, `DurationReporter` (completed 2026-03-21)
- [x] **Phase 9: Visualization** — `visualization` feature flag, `plot_fitness`, `plot_diversity`, `plot_histogram` (completed 2026-03-21)

</details>

<details>
<summary>✅ v2.1.0 — New Examples (Phases 10-12) — SHIPPED 2026-03-22</summary>

- [x] **Phase 10: Single-population Examples** — `rastrigin`, `feature_selection`, `niching` (completed 2026-03-22)
- [x] **Phase 11: Advanced Mode Examples** — `nsga2_zdt1`, `island_model`, `job_scheduling` (completed 2026-03-22)
- [x] **Phase 12: Documentation** — README `## Examples` table with all 10 examples and `cargo run` commands (completed 2026-03-22)

Full archive: `.planning/milestones/v2.1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v2.2.0 — Observability & Traceability (Phases 13-18) — SHIPPED 2026-03-28</summary>

Issues: #182, #183, #184, #185, #186

- [x] **Phase 13: GaObserver Base Trait** — Core trait + `Ga<U>` integration; foundation all other phases depend on (completed 2026-03-25)
- [x] **Phase 14: LogObserver + Log Migration** — Backward-compatible log migration; validates Phase 13 end-to-end (completed 2026-03-25)
- [x] **Phase 15: TracingObserver** — Structured tracing spans behind `observer-tracing` feature flag (completed 2026-03-26)
- [x] **Phase 16: Sub-Traits** — `IslandGaObserver` and `Nsga2Observer` for engine-specific events (completed 2026-03-27)
- [x] **Phase 17: CompositeObserver + MetricsObserver** — Fan-out composition and metrics facade behind `observer-metrics` flag (completed 2026-03-27)
- [x] **Phase 18: Observer API Polish** — Close audit gaps: TracingObserver AllObserver compatibility, ga.rs hook ordering and timing accuracy, lib.rs public API re-exports (completed 2026-03-28)

Full archive: `.planning/milestones/v2.2.0-ROADMAP.md` *(in v2.2.1 archive)*

</details>

<details>
<summary>✅ v2.2.1 — Performance Optimizations (Phases 19-24) — SHIPPED 2026-04-23</summary>

Issues: #187, #188, #189, #190, #191, #192

- [x] **Phase 19: Clone Elimination** — Defer parent clones until needed; build crossover children directly; use in-place mutation for numeric and index operators (completed 2026-03-30)
- [x] **Phase 20: Crossover Algorithm Optimization** — Replace O(n²) linear scans in PMX with O(n) HashMap position map (completed 2026-03-30)
- [x] **Phase 21: Selection Algorithm Optimization + Allocation Reduction** — Binary search for Rank and Boltzmann selection; collect fitness values once per generation; on-the-fly niching eliminates O(n²) distance matrix (completed 2026-03-31)
- [x] **Phase 22: Survivor & Extension Optimization** — O(n) elite reinsertion and mass genesis best-scan; relaxed RNG atomic ordering; parallel extension regrow (completed 2026-03-31)
- [x] **Phase 23: Memory Layout** — Shared `Arc<[(T,T)]>` range slice for Range genes; Copy-type value returns; dead field removal; incremental deduplication hash (completed 2026-04-04)
- [x] **Phase 24: Minor Improvements** — Move stats instead of clone; O(n) truncation and best-scan deduplication; O(n) island migration sort elimination; Arc migrant sharing (completed 2026-04-05)

Full archive: `.planning/milestones/v2.2.1-ROADMAP.md`

</details>

<details>
<summary>✅ v2.3.0 — Alternative Metaheuristics & Population Models (Phases 25-29) — SHIPPED 2026-04-27</summary>

Issues: #208, #209, #210, #211

- [x] **Phase 25: Directory Restructure** — Non-breaking reorganization of src/ into engines/, types/, observe/ (completed 2026-04-26)
- [x] **Phase 26: Differential Evolution Engine** — 5 mutation strategies, binomial/exponential crossover, JADE and L-SHADE adaptive variants (completed 2026-04-26)
- [x] **Phase 27: Scatter Search Engine** — Diversification, reference set management, combination, optional local search (completed 2026-04-26)
- [x] **Phase 28: Cellular GA Engine** — 2D toroidal grid, 4 neighborhood types, synchronous/asynchronous update modes (completed 2026-04-27)
- [x] **Phase 29: ALPS Engine** — Age-layered population, 3 age schemes, cross-layer mating, periodic injection (completed 2026-04-27)

Full archive: `.planning/milestones/v2.3.0-ROADMAP.md`

</details>

### v2.4.0 — Observer Integration, New Operators, Advanced Multi-Objective & Framework Extensions (In Progress)

**Milestone Goal:** Wire GaObserver lifecycle hooks into all 4 new engines, close v2.3.0 deferred tech debt, expand the operator library with 7 new strategies, extend multi-objective optimization with NSGA-III, MOEA/D, SPEA2, SMS-EMOA/IBEA, and shared quality indicators, and extend the core GA framework with constraint handling, Hall of Fame, warm starting, adaptive operator selection, benchmarks, and memetic algorithms.

- [x] **Phase 30: Observer Wiring & DE Benchmark** — Wire GaObserver into all 4 new engines and add DE-vs-GA convergence benchmark (completed 2026-05-02)
- [x] **Phase 31: Selection & Survivor Diversity Operators** — Clearing selection and Deterministic Crowding survivor strategy (completed 2026-05-04)
- [x] **Phase 32: Crossover & Differential Mutation** — Edge Recombination crossover and DE-style differential mutation for standard GA (completed 2026-05-06)
- [x] **Phase 33: Scalar Mutation Operators** — Cauchy, Levy Flight, and Uniform mutation operators (completed 2026-05-07)
- [x] **Phase 34: WASM support** — Fix time-based panics for wasm32-unknown-unknown targets (completed 2026-05-07)
- [x] **Phase 35: NSGA-III** — Reference-point based NSGA-III for many-objective optimization (completed 2026-05-09)
- [x] **Phase 36: MOEA/D** — Weight-vector decomposition with Tchebycheff or PBI scalarisation (completed 2026-05-10)
- [x] **Phase 37: SPEA2** — Archive-based strength Pareto selection (completed 2026-05-10)
- [x] **Phase 38: SMS-EMOA and IBEA** — Hypervolume-based and indicator-based MOEAs (completed 2026-05-11)
- [x] **Phase 39: Quality indicators** — Hypervolume, GD, IGD, Spread (completed 2026-05-11)
- [x] **Phase 40: Constraint Handling** — Penalty functions, Deb's feasibility rules, RepairOperator (#212, #213, #214) (completed 2026-05-11)
- [x] **Phase 41: Hall of Fame / Solution Archive** — Bounded archive with deduplication and min-distance diversity (#217)
- [x] **Phase 42: Warm Starting & Population Seeding** — Initial population, seeded population, checkpoint resumption (#216) (completed 2026-05-13)
- [ ] **Phase 43: Adaptive Operator Selection (AOS)** — Operator portfolio with Probability Matching, Adaptive Pursuit, MAB (#218)
- [ ] **Phase 44: Standard Benchmark Functions Suite** — Unimodal, multimodal, ZDT, DTLZ behind `benchmarks` feature flag (#219)
- [ ] **Phase 45: Memetic Algorithm Framework** — LocalSearchOperator with Lamarckian/Baldwinian modes (#215)

## Phase Details

### Phase 30: Observer Wiring & DE Benchmark
**Goal**: Users can attach a GaObserver to any of the four new engines and observe the same lifecycle events they get from the standard GA, with DE-vs-GA convergence data available as a benchmark
**Depends on**: Phase 29
**Requirements**: OBS-01, OBS-02, OBS-03, OBS-04, OBS-05
**Success Criteria** (what must be TRUE):
  1. User can pass an `Option<Arc<dyn GaObserver<U>>>` to `DeEngine` and receive `on_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, and `on_finish` calls during a run
  2. User can do the same with `ScatterEngine`, `CellularEngine`, and `AlpsEngine` — identical hook set, same zero-overhead guarantee when observer is `None`
  3. Running `cargo bench --bench de` produces a comparison report showing DE convergence curves alongside an equivalent GA run on the same problem
  4. All existing tests for the four engines continue to pass with no behavioral changes (observer is purely additive)
**Plans:** 3 plans

Plans:
- [x] 30-01-PLAN.md — Wire GaObserver into DeEngine and ScatterEngine
- [x] 30-02-PLAN.md — Wire GaObserver into CellularEngine and AlpsEngine
- [x] 30-03-PLAN.md — DE-vs-GA convergence benchmark

**UI hint**: no

### Phase 31: Selection & Survivor Diversity Operators
**Goal**: Users can promote population diversity through two new operator strategies — Clearing selection that removes similar individuals within a niche radius, and Deterministic Crowding that replaces parents with more-similar offspring
**Depends on**: Phase 30
**Requirements**: SEL-01, SRV-01
**Success Criteria** (what must be TRUE):
  1. User can set `Selection::Clearing` with a configurable niche radius; individuals within that radius of a niche winner are cleared from the selection pool each generation
  2. User can set `Survivor::DeterministicCrowding`; each offspring is compared against its most-similar parent, and the fitter of the two survives
  3. Both operators compose with all existing crossover and mutation operators without compile errors or panics
  4. Tests in `tests/` verify the diversity-preserving behavior of each operator in isolation
**Plans:** 2/2 plans complete
**UI hint**: no

Plans:
**Wave 1**
- [x] 31-01-PLAN.md — Clearing selection operator (SEL-01)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 31-02-PLAN.md — DeterministicCrowding survivor operator (SRV-01)

### Phase 32: Crossover & Differential Mutation
**Goal**: Users can configure Edge Recombination crossover for permutation problems and Differential mutation (DE-style) for real-valued standard GAs
**Depends on**: Phase 31
**Requirements**: CRS-01, MUT-04
**Success Criteria** (what must be TRUE):
  1. User can set `Crossover::EdgeRecombination`; offspring adjacency lists are built from both parents and the resulting chromosome preserves adjacency relationships found in either parent
  2. User can set `Mutation::Differential` with a configurable F scale factor; the mutant vector is computed from three random population members and the operator is available in the standard `Ga<U>` engine
  3. Both operators follow the enum + factory pattern and integrate with `ConfigurationT` builder methods without new required parameters on existing configurations
  4. Tests in `tests/` cover edge cases for Edge Recombination (short chromosomes, duplicate edges) and Differential mutation (population size bounds for three-member sampling)
**Plans:** 3/3 plans complete
**UI hint**: no

Plans:
**Wave 1**
- [x] 32-01-PLAN.md — Edge Recombination crossover (CRS-01)

**Wave 2** *(blocked on Wave 1)*
- [x] 32-02-PLAN.md — Differential mutation operator + config + builder (MUT-04)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 32-03-PLAN.md — Engine dispatch + serde test updates (CRS-01, MUT-04)

### Phase 33: Scalar Mutation Operators
**Goal**: Users can apply three additional real-valued mutation strategies — Cauchy heavy-tail perturbations, Levy Flight long-range jumps, and Uniform random reset — each with configurable parameters
**Depends on**: Phase 32
**Requirements**: MUT-01, MUT-02, MUT-03
**Success Criteria** (what must be TRUE):
  1. User can set `Mutation::Cauchy` with a configurable scale parameter; gene perturbations follow a Cauchy (Lorentzian) distribution, producing occasional large steps
  2. User can set `Mutation::LevyFlight` with a configurable stability index; gene perturbations follow a Levy distribution, enabling long-range jumps beyond what Gaussian mutation produces
  3. User can set `Mutation::Uniform`; each selected gene is reset to a uniformly random value within the gene's valid range
  4. All three operators follow the enum + factory pattern; `cargo test` and `cargo clippy` pass with no warnings; tests confirm distributional properties in `tests/`
**Plans:** 3/3 plans complete
**UI hint**: no

Plans:
**Wave 1**
- [x] 33-01-PLAN.md — Cauchy operator + dispatch infrastructure (config fields, builder methods, six-engine routing) + Cauchy tests + Levy/Uniform test scaffolds (MUT-01)

**Wave 2** *(blocked on Wave 1)*
- [x] 33-02-PLAN.md — LevyFlight operator (Mantegna algorithm) + activate Levy tests (MUT-02)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 33-03-PLAN.md — Uniform operator + activate Uniform tests + serde coverage + phase verification gate (MUT-03)

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1-5. Usability (partial) | v2.1 | -- | Complete | 2026-03-20 |
| 6. Diversity Estimation | v2.2 | 2/2 | Complete | 2026-03-20 |
| 7. List Genotype | v2.2 | 2/2 | Complete | 2026-03-21 |
| 8. Reporter Trait | v2.2 | 2/2 | Complete | 2026-03-21 |
| 9. Visualization | v2.2 | 2/2 | Complete | 2026-03-21 |
| 10. Single-population Examples | v2.1.0 | 3/3 | Complete | 2026-03-22 |
| 11. Advanced Mode Examples | v2.1.0 | 3/3 | Complete | 2026-03-22 |
| 12. Documentation | v2.1.0 | 1/1 | Complete | 2026-03-22 |
| 13. GaObserver Base Trait | v2.2.0 | 2/2 | Complete | 2026-03-25 |
| 14. LogObserver + Log Migration | v2.2.0 | 2/2 | Complete | 2026-03-25 |
| 15. TracingObserver | v2.2.0 | 2/2 | Complete | 2026-03-26 |
| 16. Sub-Traits | v2.2.0 | 3/3 | Complete | 2026-03-27 |
| 17. CompositeObserver + MetricsObserver | v2.2.0 | 3/3 | Complete | 2026-03-27 |
| 18. Observer API Polish | v2.2.0 | 2/2 | Complete | 2026-03-28 |
| 19. Clone Elimination | v2.2.1 | 3/3 | Complete | 2026-03-30 |
| 20. Crossover Algorithm Optimization | v2.2.1 | 1/1 | Complete | 2026-03-30 |
| 21. Selection + Allocation Reduction | v2.2.1 | 3/3 | Complete | 2026-03-31 |
| 22. Survivor & Extension Optimization | v2.2.1 | 2/2 | Complete | 2026-03-31 |
| 23. Memory Layout | v2.2.1 | 2/2 | Complete | 2026-04-04 |
| 24. Minor Improvements | v2.2.1 | 2/2 | Complete | 2026-04-05 |
| 25. Directory Restructure | v2.3.0 | 3/3 | Complete | 2026-04-26 |
| 26. Differential Evolution Engine | v2.3.0 | 2/2 | Complete | 2026-04-26 |
| 27. Scatter Search Engine | v2.3.0 | 1/1 | Complete | 2026-04-26 |
| 28. Cellular GA Engine | v2.3.0 | 1/1 | Complete | 2026-04-27 |
| 29. ALPS Engine | v2.3.0 | 1/1 | Complete | 2026-04-27 |
| 30. Observer Wiring & DE Benchmark | v2.4.0 | 3/3 | Complete | 2026-05-02 |
| 31. Selection & Survivor Diversity Operators | v2.4.0 | 2/2 | Complete | 2026-05-04 |
| 32. Crossover & Differential Mutation | v2.4.0 | 3/3 | Complete | 2026-05-06 |
| 33. Scalar Mutation Operators | v2.4.0 | 3/3 | Complete | 2026-05-07 |
| 34. WASM support — wasm32-unknown-unknown compatibility | v2.4.0 | 4/4 | Complete | 2026-05-07 |
| 35. NSGA-III for many-objective optimization | v2.4.0 | 3/3 | Complete    | 2026-05-09 |
| 36. MOEA/D decomposition-based multi-objective | v2.4.0 | 3/3 | Complete    | 2026-05-10 |
| 37. SPEA2 strength pareto evolutionary algorithm | v2.4.0 | 3/3 | Complete    | 2026-05-10 |
| 38. Indicator-based MOEAs — SMS-EMOA and IBEA | v2.4.0 | 3/3 | Complete | 2026-05-11 |
| 39. Multi-objective quality indicators | v2.4.0 | 3/3 | Complete | 2026-05-11 |
| 40. Constraint Handling | v2.4.0 | 3/3 | Complete    | 2026-05-11 |
| 41. Hall of Fame / Solution Archive | v2.4.0 | 3/3 | Complete | 2026-05-12 |
| 42. Warm Starting & Population Seeding | v2.4.0 | 3/3 | Complete    | 2026-05-13 |
| 43. Adaptive Operator Selection (AOS) | v2.4.0 | 0/3 | Pending | — |
| 44. Standard Benchmark Functions Suite | v2.4.0 | 0/0 | Pending | — |
| 45. Memetic Algorithm Framework | v2.4.0 | 0/0 | Pending | — |

### Phase 34: WASM support — fix time-based panics for wasm32-unknown-unknown targets (issue #236)

**Goal:** Users can compile and run a standard `Ga` or `Nsga2Ga` cycle on `wasm32-unknown-unknown` without panics from `Instant::now()` or rayon thread-pool initialization, while native parallel/timed behavior is preserved unchanged.
**Requirements**: N/A (issue-driven phase — see #236)
**Depends on:** Phase 33
**Plans:** 4/4 plans complete

Plans:
**Wave 1** *(disjoint files — run in parallel)*
- [x] 34-01-PLAN.md — cfg-gate Instant in DurationReporter
- [x] 34-02-PLAN.md — cfg-gate Instant + rayon + max_duration warning in src/engines/ga.rs
- [x] 34-03-PLAN.md — cfg-gate Instant + rayon in src/engines/nsga2/mod.rs

**Wave 2** *(blocked on Wave 1)*
- [x] 34-04-PLAN.md — wasm32 CI compile-check + host smoke test + phase verification gate


### Phase 35: NSGA-III for many-objective optimization

**Goal:** Users can run NSGA-III on problems with 3+ objectives; reference points are auto-generated (Das-Dennis simplex lattice) or user-supplied, and the algorithm selects survivors via reference-point association rather than crowding distance
**Requirements**: MOO-01
**Issue**: #203
**Depends on:** Phase 34
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 35-01-PLAN.md — Extract shared multi_objective module from nsga2 (MOO-01)

**Wave 2** *(blocked on Wave 1)*
- [x] 35-02-PLAN.md — NSGA-III scaffolding: error variant, Nsga3Observer trait, Nsga3Configuration, Das-Dennis generator, stub Nsga3Ga + Wave 0 tests (MOO-01)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 35-03-PLAN.md — Implement Nsga3Ga::run(): reference-point environmental selection, run() integration tests, DTLZ2 example, phase verification gate (MOO-01)

### Phase 36: MOEA/D — Decomposition-based multi-objective optimization

**Goal:** Users can run MOEA/D with configurable weight vectors and either Tchebycheff or PBI scalarisation; each sub-problem maintains a neighbourhood of similar weight vectors and offspring compete only within that neighbourhood
**Requirements**: MOO-02
**Issue**: #204
**Depends on:** Phase 35
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 36-01-PLAN.md — Scaffolding: error variant, MoeaDObserver trait, LogObserver impl, lib.rs re-exports, MoeaDConfiguration + ScalarizationFn, stub MoeaDGa with validate(), Wave 0 tests (MOO-02)

**Wave 2** *(blocked on Wave 1)*
- [x] 36-02-PLAN.md — MoeaDGa::run() with neighbourhood precomputation, Tchebycheff/PBI scalarization, ideal-point tracking, sub-problem update loop, WASM cfg-gating, integration tests (MOO-02)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 36-03-PLAN.md — examples/moead_dtlz2.rs + LogObserver smoke test + example registration + phase verification gate (MOO-02)

### Phase 37: SPEA2 — Strength Pareto Evolutionary Algorithm 2

**Goal:** Users can run SPEA2 with a configurable archive size; fitness is computed from raw strength + density (k-nearest-neighbour), and the archive is truncated using the Euclidean crowding criterion
**Requirements**: MOO-03
**Issue**: #205
**Depends on:** Phase 36
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 37-01-PLAN.md — Scaffolding: GaError variant, Spea2Configuration builder, Spea2Observer trait + LogObserver impl, lib.rs re-exports, stub Spea2Ga with validate(), Wave 0 tests (MOO-03)

**Wave 2** *(blocked on Wave 1)*
- [x] 37-02-PLAN.md — Spea2Ga::run() with fitness assignment (strength + density), archive management (environmental selection + Euclidean truncation), binary tournament mating, WASM cfg-gating, observer hooks, integration tests (MOO-03)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 37-03-PLAN.md — examples/spea2_zdt1.rs + LogObserver smoke test + example registration + phase verification gate (MOO-03)


### Phase 38: Indicator-based MOEAs — SMS-EMOA and IBEA

**Goal:** Users can run SMS-EMOA (hypervolume contribution-based steady-state removal) and IBEA (additive epsilon-indicator fitness); both share the quality-indicator library from Phase 39 and follow the same engine pattern
**Requirements**: MOO-04
**Issue**: #206
**Depends on:** Phase 39
**Plans:** 3/3 plans complete
**Completed:** 2026-05-11

Plans:
**Wave 1** *(parallel, disjoint files)*
- [x] 38-01-PLAN.md — SMS-EMOA scaffolding: error variant, SmsEmoaObserver trait, LogObserver impl, SmsEmoaConfiguration + builder, stub SmsEmoaGa + Wave 0 tests (MOO-04)
- [x] 38-02-PLAN.md — IBEA scaffolding: error variant, IbeaObserver trait, LogObserver impl, IbeaConfiguration + builder, stub IbeaGa + Wave 0 tests (MOO-04)

**Wave 2** *(blocked on Wave 1)*
- [x] 38-03-PLAN.md — SmsEmoaGa::run() + IbeaGa::run() full run loops + observer hooks + WASM gating + integration tests + examples/sms_emoa_zdt1.rs + examples/ibea_zdt1.rs + phase verification gate (MOO-04)

### Phase 39: Multi-objective quality indicators — Hypervolume, GD, IGD, Spread

**Goal:** A shared `src/engines/multi_objective/indicators/` directory exposes Hypervolume (2D Lebesgue), Generational Distance, Inverted Generational Distance, and Spread (Deb et al. 2002) as pure functions usable by any multi-objective engine and callable from user code for post-run analysis
**Requirements**: MOO-05
**Issue**: #207
**Depends on:** Phase 37
**Plans:** 3/3 plans complete
**Completed:** 2026-05-11

Plans:
**Wave 1**
- [x] 39-01-PLAN.md — Foundation: `GaError::InvalidIndicatorConfiguration` variant, wire `indicators/` module into `multi_objective`, shared validation helpers (MOO-05)

**Wave 2** *(parallel, both depend on Wave 1)*
- [x] 39-02-PLAN.md — Hypervolume + Generational Distance implementations and integration tests (MOO-05)
- [x] 39-03-PLAN.md — Inverted Generational Distance + Spread implementations, integration tests, phase verification gate (MOO-05)


### Phase 40: Constraint Handling — Penalty Functions, Feasibility Rules, RepairOperator

**Goal:** Users can solve constrained optimization problems by configuring penalty functions (static, dynamic, adaptive), Deb's feasibility rules for selection/survivor comparison, and a RepairOperator trait for fixing infeasible chromosomes after mutation
**Requirements**: CNS-01, CNS-02, CNS-03
**Issues**: #212, #213, #214
**Plans:** 3/3 plans complete

Plans:
- [x] 40-01-PLAN.md -- Fix test_constraints.rs compilation + add Adaptive Penalty / FeasibilityRules GA integration tests (CNS-01, CNS-02, CNS-03)
- [x] 40-02-PLAN.md -- NSGA-II constraint integration test module (CNS-01, CNS-02)
- [x] 40-03-PLAN.md -- Constrained G1 optimization example (CNS-01, CNS-02, CNS-03)

### Phase 41: Hall of Fame / Solution Archive

**Goal:** Users can maintain an archive of top-N unique solutions across the entire run, with optional minimum-distance diversity filtering, accessible after run completion
**Requirements**: ARC-01
**Completed:** 2026-05-12
**Issue**: #217
**Depends on:** Phase 40
**Plans:** 3 plans

Plans:
**Wave 1**
- [x] 41-01-PLAN.md — HallOfFame module foundation with core API and unit tests (HOF-01, HOF-02, HOF-03, HOF-05, HOF-07)

**Wave 2** *(blocked on Wave 1)*
- [x] 41-02-PLAN.md — Ga integration: struct field, builder, run loop, accessor, integration tests (HOF-04, HOF-06, HOF-09)
- [x] 41-03-PLAN.md — Serde round-trip, example, WASM check, phase verification gate (HOF-08, HOF-10)

### Phase 42: Warm Starting & Population Seeding

**Goal:** Users can initialize populations from known solutions, seeded individuals plus random fill, or deserialized checkpoints — enabling hot-start and transfer learning workflows
**Requirements**: WSM-01
**Issue**: #216
**Depends on:** Phase 41
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 42-01-PLAN.md — Ga struct fields, builder methods, build-time validation, test scaffolding (WSM-01-A, WSM-01-D, WSM-01-J, WSM-01-K)

**Wave 2** *(blocked on Wave 1)*
- [x] 42-02-PLAN.md — Seed-based initialization with genotypic dedup, trusted fitness, HOF admission (WSM-01-A, WSM-01-B, WSM-01-C, WSM-01-H, WSM-01-J, WSM-01-K)

**Wave 3** *(blocked on Wave 1)*
- [x] 42-03-PLAN.md — Checkpoint resumption with hybrid config override, absolute counting, stats preservation (WSM-01-D, WSM-01-E, WSM-01-F, WSM-01-G, WSM-01-I, WSM-01-J, WSM-01-K, WSM-01-L)

### Phase 43: Adaptive Operator Selection (AOS)

**Goal:** Users can configure portfolios of crossover and mutation operators, with Probability Matching, Adaptive Pursuit, or Multi-Armed Bandit selection dynamically choosing the best operator based on recent fitness-improvement credit
**Requirements**: AOS-01
**Issue**: #218
**Depends on:** Phase 42
**Plans:** 3 plans

Plans:
**Wave 1**
- [ ] 43-01-PLAN.md -- AOS core module (AosStrategy, AosState, reward model) + GaConfiguration fields + builder methods + build validation + unit tests (AOS-01)

**Wave 2** *(blocked on Wave 1)*
- [ ] 43-02-PLAN.md -- GA loop integration: AOS runtime state, offspring dispatch via AOS, reward accumulation, integration tests (AOS-01)

**Wave 3** *(blocked on Waves 1-2)*
- [ ] 43-03-PLAN.md -- Serde derives, AOS example, WASM check, phase verification gate (AOS-01)

### Phase 44: Standard Benchmark Functions Suite

**Goal:** Users can evaluate algorithms against 15+ standard benchmark functions (Sphere, Rastrigin, Ackley, ZDT1-6, DTLZ1-7) behind a `benchmarks` feature flag, each with metadata and verified optima
**Requirements**: BEN-01
**Issue**: #219
**Depends on:** Phase 43
**Plans:** 3 plans

Plans:
**Wave 1**
- [ ] 42-01-PLAN.md — Ga struct fields, builder methods, build-time validation, test scaffolding (WSM-01-A, WSM-01-D, WSM-01-J, WSM-01-K)

**Wave 2** *(blocked on Wave 1)*
- [ ] 42-02-PLAN.md — Seed-based initialization with genotypic dedup, trusted fitness, HOF admission (WSM-01-A, WSM-01-B, WSM-01-C, WSM-01-H, WSM-01-J, WSM-01-K)

**Wave 3** *(blocked on Wave 1)*
- [ ] 42-03-PLAN.md — Checkpoint resumption with hybrid config override, absolute counting, stats preservation (WSM-01-D, WSM-01-E, WSM-01-F, WSM-01-G, WSM-01-I, WSM-01-J, WSM-01-K, WSM-01-L)

### Phase 45: Memetic Algorithm Framework

**Goal:** Users can attach a LocalSearchOperator to the GA loop with configurable application strategies (AllOffspring, BestN, Probabilistic, EveryNGenerations) and Lamarckian/Baldwinian modes, with parallel execution via rayon
**Requirements**: MEM-01
**Issue**: #215
**Depends on:** Phase 44
**Plans:** 3 plans

Plans:
**Wave 1**
- [ ] 42-01-PLAN.md — Ga struct fields, builder methods, build-time validation, test scaffolding (WSM-01-A, WSM-01-D, WSM-01-J, WSM-01-K)

**Wave 2** *(blocked on Wave 1)*
- [ ] 42-02-PLAN.md — Seed-based initialization with genotypic dedup, trusted fitness, HOF admission (WSM-01-A, WSM-01-B, WSM-01-C, WSM-01-H, WSM-01-J, WSM-01-K)

**Wave 3** *(blocked on Wave 1)*
- [ ] 42-03-PLAN.md — Checkpoint resumption with hybrid config override, absolute counting, stats preservation (WSM-01-D, WSM-01-E, WSM-01-F, WSM-01-G, WSM-01-I, WSM-01-J, WSM-01-K, WSM-01-L)
