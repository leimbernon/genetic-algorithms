---
phase: 33-scalar-mutation-operators
fixed_at: 2026-05-07T00:00:00Z
review_path: .planning/phases/33-scalar-mutation-operators/33-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 33: Code Review Fix Report

**Fixed at:** 2026-05-07
**Source review:** .planning/phases/33-scalar-mutation-operators/33-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: Stale placeholder documentation on shipped `Mutation` variants

**Files modified:** `src/operations.rs`
**Commit:** 50a73fc
**Applied fix:** Replaced the placeholder doc comments on `LevyFlight` and `Uniform` variants with production docs matching the `Cauchy` variant style. `LevyFlight` now documents Mantegna's algorithm, the `step = σ_u * u / |v|^(1/α)` formula, and links to the `levy_alpha` config. `Uniform` now describes gene re-initialization with no config parameters required.

---

### CR-02: Cauchy and LevyFlight parameter aliasing through public `MutationOperator::mutate` is invisible and undocumented

**Files modified:** `src/operations/mutation.rs`, `src/traits/operators.rs`
**Commit:** e8c5495
**Applied fix:** Updated `factory_with_params` doc comment to explicitly state that `step` is also used as the `scale` (γ) parameter for `Mutation::Cauchy` and `sigma` is also used as the stability index `α` for `Mutation::LevyFlight`. Mirrored the same note in `MutationOperator::mutate` trait method docs in `src/traits/operators.rs`.

---

### WR-01: Division by zero in `levy_flight_mutation` when `v_normal == 0`

**Files modified:** `src/operations/mutation/levy_flight.rs`
**Commit:** abe2994
**Applied fix:** Extracted `v_abs = v_normal.abs().powf(1.0 / alpha_clamped)` into a separate binding and added a guard: if `v_abs < f64::MIN_POSITIVE`, return `levy_step = 0.0` (skip perturbation) rather than computing `u_normal / 0.0` which produces `±inf` that would snap the gene to a range boundary.

---

### WR-02: No validation of `cauchy_scale` and `levy_alpha` configuration values

**Files modified:** `src/configuration.rs`
**Commit:** 82f03d9
**Applied fix:** Added `debug_assert!(scale > 0.0, ...)` in `GaConfiguration::with_cauchy_scale` and `debug_assert!(alpha > 0.0 && alpha < 2.0, ...)` in `GaConfiguration::with_levy_alpha`. These fire in debug builds to catch misconfiguration early without impacting release performance.

---

### WR-03: Mutation errors silently discarded in ALPS and Cellular engines

**Files modified:** `src/engines/alps/engine.rs`, `src/engines/cellular/engine.rs`
**Commit:** 5998018
**Applied fix:** Replaced `let _ = if ... { mutation::factory_with_params(...) } ...;` with a named binding `mutation_result` followed by `if let Err(e) = mutation_result { warn!(target: "mutation_events", "Mutation error (skipped): {}", e); }`. Added `use log::warn;` import to both engine files.

---

### WR-04: `Mutation::Polynomial` reads `step` for `eta_m` but config stores it in `polynomial_eta`

**Files modified:** `src/traits/configuration.rs`, `src/configuration.rs`, `src/engines/ga.rs`, `src/engines/island/mod.rs`, `src/engines/nsga2/mod.rs`
**Commit:** 047e92b
**Applied fix:** Added `with_polynomial_eta(eta: f64) -> Self` to the `MutationConfig` trait and to both `GaConfiguration` and `Ga<U>` impls. Added dedicated `Mutation::Polynomial` dispatch arms in `ga.rs` (both child_1 and child_2 paths), `island/mod.rs`, and `nsga2/mod.rs`. Each arm passes `polynomial_eta.or(step)` as the `step` argument to maintain backward compatibility with callers who used `with_mutation_step` for eta.

---

_Fixed: 2026-05-07_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
