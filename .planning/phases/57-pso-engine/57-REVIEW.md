---
phase: 57
reviewed: 2026-06-03T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - src/traits/real_gene.rs
  - src/engines/pso/configuration.rs
  - src/engines/pso/engine.rs
  - src/engines/pso/mod.rs
  - src/lib.rs
  - examples/pso_rastrigin.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: fixed
---

# Phase 57: Code Review Report

**Reviewed:** 2026-06-03
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Reviewed the PSO engine implementation: `RealGene::bounds()` extension, `PsoConfiguration`
builder, `PsoEngine::run()` full loop, module wiring, lib.rs re-exports, and the Rastrigin
example. The core PSO math (velocity update formula, velocity clamping before position update,
absorbing boundary, synchronous gbest update) is correct. WASM safety constraints are respected
— no `Instant::now()` and no unconditional `par_iter()`. Two correctness defects were found,
both affecting the result `best` field returned from `run()`.

---

## Critical Issues

### CR-01: `best` chromosome in result may not match `gbest_position`

**File:** `src/engines/pso/engine.rs:415-419`

**Issue:** When `gbest_fitness` improves after the synchronous gbest-update pass, the engine
calls `find_best(&pop)` to pick the chromosome to store as `best`. However `find_best` finds
the particle with the highest *current* fitness (current position), not the particle whose
personal-best *is* the new gbest. The gbest was set from `pbest_positions[j]` (line 410), but
the current population position for particle `j` may have moved further since that pbest was
recorded. In the common case these will differ, so `result.best` and `result.best_fitness` can
refer to different particles — `best_fitness` holds `gbest_fitness` (the true best ever seen)
while `best` is the particle that is currently in the best position, which may be a different
individual whose current fitness is worse than `best_fitness`.

```rust
// Current (incorrect):
if self.is_better(state.gbest_fitness, best_fitness) {
    best_fitness = state.gbest_fitness;
    let (bi, _) = self.find_best(&pop);   // finds best current position, not pbest winner
    best = pop[bi].clone();
    ...
}
```

The engine needs to track *which particle index* last updated `gbest`, then clone that particle
while temporarily setting its DNA to `gbest_position`:

```rust
// Track the gbest owner index alongside the position:
// In PsoState, add: gbest_owner: usize

// In the synchronous gbest-update pass (lines 407-412):
for j in 0..state.n_particles {
    if self.is_better(state.pbest_fitness[j], state.gbest_fitness) {
        state.gbest_fitness = state.pbest_fitness[j];
        state.gbest_position = state.pbest_positions[j].clone();
        state.gbest_owner = j;  // track who owns gbest
    }
}

// When updating best (lines 415-421):
if self.is_better(state.gbest_fitness, best_fitness) {
    best_fitness = state.gbest_fitness;
    // Reconstruct best from the gbest owner's pbest position
    let owner = state.gbest_owner;
    let new_dna: Vec<U::Gene> = pop[owner]
        .dna()
        .iter()
        .enumerate()
        .map(|(d, g)| g.with_real_value(state.gbest_position[d]))
        .collect();
    best = pop[owner].clone();
    best.set_dna(Cow::Owned(new_dna));
    best.set_fitness(state.gbest_fitness);
    let best_clone = best.clone();
    self.notify(|obs| obs.on_new_best(gen, best_clone));
}
```

---

### CR-02: Empty-population guard both logs "returning empty result" and then panics

**File:** `src/engines/pso/engine.rs:301-309`

**Issue:** The guard block for an empty population emits a `log::warn!` message that says
"returning empty result" — implying a graceful return path — but then immediately calls
`panic!()`. The observer is notified with `on_run_end` (line 307) before the panic, leaving
the observer in an inconsistent state (it has seen `on_run_start` and `on_run_end` but no
generations, and the panic will unwind past any cleanup the caller may have had). The log
message is actively misleading: callers reading logs would expect a graceful empty return but
instead get a panic.

The engine should either:
- Return a `Result<PsoResult<U>, GaError>` (preferred, consistent with the rest of the
  codebase where operators return `Result<_, GaError>`), or
- Panic without calling `on_run_end` first (avoids misleading the observer), and fix the
  log message to say "panicking".

Minimal fix — make the message and behavior consistent:

```rust
// Remove the misleading log message and the premature on_run_end call.
// Just panic with a clear message:
if pop.is_empty() {
    panic!("PsoEngine: init_fn returned an empty population");
}
```

Or, preferred approach — return an error instead of panicking (requires changing the return
type to `Result<PsoResult<U>, GaError>`):

```rust
if pop.is_empty() {
    self.notify(|obs| obs.on_run_end(TerminationCause::GenerationLimitReached, &[]));
    return Err(GaError::EmptyPopulation);  // add variant if needed
}
```

---

## Warnings

### WR-01: `Range<f64>::bounds()` returns only the first range tuple

**File:** `src/traits/real_gene.rs:55-57`

**Issue:** `Range<T>` genes can carry multiple `(lo, hi)` range tuples in
`self.ranges: Arc<[(T, T)]>`. The `bounds()` implementation returns
`self.ranges.first().copied()`, silently using only the first entry. If a gene is
constructed with multiple ranges, the PSO engine will derive `v_max` and apply absorbing
boundaries using only the first range, producing incorrect velocity clamping and boundary
handling for genes where the active range is not index 0.

The doc comment for `bounds()` says it returns "the `(lo, hi)` bounds for this gene" —
singular — with no mention of this restriction. At a minimum, the doc should document the
behavior; ideally the implementation should select the correct range entry based on the gene's
current value, or the trait should be documented to return the full extent.

```rust
// Current: silently uses only ranges[0]
fn bounds(&self) -> Option<(f64, f64)> {
    self.ranges.first().copied()
}

// Option A — document the limitation clearly:
/// Returns the first (lo, hi) range entry for this gene.
/// For genes with multiple range entries only the first is used;
/// the caller is responsible for constructing such genes with a
/// representative range in position 0.
fn bounds(&self) -> Option<(f64, f64)> {
    self.ranges.first().copied()
}

// Option B — return the union of all ranges (widest safe envelope):
fn bounds(&self) -> Option<(f64, f64)> {
    if self.ranges.is_empty() { return None; }
    let lo = self.ranges.iter().map(|(l, _)| *l).fold(f64::INFINITY, f64::min);
    let hi = self.ranges.iter().map(|(_, h)| *h).fold(f64::NEG_INFINITY, f64::max);
    Some((lo, hi))
}
```

---

### WR-02: `Ring::neighborhood_size` documentation contradicts implementation

**File:** `src/engines/pso/configuration.rs:48-56`

**Issue:** The `Ring` variant doc comment states:

> "Number of neighbors on each side to include."

and describes the neighborhood as `{ i-k, ..., i-1, i, i+1, ..., i+k }` giving `2*k+1`
total. However, `lbest_position` in `engine.rs` treats `neighborhood_size` as a **total** `k`,
splits it into `k/2` left and `ceil(k/2)` right, and the neighborhood has `k+1` members
(including `i` itself), not `2*k+1`. A caller reading the config docs who passes
`neighborhood_size: 2` expecting 5 members (`i-2, i-1, i, i+1, i+2`) will get 3 members
(`i-1, i, i+1`) instead. This is a documentation/semantic contract bug.

The fix is to make the doc match the code:

```rust
/// Number of neighbors (excluding self) to include in each particle's neighborhood.
///
/// The neighborhood of particle `i` is `floor(k/2)` left neighbors and
/// `ceil(k/2)` right neighbors (ring-wrapped), giving `k+1` particles total
/// including `i` itself. For example, `neighborhood_size = 2` gives a
/// 3-particle neighborhood: `{ i-1, i, i+1 }`.
/// Common values: 2 (tight ring) or 4 (moderate neighborhood).
neighborhood_size: usize,
```

---

### WR-03: `inertia_weight` exported as public API unnecessarily

**File:** `src/engines/pso/mod.rs:6` and `src/lib.rs:366` (not re-exported there, but `mod.rs`)

**Issue:** `inertia_weight` is a pure internal calculation helper for the run loop. It is
re-exported from `pso::mod.rs` via `pub use configuration::inertia_weight`. While it is not
currently re-exported at the crate root, it is part of the `pso` module's public surface. Any
downstream crate can call `genetic_algorithms::pso::inertia_weight(...)`. Exposing internal
helpers enlarges the API surface that must be kept stable across versions.

```rust
// In src/engines/pso/mod.rs — remove inertia_weight from the re-export:
pub use configuration::{PsoConfiguration, PsoInertia, PsoTopology};  // drop inertia_weight
// Keep the function pub(crate) or pub(super) in configuration.rs:
pub(crate) fn inertia_weight(...) -> f64 { ... }
```

---

### WR-04: Example `init_population` mutates global RNG seed as a side effect

**File:** `examples/pso_rastrigin.rs:54`

**Issue:** `init_population` calls `rng::set_seed(Some(99))` unconditionally every time it is
invoked. The global seed and counter are `static Atomic` values shared across the process.
Calling this inside the initialization function (rather than in `main()` before constructing
the engine) has two problems:

1. If `init_fn` were ever called more than once (e.g., for restart logic) the seed counter
   resets to zero mid-run, breaking the RNG state for the engine's own `make_rng()` call.
2. Setting a global seed inside a library function that users may compose with other engines
   is a hidden side effect — it will reset the seed of any other engine that calls `make_rng()`
   after this returns.

The seed should be set once in `main()` before constructing the engine:

```rust
fn main() {
    rng::set_seed(Some(99));          // set once, before anything else
    let config = PsoConfiguration { ... };
    let mut engine = PsoEngine::new(config, init_population, rastrigin)  // init_population no longer calls set_seed
        .with_observer(Arc::new(LogObserver));
    ...
}

fn init_population(n: usize) -> Vec<RangeChromosome<f64>> {
    // Remove the rng::set_seed(Some(99)) call here
    let mut r = rng::make_rng();
    ...
}
```

---

## Info

### IN-01: `PsoConfiguration` accepts nonsensical values without validation

**File:** `src/engines/pso/configuration.rs:117-133`

**Issue:** `PsoConfiguration::default()` and all builder methods accept values such as
`c1 < 0.0`, `c2 < 0.0`, `w_start < w_end` for `LinearDecay` (inverted schedule),
`max_generations: 0` (loop body never executes, returns uninitialized `best`), or
`population_size: 1` (valid but degenerate). Zero `max_generations` is the most dangerous:
the main loop `for gen in 0..0` never runs, and `result.best` is whatever was in the initial
population at index `best_idx` — this works but is surprising. A validation step (or at least
doc-level contracts) would catch misconfiguration early.

---

### IN-02: `pop[0]` assumed non-empty in `PsoState::new` without assertion

**File:** `src/engines/pso/engine.rs:81`

**Issue:** `PsoState::new` calls `pop[0].dna().len()` on line 81 assuming `pop` is non-empty.
The engine guards against an empty `pop` in `run()` at line 301, but `PsoState::new` has no
corresponding assertion or bounds check. The guard in `run()` currently happens before
`PsoState::new` is called, so this is not a reachable bug today, but the internal API has no
contract that would catch a refactor reordering those calls. An `assert!(!pop.is_empty())` or
`debug_assert!(!pop.is_empty())` at the top of `PsoState::new` would make the precondition
explicit.

---

### IN-03: Magic number `1e-6` in `reached_target` for `FixedFitness`

**File:** `src/engines/pso/engine.rs:221`

**Issue:** `(fitness - target).abs() < 1e-6` uses a hard-coded epsilon with no named constant
or doc comment explaining why `1e-6` was chosen. This is the same value used in the existing
`GaEngine` implementation, but having it as a magic literal in a new engine makes it harder to
change consistently. Consider extracting to a named constant:

```rust
const FIXED_FITNESS_EPSILON: f64 = 1e-6;
```

---

_Reviewed: 2026-06-03_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
