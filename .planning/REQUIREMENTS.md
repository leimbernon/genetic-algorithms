# Requirements: genetic_algorithms

**Defined:** 2026-03-21
**Core Value:** The simplest correct way to run a genetic algorithm in Rust — generic enough for any problem domain, fast enough for real workloads.

## v2.1.0 Requirements

### Examples

- [ ] **EX-01**: User can run a Rastrigin continuous optimization example using `Range<f64>` chromosomes and gaussian/creep mutation operators
- [ ] **EX-02**: User can run an NSGA-II multi-objective example optimizing the ZDT1 benchmark (two conflicting objectives)
- [ ] **EX-03**: User can run an Island Model GA example with multiple sub-populations evolving in parallel with migration
- [ ] **EX-04**: User can run a Job Scheduling example minimizing makespan across machines via permutation-based chromosome representation
- [ ] **EX-05**: User can run a Feature Selection example using Binary chromosomes with adaptive GA to select optimal ML feature subsets
- [ ] **EX-06**: User can run a Niching / Fitness Sharing example that maintains multiple solutions in a multimodal optimization landscape

### Documentation

- [ ] **DOC-01**: README documents all available examples with a brief purpose description and the corresponding `cargo run --example <name>` command

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
| New operators or chromosome types | Separate New Operators milestone |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| EX-01 | — | Pending |
| EX-02 | — | Pending |
| EX-03 | — | Pending |
| EX-04 | — | Pending |
| EX-05 | — | Pending |
| EX-06 | — | Pending |
| DOC-01 | — | Pending |

**Coverage:**
- v2.1.0 requirements: 7 total
- Mapped to phases: 0
- Unmapped: 7 ⚠️

---
*Requirements defined: 2026-03-21*
