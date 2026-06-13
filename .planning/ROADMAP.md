# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- ✅ **v2.1.0 — New Examples** — Phases 10-12 (shipped 2026-03-22)
- ✅ **v2.2.0 — Observability & Traceability** — Phases 13-18 (shipped 2026-03-28)
- ✅ **v2.2.1 — Performance Optimizations** — Phases 19-24 (shipped 2026-04-23)
- ✅ **v2.3.0 — Alternative Metaheuristics & Population Models** — Phases 25-29 (shipped 2026-04-27)
- ✅ **v2.4.0 — Observer Integration & New Operators + Advanced Multi-Objective** — Phases 30-46 (shipped 2026-05-18)
- 🚧 **v3.0.0 — Advanced Representations, Alternative Strategies & Architecture Simplification** — Phases 47-65 (in progress)

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

- [x] **Phase 25: Directory Restructure** — Non-breaking reorganization of src/ into engines/, types/, observe/ (completed 2026-04-26)
- [x] **Phase 26: Differential Evolution Engine** — 5 mutation strategies, binomial/exponential crossover, JADE and L-SHADE adaptive variants (completed 2026-04-26)
- [x] **Phase 27: Scatter Search Engine** — Diversification, reference set management, combination, optional local search (completed 2026-04-26)
- [x] **Phase 28: Cellular GA Engine** — 2D toroidal grid, 4 neighborhood types, synchronous/asynchronous update modes (completed 2026-04-27)
- [x] **Phase 29: ALPS Engine** — Age-layered population, 3 age schemes, cross-layer mating, periodic injection (completed 2026-04-27)

Full archive: `.planning/milestones/v2.3.0-ROADMAP.md`

</details>

<details>
<summary>✅ v2.4.0 — Observer Integration, New Operators, Advanced Multi-Objective & Framework Extensions (Phases 30-46) — SHIPPED 2026-05-18</summary>

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
- [x] **Phase 43: Adaptive Operator Selection (AOS)** — Operator portfolio with Probability Matching, Adaptive Pursuit, MAB (#218)
- [x] **Phase 44: Standard Benchmark Functions Suite** — Unimodal, multimodal, ZDT, DTLZ behind `benchmarks` feature flag (#219) (completed 2026-05-14)
- [x] **Phase 45: Memetic Algorithm Framework** — LocalSearchOperator with Lamarckian/Baldwinian modes (#215)
- [x] **Phase 46: Documentation Refactor** — Comprehensive rustdoc, docs/ guides, README expansion (completed 2026-05-15)

</details>

### v3.0.0 — Advanced Representations, Alternative Strategies & Architecture Simplification (In Progress)

**Milestone Goal:** Use the major semver break to simplify library architecture, introduce three new genotype types, add two alternative strategy engines, and implement advanced chromosome representations (lexicase selection, multi-parent crossover, self-adaptive mutation, variable-length chromosomes, tree chromosome for GP).

- [ ] **Phase 47: Architecture Audit & ChromosomeT Split** — Reduce `ChromosomeT` to a minimal core; introduce `LinearChromosome` supertrait; remove `Reporter<U>`; apply 6 API simplifications; validate all 10 examples compile and run in CI
- [ ] **Phase 48: New Genotype Types** — `UniqueChromosome<T>` for permutation problems, `MultiRangeChromosome<T>` for per-gene bounds, `MultiUniqueChromosome<T>` for multiple independent permutation groups; migrate `job_scheduling` example
- [x] **Phase 49: Unified Strategy Trait + Alternative Strategy Engines** — `Strategy<U>` trait; `HillClimbEngine` (Stochastic + SteepestAscent); `PermutateEngine` with safety gate; observer hooks throughout
- [x] **Phase 50: Lexicase Selection** — `MultiCaseFitness: ChromosomeT` trait; `LexicaseSelection`; epsilon-lexicase variant; behavioral diversity CI test (completed 2026-05-23)
- [ ] **Phase 51: Multi-Parent Crossover + Self-Adaptive Mutation** — UNDX, SPX, PCX operators with `RealValued` marker trait; `SelfAdaptive: ChromosomeT` trait; `Mutation::SelfAdaptiveGaussian` with log-normal sigma update
- [x] **Phase 52: Variable-Length Chromosomes** — `ChromosomeLength::Variable { min, max }`; `Mutation::Insertion` / `Mutation::Deletion`; `Crossover::VariableLength(AlignmentStrategy)`; parsimony pressure survivor config (completed 2026-05-24)
- [x] **Phase 53: Tree Chromosome + GpGa Engine** — `TreeChromosome: ChromosomeT` supertrait; `GpGa<U>` engine; ramped half-and-half init; subtree crossover + mutation; bloat control; serde with `serde_stacker`; `Display` as expression string (completed 2026-05-25)

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
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 43-01-PLAN.md — AOS core module (AosStrategy, AosState, reward model) + GaConfiguration fields + builder methods + build validation + unit tests (AOS-01)

**Wave 2** *(blocked on Wave 1)*
- [x] 43-02-PLAN.md — GA loop integration: AOS runtime state, offspring dispatch via AOS, reward accumulation, integration tests (AOS-01)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 43-03-PLAN.md — Serde derives, AOS example, WASM check, phase verification gate (AOS-01)

### Phase 44: Standard Benchmark Functions Suite

**Goal:** Users can evaluate algorithms against 15+ standard benchmark functions (Sphere, Rastrigin, Ackley, ZDT1-6, DTLZ1-7) behind a `benchmarks` feature flag, each with metadata and verified optima
**Requirements**: BEN-01
**Issue**: #219
**Depends on:** Phase 43
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 44-01-PLAN.md — BenchmarkFn trait + single-objective benchmarks (Sphere, Rastrigin, Ackley) + Cargo.toml feature flag + WASM check (BEN-01)

**Wave 2** *(blocked on Wave 1)*
- [x] 44-02-PLAN.md — ZDT1-6 + DTLZ1-7 multi-objective benchmarks with unit tests + WASM check (BEN-01)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 44-03-PLAN.md — Serde derives, benches/de.rs migration, example migrations, phase verification gate (BEN-01)

### Phase 45: Memetic Algorithm Framework

**Goal:** Users can attach a LocalSearchOperator to the GA loop with configurable application strategies (AllOffspring, BestN, Probabilistic, EveryNGenerations) and Lamarckian/Baldwinian modes, with parallel execution via rayon
**Requirements**: MEM-01
**Issue**: #215
**Depends on:** Phase 44
**Plans:** 3 plans

Plans:
**Wave 1**
- [ ] 45-01-PLAN.md — Foundation: LocalSearchOperator trait, HillClimbing enum + factory, config types, serde, module wiring

**Wave 2** *(blocked on Wave 1)*
- [ ] 45-02-PLAN.md — Ga integration: struct field, builder method, generation loop, strategy dispatch, parallel execution, tests

**Wave 3** *(blocked on Waves 1-2)*
- [ ] 45-03-PLAN.md — Example (memetic_rastrigin), serde roundtrip test, WASM check, phase verification gate

### Phase 46: Update the documentation to explain in more details the different algorithms. A refactor of the documentation can happen if needed

**Goal:** Users (both human developers and AI models) can read comprehensive, production-quality documentation that precisely explains how and when to use every algorithm, operator, and framework extension in the library — from any entry point (docs.rs, README, docs/ directory)
**Requirements:** Documentation-only phase (requirements derived from CONTEXT.md decisions D-01 through D-11)
**Depends on:** Phase 45
**Plans:** 7/7 plans complete

Plans:
**Wave 1 — Foundation**
- [x] 46-01-PLAN.md — Crate SSOT + README expansion + docs/index.md (D-01, D-02, D-03, D-05, D-06, D-07)

**Wave 2 — Engine Ficha Tecnica //! docs (parallel)**
- [x] 46-02-PLAN.md — Single-objective + island engine //! docs to D-04 standard (D-04)
- [x] 46-03-PLAN.md — Multi-objective engine //! docs to D-04 standard (D-04)

**Wave 3 — docs/ Guide Files (parallel)**
- [x] 46-04-PLAN.md — 17 new docs/ guide files: per-engine guides + framework concept guides (D-03, D-04)
- [x] 46-05-PLAN.md — Existing docs/ updates: examples.md rewrite, engines.md expand, operator guides update (D-03)

**Wave 4 — Coverage + Verification**
- [x] 46-06-PLAN.md — Rustdoc /// on all public items, module //! docs, example inline comments, phase verification gate (D-08, D-09, D-10, D-11)

---

### Phase 47: Architecture Audit & ChromosomeT Split
**Goal**: Users can implement custom chromosomes using a clean, minimal `ChromosomeT` core and opt into flat-slice operator compatibility via `LinearChromosome`, without boilerplate from the old all-in-one trait
**Depends on**: Phase 46
**Requirements**: ARCH-01, ARCH-02, ARCH-03, ARCH-04, ARCH-05, ARCH-06, ARCH-07
**Success Criteria** (what must be TRUE):
  1. User can implement `ChromosomeT` with only `fitness()`, `set_fitness()`, `age()`, `set_age()`, and `calculate_fitness()` — no flat-slice methods required for types that are not linear
  2. User can implement `LinearChromosome: ChromosomeT` to gain full compatibility with all existing selection, crossover, mutation, and survivor operators — all operator bounds updated from `U: ChromosomeT` to `U: LinearChromosome`
  3. User who previously used `Reporter<U>` sees a compiler error with a clear message pointing to `GaObserver<U>` as the replacement; `MIGRATION.md` documents the upgrade path
  4. User can configure chromosome length as `ChromosomeLength::Fixed(n)` or `ChromosomeLength::Variable { min, max }` via the builder — existing code using the old `genes_per_chromosome` field does not compile, making the change auditable
  5. User can configure all stopping criteria via flat builder methods (`.with_stagnation_limit(50)`) without constructing a `StoppingCriteria` struct; `LocalSearch` is configured via an enum, not `Arc<dyn ...>`
  6. All 10 existing runnable examples (`cargo run --example <name>`) compile and pass their short-generation CI smoke tests on the milestone branch after every PR
**Plans:** 6/8 plans executed

Plans:
**PR 1 — ChromosomeT split (ARCH-01, ARCH-02)**
- [x] 47-01-PLAN.md — Wave 0 tests + split ChromosomeT into minimal core + LinearChromosome supertrait
- [x] 47-02-PLAN.md — Implementor updates (Binary, Range, List) + mechanical bound change across operator layer + ValueMutable supertrait upgrade
- [x] 47-03-PLAN.md — Engine orchestrators (Ga, DE, Scatter, Cellular, ALPS, NSGA-II/III, MOEA/D, SPEA2, SMS-EMOA, IBEA, Island) bound upgrade + PR 1 gate

**PR 2 — Config cleanup (ARCH-04, ARCH-05, ARCH-06)**
- [ ] 47-04-PLAN.md — ChromosomeLength enum + LimitConfiguration field removals + initializer signature cleanup
- [x] 47-05-PLAN.md — StoppingCriteria flattening into GaConfiguration + sub-struct accessor pattern + ga.rs path updates (WASM gate preserved)
- [x] 47-06-PLAN.md — Multi-obj engine + example + test caller migration + PR 2 gate

**PR 3 — Reporter removal + CI (ARCH-03, ARCH-07)**
- [ ] 47-07-PLAN.md — Reporter trait + impls + fire points removal + MIGRATION.md publication + README link + Cargo.toml include
- [x] 47-08-PLAN.md — examples-smoke.yml CI workflow + final Phase 47 verification gate

**UI hint**: no

### Phase 48: New Genotype Types
**Goal**: Users can model permutation problems, heterogeneous real-valued spaces, and multi-group permutation problems using three new semantically correct chromosome types — replacing ad-hoc hacks with purpose-built types
**Depends on**: Phase 47
**Requirements**: GEN-01, GEN-02, GEN-03, GEN-04
**Success Criteria** (what must be TRUE):
  1. User can create a `UniqueChromosome<T>` that initializes with no duplicate genes, all elements present from the given alphabet; attempting to apply `Crossover::SinglePoint` or `Crossover::Uniform` returns `GaError` at runtime
  2. User can run the `job_scheduling` example using `UniqueChromosome<i32>` in place of the old `RangeChromosome<i32>` unique-id hack — example produces valid job sequences
  3. User can create a `MultiRangeChromosome<T>` where each gene has its own `(lo_i, hi_i)` bounds; Gaussian mutation clamps each gene to its own per-gene range independently
  4. User can create a `MultiUniqueChromosome<T>` with multiple independent permutation groups; PMX/OX crossover applies within each group boundary and never corrupts group membership across the boundary
**Plans:** 4 plans
**UI hint**: no

Plans:
**Wave 1**
- [ ] 48-01-PLAN.md — OperatorCompat trait foundation + Crossover enum MultiGroup variants + build_child visibility + per-type empty impls + Wave 0 tests (GEN-01, GEN-04 foundation)

**Wave 2** *(blocked on Wave 1)*
- [ ] 48-02-PLAN.md — UniqueGenotype + UniqueChromosome + unique_random_initialization + job_scheduling example migration (GEN-01, GEN-02)

**Wave 3** *(blocked on Wave 2)*
- [ ] 48-03-PLAN.md — MultiRangeGenotype + MultiRangeChromosome + multi_range_random_initialization + per-gene Gaussian mutation (GEN-03)

**Wave 4** *(blocked on Wave 3)*
- [ ] 48-04-PLAN.md — MultiUniqueChromosome + group_ranges + multi_group_pmx + multi_group_ox dispatch + Phase 48 verification gate (GEN-04)

### Phase 49: Unified Strategy Trait + Alternative Strategy Engines
**Goal**: Users can swap between GA, hill-climbing, and exhaustive permutation search at runtime through a single `Strategy<U>` trait, and can use `Box<dyn Strategy<U>>` to select algorithms without rewriting application code
**Depends on**: Phase 47
**Requirements**: STR-01, STR-02, STR-03, STR-04
**Success Criteria** (what must be TRUE):
  1. User can write `let strategy: Box<dyn Strategy<U>> = Box::new(ga)` and call `.run()` / `.best()` identically regardless of whether the concrete type is `Ga<U>`, `HillClimbEngine<U>`, or `PermutateEngine<U>`
  2. User can run stochastic hill climbing by providing a `neighbor_fn` and an iteration limit; the engine accepts any neighbor with higher fitness and stops when no improvement is found within the limit; `GaObserver` hooks fire per iteration
  3. User can run steepest-ascent hill climbing with the same `neighbor_fn`; all returned neighbors are evaluated and only the single best is accepted per step; `GaObserver` hooks fire per iteration
  4. User can run `PermutateEngine` over a small search space; if the total permutation count exceeds the configurable safety gate, the engine emits a warning and returns the best candidate found so far rather than panicking; `GaObserver` hooks fire per candidate evaluated
**Plans**: 49-01, 49-02, 49-03, 49-04 — COMPLETE ✓
**UI hint**: no

### Phase 50: Lexicase Selection
**Goal**: Users can configure lexicase selection for any chromosome type implementing multi-case fitness evaluation, achieving specialist-preserving selection behavior that scalar fitness methods cannot produce
**Depends on**: Phase 47
**Requirements**: SEL-02, SEL-03, TRAITS-01
**Success Criteria** (what must be TRUE):
  1. User can implement `MultiCaseFitness: ChromosomeT` on a custom chromosome by adding `case_fitness() -> &[f64]` and `set_case_fitness(Vec<f64>)` — no changes to existing `ChromosomeT` methods required
  2. User can configure `Selection::Lexicase` and observe that test cases are shuffled randomly per selection event; the scalar `fitness()` is set to the mean case score for survivor and stopping-criteria compatibility
  3. User can configure `Selection::EpsilonLexicase { epsilon }` for continuous-valued case scores; individuals within epsilon of the best on each case are retained through that case's filter
  4. A CI behavioral diversity test confirms that a population evolved under `LexicaseSelection` produces measurably more specialists (individuals excelling on distinct case subsets) than `TournamentSelection` under matched effort
**Plans:** 2/2 plans complete

Plans:
**Wave 1**
- [x] 50-01-PLAN.md — MultiCaseFitness trait + Selection enum variants + SelectionConfiguration.epsilon + Wave 0 test stubs (TRAITS-01)

**Wave 2** *(blocked on Wave 1)*
- [x] 50-02-PLAN.md — lexicase + epsilon-lexicase operators + factory_lexicase + ga.rs dispatch + behavioral diversity test + phase verification gate (SEL-02, SEL-03)

**UI hint**: no

### Phase 51: Multi-Parent Crossover + Self-Adaptive Mutation
**Goal**: Users can evolve real-valued chromosomes using multi-parent crossover operators (UNDX, SPX, PCX) and self-adaptive mutation where per-chromosome sigma vectors co-evolve alongside the solution
**Depends on**: Phase 47
**Requirements**: CRS-02, CRS-03, CRS-04, MUT-05, TRAITS-02
**Success Criteria** (what must be TRUE):
  1. User can configure `Crossover::Undx { num_parents }`, `Crossover::Spx { num_parents }`, or `Crossover::Pcx { num_parents }` on any chromosome implementing the `RealValued` marker trait; binary and permutation chromosomes return `GaError` at build time
  2. User can configure `Mutation::SelfAdaptiveGaussian` on any chromosome implementing `SelfAdaptive: ChromosomeT`; per-chromosome sigma vectors update via the log-normal rule each generation; sigma values never fall below `sigma_min`
  3. After crossover of two `SelfAdaptive` chromosomes initialized with sigma=0.1 and sigma=0.9, the offspring sigma distribution spans the intermediate range — confirming that intermediate recombination is applied to strategy parameters, not copied from one parent
  4. `cargo check --target wasm32-unknown-unknown` passes for all new operators and traits without any conditional compilation errors
**Plans:** 4 plans

Plans:
**Wave 0**
- [x] 64-01-PLAN.md — Coverage baseline + cargo-llvm-cov CI gate (D-01, D-02, D-03, D-04, D-05)

**Wave 1** *(blocked on Wave 0)*
- [ ] 64-02-PLAN.md — Fix all #[allow(...)] suppressions at root cause (D-06, D-07, D-08, D-09, D-10)

**Wave 2** *(blocked on Waves 0-1, parallel with 64-04)*
- [ ] 64-03-PLAN.md — Data-driven coverage tests for lowest-coverage modules (D-05, D-14)

**Wave 2** *(blocked on Wave 1, parallel with 64-03)*
- [ ] 64-04-PLAN.md — Rustdoc # Examples blocks on all user-facing pub items (D-11, D-12, D-13)
**UI hint**: no

### Phase 52: Variable-Length Chromosomes
**Goal**: Users can evolve populations where chromosome length varies between individuals, with explicit length-aware crossover, insertion/deletion mutation, and optional parsimony pressure to prevent unbounded growth
**Depends on**: Phase 47
**Requirements**: MUT-06, CHR-01, CHR-02
**Success Criteria** (what must be TRUE):
  1. User can configure `ChromosomeLength::Variable { min, max }` and observe that `Mutation::Insertion` adds a gene at a random position (clamped to `max`) and `Mutation::Deletion` removes a gene at a random position (clamped to `min`)
  2. User can configure `Crossover::VariableLength(AlignmentStrategy)` to handle parents of different lengths; all 9 existing fixed-length crossover operators return `GaError::IncompatibleChromosomeLength` when applied to unequal-length parents instead of silently truncating
  3. The `ExtensionOperator` samples length from the current population distribution during regrowth rather than using a fixed length — new individuals have lengths drawn from the observed population range
  4. User can configure `length_penalty: f64` in the survivor configuration; longer chromosomes receive a proportional fitness penalty, preventing unbounded length growth across generations
**Plans**: 4 plans (4 complete — PHASE COMPLETE 2026-05-24)

Plans:
- [x] 52-01-PLAN.md — Wave 0: Test stubs (Nyquist compliance)
- [x] 52-02-PLAN.md — Wave 1: ChromosomeLength enum + MUT-06 length operators
- [x] 52-03-PLAN.md — Wave 2: Crossover::VariableLength + AlignmentStrategy + fixed-operator guard
- [x] 52-04-PLAN.md — Wave 3: Variable init, extension regrowth, parsimony pressure (all 13 tests enabled)

**UI hint**: no

### Phase 53: Tree Chromosome + GpGa Engine
**Goal**: Users can evolve tree-structured programs using a dedicated `GpGa<U>` engine with ramped half-and-half initialization, subtree crossover and mutation, enforced bloat limits, and full checkpoint support
**Depends on**: Phase 50
**Requirements**: CHR-03, CHR-04, CHR-05, CHR-06, CHR-07
**Success Criteria** (what must be TRUE):
  1. User can define a GP node enum implementing `GpNode` and create a `GpChromosome<G>` that satisfies `TreeChromosome: ChromosomeT` — the type does not implement `LinearChromosome`, so attempting to use it with linear operators produces a compile error, not a runtime panic
  2. User can run `GpGa<GpChromosome<G>>` with a `PrimitiveSet` containing user-defined functions and terminals (including ephemeral random constants); the engine uses ramped half-and-half initialization and produces valid trees each generation
  3. User can set `max_depth` and `max_node_count` in `GpConfiguration`; any subtree crossover or mutation that would produce a tree exceeding either limit returns `GaError::TreeDepthExceeded` or `GaError::TreeSizeExceeded` rather than silently accepting the oversized tree; `GenerationStats` includes average node count
  4. User can enable the `serde` feature flag and checkpoint/restore a GP run containing trees of depth >= 64 without stack overflow; CI runs this serialization test in the `serde` test suite
  5. User can call `.to_string()` on a `GpChromosome` and read the evolved program as a human-readable infix or prefix expression
**Plans**: 4 plans

Plans:
**Wave 0** (API contract)
- [x] 53-01-PLAN.md — Core types: GpNode trait, Node<N>, GpChromosome, TreeChromosome, GaError variants, GenerationStats.avg_node_count, tests/gp.rs stubs (CHR-03, CHR-07)

**Wave 1** *(blocked on Wave 0)*
- [x] 53-02-PLAN.md — GP operators: SubtreeCrossover + SubtreeMutation/PointMutation/HoistMutation with bloat enforcement (CHR-05)

**Wave 2** *(blocked on Wave 1)*
- [x] 53-03-PLAN.md — GpGa engine loop: ramped half-and-half init + full run() loop + observer hooks + avg_node_count (CHR-04, CHR-05)

**Wave 3** *(blocked on Wave 2)*
- [x] 53-04-PLAN.md — Serde checkpoint: serde_stacker dep + stack-safe Node<N> serde + depth-64 CI test (CHR-06)

**UI hint**: no

### Phase 56: CMA-ES Engine
**Goal**: Users can run Covariance Matrix Adaptation Evolution Strategy (CMA-ES) on real-valued black-box optimization problems via a new `CmaEngine<U>` — with `GaObserver` hooks from day 1, Hansen's default parameter formulas, and WASM-compatible execution. As part of this phase, the shared `DeGene` trait is hard-renamed to `RealGene` (v3.0.0 breaking change) and relocated to `src/traits/real_gene.rs`.
**Depends on**: Phase 55
**Requirements**: None (issue-driven phase — see issue #252; IPOP/BIPOP deferred per issue #255)
**Success Criteria** (what must be TRUE):
  1. User can call `CmaEngine::new(config, init_fn, fitness_fn).run()` and receive a `CmaResult<U>` containing population, best, best_fitness, and generations — for any chromosome implementing `LinearChromosome` where `U::Gene: RealGene`
  2. User can attach a `GaObserver<U>` via `.with_observer(...)` and receive `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, and `on_run_end` calls
  3. User can configure tuning via `CmaConfiguration` builder methods: `.with_sigma0()`, `.with_population_size()`, `.with_max_generations()`, `.with_problem_solving()`, `.with_fitness_target()`, `.with_cc()`, `.with_cs()`, `.with_c1()`, `.with_cmu()` — leaving cc/cs/c1/cmu as None defaults to Hansen's auto formulas
  4. `cargo check --target wasm32-unknown-unknown` passes; `cargo run --example cma_es_rastrigin` converges; `cargo test`, `cargo test --features serde`, `cargo clippy --all-targets -- -D warnings`, and `cargo doc --no-deps` all pass with zero warnings
  5. After the `DeGene → RealGene` cascade, all existing `DeEngine` and `ScatterEngine` tests continue to pass — the rename is purely identifier-level with no behavioral change
**Plans:** 4/4 plans complete

Plans:
**Wave 1**
- [x] 56-01-PLAN.md — DeGene → RealGene rename cascade across DE, Scatter, lib re-exports; add `RealGene` impl for `MultiRangeGenotype<f64>`

**Wave 2** *(blocked on Wave 1)*
- [x] 56-02-PLAN.md — `CmaConfiguration` (Default, `default_for_dim`, 9 builder methods) + `src/engines/cma/mod.rs` skeleton + Nyquist test scaffold with 11 `#[ignore]`-gated stubs

**Wave 3** *(blocked on Wave 2)*
- [x] 56-03-PLAN.md — `CmaEngine` core: private `CmaState`, Jacobi eigendecomposition, Box-Muller sampling, full run() loop with observer hooks (D-06), and un-ignoring of the 7 engine-dependent tests

**Wave 4** *(blocked on Wave 3)*
- [x] 56-04-PLAN.md — `examples/cma_es_rastrigin.rs` + phase verification gate (cargo test + serde + clippy + rustdoc + WASM target)

**UI hint**: no

### Phase 57: PSO Engine
**Goal**: Users can run Particle Swarm Optimization on real-valued black-box optimization problems via a new `PsoEngine<U>` — with `GaObserver` hooks, configurable inertia/cognitive/social coefficients, and WASM-compatible execution
**Depends on**: Phase 56
**Requirements**: None (issue-driven; see issue #255 PSO track)
**Success Criteria** (what must be TRUE):
  1. User can call `PsoEngine::new(config, init_fn, fitness_fn).run()` and receive a `PsoResult<U>` for any chromosome implementing `LinearChromosome` where `U::Gene: RealGene`
  2. User can attach a `GaObserver<U>` and receive `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, and `on_run_end` calls
  3. User can configure inertia weight, cognitive coefficient (`c1`), and social coefficient (`c2`) via `PsoConfiguration` builder methods
  4. `cargo check --target wasm32-unknown-unknown` passes; `cargo run --example pso_rastrigin` converges; all CI gates pass with zero warnings
**Plans:** 4/4 plans complete
**UI hint**: no

Plans:
**Wave 1**
- [x] 57-01-PLAN.md — RealGene::bounds() trait extension + impls + Nyquist test scaffold
**Wave 2** *(blocked on Wave 1)*
- [x] 57-02-PLAN.md — PsoConfiguration + PsoInertia + PsoTopology + PsoEngine skeleton + lib.rs re-exports + LinearDecay test
**Wave 3** *(blocked on Wave 2)*
- [x] 57-03-PLAN.md — PsoEngine::run() full PSO loop (PsoState, velocity update, topology dispatch, absorbing boundary, observer hooks) + 9 engine-runtime tests
**Wave 4** *(blocked on Wave 3)*
- [x] 57-04-PLAN.md — pso_rastrigin example + phase verification gate (cargo test + serde + clippy + rustdoc + WASM)

### Phase 58: EDA / UMDA Engine
**Goal**: Users can run Estimation of Distribution Algorithm (UMDA variant) on binary or categorical optimization problems via a new `EdaEngine<U>` that learns and samples a probabilistic model over the population
**Depends on**: Phase 56
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. User can call `EdaEngine::new(config, init_fn, fitness_fn).run()` and receive an `EdaResult<U>` for any chromosome implementing `LinearChromosome`
  2. The engine estimates a univariate marginal distribution from selected parents and samples offspring from it each generation
  3. User can attach a `GaObserver<U>` and receive all standard lifecycle hooks
  4. `cargo check --target wasm32-unknown-unknown` passes; `cargo run --example eda_onemax` converges; all CI gates pass
**Plans:** 3/3 plans complete

Plans:
**Wave 1**
- [x] 58-01-PLAN.md — EdaConfiguration + module skeleton + lib.rs registration + Nyquist test scaffold

**Wave 2** *(blocked on Wave 1)*
- [x] 58-02-PLAN.md — EdaEngine UMDA run() loops (Bernoulli + Gaussian dispatch) + observer wiring + un-ignore engine runtime tests

**Wave 3** *(blocked on Wave 2)*
- [x] 58-03-PLAN.md — examples/eda_trap.rs + Cargo.toml registration + phase verification gate

**UI hint**: no

### Phase 59: Restart Strategies — IPOP / BIPOP
**Goal**: Users can configure automatic restart strategies (IPOP: increasing population, BIPOP: bi-population alternating) for CmaEngine to escape local optima on multimodal problems
**Depends on**: Phase 56
**Requirements**: None (deferred from Phase 56 per issue #255)
**Success Criteria** (what must be TRUE):
  1. User can configure `RestartStrategy::Ipop { population_scale }` on `CmaConfiguration`; after stagnation, engine restarts with scaled population and fresh covariance
  2. User can configure `RestartStrategy::Bipop`; engine alternates between large and small restarts as in Hansen 2009
  3. `GaObserver::on_restart` hook fires on each restart event (new hook)
  4. `cargo check --target wasm32-unknown-unknown` passes; all CI gates pass
**Plans**: 3 plans
**UI hint**: no

Plans:

**Wave 1**
- [x] 59-01-PLAN.md — Nyquist test stubs + RestartStrategy/RestartEvent/RestartKind types + observer on_restart hook + CmaConfiguration field + CmaResult field + lib.rs re-exports

**Wave 2** *(blocked on Wave 1)*
- [x] 59-02-PLAN.md — CmaEngine outer restart loop (IPOP/BIPOP) + compute_next_lambda + restart_kind helpers + un-ignore CMA-12 through CMA-17

**Wave 3** *(blocked on Wave 2)*
- [x] 59-03-PLAN.md — ipop_rastrigin example + phase verification gate (cargo test + serde + clippy + rustdoc + WASM)

### Phase 60: Batch Fitness / Fitness Cache Extension
**Goal**: Users can evaluate fitness for a batch of chromosomes in a single call (enabling GPU/API-based evaluators) and optionally cache results to avoid redundant re-evaluation of unchanged chromosomes across generations
**Depends on**: Phase 56
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. User can implement `BatchFitnessEvaluator::evaluate_batch(&[U]) -> Vec<f64>` and wire it into `Ga` / `CmaEngine` via a builder method; individual-level `calculate_fitness` is not called when batch evaluator is configured
  2. User can enable `FitnessCache` via a builder flag; chromosomes with unchanged DNA are returned cached fitness without re-evaluation; cache hit rate is exposed in `GenerationStats`
  3. WASM-compatible: no threads or `std::time` required in the cache path
  4. All CI gates pass with zero warnings
**Plans**: 3/3 plans complete

Plans:
**Wave 1**
- [x] 60-01-PLAN.md — BatchFitnessEvaluator trait + wrap_with_cache tuple refactor + GenerationStats cache fields + Wave 0 test stubs

**Wave 2** *(blocked on Wave 1)*
- [x] 60-02-PLAN.md — Ga batch + cache integration: field, builder, mutual-exclusivity check, batch_evaluate_pop helper, run() wiring (initial pop + offspring + delta stats), activate 8 Ga tests

**Wave 3** *(blocked on Wave 2)*
- [x] 60-03-PLAN.md — CMA batch + cache integration (both eval sites + delta stats) + activate 5 CMA tests + phase verification gate (full CI matrix + SUMMARY)

**UI hint**: no

### Phase 61: Performance — Clone Reduction & Parallel Survivor
**Goal**: Systematically reduce unnecessary chromosome clones in the GA hot path and enable parallel survivor selection where the algorithm permits, measurably improving throughput on large populations
**Depends on**: Phase 56
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. Profiling (cargo bench before/after) shows ≥2% wall-time reduction on the `rastrigin` benchmark at population size 500 (amended after bench results — see 61-BENCH-RESULTS.md; original target was ≥10%)
  2. `SurvivorOperator` implementations that are order-independent use `rayon::par_iter` for ranking/scoring (gated behind `#[cfg(not(target_arch = "wasm32"))]`)
  3. No behavioral regression: all existing tests pass with identical outputs (modulo floating-point ordering ties broken deterministically)
  4. `cargo check --target wasm32-unknown-unknown` passes; all CI gates pass
**Plans**: 4/4 plans complete
**UI hint**: no

### Phase 62: Surrogate-Assisted Evaluation
**Goal**: Users can attach a surrogate model (e.g. Gaussian Process or polynomial regression) to pre-screen candidates before expensive fitness evaluation, reducing the number of true fitness calls on costly black-box problems
**Depends on**: Phase 60
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. User can implement `SurrogateModel::predict(&U) -> f64` and attach it via `.with_surrogate(model, prescreening_fraction)`; only the top fraction of surrogate-ranked offspring proceed to true fitness evaluation
  2. True fitness call count is exposed in `GenerationStats` and observable via `GaObserver`
  3. WASM-compatible; all CI gates pass
**Plans**: 3/3 plans complete

Plans:
**Wave 1**
- [x] 62-01-PLAN.md — SurrogateModel trait + module wiring + GenerationStats.true_fitness_calls + Wave 0 test stubs

**Wave 2** *(blocked on Wave 1)*
- [x] 62-02-PLAN.md — Ga<U> surrogate field + with_surrogate() builder + build() validation + prescreening insertion + gen_stats wiring + activated engine tests

**Wave 3** *(blocked on Waves 1-2)*
- [x] 62-03-PLAN.md — examples/surrogate_rastrigin.rs + Cargo.toml registration + phase verification gate (full CI matrix + SUMMARY)

**UI hint**: no

### Phase 63: Visualization — Pareto Front Plotting & Example Images
**Goal**: Users can generate Pareto front plots and fitness-progress charts as PNG/SVG files from multi-objective runs, and all major examples have rendered output images committed to `docs/`
**Depends on**: Phase 56
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. A `visualization` feature flag exposes `plot_pareto_front(population, path)` for 2- and 3-objective problems using `plotters`
  2. `cargo run --example nsga2_zdt1 -- --plot` produces a `docs/images/nsga2_zdt1.png` Pareto front image
  3. All example images are committed to `docs/images/` and linked from `README.md`
  4. Feature compiles and links on WASM (plotters supports wasm32); all CI gates pass
**Plans**: 3 plans
**UI hint**: no

Plans:
**Wave 1**
- [x] 63-01-PLAN.md — visualization module extension (point_series feature, plot_pareto_front_2d/3d, plot_true_fitness_calls, WASM gates, tests, wasm-check CI step)

**Wave 2** *(blocked on Wave 1)*
- [x] 63-02-PLAN.md — --plot blocks in six examples (nsga2_zdt1, spea2_zdt1, sms_emoa_zdt1, ibea_zdt1, nsga3_dtlz2, rastrigin)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 63-03-PLAN.md — Generate 6 PNG images, commit to docs/images/, extend README.md Visualization section

### Phase 64: Test & Doc Quality
**Goal**: Achieve ≥80% line coverage on all engine modules, eliminate all `#[allow(...)]` suppressions in non-generated code, and ensure every public API item has a rustdoc example that compiles under `cargo test --doc`
**Depends on**: Phase 56
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. `cargo llvm-cov --all-features` reports ≥80% line coverage for `src/engines/` and `src/operations/`
  2. Zero `#[allow(dead_code)]`, `#[allow(unused_imports)]`, or `#[allow(clippy::...)]` attributes remain in non-generated source files
  3. Every `pub` item in `src/` has a rustdoc `# Examples` block that compiles via `cargo test --doc`
  4. All CI gates pass with zero warnings
**Plans:** 1/4 plans executed

Plans:
**Wave 0**
- [ ] 64-01-PLAN.md — Coverage baseline + cargo-llvm-cov CI gate (D-01, D-02, D-03, D-04, D-05)

**Wave 1** *(blocked on Wave 0)*
- [ ] 64-02-PLAN.md — Fix all #[allow(...)] suppressions at root cause (D-06, D-07, D-08, D-09, D-10)

**Wave 2** *(blocked on Waves 0-1, parallel with 64-04)*
- [ ] 64-03-PLAN.md — Data-driven coverage tests for lowest-coverage modules (D-05, D-14)

**Wave 2** *(blocked on Wave 1, parallel with 64-03)*
- [ ] 64-04-PLAN.md — Rustdoc # Examples blocks on all user-facing pub items (D-11, D-12, D-13)
**UI hint**: no

### Phase 65: v3.0.0 Migration Guide & Release Notes
**Goal**: Users upgrading from v2.x to v3.0.0 can follow a single authoritative `MIGRATION.md` guide that covers every breaking change with before/after code snippets, compiler error messages, and migration hints; the v3.0.0 CHANGELOG entry summarizes the full milestone for release notes.
**Depends on**: Phase 64
**Requirements**: None
**Success Criteria** (what must be TRUE):
  1. `MIGRATION.md` covers all breaking changes: `ChromosomeT` split, `LinearChromosome` bound requirement, `DeGene → RealGene` rename, `SelectionOperator::select` return-type change, `Mutation` enum variant parameter changes, `StoppingCriteria` flattening, `Reporter` removal, `LimitConfiguration` field removals, `GaConfiguration` accessor methods, `LinearChromosome::default → reset`
  2. Every breaking change entry includes: the old API, the new API, the compiler error a user will see, and the fix
  3. `README.md` links to `MIGRATION.md` in the "Upgrading" section (header banner above the badge)
  4. `CHANGELOG.md` contains a `## [3.0.0]` entry following Keep-a-Changelog with Added / Changed (breaking) / Removed buckets covering all v3 phases (47–65)
  5. All CI gates pass (cargo test, cargo test --features serde, cargo clippy --all-targets -D warnings, cargo doc --no-deps with zero warnings, cargo check --target wasm32-unknown-unknown)
**Plans:** 3 plans

Plans:
**Wave 0**
- [x] 65-01-PLAN.md — Author MIGRATION.md with 10 breaking-change recipes (before / after / compiler error / fix); README upgrade banner link

**Wave 1** *(blocked on Wave 0)*
- [x] 65-02-PLAN.md — Author CHANGELOG `## [3.0.0]` entry aggregating phases 47–65; compare links

**Wave 2** *(blocked on Wave 1)*
- [ ] 65-03-PLAN.md — Final release-gate verification (full CI matrix, examples smoke-run, MIGRATION.md cross-check against actual compiler errors on a representative v2 sample crate)

**UI hint**: no

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
| 25. Directory Restructure | v2.3.0 | 3/3 | Complete   | 2026-06-10 |
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
| 43. Adaptive Operator Selection (AOS) | v2.4.0 | 3/3 | Complete | 2026-05-14 |
| 44. Standard Benchmark Functions Suite | v2.4.0 | 3/3 | Complete    | 2026-05-14 |
| 45. Memetic Algorithm Framework | v2.4.0 | 3 | Pending | — |
| 46. Documentation Refactor | v2.4.0 | 7/7 | Complete    | 2026-05-15 |
| 47. Architecture Audit & ChromosomeT Split | v3.0.0 | 8/8 | Complete | 2026-05-31 |
| 48. New Genotype Types | v3.0.0 | 4/4 | Complete | 2026-05-31 |
| 49. Unified Strategy Trait + Alternative Strategy Engines | v3.0.0 | 4/4 | Complete | 2026-05-31 |
| 50. Lexicase Selection | v3.0.0 | 2/2 | Complete | 2026-05-23 |
| 51. Multi-Parent Crossover + Self-Adaptive Mutation | v3.0.0 | 4/4 | Complete | 2026-05-31 |
| 52. Variable-Length Chromosomes | v3.0.0 | 4/4 | Complete | 2026-05-24 |
| 53. Tree Chromosome + GpGa Engine | v3.0.0 | 4/4 | Complete | 2026-05-25 |
| 54. N-ary Selection / Per-Operator Mutation Params | v3.0.0 | 2/2 | Complete | 2026-05-31 |
| 55. RFC Multi-Valued Fitness (VectorFitness) | v3.0.0 | 6/6 | Complete | 2026-05-31 |
| 56. CMA-ES Engine | v3.0.0 | 4/4 | Complete | 2026-06-01 |
| 57. PSO Engine | v3.0.0 | 4/4 | Complete   | 2026-06-03 |
| 58. EDA / UMDA Engine | v3.0.0 | 3/3 | Complete   | 2026-06-04 |
| 59. Restart Strategies — IPOP / BIPOP | v3.0.0 | 3/3 | Complete    | 2026-06-06 |
| 60. Batch Fitness / Fitness Cache Extension | v3.0.0 | 3/3 | Complete | 2026-06-08 |
| 61. Performance — Clone Reduction & Parallel Survivor | v3.0.0 | 5/4 | Complete    | 2026-06-09 |
| 62. Surrogate-Assisted Evaluation | v3.0.0 | 3/3 | Complete | 2026-06-09 |
| 63. Visualization — Pareto Front Plotting & Example Images | v3.0.0 | 3/3 | Complete   | 2026-06-10 |
| 64. Test & Doc Quality | v3.0.0 | 1/4 | In Progress|  |
| 65. v3.0.0 Migration Guide | v3.0.0 | TBD | Pending | — |
