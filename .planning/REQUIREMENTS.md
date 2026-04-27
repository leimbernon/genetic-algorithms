# Requirements — v2.4.0 Observer Integration & New Operators

## v1 Requirements

### OBS — Observer Integration

- [ ] **OBS-01**: User can attach a `GaObserver` to `DeEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-02**: User can attach a `GaObserver` to `ScatterEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-03**: User can attach a `GaObserver` to `CellularEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-04**: User can attach a `GaObserver` to `AlpsEngine` and receive `on_start`, `on_finish`, `on_new_best`, `on_generation_start`, `on_generation_end` lifecycle hooks
- [ ] **OBS-05**: User can run `cargo bench --bench de` to compare DE vs GA convergence on a shared benchmark function

### SEL — Selection Operators

- [ ] **SEL-01**: User can configure Clearing selection to promote diversity by clearing dominated individuals within a configurable niche radius

### SRV — Survivor Strategies

- [ ] **SRV-01**: User can configure Deterministic Crowding as a survivor strategy, pairing each offspring with its most similar parent for replacement decisions

### CRS — Crossover Operators

- [ ] **CRS-01**: User can configure Edge Recombination crossover for permutation chromosomes, preserving adjacency relationships from both parents

### MUT — Mutation Operators

- [ ] **MUT-01**: User can configure Cauchy mutation to apply heavy-tailed perturbations to real-valued genes with a configurable scale parameter
- [ ] **MUT-02**: User can configure Lévy Flight mutation to apply long-range jumps to real-valued genes with a configurable stability index
- [ ] **MUT-03**: User can configure Uniform mutation to randomly reset gene values uniformly within the gene's valid range
- [ ] **MUT-04**: User can configure Differential mutation (DE-style) in the standard GA, using three random population members to generate a mutant vector with configurable F scale factor

## Future Requirements

<!-- Validated direction, not yet scheduled. -->

- Advanced Multi-Objective: NSGA-III, MOEA/D, SPEA2 — issues #203–#207
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
| OBS-01 | — | — |
| OBS-02 | — | — |
| OBS-03 | — | — |
| OBS-04 | — | — |
| OBS-05 | — | — |
| SEL-01 | — | — |
| SRV-01 | — | — |
| CRS-01 | — | — |
| MUT-01 | — | — |
| MUT-02 | — | — |
| MUT-03 | — | — |
| MUT-04 | — | — |
