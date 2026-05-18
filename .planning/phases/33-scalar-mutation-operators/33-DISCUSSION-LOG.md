# Phase 33: Scalar Mutation Operators - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-06
**Phase:** 33-scalar-mutation-operators
**Areas discussed:** Chromosome type scope, Uniform semantics, Mutation scope, Lévy step algorithm

---

## Chromosome Type Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Range<T>-only | All three operators are Range<T>-only — Uniform resets within [lo, hi]. Error for Binary/List. | ✓ |
| Uniform handles all types | Uniform gets special handling per chromosome type (Range reset, Binary flip, List resample) | |

**User's choice:** Range<T>-only for all three operators
**Notes:** Simpler, consistent across all three operators. Uniform doesn't need special-casing.

---

## Uniform Semantics

### What Uniform does

| Option | Description | Selected |
|--------|-------------|----------|
| Full reset to random in [lo, hi] | Picks a completely new value uniformly at random within the gene's declared range. No extra config. | ✓ |
| Additive uniform noise in [-step, +step] | Uniform perturbation with configurable step, clamped to range. Needs `uniform_step` config field. | |

**User's choice:** Full reset — uniform random within [lo, hi]
**Notes:** No new config parameter needed. Conceptually "re-initialize this gene."

### Multi-range behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Pick a random range, reset within it | Mirrors gaussian.rs behavior — randomly select which range, then reset within it. | ✓ |
| Always use the first range | Simpler — always uses ranges[0]. Slight divergence from Gaussian pattern. | |
| You decide | Claude picks (would be random range to stay consistent). | |

**User's choice:** Pick a random range — consistent with gaussian.rs `range_idx` pattern

---

## Mutation Scope

| Option | Description | Selected |
|--------|-------------|----------|
| One random gene (like Gaussian/Creep/Value) | Consistent with existing scalar operators. Mutation probability controls call frequency. | ✓ |
| All genes | Perturbs every gene per call. More aggressive, inconsistent with Gaussian. | |

**User's choice:** One randomly selected gene per `mutate()` call
**Notes:** Consistent with the entire scalar mutation family.

---

## Lévy Step Algorithm

### Implementation approach

| Option | Description | Selected |
|--------|-------------|----------|
| Mantegna's algorithm | Two normal samples approximating Lévy-stable distribution. Academically standard, correct tail behavior. | ✓ |
| Simple power-law approximation | Step ~ u^(-1/alpha). Faster but less numerically accurate. | |
| You decide | Claude picks — would choose Mantegna anyway. | |

**User's choice:** Mantegna's algorithm

### Lévy alpha configurability

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable via levy_alpha (Recommended) | `levy_alpha: Option<f64>` in MutationConfiguration, default 1.5. Same pattern as polynomial_eta. | ✓ |
| Fixed at 1.5 | Hardcode. No config field. | |

**User's choice:** Configurable — `levy_alpha: Option<f64>` field, default 1.5

### Cauchy scale field

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated cauchy_scale field | `cauchy_scale: Option<f64>` in MutationConfiguration. Named field per operator parameter. | ✓ |
| Reuse sigma field | Cauchy scale plays similar role to Gaussian sigma. Saves one field but creates semantic confusion. | |

**User's choice:** Dedicated `cauchy_scale: Option<f64>` field, default 1.0

---

## Claude's Discretion

- Exact Gamma function implementation for Mantegna's σ_u (Lanczos approximation or inline constant for α=1.5)
- Whether to precompute σ_u or compute inline
- Cauchy step formula: `cauchy_scale * tan(π * (u - 0.5))` where `u ~ Uniform(0,1)` (standard inverse-CDF)
- Log target names (`mutation_events`)
- Internal helper function names and file structure

## Deferred Ideas

None — discussion stayed within phase scope.
