---
phase: 75
status: partially_fixed
severity_counts:
  critical: 1
  high: 1
  medium: 2
  low: 3
reviewed_at: 2026-06-19
fixed_at: 2026-06-19
fixed_findings:
  - CRITICAL: MuCommaLambda wipes population every generation (commit b104b46)
  - HIGH: AOS select_operator called before crossover probability roll (commit c369afd)
---

# Phase 75 Code Review

## Summary

Phase 75 added `Copy` derives to mutation types, converted `parent_crossover` to an output-buffer API, changed crossover-fail behavior to produce zero offspring (instead of parent clones), and changed `extract_elite` to return indices. The changes are mechanically sound but introduce two behavioral regressions and surface one pre-existing critical bug.

---

## Findings

### [CRITICAL] MuCommaLambda wipes population every generation

**File:** `src/engines/ga/generation.rs:331` + `src/operations/survivor/mu_comma_lambda.rs:44`

`MuCommaLambda` retains `age == 0` (offspring). But offspring are stamped with `child_1.set_age(age)` where `age` is the loop counter **after** `age += 1` — so offspring always have `age >= 1`. No chromosome in the merged population ever has `age == 0`. Every generation, `retain` removes every individual; the population collapses to an empty `Vec`. The next selection call panics or silently produces zero parents.

Pre-existing bug (not introduced by Phase 75), but Phase 75's zero-offspring case makes it visible on the first generation rather than hiding behind the parent-clone fallback.

**Fix:** Either stamp offspring with age `0` (born-this-generation semantics) or change the `retain` predicate to `age >= current_generation - 1`.

---

### [HIGH] AOS `select_operator` called before crossover probability roll — pull counts inflated, rewards never recorded

**File:** `src/engines/ga/generation.rs:154–174` (AOS lock + `select_operator`) vs `line 217` (early return)

**Phase 75 regression.** Before Phase 75, failed crossover pairs still produced parent clones, reached the reward accumulation block, and pushed a (near-zero) reward — keeping pull counts and rewards in sync. Phase 75's `return Ok(Vec::new())` at line 217 exits before the reward block at lines 336–358.

Now, every skipped pair increments `selection_counts[op_idx]` in the AOS state without any corresponding reward. Under UCB1 the exploration bonus shrinks as `n` grows — operators selected on skipped pairs accumulate phantom pull counts, biasing future selection away from them even though no feedback was received. With 30% crossover probability, 70% of pairs corrupt AOS state every generation.

**Fix:** Move AOS operator selection to after the probability roll gate (after line 221), or record a neutral reward (0.0) for skipped pairs.

---

### [MEDIUM] Mutation AOS reward always measures `child_1` regardless of which child was mutated

**File:** `src/engines/ga/generation.rs:349–357`

Each child has its own independent mutation probability roll: `child_1` at line 268, `child_2` at lines 292–293 (fresh `rng.random_range`). The mutation AOS reward block unconditionally computes `child_1.fitness()` vs `parent_1.fitness()`. When `child_1`'s roll fails but `child_2`'s passes (probability `(1-p) * p` per pair), the reward measures the unmutated child_1 — attributing crossover-only fitness change as the mutation operator's effect. Over many generations this degrades AOS mutation-operator ranking.

**Fix:** Record the reward for whichever child was mutated (or average both), gated on the respective roll outcome.

---

### [MEDIUM] Multi-parent crossover injects `parent_2.clone()` as `child_2` into offspring every generation

**File:** `src/engines/ga/generation.rs:260`

`factory_multi_parent_dispatch` (UNDX/SPX/PCX) returns 1 child. `children.pop()` yields the real offspring as `child_1`; the second `pop()` returns `None`, so `child_2 = parent_2.clone()`. Both are pushed into `offspring_buf` via `Ok(vec![child_1, child_2])`. Every multi-parent pair injects one parent duplicate into the offspring pool — the duplicate occupies a survivor slot and degrades population diversity each generation.

Additionally, lines 320–329 unconditionally inject a new fitness closure into `child_2` and call `calculate_fitness()` on the parent's unchanged DNA — a full spurious fitness evaluation per multi-parent pair per generation.

**Fix:** Return only `child_1` for multi-parent paths (change the `Ok(vec![child_1, child_2])` to `Ok(vec![child_1])`). The fitness-fn injection and re-evaluation for parent clones should also be gated on `child_2` being a genuine offspring.

---

### [LOW] Surrogate prescreening rebinds `offspring_buf` via `.collect()`, permanently losing D-08 pre-allocated capacity

**File:** `src/engines/ga/mod.rs:1548`

```rust
offspring_buf = scores.into_iter().map(|(idx, _)| offspring_buf[idx].clone()).collect();
```

This creates a new `Vec<U>` sized to `keep` (≤ pop_size), discarding the `Vec::with_capacity(pop_size * 2)` allocated at line 1436. `Vec::append` in `add_chromosomes` drains to len=0 but preserves the new, smaller capacity. On the next generation, `out.clear()` operates on this small buffer, and `out.extend` reallocates once offspring count exceeds `keep`.

The D-08 comment ("allocate once, reuse every generation") is silently false whenever surrogate fires.

**Fix:**
```rust
offspring_buf.clear();
for (idx, _) in scores {
    offspring_buf.push(offspring_buf_snapshot[idx].clone());
}
```
Or retain indices in-place and swap. Preserve the binding rather than rebinding.

---

### [LOW] `child_2.set_age(age)` resets parent_2 clone's age in multi-parent path, corrupting ALPS layer assignment

**File:** `src/engines/ga/generation.rs:332`

In the multi-parent 1-child path, `child_2 = parent_2.clone()` inherits the parent's true age (the generation it was created). `set_age(age)` then stamps it with the current generation counter — typically lower than the parent's real age. Under ALPS, this makes the parent duplicate appear as a fresh offspring, escaping age-based elimination and potentially displacing genuine young individuals from the youngest layer.

**Fix:** Skip `set_age` for the parent-clone fallback, or only call it when `child_2` is a genuine offspring (i.e., `children` had 2 elements).

---

### [LOW] `best_fitness_so_far` updated from niching-adjusted fitness, poisoning AOS reward normalization

**File:** `src/engines/ga/mod.rs` (best-chromosome update after niching write-back)

When niching is active, the sharing-adjusted fitness is written back to all chromosomes before `best_chromosome` is cloned. `best_fitness_so_far` is then derived from the niche-penalized value. Next generation, `compute_normalized_reward` uses this as the normalization denominator, producing incorrect AOS rewards. Stagnation detection fires on niche-penalized values, not raw fitness, potentially triggering false stagnation.

Pre-existing interaction, not introduced by Phase 75. Surfaced here because Phase 75 added explicit AOS reward normalization paths.

---

## Not Bugs

- **AOS reward argument swap for maximization** (generation.rs:338): the `(child, parent)` order is intentional — `compute_normalized_reward` assumes lower-is-better, so for maximization the arguments are swapped to preserve sign semantics. Correct.
- **`mutation_method.mutate(&mut child, &mutation_method)`**: the `Mutation` impl dispatches entirely on the second argument and ignores `self`. The self-aliasing is harmless. Pre-existing pattern.
- **`extract_elite` index validity**: indices are derived after `add_chromosomes` and cloned immediately before survivor selection reorders the population. Ordering is correct.
- **`out.clear()` + `Vec::append` interaction**: `append` drains `offspring_buf` to len=0 preserving capacity (the surrogate case aside). The `out.clear()` at `parent_crossover` entry is a correct defensive guard.
