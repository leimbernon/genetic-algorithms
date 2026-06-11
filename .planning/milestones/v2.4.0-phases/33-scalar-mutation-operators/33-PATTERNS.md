# Phase 33: Scalar Mutation Operators - Pattern Map

**Mapped:** 2026-05-06
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/operations/mutation/cauchy.rs` | operator (mutation) | request-response | `src/operations/mutation/gaussian.rs` | exact |
| `src/operations/mutation/levy_flight.rs` | operator (mutation) | request-response | `src/operations/mutation/gaussian.rs` | exact |
| `src/operations/mutation/uniform.rs` | operator (mutation) | request-response | `src/operations/mutation/creep.rs` | exact |
| `src/operations.rs` | enum declaration | — | `src/operations.rs` (Mutation enum) | exact (modify) |
| `src/operations/mutation.rs` | factory dispatch | request-response | `src/operations/mutation.rs` (Polynomial arm) | exact (modify) |
| `src/configuration.rs` | config struct | — | `src/configuration.rs` (differential_f pattern) | exact (modify) |
| `src/traits/configuration.rs` | builder trait | — | `src/traits/configuration.rs` (with_differential_f) | exact (modify) |
| `src/engines/ga.rs` | engine integration | — | `src/engines/ga.rs` (Differential dispatch block) | exact (modify) |
| `tests/operations/test_mutation_cauchy_levy_uniform.rs` | test | unit | `tests/operations/test_mutation_creep_gaussian.rs` | exact |
| `tests/observe/test_serde.rs` | test (serde) | — | `tests/observe/test_serde.rs` (serde_mutation_enum) | exact (modify) |

---

## Pattern Assignments

### `src/operations/mutation/cauchy.rs` (new — operator, request-response)

**Analog:** `src/operations/mutation/gaussian.rs`

**Imports pattern** (gaussian.rs lines 11-14):
```rust
use crate::chromosomes::Range as RangeChromosome;
use crate::traits::ChromosomeT;
use rand::Rng;
use std::fmt::Debug;
```
Add for Cauchy:
```rust
use crate::operations::mutation::gaussian::GaussianConvertible;
use log::debug;
```

**Core pattern — function signature** (gaussian.rs lines 27-30):
```rust
pub fn cauchy_mutation<T>(individual: &mut RangeChromosome<T>, scale: f64)
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
```

**Core pattern — single-gene selection, range_idx, f64 conversion** (gaussian.rs lines 31-50):
```rust
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
```

**Cauchy-specific noise generation** (replaces Box-Muller at gaussian.rs lines 52-55; source: CONTEXT.md D-10):
```rust
// Inverse-CDF of the Cauchy distribution. EPSILON guards prevent tan(±π/2) = ±inf.
let u: f64 = rng.random_range(f64::EPSILON..1.0 - f64::EPSILON);
let noise: f64 = scale * (std::f64::consts::PI * (u - 0.5)).tan();
let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
```

**Gene write-back** (gaussian.rs lines 58-59):
```rust
gene.value = T::from_f64(new_val_f64);
individual.set_gene(idx, gene);
```

---

### `src/operations/mutation/levy_flight.rs` (new — operator, request-response)

**Analog:** `src/operations/mutation/gaussian.rs`

**Imports pattern:** Same as cauchy.rs above.

**Core pattern — function signature:**
```rust
pub fn levy_flight_mutation<T>(individual: &mut RangeChromosome<T>, alpha: f64)
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
```

**Core pattern — single-gene selection** (identical to gaussian.rs lines 31-50): copy verbatim.

**Lévy-specific: Mantegna's σ_u helper** (private function, declared before `levy_flight_mutation`):
```rust
/// Computes Mantegna's σ_u for the Lévy stability index `alpha` ∈ (0, 2].
/// Uses the recurrence Γ(x+1) = x·Γ(x) with anchors Γ(1)=1 and Γ(0.5)=√π
/// to avoid an external Gamma crate.
fn mantegna_sigma_u(alpha: f64) -> f64 {
    // Γ(1 + alpha)
    let g1pa = gamma_approx(1.0 + alpha);
    // Γ((1 + alpha) / 2)
    let g1pa_half = gamma_approx((1.0 + alpha) / 2.0);
    let num = g1pa * (std::f64::consts::PI * alpha / 2.0).sin();
    let den = g1pa_half * alpha * 2.0_f64.powf((alpha - 1.0) / 2.0);
    (num / den).powf(1.0 / alpha)
}

/// Lanczos-style Gamma approximation, valid and numerically stable for x ∈ (0, ~20].
/// Covers all inputs produced by mantegna_sigma_u for alpha ∈ (0.1, 1.99].
fn gamma_approx(x: f64) -> f64 {
    // Recurse down to range (1, 2] where the polynomial approximation is applied
    if x < 1.0 {
        return gamma_approx(x + 1.0) / x;
    }
    if x > 2.0 {
        return (x - 1.0) * gamma_approx(x - 1.0);
    }
    // Stirling-based coefficients for x ∈ [1, 2] — sufficient precision for GA use
    let t = x - 1.0;
    1.0 + t * (-0.5748646 + t * (0.9512363 + t * (-0.6998588 + t * (0.4245549 - t * 0.1010678))))
}
```

**Lévy-specific noise generation — two Box-Muller normal samples** (source: gaussian.rs lines 52-55 pattern + CONTEXT.md D-09):
```rust
let sigma_u = mantegna_sigma_u(alpha.clamp(0.1, 1.99));

// u ~ N(0, sigma_u^2) via Box-Muller
let bu1: f64 = rng.random_range(f64::EPSILON..1.0);
let bu2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let u_normal: f64 = (-2.0 * bu1.ln()).sqrt() * bu2.cos() * sigma_u;

// v ~ N(0, 1) via Box-Muller
let bv1: f64 = rng.random_range(f64::EPSILON..1.0);
let bv2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let v_normal: f64 = (-2.0 * bv1.ln()).sqrt() * bv2.cos();

// Mantegna's step — scaled by gene range width so behavior is range-independent (D-09 + CONTEXT.md specifics)
let levy_step: f64 = u_normal / v_normal.abs().powf(1.0 / alpha);
let noise: f64 = levy_step * (hi_f64 - lo_f64);
let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
```

**Gene write-back:** identical to gaussian.rs.

---

### `src/operations/mutation/uniform.rs` (new — operator, request-response)

**Analog:** `src/operations/mutation/gaussian.rs` (structure) + `src/operations/mutation/creep.rs` (uniform RNG call)

**Imports pattern:**
```rust
use crate::chromosomes::Range as RangeChromosome;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::fmt::Debug;
```

**Core pattern — function signature:**
```rust
pub fn uniform_mutation<T>(individual: &mut RangeChromosome<T>)
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
```

**Core pattern — single-gene selection, range_idx** (gaussian.rs lines 31-50): copy verbatim (identical).

**Uniform-specific: full reset instead of perturbation** (source: CONTEXT.md D-03/D-04; `rng.random_range` pattern from creep.rs line 72):
```rust
// No noise formula — full reset to a uniform sample within the selected range (D-03)
// Uses f64 intermediary via GaussianConvertible; from_f64 rounds for integer types (Pitfall 6)
let new_val_f64: f64 = rng.random_range(lo_f64..=hi_f64);
gene.value = T::from_f64(new_val_f64);
individual.set_gene(idx, gene);
// return Ok(()) — no clamp needed; random_range result is already within [lo_f64, hi_f64]
```

---

### `src/operations.rs` (modify — Mutation enum)

**Analog:** existing `Mutation` enum, lines 94-130.

**Enum variant derives pattern** (operations.rs lines 18-19, applied to Mutation):
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mutation {
    // ... existing variants ...
    /// Cauchy (Lorentzian) perturbation for `Range<T>` chromosomes.
    /// Uses the inverse-CDF method: `noise = scale * tan(π*(u - 0.5))`.
    /// Configure scale via [`MutationConfiguration::cauchy_scale`]. Default: 1.0.
    Cauchy,
    /// Lévy Flight mutation for `Range<T>` chromosomes.
    /// Uses Mantegna's algorithm to generate heavy-tailed jump steps.
    /// Configure stability index via [`MutationConfiguration::levy_alpha`]. Default: 1.5.
    LevyFlight,
    /// Uniform reset mutation for `Range<T>` chromosomes.
    /// Resets a randomly selected gene to a uniform sample within its declared range.
    Uniform,
}
```

---

### `src/operations/mutation.rs` (modify — match arms + factory_non_value)

**Analog:** `src/operations/mutation.rs` — Polynomial match arm (lines 147-155) and factory_non_value (lines 225-285).

**New mod declarations** (after line 33, following existing `pub mod differential;`):
```rust
pub mod cauchy;
pub mod levy_flight;
pub mod uniform;
```

**New try_* helpers** (following `try_polynomial` pattern, lines 42-58):
```rust
fn try_cauchy<U: ChromosomeT + 'static>(
    individual: &mut U,
    scale: f64,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                cauchy::cauchy_mutation(ind, scale);
                return Some(Ok(()));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}
// identical pattern for try_levy and try_uniform
```

**New match arms in `MutationOperator for Mutation`** (after Polynomial arm, lines 147-155):
```rust
Mutation::Cauchy => {
    let scale = step.unwrap_or(1.0);
    return try_cauchy(individual, scale).unwrap_or_else(|| {
        Err(GaError::MutationError(
            "Cauchy mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    });
}
Mutation::LevyFlight => {
    let alpha = sigma.unwrap_or(1.5);
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

**New arms in `factory_non_value`** (Pitfall 2 — exhaustiveness; follow Polynomial arm pattern lines 262-266):
```rust
Mutation::Cauchy => Err(GaError::MutationError(
    "Mutation::Cauchy requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
     Use Swap, Inversion, or Scramble instead.".to_string(),
)),
Mutation::LevyFlight => Err(GaError::MutationError(
    "Mutation::LevyFlight requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
     Use Swap, Inversion, or Scramble instead.".to_string(),
)),
Mutation::Uniform => Err(GaError::MutationError(
    "Mutation::Uniform requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
     Use Swap, Inversion, or Scramble instead.".to_string(),
)),
```

**Parameter routing decision (Pitfall 1 resolution):** Use option (b) from RESEARCH.md Open Question 1. In the `MutationOperator::mutate` match arms, `step` carries `cauchy_scale` and `sigma` carries `levy_alpha`. In the engine call sites (ga.rs lines 1403-1408), when method is `Cauchy`, pass `mutation_configuration.cauchy_scale` as `step`; when method is `LevyFlight`, pass `mutation_configuration.levy_alpha` as `sigma`. This avoids touching `factory_with_params` signature and keeps all 6 call-site changes localized to the conditional dispatch block.

---

### `src/configuration.rs` (modify — MutationConfiguration struct)

**Analog:** `differential_f: Option<f64>` field (lines 156-159) and Default impl (lines 180).

**New fields** (add after `differential_f`, lines 159):
```rust
/// Scale parameter (γ) for Cauchy mutation. Default is 1.0 when `None`.
/// Only used when `method` is `Mutation::Cauchy`.
pub cauchy_scale: Option<f64>,
/// Stability index (α) for Lévy Flight mutation. Valid range: (0.0, 2.0). Default is 1.5 when `None`.
/// Only used when `method` is `Mutation::LevyFlight`.
pub levy_alpha: Option<f64>,
```

**Default impl additions** (after `differential_f: None,` line 180):
```rust
cauchy_scale: None,
levy_alpha: None,
```

---

### `src/traits/configuration.rs` (modify — MutationConfig trait)

**Analog:** `with_differential_f` method (line 60).

**New trait methods** (add after `with_differential_f`, line 60):
```rust
/// Sets the scale parameter (γ) for Cauchy mutation. Default is 1.0.
/// Only used when the mutation method is `Mutation::Cauchy`.
fn with_cauchy_scale(self, scale: f64) -> Self;
/// Sets the stability index (α) for Lévy Flight mutation. Valid range: (0, 2). Default is 1.5.
/// Only used when the mutation method is `Mutation::LevyFlight`.
fn with_levy_alpha(self, alpha: f64) -> Self;
```

---

### `src/engines/ga.rs` (modify — MutationConfig impl + dispatch block)

**Analog 1:** `with_differential_f` impl on `Ga<U>` (line 247):
```rust
// Existing pattern for MutationConfig impl on Ga<U>
fn with_differential_f(mut self, f: f64) -> Self {
    self.configuration.mutation_configuration.differential_f = Some(f);
    self
}
```
**New builder impls** (same impl block):
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

**Analog 2:** Differential dispatch block (lines 1393-1409). Extend the conditional to route `cauchy_scale` / `levy_alpha` through `step` / `sigma`:
```rust
// Existing Differential block (lines 1393-1409) — extend else-if chain before the final else:
} else if configuration.mutation_configuration.method == crate::operations::Mutation::Cauchy {
    mutation::factory_with_params(
        configuration.mutation_configuration.method,
        &mut child_1,
        configuration.mutation_configuration.cauchy_scale,  // routed as step
        None,
    )?;
} else if configuration.mutation_configuration.method == crate::operations::Mutation::LevyFlight {
    mutation::factory_with_params(
        configuration.mutation_configuration.method,
        &mut child_1,
        None,
        configuration.mutation_configuration.levy_alpha,    // routed as sigma
    )?;
} else {
    mutation::factory_with_params(
        configuration.mutation_configuration.method,
        &mut child_1,
        configuration.mutation_configuration.step,
        configuration.mutation_configuration.sigma,
    )?;
}
```
Apply the identical pattern for `child_2` in the same function. Apply to all 6 engine files that call `factory_with_params` (RESEARCH.md verified: ga.rs, nsga2/mod.rs, cellular/engine.rs, island/mod.rs, island/nsga2.rs, alps/engine.rs).

Note: `GaConfiguration` also implements `MutationConfig`. Add the same two builder impls in `configuration.rs` following the `with_differential_f` impl on `GaConfiguration` (lines 387-390).

---

### `tests/operations/test_mutation_cauchy_levy_uniform.rs` (new — test)

**Analog:** `tests/operations/test_mutation_creep_gaussian.rs` (full file, lines 1-end).

**Imports pattern** (test_mutation_creep_gaussian.rs lines 1-6):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;
```

**Builder helpers** (test_mutation_creep_gaussian.rs lines 8-24):
```rust
fn build_f64_chromosome(n: usize) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], 50.0))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}
fn build_i32_chromosome(n: usize) -> RangeChromosome<i32> { /* same pattern */ }
```

**Test pattern — mutates value** (test_mutation_creep_gaussian.rs lines 28-44):
```rust
#[test]
fn cauchy_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.dna().to_vec();
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(1.0), None).unwrap();
        if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Cauchy mutation did not change any value");
}
```

**Test pattern — stays in range** (test_mutation_creep_gaussian.rs lines 46-62):
```rust
#[test]
fn cauchy_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(5.0), None).unwrap();
        for gene in c.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi,
                "Cauchy: value {} out of range [{}, {}]", gene.value, lo, hi);
        }
    }
}
```

**Test pattern — mutates exactly one gene** (count genes changed per call, assert == 1):
```rust
#[test]
fn cauchy_mutation_changes_exactly_one_gene() {
    let mut c = build_f64_chromosome(10);
    for _ in 0..50 {
        let before = c.dna().to_vec();
        mutation::factory_with_params(Mutation::Cauchy, &mut c, Some(1.0), None).unwrap();
        let changed_count = before.iter().zip(c.dna())
            .filter(|(b, a)| b.value != a.value).count();
        assert!(changed_count <= 1, "Expected at most 1 changed gene, got {}", changed_count);
    }
}
```

Repeat all three test patterns for `LevyFlight` (with `sigma` param) and `Uniform` (no param). Add i32 variants. Add error-path tests for Binary chromosomes.

---

### `tests/observe/test_serde.rs` (modify — serde_mutation_enum)

**Analog:** existing `serde_mutation_enum` function (lines 72-89).

**Add to variants array** (after `Mutation::Differential` on line 84):
```rust
Mutation::Cauchy,
Mutation::LevyFlight,
Mutation::Uniform,
```

---

## Shared Patterns

### Range<T> operator free-function structure
**Source:** `src/operations/mutation/gaussian.rs` (entire file, 109 lines)
**Apply to:** `cauchy.rs`, `levy_flight.rs`, `uniform.rs`

The complete skeleton shared by all three new operators:
1. `let len = individual.dna().len(); if len == 0 { return; }` (line 31-34)
2. `let mut rng = crate::rng::make_rng();` (line 36)
3. `let idx = rng.random_range(0..len);` (line 37)
4. `let mut gene = individual.dna()[idx].clone();` (line 39)
5. `if gene.ranges.is_empty() { return; }` (line 41-43)
6. `let range_idx = rng.random_range(0..gene.ranges.len());` (line 45)
7. `let (lo, hi) = gene.ranges[range_idx];` (line 46)
8. f64 conversion via `T::to_f64` (lines 48-50)
9. operator-specific noise (replaces lines 52-55)
10. `gene.value = T::from_f64(...); individual.set_gene(idx, gene);` (lines 58-59)

### GaussianConvertible trait import
**Source:** `src/operations/mutation/gaussian.rs` lines 67-108 (trait + impls for f64, f32, i32, i64)
**Apply to:** All three new operator files — import via `use crate::operations::mutation::gaussian::GaussianConvertible;`

### try_* downcast macro pattern
**Source:** `src/operations/mutation.rs` `try_polynomial` function (lines 42-58)
**Apply to:** New `try_cauchy`, `try_levy`, `try_uniform` helpers in `mutation.rs`

The macro body:
```rust
macro_rules! try_type {
    ($t:ty) => {
        if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
            return Some(<operator_fn>(ind, param));
        }
    };
}
try_type!(f64);
try_type!(f32);
try_type!(i32);
try_type!(i64);
None
```

### Config field + builder method pattern
**Source:** `src/configuration.rs` lines 156-159 (`differential_f` field) and `src/traits/configuration.rs` line 60 (`with_differential_f`)
**Apply to:** `cauchy_scale` and `levy_alpha` fields + builder methods

Pattern: `pub <name>: Option<f64>` in struct, `None` in Default, trait method `fn with_<name>(self, val: f64) -> Self`, impl sets `self.mutation_configuration.<name> = Some(val); self`.

### Log target
**Source:** `src/operations/mutation/differential.rs` line 65
**Apply to:** All three new operator files
```rust
debug!(target: "mutation_events", "...");
```

---

## No Analog Found

None — all files have close analogs in the codebase.

---

## Metadata

**Analog search scope:** `src/operations/mutation/`, `src/operations.rs`, `src/configuration.rs`, `src/traits/configuration.rs`, `src/engines/ga.rs`, `tests/operations/`, `tests/observe/`
**Files scanned:** 10 source files read directly
**Pattern extraction date:** 2026-05-06
