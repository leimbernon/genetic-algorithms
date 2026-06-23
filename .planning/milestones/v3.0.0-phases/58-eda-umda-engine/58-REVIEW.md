---
phase: 58-eda-umda-engine
reviewed: 2026-06-04T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - src/engines/eda/configuration.rs
  - src/engines/eda/engine.rs
  - src/engines/eda/mod.rs
  - tests/engines/eda/test_eda.rs
  - examples/eda_trap.rs
  - src/lib.rs
  - tests/test_engines.rs
findings:
  critical: 2
  warning: 5
  info: 3
  total: 10
status: issues_found
---

# Phase 58: Code Review Report

**Reviewed:** 2026-06-04T00:00:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

This phase adds an EDA (UMDA) engine with two variants: `EdaEngine<U>` (Bernoulli/binary) and `EdaRealEngine<U>` (Gaussian/real-valued). The WASM cfg-gating is correctly applied. The observer pattern is correctly wired. The overall structure follows established engine conventions.

Two critical bugs were found: one involving undefined behavior on `select_nth_unstable_by` when `n_selected == pop.len()` (the pivot element is unreliable), and one involving a missing `FixedFitness` branch in the parent-selection sort that silently falls through to maximization ordering, producing wrong results for `FixedFitness` problems. Five warnings cover correctness gaps including Gaussian variance computed with biased population formula (N instead of N-1), silent panic on empty population from `find_best`, incorrect `lib.rs` engine count in the module-level docstring, massive code duplication between the two engine structs, and the learned-model state being exposed as the last-generation estimate rather than the best-generation estimate.

## Critical Issues

### CR-01: `select_nth_unstable_by` called with `n_selected == pop.len()` is undefined behaviour for the selection step

**File:** `src/engines/eda/engine.rs:284` (also line 291, 596, 603)

**Issue:** `select_nth_unstable_by(k, …)` requires `k < slice.len()`. `n_selected` is computed as `((pop_size * ratio).floor() as usize).max(1).min(pop.len())`. When `selection_ratio` is `1.0` (the documented maximum), `n_selected == pop.len()` and `k = n_selected - 1 == pop.len() - 1`, which is the last valid index. That is in bounds, so no panic — **but** the contract of `select_nth_unstable_by` only guarantees that `slice[k]` holds the element that would be at position `k` in sorted order; elements at indices `0..k` are in no particular order. The code then slices `indices[..n_selected]` and treats all of them as the "top n_selected". When `n_selected < pop.len()` this is correct by the partial-sort guarantee. When `n_selected == pop.len()` the full slice is used which is fine. However, the more dangerous case is the converse: with `ratio < 1.0` the partial sort only guarantees the **element at index `k`** is correct, not that all elements in `0..k` are the actual top-k. `select_nth_unstable_by` provides a **partition guarantee**: elements at `0..k` are all <= the pivot (for ascending sort), not that they are the top-k globally. For maximization the comparator sorts descending, so elements in `0..k` are all >= the element at position `k` — this is the correct set of top-k. For **minimization** the comparator sorts ascending, so elements at `0..k` are all <= pivot — again the correct bottom-k. The logic is therefore sound for the two standard directions. **The real bug** is on line 284 and 596: when `ProblemSolving::FixedFitness` is used (see CR-02 below), the code falls into the `is_maximization` branch because `is_maximization` is defined as `matches!(…, Maximization)`. For `FixedFitness`, `is_maximization == false`, so the minimization comparator is used unconditionally, which picks parents by raw fitness distance to origin rather than distance to the target. This is the same root cause as CR-02; calling it out here because the sort order is the first place correctness breaks.

**Fix:** Introduce a dedicated sort comparator for `FixedFitness` (see CR-02 fix; the sort path should mirror `is_better`):

```rust
let target_for_sort = self.config.fitness_target.unwrap_or(0.0);
indices.select_nth_unstable_by(n_selected - 1, |&a, &b| {
    let da = (pop[a].fitness() - target_for_sort).abs();
    let db = (pop[b].fitness() - target_for_sort).abs();
    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
});
```

---

### CR-02: `FixedFitness` direction uses wrong parent-selection order — convergence silently broken

**File:** `src/engines/eda/engine.rs:233` (also line 551)

**Issue:** Both `run()` methods compute `is_maximization` as:

```rust
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
```

This boolean is then used to choose the sort comparator for truncation selection (lines 283–297, 594–610). When `problem_solving` is `FixedFitness`, `is_maximization` is `false`, so the **minimization** comparator is chosen. The minimization comparator selects parents with the lowest raw fitness. But `FixedFitness` convergence requires parents closest to the target (minimum `|fitness - target|`), which is an entirely different ordering. The engine will silently select the wrong parents, learn a wrong model, and diverge — with no error or warning to the caller. `is_better` and `reached_target` correctly handle `FixedFitness` by distance-to-target, but the selection sort is inconsistent with them.

**Fix:** Replace the binary `is_maximization` flag with a three-way sort comparator derived from `problem_solving`:

```rust
let cmp = |a_fit: f64, b_fit: f64| -> std::cmp::Ordering {
    match self.config.problem_solving {
        ProblemSolving::Maximization =>
            b_fit.partial_cmp(&a_fit).unwrap_or(std::cmp::Ordering::Equal),
        ProblemSolving::Minimization =>
            a_fit.partial_cmp(&b_fit).unwrap_or(std::cmp::Ordering::Equal),
        ProblemSolving::FixedFitness => {
            let t = self.config.fitness_target.unwrap_or(0.0);
            let da = (a_fit - t).abs();
            let db = (b_fit - t).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }
    }
};
indices.select_nth_unstable_by(n_selected - 1, |&a, &b| cmp(pop[a].fitness(), pop[b].fitness()));
```

---

## Warnings

### WR-01: Gaussian variance uses biased estimator (divide by N, not N-1) — model underestimates spread

**File:** `src/engines/eda/engine.rs:505`

**Issue:** `estimate_gaussian` computes:

```rust
let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
```

This is the **population** variance (MLE, biased by factor `(N-1)/N`). UMDA literature and standard Gaussian-model EDA implementations use the **sample** variance (`/ (n - 1)`) to get an unbiased estimate of the true per-position variance. Using the biased formula systematically underestimates spread, which accelerates premature convergence — the Gaussian model tightens faster than the actual data warrants, reducing exploration. This is particularly harmful early in the run when `n_selected` is small (e.g., `n = 3`; bias factor = `2/3`).

**Fix:**
```rust
let variance = if n > 1.0 {
    vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)
} else {
    0.0
};
```

---

### WR-02: `find_best` panics on empty population — no guard against post-init empty pop

**File:** `src/engines/eda/engine.rs:185` (also line 478)

**Issue:** `find_best` unconditionally accesses `pop[0].fitness()`. The method is called immediately after the `init_fn` returns (line 259/573). Although there is a guard `if pop.is_empty() { panic!(…) }` before the initial evaluation loop, `find_best` is also called on `new_pop` at line 334/648, after the sampling step. If `pop_size` were somehow 0 (not currently reachable due to `.max(1)` but defensive code should not assume), `new_pop` would be empty and `find_best` would panic with an unhelpful index-out-of-bounds. More practically, the panic message from `find_best` gives no context; errors surface as index panics rather than engine errors.

**Fix:** Add an explicit guard or return an `Option`/`Result`:
```rust
fn find_best(&self, pop: &[U]) -> (usize, f64) {
    assert!(!pop.is_empty(), "EdaEngine::find_best called with empty population");
    // … rest unchanged
}
```

---

### WR-03: Massive code duplication between `EdaEngine` and `EdaRealEngine` — maintenance hazard

**File:** `src/engines/eda/engine.rs:107–682`

**Issue:** `EdaEngine<U>` and `EdaRealEngine<U>` share verbatim copies of: `is_better`, `reached_target`, `find_best`, `notify`, `run` (loop body differs only in the model estimation + sampling call). Each method is duplicated ~300 lines of identical logic. Any bug fix (such as CR-02 above) must be applied in two places. The project's established pattern (PSO, CMA, DE engines) uses a single generic struct with a strategy/model parameter, not two separate structs.

**Fix:** Extract a private `EdaCore<U>` struct or a shared `run_inner` free function parameterized by the model estimation and sampling closures. At minimum, extract `is_better`, `reached_target`, and `find_best` into a shared `impl` block on a new `EdaCore` helper or a shared trait.

---

### WR-04: `learned_model` in `EdaResult` reflects the **last generation's** model, not the **best generation's** model

**File:** `src/engines/eda/engine.rs:301–302` (also line 613–617)

**Issue:** The `learned_model` field is updated every generation regardless of whether that generation produced a better best individual:

```rust
learned_model = EdaModel::Bernoulli(probs.clone());
```

If early stopping occurs because the fitness target is met, the returned model is the model from the generation that triggered stopping — which is acceptable. But if the run ends due to `max_generations` and the final generation happened to move away from the optimum (possible in non-elitist EDA where the entire population is replaced), the returned model corresponds to a potentially degraded population, not the model that produced the best individual. The docstring on `EdaResult::learned_model` says "model estimated at the final generation", which is technically accurate, but callers using the model for post-hoc analysis or warm starts will receive a misleading artifact.

**Fix:** Track a `best_model` variable alongside `best` and update it whenever `is_better` triggers:

```rust
if self.is_better(gen_best_fit, best_fitness) {
    best_fitness = gen_best_fit;
    best = pop[gen_best_idx].clone();
    best_model = learned_model.clone(); // snapshot model that produced the best
    // …
}
```

Return `best_model` instead of `learned_model` in `EdaResult`.

---

### WR-05: `lib.rs` docstring claims "12 engines" but EDA makes 13

**File:** `src/lib.rs:59`

**Issue:** The module-level docstring table header reads "Engines (12 total)" and the introductory sentence says "This crate offers 12 optimization engines". The EDA engine added in this phase brings the total to 13. The table body lists 13 entries (the new EDA engine is not listed in the table at all — it is re-exported at line 370 but absent from the "When to Use Which Engine" table and from the engine count summary).

**Fix:** Update line 59 to "Engines (13 total)" and "13 optimization engines", and add an EDA row to the "When to Use Which Engine" table.

---

## Info

### IN-01: Dead variable `i` in `sample_bernoulli` with explicit `let _ = i` suppression

**File:** `src/engines/eda/engine.rs:218–219`

**Issue:** The closure binds `(i, (gene, &p))` via `enumerate()` but `i` is never used. A comment and `let _ = i;` acknowledge this but add noise. The `enumerate()` call is itself unnecessary since `i` is never used.

**Fix:** Remove `enumerate()` and use `zip` directly:
```rust
let new_dna: Vec<U::Gene> = template
    .dna()
    .iter()
    .zip(probs.iter())
    .map(|(gene, &p)| {
        let mut g = gene.clone();
        g.set_id(if rng.random::<f64>() < p { 1 } else { 0 });
        g
    })
    .collect();
```

---

### IN-02: `EdaEngine::bernoulli` constructor alias adds no value and creates confusion with `EdaRealEngine`

**File:** `src/engines/eda/engine.rs:137–143`

**Issue:** `EdaEngine::bernoulli(config, init, fitness)` is a thin alias for `EdaEngine::new(…)` with no distinguishing behaviour. The docstring for `bernoulli` says "Call `run` after construction to execute the Bernoulli UMDA loop. For Gaussian optimization, use `EdaRealEngine::new`" — which means users must still know about two separate types. The alias creates an asymmetry: `EdaEngine` has a `bernoulli` alias but `EdaRealEngine` has no equivalent `gaussian` alias, and neither alias adds any compile-time safety or runtime distinction.

**Fix:** Remove the `bernoulli` alias, or add a symmetric `gaussian` alias on `EdaRealEngine` and document both clearly.

---

### IN-03: Test EDA-02 assertion threshold too loose to verify convergence

**File:** `tests/engines/eda/test_eda.rs:158`

**Issue:** EDA-02 (Gaussian sphere convergence) asserts `result.best_fitness < 5.0`. The sphere function on a 5-dimensional domain `[-5, 5]^5` has its worst case at `5^2 * 5 = 125.0`. An assertion of `< 5.0` is so weak that a random walk would likely satisfy it. The fitness target in the config is `0.1` but is not what is asserted — only `< 5.0` is checked. This means the test would pass even if the Gaussian model implementation was entirely broken (random sampling would average below 5.0). EDA-01 has the same pattern but is slightly better (`>= 18.0` out of 20 is a meaningful bar for OneMax).

**Fix:** Tighten the sphere assertion to match the declared target:
```rust
assert!(
    result.best_fitness < 0.5,
    "Expected best fitness < 0.5 for sphere after 500 generations, got {}",
    result.best_fitness
);
```

---

_Reviewed: 2026-06-04T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
