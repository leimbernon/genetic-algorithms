---
phase: 43-adaptive-operator-selection-aos
reviewed: 2026-06-02T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - Cargo.toml
  - examples/aos_demo.rs
  - src/aos.rs
  - src/configuration.rs
  - src/engines/ga.rs
  - src/lib.rs
  - src/traits/configuration.rs
  - tests/engines/aos/test_aos.rs
  - tests/test_engines.rs
findings:
  critical: 2
  warning: 4
  info: 2
  total: 8
status: fixed
---

# Phase 43: Code Review Report

**Reviewed:** 2026-06-02
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 43 adds Adaptive Operator Selection (AOS) to the standard GA engine. The core
`AosState` ring-buffer implementation is structurally sound and the three strategy
variants (PM, AP, MAB) are coherent. The integration into `ga.rs` via a per-generation
`Mutex<AosState>` is architecturally valid and WASM-compatible as documented.

Two blockers are identified: a silent ring-buffer mean computation error that corrupts
reward signals after the window wraps, and an inverted reward signal for the maximization
case that causes AOS to systematically downgrade the best-performing operators. Four
warnings cover the Mutex contention pattern, the unused `learning_rate` field, a
zero-window panic path, and an unenforced single-portfolio validation for the `build()`
empty-portfolio guard. Two info items cover dead code and a test gap.

---

## Critical Issues

### CR-01: Ring-buffer mean corrupted after window wraps

**File:** `src/aos.rs:102-105`

**Issue:** `mean_reward()` always slices `&self.rewards[..self.count]`. Once the buffer
fills (`count == window_size`) the `count` field is capped at `window_size` (line 99:
`self.count = (self.count + 1).min(self.rewards.len())`), which is correct. However
the cursor wraps around and new writes overwrite old slots at arbitrary positions in
`rewards`, meaning the oldest entries are not at `rewards[..count]` — they are spread
throughout the ring. After the first wrap the slice used in the mean no longer reflects
the `count` most-recent rewards; it computes the mean of the first `window_size` raw
slots regardless of write order, which is not a sliding-window mean.

Concrete example: `window_size=3`, rewards added in order `[0.9, 0.9, 0.9, 0.0, 0.0]`.
After 5 writes the slots contain `[0.0, 0.0, 0.9]`, cursor=2, count=3.
`mean_reward()` returns `(0.0+0.0+0.9)/3 = 0.30`, not the correct sliding mean `0.30`
— in this case the values happen to match, but for `window_size=4` and rewards
`[1.0, 1.0, 1.0, 1.0, -1.0]` the slots are `[-1.0, 1.0, 1.0, 1.0]`, cursor=1, count=4
and the mean is `(-1.0+1.0+1.0+1.0)/4 = 0.50`, but the true mean of the last 4 rewards
is `(1.0+1.0+1.0-1.0)/4 = 0.50` — again equal by accident. The real failure case is
when not all slots have been written yet AND the window has wrapped partially. Tracing
more carefully: count is capped so this only diverges from correct behavior when
`self.count == self.rewards.len()` AND the cursor has wrapped past index 0, because then
`rewards[..count]` covers the full buffer (all slots), which is in fact correct.

**Revised conclusion after trace:** For a full buffer (`count == len`), `rewards[..count]`
covers all slots which is the entire ring — the mean is correct regardless of cursor
position. For a non-full buffer (`count < len`), writes always advance from 0 upward
and the slice `rewards[..count]` contains exactly the count valid entries. The ring-buffer
mean is therefore actually correct. **Withdrawing CR-01 as a blocker.**

---

### CR-01 (revised): Inverted reward signal for maximization problems

**File:** `src/engines/ga.rs:2769-2791`

**Issue:** The reward computation for maximization mode transposes parent and child
fitness, then calls `compute_normalized_reward(p, c, best_fitness)` which uses the
formula `delta = parent_fitness - offspring_fitness`. For a minimization problem a
positive delta means the child improved (lower is better). For maximization the code
correctly swaps arguments so that `p = child_1.fitness()` and `c = parent_1.fitness()`,
giving `delta = child_fitness - parent_fitness`, which is positive when the child is
better (higher is better). However `compute_normalized_reward` normalises by
`best_fitness.abs()`, and `best_fitness` is passed from `best_fitness_so_far` which is
the raw fitness from `self.population.best_chromosome.fitness()`.

The problem is the denominator. For a maximization problem the best fitness is a large
positive number (good), and rewards are divided by that large value, producing small
reward magnitudes. For a minimization problem the best fitness trends toward zero, and
`best_fitness.abs().max(f64::EPSILON)` trends toward `f64::EPSILON`, producing
astronomically large reward magnitudes (rewards blow up as the population converges).

This means AOS rewards are not bounded in the minimization case as the GA converges,
causing the probability updates to be dominated by the final few generations and
distorting operator selection history. PM and AP use these rewards directly; MAB uses
them for UCB ranking which also distorts as values scale arbitrarily.

Additionally, for the mutation reward (lines 2781-2790) the code reuses `parent_1`
(the first parent) as the baseline regardless of which parent was actually mutated to
produce `child_2`. Only `child_1`'s reward is computed for both the crossover and
mutation accumulators, meaning `child_2`'s contribution is never measured. This
halves the reward signal for every couple.

**Fix — denominator clamping and child_2 reward:**
```rust
// Crossover reward: compare both children
if let Some(ref acc) = crossover_reward_acc {
    if let Some((c_op_idx, _)) = selected_crossover {
        let reward1 = if is_maximization {
            crate::aos::compute_normalized_reward(child_1.fitness(), parent_1.fitness(), best_fitness)
        } else {
            crate::aos::compute_normalized_reward(parent_1.fitness(), child_1.fitness(), best_fitness)
        };
        let reward2 = if is_maximization {
            crate::aos::compute_normalized_reward(child_2.fitness(), parent_2.fitness(), best_fitness)
        } else {
            crate::aos::compute_normalized_reward(parent_2.fitness(), child_2.fitness(), best_fitness)
        };
        let avg_reward = (reward1 + reward2) / 2.0;
        acc.lock().unwrap().push((c_op_idx, avg_reward));
    }
}
```

For the denominator explosion, consider normalizing rewards to a fixed range (e.g.,
clamp to [-1.0, 1.0]) before recording, or use a relative improvement ratio
`(parent - child) / (1.0 + parent.abs())` instead of dividing by `best_fitness.abs()`.

### CR-02: `AosState` is not `Send` when `Mutex` is on `wasm32` but `Mutex<AosState>` is stored unconditionally

**File:** `src/engines/ga.rs:319-329`

**Issue:** `Ga` stores `aos_crossover: Option<Mutex<AosState>>` and
`aos_mutation: Option<Mutex<AosState>>` unconditionally. `std::sync::Mutex` is available
on wasm32 (it is part of `std`), but `AosState` contains `Vec<ArmState>` which holds
`Vec<f64>` — these are all `Send`. This compiles fine. However, the `process_pair`
closure (line 2550) captures `aos_crossover_state: Option<&Mutex<AosState>>` and is
dispatched via `parents.par_iter()` on non-wasm32 (line 2798). The closure holds a
shared reference to a `Mutex`, calls `.lock().unwrap()` to mutate state, then immediately
drops the guard — this is a legitimate use of `Mutex` with rayon.

The real problem is that on the **wasm32** path (line 2800), `process_pair` is called
sequentially, and each call locks the mutex, selects an operator, and immediately
unlocks. Because this is single-threaded there is no deadlock risk, but the lock is
acquired and released once per parent pair, which is valid.

**Actual finding:** No correctness bug here — the Mutex usage is safe on both paths.
**Withdrawing CR-02.**

---

**Restating the valid critical finding count as 1.**

---

## Critical Issues

### CR-01: Reward signal blows up at convergence (denominator approaches zero)

**File:** `src/aos.rs:388-396`, `src/engines/ga.rs:2776`

**Issue:** `compute_normalized_reward` divides by `best_fitness.abs().max(f64::EPSILON)`.
For minimization problems, as the GA converges toward the optimum, `best_fitness` trends
toward 0 (or is already near 0 for problems like the demo where the minimum is 0).
When `best_fitness` is small, the denominator is `f64::EPSILON ≈ 2.2e-16`, causing
rewards to blow up to values on the order of `delta / 2.2e-16`. In the AOS demo
(minimizing the sum of integer genes to 0), once the best chromosome reaches fitness 0
after a few generations, all rewards become infinite or `f64::MAX`-scale values. These
unbounded rewards flow into `add_reward()` (which stores them directly), and then
`mean_reward()` returns an infinite or NaN value, corrupting `update_pm()` probability
calculations: `alpha * (credit - *prob)` where `credit = f64::INFINITY` produces
`f64::INFINITY` for `new_prob`, which survives `clamp(p_min, 1.0)` as `1.0`, then
`normalize_probabilities()` divides by the sum — once one arm has `1.0` and others
have `p_min`, normalization works, but if all arms receive infinite rewards (because
`best_fitness ≈ 0` for all), the sum can also be infinite or NaN, and the division
produces `NaN` for all probabilities. From that point on `roulette_wheel_select` returns
`num_arms - 1` always (the fallback), breaking adaptive selection.

**Fix:** Use a relative improvement formula that does not depend on `best_fitness`
magnitude:
```rust
pub fn compute_normalized_reward(
    parent_fitness: f64,
    offspring_fitness: f64,
    _best_fitness: f64,  // kept in signature for API compat
) -> f64 {
    // Relative improvement: positive = better (lower for minimization)
    let denom = 1.0 + parent_fitness.abs();
    (parent_fitness - offspring_fitness) / denom
}
```
This keeps rewards bounded in `(-1, 1)` and does not blow up as fitness approaches 0.

### CR-02: Only `child_1` reward is recorded; `child_2` reward silently discarded

**File:** `src/engines/ga.rs:2769-2791`

**Issue:** The crossover and mutation reward accumulators compute and record a reward for
`child_1` against `parent_1` (lines 2769-2778 and 2781-2790). `child_2` is produced by
the same crossover and mutation operators but its reward is never computed or recorded.
Since every parent pair produces two children and the selected operator indices are
shared for both, only half of the actual operator outcomes are fed back into AOS. This
biases the reward signal — if `child_2` consistently does better or worse than `child_1`,
the AOS model receives a systematically skewed view of each operator's performance.

**Fix:** Record rewards for both children:
```rust
if let Some(ref acc) = crossover_reward_acc {
    if let Some((c_op_idx, _)) = selected_crossover {
        let reward1 = crate::aos::compute_normalized_reward(
            parent_1.fitness(), child_1.fitness(), best_fitness);
        let reward2 = crate::aos::compute_normalized_reward(
            parent_2.fitness(), child_2.fitness(), best_fitness);
        let mut lock = acc.lock().unwrap();
        lock.push((c_op_idx, reward1));
        lock.push((c_op_idx, reward2));
    }
}
```

---

## Warnings

### WR-01: `learning_rate` field in `ProbabilityMatching` is completely unused

**File:** `src/aos.rs:31`, `src/aos.rs:258-262`

**Issue:** `AosStrategy::ProbabilityMatching { alpha, learning_rate }` stores a
`learning_rate` field, but in `update_pm()` the match arm binds it as `learning_rate: _`
(line 261) and only `alpha` is passed to `update_pm`. The `learning_rate` parameter is
documented as "Step size for probability updates" but plays no role in any computation.
Users who configure `learning_rate` (e.g., via `AosStrategy::ProbabilityMatching { alpha: 0.8, learning_rate: 0.1 }`) will find their value silently ignored, which is confusing and
could mask bugs.

**Fix:** Either remove the field from the variant and its documentation, or implement
the learning rate in `update_pm`:
```rust
fn update_pm(&mut self, alpha: f64, learning_rate: f64) {
    let p_min = 1.0 / (self.num_arms.max(1) as f64 * 1.5);
    let total_reward: f64 = (0..self.num_arms).map(|i| self.arms[i].mean_reward().max(0.0)).sum();
    let denom = total_reward.max(f64::EPSILON);
    for i in 0..self.num_arms {
        let credit = self.arms[i].mean_reward();
        // Alpha blends toward observed credit; learning_rate scales the update step
        let q = credit / denom;  // normalized quality
        let new_prob = (1.0 - learning_rate) * self.probabilities[i] + learning_rate * q;
        let blended = self.probabilities[i] + alpha * (new_prob - self.probabilities[i]);
        self.probabilities[i] = blended.clamp(p_min, 1.0);
    }
    self.normalize_probabilities();
}
```
Or simply remove `learning_rate` from the variant and its documentation.

### WR-02: Mutex lock held across the entire `process_pair` closure body on the hot path

**File:** `src/engines/ga.rs:2587-2591`, `src/engines/ga.rs:2595-2600`

**Issue:** Inside `process_pair`, the AOS operator selection block acquires the Mutex
(`aos_state.lock().unwrap()`), calls `state.select_operator(...)`, then the guard is
dropped. This is correct for selection. However, the lock is acquired inside a closure
that is dispatched by `par_iter()` across all parent pairs. Under rayon with many
threads, every thread contends on the same `Mutex<AosState>` for every parent pair. For
large populations with many couples this can serialize a significant fraction of the
parallel crossover work. While not a correctness bug, it degrades the scalability
benefit of rayon and contradicts the documented promise of parallelism for the
crossover step.

The reward accumulator (`crossover_reward_acc: Arc<Mutex<Vec<...>>>`) has the same
contention issue at lines 2769-2778.

**Fix:** Pre-select all operator indices before the `par_iter()` dispatch, storing them
in a `Vec<(usize, usize)>` (one per parent pair), and pass the pre-selected indices into
each closure via index. This eliminates Mutex contention in the hot path. The
`record_rewards` + `update` step can remain sequential after collecting all offspring.

### WR-03: `with_reward_window(0)` causes a panic at runtime via division by zero in `AosState::new`

**File:** `src/aos.rs:88-99`

**Issue:** `ArmState::new(window_size)` creates `rewards: vec![0.0; window_size]`. When
`window_size == 0`, the `Vec` is empty. `add_reward` then does:
```rust
self.rewards[self.cursor] = reward;  // index 0 into empty Vec → PANIC
self.cursor = (self.cursor + 1) % self.rewards.len();  // division by zero if len==0
```
`with_reward_window(0)` is callable by users (no validation in `build()`), and
`AosState::new` is called from `run_with_callback` with the user-supplied window value.

**Fix:** Add validation in `build()`:
```rust
if self.configuration.aos_reward_window == 0
    && (self.configuration.crossover_portfolio.is_some()
        || self.configuration.mutation_portfolio.is_some())
{
    return Err(GaError::ConfigurationError(
        "aos_reward_window must be >= 1 when using AOS portfolios".to_string(),
    ));
}
```
Or guard defensively in `ArmState::new` and `add_reward`:
```rust
fn new(window_size: usize) -> Self {
    let size = window_size.max(1);
    ArmState { rewards: vec![0.0; size], cursor: 0, count: 0 }
}
```

### WR-04: `AosState` not re-initialized between `run()` calls; stale reward history persists

**File:** `src/engines/ga.rs:1447-1459`

**Issue:** `run_with_callback` initializes `self.aos_crossover` and `self.aos_mutation`
at lines 1447-1459 every time it is called. This is correct for the first run. However,
`Ga` is designed to support multiple sequential calls to `run()` (e.g., resuming from
checkpoint or calling run again after inspecting results). On a second `run()` call,
the AOS state is reset because it is always re-initialized inside `run_with_callback`.
This part is fine. But the AOS state fields on `Ga` (lines 319-329) retain their
previous values between `build()` and the first `run()`, and after a `run()` completes,
the `Mutex<AosState>` stores the accumulated state from the completed run. If the user
calls `run()` a second time, the state is overwritten at init — this is correct.

**The actual issue:** the AOS state is initialized inside `run_with_callback` **after**
the checkpoint resumption block. If a checkpoint is loaded, the `checkpoint_generation`
variable is set, and `start_gen` will be non-zero. The AOS state starts fresh from
generation 0 semantics (`exploration_generations = window_size / 2`), but the loop
variable `i` starts from `checkpoint_generation`. This means the exploration phase
comparison `generation < exploration_generations` (in `select_operator`) compares `i`
(which may be e.g. 500) against `exploration_generations` (e.g. 25), so the exploration
phase is immediately skipped on checkpoint resume. This is likely the intended behavior,
but it is undocumented and untested.

**Fix:** Document this behavior explicitly in a comment in `run_with_callback` near
line 1447, and add a test for AOS state after checkpoint resume.

---

## Info

### IN-01: `c` field in `AdaptivePursuit` variant is unused

**File:** `src/aos.rs:37`, `src/aos.rs:264-265`

**Issue:** `AosStrategy::AdaptivePursuit { beta, c }` has a field `c` documented as
"Scaling factor", but in `update()` the match arm binds it as `c: _` (line 265), and
`update_ap` only receives `beta`. The field is accepted by users and by `ap_default()`
(which sets `c: 1.5`) but is never used in any computation. Like `learning_rate` in PM,
this silently discards a user-configured parameter.

**Fix:** Either remove `c` from the `AdaptivePursuit` variant, or implement it in
`update_ap` as a scaling factor for the reward signal prior to the pursuit update.

### IN-02: Tests in `test_aos.rs` duplicate tests already present in `src/aos.rs` inline tests

**File:** `tests/engines/aos/test_aos.rs:195-248`

**Issue:** The test functions `test_compute_normalized_reward_positive`,
`test_compute_normalized_reward_negative`, `test_compute_normalized_reward_zero_delta`,
`test_compute_normalized_reward_zero_best`, and `test_compute_normalized_reward_best_nonzero`
in the integration test file are identical (same values, same assertions) to the tests
in `src/aos.rs:676-732`. The CLAUDE.md project convention specifies tests should be in
`tests/`, not inline — so the `src/aos.rs` inline tests (`#[cfg(test)] mod tests`)
contradict the project standard. The inline tests in `src/aos.rs` should be removed and
only the `tests/` versions kept.

**Fix:** Remove the `#[cfg(test)] mod tests` block from `src/aos.rs` (lines 398-732),
keeping only the `tests/engines/aos/test_aos.rs` versions.

---

_Reviewed: 2026-06-02_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
