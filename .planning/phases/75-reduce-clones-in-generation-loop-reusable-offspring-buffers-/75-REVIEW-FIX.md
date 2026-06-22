---
phase: 75
fixed_at: 2026-06-19T00:00:00Z
review_path: .planning/phases/75-reduce-clones-in-generation-loop-reusable-offspring-buffers-/75-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 75: Code Review Fix Report

**Fixed at:** 2026-06-19
**Source review:** `.planning/phases/75-reduce-clones-in-generation-loop-reusable-offspring-buffers-/75-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (Critical + High only)
- Fixed: 2
- Skipped: 0

## Fixed Issues

### CRITICAL: MuCommaLambda wipes population every generation

**Files modified:** `src/operations/survivor/mu_comma_lambda.rs`, `tests/operations/test_survivor_mu_comma_lambda.rs`
**Commit:** b104b46
**Applied fix:** Replaced `retain(|c| c.age() == 0)` with a two-step approach: compute `offspring_age = chromosomes.iter().map(|c| c.age()).max().unwrap_or(0)`, then `retain(|c| c.age() == offspring_age)`. Since the GA loop stamps offspring with `set_age(age)` where `age` is the generation counter (starting at 1), offspring always have the maximum age value in the merged population. Parents from prior generations have strictly lower age values. The old `age == 0` predicate never matched any individual, silently emptying the population every generation.

Updated all five tests in `tests/operations/test_survivor_mu_comma_lambda.rs` to use age values reflecting real-world GA stamping: offspring receive the current generation counter as their age (higher values), parents have lower prior-generation ages. The "no offspring" edge-case test was revised to use same-age-for-all scenario (all treated as current generation's youngest).

### HIGH: AOS `select_operator` called before crossover probability roll

**Files modified:** `src/engines/ga/generation.rs`
**Commit:** c369afd
**Applied fix:** Moved the `selected_crossover` and `selected_mutation` computation blocks from before the crossover probability gate to immediately after the `return Ok(Vec::new())` early return. The probability roll variables (`crossover_probability`, `effective_crossover_prob`, `mutation_probability`, `effective_mutation_prob`) remain before the gate since they are needed for it. All downstream usages of `selected_crossover` (crossover dispatch and reward block) and `selected_mutation` (mutation dispatch and reward block) are unaffected since they follow the gate. Added a detailed comment explaining why the ordering is intentional.

---

_Fixed: 2026-06-19_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
