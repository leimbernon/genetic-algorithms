---
phase: "51"
reviewed: "2026-05-24T00:00:00Z"
depth: standard
files_reviewed: 23
files_reviewed_list:
  - src/configuration.rs
  - src/engines/ga.rs
  - src/operations.rs
  - src/operations/crossover.rs
  - src/operations/crossover/pcx.rs
  - src/operations/crossover/spx.rs
  - src/operations/crossover/undx.rs
  - src/operations/mutation.rs
  - src/operations/mutation/self_adaptive_gaussian.rs
  - src/traits.rs
  - src/traits/configuration.rs
  - src/traits/real_valued.rs
  - src/traits/self_adaptive.rs
  - src/types/chromosomes/multi_range.rs
  - src/types/chromosomes/range.rs
  - tests/operations/test_crossover_pcx.rs
  - tests/operations/test_crossover_spx.rs
  - tests/operations/test_crossover_undx.rs
  - tests/operations/test_mutation_self_adaptive.rs
  - tests/test_multi_parent_integration.rs
  - tests/test_operations.rs
  - tests/test_traits.rs
  - tests/traits/test_self_adaptive.rs
findings:
  critical: 1
  warning: 5
  info: 3
  total: 9
status: issues_found
---

# Phase 51: Code Review Report

**Reviewed:** 2026-05-24
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Phase 51 adds three multi-parent crossover operators (UNDX, SPX, PCX), a self-adaptive Gaussian mutation operator, and two supporting traits (`RealValued`, `SelfAdaptive`). The overall architecture is sound: the enum + factory pattern is applied consistently, WASM compatibility constraints are respected (no unchecked `Instant` calls, no unconditional `par_iter`), and all new types conform to the project's no-breaking-change policy. Tests pass and the trait/dispatch machinery is correct.

One **critical** algorithmic bug was found in PCX: the directional noise `eta` is sampled independently per gene per parent, when it must be sampled once per parent direction vector. This fundamentally changes the distribution of offspring — PCX produces component-wise independent noise masquerading as directional perturbation, which means the operator does not implement the PCX algorithm.

Five **warnings** cover: a missing build-time guard for `num_parents < 3`, an SPX dead-code element that is never read, no upper bound on self-adaptive sigma, hardcoded UNDX/SPX/PCX parameters with no user-accessible configuration, and a simplification in UNDX where the orthogonal noise is isotropic rather than truly orthogonal to the primary direction.

---

## Critical Issues

### CR-01: PCX directional noise sampled per gene per parent instead of per parent vector

**File:** `src/operations/crossover/pcx.rs:85-96`

**Issue:** The outer loop iterates over gene dimensions (`for i in 0..expected`). Inside it, the inner loop over non-primary parents draws a fresh Box-Muller sample `eta_noise` for every `(gene, parent)` pair:

```rust
for p in parents.iter().skip(1) {
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let eta_noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_eta;
    directional += eta_noise * (T::to_f64(p.dna()[i].value) - p0);
}
```

In the standard PCX algorithm (Deb et al., 2002), `eta_j` is a single scalar drawn per non-primary parent `j` and then multiplied by the full direction vector `d_j = parent_j - primary`. Using independent noise per `(gene, parent)` pair destroys the directional correlation: gene dimension 0 and gene dimension 1 get different `eta` values for the same parent, so the operator is not applying a coherent directional step. The result is equivalent to component-wise independent Gaussian noise scaled by per-gene parent spread — a different (and weaker) operator than PCX.

**Fix:** Restructure the loop to draw one `eta_j` per non-primary parent and apply it across all gene dimensions:

```rust
// Compute direction vectors and draw one eta per parent
let mut directional = vec![0.0_f64; expected];
for p in parents.iter().skip(1) {
    // ONE Box-Muller draw per parent direction
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let eta_j: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_eta;
    // Apply the same eta_j to all gene components of this parent's direction
    for i in 0..expected {
        directional[i] += eta_j * (T::to_f64(p.dna()[i].value) - T::to_f64(dna0[i].value));
    }
}

// Then per-gene orthogonal noise
for i in 0..expected {
    let u1_z: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2_z: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let zeta: f64 = (-2.0 * u1_z.ln()).sqrt() * u2_z.cos() * sigma_zeta * spread[i];
    let raw = p0_vals[i] + directional[i] + zeta;
    // ... clamping and gene construction
}
```

---

## Warnings

### WR-01: No build-time validation that `num_parents >= 3` for Undx/Spx/Pcx

**File:** `src/engines/ga.rs:733` (build method) / `src/validators/validator_factory.rs`

**Issue:** `Crossover::Undx { num_parents: 2 }` (or any value below 3) passes `build()` validation silently. At runtime, for every pair of parents selected, `ga.rs:2558` computes `extras = num_parents.saturating_sub(2) = 0`, producing `parent_refs.len() == 2`. Then `factory_multi_parent_dispatch` immediately returns `Err(GaError::CrossoverError("Multi-parent crossover requires at least 3 parents"))`, which propagates up and aborts the run with an error on every generation's first couple. This should be caught at build time.

**Fix:** In `build()` or the validator, after the existing operator-compat check, add:

```rust
match self.configuration.crossover_configuration.method {
    Crossover::Undx { num_parents }
    | Crossover::Spx { num_parents }
    | Crossover::Pcx { num_parents } if num_parents < 3 => {
        return Err(GaError::ConfigurationError(format!(
            "Multi-parent crossover requires num_parents >= 3, got {}",
            num_parents
        )));
    }
    _ => {}
}
```

---

### WR-02: SPX `r.push(1.0)` is dead code — the element is never read

**File:** `src/operations/crossover/spx.rs:95-103`

**Issue:** The vector `r` is built with `n_par - 1` draws (indices `0..n_par-2`), then a sentinel `1.0` is pushed at index `n_par-1` (line 95). The sampling loop on line 99 iterates `for k in (0..n_par - 1).rev()`, covering indices `0..=(n_par-2)`. The element `r[n_par-1] = 1.0` is never accessed. This is misleading: the comment "push 1.0 so r.len() == n_par" suggests it was intended to match indices to `expanded[k]`, but the loop body uses `expanded[k]` directly (not via `r[n_par-1]`).

The extra push does not affect correctness because the SPX sampling is mathematically correct without it, but the dead element adds noise and implies a different algorithm than what is actually executed.

**Fix:** Remove the push and adjust the comment:

```rust
// Sample r_k = U^(1/(n_par-1-k)) for k in 0..n_par-1 (no sentinel needed)
let r: Vec<f64> = (0..n_par - 1)
    .map(|k| {
        rng.random_range(0.0_f64..1.0)
            .powf(1.0 / (n_par - 1 - k) as f64)
    })
    .collect();
```

---

### WR-03: No upper bound on self-adaptive sigma — unbounded explosion risk

**File:** `src/traits/self_adaptive.rs:98-106`

**Issue:** The log-normal update `sigma_i = sigma_i * exp(tau' * N_global + tau * N_local)` has a lower bound (`sigma_min`) but no upper bound. With aggressive learning rates (large `tau` or `tau_prime`), sigma can grow exponentially over many generations. When sigma explodes, all gene mutations are clamped to the gene's bounds on every call, eliminating variation and degrading to random search confined to the boundaries. This failure mode is silent — the GA completes successfully but stops exploring the interior of the search space.

There is no `with_sigma_max()` builder method, no enforcement in `adapt_strategy_params`, and no warning in the GA loop when sigma values become very large relative to the gene range.

**Fix:** Add an optional `sigma_max` parameter (defaulting to `None` for backward compatibility) and enforce it in `adapt_strategy_params`:

```rust
// In MutationConfiguration:
pub sigma_max: Option<f64>,

// In adapt_strategy_params, after the current .max(sigma_min):
*sigma = (*sigma * (tau_prime * global_noise + tau * local_noise).exp())
    .max(sigma_min)
    .min(effective_sigma_max);   // where effective_sigma_max = config or f64::MAX
```

At minimum, emit a `log::warn!` during the GA loop when any sigma value exceeds, say, 10x the gene range width.

---

### WR-04: UNDX/SPX/PCX parameters are hardcoded — `_configuration` argument is dead

**File:** `src/operations/crossover.rs:279,324,369` / `src/configuration.rs`

**Issue:** The functions `try_undx`, `try_spx`, and `try_pcx` each accept a `CrossoverConfiguration` parameter named `_configuration` (underscore-prefixed, explicitly unused). The algorithm parameters `sigma_eta = 0.1`, `sigma_zeta = 0.1` (PCX), `sigma_xi = 0.35/sqrt(n_par-1)`, `sigma_eta = 0.35/sqrt(n_par)` (UNDX), and `epsilon = sqrt(n_par+2)` (SPX) are all hardcoded constants that users cannot tune. This is a functional regression from what `_configuration` implies: users who want to compare the effect of different UNDX sigma parameters must fork the library.

**Fix (non-breaking):** Add optional fields to `CrossoverConfiguration`:

```rust
pub undx_sigma_xi: Option<f64>,   // default: 0.35 / sqrt(n_par - 1)
pub undx_sigma_eta: Option<f64>,  // default: 0.35 / sqrt(n_par)
pub pcx_sigma_eta: Option<f64>,   // default: 0.1
pub pcx_sigma_zeta: Option<f64>,  // default: 0.1
// SPX epsilon is derived from n_par so no field needed unless overriding
```

Then remove the `_` prefix from the `configuration` parameter and read these fields in the respective `try_*` functions.

---

### WR-05: UNDX xi (orthogonal) noise is isotropic, not orthogonal to the primary direction

**File:** `src/operations/crossover/undx.rs:100-106`

**Issue:** The UNDX algorithm (Kita, Ono, Kobayashi 1999) requires that the secondary perturbation components be drawn from directions **orthogonal** to the primary inter-parent direction `d = parent_0 - centroid`. The standard implementation uses Gram-Schmidt or a random subspace projection to ensure orthogonality. The current code draws independent per-gene Gaussian noise `xi_noise_j ~ N(0, sigma_xi)`:

```rust
let raw = centroid[i] + eta_noise * dir[i] + xi_noise;
```

This is isotropic noise, not orthogonal noise. In high-dimensional spaces, a random Gaussian vector is approximately orthogonal to any fixed vector (by concentration of measure), so the approximation degrades gracefully as dimension increases. However, in low-dimensional spaces (the primary use case for UNDX, e.g., 2–5 genes), a significant fraction of the xi noise is aligned with `dir`, adding noise in the primary direction and reducing the operator's ability to maintain the parent's relative position along that axis.

The docstring describes "orthogonal directions" which does not match the implementation.

**Fix:** Add a projection step to remove the primary-direction component from the raw noise before applying it:

```rust
// Compute raw isotropic noise
let raw_noise: Vec<f64> = (0..expected)
    .map(|_| { /* Box-Muller */ })
    .collect();
// Project out the primary direction component
let dot: f64 = raw_noise.iter().zip(dir.iter()).map(|(n, d)| n * d).sum();
let xi_perp: Vec<f64> = raw_noise.iter().zip(dir.iter())
    .map(|(n, d)| (n - dot * d) * sigma_xi)
    .collect();
// Apply per-gene
for i in 0..expected {
    let raw = centroid[i] + eta_noise * dir[i] + xi_perp[i];
    // ...
}
```

Note: this is a **behavioral change** but not a breaking change to the public API.

---

## Info

### IN-01: `_num_parents` parameter is accepted but silently ignored in all three crossover functions

**File:** `src/operations/crossover/undx.rs:32`, `src/operations/crossover/spx.rs:30`, `src/operations/crossover/pcx.rs:32`

**Issue:** All three public crossover functions declare `_num_parents: usize` in their signature but use `parents.len()` exclusively. The underscore prefix documents intentionality but the parameter is part of the public API surface (these functions are `pub`). Callers who pass a value expecting it to control parent count will be silently ignored.

**Fix:** Either remove the parameter (breaking change, minor), rename it to convey its no-op nature in the docstring more prominently, or use it as a validation hint (`assert_eq!(parents.len(), _num_parents)` style debug assert).

---

### IN-02: No build-time validation for `sigma_min`, `tau`, `tau_prime` parameter ranges

**File:** `src/engines/ga.rs:479-490` / `src/configuration.rs:195-205`

**Issue:** `with_sigma_min(value)`, `with_self_adaptive_tau(value)`, and `with_self_adaptive_tau_prime(value)` accept any `f64` including `NaN`, `f64::INFINITY`, and negative values. Passing `sigma_min = f64::INFINITY` causes all sigmas to become `infinity` after the first `adapt_strategy_params` call, then gene mutations produce `NaN` from `current + N(0, ∞)`. Passing `tau < 0.0` is technically valid for the log-normal formula but produces decay-only behavior that may surprise users expecting standard ES semantics. No validation or documentation of valid ranges exists.

**Fix:** Add range checks in `build()`:

```rust
if let Some(sm) = config.mutation_configuration.sigma_min {
    if !sm.is_finite() || sm < 0.0 {
        return Err(GaError::ConfigurationError(
            "sigma_min must be a finite non-negative value".to_string()
        ));
    }
}
```

---

### IN-03: Test coverage for degenerate UNDX/SPX/PCX inputs (all parents identical) is absent

**File:** `tests/operations/test_crossover_undx.rs`, `tests/operations/test_crossover_spx.rs`, `tests/operations/test_crossover_pcx.rs`

**Issue:** None of the three test files exercise the scenario where all parents have identical DNA (population convergence). This is the most common failure mode in late-generation GA runs. For UNDX, it means `dir_norm` approaches zero, the `dir_norm.max(1e-14)` guard kicks in, and the offspring is effectively isotropic noise around the centroid. For SPX, all expanded vertices collapse to the same point and `r_k` sampling still works (the simplex degenerates to a point, offspring = centroid). For PCX, `spread[i] = 0` for all genes and `directional = 0` for all i (all directions are zero), leaving offspring = primary with only zeta = 0 noise, effectively a clone.

These are not panics but they represent silent behavioral changes that should be covered.

**Fix:** Add a test in each file:

```rust
#[test]
fn undx_with_identical_parents_does_not_panic() {
    let mut p = RangeChromosome::<f64>::new();
    p.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0.0, 10.0)], 5.0),
    ]));
    let parents = vec![p.clone(), p.clone(), p.clone()];
    let refs: Vec<_> = parents.iter().collect();
    for _ in 0..20 {
        let result = undx(&refs, 3);
        assert!(result.is_ok());
    }
}
```

---

_Reviewed: 2026-05-24_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
