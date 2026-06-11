---
phase: 31-selection-survivor-diversity-operators
reviewed: 2026-05-04T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - src/configuration.rs
  - src/engines/ga.rs
  - src/operations.rs
  - src/operations/selection.rs
  - src/operations/selection/clearing.rs
  - src/operations/survivor.rs
  - src/operations/survivor/deterministic_crowding.rs
  - src/traits/configuration.rs
  - tests/observe/test_serde.rs
  - tests/operations/test_selection_clearing.rs
  - tests/operations/test_survivor_deterministic_crowding.rs
  - tests/test_operations.rs
findings:
  critical: 2
  warning: 2
  info: 1
  total: 5
status: issues_found
---

# Phase 31: Code Review Report

**Reviewed:** 2026-05-04T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

This phase adds two new diversity-promoting operators: `Clearing` parent selection and `DeterministicCrowding` survivor selection. The core algorithmic logic is sound. However, there are two critical defects: (1) the new `Selection::Clearing` and `Survivor::DeterministicCrowding` variants are missing from the serde round-trip tests, meaning serialized configurations using these variants will fail to deserialize in checkpoint/restore scenarios even though the types derive `Serialize/Deserialize`; and (2) `clearing_selection` silently ignores the `number_of_couples` configuration, producing an unpredictable pair count that diverges from the contract upheld by every other selection operator.

---

## Critical Issues

### CR-01: `Selection::Clearing` and `Survivor::DeterministicCrowding` omitted from serde tests

**File:** `tests/observe/test_serde.rs:34-98`

**Issue:** `serde_selection_enum` (line 34) tests every `Selection` variant except `Selection::Clearing`. `serde_survivor_enum` (line 88) tests every `Survivor` variant except `Survivor::DeterministicCrowding`. Both new variants derive `Serialize/Deserialize` alongside the enum body, so the derive is present — but its correctness is never exercised.

A checkpoint saved with `method: Selection::Clearing` or `survivor: Survivor::DeterministicCrowding` could silently deserialize to a wrong variant if the serde representation is unexpectedly altered in a refactor (e.g., a rename, a reorder, or an added `#[serde(rename)]` elsewhere in the enum). The gap also means a broken derive would ship without any test failure.

**Fix:** Add the two missing variants to the existing test arrays:

```rust
// serde_selection_enum
let variants = [
    Selection::Random,
    Selection::RouletteWheel,
    Selection::Tournament,
    Selection::Boltzmann,
    Selection::Truncation,
    Selection::Rank,
    Selection::StochasticUniversalSampling,
    Selection::Clearing,          // ADD
];

// serde_survivor_enum
let variants = [
    Survivor::Fitness,
    Survivor::Age,
    Survivor::MuPlusLambda,
    Survivor::MuCommaLambda,
    Survivor::DeterministicCrowding,  // ADD
];
```

---

### CR-02: `clearing_selection` ignores `number_of_couples` — produces silently wrong pair count

**File:** `src/operations/selection/clearing.rs:27-98`

**Issue:** `clearing_selection` takes no `number_of_couples` parameter and instead produces `eligible_size / 2` pairs (lines 77-93). Every other selection operator (tournament, roulette wheel, rank, truncation, SUS, Boltzmann) accepts and honors `number_of_couples`. A user who sets:

```rust
.with_number_of_couples(50)
.with_selection_method(Selection::Clearing)
```

will silently receive far fewer than 50 pairs (as few as 1 in a heavily-niched population), producing drastically fewer offspring than the configured population size. There is no warning, no error, and no documentation on the function signature that explains this divergence.

The `factory` function in `src/operations/selection.rs` (line 96) already passes `configuration.niche_radius` but does not pass `configuration.number_of_couples`, so the factory path is equally affected.

**Fix:** Accept `number_of_couples` and use it as the target pair count. After collecting `eligible`, sample pairs until `number_of_couples` is reached (with replacement if the eligible pool is smaller):

```rust
pub fn clearing_selection<U: ChromosomeT>(
    chromosomes: &[U],
    niche_radius: f64,
    number_of_couples: usize,
) -> Vec<(usize, usize)> {
    // ... (niche logic unchanged) ...

    // Draw exactly `number_of_couples` pairs from the eligible pool (with
    // replacement when the pool is smaller than 2 * number_of_couples).
    let mut mating = Vec::with_capacity(number_of_couples);
    while mating.len() < number_of_couples {
        let i1 = rng.random_range(0..eligible.len());
        let mut i2 = rng.random_range(0..eligible.len() - 1);
        if i2 >= i1 { i2 += 1; }
        mating.push((eligible[i1], eligible[i2]));
    }
    mating
}
```

The call site in `factory` should pass `configuration.number_of_couples`, and the `SelectionOperator` dispatch in `selection.rs:54` should pass `number_of_couples` from the call arguments.

---

## Warnings

### WR-01: `SelectionOperator::select` for `Clearing` uses hardcoded `niche_radius = 0.1` instead of configured value

**File:** `src/operations/selection.rs:54`

**Issue:** The `SelectionOperator` trait impl (used by the island model and NSGA-II, which call `self.select(...)` directly rather than going through `factory`) calls:

```rust
Selection::Clearing => clearing_selection(chromosomes, 0.1),
```

The `0.1` is hardcoded. Any island or NSGA-II user who configures a custom `niche_radius` via `.with_niche_radius(r)` will have their configuration silently ignored when the operator is dispatched through the trait path. The `factory` path (used by the single-population GA) correctly uses `configuration.niche_radius`, so single-population GA is unaffected — but the gap between the two dispatch paths is a maintenance hazard.

The same pattern exists for `Boltzmann` (line 50 hardcodes `1.0`), but that is pre-existing. This review focuses on the newly added `Clearing` arm.

**Fix:** `SelectionOperator::select` should accept a `SelectionConfiguration` reference so all parameters flow through. If that is a breaking change, a minimal fix is to document prominently that the trait path does not honor operator-specific configuration, and add a compile-time or runtime assertion in the island/NSGA-II code that Clearing is not used through that path.

---

### WR-02: NaN fitness in `deterministic_crowding` causes silent wrong winner selection

**File:** `src/operations/survivor/deterministic_crowding.rs:93`

**Issue:** The fitness comparison at line 93 is:

```rust
if off_fitness >= par_fitness {
```

In Rust, any comparison involving `NaN` evaluates to `false`. If `off_fitness` is `NaN`, the condition is `false` and the parent "wins" — the offspring is discarded silently. If `par_fitness` is `NaN`, the offspring always wins, replacing a parent regardless of actual merit. Neither case raises an error.

The `factory` function in `src/operations/survivor.rs` (lines 57-65) does guard against NaN before dispatching, so NaN reaching `deterministic_crowding` through `factory` is caught. However, `deterministic_crowding` is also called directly through `SurvivorOperator::select_survivors` (line 40), which performs no NaN check. Any caller using the trait path (island, NSGA-II, or future callers) bypasses the guard.

**Fix:** Add an explicit NaN guard at the top of `deterministic_crowding`, or replace the raw comparison with an `unwrap_or` that treats NaN as negative infinity:

```rust
let off_fitness = chromosomes[off_idx].fitness();
let par_fitness = chromosomes[best_parent_idx].fitness();

// Treat NaN as -infinity so NaN chromosomes always lose.
let off_wins = off_fitness.partial_cmp(&par_fitness)
    .map(|ord| ord != std::cmp::Ordering::Less)
    .unwrap_or(false);

if off_wins { ... } else { ... }
```

---

## Info

### IN-01: `test_clearing_with_zero_radius_keeps_all_eligible` comment is misleading about the niche condition

**File:** `tests/operations/test_selection_clearing.rs:98-104`

**Issue:** The comment states "distance is always > 0 unless they have identical fitness". But the clearing condition in the implementation is `(winner_fitness - candidate_fitness).abs() <= niche_radius` (line 60 of `clearing.rs`). With `niche_radius = 0.0`, two individuals with exactly identical fitness values (`a.1 == b.1`) would satisfy `0.0 <= 0.0` and one would be cleared. The test population uses `&[4.0, 3.0, 2.0, 1.0]` which has all distinct values, so the test passes — but the comment's claim is slightly wrong and could mislead future test authors or reviewers about behavior at the boundary.

**Fix:** Update the comment to accurately state the boundary condition:

```rust
// With niche_radius=0.0, only individuals with *exactly identical* fitness
// are cleared (|f_a - f_b| == 0.0 satisfies <= 0.0). All 4 have distinct
// fitness values, so all survive -> 2 pairs.
```

---

_Reviewed: 2026-05-04T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
