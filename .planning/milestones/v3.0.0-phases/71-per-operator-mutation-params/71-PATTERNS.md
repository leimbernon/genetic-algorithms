# Phase 71: Per-Operator Mutation Parameters - Pattern Map

**Mapped:** 2026-06-18
**Files analyzed:** 11 (primary src files + engine call sites)
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/operations.rs` | model/enum | transform | `src/operations.rs` (existing enum + derive pattern) | self |
| `src/operations/mutation.rs` | service/dispatch | request-response | `src/operations/mutation.rs` (existing factory + match dispatch) | self |
| `src/engines/ga/generation.rs` | engine/call-site | event-driven | `src/engines/island/mod.rs` (same factory_with_chromosome_length pattern) | exact |
| `src/engines/island/mod.rs` | engine/call-site | event-driven | `src/engines/ga/generation.rs` (same call pattern) | exact |
| `src/engines/moead/mod.rs` | engine/guard | request-response | `src/engines/nsga2/mod.rs` (identical wildcard guard pattern) | exact |
| `src/engines/nsga2/mod.rs` | engine/guard | request-response | `src/engines/moead/mod.rs` | exact |
| `src/engines/nsga3/mod.rs` | engine/guard | request-response | `src/engines/moead/mod.rs` | exact |
| `src/engines/cellular/configuration.rs` | config | CRUD | `src/engines/alps/configuration.rs` (identical construction site) | exact |
| `src/engines/alps/configuration.rs` | config | CRUD | `src/engines/cellular/configuration.rs` | exact |

---

## Pattern Assignments

### `src/operations.rs` — Param struct definitions + enum variant reshape

**Analog:** Same file, existing `Mutation` enum (lines 253–391) and sibling enums `Selection` (line 55), `Crossover` (line 116), `Extension` (line 142).

**Derive + serde pattern** (lines 253–254, replicated on every sibling enum):
```rust
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mutation { ... }
```

**Inline field variant pattern** (lines 270–274, 279–283 — the before-state):
```rust
Creep {
    /// Step size for the perturbation. Default: `0.01`.
    #[cfg_attr(feature = "serde", serde(default))]
    step: Option<f64>,
},
Gaussian {
    /// Standard deviation of the Gaussian noise. Default: `0.1`.
    #[cfg_attr(feature = "serde", serde(default))]
    sigma: Option<f64>,
},
```

**Target pattern — param struct definition** (copy derive list from `Mutation` minus `Copy` since `Mutation` doesn't have `Copy`; sibling enums at line 55 use `Copy, Clone, Debug, PartialEq` but `Mutation` uses `Clone, Debug, PartialEq` only):
```rust
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaussianParams {
    /// Standard deviation of the Gaussian noise. Default: `0.1`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma: Option<f64>,
}
```

**Target pattern — tuple variant** (reshape from inline struct to tuple):
```rust
// Before (lines 279–283)
Gaussian {
    #[cfg_attr(feature = "serde", serde(default))]
    sigma: Option<f64>,
},

// After
Gaussian(GaussianParams),
```

**Multi-field struct pattern** (SelfAdaptiveGaussian, lines 377–390 — 4 fields):
```rust
SelfAdaptiveGaussian {
    #[cfg_attr(feature = "serde", serde(default))]
    tau: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    tau_prime: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    sigma_min: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    sigma_max: Option<f64>,
},
```

**Doc example in enum docstring** (line 251 — must update to new construction syntax):
```rust
// Before
.with_mutation_method(Mutation::Gaussian { sigma: Some(0.1) });
// After
.with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: Some(0.1) }));
```

---

### `src/operations/mutation.rs` — Match arm destructuring, factory cleanup

**Analog:** Same file, existing match arms and factory functions.

**Existing match arm pattern with field destructuring** (lines 183–211 — before state):
```rust
Mutation::Cauchy { scale } => {
    let s = scale.unwrap_or(1.0);
    return individual.cauchy_mutation(s);
}
Mutation::LevyFlight { alpha } => {
    let a = alpha.unwrap_or(1.5);
    return individual.levy_flight_mutation(a);
}
Mutation::SelfAdaptiveGaussian {
    tau,
    tau_prime,
    sigma_min,
    sigma_max,
} => {
    let n_hint = individual.dna().len().max(1);
    let effective_tau = tau.unwrap_or_else(|| 1.0 / (2.0 * n_hint as f64).sqrt());
    let effective_tau_prime =
        tau_prime.unwrap_or_else(|| 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt());
    let effective_sigma_min = sigma_min.unwrap_or(1e-5_f64);
    return individual.self_adaptive_gaussian_mutation(
        effective_tau,
        effective_tau_prime,
        effective_sigma_min,
        *sigma_max,
    );
}
```

**Target pattern — tuple destructuring** (change `{ field }` to `(ParamStruct { field })`):
```rust
Mutation::Cauchy(CauchyParams { scale }) => {
    let s = scale.unwrap_or(1.0);
    return individual.cauchy_mutation(s);
}
Mutation::LevyFlight(LevyFlightParams { alpha }) => {
    let a = alpha.unwrap_or(1.5);
    return individual.levy_flight_mutation(a);
}
Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
    tau,
    tau_prime,
    sigma_min,
    sigma_max,
}) => {
    // body unchanged — only the destructuring pattern changes
}
```

**Wildcard arm pattern** (line 175 — before state):
```rust
Mutation::Differential { .. } => {
    return Err(GaError::MutationError("...".to_string()));
}
```

**Target pattern for wildcard**:
```rust
Mutation::Differential(..) => {
    return Err(GaError::MutationError("...".to_string()));
}
```

**`factory_with_params` function** (lines 268–278 — DELETE entirely):
```rust
pub fn factory_with_params<U>(
    mutation: Mutation,
    individual: &mut U,
    _step: Option<f64>,
    _sigma: Option<f64>,
) -> Result<(), GaError>
where
    U: LinearChromosome + ValueMutable + RealValuedMutation + 'static,
{
    mutation.mutate(individual, &mutation.clone())
}
```

**`factory_with_chromosome_length` function** (lines 313–334 — remove `_step`/`_sigma` params):
```rust
// Before signature (lines 313–319)
pub fn factory_with_chromosome_length<U>(
    mutation: Mutation,
    individual: &mut U,
    chromosome_length: Option<ChromosomeLength>,
    _step: Option<f64>,
    _sigma: Option<f64>,
) -> Result<(), GaError>

// After signature
pub fn factory_with_chromosome_length<U>(
    mutation: Mutation,
    individual: &mut U,
    chromosome_length: Option<ChromosomeLength>,
) -> Result<(), GaError>
```

**`factory_self_adaptive` function** (lines 353–367 — construction site update):
```rust
// Before (line 360–365)
let variant = Mutation::SelfAdaptiveGaussian {
    tau,
    tau_prime,
    sigma_min,
    sigma_max,
};

// After
let variant = Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
    tau,
    tau_prime,
    sigma_min,
    sigma_max,
});
```

---

### `src/engines/ga/generation.rs` — Call-site updates

**Analog:** `src/engines/island/mod.rs` (identical call pattern).

**Existing `Differential` match arm** (lines 267–274 — before state):
```rust
Mutation::Differential { f } => {
    let f_val = f.unwrap_or(0.5);
    crate::operations::mutation::differential::differential_mutation(
        &mut child_1,
        chromosomes,
        key,
        f_val,
    )?;
}
```

**Target pattern**:
```rust
Mutation::Differential(DifferentialParams { f }) => {
    let f_val = f.unwrap_or(0.5);
    // body unchanged
}
```

**Existing `factory_with_chromosome_length` call** (lines 277–283):
```rust
mutation::factory_with_chromosome_length(
    mutation_method.clone(),
    &mut child_1,
    Some(configuration.limit_configuration.chromosome_length),
    None,
    None,
)?;
```

**Target pattern** (drop last two `None, None` args):
```rust
mutation::factory_with_chromosome_length(
    mutation_method.clone(),
    &mut child_1,
    Some(configuration.limit_configuration.chromosome_length),
)?;
```

Note: Same two changes apply at lines 294–310 (second child mutation block).

---

### `src/engines/island/mod.rs` — Call-site updates (2 locations)

**Analog:** `src/engines/ga/generation.rs` (same pattern, simpler — no Differential arm).

**Existing call** (lines 592–598):
```rust
mutation::factory_with_chromosome_length(
    mutation_config.method.clone(),
    child,
    None,
    None,
    None,
)?;
```

**Target pattern** (drop last two `None, None` args):
```rust
mutation::factory_with_chromosome_length(
    mutation_config.method.clone(),
    child,
    None,
)?;
```

Note: Identical change at line ~691.

---

### `src/engines/moead/mod.rs`, `nsga2/mod.rs`, `nsga3/mod.rs` — Wildcard guard fix

**Analog:** All three files contain the identical pattern.

**Existing guard** (moead/mod.rs line 627, nsga2/mod.rs line 552, nsga3/mod.rs line 572):
```rust
if matches!(
    mutation_config.method,
    crate::operations::Mutation::Differential { .. }
) {
```

**Target pattern** (`{ .. }` → `(..)`):
```rust
if matches!(
    mutation_config.method,
    crate::operations::Mutation::Differential(..)
) {
```

---

### `src/engines/cellular/configuration.rs` and `src/engines/alps/configuration.rs` — Construction site

**Analog:** Each other (identical pattern in both files).

**Existing construction** (alps/configuration.rs line 95, cellular/configuration.rs line 112):
```rust
mutation: Mutation::Gaussian { sigma: Some(0.1) },
```

**Target pattern**:
```rust
mutation: Mutation::Gaussian(GaussianParams { sigma: Some(0.1) }),
```

---

## Shared Patterns

### Param struct derives (apply to all 8 new structs)
**Source:** `src/operations.rs` lines 253–254 (Mutation enum derive block)
```rust
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
```
Note: Add `Default` (all fields `Option<f64>` default to `None` via derive). Do NOT add `Copy` — `f64` is `Copy` but `Option<f64>` fields are fine either way; match `Mutation` enum which does not derive `Copy`.

### Serde field annotation (apply to every `Option<f64>` field in each param struct)
**Source:** `src/operations.rs` lines 272, 281, 290, 299, 332, 341, 350, 379–388
```rust
#[cfg_attr(feature = "serde", serde(default))]
pub field_name: Option<f64>,
```

### Field visibility (apply to all param struct fields)
**Source:** Anti-pattern prevention — all fields must be `pub` for external crate construction.
```rust
pub sigma: Option<f64>,  // NOT: sigma: Option<f64>,
```

### `unwrap_or(default)` dispatch pattern (existing — do not change)
**Source:** `src/operations/mutation.rs` lines 183–211
```rust
let s = scale.unwrap_or(1.0);          // Cauchy default
let a = alpha.unwrap_or(1.5);          // LevyFlight default
let f_val = f.unwrap_or(0.5);          // Differential default (generation.rs line 268)
```
Defaults remain at dispatch, not in `Default` impl.

---

## Test / Example Construction Site Pattern

All test files and examples that use struct-field construction syntax must switch to tuple construction. The compiler drives completeness — every missed site is a compile error.

**Before (all tests/examples):**
```rust
Mutation::Gaussian { sigma: None }
Mutation::Gaussian { sigma: Some(0.1) }
Mutation::Creep { step: Some(0.01) }
Mutation::SelfAdaptiveGaussian { tau: None, tau_prime: None, sigma_min: None, sigma_max: None }
Mutation::Differential { f: None }
Mutation::Cauchy { scale: None }
Mutation::LevyFlight { alpha: None }
```

**After:**
```rust
Mutation::Gaussian(GaussianParams { sigma: None })
Mutation::Gaussian(GaussianParams { sigma: Some(0.1) })
Mutation::Creep(CreepParams { step: Some(0.01) })
Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams { tau: None, tau_prime: None, sigma_min: None, sigma_max: None })
// OR shorthand:
Mutation::Gaussian(GaussianParams::default())
```

**test_variable_length.rs line 58** — unique case: `factory_with_params` call → migrate to `factory`:
```rust
// Before
factory_with_params(Mutation::PermutationInsert, &mut individual, None, None).unwrap();
// After
factory(Mutation::PermutationInsert, &mut individual).unwrap();
```

---

## Public Re-export Check

**Source:** Verify `src/lib.rs` re-export of `operations`. If it uses `pub use crate::operations::Mutation`, add the 8 new param structs:
```rust
pub use crate::operations::{
    Mutation,
    CreepParams, GaussianParams, PolynomialParams, NonUniformParams,
    DifferentialParams, CauchyParams, LevyFlightParams, SelfAdaptiveGaussianParams,
};
```
If `operations` is re-exported as a module via `pub mod operations` or `pub use crate::operations::*`, the structs are covered automatically.

---

## No Analog Found

None — all files modified in Phase 71 have direct analogs or are self-referential edits to existing files.

---

## Metadata

**Analog search scope:** `src/operations.rs`, `src/operations/mutation.rs`, `src/engines/ga/generation.rs`, `src/engines/island/mod.rs`, `src/engines/moead/mod.rs`, `src/engines/nsga2/mod.rs`, `src/engines/nsga3/mod.rs`, `src/engines/cellular/configuration.rs`, `src/engines/alps/configuration.rs`
**Files scanned:** 9 primary source files
**Pattern extraction date:** 2026-06-18
