---
phase: "42-warm-starting-population-seeding"
reviewed: "2026-05-13T17:00:00Z"
depth: standard
files_reviewed: 2
files_reviewed_list:
  - src/engines/ga.rs
  - tests/engines/warm_starting/test_warm_starting.rs
findings:
  critical: 6
  warning: 4
  info: 1
  total: 11
status: issues_found
---

# Phase 42: Code Review Report — Warm Starting / Population Seeding

**Reviewed:** 2026-05-13T17:00:00Z
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

Phase 42 adds seed-based population initialization and checkpoint resumption to the GA orchestrator (`src/engines/ga.rs`), with integration tests in `tests/engines/warm_starting/test_warm_starting.rs`. The general approach is sound, but the review uncovered **6 critical issues** and **4 warnings**.

The most significant issues involve **lost runtime state on checkpoint resumption** — several fields on `Ga<U>` (adaptive penalty coefficient, dynamic mutation probability, wall-clock timer) are not included in the `Checkpoint` struct, so resumption silently resets them to defaults, producing different behavior than a continuous run. The **hybrid config override** logic also has blind spots: only 7 configuration fields are restored from the builder, leaving all other builder settings silently overwritten by checkpoint configuration.

On the seed initialization path, an **unsigned underflow** risk exists when `initialize_with_seeds()` is invoked through the public `initialization()` method without first calling `build()` (which validates the seed count).

---

## Critical Issues

### CR-01: Adaptive penalty state (`penalty_coefficient`, `adaptive_penalty_counter`) lost on checkpoint resumption

**File:** `src/engines/ga.rs:170-175`
**Issue:** The `penalty_coefficient` and `adaptive_penalty_counter` fields are part of `Ga<U>`'s runtime state but are **not** included in the `Checkpoint` struct (`src/observe/checkpoint.rs:27-39`, which only serializes `population`, `configuration`, `generation`, and `stats`). After checkpoint resumption, these fields reset to their `Default` values (0.0 and 0 respectively, `src/engines/ga.rs:220-221`).

Consequence: If the penalty coefficient had been adjusted over, say, 200 generations to 50.0 under the Adaptive strategy, the resumed run starts with coefficient 0.0. Line 1896-1899:
```rust
let coeff = if self.penalty_coefficient == 0.0 {
    initial_coefficient
} else {
    self.penalty_coefficient
};
```
uses `initial_coefficient` as fallback, so penalty enforcement effectively restarts from scratch. Infeasible solutions that were previously correctly penalized may now be treated as feasible, corrupting selection pressure.

**Fix:** Add `penalty_coefficient` and `adaptive_penalty_counter` to the `Checkpoint<U>` struct and restore them during checkpoint loading (in the `#[cfg(feature = "serde")]` block of `run_with_callback`, around line 1095). In the checkpoint-load block, after restoring `self.population`:
```rust
// Restore adaptive penalty runtime state
self.penalty_coefficient = ckpt.penalty_coefficient;
self.adaptive_penalty_counter = ckpt.adaptive_penalty_counter;
```

If adding fields to the existing `Checkpoint` struct is undesirable (backward compat concerns for stored checkpoints), gate them behind `#[serde(default)]`.

---

### CR-02: Dynamic mutation probability reset to `probability_max` on checkpoint resumption

**File:** `src/engines/ga.rs:142, 1133-1138`
**Issue:** `dynamic_mutation_probability` is a field on `Ga<U>` (line 142) but is **not** part of the `Checkpoint` struct. After checkpoint resumption, lines 1133-1138 unconditionally reset it:
```rust
if self.configuration.mutation_configuration.dynamic_mutation {
    self.dynamic_mutation_probability = self
        .configuration
        .mutation_configuration
        .probability_max
        .unwrap_or(1.0);
}
```
If the probability had adapted down from 1.0 to 0.15 over 100 generations of the initial run, the resumed run jumps back to 1.0, causing excessive mutation and fundamentally changing evolutionary dynamics.

**Fix:** Add `dynamic_mutation_probability` to the `Checkpoint<U>` struct and restore it after checkpoint loading. Alternatively, gate the `dynamic_mutation_probability` re-initialization block (lines 1133-1138) behind `if checkpoint_generation.is_none()` so it only runs on fresh starts.

---

### CR-03: Hardcoded generation 0 passed to `process_constraints_population` on checkpoint resumption

**File:** `src/engines/ga.rs:1153`
**Issue:** After checkpoint loading, `process_constraints_population(0)` is called with hardcoded generation `0`. Inside `apply_penalty_to_chromosomes` (line 1893-1928), the Adaptive penalty window adjustment check is:
```rust
if generation > 0 && generation % window_size == 0 { ... }
```
Since `generation = 0`, the adjustment check is always skipped on the first initialization call of a resumed run. If the adaptive penalty adjustment was due at the checkpoint boundary, it is silently missed, delaying the next adjustment by a full window.

**Fix:** Use the checkpoint's actual generation when resuming:
```rust
let constraint_gen = checkpoint_generation.unwrap_or(0);
self.process_constraints_population(constraint_gen)?;
```

---

### CR-04: `initialize_with_seeds()` unsigned `usize` underflow when called without `build()`

**File:** `src/engines/ga.rs:919`
**Issue:** Line 919 computes `fill_count` as `usize - usize`:
```rust
let fill_count = population_size - seeds.len();
```
If `seeds.len() > population_size` (no validation has run), this **underflows**: panics in debug mode, wraps to `usize::MAX` in release. While `build()` (lines 586-595) does validate `seeds.len() <= population_size`, the `initialization()` method (line 828) is a **public API** that can be called directly. Any caller who constructs `Ga` via `Ga::new()` (or `Ga::default()`) and calls `initialization()` without first calling `build()` triggers the underflow.

**Fix:** Add a defensive guard at the top of `initialize_with_seeds()`:
```rust
let population_size = self.configuration.limit_configuration.population_size;
if seeds.len() > population_size {
    return Err(GaError::ConfigurationError(format!(
        "Number of seeds ({}) exceeds population_size ({})",
        seeds.len(),
        population_size,
    )));
}
```
And also guard `initialize_random()` for consistency (empty seeds is fine there).

---

### CR-05: Builder configuration settings outside 7 restored fields silently overwritten by checkpoint config

**File:** `src/engines/ga.rs:1072-1092`
**Issue:** The hybrid config override restores only 7 builder fields after checkpoint loading:
- `selection_configuration.method`
- `crossover_configuration.method`
- `mutation_configuration.method`
- `survivor`
- `limit_configuration.problem_solving`
- `limit_configuration.max_generations`
- `limit_configuration.population_size`

Every other builder-set configuration value is **silently overwritten** by the checkpoint's `GaConfiguration`. This includes (but is not limited to):
- `elitism_count` — builder's `with_elitism(N)` is lost
- `stopping_criteria` — builder's `with_stopping_criteria(...)` is lost
- `rng_seed` — builder's `with_rng_seed(...)` is overwritten
- `niching_configuration` — builder's `with_niching_enabled(...)` etc. are lost
- `extension_configuration` — builder's extension settings are lost
- `save_progress_configuration` — builder's checkpoint settings are lost
- `log_level` — builder's `with_logs(...)` is lost

This is contrary to the documented intent ("builder operators override checkpoint operators; checkpoint state for population/stats/generation") and will cause silent, hard-to-debug configuration mismatches. For example, a user who sets `.with_elitism(5)` in the builder chain and resumes from a checkpoint will silently get the checkpoint's `elitism_count` (likely 0).

**Fix:** Either:
(a) Document this as intentional ("builder overrides ONLY operators, all other config comes from checkpoint") and add a test or log warning to make it visible, OR
(b) Expand the override list to include all builder-settable fields. A cleaner approach: save and restore the entire `configuration` field except the 4 checkpoint-owned fields (`population`, `generation`, `stats`, and possibly `save_progress_configuration`):

```rust
// Save builder's entire configuration
let builder_config = self.configuration.clone();
self.configuration = ckpt.configuration;
// Restore builder's operator settings
self.configuration.selection_configuration.method = builder_config.selection_configuration.method;
self.configuration.crossover_configuration.method = builder_config.crossover_configuration.method;
self.configuration.mutation_configuration.method = builder_config.mutation_configuration.method;
self.configuration.survivor = builder_config.survivor;
// Restore builder's limit config
self.configuration.limit_configuration.max_generations = builder_config.limit_configuration.max_generations;
self.configuration.limit_configuration.population_size = builder_config.limit_configuration.population_size;
self.configuration.limit_configuration.problem_solving = builder_config.limit_configuration.problem_solving;
// Restore ALL other builder settings
self.configuration.elitism_count = builder_config.elitism_count;
self.configuration.stopping_criteria = builder_config.stopping_criteria;
self.configuration.niching_configuration = builder_config.niching_configuration;
self.configuration.extension_configuration = builder_config.extension_configuration;
self.configuration.log_level = builder_config.log_level;
self.configuration.rng_seed = builder_config.rng_seed;
self.configuration.save_progress_configuration = builder_config.save_progress_configuration;
```

---

### CR-06: Skipping `build()` bypasses seeds/checkpoint mutual exclusivity and seed count validation

**File:** `src/engines/ga.rs:1105, 1061`
**Issue:** The `build()` method validates mutual exclusivity of seeds and checkpoint (lines 578-583) and seed count vs population size (lines 586-595). However, `run_with_callback()` does not repeat these checks. A caller who constructs `Ga::new().with_seeds(s).with_checkpoint(p)` and calls `run()` directly (without `build()`) bypasses both validations.

At line 1061, the checkpoint path is checked FIRST:
```rust
if self.checkpoint_path.is_some() {
    // ... loads checkpoint, seeds are completely ignored
} else if ... {
    self.initialization()?;
}
```
Seeds are silently discarded when both are set and `build()` is skipped.

**Fix:** Add defensive validation at the top of `run_with_callback`:
```rust
if self.seeds.is_some() && self.checkpoint_path.is_some() {
    return Err(GaError::ConfigurationError(
        "Cannot use both with_seeds() and with_checkpoint() — they are mutually exclusive"
            .to_string(),
    ));
}
if let Some(ref seeds) = self.seeds {
    let pop_size = self.configuration.limit_configuration.population_size;
    if seeds.len() > pop_size {
        return Err(GaError::ConfigurationError(/* ... */));
    }
}
```

---

## Warnings

### WR-01: `unreachable!()` pattern in tests is fragile

**File:** `tests/engines/warm_starting/test_warm_starting.rs:79, 112`
**Issue:** Tests `test_wsm_with_seeds_exceeds_population_errors` and `test_wsm_seeds_and_checkpoint_mutually_exclusive` use:
```rust
_ => unreachable!()
```
in a `match` arm that is reached when `result` is `Ok(...)`. Both tests assert `result.is_err()` before the match, so `unreachable!()` is never hit today. However, if someone modifies the test and removes the `assert!`, the `unreachable!()` would panic with an unhelpful message instead of producing the actual `Ok` value for debugging.

**Fix:** Replace with a direct `unwrap_err()` pattern:
```rust
let err = result.unwrap_err();
let err_msg = err.to_string();
```

---

### WR-02: Hardcoded checkpoint `generation` in test instead of deriving from GA state

**File:** `tests/engines/warm_starting/test_warm_starting.rs:310`
**Issue:** `test_wsm_checkpoint_save_and_resume` hardcodes `generation: 3` when building the checkpoint:
```rust
generation: 3, // 3 generations completed
```
This assumes the GA always runs exactly 3 generations without early termination. If the initial run stops early (e.g., fitness target reached or stagnation), `initial_stats_len` would not be 3, and the assertion `assert_eq!(total_stats, initial_stats_len + 5)` at line 346 could fail or pass for the wrong reason.

**Fix:** Derive the generation from the GA's state after the initial run completes. If the stats vec length reflects completed generations, use that:
```rust
generation: ga.stats().len(), // number of generations actually completed
```

---

### WR-03: Builder's `stopping_criteria` silently overwritten by checkpoint

**File:** `src/engines/ga.rs:1081`
**Issue:** After `self.configuration = ckpt.configuration`, the builder's `stopping_criteria` is overwritten. A user who sets `.with_stopping_criteria(...)` in the builder chain and resumes from a checkpoint will get the checkpoint's stopping criteria instead. In particular, `max_duration_secs` is reset (since `start_time = Instant::now()` at line 1172), potentially causing a resumed run to run much longer than the original time budget would have allowed.

**Fix:** Restore the builder's `stopping_criteria` in the hybrid config block (or add it to the list of explicitly overridden fields).

---

### WR-04: Redundant dereference in test fitness closures

**File:** `tests/engines/warm_starting/test_warm_starting.rs:284, 475`
**Issue:** `RangeGene::value()` returns `T` by value (line 110 of `range.rs`). The test fitness closures use `*g.value() as f64` where `g.value()` already returns an owned `i32`. The `*` dereference is a no-op. Compare with `base_ga()` at line 31 which correctly uses `g.value() as f64` without the redundant dereference.

**Fix:** Remove the unnecessary `*`:
```rust
.with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum())
```

---

## Info

### IN-01: `with_checkpoint()` docs state checkpoint loads at build time but actual loading happens at run time

**File:** `src/engines/ga.rs:797-799`
**Issue:** The doc comment for `with_checkpoint()` says:
> The checkpoint is loaded at build time, restoring the population, generation counter, and accumulated statistics from the checkpoint file.

But the actual loading happens in `run_with_callback()` (line 1064-1065), not in `build()`. The `build()` method only checks file existence (line 601) but does not deserialize the checkpoint. The deserialization requires `serde` trait bounds that are only available in `run_with_callback`'s method-level bound, not in `build()`'s context. The docs should accurately reflect this deferred loading.

**Fix:** Update doc comment to:
```
The checkpoint file existence is validated at build time. The full checkpoint 
(population, generation counter, accumulated stats) is deserialized at run 
time, inside `run()` / `run_with_callback()`.
```

---

_Reviewed: 2026-05-13T17:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
