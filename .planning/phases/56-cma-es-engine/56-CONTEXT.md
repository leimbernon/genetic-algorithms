# Phase 56: CMA-ES Engine - Context

**Gathered:** 2026-06-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 56 implements a `CmaEngine<U>` — the Covariance Matrix Adaptation Evolution Strategy — as a new engine module under `src/engines/cma/`. CMA-ES is the reference algorithm for real-valued black-box continuous optimization and is currently absent from the library.

This phase:
1. **Renames `DeGene` → `RealGene`** — a shared gene-arithmetic trait used by both `DeEngine` and `CmaEngine`. Hard rename, no alias, fits the v3.0.0 breaking-change milestone.
2. **Implements `CmaEngine<U>`** following the established engine pattern (`engine.rs`, `configuration.rs`, `mod.rs` under `src/engines/cma/`) with a `CmaResult<U>` return type.
3. **Wires `GaObserver` hooks** from day 1 (on_start, on_generation_start, on_generation_end, on_new_best, on_finish) — mandatory per the observability initiative in CLAUDE.md.
4. **Exposes tunable adaptation parameters** (cc, cs, c1, cmu) as `Option<f64>` fields with `None` defaulting to Hansen's automatic formulas.
5. Does NOT include restart strategies (IPOP/BIPOP) — deferred to issue #255.

</domain>

<decisions>
## Implementation Decisions

### Gene Arithmetic Trait

- **D-01:** `DeGene` is **hard-renamed to `RealGene`** in this phase. The trait keeps the same interface: `real_value() -> f64` and `with_real_value(f64) -> Self` (rename methods accordingly). All existing `DeGene` bounds in `DeEngine` update to `RealGene`. `CmaEngine` also bounds on `U::Gene: RealGene`. This is a v3.0.0 breaking change — no deprecated alias.

- **D-02:** The `DeGene` impl on `Range<f64>` becomes a `RealGene` impl. No behavioral change. The module file `src/engines/de/gene.rs` is either moved/merged into a shared location or kept as a re-export — planner decides the file placement.

### Restart Strategy

- **D-03:** No restart logic in this phase. `CmaEngine` runs a fixed `max_generations` loop with optional `fitness_target` early stopping. Restart strategies (IPOP, BIPOP) are deferred to issue #255.

### Configuration Depth

- **D-04:** `CmaConfiguration` exposes the following optional tuning fields, all `Option<f64>` defaulting to `None` (= Hansen's auto-computed formulas based on problem dimension `n`):
  - `cc: Option<f64>` — covariance matrix update cumulation
  - `cs: Option<f64>` — step-size control cumulation
  - `c1: Option<f64>` — rank-one update learning rate
  - `cmu: Option<f64>` — rank-mu update learning rate
  - Builder methods: `.with_cc(f64)`, `.with_cs(f64)`, `.with_c1(f64)`, `.with_cmu(f64)`

- **D-05:** Required/common config fields: `sigma0: f64` (initial step size, default 0.3), `population_size: usize` (λ, defaults to `4 + floor(3 * ln(n))` if 0 is passed or via a `default_for_dim(n)` constructor), `max_generations: usize`, `problem_solving: ProblemSolving`, `fitness_target: Option<f64>`.

### Observer Hooks

- **D-06:** `CmaEngine` includes `Option<Arc<dyn GaObserver<U> + Send + Sync>>` and a `with_observer()` builder method from day 1. Observer hooks fire: `on_start` before the loop, `on_generation_start` / `on_generation_end` per generation (with `GenerationStats`), `on_new_best` when best fitness improves, `on_finish` after the loop. This is mandatory per CLAUDE.md.

### Example / Benchmark

- **D-07:** Left to the planner/executor. A natural choice is `cma_es_rastrigin` (demonstrating CMA-ES strength vs. plain GA on a multimodal benchmark), but the researcher/planner decides based on what best showcases CMA-ES vs. existing engine examples.

### Claude's Discretion
- File placement of the `RealGene` trait (new shared module vs. kept in `src/engines/de/gene.rs` with re-export)
- Whether `GenerationStats` fields (`best_fitness`, `diversity`) are populated from CMA-ES's internal state or computed separately
- Internal CMA-ES bookkeeping structures (path vectors pc, ps; covariance matrix C; eigendecomposition scheduling)
- Example benchmark choice (see D-07)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Engine Pattern (follow this exactly)
- `src/engines/de/engine.rs` — Reference engine implementation; CmaEngine follows the same struct layout, `new()`, and `run()` signature
- `src/engines/de/configuration.rs` — Reference configuration pattern; CmaConfiguration mirrors this structure
- `src/engines/de/gene.rs` — Current `DeGene` definition being renamed to `RealGene`
- `src/engines/de/mod.rs` — Module wiring pattern

### Trait System
- `src/traits/real_valued.rs` — `RealValued: LinearChromosome` marker trait (from Phase 51); CmaEngine uses `U: LinearChromosome where U::Gene: RealGene` — not `RealValued`, since gene-level arithmetic is the requirement
- `src/traits/chromosome.rs` — `ChromosomeT` base trait
- `src/traits/linear_chromosome.rs` — `LinearChromosome` supertrait; CmaEngine chromosome bound
- `src/observer/mod.rs` — `GaObserver<U>` trait; hooks CmaEngine must fire

### Observer Integration Reference
- `src/engines/gp/engine.rs` — Most recent engine with observer hooks wired; use as the observer integration pattern reference
- `src/engines/ga.rs` — Full observer hook set reference (on_start, on_generation_start, on_generation_end, on_new_best, on_finish)

### Chromosome Types That Work Out-of-Box
- `src/types/chromosomes/range.rs` — `Range<T>` implements `RealValued`; after rename, its `RealGene` impl makes it the primary CMA-ES chromosome type
- `src/types/chromosomes/multi_range.rs` — `MultiRangeChromosome<T>` also implements `RealValued`; will need `RealGene` impl too

### Breaking Change Context
- `.planning/REQUIREMENTS.md` — Check for any CMA-ES-related requirement IDs
- GitHub issue #252 — CMA-ES feature request (scope confirmation)
- GitHub issue #255 — IPOP/BIPOP restart strategies (explicitly deferred from this phase)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/de/gene.rs` (`DeGene`) — Rename to `RealGene`; identical interface. `de_value()` → `real_value()`, `with_de_value()` → `with_real_value()`. One blanket impl for `Range<f64>` already exists.
- `src/configuration.rs` (`ProblemSolving`) — Reused as-is in `CmaConfiguration.problem_solving`
- `crate::rng::make_rng()` — Standard RNG initialization used by all engines
- `src/observer/mod.rs` (`GaObserver`) — Wire exactly as done in `GpGa` engine (most recent)

### Established Patterns
- Engine struct holds `config`, `init_fn: Arc<Fn>`, `fitness_fn: Arc<FitnessFn>`, `observer: Option<Arc<dyn GaObserver<U>>>`
- `run()` returns `*Result<U>` struct with `population`, `best`, `best_fitness`, `generations`
- `#[cfg(not(target_arch = "wasm32"))]` gates around `Instant::now()` / `elapsed()`; no rayon `par_iter()` needed in CMA-ES core (inherently sequential per-generation covariance update)
- All tests in `tests/cma.rs`, never inline in `src/`

### Integration Points
- `src/engines/de/gene.rs` → rename `DeGene` → `RealGene`; all `DeEngine` bounds update
- `src/lib.rs` → add `pub use engines::cma::{CmaEngine, CmaConfiguration, CmaResult}` re-export
- `src/engines/mod.rs` (or equivalent) → add `pub mod cma`
- `examples/` → new `cma_es_*` example (planner decides name/content)
- `tests/cma.rs` → new test file following Phase 52/53 Nyquist pattern

</code_context>

<specifics>
## Specific Ideas

- Hansen's reference CMA-ES defaults: λ = 4 + floor(3 · ln(n)), μ = floor(λ/2), recombination weights w_i = ln((λ+1)/2) - ln(i) (positive only). If user passes `population_size: 0` (or we add `.default_for_dim(n)`), the engine auto-computes λ from problem dimension.
- Eigendecomposition of the covariance matrix C is the expensive step — schedule it every `floor(1/(10·n·sqrt(n)))` generations (standard CMA-ES practice) rather than every generation.
- The `DeGene` rename cascades: update `src/engines/de/gene.rs`, `src/engines/de/engine.rs`, `src/engines/de/mutation.rs`, and any test files that reference `DeGene` directly.

</specifics>

<deferred>
## Deferred Ideas

- **Restart strategies (IPOP/BIPOP)** — Issue #255; explicitly out of scope for this phase. `CmaEngine` runs a single fixed run.
- **Active CMA-ES** — Negative update for bad steps; defer to a future CMA-ES enhancement phase.
- **CMA-ES in multi-objective mode** — CMA-ES-MO or MO-CMA-ES variants; not in scope.

</deferred>

---

*Phase: 56-cma-es-engine*
*Context gathered: 2026-06-01*
