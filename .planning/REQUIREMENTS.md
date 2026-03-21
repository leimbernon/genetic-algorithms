# Requirements: genetic_algorithms

**Defined:** 2026-03-20
**Core Value:** The simplest correct way to run a genetic algorithm in Rust — generic enough for any problem domain, fast enough for real workloads.

## v2.2 Requirements

### Diversity Estimation

- [x] **DIV-01**: User can read a diversity metric from per-generation statistics
- [x] **DIV-02**: Extension strategies use the diversity metric to determine when to trigger
- [x] **DIV-03**: Dynamic mutation probability uses the diversity metric for adjustment decisions

### List Genotype

- [x] **LIST-01**: User can define a `List<T>` gene drawn from a finite allele set
- [x] **LIST-02**: User can create a `List<T>` chromosome compatible with `ChromosomeT`
- [x] **LIST-03**: List chromosomes work with all existing selection, crossover, mutation, and survivor operators
- [x] **LIST-04**: User can initialize a List population with a built-in initializer

### Visualization

- [ ] **VIZ-01**: User can plot fitness over generations (best, worst, average) to PNG/SVG
- [ ] **VIZ-02**: User can plot population diversity over generations to PNG/SVG
- [ ] **VIZ-03**: User can plot fitness distribution at a given generation to PNG/SVG
- [ ] **VIZ-04**: Visualization is only available when the `visualization` feature flag is enabled

### Reporter

- [x] **REP-01**: User can attach a reporter to `Ga` via `.with_reporter()` that receives lifecycle hooks (`on_start`, `on_generation_complete`, `on_new_best`, `on_finish`)
- [x] **REP-02**: Default (no reporter configured) has zero overhead via `NoopReporter`
- [x] **REP-03**: Built-in `SimpleReporter` logs progress to stdout every N generations
- [x] **REP-04**: Built-in `DurationReporter` reports wall-clock timing summary (total elapsed and per-generation average)

## Future Requirements

### Alternative Strategies (#172–#177)

- **STRAT-01**: HillClimb strategy
- **STRAT-02**: Permutate strategy
- **STRAT-03**: Unique genotype
- **STRAT-04**: MultiRange genotype
- **STRAT-05**: MultiUnique genotype
- **STRAT-06**: Unified strategy trait

## Out of Scope

| Feature | Reason |
|---------|--------|
| Breaking API changes | Deferred to v3.0+ Advanced Representations milestone |
| NSGA-III / MOEA/D / SPEA2 | Separate Advanced Multi-Objective milestone |
| Differential Evolution engine | Separate Alt. Metaheuristics milestone |
| Observer/tracing system | Separate Observability milestone (#182–#186) |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DIV-01 | Phase 6 | Complete |
| DIV-02 | Phase 6 | Complete |
| DIV-03 | Phase 6 | Complete |
| LIST-01 | Phase 7 | Complete |
| LIST-02 | Phase 7 | Complete |
| LIST-03 | Phase 7 | Complete |
| LIST-04 | Phase 7 | Complete |
| REP-01 | Phase 8 | Complete |
| REP-02 | Phase 8 | Complete |
| REP-03 | Phase 8 | Complete |
| REP-04 | Phase 8 | Complete |
| VIZ-01 | Phase 9 | Pending |
| VIZ-02 | Phase 9 | Pending |
| VIZ-03 | Phase 9 | Pending |
| VIZ-04 | Phase 9 | Pending |

**Coverage:**
- v2.2 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
*Requirements defined: 2026-03-20*
*Last updated: 2026-03-21 — REP-04 updated to reflect wall-clock timing summary (per-operator breakdown deferred to Observability milestone)*
