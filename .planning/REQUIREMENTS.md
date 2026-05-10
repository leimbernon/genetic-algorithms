# Requirements — v2.4.0 Observer Integration, New Operators & Advanced Multi-Objective

## v1 Requirements

### OBS — Observer Integration

- [ ] **OBS-01**: User can attach a `GaObserver` to `DeEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-02**: User can attach a `GaObserver` to `ScatterEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-03**: User can attach a `GaObserver` to `CellularEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-04**: User can attach a `GaObserver` to `AlpsEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-05**: User can run `cargo bench --bench de` to compare DE vs GA convergence on a shared benchmark function

### SEL — Selection Operators

- [x] **SEL-01**: User can configure Clearing selection to promote diversity by clearing dominated individuals within a configurable niche radius

### SRV — Survivor Strategies

- [x] **SRV-01**: User can configure Deterministic Crowding as a survivor strategy, pairing each offspring with its most similar parent for replacement decisions

### CRS — Crossover Operators

- [ ] **CRS-01**: User can configure Edge Recombination crossover for permutation chromosomes, preserving adjacency relationships from both parents

### MUT — Mutation Operators

- [x] **MUT-01**: User can configure Cauchy mutation to apply heavy-tailed perturbations to real-valued genes with a configurable scale parameter
- [x] **MUT-02**: User can configure Lévy Flight mutation to apply long-range jumps to real-valued genes with a configurable stability index
- [x] **MUT-03**: User can configure Uniform mutation to randomly reset gene values uniformly within the gene's valid range
- [ ] **MUT-04**: User can configure Differential mutation (DE-style) in the standard GA, using three random population members to generate a mutant vector with configurable F scale factor

### MOO — Advanced Multi-Objective Optimization

- [x] **MOO-01**: User can run NSGA-III on problems with 3+ objectives; reference points are auto-generated (Das-Dennis simplex lattice) or user-supplied, and the algorithm selects survivors via reference-point association rather than crowding distance (#203)
- [x] **MOO-02**: User can run MOEA/D with configurable weight vectors and either Tchebycheff or PBI scalarisation; each sub-problem maintains a neighbourhood of similar weight vectors and offspring compete only within that neighbourhood (#204)
- [x] **MOO-03**: User can run SPEA2 with a configurable archive size; fitness is computed from raw strength + density (k-nearest-neighbour), and the archive is truncated using the Euclidean crowding criterion (#205)
- [ ] **MOO-04**: User can run SMS-EMOA (hypervolume contribution-based steady-state removal) and IBEA (additive epsilon-indicator fitness); both share the quality-indicator library (#206)
- [ ] **MOO-05**: User can compute Hypervolume, Generational Distance (GD), Inverted GD (IGD), and Spread from any set of Pareto-front solutions via a shared quality-indicator module (#207)

## Future Requirements

<!-- Validated direction, not yet scheduled. -->

- Framework extensions: constraint handling, memetic algorithms, warm start, AOS — issues #212–#219

## Out of Scope

- Full DE crossover operators for standard GA — Differential mutation composes with existing crossover operators; a separate DE crossover adds complexity without clear user benefit
- Per-engine observer sub-traits (e.g. `DeEngineObserver`) — standard `GaObserver<U>` provides sufficient hooks; sub-traits would fragment the observer API
- GUI/interactive visualization — library generates static PNG/SVG charts only
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick

## Traceability

<!-- Filled by roadmapper -->

| REQ-ID | Phase | Plan |
|--------|-------|------|
| OBS-01 | Phase 30 | — |
| OBS-02 | Phase 30 | — |
| OBS-03 | Phase 30 | — |
| OBS-04 | Phase 30 | — |
| OBS-05 | Phase 30 | — |
| SEL-01 | Phase 31 | — |
| SRV-01 | Phase 31 | — |
| CRS-01 | Phase 32 | — |
| MUT-04 | Phase 32 | — |
| MUT-01 | Phase 33 | 33-01-PLAN.md |
| MUT-02 | Phase 33 | 33-02-PLAN.md |
| MUT-03 | Phase 33 | 33-03-PLAN.md |
| MOO-01 | Phase 35 | — |
| MOO-02 | Phase 36 | — |
| MOO-03 | Phase 37 | — |
| MOO-04 | Phase 38 | — |
| MOO-05 | Phase 39 | — |
