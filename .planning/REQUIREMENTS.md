# Requirements: genetic_algorithms

**Defined:** 2026-03-20
**Core Value:** The simplest correct way to run a genetic algorithm in Rust — generic enough for any problem domain, fast enough for real workloads.

## v2.2 Requirements

### Diversity Estimation

- [ ] **DIV-01**: User can read a diversity metric from per-generation statistics
- [ ] **DIV-02**: Extension strategies use the diversity metric to determine when to trigger
- [ ] **DIV-03**: Dynamic mutation probability uses the diversity metric for adjustment decisions

### List Genotype

- [ ] **LIST-01**: User can define a `List<T>` gene drawn from a finite allele set
- [ ] **LIST-02**: User can create a `List<T>` chromosome compatible with `ChromosomeT`
- [ ] **LIST-03**: List chromosomes work with all existing selection, crossover, mutation, and survivor operators
- [ ] **LIST-04**: User can initialize a List population with a built-in initializer

### Visualization

- [ ] **VIZ-01**: User can plot fitness over generations (best, worst, average) to PNG/SVG
- [ ] **VIZ-02**: User can plot population diversity over generations to PNG/SVG
- [ ] **VIZ-03**: User can plot fitness distribution at a given generation to PNG/SVG
- [ ] **VIZ-04**: Visualization is only available when the `visualization` feature flag is enabled

### Reporter

- [ ] **REP-01**: User can attach a reporter to `Ga` via `.with_reporter()` that receives lifecycle hooks (`on_start`, `on_generation_complete`, `on_new_best`, `on_finish`)
- [ ] **REP-02**: Default (no reporter configured) has zero overhead via `NoopReporter`
- [ ] **REP-03**: Built-in `SimpleReporter` logs progress to stdout every N generations
- [ ] **REP-04**: Built-in `DurationReporter` reports per-phase timing breakdown

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
| DIV-01 | — | Pending |
| DIV-02 | — | Pending |
| DIV-03 | — | Pending |
| LIST-01 | — | Pending |
| LIST-02 | — | Pending |
| LIST-03 | — | Pending |
| LIST-04 | — | Pending |
| VIZ-01 | — | Pending |
| VIZ-02 | — | Pending |
| VIZ-03 | — | Pending |
| VIZ-04 | — | Pending |
| REP-01 | — | Pending |
| REP-02 | — | Pending |
| REP-03 | — | Pending |
| REP-04 | — | Pending |

**Coverage:**
- v2.2 requirements: 15 total
- Mapped to phases: 0
- Unmapped: 15 ⚠️

---
*Requirements defined: 2026-03-20*
*Last updated: 2026-03-20 after initial definition*
