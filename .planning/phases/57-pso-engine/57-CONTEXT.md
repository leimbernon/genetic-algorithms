# Phase 57: PSO Engine - Context

**Gathered:** 2026-06-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 57 implements a `PsoEngine<U>` — Particle Swarm Optimization — as a new engine module under `src/engines/pso/`. PSO maintains a swarm of particles, each with a position (chromosome gene values) and a velocity (per-gene rate of change). Each generation, velocities are updated by blending inertia, cognitive attraction (toward each particle's personal best), and social attraction (toward the neighborhood best), then positions are updated accordingly.

This phase:
1. **Implements `PsoEngine<U>`** following the established engine pattern (`engine.rs`, `configuration.rs`, `mod.rs` under `src/engines/pso/`) with a `PsoResult<U>` return type.
2. **Stores velocities in engine-internal state** — a `PsoState` struct parallel to the population, analogous to `CmaState`. No chromosome API changes needed.
3. **Supports both gbest and ring (lbest) topologies** via a `PsoTopology` enum in `PsoConfiguration`.
4. **Exposes `PsoInertia` enum** for constant and linearly-decaying inertia weight strategies.
5. **Enforces absorbing boundary conditions** — clamped position + zeroed velocity component at gene bounds.
6. **Wires `GaObserver` hooks** from day 1 (mandatory per CLAUDE.md observability initiative).

</domain>

<decisions>
## Implementation Decisions

### Velocity Storage

- **D-01:** Velocities are stored in `PsoEngine`-internal state (`PsoState` struct) as a `Vec<Vec<f64>>` indexed parallel to the population. Zero chromosome API changes — `LinearChromosome + RealGene` bound only, same as `CmaEngine`. Pattern mirrors `CmaState`.

- **D-02:** Initial velocities are derived from gene bounds: `v_init_i ∈ [-(hi_i - lo_i), +(hi_i - lo_i)]` per gene. No `v_init` configuration field — auto-computed from the initial population's gene ranges via `RealGene::real_value()` and the chromosome's gene metadata.

### Topology

- **D-03:** `PsoTopology` enum with two variants:
  - `PsoTopology::Global` — gbest: all particles attracted to the single best position ever found across the swarm.
  - `PsoTopology::Ring { neighborhood_size: usize }` — lbest: each particle uses the best in its `k` nearest neighbors (by index, ring-wrapped: `(i ± k/2) mod n`).
  - Default: `PsoTopology::Global`.

- **D-04:** Neighborhood in ring topology is purely index-based (not distance-based) — O(k) lookup per particle, no distance matrix.

### Inertia Weight

- **D-05:** `PsoInertia` enum:
  - `PsoInertia::Constant(f64)` — fixed w each generation.
  - `PsoInertia::LinearDecay { w_start: f64, w_end: f64 }` — linearly interpolates from `w_start` (generation 0) to `w_end` (last generation).
  - Default and specific values left to planner (standard TPSO heuristic: `w_start = 0.9, w_end = 0.4`).

- **D-06:** Velocity magnitude clamping (`v_max`) is handled at the boundary enforcement level, not in inertia config.

### Boundary Enforcement

- **D-07:** Absorbing boundary (clamp + zero): when a velocity update would push a gene beyond `[lo, hi]`, the position is clamped to the boundary and the velocity component at that gene is set to 0. Standard absorbing wall behavior.

- **D-08:** `v_max` per gene is auto-derived as `hi_i - lo_i` — not a configuration field. Applied as a post-update clamp on |velocity component| before position update.

### Observer Hooks

- **D-09:** `PsoEngine` includes `Option<Arc<dyn GaObserver<U> + Send + Sync>>` and a `with_observer()` builder method from day 1. Standard 5 hooks: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end`. Mandatory per CLAUDE.md.

### Example

- **D-10:** Example is `pso_rastrigin` (10-dimensional Rastrigin, consistent with `cma_es_rastrigin`). Left to planner/executor for exact content.

### Claude's Discretion

- `PsoState` field names and internal struct layout
- Personal best update logic (update when new fitness strictly improves, or ≥?)
- `PsoResult<U>` field set (minimum: `population`, `best`, `best_fitness`, `generations`)
- Whether `GenerationStats` fields reuse existing fields or get PSO-specific additions (`swarm_velocity_norm` as diversity proxy?)
- Default `neighborhood_size` for ring topology (common: 3 or 5)
- Whether `c1` (cognitive) and `c2` (social) are hardcoded defaults or exposed in `PsoConfiguration` (recommend exposing: standard values 2.0 each, but users may want to tune)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Engine Pattern (follow exactly)
- `src/engines/cma/engine.rs` — Most recent engine implementation; `PsoEngine` follows the same struct layout, `new()`, and `run()` signature
- `src/engines/cma/configuration.rs` — Most recent configuration pattern; `PsoConfiguration` mirrors this structure
- `src/engines/cma/mod.rs` — Module wiring pattern (most recent)
- `src/engines/de/engine.rs` — Secondary engine reference (older but well-established pattern)

### Internal State Pattern
- `src/engines/cma/engine.rs` (`CmaState`) — Internal state struct pattern; `PsoState` follows the same approach (private struct, allocated once before the loop)

### Trait System (chromosome bound)
- `src/traits/real_gene.rs` — `RealGene: GeneT` trait; `PsoEngine` uses `U::Gene: RealGene` for gene value read/write
- `src/traits/linear_chromosome.rs` — `LinearChromosome` supertrait; `PsoEngine` chromosome bound
- `src/traits/chromosome.rs` — `ChromosomeT` base trait

### Observer Integration
- `src/engines/gp/engine.rs` — Most recent observer wiring; use as reference for `on_run_start`/`on_generation_start`/`on_generation_end`/`on_new_best`/`on_run_end`
- `src/observer/mod.rs` — `GaObserver<U>` trait definition

### Chromosome Types That Work Out-of-Box
- `src/types/chromosomes/range.rs` — `Range<f64>` implements `RealGene`; primary PSO chromosome type
- `src/types/chromosomes/multi_range.rs` — `MultiRangeChromosome<f64>` also implements `RealGene`; works for heterogeneous-bounds PSO

### lib.rs Re-export Pattern
- `src/lib.rs` — add `pub use engines::pso::{PsoEngine, PsoConfiguration, PsoResult, PsoTopology, PsoInertia}` re-exports

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/cma/engine.rs` (`CmaState`, run loop) — Direct template: allocate internal state before loop, iterate generations, fire observer hooks per generation
- `src/engines/de/engine.rs` — Secondary engine template
- `src/traits/real_gene.rs` (`RealGene`) — Gene read/write: `gene.real_value()` to read position, `gene.with_real_value(v)` to write updated position
- `crate::rng::make_rng()` — Standard RNG initialization (used by all engines)
- `src/configuration.rs` (`ProblemSolving`) — Reuse for `PsoConfiguration.problem_solving`
- `src/stats.rs` (`GenerationStats`) — Reuse existing stats struct; planner decides if PSO-specific fields are needed

### Established Patterns
- Engine struct holds `config`, `init_fn: Arc<Fn>`, `fitness_fn: Arc<FitnessFn>`, `observer: Option<Arc<dyn GaObserver<U>>>`
- `run()` returns `*Result<U>` with `population`, `best`, `best_fitness`, `generations`
- `#[cfg(not(target_arch = "wasm32"))]` gates around `Instant::now()` / `elapsed()`
- PSO core loop is inherently sequential (no rayon needed in the velocity update) — WASM-safe by default
- All tests in `tests/pso.rs`, never inline in `src/`
- Nyquist test pattern: stub tests with `#[ignore]` gate in wave 1, un-ignore in later waves

### Integration Points
- `src/engines/mod.rs` → add `pub mod pso`
- `src/lib.rs` → add PSO re-exports
- `examples/pso_rastrigin.rs` → new example (planner decides content; follow `cma_es_rastrigin.rs` structure)
- `tests/pso.rs` → new test file

### Velocity Update Formula (for planner reference)
Standard PSO update (each generation, each particle i, each gene d):
```
v[i][d] = w * v[i][d]
         + c1 * r1 * (pbest[i][d] - x[i][d])
         + c2 * r2 * (gbest[d] - x[i][d])     // or lbest[i][d] for ring
x[i][d] = x[i][d] + v[i][d]
```
Where `r1`, `r2` are uniform random in [0,1] per gene per generation. `w` is the inertia weight from `PsoInertia`. After update: clamp `|v[i][d]|` ≤ `v_max[d]`, then clamp `x[i][d]` to `[lo_d, hi_d]` with zero-velocity absorbing rule.

</code_context>

<specifics>
## Specific Ideas

- Standard TPSO defaults: `w_start = 0.9`, `w_end = 0.4` (linear decay), `c1 = c2 = 2.0`
- For ring topology: `neighborhood_size` default of 3 or 5 (planner decides); ring is purely index-based, not distance-based
- Example: `pso_rastrigin` at 10 dimensions, consistent with `cma_es_rastrigin` — shows PSO converging where GA struggles
- Personal best: update when `new_fitness` strictly improves over `pbest_fitness` (for `ProblemSolving::Maximize`: `new > pbest`; for `Minimize`: `new < pbest`)

</specifics>

<deferred>
## Deferred Ideas

- **Constriction factor** — Alternative to inertia weight (Clerc-Kennedy constriction). Different velocity control approach; can be a future `PsoInertia::Constriction` variant.
- **Velocity-based stagnation stopping** — Stop when swarm velocity norm falls below a threshold. Natural PSO convergence criterion; out of scope for phase 57.
- **Discrete PSO** — Velocity-to-probability mapping for binary/categorical chromosomes (BPSO). Separate concern; out of scope.
- **Adaptive c1/c2** — APSO variants where cognitive/social coefficients adapt over time. Future enhancement.

</deferred>

---

*Phase: 57-pso-engine*
*Context gathered: 2026-06-01*
