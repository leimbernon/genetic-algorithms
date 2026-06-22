# Phase 74: Add Missing Engine and Feature Benchmarks - Context

**Gathered:** 2026-06-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Add divan benchmarks for every engine and major feature that currently has no coverage in `benches/`. The result: PSO, CMA-ES, EDA, and GP each get a dedicated bench file, and AOS, surrogate-assisted evaluation, and batch fitness each get a dedicated bench file. `cargo bench --no-run` must compile all new benchmarks cleanly.

**Already covered — no action needed:**
- ALPS → `benches/alps.rs`
- Island GA → `benches/island_ga.rs`

**Out of scope:**
- Benchmarks for operators already covered (selection, crossover, mutation, survivor)
- Benchmarks for NSGA-2/3, MOEA/D, SPEA2, SMS-EMOA, IBEA (multi-objective engines)
- Criterion migration (codebase uses divan; ROADMAP.md says "Criterion" but that's a documentation error — divan is the framework)
- Any source code changes to engine implementations

</domain>

<decisions>
## Implementation Decisions

### AOS / Surrogate / Batch fitness structure
- **D-01:** Each feature gets its own bench file: `benches/aos.rs`, `benches/surrogate.rs`, `benches/batch_fitness.rs`. Mirrors the `benches/metrics_observer.rs` pattern.
- **D-02:** AOS benchmark measures on-vs-off overhead: two groups — GA with AOS enabled vs. GA without AOS, same problem (Rastrigin 10D), same population size.
- **D-03:** Surrogate benchmark measures throughput: surrogate-assisted GA (cheap model replaces most fitness calls) vs. plain GA on a slow-fitness problem.
- **D-04:** Batch fitness benchmark measures throughput: batch evaluator (all chromosomes in one call) vs. individual `FitnessFnWrapper`. Two groups at the same population sizes.

### GP benchmark design
- **D-05:** Problem: symbolic regression — evolve a tree that approximates a target function (e.g. `f(x) = x^2 + x + 1`). Standard GP benchmark that directly exercises `GpGa`'s intended use case.
- **D-06:** Benchmark axis: population size — groups `pop_50`, `pop_200`, `pop_500`. The `genes_N` dimension pattern does not apply to tree chromosomes; population size is the natural scaling axis.

### Problem selection for PSO / CMA-ES / EDA
- **D-07:** Both sphere and Rastrigin as benchmark problems. Sphere (convex, trivial) and Rastrigin (multimodal, hard) are already used in `benches/alps.rs` and `benches/rastrigin.rs` — using the same problems enables cross-engine comparison.
- **D-08:** Dimension groups: `dims_10`, `dims_30`, `dims_100`. Matches standard continuous optimization benchmark practice and stays within CI-friendly runtimes.

### Benchmark framework
- **D-09:** Use divan throughout. The `[[bench]]` entries in `Cargo.toml` all use `harness = false` with divan. Do not introduce criterion.

### Claude's Discretion
- Exact population size and generation count per engine (keep small enough that `cargo bench --no-run` plus a quick single-iteration smoke passes fast)
- Whether to add `required-features = ["benchmarks"]` on feature bench entries (follow the `de.rs` precedent if the feature requires non-default crate features)
- Exact `Cargo.toml` `[[bench]]` ordering for new entries

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing benchmark files to follow as patterns
- `benches/alps.rs` — engine bench with sphere fitness, multi-engine groups
- `benches/de.rs` — engine bench, `required-features = ["benchmarks"]` pattern
- `benches/rastrigin.rs` — real-valued GA bench with Rastrigin problem
- `benches/metrics_observer.rs` — feature bench (observer on/off comparison pattern)

### Engine source directories
- `src/engines/pso/` — PSO engine implementation
- `src/engines/cma/` — CMA-ES engine implementation
- `src/engines/eda/` — EDA / UMDA engine implementation
- `src/engines/gp/` — GP engine; `chromosome.rs`, `engine.rs`, `configuration.rs`

### Feature source locations
- `src/operations/` (AOS) — Adaptive Operator Selection
- `src/fitness/surrogate.rs` — Surrogate-assisted evaluation
- `src/fitness/batch.rs` — Batch fitness evaluator

### Configuration
- `Cargo.toml` — `[[bench]]` section and `[dev-dependencies]` (divan = "0.1.21"); add new `[[bench]]` entries here

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `benches/alps.rs` sphere helper + `make_pop` — copy-paste starting point for PSO/CMA-ES/EDA benches
- `benches/metrics_observer.rs` — template for feature on/off comparison bench structure
- `benches/rastrigin.rs` `build_rastrigin_ga` builder — reference for wiring GA config

### Established Patterns
- divan groups use `#[divan::bench_group]` + `#[divan::bench(args = [...])]` for parameterized runs
- Engine benches define a local `sphere`/`rastrigin` fn and a `make_pop` helper — no shared test utilities
- `Cargo.toml` bench entries with non-default features use `required-features`
- All bench files use `harness = false`

### Integration Points
- New `[[bench]]` entries in `Cargo.toml` — one per new file
- `cargo bench --no-run` is the validation gate (from success criteria)

</code_context>

<specifics>
## Specific Ideas

No specific references from discussion — standard divan patterns apply.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 74-add-missing-benchmarks*
*Context gathered: 2026-06-18*
