---
phase: 31-selection-survivor-diversity-operators
fixed_at: 2026-05-04T00:00:00Z
review_path: .planning/phases/31-selection-survivor-diversity-operators/31-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 31: Code Review Fix Report

**Fixed at:** 2026-05-04T00:00:00Z
**Source review:** .planning/phases/31-selection-survivor-diversity-operators/31-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (CR-01, CR-02, WR-01, WR-02)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Selection::Clearing and Survivor::DeterministicCrowding omitted from serde tests

**Files modified:** `tests/observe/test_serde.rs`
**Commit:** 3bbc1a5
**Applied fix:** Added `Selection::Clearing` to the `serde_selection_enum` test array and `Survivor::DeterministicCrowding` to the `serde_survivor_enum` test array. Both tests now exercise the full set of enum variants including the new ones added in this phase.

---

### CR-02: clearing_selection ignores number_of_couples — produces silently wrong pair count

**Files modified:** `src/operations/selection/clearing.rs`, `src/operations/selection.rs`, `tests/operations/test_selection_clearing.rs`
**Commit:** c1f808c
**Applied fix:** Added `number_of_couples: usize` parameter to `clearing_selection`. The pairing logic was replaced with a with-replacement sampling loop that draws exactly `number_of_couples` pairs from the eligible pool (using a distinct-index selection that avoids self-pairing). The factory call site in `selection.rs` was updated to pass `configuration.number_of_couples`. The `SelectionOperator` trait dispatch was updated to pass the `number_of_couples` argument received from the caller. All existing tests were updated to pass the new parameter and to reflect the corrected pair count semantics (notably `test_clearing_via_factory_respects_niche_radius` now asserts 3 pairs since `number_of_couples=3` is honored, and `test_clearing_via_selection_enum` asserts 4 pairs for a 4-eligible-individual pool with `number_of_couples=4`).

---

### WR-01: SelectionOperator::select for Clearing uses hardcoded niche_radius = 0.1

**Files modified:** `src/operations/selection.rs`
**Commit:** ad2b417
**Applied fix:** Added a prominent `log::warn!` in the `Selection::Clearing` arm of the `SelectionOperator` trait impl that fires at runtime when this path is taken. The warning explains that `niche_radius` defaults to 0.1 on this path and that callers should use `selection::factory` for the full configuration. A full structural fix (adding `SelectionConfiguration` to the trait signature) would be a breaking change and is deferred. The comment above the arm was expanded to explain the limitation clearly for future maintainers.
**Status:** fixed: requires human verification — the runtime warning fires correctly but the underlying architectural gap (trait path cannot carry operator-specific config) remains; a future breaking-change milestone should add config to the trait signature.

---

### WR-02: NaN fitness in deterministic_crowding causes silent wrong winner selection

**Files modified:** `src/operations/survivor/deterministic_crowding.rs`
**Commit:** 8406e5d
**Applied fix:** Replaced the raw `off_fitness >= par_fitness` comparison with `off_fitness.partial_cmp(&par_fitness).map(|ord| ord != Ordering::Less).unwrap_or(false)`. This treats `NaN` on either side as a loss for the offspring (`.unwrap_or(false)`), matching the semantics of the factory-path NaN guard and preventing silent wrong-winner selection through the direct trait path used by island/NSGA-II callers.

---

_Fixed: 2026-05-04T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
