# Requirements: genetic_algorithms

**Defined:** 2026-04-26
**Core Value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library.

## v2.3.0 Requirements

Requirements for this milestone. Each maps to a roadmap phase.

### Directory Restructure (STRUCT)

- [x] **STRUCT-01**: `src/engines/` groups all engine modules (ga, island, nsga2, de, scatter, cellular, alps); lib.rs re-exports preserve all existing public paths
- [x] **STRUCT-02**: `src/types/` groups chromosomes and genotypes modules; lib.rs re-exports preserve all existing public paths
- [x] **STRUCT-03**: `src/observe/` groups observer, reporter, visualization, and checkpoint modules; lib.rs re-exports preserve all existing public paths
- [x] **STRUCT-04**: All existing tests pass after restructure (`cargo test`, `cargo test --features serde`, `cargo clippy`, zero rustdoc warnings)

### Differential Evolution (DE)

- [x] **DE-01**: User can run Differential Evolution with 5 mutation strategies: rand/1, best/1, current-to-best/1, rand/2, best/2
- [x] **DE-02**: User can configure binomial or exponential crossover for DE
- [x] **DE-03**: User can run JADE adaptive variant with self-adaptive F and CR parameters
- [x] **DE-04**: User can run L-SHADE adaptive variant with historical memory for F and CR
- [x] **DE-05**: DE engine accepts any type implementing `ChromosomeT` + `GeneT` (consistent with existing engine pattern)
- [x] **DE-06**: Unit tests in `tests/` cover all 5 mutation strategies and both crossover modes
- [x] **DE-07**: Criterion benchmark compares DE convergence on a continuous optimization problem (sphere/rastrigin) — mutation strategy comparison benchmark delivered; DE-vs-GA head-to-head deferred

### Scatter Search (SCAT)

- [x] **SCAT-01**: User can run Scatter Search with a configurable diversification phase that generates an initial diverse solution set
- [x] **SCAT-02**: User can configure reference set size; engine manages reference set updates automatically
- [x] **SCAT-03**: Engine combines reference set solutions to generate new candidate solutions
- [x] **SCAT-04**: User can optionally enable local search as a post-processing step on candidate solutions
- [x] **SCAT-05**: Scatter Search engine accepts any type implementing `ChromosomeT` + `GeneT`
- [x] **SCAT-06**: Unit tests in `tests/` cover diversification phase, reference set management, and combination logic
- [x] **SCAT-07**: Criterion benchmark measures Scatter Search performance on standard test functions

### Cellular GA (CELL)

- [x] **CELL-01**: User can run Cellular GA on a 2D toroidal grid with configurable dimensions
- [x] **CELL-02**: User can choose from 4 neighborhood types: von Neumann (4-cell), Moore (8-cell), compact (r=2, 25-cell), linear
- [x] **CELL-03**: User can choose synchronous or asynchronous update mode
- [x] **CELL-04**: Cellular GA accepts any type implementing `ChromosomeT` + `GeneT` and reuses existing selection/crossover/mutation operators
- [x] **CELL-05**: Unit tests in `tests/` cover each neighborhood type and both update modes
- [x] **CELL-06**: Criterion benchmark compares all 4 neighborhoods and sync vs async throughput on sphere function

### ALPS (ALPS)

- [x] **ALPS-01**: User can run ALPS with age-layered population structure and configurable number of layers
- [x] **ALPS-02**: User can choose from 3 age schemes: linear, Fibonacci, polynomial
- [x] **ALPS-03**: Engine supports cross-layer mating between adjacent layers based on individual age
- [x] **ALPS-04**: Engine periodically injects fresh random individuals into the youngest layer
- [x] **ALPS-05**: ALPS engine accepts any type implementing `ChromosomeT` + `GeneT` and reuses existing operators
- [x] **ALPS-06**: Unit tests in `tests/` cover each age scheme, cross-layer mating logic, and injection
- [x] **ALPS-07**: Criterion benchmark compares ALPS vs DE on sphere function; all 3 age schemes benchmarked

## Future Requirements

Acknowledged but deferred to a future milestone.

### New Operators

- **OPS-01**: Additional selection operators (#196–#202)
- **OPS-02**: Additional crossover operators
- **OPS-03**: Additional mutation operators

### Framework Extensions

- **FW-01**: Constraint handling (#212–#219)
- **FW-02**: Memetic algorithms
- **FW-03**: Warm start
- **FW-04**: Adaptive operator selection (AOS)

### Advanced Multi-Objective

- **MOO-01**: NSGA-III (#203–#207)
- **MOO-02**: MOEA/D
- **MOO-03**: SPEA2

## Out of Scope

| Feature | Reason |
|---------|--------|
| Per-gene observer hooks in new engines | Too granular; unacceptable overhead in hot loops |
| GUI / interactive visualization for new engines | Library generates static PNG/SVG; dashboards are users' concern |
| Feature flags gating new engines | Always compiled; consistent with existing ga/island/nsga2 pattern |
| Breaking public API changes | lib.rs re-exports preserve all existing paths; no semver bump |
| Parallel grid-cell updates in Cellular GA | Rayon applied at operator level; per-cell parallelism deferred |
| Specific telemetry backends (Prometheus, Jaeger) | Facade pattern lets users pick their own backend |

## Traceability

Populated by roadmapper. Updated as phases complete.

| Requirement | Phase | Status |
|-------------|-------|--------|
| STRUCT-01 | Phase 25 | Complete |
| STRUCT-02 | Phase 25 | Complete |
| STRUCT-03 | Phase 25 | Complete |
| STRUCT-04 | Phase 25 | Complete |
| DE-01 | Phase 26 | Complete |
| DE-02 | Phase 26 | Complete |
| DE-03 | Phase 26 | Complete |
| DE-04 | Phase 26 | Complete |
| DE-05 | Phase 26 | Complete |
| DE-06 | Phase 26 | Complete |
| DE-07 | Phase 26 | Complete |
| SCAT-01 | Phase 27 | Complete |
| SCAT-02 | Phase 27 | Complete |
| SCAT-03 | Phase 27 | Complete |
| SCAT-04 | Phase 27 | Complete |
| SCAT-05 | Phase 27 | Complete |
| SCAT-06 | Phase 27 | Complete |
| SCAT-07 | Phase 27 | Complete |
| CELL-01 | Phase 28 | Complete |
| CELL-02 | Phase 28 | Complete |
| CELL-03 | Phase 28 | Complete |
| CELL-04 | Phase 28 | Complete |
| CELL-05 | Phase 28 | Complete |
| CELL-06 | Phase 28 | Complete |
| ALPS-01 | Phase 29 | Complete |
| ALPS-02 | Phase 29 | Complete |
| ALPS-03 | Phase 29 | Complete |
| ALPS-04 | Phase 29 | Complete |
| ALPS-05 | Phase 29 | Complete |
| ALPS-06 | Phase 29 | Complete |
| ALPS-07 | Phase 29 | Complete |

**Coverage:**
- v2.3.0 requirements: 31 total
- Mapped to phases: 31
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-26*
*Last updated: 2026-04-27 — all 31 requirements marked Complete at v2.3.0 milestone close*
