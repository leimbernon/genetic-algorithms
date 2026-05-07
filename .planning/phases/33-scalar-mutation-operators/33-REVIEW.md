---
phase: 33-scalar-mutation-operators
reviewed: 2026-05-07T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - src/configuration.rs
  - src/engines/alps/engine.rs
  - src/engines/cellular/engine.rs
  - src/engines/ga.rs
  - src/engines/island/mod.rs
  - src/engines/island/nsga2.rs
  - src/engines/nsga2/mod.rs
  - src/operations.rs
  - src/operations/mutation.rs
  - src/operations/mutation/cauchy.rs
  - src/operations/mutation/levy_flight.rs
  - src/operations/mutation/uniform.rs
  - src/traits/configuration.rs
  - tests/observe/test_serde.rs
  - tests/observe/visualization/test_visualization.rs
  - tests/operations/test_mutation_cauchy_levy_uniform.rs
  - tests/test_operations.rs
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: fixed
---

# Phase 33: Code Review Report

**Reviewed:** 2026-05-07
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 33 adds three new scalar mutation operators for `Range<T>` chromosomes: Cauchy, Lévy Flight, and Uniform reset. The implementations in `cauchy.rs`, `levy_flight.rs`, and `uniform.rs` are well-structured and the dispatch wiring through `mutation.rs` is consistent. The Gamma polynomial approximation in `levy_flight.rs` is non-trivial but is well-guarded against edge cases.

Two blockers exist. First, the `Mutation::Cauchy` dispatch uses the `step` argument to carry the `cauchy_scale` parameter, while `Mutation::LevyFlight` uses the `sigma` argument to carry the `levy_alpha` parameter. This is not explicit in the public `MutationOperator::mutate` signature and is actively misleading: the `MutationConfig` trait exposes `with_mutation_step` as the step for Creep mutation, yet the Cauchy match arm silently aliases it. The engine call sites work correctly because they bypass the generic `step`/`sigma` args and pass from the dedicated config fields, but any caller who reaches `mutation.rs` through the `MutationOperator::mutate` path with a naively constructed call will get wrong behavior with no compile-time or runtime warning. Second, the `Mutation::Uniform` variant docs in `src/operations.rs` still contain stale placeholder text ("placeholder — implemented in Phase 33 Plan 03"), and similarly `Mutation::LevyFlight` still says "placeholder — implemented in Phase 33 Plan 02". Shipping placeholder copy in public-facing documentation on a library published to crates.io is a correctness failure for the docs that will mislead users.

Warnings cover: a potential division-by-zero in `levy_flight_mutation` when `v_normal` is zero (extremely rare but mathematically possible), missing validation of the `cauchy_scale` and `levy_alpha` parameters (a scale of 0 or a negative alpha produces degenerate or nonsensical noise), silent mutation error suppression in the ALPS and Cellular engines (`let _ = ...`), and the cross-file parameter-slot aliasing documented above.

---

## Critical Issues

### CR-01: Stale placeholder documentation on shipped `Mutation` variants

**File:** `src/operations.rs:136-139`
**Issue:** Two `Mutation` enum variants carry doc comments that explicitly identify them as "placeholder — implemented in Phase 33 Plan 0X". These are now fully implemented. This documentation ships verbatim to crates.io users who will read the API docs and be told the feature is a placeholder that has not yet been built. This is incorrect and will actively mislead library consumers.

```rust
// Current (incorrect):
/// Lévy Flight mutation for `Range<T>` chromosomes (placeholder — implemented in Phase 33 Plan 02).
LevyFlight,
/// Uniform reset mutation for `Range<T>` chromosomes (placeholder — implemented in Phase 33 Plan 03).
Uniform,

// Fix — replace with production docs matching the Cauchy pattern:
/// Lévy Flight mutation for `Range<T>` chromosomes (Mantegna's algorithm).
/// Generates heavy-tailed steps via `step = σ_u * u / |v|^(1/α)`.
/// Configure the stability index (α) via [`MutationConfiguration::levy_alpha`]
/// or [`MutationConfig::with_levy_alpha`]. Valid range: (0.0, 2.0). Default α: `1.5`.
/// Returns `GaError::MutationError` for non-`Range<T>` chromosomes.
LevyFlight,
/// Uniform reset mutation for `Range<T>` chromosomes.
/// Resets a single randomly chosen gene to a uniform sample within its declared range.
/// Equivalent to gene re-initialization. No configuration parameters required.
/// Returns `GaError::MutationError` for non-`Range<T>` chromosomes.
Uniform,
```

---

### CR-02: Cauchy and LevyFlight parameter aliasing through public `MutationOperator::mutate` is invisible and undocumented

**File:** `src/operations/mutation.rs:245-261`
**Issue:** The `MutationOperator::mutate` signature accepts `step: Option<f64>` and `sigma: Option<f64>`. The Cauchy arm reads `scale` from `step`, and the LevyFlight arm reads `alpha` from `sigma`. There is no documentation on these args explaining this aliasing, and the public `factory_with_params` function's doc comment says `step` is "Optional step size for Creep mutation" and `sigma` is "Optional sigma for Gaussian mutation" — both descriptions are wrong for the Cauchy and LevyFlight cases.

Any caller who reads the public API docs and calls `factory_with_params(Mutation::Cauchy, &mut ind, None, Some(2.0))` will silently apply the default scale of 1.0 instead of 2.0 because the scale maps to `step`, not `sigma`. This is a silent wrong-result bug for any external caller. Internal engine call sites happen to be correct, but they bypass the generic path entirely.

**Fix:** Document the aliasing contract explicitly in the function docs, or — better — introduce dedicated params. At minimum update the `factory_with_params` doc:

```rust
/// * `step` — Step size for Creep mutation; **also used as the `scale` (γ) parameter
///   for `Mutation::Cauchy`** (default 1.0 when `None`).
/// * `sigma` — Sigma for Gaussian mutation; **also used as the stability index `α`
///   for `Mutation::LevyFlight`** (default 1.5 when `None`).
```

And mirror this note in the `MutationOperator::mutate` trait method docs. Without this, the API contract is unknowable without reading the source.

---

## Warnings

### WR-01: Division by zero in `levy_flight_mutation` when `v_normal == 0`

**File:** `src/operations/mutation/levy_flight.rs:85`
**Issue:** `v_normal` is drawn from `N(0, 1)` via Box-Muller. The Box-Muller transform with `bv1 ∈ (ε, 1.0)` ensures `bv1 > 0`, so `-2 * ln(bv1)` is finite and positive. However the cosine of `bv2` can be exactly zero when `bv2 = π/2 + kπ`, making `v_normal = 0.0`. The expression `v_normal.abs().powf(1.0 / alpha_clamped)` then yields `0.0`, and `u_normal / 0.0` is `±inf`. The subsequent `.clamp(lo_f64, hi_f64)` call turns `inf` into `hi_f64` and `-inf` into `lo_f64`, so the gene is silently snapped to a boundary rather than producing NaN, but the mutation distribution is violated for that sample.

```rust
// Current:
let levy_step: f64 = u_normal / v_normal.abs().powf(1.0 / alpha_clamped);

// Fix — guard against the zero case:
let v_abs = v_normal.abs().powf(1.0 / alpha_clamped);
let levy_step: f64 = if v_abs < f64::MIN_POSITIVE {
    0.0 // degenerate sample; skip perturbation this call
} else {
    u_normal / v_abs
};
```

---

### WR-02: No validation of `cauchy_scale` and `levy_alpha` configuration values

**File:** `src/configuration.rs:162-165`, `src/traits/configuration.rs:62-66`
**Issue:** Neither the builder methods `with_cauchy_scale` and `with_levy_alpha` nor any validator checks the values passed in. A `cauchy_scale` of `0.0` produces `tan(…) * 0.0 = 0.0`, which is a no-op mutation with no error or warning. A negative `levy_alpha` is clamped silently to `0.1` inside `levy_flight_mutation` but the user receives no feedback that their configuration was rejected. An `alpha >= 2.0` is similarly silently clamped. These are silent wrong-behavior cases for users who misconfigure the operators.

**Fix:** Add range validation in the builder or in the existing validator factory:

```rust
// In with_cauchy_scale:
fn with_cauchy_scale(mut self, scale: f64) -> Self {
    // A scale of 0 or negative makes the perturbation a no-op or invalid.
    // Enforce scale > 0 at the validator level or here with a debug_assert.
    debug_assert!(scale > 0.0, "cauchy_scale must be positive; got {}", scale);
    self.mutation_configuration.cauchy_scale = Some(scale);
    self
}

// In with_levy_alpha:
fn with_levy_alpha(mut self, alpha: f64) -> Self {
    debug_assert!(
        alpha > 0.0 && alpha < 2.0,
        "levy_alpha must be in (0.0, 2.0); got {}. Values outside this range are clamped.",
        alpha
    );
    self.mutation_configuration.levy_alpha = Some(alpha);
    self
}
```

Alternatively add these checks to the existing `validator_factory` so a `build()` call returns an explicit error.

---

### WR-03: Mutation errors silently discarded in ALPS and Cellular engines

**File:** `src/engines/alps/engine.rs:223-244`, `src/engines/cellular/engine.rs:222-243`
**Issue:** Both engines apply mutation with `let _ = if ... { mutation::factory_with_params(...) } ...;`. Any `Err(GaError::MutationError)` returned by the mutation operator is silently dropped. For the three new operators this means a chromosome with an unsupported type (e.g., a non-Range chromosome used with Cauchy) would silently proceed unmutated, with no log warning and no error propagation. The standard `Ga` engine in `engines/ga.rs` propagates these errors via `?`.

**Fix:** Propagate the error. Since `AlpsEngine::run` returns `AlpsResult` (not a `Result`), the pattern should at minimum log a warning:

```rust
// In alps/engine.rs and cellular/engine.rs — replace:
let _ = mutation::factory_with_params(...);

// With:
if let Err(e) = mutation::factory_with_params(...) {
    log::warn!(target: "mutation_events", "Mutation error (skipped): {}", e);
}
```

Ideally, propagate the error up through `run` by changing the return type to `Result<AlpsResult<U>, GaError>`, matching the pattern established by `Ga::run`.

---

### WR-04: `Mutation::Polynomial` in `MutationOperator::mutate` reads `step` for `eta_m` but config stores it in `polynomial_eta`

**File:** `src/operations/mutation.rs:218-225`
**Issue:** The Polynomial match arm extracts `eta_m` from `step.unwrap_or(DEFAULT_POLYNOMIAL_ETA)`. However `MutationConfiguration` stores this in the dedicated field `polynomial_eta`, not in `step`. In `engines/ga.rs:parent_crossover` the Polynomial method falls through to the generic else-branch which passes `configuration.mutation_configuration.step` and `sigma` — so if a user sets `with_polynomial_eta()` (if such a builder exists) but not `with_mutation_step()`, the `eta_m` silently uses the wrong value. Checking `configuration.rs`, there is a `polynomial_eta` field on `MutationConfiguration`, but no `with_polynomial_eta` builder method exists in either `MutationConfig` trait or the `GaConfiguration` impl. The field is a config-level dead weight — `step` is used instead. This is an inconsistency in the config surface that could confuse future contributors and is a pre-existing design issue now made more salient by the new Cauchy/LevyFlight dedicated fields that do have dedicated builder methods.

**Fix:** Either add a `with_polynomial_eta` builder that sets `polynomial_eta` and update the Polynomial dispatch to read from `polynomial_eta.unwrap_or(DEFAULT_POLYNOMIAL_ETA)` (passing through the engine), or remove the `polynomial_eta` field from `MutationConfiguration` and document that `step` carries the eta value. The current state exposes a trap for users who discover `polynomial_eta` in the struct and set it directly, only to find it has no effect.

---

## Info

### IN-01: `Mutation::ListValue` delegates to `value_mutate()` silently — same as `Mutation::Value`

**File:** `src/operations/mutation.rs:237`
**Issue:** `Mutation::ListValue => individual.value_mutate()` is identical to `Mutation::Value => individual.value_mutate()`. The distinction between the two variants is only meaningful if the chromosome type implements `ValueMutable::value_mutate` differently for list vs range chromosomes. The dispatch code itself provides no differentiation. This is a dead-code smell and may cause confusion in the future.

---

### IN-02: Inline test in `levy_flight.rs` violates project convention

**File:** `src/operations/mutation/levy_flight.rs:99-119`
**Issue:** The project memory file records the convention: "All unit tests must be in `tests/`, never inline with implementation code." `levy_flight.rs` contains an inline `#[cfg(test)] mod tests` block with two tests (`mantegna_sigma_u_finite_positive_at_default_alpha` and `gamma_approx_known_values`). These should be moved to `tests/operations/` per project rules.

---

### IN-03: `CellularEngine::run` termination cause logic is inverted

**File:** `src/engines/cellular/engine.rs:292-295`
**Issue:** The termination cause is derived as:
```rust
let cause = if generations < self.config.max_generations {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
```
This is correct when early exit happened via the `break` on target reached. However the loop breaks unconditionally when the target is reached without incrementing `generations` past `max_generations`, so `generations < max_generations` is a reliable signal only if the loop counter is known not to reach `max_generations` except on the final iteration. In practice the logic works, but it relies on `generations` being incremented before the target check and the loop index being 0-based. A more explicit boolean flag (as used in `AlpsEngine`) would be safer and clearer. This is a maintainability/clarity issue.

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
