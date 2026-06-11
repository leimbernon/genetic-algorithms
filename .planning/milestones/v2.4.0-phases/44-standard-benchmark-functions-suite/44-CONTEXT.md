# Phase 44: Standard Benchmark Functions Suite - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can evaluate their GA configurations against a library of 17+ standard benchmark functions (Sphere, Rastrigin, Ackley for single-objective, ZDT1-6 for bi-objective, DTLZ1-7 for many-objective) behind a `benchmarks` feature flag. Each function is a named struct implementing a shared `BenchmarkFn` trait with metadata (name, bounds, known optimum) and a unified `evaluate(&[f64]) -> Vec<f64>` interface.

**In scope:**
- `src/benchmarks/` module organized by family: `single_objective.rs`, `zdt.rs`, `dtlz.rs`
- `BenchmarkFn` trait with metadata (`name()`, `bounds()`, `optimum_value()`) and `evaluate(&[f64]) -> Vec<f64>`
- Single-objective: Sphere, Rastrigin, Ackley
- Bi-objective: ZDT1 through ZDT6
- Many-objective: DTLZ1 through DTLZ7
- `benchmarks` feature flag (follows established serde flag pattern)
- All functions behind the feature flag
- WASM compatibility (pure math, no constraints)
- Refactor existing code: benches and examples that define inline fitness functions should be migrated to import from the shared library

**Out of scope:**
- Functions beyond the roadmap set (Rosenbrock, Schwefel, Griewank, CEC) — may be added in a future phase
- Dynamic benchmark registration or discovery — compile-time only
- Per-benchmark visualization or plotting
- Non-standard or custom benchmark functions — users define their own via FitnessFn closures
- Benchmark suites for non-GA engines (simple fn pointers work for any Rust code)
</domain>

<decisions>
## Implementation Decisions

### Module Structure
- **D-01:** Benchmarks live in `src/benchmarks/` with sub-modules per family: `single_objective.rs`, `zdt.rs`, `dtlz.rs`
- **D-02:** Re-exported via `pub use` in `src/benchmarks/mod.rs` for easy imports
- **D-03:** `lib.rs` adds `#[cfg(feature = "benchmarks")] pub mod benchmarks;` re-export

### API Design
- **D-04:** All benchmarks implement a shared `BenchmarkFn` trait:
  ```rust
  pub trait BenchmarkFn {
      fn name(&self) -> &'static str;
      fn bounds(&self) -> &[(f64, f64)];        // per-dimension bounds
      fn optimum_value(&self) -> Vec<f64>;      // known optimum (length = num_objectives)
      fn evaluate(&self, x: &[f64]) -> Vec<f64>; // unified interface
  }
  ```
- **D-05:** Single-objective returns `vec![value]`, multi-objective returns `vec![f1, f2, ...]`
- **D-06:** Each benchmark is a named struct (e.g., `Sphere`, `Rastrigin`, `ZDT1`, `DTLZ2`) that implements `BenchmarkFn`
- **D-07:** Constructed via `Default` or `::new(dimensions)` where configurable (DTLZ takes num_objectives + variables)

### Scope & Coverage
- **D-08:** Stick to roadmap set: Sphere, Rastrigin, Ackley, ZDT1-6, DTLZ1-7 (~17 functions)
- **D-09:** All behind a single `benchmarks` feature flag (no per-family flags)

### Integration & Migration
- **D-10:** Refactor `benches/de.rs` to import `Sphere` from the shared library instead of defining it locally
- **D-11:** Refactor `benches/ga_run.rs` to use shared benchmark functions where applicable
- **D-12:** Review and migrate other benches and examples (rastrigin example, sms_emoa_zdt1 example) to use the shared library
- **D-13:** Each migration preserves existing behavior identically — no semantic changes to how benchmarks are evaluated

### Claude's Discretion
- Dimension defaults for ZDT (30 vars) and DTLZ (n vars, m objectives with defaults)
- Exact struct/function naming conventions
- Whether to include convenience constants (e.g., `SPHERE_BOUNDS`, `ACKLEY_OPTIMUM`)
- Test strategy: verify known optima are minima, test on random inputs within bounds
- serde derives on benchmark structs (behind the serde flag, not the benchmarks flag)
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Benchmark Patterns (to be refactored)
- `benches/de.rs` — Local `sphere()` fn definition (target for migration to shared library — D-10)
- `benches/ga_run.rs` — Local gene/chromosome custom types (review for migration — D-11)
- `benches/` directory — All 12 bench files (review for shared library usage — D-12)
- `examples/sms_emoa_zdt1.rs` — ZDT1 fitness embedded in example (review for migration)
- `examples/constrained_g1.rs` — Has inline fitness function (review for migration)

### Existing Feature Flag Pattern
- `src/configuration.rs` — `#[cfg_attr(feature = "serde", derive(...))]` pattern to follow for `benchmarks` flag
- `Cargo.toml` — Existing feature flag definitions (serde, observer-tracing, observer-metrics)

### Code Architecture
- `src/lib.rs` — Module registration pattern (`pub mod benchmarks` behind feature flag)
- `src/` — Existing new-module structure (follows `src/<name>/` pattern)
- `.planning/codebase/ARCHITECTURE.md` — Overall architecture reference
- `.planning/codebase/CONVENTIONS.md` — Code style patterns

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 44 — Goal: "Users can evaluate algorithms against 15+ standard benchmark functions (Sphere, Rastrigin, Ackley, ZDT1-6, DTLZ1-7) behind a `benchmarks` feature flag, each with metadata and verified optima"
- Issue #219 — Standard Benchmark Functions Suite
- `.planning/REQUIREMENTS.md` §BEN-01 (phase requirement)

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Must apply cfg-gating rules (benchmark functions are pure math, no constraints)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`benches/de.rs` sphere()** — Simple 3-line fn showing the current ad-hoc pattern
- **Criterion benchmark infrastructure** — Already configured for all 12 bench files
- **Feature flag infra** — `Cargo.toml` has `[features]` section; pattern for adding `benchmarks` is well-established

### Established Patterns
- Feature flags gate whole modules: `#[cfg(feature = "serde")] pub mod ...`
- New top-level modules re-exported via `pub use` in `src/lib.rs`
- Builder pattern for configurable types (e.g., DTLZ taking dimension parameters)

### Integration Points
- `Cargo.toml` — Add `benchmarks = []` feature flag
- `src/lib.rs` — Add `#[cfg(feature = "benchmarks")] pub mod benchmarks;`
- `benches/de.rs` — Replace local `sphere()` with `use genetic_algorithms::benchmarks::Sphere;`
- `benches/ga_run.rs` — Consider if GA run bench should use shared benchmark fns
- `examples/` — Review which examples have inline fitness functions that could use shared benchmarks

</code_context>

<specifics>
## Specific Ideas

- BenchmarkFn trait with `evaluate(&[f64]) -> Vec<f64>` allows single-objective and multi-objective to share one interface
- Default dimension: ZDT uses 30 decision variables (standard in literature), DTLZ uses n_vars = n_objectives + 9 (common test setting)
- All ZDT functions use the same domain [0,1] for all variables (except ZDT1/ZDT4 which have varying ranges)
- Verify optima via simple test: assert that `evaluate(optimum)` produces the minimum vector within tolerance

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 44-Standard Benchmark Functions Suite*
*Context gathered: 2026-05-14*
