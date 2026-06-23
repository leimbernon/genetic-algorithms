# Phase 70: Replace Operator Downcasting - Pattern Map

**Mapped:** 2026-06-18
**Files analyzed:** 5 (1 new, 4 modified)
**Analogs found:** 4 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/traits/real_valued_mutation.rs` | trait-definition | dispatch | `src/operations/mutation.rs` (ValueMutable trait, lines 186-238) | exact |
| `src/traits/mod.rs` (aka `src/traits.rs`) | module-wiring | wiring | `src/traits.rs` (existing pub mod/pub use pattern) | exact |
| `src/types/chromosomes/range.rs` | trait-implementation | dispatch | `src/operations/mutation/value.rs` (ValueMutable impl for Range<T>) | exact |
| `src/operations/mutation.rs` | factory/dispatch | mutation-dispatch | itself (current code to be modified) | self |
| `src/lib.rs` | re-export | wiring | `src/lib.rs` line 423 (existing trait re-exports) | exact |

## Pattern Assignments

### `src/traits/real_valued_mutation.rs` (trait-definition, dispatch) — NEW FILE

**Analog:** `src/operations/mutation.rs` lines 186-238 (ValueMutable trait)

**Imports pattern** (following existing trait file conventions):
```rust
use crate::error::GaError;
use crate::traits::LinearChromosome;
```

**Core trait definition pattern** (copied from ValueMutable at mutation.rs:186-238):
```rust
/// Opt-in trait for chromosomes that support real-valued mutation operators.
///
/// Default implementations return `Err(GaError::MutationError(...))`.
/// Override for chromosome types that support real-valued mutations.
pub trait RealValuedMutation: LinearChromosome {
    fn polynomial_mutation(&mut self, _eta_m: f64) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Polynomial mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }
    fn cauchy_mutation(&mut self, _scale: f64) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Cauchy mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }
    fn levy_flight_mutation(&mut self, _alpha: f64) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Lévy Flight mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }
    fn uniform_mutation(&mut self) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Uniform mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }
    fn self_adaptive_gaussian_mutation(
        &mut self,
        _tau: f64,
        _tau_prime: f64,
        _sigma_min: f64,
        _sigma_max: Option<f64>,
    ) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive (RangeChromosome<T>)."
                .to_string(),
        ))
    }
}
```

**Key pattern notes:**
- `ValueMutable` uses `log_warn!` + fallback to swap. `RealValuedMutation` uses `Err(GaError::MutationError)` — matches the current error behavior when downcasting fails (see mutation.rs lines 265-270, 311-316, 320-325, 328-333, 342-347).
- No `#[inline]` on default methods — matches ValueMutable pattern.
- No `Self: Sized` bound — avoids breaking `U: 'static` bound pattern.

---

### `src/traits.rs` (module-wiring, wiring) — MODIFY

**Analog:** existing file (lines 38-72)

**Add module declaration** (after line 50 `pub mod self_adaptive;`):
```rust
pub mod real_valued_mutation;
```

**Add re-export** (after line 60 `pub use self_adaptive::SelfAdaptive;`):
```rust
pub use real_valued_mutation::RealValuedMutation;
```

---

### `src/types/chromosomes/range.rs` (trait-implementation, dispatch) — MODIFY

**Analog:** `src/operations/mutation/value.rs` lines 62-111 (ValueMutable impl for Range<T> per concrete type)

**Import pattern** (add to existing imports at line 10):
```rust
use crate::traits::RealValuedMutation;
```
And add the operator function imports (only needed for the impl block):
```rust
use crate::operations::mutation::{cauchy, levy_flight, polynomial, self_adaptive_gaussian, uniform};
```

**Core impl pattern** (following value.rs lines 62-111 — per-concrete-type impls, but trait methods delegate to standalone functions):
```rust
impl<T: Sync + Send + Copy + Default + Debug + PartialOrd + 'static + GaussianConvertible + PolynomialConvertible>
    RealValuedMutation for Range<T>
{
    fn polynomial_mutation(&mut self, eta_m: f64) -> Result<(), GaError> {
        polynomial::polynomial_mutation(self, eta_m)
    }
    fn cauchy_mutation(&mut self, scale: f64) -> Result<(), GaError> {
        cauchy::cauchy_mutation(self, scale);
        Ok(())
    }
    fn levy_flight_mutation(&mut self, alpha: f64) -> Result<(), GaError> {
        levy_flight::levy_flight_mutation(self, alpha);
        Ok(())
    }
    fn uniform_mutation(&mut self) -> Result<(), GaError> {
        uniform::uniform_mutation(self);
        Ok(())
    }
    fn self_adaptive_gaussian_mutation(
        &mut self,
        tau: f64,
        tau_prime: f64,
        sigma_min: f64,
        sigma_max: Option<f64>,
    ) -> Result<(), GaError> {
        self_adaptive_gaussian::self_adaptive_gaussian_mutation(
            self, tau, tau_prime, sigma_min, sigma_max,
        )
    }
}
```

**Key pattern notes:**
- `cauchy_mutation`, `levy_flight_mutation`, `uniform_mutation` return `()` (not `Result`), so wrapped in `Ok(())`. Verified from source:
  - `cauchy.rs:36` — `pub fn cauchy_mutation<T>(individual: &mut RangeChromosome<T>, scale: f64)` (no Result return)
  - `levy_flight.rs:50` — `pub fn levy_flight_mutation<T>(individual: &mut RangeChromosome<T>, alpha: f64)` (no Result return)
  - `uniform.rs:29` — `pub fn uniform_mutation<T>(individual: &mut RangeChromosome<T>)` (no Result return)
- `polynomial_mutation` returns `Result<(), GaError>` — pass through directly.
- `self_adaptive_gaussian_mutation` returns `Result<(), GaError>` — pass through directly.
- The `where` clause mirrors existing ValueMutable impls but uses the `RealValuedMutation` trait bound. Note `GaussianConvertible` and `PolynomialConvertible` are from the operator modules (polynomial.rs:120, gaussian.rs:84).

---

### `src/operations/mutation.rs` (factory/dispatch, mutation-dispatch) — MODIFY

**Analog:** itself (current code at lines 240-351)

**Changes to make (4 categories):**

#### 1. Remove imports (lines 24, 27)
```rust
// REMOVE these two lines:
use crate::chromosomes::Range as RangeChromosome;
use std::any::Any;
```

#### 2. Add RealValuedMutation import (add near line 26)
```rust
use crate::traits::RealValuedMutation;
```

#### 3. Remove 5 try_* functions (lines 54-166)
Delete entirely: `try_polynomial`, `try_cauchy`, `try_levy`, `try_uniform`, `try_self_adaptive` — these are replaced by trait method calls.

#### 4. Update `mutate()` bound and match arms (lines 247, 263-348)

**Bound change** (line 247):
```rust
// Before:
U: LinearChromosome + ValueMutable + 'static,
// After:
U: LinearChromosome + ValueMutable + RealValuedMutation + 'static,
```

**Match arm replacements** (lines 263-348):
```rust
// Polynomial (was lines 263-271):
Mutation::Polynomial { eta } => {
    let eta_val = eta.unwrap_or(DEFAULT_POLYNOMIAL_ETA);
    return individual.polynomial_mutation(eta_val);
}
// Cauchy (was lines 309-317):
Mutation::Cauchy { scale } => {
    let s = scale.unwrap_or(1.0);
    return individual.cauchy_mutation(s);
}
// LevyFlight (was lines 318-326):
Mutation::LevyFlight { alpha } => {
    let a = alpha.unwrap_or(1.5);
    return individual.levy_flight_mutation(a);
}
// Uniform (was lines 327-334):
Mutation::Uniform => {
    return individual.uniform_mutation();
}
// SelfAdaptiveGaussian (was lines 335-348):
Mutation::SelfAdaptiveGaussian { tau, tau_prime, sigma_min, sigma_max } => {
    let n_hint = individual.dna().len().max(1);
    let effective_tau = tau.unwrap_or_else(|| 1.0 / (2.0 * n_hint as f64).sqrt());
    let effective_tau_prime =
        tau_prime.unwrap_or_else(|| 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt());
    let effective_sigma_min = sigma_min.unwrap_or(1e-5_f64);
    return individual.self_adaptive_gaussian_mutation(
        effective_tau, effective_tau_prime, effective_sigma_min, *sigma_max,
    );
}
```

#### 5. Update factory function bounds (lines 377, 412, 458, 490)
Add `RealValuedMutation` to the `where` clause of:
- `factory()` (line 377)
- `factory_with_params()` (line 412)
- `factory_with_chromosome_length()` (line 458)
- `factory_self_adaptive()` (line 490)

**DO NOT** add `RealValuedMutation` to `factory_non_value()` (line 525) — this is intentionally for non-ValueMutable types.

---

### `src/lib.rs` (re-export, wiring) — MODIFY

**Analog:** existing re-export at line 423

**Add re-export** (after line 423):
```rust
pub use traits::{LinearChromosome, OperatorCompat, RealGene, RealValuedMutation, Strategy, VectorFitness};
```

---

## Shared Patterns

### Error Handling
**Source:** `src/error.rs` lines 46-89
**Apply to:** trait default implementations, factory match arms
```rust
GaError::MutationError("error message".to_string())
```
All error messages match the existing ones in mutation.rs lines 265-270, 311-316, 320-325, 328-333, 342-347.

### Trait Pattern (Default Impl with Fallback)
**Source:** `src/operations/mutation.rs` lines 186-238 (ValueMutable)
**Apply to:** new RealValuedMutation trait
```rust
pub trait RealValuedMutation: LinearChromosome {
    fn method_name(&mut self, _param: Type) -> Result<(), GaError> {
        Err(GaError::MutationError("clear actionable message".to_string()))
    }
}
```

### Trait Impl for Range<T>
**Source:** `src/operations/mutation/value.rs` lines 62-111
**Apply to:** RealValuedMutation impl for Range<T>
Pattern: delegate to standalone operator function; wrap `()` returns in `Ok(())`.

### Re-export Wiring
**Source:** `src/traits.rs` lines 38-72, `src/lib.rs` line 423
**Apply to:** new trait must be registered in traits.rs (pub mod + pub use) and re-exported from lib.rs.

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| (none) | — | — | All files have close analogs |

## Metadata

**Analog search scope:** src/operations/mutation.rs, src/traits/, src/types/chromosomes/, src/operations/mutation/value.rs, src/operations/mutation/polynomial.rs, src/operations/mutation/cauchy.rs, src/operations/mutation/levy_flight.rs, src/operations/mutation/uniform.rs, src/operations/mutation/self_adaptive_gaussian.rs, src/lib.rs, src/error.rs
**Files scanned:** 12
**Pattern extraction date:** 2026-06-18
