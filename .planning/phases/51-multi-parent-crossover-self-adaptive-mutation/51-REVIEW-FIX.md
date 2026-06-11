---
phase: "51"
fixed_at: "2026-05-24T00:00:00Z"
review_path: .planning/phases/51-multi-parent-crossover-self-adaptive-mutation/51-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 51: Code Review Fix Report

**Fixed at:** 2026-05-24
**Source review:** `.planning/phases/51-multi-parent-crossover-self-adaptive-mutation/51-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (1 Critical + 5 Warning)
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: PCX directional noise sampled per gene per parent instead of per parent vector

**Files modified:** `src/operations/crossover/pcx.rs`
**Applied fix:** Restructured the PCX loop to draw one Box-Muller sample (`eta_j`) per non-primary parent and apply it coherently across all gene dimensions. The outer loop now iterates over parents, drawing one `eta_j` scalar per parent and accumulating `directional[i] += eta_j * (parent[i] - primary[i])` for all `i`. The per-gene orthogonal zeta noise remains as a separate second pass. The old code drew a fresh `eta_noise` per `(gene, parent)` pair, destroying directional correlation between dimensions.

---

### WR-01: No build-time validation that `num_parents >= 3` for Undx/Spx/Pcx

**Files modified:** `src/engines/ga.rs`
**Applied fix:** Added a `match` block in `build()` immediately after the `operator_compat_check` call. When any of `Crossover::Undx`, `Crossover::Spx`, or `Crossover::Pcx` has `num_parents < 3`, `build()` returns `Err(GaError::ConfigurationError(...))` with a clear message. This converts the silent runtime failure into a build-time error.

---

### WR-02: SPX `r.push(1.0)` is dead code

**Files modified:** `src/operations/crossover/spx.rs`
**Applied fix:** Removed the `r.push(1.0)` sentinel and changed the `Vec` from `mut` to immutable. Updated the comment to accurately describe what the collection contains. The sampling loop iterates `(0..n_par - 1)` and never accessed `r[n_par-1]`, so removal is correct with no behavioral change.

---

### WR-03: No upper bound on self-adaptive sigma — unbounded explosion risk

**Files modified:** `src/configuration.rs`, `src/traits/configuration.rs`, `src/engines/ga.rs`, `src/operations/mutation.rs`, `src/operations/mutation/self_adaptive_gaussian.rs`, `tests/operations/test_mutation_self_adaptive.rs`
**Applied fix:** Added `sigma_max: Option<f64>` field to `MutationConfiguration` (default `None` — no breaking change). Added `with_sigma_max(value: f64)` builder method to the `MutationConfig` trait and implemented it in both `Ga<U>` and `GaConfiguration`. Updated `self_adaptive_gaussian_mutation` to accept `sigma_max: Option<f64>` and apply the cap after `adapt_strategy_params`. Updated `try_self_adaptive` and `factory_self_adaptive` to thread `sigma_max` through. Updated both `Mutation::SelfAdaptiveGaussian` call sites in `ga.rs` to pass `configuration.mutation_configuration.sigma_max`. Updated test call sites to pass `None`.

---

### WR-04: UNDX/SPX/PCX parameters are hardcoded — `_configuration` argument is dead

**Files modified:** `src/configuration.rs`, `src/traits/configuration.rs`, `src/engines/ga.rs`, `src/operations/crossover.rs`, `src/operations/crossover/undx.rs`, `src/operations/crossover/pcx.rs`, `tests/operations/test_crossover_undx.rs`, `tests/operations/test_crossover_pcx.rs`
**Applied fix:** Added four optional override fields to `CrossoverConfiguration`: `undx_sigma_xi: Option<f64>`, `undx_sigma_eta: Option<f64>`, `pcx_sigma_eta: Option<f64>`, `pcx_sigma_zeta: Option<f64>` (all defaulting to `None`). Added corresponding builder methods to the `CrossoverConfig` trait and implemented them in both `Ga<U>` and `GaConfiguration`. Updated `undx()` to accept `sigma_xi_override` and `sigma_eta_override` parameters (falling back to literature defaults when `None`). Updated `pcx()` to accept `sigma_eta_override` and `sigma_zeta_override`. Updated `try_undx` and `try_pcx` in `crossover.rs` to remove the `_` prefix from the `configuration` parameter and pass the override values. SPX has no user-tunable sigma parameters beyond `epsilon` (derived from `n_par`), so `try_spx` keeps `_configuration` unchanged. Updated test call sites to pass `None, None` for the new parameters.

---

### WR-05: UNDX xi (orthogonal) noise is isotropic, not orthogonal to the primary direction

**Files modified:** `src/operations/crossover/undx.rs`
**Applied fix:** Added a projection step to make the xi noise orthogonal to the primary direction `dir`. The raw per-gene Box-Muller samples are collected into `raw_xi: Vec<f64>`. The dot product `dot = sum(raw_xi[i] * dir[i])` is computed, and `xi_perp[i] = (raw_xi[i] - dot * dir[i]) * sigma_xi` projects out the primary-direction component. This ensures the secondary perturbation does not add noise along `dir`, matching the UNDX specification (Kita, Ono, Kobayashi 1999). Behavior changes in low-dimensional spaces (2–5 genes) where a random vector is not approximately orthogonal to a fixed vector. The public API is unchanged.

---

## Skipped Issues

None — all 6 in-scope findings were fixed.

---

_Fixed: 2026-05-24_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
