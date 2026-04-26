# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- ✅ **v2.1.0 — New Examples** — Phases 10-12 (shipped 2026-03-22)
- ✅ **v2.2.0 — Observability & Traceability** — Phases 13-18 (shipped 2026-03-28)
- ✅ **v2.2.1 — Performance Optimizations** — Phases 19-24 (shipped 2026-04-23)
- [ ] **v2.3.0 — Alternative Metaheuristics & Population Models** — Phases 25-29

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

### v2.3.0 — Alternative Metaheuristics & Population Models

- [ ] **Phase 25: Directory Restructure** — Non-breaking reorganization of src/ into engines/, types/, observe/
- [ ] **Phase 26: Differential Evolution Engine** — 5 mutation strategies, binomial/exponential crossover, JADE and L-SHADE adaptive variants
- [ ] **Phase 27: Scatter Search Engine** — Diversification, reference set management, combination, optional local search
- [ ] **Phase 28: Cellular GA Engine** — 2D toroidal grid, 4 neighborhood types, synchronous/asynchronous update modes
- [ ] **Phase 29: ALPS Engine** — Age-layered population, 3 age schemes, cross-layer mating, periodic injection

## Phase Details

### Phase 25: Directory Restructure
**Goal**: The src/ tree is logically organized into engines/, types/, and observe/ groups with no impact on downstream users
**Depends on**: Nothing (first phase of v2.3.0)
**Requirements**: STRUCT-01, STRUCT-02, STRUCT-03, STRUCT-04
**Success Criteria** (what must be TRUE):
  1. A downstream user upgrading to v2.3.0 sees no compiler errors — all existing `use genetic_algorithms::...` paths resolve identically
  2. `cargo test` and `cargo test --features serde` pass with zero failures after the restructure
  3. `cargo clippy` reports zero warnings and `cargo doc --no-deps` reports zero rustdoc warnings
  4. src/engines/ contains ga, island, nsga2 (and placeholder dirs for de, scatter, cellular, alps); src/types/ contains chromosomes and genotypes; src/observe/ contains observer, reporter, visualization, checkpoint
**Plans**: 3 plans
Plans:
- [ ] 25-01-PLAN.md — Move chromosomes and genotypes into src/types/
- [ ] 25-02-PLAN.md — Move observer, reporter, visualization, checkpoint into src/observe/
- [ ] 25-03-PLAN.md — Move ga, island, nsga2 into src/engines/ with placeholder stubs

### Phase 26: Differential Evolution Engine
**Goal**: Users can run Differential Evolution with any of 5 mutation strategies and 2 crossover modes, including adaptive JADE and L-SHADE variants
**Depends on**: Phase 25
**Requirements**: DE-01, DE-02, DE-03, DE-04, DE-05, DE-06, DE-07
**Success Criteria** (what must be TRUE):
  1. User can construct a `DeEngine` and run it with any of the 5 mutation strategies (rand/1, best/1, current-to-best/1, rand/2, best/2) using any `ChromosomeT + GeneT` type
  2. User can switch between binomial and exponential crossover modes via configuration
  3. User can opt into the JADE variant and observe self-adaptive F and CR parameters updating each generation
  4. User can opt into the L-SHADE variant and observe historical memory for F and CR influencing parameter draws
  5. `cargo test --test de` passes; `cargo bench --bench de` runs to completion and prints a comparison result
**Plans**: 3 plans
Plans:
- [ ] 25-01-PLAN.md — Move chromosomes and genotypes into src/types/
- [ ] 25-02-PLAN.md — Move observer, reporter, visualization, checkpoint into src/observe/
- [ ] 25-03-PLAN.md — Move ga, island, nsga2 into src/engines/ with placeholder stubs

### Phase 27: Scatter Search Engine
**Goal**: Users can run Scatter Search with a configurable diversification phase, reference set, combination step, and optional local search
**Depends on**: Phase 25
**Requirements**: SCAT-01, SCAT-02, SCAT-03, SCAT-04, SCAT-05, SCAT-06, SCAT-07
**Success Criteria** (what must be TRUE):
  1. User can construct a `ScatterEngine` with a configurable reference set size and run it on any `ChromosomeT + GeneT` type
  2. The engine automatically generates a diverse initial solution set and maintains the reference set across iterations
  3. The engine combines reference set solutions to produce new candidates each iteration
  4. User can enable optional local search post-processing via a configuration flag; the engine applies it to candidates when enabled
  5. `cargo test --test scatter` passes; `cargo bench --bench scatter` runs to completion
**Plans**: 3 plans
Plans:
- [ ] 25-01-PLAN.md — Move chromosomes and genotypes into src/types/
- [ ] 25-02-PLAN.md — Move observer, reporter, visualization, checkpoint into src/observe/
- [ ] 25-03-PLAN.md — Move ga, island, nsga2 into src/engines/ with placeholder stubs

### Phase 28: Cellular GA Engine
**Goal**: Users can run a Cellular GA on a 2D toroidal grid with their choice of neighborhood topology and update mode
**Depends on**: Phase 25
**Requirements**: CELL-01, CELL-02, CELL-03, CELL-04, CELL-05, CELL-06
**Success Criteria** (what must be TRUE):
  1. User can construct a `CellularEngine` with configurable grid dimensions and run it on any `ChromosomeT + GeneT` type using existing selection/crossover/mutation operators
  2. User can select any of the 4 neighborhood types (von Neumann 4-cell, Moore 8-cell, compact r=2 25-cell, linear) and the engine applies the correct neighbor set during evolution
  3. User can choose synchronous or asynchronous update mode; in synchronous mode all cells update from the previous generation's state
  4. `cargo test --test cellular` passes covering all 4 neighborhoods and both update modes; `cargo bench --bench cellular` runs to completion
**Plans**: 3 plans
Plans:
- [ ] 25-01-PLAN.md — Move chromosomes and genotypes into src/types/
- [ ] 25-02-PLAN.md — Move observer, reporter, visualization, checkpoint into src/observe/
- [ ] 25-03-PLAN.md — Move ga, island, nsga2 into src/engines/ with placeholder stubs
**UI hint**: no

### Phase 29: ALPS Engine
**Goal**: Users can run ALPS with age-layered populations, configurable age schemes, cross-layer mating, and periodic fresh-individual injection
**Depends on**: Phase 25
**Requirements**: ALPS-01, ALPS-02, ALPS-03, ALPS-04, ALPS-05, ALPS-06, ALPS-07
**Success Criteria** (what must be TRUE):
  1. User can construct an `AlpsEngine` with a configurable number of layers and run it on any `ChromosomeT + GeneT` type using existing operators
  2. User can select any of the 3 age schemes (linear, Fibonacci, polynomial) and the engine assigns individuals to layers accordingly
  3. Individuals from adjacent layers mate when age permits; the engine enforces cross-layer eligibility each generation
  4. The engine periodically injects fresh random individuals into the youngest layer at a user-configurable interval
  5. `cargo test --test alps` passes covering age schemes, cross-layer mating, and injection; `cargo bench --bench alps` runs to completion and prints a comparison against standard GA
**Plans**: 3 plans
Plans:
- [ ] 25-01-PLAN.md — Move chromosomes and genotypes into src/types/
- [ ] 25-02-PLAN.md — Move observer, reporter, visualization, checkpoint into src/observe/
- [ ] 25-03-PLAN.md — Move ga, island, nsga2 into src/engines/ with placeholder stubs

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1–5. Usability (partial) | v2.1 | — | Complete | 2026-03-20 |
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
| 25. Directory Restructure | v2.3.0 | 0/3 | Not started | - |
| 26. Differential Evolution Engine | v2.3.0 | 0/TBD | Not started | - |
| 27. Scatter Search Engine | v2.3.0 | 0/TBD | Not started | - |
| 28. Cellular GA Engine | v2.3.0 | 0/TBD | Not started | - |
| 29. ALPS Engine | v2.3.0 | 0/TBD | Not started | - |
