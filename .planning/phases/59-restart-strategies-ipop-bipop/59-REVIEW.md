---
phase: 59-restart-strategies-ipop-bipop
reviewed: 2026-06-05T14:30:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - src/engines/cma/restart.rs
  - src/engines/cma/engine.rs
  - src/engines/cma/configuration.rs
  - src/engines/cma/mod.rs
  - src/observe/observer/mod.rs
  - src/observe/observer/composite.rs
  - src/lib.rs
  - examples/ipop_rastrigin.rs
  - tests/engines/cma/test_cma.rs
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 59: Code Review Report

**Reviewed:** 2026-06-05T14:30:00Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 59 adds IPOP/BIPOP restart strategies to the CMA-ES engine via a clean pure-types module (`restart.rs`), a new `GaObserver::on_restart` hook (13th), wiring into `CmaConfiguration` and `CmaResult`, and a restructured outer `'restart_loop` in `CmaEngine::run()`. The architecture is sound and the BIPOP parity logic is correctly indexed. One critical bug is present: `init_fn` is called three times on the first run (peek, pre-loop, and first restart-iteration check), wasting two extra population evaluations and potentially having side effects when `init_fn` is stateful. Four warnings cover: `population_scale ≤ 1.0` producing non-growing IPOP (undocumented/unvalidated), incorrect doc hook-count in `composite.rs`, `stagnation_threshold = 0` causing an immediate restart on generation 0 with no user warning, and the `CMA-14` test not actually asserting `RestartEvent` field values despite claiming to (the assertions were deferred but the test name implies coverage). Three info items cover: the module-level doc in `observer/mod.rs` still reads "12 hooks" in one sentence, the `observer/mod.rs` doc table omits `on_restart` from the count summary line, and `ipop_rastrigin.rs` resets the global RNG seed unconditionally inside `init_population` which will corrupt reproducibility when the function is called multiple times per restart run.

---

## Critical Issues

### CR-01: `init_fn` called three times before the first generation runs

**File:** `src/engines/cma/engine.rs:492-543`

**Issue:** On every call to `run()`, `init_fn` is invoked three separate times before a single generation executes:

1. **Line 494** — `peek_pop` call (used only to read `n`, the dimension)
2. **Line 543** — pre-loop `pop` init (used as first population on `total_restarts == 0`)
3. **Line 553** — inside `'restart_loop` on `total_restarts > 0` path (restarts only)

The `peek_pop` population (line 494) is evaluated for dimension, then discarded. The `pop` at line 543 is re-initialised from scratch. This means the first outer-loop iteration discards one full population worth of chromosomes, and the fitness function at line 564 is computing fitness on the second init's population — not the peek one.

For stateless `init_fn` closures this is a pure waste. For closures that advance a shared RNG or maintain side effects (e.g. the example's `rng::set_seed(Some(42))` inside `init_population`), the second init overwrites the global seed state set during peek, producing different gene values than users who read the doc comment ("called once with `population_size`") expect.

The doc on `CmaEngine::new` (line 356) says `init_fn` is "called once with `population_size`" — this is false.

**Fix:** Use `peek_pop` directly as the first population instead of discarding it:

```rust
// Replace the peek + separate first-pop pattern with a single call:
let default_lambda = if self.config.population_size == 0 {
    4 + (3.0 * (n as f64).ln()).floor() as usize
} else {
    self.config.population_size
};

// peek_pop was sampled with peek_size which may differ from default_lambda
// when population_size == 0. Resample only if sizes differ:
let mut pop: Vec<U> = if peek_pop.len() == default_lambda {
    peek_pop
} else {
    (self.init_fn)(default_lambda)
};
```

Or more cleanly: compute `default_lambda` before calling `init_fn` at all (requires computing `n` another way, e.g. reading from a sample chromosome the user provides, or requiring `n` explicitly in the API). The minimal fix is to reuse `peek_pop` as the pre-loop `pop` and remove the duplicate call at line 543.

---

## Warnings

### WR-01: `population_scale ≤ 1.0` silently degrades IPOP to a no-growth restart (or shrinking population)

**File:** `src/engines/cma/restart.rs:44-46`, `src/engines/cma/engine.rs:436-437`

**Issue:** `RestartStrategy::Ipop::population_scale` is documented as "Must be > 1.0 for population growth. A value of 1.0 restarts with the same population size (no IPOP benefit)." The doc warns but nothing enforces it. A caller passing `population_scale: 0.5` will halve the population on each restart; after a few restarts `compute_next_lambda` hits `lambda = 2` (the clamp floor) and stays there forever — the outer loop fires `max_restarts` restarts all at lambda=2, consuming the full budget without any exploration benefit. There is no `log::warn!` or returned error.

The same issue applies to `RestartStrategy::Bipop::population_scale`.

**Fix:** Add a `log::warn!` at restart-trigger time if `population_scale <= 1.0`:

```rust
// In compute_next_lambda, before computing raw:
if let RestartStrategy::Ipop { population_scale, .. }
    | RestartStrategy::Bipop { population_scale, .. } = strategy
{
    if *population_scale <= 1.0 {
        log::warn!(
            target: "cma_events",
            "RestartStrategy: population_scale={} <= 1.0; population will not grow on restart",
            population_scale
        );
    }
}
```

Alternatively add a `validate()` method on `CmaConfiguration` similar to the pattern in `src/validators/`.

---

### WR-02: `stagnation_threshold = 0` triggers an immediate restart on generation 0

**File:** `src/engines/cma/engine.rs:815`

**Issue:** `stagnation_count` starts at `0` (line 594). The restart-trigger check at line 815 is `stagnation_count >= threshold`. When `threshold = 0`, this condition is `true` on the very first generation check (gen=0), before any fitness evaluation has been used to judge stagnation. The engine fires a restart immediately, `total_restarts` increments to 1, and the outer loop re-initialises with a larger population — this repeats `max_restarts` times, consuming the entire restart budget without ever running a meaningful generation.

The doc on `stagnation_threshold` says "Set low (e.g. 10–50)" but does not mention 0 as a degenerate case, and no guard exists.

**Fix:** Clamp or reject `stagnation_threshold = 0` at construction or at the trigger check:

```rust
// In the restart trigger block:
if stagnation_count >= threshold && threshold > 0 {
    // ... existing restart logic
}
```

Or add a warn + clamp in `with_restart_strategy` or a dedicated `validate()` call.

---

### WR-03: Stale doc hook count in `composite.rs` struct doc comment

**File:** `src/observe/observer/composite.rs:37`

**Issue:** The struct doc comment reads "dispatches all **19** lifecycle hooks to every inner observer". After Plan 01 added `on_restart` as the 13th `GaObserver` hook, the correct total is 13 (GaObserver) + 4 (IslandGaObserver) + 3 (Nsga2Observer) = **20**. The module-level doc at line 1 correctly says "20 lifecycle hooks" but the struct-level doc on line 37 still says 19. Both a reader of the module doc and a reader of the struct doc will see different numbers.

**Fix:**
```rust
// Line 37 — change:
/// Fan-out observer that dispatches all 19 lifecycle hooks to every inner
// to:
/// Fan-out observer that dispatches all 20 lifecycle hooks to every inner
```

---

### WR-04: CMA-14 test asserts nothing about `RestartEvent` fields despite being named `test_cma_restart_observer`

**File:** `tests/engines/cma/test_cma.rs:487-532`

**Issue:** The test is named `test_cma_restart_observer` and documents (SC-3): "verifies that `restart_number` is 1-based, `population_size_after` reflects the IPOP scaling, and `kind` matches the restart type." The `kind` assertion (line 519-523) is the only field actually checked. The `restart_number` and `population_size_after` assertions referenced in the doc are explicitly deferred to a comment at line 528-531 that says "Full event field verification is wired by Plan 02 when RestartEvent is populated." Plan 02 is now complete but the assertions were not added. SC-3 is listed as completed in the SUMMARY, but the test does not verify `restart_number == 1` or `population_size_after == floor(initial_lambda * scale)`.

**Fix:** Add the deferred field assertions now that Plan 02 has implemented the full restart event:

```rust
// Capture the RestartEvent to assert on its fields — requires extending SpyObserver
// to store the last RestartEvent rather than just the last kind:
//   last_restart_event: Mutex<Option<RestartEvent>>
// Then assert:
let ev = spy.last_restart_event.lock().unwrap();
let ev = ev.as_ref().expect("restart event must be captured");
assert_eq!(ev.restart_number, 1, "first restart must be restart_number=1");
let expected_pop_after = ((initial_lambda as f64) * scale).floor() as usize;
assert_eq!(ev.population_size_after, expected_pop_after.max(2),
    "population_size_after should reflect IPOP scaling");
```

---

## Info

### IN-01: `observer/mod.rs` module-level doc says "12 hooks" in the opening paragraph

**File:** `src/observe/observer/mod.rs:7`

**Issue:** The opening paragraph says "Cover 13 lifecycle, operator-timing, and special-event hooks" (correct after Plan 01), but the second sentence still reads "exposes **12** hooks that fire at precise points". This is contradicted by the table below which lists 13 rows including `on_restart`.

**Fix:** Change line 2 of the module doc from "12 hooks" to "13 hooks":
```rust
// Line 2:
//! This module provides the [`GaObserver`] trait, which exposes 13 hooks that
```

---

### IN-02: `ipop_rastrigin.rs` resets global RNG seed inside `init_population`, called once per restart

**File:** `examples/ipop_rastrigin.rs:50`

**Issue:** `init_population` calls `rng::set_seed(Some(42))` before building the population. `CmaEngine::run()` calls `init_fn` multiple times (three times on the first run per CR-01, and once per restart thereafter). Each call resets the global seed to 42, producing the same gene values in every restarted population — IPOP with three restarts initialises all four populations from identical gene sequences (before RNG state diverges). This defeats the purpose of restarts which should explore different basins.

This is an example file, but it demonstrates the intended usage pattern and will be copied by users.

**Fix:** Move the seed call outside `init_population`:

```rust
fn main() {
    rng::set_seed(Some(42)); // set once, not on every init call

    let mut engine = CmaEngine::new(config, |n| {
        let mut r = rng::make_rng();
        // ... population building without resetting seed
    }, rastrigin);
```

---

### IN-03: `RestartEvent::generation` field doc says "Restarts are triggered after `stagnation_threshold` consecutive generations" — inaccurate for forced restarts

**File:** `src/engines/cma/restart.rs:130-133`

**Issue:** The doc on `RestartEvent::generation` says "Restarts are triggered after `stagnation_threshold` consecutive generations without improvement. This field records the generation that crossed the threshold." However, Plan 02 introduced a forced-restart-on-max-generations path (engine.rs:866-884) where the `generation` field is set to `self.config.max_generations.saturating_sub(1)` — not necessarily the generation that crossed the stagnation threshold. The doc does not mention this forced-restart case.

**Fix:** Expand the field doc to acknowledge forced restarts:

```rust
/// The generation at which this restart was triggered.
///
/// For stagnation-triggered restarts, this is the generation at which the
/// stagnation counter reached `stagnation_threshold`. For forced restarts
/// (when `max_generations` is exhausted without stagnation triggering),
/// this is `max_generations - 1`.
pub generation: usize,
```

---

_Reviewed: 2026-06-05T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
