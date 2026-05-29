# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- ✅ **v2.1.0 — New Examples** — Phases 10-12 (shipped 2026-03-22)
- ✅ **v2.2.0 — Observability & Traceability** — Phases 13-18 (shipped 2026-03-28)
- ✅ **v2.2.1 — Performance Optimizations** — Phases 19-24 (shipped 2026-04-23)
- ✅ **v2.3.0 — Alternative Metaheuristics & Population Models** — Phases 25-29 (shipped 2026-04-27)
- ✅ **v2.4.0 — Observer Integration & New Operators + Advanced Multi-Objective** — Phases 30-46 (shipped 2026-05-18)
- 🚧 **v3.0.0 — Advanced Representations, Alternative Strategies & Architecture Simplification** — Phases 47-53 (in progress)

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
- [x] **Phase 51: Multi-Parent Crossover + Self-Adaptive Mutation** — UNDX, SPX, PCX operators with `RealValued` marker trait; `SelfAdaptive: ChromosomeT` trait; `Mutation::SelfAdaptiveGaussian` with log-normal sigma update (completed 2026-05-23)
- [x] **Phase 52: Variable-Length Chromosomes** — `ChromosomeLength::Variable { min, max }`; `Mutation::Insertion` / `Mutation::Deletion`; `Crossover::VariableLength(AlignmentStrategy)`; parsimony pressure survivor config (completed 2026-05-24)
- [x] **Phase 53: Tree Chromosome + GpGa Engine** — `TreeChromosome: ChromosomeT` supertrait; `GpGa<U>` engine; ramped half-and-half init; subtree crossover + mutation; bloat control; serde with `serde_stacker`; `Display` as expression string (completed 2026-05-25)
- [x] **Phase 54: N-ary Selection + Per-Operator Mutation Params** — Generalize `SelectionOperator::select` to return `Vec<Vec<usize>>` (N-ary groups, #248); replace `mutate(step, sigma)` overloaded signature with typed per-operator params (#249); update all built-in operators and GA loop
- [ ] **Phase 55: RFC Multi-Valued Fitness** — Design and implement `MultiCaseFitness` → first-class `fitness() -> &[f64]` decision (#251); coordinate with MO engines (nsga2/nsga3/moead/spea2/sms_emoa/ibea); document migration impact
- [ ] **Phase 56: CMA-ES Engine** — `CmaEsEngine` under `src/engines/`; covariance matrix adaptation; configurable strategy params; observer hooks; WASM-compatible (#252)
- [ ] **Phase 57: PSO Engine** — `PsoEngine` under `src/engines/`; velocity/position update; gbest/lbest topologies; inertia, cognitive, social coefficients; WASM-compatible (#253)
- [ ] **Phase 58: EDA / UMDA Engine** — `EdaEngine` with UMDA for binary/continuous; probabilistic model build + sample loop; observer hooks; WASM-compatible (#254)
- [ ] **Phase 59: Restart Strategies (IPOP/BIPOP)** — Restart triggers (stagnation, convergence threshold); increasing- and bi-population variants; primarily for CMA-ES; configurable (#255)
- [ ] **Phase 60: Batch Fitness + Fitness Cache Extension** — Optional `fn(&[&[Gene]]) -> Vec<f64>` batch API (#257); extend `src/fitness/cache.rs` to all engines (#260)
- [ ] **Phase 61: Performance — Clone Reduction + Parallel Survivor** — Reusable offspring buffers across generations (#258); rayon-parallel survivor selection and non-dominated sorting with WASM cfg-gates (#259)
- [ ] **Phase 62: Surrogate-Assisted Evaluation** — Pluggable surrogate model (regression/kriging-lite) for expensive fitness; opt-in; overlaps with batch-fitness API (#256)
- [ ] **Phase 63: Visualization — Pareto-Front Plotting + Example Images** — 2D/3D Pareto-front plot behind `visualization` feature (#261); generate rendered example images from real runs (#262); embed images in README and docs (#264)
- [ ] **Phase 64: Test + Doc Quality** — Move remaining inline `#[cfg(test)]` modules to `tests/` (#266); audit and fix `rust,ignore` doctests (#265); add missing criterion benchmarks for nsga3/moead/spea2/sms_emoa/ibea/constraints/niching/memetic (#267)
- [ ] **Phase 65: v3.0.0 Migration Guide** — Complete MIGRATION.md with every breaking change (ConfigurationT decomposition, operator dispatch, N-ary selection, mutation params, Reporter removal, multi-valued fitness); before/after code snippets (#263)

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

### Phase 54: N-ary Selection + Per-Operator Mutation Params

**Goal:** Users can drive both standard 2-parent and N-parent (UNDX/SPX/PCX) crossover from a single unified selection API returning `Vec<Vec<usize>>`, and configure mutation parameters inline on each `Mutation` enum variant instead of through global `MutationConfiguration` fields — a v3.0.0 breaking-change cleanup of the operator layer.
**Requirements**: SEL-NARY-01, MUT-PARAM-01
**Depends on:** Phase 53
**Plans:** 2 plans

Plans:
**Wave 1**
- [x] 54-01-PLAN.md — N-ary selection: SelectionOperator/factory return Vec<Vec<usize>> + group.len() crossover dispatch + island/GP/cellular call sites (SEL-NARY-01)

**Wave 2** *(blocked on Wave 1 — shares ga.rs and traits/operators.rs)*
- [x] 54-02-PLAN.md — Per-operator mutation params: parameterized non-Copy Mutation enum + &Mutation trait + slimmed MutationConfiguration + collapsed GA dispatch (MUT-PARAM-01)

### Phase 55: RFC Multi-Valued Fitness

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 54
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 55 to break down)

### Phase 56: CMA-ES Engine

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 55
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 56 to break down)

### Phase 57: PSO Engine

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 56
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 57 to break down)

### Phase 58: EDA / UMDA Engine

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 57
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 58 to break down)

### Phase 59: Restart Strategies (IPOP/BIPOP)

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 58
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 59 to break down)

### Phase 60: Batch Fitness + Fitness Cache Extension

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 59
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 60 to break down)

### Phase 61: Performance Clone Reduction + Parallel Survivor

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 60
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 61 to break down)

### Phase 62: Surrogate-Assisted Evaluation

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 61
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 62 to break down)

### Phase 63: Visualization Pareto-Front Plotting + Example Images

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 62
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 63 to break down)

### Phase 64: Test + Doc Quality

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 63
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 64 to break down)

### Phase 65: v3.0.0 Migration Guide

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 64
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 65 to break down)

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
**Plans:** 4/4 plans complete
**UI hint**: no

Plans:
**Wave 1**
- [x] 51-01-PLAN.md — Wave 0 test stubs + RealValued/SelfAdaptive traits + enum variants + MutationConfiguration fields + RangeChromosome RealValued/SelfAdaptive impls + MultiRangeChromosome RealValued stub (TRAITS-02, CRS-02, CRS-03, CRS-04, MUT-05)

**Wave 2** *(parallel — disjoint files; both depend on Wave 1)*
- [x] 51-02-PLAN.md — UNDX, SPX, PCX operator implementations + factory_multi_parent dispatcher (CRS-02, CRS-03, CRS-04)
- [x] 51-03-PLAN.md — SelfAdaptiveGaussian operator + mutation.rs Mutation::SelfAdaptiveGaussian dispatch (MUT-05, TRAITS-02)

**Wave 3** *(blocked on Waves 1-2)*
- [x] 51-04-PLAN.md — ga.rs multi-parent dispatch branch + 1-vs-2 offspring handling + integration tests + phase verification gate + human checkpoint (CRS-02, CRS-03, CRS-04, MUT-05)

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

Plans:
**Wave 0**
- [ ] 52-01-PLAN.md — Wave 0 test stubs for MUT-06, CHR-01, CHR-02

**Wave 1** *(blocked on Wave 0)*
- [ ] 52-02-PLAN.md — Mutation enum rename (PermutationInsert) + Insertion/Deletion operators + factory_variable_length (MUT-06)

**Wave 2** *(blocked on Wave 1)*
- [ ] 52-03-PLAN.md — AlignmentStrategy enum + Crossover::VariableLength + check_compatible_length guard on all fixed operators (CHR-01)

**Wave 3** *(blocked on Wave 2)*
- [ ] 52-04-PLAN.md — length_penalty field + survivor parsimony + ga.rs Variable init/regrowth unlock + validator (CHR-01, CHR-02)

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
| 43. Adaptive Operator Selection (AOS) | v2.4.0 | 3/3 | Complete | 2026-05-14 |
| 44. Standard Benchmark Functions Suite | v2.4.0 | 3/3 | Complete    | 2026-05-14 |
| 45. Memetic Algorithm Framework | v2.4.0 | 3 | Pending | — |
| 46. Documentation Refactor | v2.4.0 | 7/7 | Complete    | 2026-05-15 |
| 47. Architecture Audit & ChromosomeT Split | v3.0.0 | 6/8 | In Progress|  |
| 48. New Genotype Types | v3.0.0 | 4 | In Progress | — |
| 49. Unified Strategy Trait + Alternative Strategy Engines | v3.0.0 | 0 | Not started | — |
| 50. Lexicase Selection | v3.0.0 | 2/2 | Complete   | 2026-05-23 |
| 51. Multi-Parent Crossover + Self-Adaptive Mutation | v3.0.0 | 4/4 | Complete   | 2026-05-23 |
| 52. Variable-Length Chromosomes | v3.0.0 | 4/4 | Complete   | 2026-05-24 |
| 53. Tree Chromosome + GpGa Engine | v3.0.0 | 4/4 | Complete   | 2026-05-25 |
