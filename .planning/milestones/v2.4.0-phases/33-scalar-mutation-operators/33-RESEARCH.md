# Phase 33: Scalar Mutation Operators - Research

**Researched:** 2026-05-06
**Domain:** Rust genetic algorithm mutation operators — continuous real-valued perturbation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** All three operators are `Range<T>`-only. Return `GaError::MutationError` with a clear message for Binary and List chromosomes.
- **D-02:** Each operator mutates **one randomly selected gene** per `mutate()` call — identical to Gaussian, Creep, and Value.
- **D-03:** Uniform mutation = **full reset** — picks a new gene value uniformly at random within the gene's declared `[lo, hi]` range. No new config parameter needed.
- **D-04:** When a gene has multiple declared ranges, Uniform picks a **random range** and resets within it — mirrors the `gaussian.rs` `range_idx` selection pattern exactly.
- **D-05:** Add `cauchy_scale: Option<f64>` to `MutationConfiguration`. Default: `1.0` when `None`. Add `with_cauchy_scale(scale: f64)` builder method to `ConfigurationT` and `MutationConfig`.
- **D-06:** Add `levy_alpha: Option<f64>` to `MutationConfiguration`. Default: `1.5` when `None`. Add `with_levy_alpha(alpha: f64)` builder method.
- **D-07:** Uniform needs no new config field — uses gene's declared range directly.
- **D-08:** Do NOT reuse `sigma` for Cauchy scale — separate named fields preserve clear intent.
- **D-09:** Use **Mantegna's algorithm** for Lévy step generation. Formula: `step = σ_u * u / |v|^(1/α)` where `u ~ N(0, σ_u²)`, `v ~ N(0, 1)`, `σ_u = (Γ(1+α) * sin(πα/2) / (Γ((1+α)/2) * α * 2^((α-1)/2)))^(1/α)`.
- **D-10:** Cauchy perturbation: `noise = cauchy_scale * tan(π * (u - 0.5))` where `u ~ Uniform(0, 1)`. Result clamped to `[lo, hi]`.
- **D-11:** Add `Mutation::Cauchy`, `Mutation::LevyFlight`, `Mutation::Uniform` to the serde round-trip test array in `tests/observe/test_serde.rs` (mandatory per Phase 32 CR-01 lesson).

### Claude's Discretion

- Exact Gamma function implementation for Mantegna's σ_u (Lanczos approximation or inline constant for α=1.5: σ_u ≈ 0.6966)
- Whether to precompute Mantegna's σ_u once in the function body or compute inline each call
- Log target names: follow existing patterns (`mutation_events`)
- Internal helper function names and file structure within `src/operations/mutation/`

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MUT-01 | User can configure Cauchy mutation to apply heavy-tailed perturbations to real-valued genes with a configurable scale parameter | Inverse-CDF method (D-10); `cauchy_scale` config field (D-05); `Range<T>` type dispatch via `try_type!` macro pattern (VERIFIED: codebase) |
| MUT-02 | User can configure Lévy Flight mutation to apply long-range jumps to real-valued genes with a configurable stability index | Mantegna's algorithm (D-09); `levy_alpha` config field (D-06); Box-Muller normal samples reused from `gaussian.rs` (VERIFIED: codebase) |
| MUT-03 | User can configure Uniform mutation to randomly reset gene values uniformly within the gene's valid range | Full gene reset via `rng.random_range(lo..=hi)` analogous to `creep.rs`; multi-range via `range_idx` pattern (VERIFIED: codebase); no new config field |
</phase_requirements>

---

## Summary

Phase 33 adds three new real-valued mutation operators to the `Mutation` enum: `Cauchy`, `LevyFlight`, and `Uniform`. All three follow the established enum + factory + free-function pattern used by every existing range mutation operator. The technical domain is well-understood: inverse-CDF Cauchy sampling, Mantegna's Lévy approximation, and uniform random reset are standard continuous optimization techniques.

The codebase infrastructure is complete and mature. The primary implementation challenge is mechanical — correctly following the existing pattern established by `gaussian.rs`, `differential.rs`, and `creep.rs`. No new traits, no signature changes, no engine rewiring. The engine already routes non-Differential mutation variants through `factory_with_params`, so the three new operators will be picked up automatically once their match arms are added to `MutationOperator for Mutation`.

The key mathematical detail is Mantegna's σ_u formula for Lévy steps. For the default α=1.5, σ_u ≈ 0.6966 can be used as an inline constant, eliminating the need for a Gamma function implementation. For other α values, a simple closed-form computation using `f64::sin`, `f64::ln`, and `f64::powi` suffices without external dependencies.

**Primary recommendation:** Implement all three operators as thin free functions in dedicated files (`cauchy.rs`, `levy_flight.rs`, `uniform.rs`), each following the exact `gaussian.rs` structure with only the noise-generation section replaced. Wire into `MutationOperator for Mutation` match arms and `factory_with_params` dispatch. Add config fields and builder methods. Write tests modeled on `test_mutation_creep_gaussian.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Operator math (Cauchy/Lévy/Uniform noise) | `src/operations/mutation/<name>.rs` | — | Free functions; no state; matches all existing operators |
| Enum variant declaration | `src/operations.rs` | — | All `Mutation` variants live here |
| Factory dispatch | `src/operations/mutation.rs` (`MutationOperator for Mutation`) | — | Match arm routes variant to free function |
| Configuration fields | `src/configuration.rs` (`MutationConfiguration`) | — | Same struct as `differential_f`, `polynomial_eta` |
| Builder trait methods | `src/traits/configuration.rs` (`MutationConfig`) | `src/engines/ga.rs` (`Ga<U>`) | Trait declared here; `GaConfiguration` and `Ga<U>` both impl it |
| Engine integration | `src/engines/ga.rs` | other engines | Already handled via `factory_with_params`; no special-casing needed |
| Serde round-trip test | `tests/observe/test_serde.rs` | — | All enum variants must appear in the serde test array |
| Operator behavioral tests | `tests/operations/test_mutation_cauchy_levy_uniform.rs` | — | New dedicated test file following `test_mutation_creep_gaussian.rs` |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` | already in Cargo.toml | RNG (`random_range`, uniform sampling) | Project-standard; all existing operators use it |
| `log` | already in Cargo.toml | `debug!(target: "mutation_events", ...)` | Project-standard logging |

No new dependencies required. All math uses `f64` intrinsics (`sin`, `abs`, `ln`, `powi`, `clamp`). [VERIFIED: codebase — differential.rs, gaussian.rs use only `rand` and `log`]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` (feature-gated) | already in Cargo.toml | Serialize/Deserialize derives on `Mutation` enum | Only needed for `#[cfg_attr(feature = "serde", ...)]` — already on enum |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Inline Mantegna σ_u computation | `statrs` crate for Gamma function | `statrs` is not in dependencies; inline math handles the full range of α without adding a dep |
| Inverse-CDF Cauchy | `rand_distr::Cauchy` | `rand_distr` is not a project dependency; inverse-CDF is 1 line |

---

## Architecture Patterns

### System Architecture Diagram

```
MutationOperator::mutate() called by engine
         |
    match self { ... }
         |
    ┌────┴────────────────────────────────────────┐
    │  new variants                                │
    ├── Mutation::Cauchy  → cauchy_mutation<T>()   │
    ├── Mutation::LevyFlight → levy_mutation<T>()  │
    └── Mutation::Uniform → uniform_mutation<T>()  │
                                                   │
    Each free function:                            │
    ┌──────────────────────────────────────────┐   │
    │ 1. make_rng()                            │   │
    │ 2. pick gene idx (random_range(0..len))  │   │
    │ 3. pick range_idx (multi-range support)  │   │
    │ 4. compute noise (operator-specific)     │   │
    │ 5. new_val = (current ± noise).clamp()   │   │
    │ 6. set_gene(idx, gene)                   │   │
    └──────────────────────────────────────────┘   │
         |                                         │
    Config params passed via factory_with_params:  │
    (step, sigma args REUSED — or new direct args) │
    └────────────────────────────────────────────┘

config fields:
  MutationConfiguration {
    cauchy_scale: Option<f64>,   // new — D-05
    levy_alpha: Option<f64>,     // new — D-06
    ...existing fields...
  }

builder methods (MutationConfig trait + Ga<U> impl + GaConfiguration impl):
  .with_cauchy_scale(f64)
  .with_levy_alpha(f64)
```

### Recommended Project Structure

The three new files follow the existing `src/operations/mutation/` layout:

```
src/operations/mutation/
├── cauchy.rs          # new — Cauchy inverse-CDF perturbation
├── levy_flight.rs     # new — Mantegna's Lévy step
├── uniform.rs         # new — uniform gene reset
├── gaussian.rs        # reference implementation (existing)
├── creep.rs           # reference for type-generic pattern (existing)
├── differential.rs    # reference for try_type! macro (existing)
└── ...                # other existing operators
```

Test files (in `tests/operations/`):

```
tests/operations/
└── test_mutation_cauchy_levy_uniform.rs   # new
```

### Pattern 1: Range<T> Mutation Free Function

All three operators follow this exact skeleton from `gaussian.rs`:

```rust
// Source: src/operations/mutation/gaussian.rs (VERIFIED: codebase)
pub fn <name>_mutation<T>(individual: &mut RangeChromosome<T>, param: f64)
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
{
    let len = individual.dna().len();
    if len == 0 { return; }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut gene = individual.dna()[idx].clone();
    if gene.ranges.is_empty() { return; }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current: f64 = T::to_f64(gene.value);
    let lo_f64: f64 = T::to_f64(lo);
    let hi_f64: f64 = T::to_f64(hi);

    // === operator-specific noise generation replaces Box-Muller here ===
    let noise: f64 = /* ... */;

    let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);
}
```

### Pattern 2: Operator-specific noise generation

**Cauchy (D-10):**
```rust
// Source: CONTEXT.md D-10 [CITED: context]
// Inverse-CDF of standard Cauchy distribution
let u: f64 = rng.random_range(f64::EPSILON..1.0 - f64::EPSILON);
let noise: f64 = cauchy_scale * (std::f64::consts::PI * (u - 0.5)).tan();
```

**Lévy Flight — Mantegna's algorithm (D-09):**
```rust
// Source: CONTEXT.md D-09; Yang 2010 Cuckoo Search [CITED: context]
// σ_u for default α=1.5 can be inlined as ≈ 0.6966
// Full formula: σ_u = (Γ(1+α)*sin(πα/2) / (Γ((1+α)/2)*α*2^((α-1)/2)))^(1/α)
fn mantegna_sigma_u(alpha: f64) -> f64 {
    // Γ(x+1) = x! for positive integers, but for general f64 we use
    // the relation Γ(n+1) = n·Γ(n) and a small Lanczos or series.
    // For the typical range α ∈ (0, 2], a direct computation is stable:
    let num = gamma(1.0 + alpha) * (std::f64::consts::PI * alpha / 2.0).sin();
    let den = gamma((1.0 + alpha) / 2.0) * alpha * 2.0_f64.powf((alpha - 1.0) / 2.0);
    (num / den).powf(1.0 / alpha)
}
// Box-Muller for u ~ N(0, σ_u²):
let u1: f64 = rng.random_range(f64::EPSILON..1.0);
let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let u_sample: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_u;
// Box-Muller for v ~ N(0,1):
let v1: f64 = rng.random_range(f64::EPSILON..1.0);
let v2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let v_sample: f64 = (-2.0 * v1.ln()).sqrt() * v2.cos();
let levy_step: f64 = u_sample / v_sample.abs().powf(1.0 / alpha);
let noise: f64 = levy_step * (hi_f64 - lo_f64);  // scale by gene range width
```

**Uniform (D-03, D-04):**
```rust
// Source: CONTEXT.md D-03/D-04; mirrors creep.rs range_idx pattern [CITED: context]
// No noise formula — full reset to uniform sample within range
let new_val_f64: f64 = rng.random_range(lo_f64..=hi_f64);
// No 'current' used; set directly:
gene.value = T::from_f64(new_val_f64);
```

### Pattern 3: try_type! macro for MutationOperator match arm

New match arms in `MutationOperator for Mutation` (in `src/operations/mutation.rs`):

```rust
// Source: src/operations/mutation.rs existing Polynomial arm pattern [VERIFIED: codebase]
Mutation::Cauchy => {
    let scale = /* config_field */; // passed via step or a new arg
    return try_cauchy(individual, scale).unwrap_or_else(|| {
        Err(GaError::MutationError(
            "Cauchy mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    });
}
```

However, the `mutate` signature only receives `step: Option<f64>` and `sigma: Option<f64>`. The new config fields `cauchy_scale` and `levy_alpha` are NOT passed through `factory_with_params`. **See Pitfall 1 below for the resolution.**

### Pattern 4: factory_with_params call sites

`factory_with_params` is called from multiple engines:

```
src/engines/ga.rs          — primary engine
src/engines/nsga2/mod.rs   — NSGA-II engine
src/engines/cellular/engine.rs
src/engines/island/mod.rs
src/engines/island/nsga2.rs
src/engines/alps/engine.rs
```

[VERIFIED: codebase grep] All six call sites pass `step` and `sigma` from `mutation_configuration`. The new `cauchy_scale` and `levy_alpha` fields need to be routed through the same call chain. The cleanest approach (matching the `differential_f` precedent) is to have `factory_with_params` signature accept them or extract them from config — but `factory_with_params` does NOT take the full config. **See Pitfall 1.**

### Anti-Patterns to Avoid

- **Reusing `step` for `cauchy_scale` or `sigma` for `levy_alpha`:** D-08 explicitly forbids semantic overloading. Each operator-specific parameter must have its own named field.
- **Calling `factory_with_params` for Differential-style context-dependent ops:** The new operators are standard single-individual ops; they belong in the `factory_with_params` path.
- **Using `.to_vec()` unnecessarily:** `gaussian.rs` clones a single gene and calls `set_gene()`. Do not call `dna().to_vec()` and rebuild the whole DNA slice.
- **Not guarding against empty `ranges`:** If `gene.ranges.is_empty()`, return early (same guard as gaussian.rs).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Normal samples for Mantegna | Custom RNG or external crate | Box-Muller from gaussian.rs | Already in-repo; exact same pattern |
| Gamma function | General Gamma implementation | Inline Mantegna σ_u for α∈(0,2] using f64 trig | Range is bounded; no general Gamma needed |
| Cauchy sampling | Rejection sampling | Inverse-CDF: `scale * tan(π*(u-0.5))` | One line; no iteration |
| Uniform sampling | Custom uniform | `rng.random_range(lo..=hi)` | Already in creep.rs; rand provides this |

---

## Common Pitfalls

### Pitfall 1: `cauchy_scale` and `levy_alpha` are not passed through `factory_with_params`

**What goes wrong:** `factory_with_params(mutation, individual, step, sigma)` only has two optional f64 parameters. `cauchy_scale` and `levy_alpha` are new config fields. If the match arms for `Cauchy` and `LevyFlight` in `MutationOperator::mutate` try to use `step` or `sigma` for these values, it violates D-08 and produces confusing behavior.

**Why it happens:** The `factory_with_params` signature predates these operators. The six engine call sites pass `mutation_configuration.step` and `mutation_configuration.sigma` — not the new fields.

**How to avoid:** Expand `factory_with_params` to accept the new parameters OR — following the Differential precedent more closely — have the engine call sites extract `cauchy_scale` / `levy_alpha` from config and pass them alongside `step`/`sigma`. The cleanest backward-compatible approach is to add `cauchy_scale: Option<f64>` and `levy_alpha: Option<f64>` as additional parameters to `factory_with_params`, with defaults for callers. All six call sites must be updated.

**Alternative (simpler, avoids touching all engines):** Expose `cauchy_mutation` and `levy_mutation` as direct free functions (like `differential_mutation`) and have the `MutationOperator::mutate` impl call them directly with dummy defaults (read from a global or use hardcoded defaults in the trait impl). This is less clean but avoids touching six files.

**Recommended resolution:** Add two optional parameters to `factory_with_params` — this is consistent with how Creep/Gaussian already receive `step`/`sigma`, just extending the pattern. All six call sites pass `None` for backward compat. The planner must decide which approach to commit to.

**Warning signs:** Tests pass for default α=1.5 but break with custom `levy_alpha` values — indicates the config field is not being routed to the free function.

### Pitfall 2: `factory_non_value` not updated

**What goes wrong:** `factory_non_value` in `src/operations/mutation.rs` has an explicit arm for every `Mutation` variant. Adding new variants without adding arms causes a compile error (non-exhaustive match).

**Why it happens:** Rust's exhaustive pattern matching on enums. Every new variant must be handled.

**How to avoid:** Add `Mutation::Cauchy`, `Mutation::LevyFlight`, `Mutation::Uniform` arms to `factory_non_value` with appropriate `GaError::MutationError` messages (these operators require Range<T>, not valid for non-value use).

**Warning signs:** `cargo build` fails with "non-exhaustive patterns" referencing `factory_non_value`.

### Pitfall 3: Lévy step without range scaling produces scale-dependent behavior

**What goes wrong:** Applying `levy_step` directly to gene value without scaling by `(hi - lo)` makes behavior depend on the absolute gene range magnitude. A gene in `[0, 1]` would behave very differently from one in `[0, 1000]`.

**Why it happens:** Lévy step magnitude is in "natural" units. Gene ranges vary arbitrarily.

**How to avoid:** Scale: `noise = levy_step * (hi_f64 - lo_f64)` before adding to current value. Clamping then handles overshoot. [CITED: CONTEXT.md specifics section]

### Pitfall 4: Mantegna σ_u overflow for extreme α values

**What goes wrong:** For α very close to 0 or approaching 2, the Gamma function terms can overflow or produce NaN.

**Why it happens:** `Γ(1+α)` and `Γ((1+α)/2)` are well-behaved for α ∈ (0, 2], but the formula combines several operations.

**How to avoid:** Use the known safe default α=1.5 as the starting point. If implementing the full formula, add a guard: `alpha.clamp(0.1, 1.99)` in the `levy_alpha` getter or document the valid range. For α=1.5, use the precomputed constant σ_u ≈ 0.6966 to avoid any computation risk.

### Pitfall 5: Serde test missing new variants (Phase 32 CR-01 lesson)

**What goes wrong:** New `Mutation` enum variants added but not included in `serde_mutation_enum` test array in `tests/observe/test_serde.rs`. Phase 32 CR-01 caught this exact issue.

**Why it happens:** Test was written when fewer variants existed and is not auto-updated.

**How to avoid:** D-11 mandates this explicitly. The test file is at `tests/observe/test_serde.rs` lines 72-89. Add `Mutation::Cauchy`, `Mutation::LevyFlight`, `Mutation::Uniform` to the variants array.

### Pitfall 6: Integer types (i32, i64) with Uniform reset

**What goes wrong:** `rng.random_range(lo..=hi)` with `f64` bounds works directly. But the uniform reset operates on `f64` via `GaussianConvertible`, so `T::from_f64(rng.random_range(lo_f64..=hi_f64))` rounds for integer types — this is correct behavior but must be intentional.

**Why it happens:** The `f64`-intermediary conversion is required for all Range<T> operators. The rounding in `i32::from_f64` / `i64::from_f64` is `.round() as i32/i64`.

**How to avoid:** Use `GaussianConvertible` throughout (same as gaussian.rs) — don't try to call `rng.random_range` directly on `T` which doesn't implement `SampleUniform` in this trait bound context.

---

## Code Examples

### Cauchy noise generation (inverse-CDF)

```rust
// Source: CONTEXT.md D-10 — inverse-CDF of standard Cauchy distribution [CITED: context]
let u: f64 = rng.random_range(f64::EPSILON..1.0 - f64::EPSILON);
let noise: f64 = cauchy_scale * (std::f64::consts::PI * (u - 0.5)).tan();
let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
```

Note: Using `EPSILON` as lower bound guards against `tan(±π/2)` = ±infinity at u=0 or u=1.

### Mantegna σ_u for default α=1.5

```rust
// Source: Yang 2010 "Engineering Optimization" Ch. 9; CONTEXT.md D-09 [CITED: context]
// For α=1.5, σ_u ≈ 0.6966 — precomputed constant eliminates Gamma function need
const MANTEGNA_SIGMA_U_DEFAULT: f64 = 0.6966;

// General case using f64 trig (valid for α ∈ (0.1, 1.99)):
fn mantegna_sigma_u(alpha: f64) -> f64 {
    // Uses lgamma pattern: Γ(n) via series expansion
    // For the range α ∈ (0, 2], direct formula is numerically stable
    let num = gamma_approx(1.0 + alpha) * (std::f64::consts::PI * alpha / 2.0).sin();
    let den = gamma_approx((1.0 + alpha) / 2.0)
        * alpha
        * 2.0_f64.powf((alpha - 1.0) / 2.0);
    (num / den).powf(1.0 / alpha)
}
```

### Lévy step via two Box-Muller normal samples

```rust
// Source: gaussian.rs Box-Muller pattern (VERIFIED: codebase) + Mantegna's formula [CITED: context]
let sigma_u = mantegna_sigma_u(alpha); // or MANTEGNA_SIGMA_U_DEFAULT for alpha=1.5

// u ~ N(0, sigma_u^2)
let bu1: f64 = rng.random_range(f64::EPSILON..1.0);
let bu2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let u_normal: f64 = (-2.0 * bu1.ln()).sqrt() * bu2.cos() * sigma_u;

// v ~ N(0, 1)
let bv1: f64 = rng.random_range(f64::EPSILON..1.0);
let bv2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let v_normal: f64 = (-2.0 * bv1.ln()).sqrt() * bv2.cos();

let levy_step: f64 = u_normal / v_normal.abs().powf(1.0 / alpha);
let noise: f64 = levy_step * (hi_f64 - lo_f64);
let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
```

### Uniform reset

```rust
// Source: gaussian.rs range_idx pattern + creep.rs rng.random_range [VERIFIED: codebase]
let new_val_f64: f64 = rng.random_range(lo_f64..=hi_f64);
gene.value = T::from_f64(new_val_f64);
individual.set_gene(idx, gene);
// return Ok(()) — no 'noise' computed, no clamp needed (already within range)
```

### New match arms in `MutationOperator for Mutation`

```rust
// Source: src/operations/mutation.rs Polynomial arm pattern [VERIFIED: codebase]
Mutation::Cauchy => {
    let scale = step.unwrap_or(1.0); // step reused OR new param — see Pitfall 1
    return try_cauchy(individual, scale).unwrap_or_else(|| {
        Err(GaError::MutationError(
            "Cauchy mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    });
}
Mutation::LevyFlight => {
    let alpha = sigma.unwrap_or(1.5); // sigma reused OR new param — see Pitfall 1
    return try_levy(individual, alpha).unwrap_or_else(|| {
        Err(GaError::MutationError(
            "Lévy Flight mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    });
}
Mutation::Uniform => {
    return try_uniform(individual).unwrap_or_else(|| {
        Err(GaError::MutationError(
            "Uniform mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    });
}
```

**Note on Pitfall 1 resolution in match arms:** The `mutate` signature only has `step` and `sigma`. Until `factory_with_params` is extended, the planner must decide whether to temporarily wire `cauchy_scale` via `step` and `levy_alpha` via `sigma` in the engine call sites, or to extend `factory_with_params`. The existing `step`/`sigma` fields are unused for Cauchy/LevyFlight so semantic overloading within the factory signature would be transparent to users (who use named builder methods). The planner should make a definitive decision here.

### MutationConfiguration additions

```rust
// Source: src/configuration.rs differential_f pattern [VERIFIED: codebase]
/// Scale parameter (γ) for Cauchy mutation. Default is 1.0 when `None`.
/// Only used when `method` is `Mutation::Cauchy`.
pub cauchy_scale: Option<f64>,
/// Stability index (α) for Lévy Flight mutation. Default is 1.5 when `None`.
/// Typical range: (0, 2]. Only used when `method` is `Mutation::LevyFlight`.
pub levy_alpha: Option<f64>,
```

Default impl additions:
```rust
cauchy_scale: None,
levy_alpha: None,
```

### Builder trait and impl additions

In `src/traits/configuration.rs` (`MutationConfig` trait):
```rust
// Source: with_differential_f pattern [VERIFIED: codebase]
/// Sets the scale parameter (γ) for Cauchy mutation. Default is 1.0.
fn with_cauchy_scale(self, scale: f64) -> Self;
/// Sets the stability index (α) for Lévy Flight mutation. Default is 1.5.
fn with_levy_alpha(self, alpha: f64) -> Self;
```

In `src/configuration.rs` (`impl MutationConfig for GaConfiguration`):
```rust
fn with_cauchy_scale(mut self, scale: f64) -> Self {
    self.mutation_configuration.cauchy_scale = Some(scale);
    self
}
fn with_levy_alpha(mut self, alpha: f64) -> Self {
    self.mutation_configuration.levy_alpha = Some(alpha);
    self
}
```

Same pattern in `src/engines/ga.rs` (`impl MutationConfig for Ga<U>`):
```rust
fn with_cauchy_scale(mut self, scale: f64) -> Self {
    self.configuration.mutation_configuration.cauchy_scale = Some(scale);
    self
}
fn with_levy_alpha(mut self, alpha: f64) -> Self {
    self.configuration.mutation_configuration.levy_alpha = Some(alpha);
    self
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-rolled Gamma for Mantegna | Inline σ_u formula using f64 trig only | N/A — design choice | No external dependency |
| `sigma` reused for all perturbation params | Separate named fields per operator | Phase 32 (differential_f pattern) | Cleaner config API |

**Deprecated/outdated:**
- None relevant to this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Six engine call sites all follow the same `factory_with_params(method, individual, step, sigma)` pattern and none special-case Cauchy/Lévy | Architecture Patterns / Pitfall 1 | If any engine has additional dispatch logic, the match arm approach may need adjustment |
| A2 | σ_u ≈ 0.6966 is accurate for α=1.5 to sufficient precision for GA use | Code Examples | Minor numerical inaccuracy in Lévy steps; not a correctness issue for GA |
| A3 | `rand`'s `random_range(f64::EPSILON..1.0 - f64::EPSILON)` avoids tan(±π/2) in practice | Code Examples | Numerical edge case if EPSILON bound is too loose; negligible in practice |

**Verified claims:** All codebase file structures, enum variants, method signatures, pattern locations confirmed by direct file reads in this session.

---

## Open Questions

1. **`factory_with_params` parameter extension vs. in-match config access**
   - What we know: `factory_with_params(mutation, individual, step, sigma)` is called from 6 engine files. `cauchy_scale` and `levy_alpha` are not accessible inside `MutationOperator::mutate` unless passed through.
   - What's unclear: The planner must choose one of: (a) extend `factory_with_params` with 2 new optional params — clean but requires 6 call-site updates; (b) route `cauchy_scale` through `step` and `levy_alpha` through `sigma` inside engine call sites only for Cauchy/LevyFlight — minimal changes but semantically overloads the params; (c) follow Differential's precedent and handle them in the engine's conditional block — cleanest semantics but adds engine complexity.
   - Recommendation: Option (b) is simplest and backward-compatible: in the 6 engine call sites, when `method == Mutation::Cauchy`, pass `mutation_configuration.cauchy_scale` as `step`; when `method == Mutation::LevyFlight`, pass `mutation_configuration.levy_alpha` as `sigma`. Inside `MutationOperator::mutate`, the match arms just use `step.unwrap_or(1.0)` and `sigma.unwrap_or(1.5)`. Users only interact with the named builder methods — the internal routing is invisible.

2. **Gamma function for non-default α values**
   - What we know: CONTEXT.md delegates this to Claude's discretion. Inline computation with f64 trig is feasible for α ∈ (0, 2].
   - What's unclear: The simplest correct implementation for Γ in the range needed.
   - Recommendation: Use the Stirling-based approximation or the Lanczos approximation for general α. For the specific range (0, 2], even a simple recursion `Γ(x+1) = x·Γ(x)` with `Γ(0.5) = sqrt(π)` and `Γ(1) = 1` covers all needed evaluations. Plan should include a private `gamma_approx(x: f64) -> f64` helper in `levy_flight.rs`.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — all math uses `std` and existing `rand`/`log` crates already in Cargo.toml).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (`cargo test`) |
| Config file | none — standard Cargo test discovery |
| Quick run command | `cargo test test_mutation_cauchy` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MUT-01 | Cauchy mutates exactly one gene per call | unit | `cargo test -p genetic_algorithms test_mutation_cauchy` | ❌ Wave 0 |
| MUT-01 | Cauchy output clamped to gene range | unit | `cargo test -p genetic_algorithms cauchy_stays_in_range` | ❌ Wave 0 |
| MUT-01 | Cauchy returns error for non-Range chromosomes | unit | `cargo test -p genetic_algorithms cauchy_error_on_binary` | ❌ Wave 0 |
| MUT-01 | Cauchy scale config wired correctly (default 1.0) | unit | `cargo test -p genetic_algorithms cauchy_default_scale` | ❌ Wave 0 |
| MUT-02 | LevyFlight mutates exactly one gene per call | unit | `cargo test -p genetic_algorithms levy_mutates_one_gene` | ❌ Wave 0 |
| MUT-02 | LevyFlight output clamped to gene range | unit | `cargo test -p genetic_algorithms levy_stays_in_range` | ❌ Wave 0 |
| MUT-02 | LevyFlight returns error for non-Range chromosomes | unit | `cargo test -p genetic_algorithms levy_error_on_binary` | ❌ Wave 0 |
| MUT-03 | Uniform resets gene to within declared range | unit | `cargo test -p genetic_algorithms uniform_stays_in_range` | ❌ Wave 0 |
| MUT-03 | Uniform mutates exactly one gene per call | unit | `cargo test -p genetic_algorithms uniform_mutates_one_gene` | ❌ Wave 0 |
| MUT-03 | Uniform returns error for non-Range chromosomes | unit | `cargo test -p genetic_algorithms uniform_error_on_binary` | ❌ Wave 0 |
| D-11 | Serde round-trip for all three new variants | unit | `cargo test --features serde serde_mutation_enum` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test` (fast, no serde)
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/operations/test_mutation_cauchy_levy_uniform.rs` — covers MUT-01, MUT-02, MUT-03 behavioral tests
- [ ] Update `tests/observe/test_serde.rs` serde_mutation_enum array — covers D-11

*(Existing test infrastructure: `test_mutation_creep_gaussian.rs` as structural template.)*

---

## Security Domain

Phase 33 adds numerical mutation operators with no I/O, authentication, session, cryptographic, or access-control surface. No ASVS categories apply.

| ASVS Category | Applies | Notes |
|---------------|---------|-------|
| V5 Input Validation | Marginal | `cauchy_scale` and `levy_alpha` are f64 from config; should document valid ranges in rustdoc (α ∈ (0, 2]) |
| All others | No | Pure in-memory numerical computation |

---

## Sources

### Primary (HIGH confidence)
- `src/operations/mutation/gaussian.rs` — canonical Range<T> mutation pattern; all implementation decisions verified against this file [VERIFIED: codebase]
- `src/operations/mutation/differential.rs` — `try_type!` macro pattern and error path [VERIFIED: codebase]
- `src/operations/mutation/creep.rs` — uniform sampling pattern (`rng.random_range`) [VERIFIED: codebase]
- `src/operations/mutation.rs` — `MutationOperator for Mutation` impl; `factory_with_params` signature; `factory_non_value` exhaustiveness requirement [VERIFIED: codebase]
- `src/operations.rs` — `Mutation` enum; existing variant list; derive pattern [VERIFIED: codebase]
- `src/configuration.rs` — `MutationConfiguration` struct; `differential_f` naming pattern; builder impl [VERIFIED: codebase]
- `src/traits/configuration.rs` — `MutationConfig` trait; `with_differential_f` builder method pattern [VERIFIED: codebase]
- `src/engines/ga.rs` — six engine call sites for `factory_with_params`; Differential special-case dispatch; MutationConfig impl on Ga<U> [VERIFIED: codebase]
- `tests/observe/test_serde.rs` — serde mutation enum test array; exact lines requiring update [VERIFIED: codebase]
- `.planning/phases/33-scalar-mutation-operators/33-CONTEXT.md` — all locked decisions [CITED: context]

### Secondary (MEDIUM confidence)
- Yang, X.S. (2010). *Engineering Optimization: An Introduction with Metaheuristic Applications*. — Mantegna's algorithm for Lévy step; σ_u formula [CITED: context D-09]

### Tertiary (LOW confidence)
- σ_u ≈ 0.6966 for α=1.5 — numeric constant from training knowledge, consistent with Yang 2010 formula; not independently verified by computation in this session [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all existing patterns verified in codebase
- Architecture: HIGH — all integration points read directly from source
- Pitfalls: HIGH — derived from direct code inspection of call sites and existing patterns
- Lévy σ_u constant: LOW — numeric value is [ASSUMED]; planner should verify or compute at implementation time

**Research date:** 2026-05-06
**Valid until:** 2026-06-06 (stable codebase; mutation operator infrastructure is mature)
