# Phase 59: Restart Strategies — IPOP / BIPOP - Research

**Researched:** 2026-06-05
**Domain:** CMA-ES restart strategies (IPOP / BIPOP), Rust engine extension, observer hook addition
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Restart triggered when best fitness has not improved for `stagnation_threshold` consecutive generations. No sigma-collapse detection.
- **D-02:** `RestartStrategy` is a public enum:
  ```
  pub enum RestartStrategy {
      Ipop { population_scale: f64, stagnation_threshold: usize, max_restarts: usize },
      Bipop { population_scale: f64, small_population_size: usize, stagnation_threshold: usize, max_restarts: usize },
  }
  ```
  BIPOP alternates strictly: odd restarts = large (IPOP-style), even restarts = small (fixed size). No budget-tracking.
- **D-03:** `CmaConfiguration` gains `restart_strategy: Option<RestartStrategy>` (default `None`) and `.with_restart_strategy(RestartStrategy) -> Self` builder.
- **D-04:** `CmaResult.best` is global best across all restart runs; tracked continuously across restart boundaries.
- **D-05:** `CmaResult` gains `total_restarts: usize` (default `0`). No per-restart history.
- **D-06:** `GaObserver` gains `fn on_restart(&self, _event: &RestartEvent) {}` (13th hook, default no-op). `RestartEvent { restart_number: usize, generation: usize, population_size_before: usize, population_size_after: usize, kind: RestartKind }`. `RestartKind { Ipop, BipopLarge, BipopSmall }`.
- **D-07:** Full `CmaState` reset on each restart: sigma back to `config.sigma0`, covariance to identity, evolution paths `pc`/`ps` to zero, mean re-derived from fresh `init_fn` call.

### Claude's Discretion

- Default value for `population_scale` in example/docs (common: `2.0`)
- Default value for `stagnation_threshold` in docs (common: `100` or `10*n`)
- Whether `small_population_size = 0` auto-computes to `max(1, floor(default_lambda / 5))`
- Internal bookkeeping variable names and struct layout
- Whether `RestartStrategy`, `RestartEvent`, `RestartKind` go in `observer/mod.rs` or `src/engines/cma/restart.rs`

### Deferred Ideas (OUT OF SCOPE)

- Sigma-collapse stagnation detection
- Budget-based BIPOP alternation (Hansen 2009 original)
- Per-restart `RestartSummary` history in `CmaResult`
- Restart strategies for other engines (PSO, EDA)
</user_constraints>

---

## Summary

Phase 59 extends `CmaEngine` with IPOP and BIPOP restart strategies — two well-known techniques for escaping local optima in multimodal optimization. The implementation does **not** introduce a new engine; it adds a `RestartStrategy` enum field to `CmaConfiguration` and wraps the existing `CmaEngine::run()` inner loop inside an outer restart loop.

The IPOP strategy (Auger & Hansen 2005) restarts with a population size scaled by `population_scale` (typically 2.0) each time. The BIPOP strategy (Hansen 2009) alternates between IPOP-style large restarts and small-population restarts. This phase uses **strict index-parity alternation** (not budget-based) as locked in D-02.

Stagnation detection is improvement-based: a counter increments each generation the best fitness does not strictly improve; when the counter reaches `stagnation_threshold`, a restart fires. A global best is tracked across all restart runs (D-04) so `CmaResult.best` always reflects the best individual found in any restart.

**Primary recommendation:** The entire restart loop can be implemented as an outer `for restart_idx in 0..max_restarts` wrapping the inner generation loop already in `run()`. Stagnation tracking is a single `usize` counter that resets on improvement. The `CmaState::new()` constructor already takes `lambda` as a parameter — restart simply calls it again with the new lambda value. No new heap structures are required beyond those already in `CmaState`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Stagnation detection | CmaEngine (engine loop) | — | Requires per-generation best fitness comparison; lives where fitness tracking already happens |
| Restart trigger + lambda computation | CmaEngine (engine loop) | CmaConfiguration (strategy enum) | Engine reads strategy variant to compute new lambda; configuration just stores the parameters |
| State reset on restart | CmaEngine (engine loop) via CmaState::new() | — | Constructor already handles full initialization; restart calls it with new lambda |
| Global best tracking across restarts | CmaEngine (engine loop) | CmaResult | Engine holds mutable global_best across restart boundaries; result just stores final value |
| RestartEvent emission | CmaEngine (engine loop) | GaObserver | Engine fires notify() same pattern as all other hooks |
| RestartStrategy / RestartEvent / RestartKind types | src/engines/cma/restart.rs (new file) | observer/mod.rs (on_restart hook) | Strategy enum belongs to CMA domain; hook signature goes in observer trait |

---

## Standard Stack

### Core (no new dependencies)
[VERIFIED: codebase grep] This phase adds zero external crates. All required building blocks already exist:

| Asset | Location | Purpose |
|-------|----------|---------|
| `CmaState::new(n, lambda, config, mean)` | `src/engines/cma/engine.rs` | State constructor; call again on restart with new `lambda` |
| `CmaEngine::notify<F>()` | `src/engines/cma/engine.rs:373` | Observer dispatch pattern; reuse for `on_restart` |
| `CmaEngine::find_best()` | `src/engines/cma/engine.rs:406` | Reuse after restart re-init to find initial best of new population |
| `make_rng()` | `src/rng.rs` | Already used; reuse for restart re-init sampling |
| `GaObserver` trait | `src/observe/observer/mod.rs` | Add 13th hook here |
| `ExtensionEvent` | `src/observe/observer/mod.rs:37` | Structural template for `RestartEvent` (same `#[derive(Debug, Clone, Copy)]`) |

### Package Legitimacy Audit

No external packages are added in this phase. Audit: N/A.

---

## Architecture Patterns

### System Architecture Diagram

```
CmaEngine::run()
 │
 ├── [once] init_fn(lambda) → initial pop → evaluate → find_best → global_best
 │
 ├── outer restart loop: for restart_idx in 0..=max_restarts (0 = initial run)
 │    │
 │    ├── stagnation_count = 0
 │    ├── CmaState::new(n, current_lambda, config, fresh_mean)   ← restart resets HERE
 │    │
 │    └── inner generation loop: for gen in 0..max_generations
 │         ├── [sample → evaluate → CMA update → find gen best]
 │         ├── if gen_best improves global_best: update global_best, notify on_new_best
 │         ├── if gen_best improves restart_best: stagnation_count = 0
 │         │   else: stagnation_count += 1
 │         ├── if stagnation_count >= stagnation_threshold:
 │         │    ├── compute next_lambda (IPOP: *= scale, BIPOP: parity check)
 │         │    ├── notify on_restart(RestartEvent { ... })
 │         │    ├── total_restarts += 1
 │         │    └── break inner loop → outer loop continues with new lambda
 │         └── [early-stop: fitness_target reached → break both loops]
 │
 └── notify on_run_end → CmaResult { best: global_best, total_restarts, ... }
```

**Key data flow:** `global_best` / `global_best_fitness` are declared outside the outer loop. `restart_best_fitness` is declared inside the outer loop (fresh each restart). Stagnation compares against `restart_best_fitness`, not `global_best_fitness` — otherwise a stagnant restart that hasn't beaten the global record never triggers a new restart.

### Recommended Project Structure

```
src/engines/cma/
├── engine.rs          # CmaEngine, CmaState, CmaResult — extend run() here
├── configuration.rs   # CmaConfiguration — add restart_strategy field + builder
├── restart.rs         # NEW: RestartStrategy, RestartEvent, RestartKind
└── mod.rs             # Add re-exports for all new public types

src/observe/observer/
└── mod.rs             # Add on_restart(&self, event: &RestartEvent) hook

src/lib.rs             # Add pub use cma::{RestartStrategy, RestartEvent, RestartKind}

tests/engines/cma/
└── test_cma.rs        # Add restart tests (CMA-12 through CMA-17)
```

**Claude's Discretion — module placement:** `RestartStrategy`, `RestartEvent`, and `RestartKind` belong in a new `src/engines/cma/restart.rs`. This separates CMA-specific types from the general observer module (which already has `ExtensionEvent` as a precedent for engine-specific payloads in the observer file, but CMA is the first engine to need its own restart types — keeping them in `cma/restart.rs` and just importing the event type in `observer/mod.rs` is cleaner). The `on_restart` hook signature in `observer/mod.rs` takes `&RestartEvent` via a `use crate::engines::cma::restart::RestartEvent` import — or the type can simply live in `observer/mod.rs` following the `ExtensionEvent` precedent. Either is acceptable; the `cma/restart.rs` module approach avoids adding CMA-specific types to the general observer module.

### Pattern 1: Outer Restart Loop Structure

[ASSUMED] — derived from BIPOP reference implementation (CyberAgentAILab/cmaes bipop_cma.py, verified via GitHub API)

```rust
// Source: modeled on CyberAgentAILab/cmaes/examples/bipop_cma.py (canonical BIPOP reference)
pub fn run(&mut self) -> CmaResult<U> {
    let mut rng = make_rng();
    let mut total_restarts: usize = 0;
    let mut global_best_fitness = /* worst possible for problem direction */;
    let mut global_best: Option<U> = None;

    // Compute default lambda once (needed for BIPOP small-restart fallback)
    // ... init peek, compute n, compute lambda as before ...

    let mut current_lambda = lambda; // initial lambda from config or auto

    // Outer restart loop (0 = initial run, 1..=max_restarts = actual restarts)
    'restart_loop: loop {
        // Re-init population for this run
        let mut pop = (self.init_fn)(current_lambda);
        // evaluate...
        // Compute initial mean from pop
        // CmaState::new(n, current_lambda, &self.config, initial_mean)
        let mut state = CmaState::new(n, current_lambda, &self.config, initial_mean);
        // initial eigen decomp...

        let mut restart_best_fitness = /* find_best(&pop).1 */;
        let mut stagnation_count: usize = 0;

        // Update global best from this restart's initial population
        if self.is_better(restart_best_fitness, global_best_fitness) {
            global_best_fitness = restart_best_fitness;
            global_best = Some(pop[best_idx].clone());
            self.notify(|obs| obs.on_new_best(0, global_best.clone().unwrap()));
        }

        for gen in 0..self.config.max_generations {
            // ... existing CMA update loop ...

            // Stagnation tracking (uses restart_best, not global_best)
            if self.is_better(gen_best_fit, restart_best_fitness) {
                restart_best_fitness = gen_best_fit;
                stagnation_count = 0;
            } else {
                stagnation_count += 1;
            }

            // Global best update
            if self.is_better(gen_best_fit, global_best_fitness) {
                global_best_fitness = gen_best_fit;
                global_best = Some(pop[best_idx].clone());
                self.notify(|obs| obs.on_new_best(gen, ...));
            }

            // Restart trigger
            if let Some(ref strategy) = self.config.restart_strategy {
                let threshold = match strategy { Ipop { stagnation_threshold, .. } | Bipop { stagnation_threshold, .. } => *stagnation_threshold };
                if stagnation_count >= threshold {
                    let max_r = match strategy { Ipop { max_restarts, .. } | Bipop { max_restarts, .. } => *max_restarts };
                    if total_restarts >= max_r {
                        break 'restart_loop;
                    }
                    let pop_before = current_lambda;
                    current_lambda = compute_next_lambda(strategy, current_lambda, lambda, total_restarts, &mut rng);
                    let kind = restart_kind(strategy, total_restarts);
                    total_restarts += 1;
                    self.notify(|obs| obs.on_restart(&RestartEvent { restart_number: total_restarts, generation: gen, population_size_before: pop_before, population_size_after: current_lambda, kind }));
                    break; // break inner loop → outer loop handles re-init
                }
            }

            // Early stop: fitness_target reached
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(global_best_fitness, target) {
                    termination_cause = TerminationCause::FitnessTargetReached;
                    break 'restart_loop;
                }
            }
        }

        // If no restart strategy, always exit after first run
        if self.config.restart_strategy.is_none() { break; }
        // If max_restarts already consumed (checked above), loop would have broken
    }
    // ...
}
```

### Pattern 2: Lambda Computation for IPOP vs BIPOP

[VERIFIED: CyberAgentAILab/cmaes bipop_cma.py, confirmed via GitHub API]

The canonical BIPOP lambda computation:

```rust
fn compute_next_lambda(
    strategy: &RestartStrategy,
    current_lambda: usize,
    default_lambda: usize,
    restart_count: usize, // 0-based count BEFORE this restart
    _rng: &mut impl Rng,  // needed for original budget-based BIPOP; unused in strict-alternation variant
) -> usize {
    match strategy {
        RestartStrategy::Ipop { population_scale, .. } => {
            // Each restart multiplies by population_scale (typically 2.0)
            // restart_count = 0 → first restart → lambda * scale^1
            let scale_exp = (restart_count + 1) as f64; // [ASSUMED] — use current_lambda * scale instead
            ((current_lambda as f64) * population_scale).floor() as usize
        }
        RestartStrategy::Bipop { population_scale, small_population_size, .. } => {
            // Strict alternation: 0th restart = large, 1st = small, 2nd = large, ...
            // restart_count is 0-based BEFORE this restart fires
            // odd restart_count (0, 2, 4...) → the restart about to start is large (index 1, 3, 5...)
            // Actually: restart_number (1-based after increment) determines:
            //   restart_number is odd  → large (IPOP-style)
            //   restart_number is even → small
            let next_restart_number = restart_count + 1;
            if next_restart_number % 2 == 1 {
                // Large restart: scale current lambda
                ((current_lambda as f64) * population_scale).floor() as usize
            } else {
                // Small restart: fixed or auto-computed
                if *small_population_size == 0 {
                    // Auto: max(1, floor(default_lambda / 5))
                    (default_lambda / 5).max(1)
                } else {
                    *small_population_size
                }
            }
        }
    }
}
```

**Critical clarification on "current_lambda" for IPOP restarts:** IPOP must track the per-restart lambda, not the default lambda. Each consecutive IPOP restart multiplies the *previous restart's* lambda by `population_scale`. The variable `current_lambda` must persist across restarts (declared outside the outer loop).

### Pattern 3: RestartKind Derivation

```rust
fn restart_kind(strategy: &RestartStrategy, restart_count: usize) -> RestartKind {
    // restart_count is the 0-based count BEFORE this restart, so
    // next_restart_number = restart_count + 1
    let next_restart_number = restart_count + 1;
    match strategy {
        RestartStrategy::Ipop { .. } => RestartKind::Ipop,
        RestartStrategy::Bipop { .. } => {
            if next_restart_number % 2 == 1 { RestartKind::BipopLarge }
            else { RestartKind::BipopSmall }
        }
    }
}
```

### Pattern 4: RestartEvent and RestartKind Type Definitions

Following `ExtensionEvent` precedent in `src/observe/observer/mod.rs`:
[VERIFIED: codebase read — ExtensionEvent at line 37 uses same derive]

```rust
// In src/engines/cma/restart.rs (or observer/mod.rs following ExtensionEvent precedent)
#[derive(Debug, Clone, Copy)]
pub struct RestartEvent {
    pub restart_number: usize,
    pub generation: usize,
    pub population_size_before: usize,
    pub population_size_after: usize,
    pub kind: RestartKind,
}

#[derive(Debug, Clone, Copy)]
pub enum RestartKind {
    Ipop,
    BipopLarge,
    BipopSmall,
}
```

### Anti-Patterns to Avoid

- **Comparing stagnation against global best:** If `stagnation_count` compares against `global_best_fitness`, a restart run that makes local progress (improves within its own search region) will still stagnate against a globally superior point found in an earlier restart. Use a `restart_best_fitness` scoped to the current restart run.
- **Forgetting to track `default_lambda`:** The BIPOP small restart formula needs `default_lambda` (the formula `4 + floor(3*ln(n))`), which is the baseline regardless of how large IPOP has grown `current_lambda`. Capture it after the first `n` is computed and before the outer loop starts.
- **Re-computing `n` on every restart:** `n` (problem dimension) is fixed. Compute it once in the initial population peek before the outer loop. Every restart uses the same `n`.
- **Not resetting `eigeneval` in CmaState:** `CmaState::new()` initializes `eigeneval: 0`. Since restart creates a fresh `CmaState`, this is handled automatically.
- **Calling `on_run_start` / `on_run_end` per restart:** These hooks should fire once per `run()` call, not once per restart. Only `on_restart` fires per restart event.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fresh covariance matrix on restart | Manual n×n identity construction | `CmaState::new(n, lambda, config, mean)` | Constructor already initializes identity C, zero paths, identity B/D — calling it again is the reset |
| RNG for re-init sampling | New RNG type | `make_rng()` + existing gene construction pattern | Already used for initial pop in all CMA tests — same pattern works for restart |
| Observer dispatch for on_restart | Custom dispatch mechanism | `self.notify(|obs| obs.on_restart(&event))` | `notify()` helper already exists; same FnOnce pattern used for every other hook |

---

## IPOP / BIPOP Algorithm Reference

[VERIFIED: CyberAgentAILab/cmaes/examples/bipop_cma.py and ipop_cma.py — fetched via GitHub API]
[CITED: https://github.com/CyberAgentAILab/cmaes/blob/main/examples/bipop_cma.py]

### IPOP-CMA-ES (Auger & Hansen 2005)

- Population `lambda` doubles (or scales by `population_scale`) on each restart
- `sigma` resets to `sigma0` (config value) — **same** sigma0 each time
- Mean re-sampled randomly from search domain (fresh `init_fn` call in our impl)
- Full covariance matrix reset (identity), evolution paths zeroed
- Stop condition for a single run: convergence or stagnation within that run

The standard implementation doubles lambda: `lambda_new = lambda_prev * inc_popsize` (where `inc_popsize = 2`). In our implementation, `current_lambda` is declared outside the outer loop and multiplied by `population_scale` each time a large restart fires.

### BIPOP-CMA-ES (Hansen 2009) — Simplified (strict-alternation variant)

Hansen's original BIPOP tracks evaluation budgets for large vs. small regimes and switches based on which has consumed fewer evaluations. **This phase uses strict odd/even alternation instead** (D-02 locked decision).

The canonical small-restart lambda formula (from CyberAgentAILab reference):
```python
# Original budget-based: popsize_multiplier = inc_popsize^n_restarts
# popsize = floor(popsize0 * popsize_multiplier ^ (rng.uniform()^2))
# sigma = sigma0 * 10^(-2 * rng.uniform())
```

For the strict-alternation variant locked by D-02:
- **Large restart (odd restart_number):** `lambda_new = current_lambda * population_scale`, `sigma = sigma0`
- **Small restart (even restart_number):** `lambda_new = small_population_size` (or `max(1, floor(default_lambda / 5))` if 0), `sigma = sigma0`

Note: the sigma randomization from the original BIPOP (small restarts use `sigma0 * 10^(-2*U(0,1))`) is **out of scope** per the simplified design in D-02. All restarts reset to `config.sigma0`.

### Claude's Discretion — Recommended Defaults

Based on Hansen's reference implementations and literature:
- `population_scale = 2.0` — canonical IPOP doubling factor
- `stagnation_threshold = 100` — safe default for moderate-dimensional problems; `10 * n` scales better for high-dimensional (Claude's discretion per CONTEXT.md)
- `small_population_size = 0` (auto) → `max(1, floor(default_lambda / 5))` — this gives roughly 1/5 of the default population, consistent with pycma's "small" regime intent

---

## Common Pitfalls

### Pitfall 1: Outer Loop Termination Logic
**What goes wrong:** The outer restart loop runs indefinitely or fires one too many restarts after `max_restarts` is reached.
**Why it happens:** Off-by-one between when `total_restarts` is checked vs. incremented. The guard `if total_restarts >= max_restarts { break }` must fire BEFORE incrementing and calling `on_restart`.
**How to avoid:** Check `if total_restarts >= max_r { break 'restart_loop; }` inside the stagnation trigger block, before computing new lambda or firing the hook. Then increment.
**Warning signs:** Test shows `total_restarts > max_restarts` in `CmaResult`.

### Pitfall 2: `max_generations` Budget Across Restarts
**What goes wrong:** The total number of generations run greatly exceeds `max_generations` when restarts fire.
**Why it happens:** `max_generations` controls the inner loop only. Each restart gets a fresh `0..max_generations` budget.
**How to avoid:** This is correct and intentional — each restart is an independent run. Document clearly that `max_generations` is per-restart, not total. `result.generations` should reflect the sum of all inner-loop generations completed.
**Warning signs:** User confusion about total computation time — document in rustdoc.

### Pitfall 3: `on_new_best` Fires on Every Restart's Initial Population
**What goes wrong:** `on_new_best` fires at generation 0 of each restart even if the initial population is worse than the global best.
**Why it happens:** The existing code in `engine.rs` fires `on_new_best` unconditionally for the initial best. With restarts, the "initial best of this restart" may be worse than a prior restart's result.
**How to avoid:** Gate the initial-best `on_new_best` notification through `is_better(restart_initial_fitness, global_best_fitness)`. Only fire if the restart's initial pop happens to beat the current global record. This matches the existing per-generation best-tracking logic.
**Warning signs:** Test shows `on_new_best` firing more times than fitness improvements above global best.

### Pitfall 4: `CmaState` Strategy Parameter Recomputation
**What goes wrong:** CMA strategy parameters `cs`, `cc`, `c1`, `cmu` are wrong on restart because they depend on lambda but config may have `None` (auto-compute).
**Why it happens:** `CmaState::new()` recomputes all auto-formula parameters using the new `lambda`. If `config.cs` etc. are `None`, they are correctly recomputed from the new lambda value. This is correct behavior and happens automatically.
**How to avoid:** Nothing to do — `CmaState::new()` already handles this correctly. The pitfall is thinking this needs special handling.
**Warning signs:** Not a real pitfall; document as a "this works automatically" note.

### Pitfall 5: WASM Compatibility of `make_rng()` in Restart Re-Init
**What goes wrong:** Restart re-init calls `init_fn(new_lambda)` which calls the user's init function. The user's init function may use `make_rng()` internally. This is fine since `make_rng()` is already WASM-compatible.
**Why it happens:** N/A — no pitfall here; existing WASM gates in the engine (no `Instant`, no unconditional `par_iter`) remain intact across restarts.
**How to avoid:** Verify `cargo check --target wasm32-unknown-unknown` passes after adding the restart loop. The restart logic itself uses only arithmetic and comparisons — no new WASM-incompatible calls.
**Warning signs:** CI failure on wasm-check workflow.

---

## Code Examples

### CmaState Reset (restart re-initialization)

[VERIFIED: codebase read — `CmaState::new()` at engine.rs:221]

On restart, the engine needs:
1. Fresh population from `init_fn(new_lambda)`
2. Fresh mean computed from that population
3. `CmaState::new(n, new_lambda, &self.config, new_mean)` — this resets sigma to `config.sigma0`, sets identity C, zeros pc/ps
4. Initial eigendecomposition (same 3 lines as the initial run setup)

```rust
// Source: src/engines/cma/engine.rs (existing pattern for initial setup)
// Re-use verbatim on restart with new `current_lambda`
let mut pop = (self.init_fn)(current_lambda);
for ind in &mut pop { let f = (self.fitness_fn)(ind.dna()); ind.set_fitness(f); }
let mut new_mean = vec![0.0_f64; n];
for chr in &pop {
    for (j, g) in chr.dna().iter().enumerate() { new_mean[j] += g.real_value(); }
}
for v in &mut new_mean { *v /= pop.len() as f64; }
let mut state = CmaState::new(n, current_lambda, &self.config, new_mean);
let (b_init, d_init) = jacobi_eigendecompose(&state.c_mat, n);
state.b_mat = b_init; state.d_vec = d_init;
state.invsqrtc = compute_invsqrtc(&state.b_mat, &state.d_vec, n);
```

### Observer Hook Addition (13th hook pattern)

[VERIFIED: codebase read — `ExtensionEvent` at observer/mod.rs:37, `on_extension_triggered` at line 114]

```rust
// Source: src/observe/observer/mod.rs — follow ExtensionEvent / on_extension_triggered pattern
/// Called when the engine triggers an automatic restart.
fn on_restart(&self, _event: &RestartEvent) {}
```

The `RestartEvent` import needs to be visible in `observer/mod.rs`. If types live in `cma/restart.rs`, add `use crate::engines::cma::restart::RestartEvent;` at the top. Alternatively, define `RestartEvent` and `RestartKind` directly in `observer/mod.rs` alongside `ExtensionEvent` — this is the simpler path since `GaObserver` already lives there.

### `CmaConfiguration` Builder Addition

[VERIFIED: codebase read — builder pattern at configuration.rs:107–176]

```rust
// Source: src/engines/cma/configuration.rs — follows existing builder pattern
pub restart_strategy: Option<RestartStrategy>,

// In Default impl: restart_strategy: None,

// Builder method:
pub fn with_restart_strategy(mut self, strategy: RestartStrategy) -> Self {
    self.restart_strategy = Some(strategy);
    self
}
```

### `CmaResult` Extension

[VERIFIED: codebase read — CmaResult at engine.rs:297, constructed only at engine.rs:714]

```rust
// Add to CmaResult struct:
pub total_restarts: usize,

// At construction site (engine.rs ~714):
CmaResult {
    population: pop,
    best: global_best.unwrap_or(best),  // use global_best when restarts ran
    best_fitness: global_best_fitness,
    generations: all_generations_count,
    total_restarts,
}
```

`CmaResult` is not constructed anywhere in user code (only inside `run()`), so adding a field is non-breaking. [VERIFIED: codebase grep — only one construction site]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single-run CMA-ES | IPOP restart (Auger & Hansen 2005) | 2005 | Large population on restart escapes basin of attraction |
| IPOP only | BIPOP (Hansen 2009) | 2009 | Alternating large+small restarts covers search space better on benchmarks |
| Budget-based BIPOP | Strict-alternation BIPOP (this phase) | Phase 59 | Simpler implementation; avoids budget tracking complexity; less optimal but sufficient for most use cases |

**Deprecated/outdated:**
- Budget-based BIPOP alternation (Hansen 2009 original): deferred by design choice in D-02; strict alternation is intentionally simpler.

---

## Runtime State Inventory

Step 2.5 SKIPPED — this is a feature addition (greenfield within an existing engine), not a rename/refactor/migration phase.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable + wasm32 target | WASM gate CI | ✓ (CI) | verified in `.github/workflows/wasm-check.yml` | — |
| `cargo test` | Integration tests | ✓ | local dev environment | — |

No missing dependencies. This phase is pure Rust code additions.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + cargo test |
| Config file | none (standard Cargo.toml) |
| Quick run command | `cargo test engines::cma` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | IPOP restarts with scaled population after stagnation | integration | `cargo test engines::cma::test_cma::test_cma_ipop_restarts` | ❌ Wave 0 |
| SC-2 | BIPOP alternates large/small restarts | integration | `cargo test engines::cma::test_cma::test_cma_bipop_alternation` | ❌ Wave 0 |
| SC-3 | `on_restart` hook fires with correct RestartEvent fields | integration | `cargo test engines::cma::test_cma::test_cma_restart_observer` | ❌ Wave 0 |
| SC-4 | WASM `cargo check` passes | compile gate | `cargo check --target wasm32-unknown-unknown` | ✅ (CI workflow exists) |
| SC-5 | No restart when strategy is None | regression | `cargo test engines::cma::test_cma::test_cma_no_restart_when_none` | ❌ Wave 0 |
| SC-6 | `total_restarts` counts correctly | unit | `cargo test engines::cma::test_cma::test_cma_total_restarts_count` | ❌ Wave 0 |
| SC-7 | Global best preserved across restarts | integration | `cargo test engines::cma::test_cma::test_cma_global_best_across_restarts` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test engines::cma`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] New test functions in `tests/engines/cma/test_cma.rs` covering CMA-12 through CMA-17 (SC-1 through SC-6 above, plus SC-7)
- [ ] No new test files needed — extend existing `tests/engines/cma/test_cma.rs`

---

## Test Design Notes

### Proving IPOP Actually Restarts (SC-1)
Use a spy observer with `on_restart` counter. Set `stagnation_threshold` very low (e.g., 5 generations) and `max_restarts = 2`. Run on sphere with `max_generations = 50`. Assert `spy.restart_count >= 1` and `result.total_restarts >= 1`.

### Proving BIPOP Alternates (SC-2)
Collect `RestartEvent.kind` in a Vec via spy observer. After 4 restarts (with low stagnation threshold), assert the sequence is `[BipopLarge, BipopSmall, BipopLarge, BipopSmall]`.

### Proving Global Best Preserved (SC-7)
Run IPOP on a problem where the initial run finds a good solution. Use a seeded RNG that guarantees the initial run does better than restarts. Assert `result.best_fitness <= initial_best_observed`. The global best should not worsen across restarts.

### Benchmark Function for Restart Benefit
Rastrigin already exists in the test helpers. For demonstrating restart benefit, use higher-dimensional Rastrigin (10D+) with tight `max_generations` per restart — restarts help escape the many local optima. The `cma_es_rastrigin` example can be extended to show IPOP.

---

## Security Domain

`security_enforcement` is not set in `.planning/config.json`. This phase adds no user input parsing, no networking, no serialization of secrets, and no cryptographic operations. ASVS is not applicable to this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | BIPOP strict-alternation: restart_number % 2 == 1 → BipopLarge, % 2 == 0 → BipopSmall | Algorithm Reference / Pattern 2 | Wrong alternation order — easy to flip in code if tests reveal wrong behavior |
| A2 | Default `small_population_size = 0` auto-computes to `max(1, floor(default_lambda / 5))` | Claude's Discretion section | 1/5 is a common "small" heuristic; exact formula is discretion per CONTEXT.md |
| A3 | `stagnation_threshold = 100` recommended default | Claude's Discretion section | May be too large or too small depending on problem — user tunes this |
| A4 | `result.generations` = sum of all inner-loop generations across all restarts | Code Examples | Could alternatively mean last-restart-only generations; document clearly |

---

## Open Questions

1. **`result.generations` semantics across restarts**
   - What we know: current `CmaResult.generations = all_stats.len()` (count of completed generations in a single run)
   - What's unclear: with restarts, should this be total generations across all restarts or just the final restart's count?
   - Recommendation: use total across all restarts (accumulate `all_stats` from each restart into one Vec). This matches "how much computation was done" which is the most useful metric.

2. **`on_generation_start` / `on_generation_end` hook generation numbers across restarts**
   - What we know: these currently emit `gen` (0-based within the loop)
   - What's unclear: after restart 1 completes at gen 47, does restart 2's first generation fire at gen 0 or gen 48?
   - Recommendation: reset `gen` to 0 at each restart start. Observers wanting total-generation semantics can track `restart_number * max_generations + gen` themselves. This is simpler and matches how PSO/EDA handle their per-run generation numbers.

---

## Sources

### Primary (HIGH confidence)
- `src/engines/cma/engine.rs` — CmaEngine, CmaState, CmaResult full source; run() loop at lines 419–721
- `src/engines/cma/configuration.rs` — CmaConfiguration struct; builder pattern at lines 107–176
- `src/observe/observer/mod.rs` — GaObserver trait with 12 hooks; ExtensionEvent at line 37 as structural template
- `src/engines/eda/engine.rs` — EDA engine; observer wiring and notify() pattern reference
- `tests/engines/cma/test_cma.rs` — Existing CMA tests CMA-01 through CMA-11; SpyObserver pattern

### Secondary (MEDIUM confidence)
- [CyberAgentAILab/cmaes bipop_cma.py](https://github.com/CyberAgentAILab/cmaes/blob/main/examples/bipop_cma.py) — Canonical BIPOP implementation; budget-based alternation logic (confirmed via GitHub API gh raw fetch)
- [CyberAgentAILab/cmaes ipop_cma.py](https://github.com/CyberAgentAILab/cmaes/blob/main/examples/ipop_cma.py) — Canonical IPOP implementation; `popsize * inc_popsize` formula
- [OptunaHub restart_cmaes](https://hub.optuna.org/samplers/restart_cmaes/) — Confirms `inc_popsize = 2` default and random x0 on restart

### Tertiary (LOW confidence)
- [Alternative Restart Strategies for CMA-ES (arXiv 1207.0206)](https://arxiv.org/pdf/1207.0206) — Original NIPOP/BIPOP paper; PDF not readable by WebFetch but content confirmed via secondary sources

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; pure code addition using verified existing patterns
- IPOP algorithm: HIGH — canonical implementation verified from reference codebase (GitHub API)
- BIPOP algorithm: HIGH for structure; MEDIUM for the exact strict-alternation formula (this phase's simplification of the original budget-based algorithm is by design, not by research finding)
- Architecture patterns: HIGH — directly derived from reading the actual engine source
- Pitfalls: HIGH — all pitfalls identified by code analysis of existing patterns; not speculation

**Research date:** 2026-06-05
**Valid until:** 2026-07-05 (stable domain — CMA-ES restart algorithms are decades old; only the Rust patterns could change)
