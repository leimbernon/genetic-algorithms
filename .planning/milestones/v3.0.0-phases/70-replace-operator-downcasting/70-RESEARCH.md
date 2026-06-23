# Phase 70: Replace Operator Downcasting - Research

**Researched:** 2026-06-18
**Domain:** Rust trait design, type-level dispatch, genetic algorithm operator architecture
**Confidence:** HIGH

## Summary

Phase 70 eliminates runtime `as_any().downcast_mut()` calls in `src/operations/mutation.rs` by introducing a `RealValuedMutation` trait that chromosomes implement directly. Currently, 5 `try_*` functions (lines 54–166) each use a `try_type!` macro that attempts downcasting to `RangeChromosome<f64>`, `RangeChromosome<f32>`, `RangeChromosome<i32>`, and `RangeChromosome<i64>` in sequence — 11 total `downcast_mut` calls. The new trait routes operators to the correct chromosome type via trait method dispatch instead.

The refactoring is scoped to mutation operators only. The `std::any::Any` import (line 27) will be removed from `mutation.rs`. All existing tests (267 pass, 29 ignored) must continue to pass identically. The crossover module (`crossover.rs`) has similar downcasting but is explicitly out of scope for this phase.

**Primary recommendation:** Create `RealValuedMutation` trait in `src/traits/real_valued_mutation.rs` with 5 optional methods, implement it for `Range<T>`, and replace the 5 `try_*` function calls in `Mutation::mutate()` with trait method calls.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Create a new trait `RealValuedMutation` with optional methods: `polynomial_mutation()`, `cauchy_mutation()`, `levy_flight_mutation()`, `uniform_mutation()`, `self_adaptive_gaussian_mutation()`. Each method takes the operator-specific parameters (eta, scale, alpha, tau/tau_prime/sigma_min/sigma_max) and returns `Result<(), GaError>`.
- **D-02:** Default trait implementations return `Err(GaError::MutationError(...))` with a clear message stating the chromosome doesn't support the operator. This matches the current error behavior when downcasting fails.
- **D-03:** The trait is separate from `ValueMutable` — it groups only the 5 operators that currently require `Range<T>` downcasting. `ValueMutable` continues to handle `value_mutate`, `bit_flip_mutate`, `creep_mutate`, `gaussian_mutate` as before.
- **D-04:** Phase 70 keeps all existing parameter signatures unchanged. The `try_*` functions become trait method calls but the factory match arms in `Mutation::mutate()` keep their current parameter extraction logic (eta, scale, alpha, tau, etc.). Phase 71 will clean up signatures with per-operator parameter structs.
- **D-05:** When a chromosome type doesn't support an operator, return `Err(GaError::MutationError(...))` with the same error messages currently used. This is the existing behavior and downstream code already handles it.
- **D-06:** `RealValuedMutation` goes in `src/traits/` (like `ValueMutable`, `LinearChromosome`, etc.) and is re-exported from `src/lib.rs`. Implementations go in `src/types/chromosomes/range.rs` (where `ValueMutable` is already implemented for `Range<T>`).

### the agent's Discretion
- Exact error message strings for default trait method implementations — use clear, actionable messages matching the current style.
- Whether to use `#[inline]` on trait default methods — follow existing trait patterns in the codebase.

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| (architecture refactor — no REQ IDs) | Eliminate all `downcast`/`as_any` calls in `src/operations/mutation.rs` | 5 try_* functions (lines 54–166) with 11 downcast calls identified; trait dispatch replaces all of them |
| (architecture refactor) | All existing mutation operators continue to work identically | All 267 tests pass currently; trait default-impl pattern matches `ValueMutable` precedent |
| (architecture refactor) | `cargo check --target wasm32-unknown-unknown` passes | WASM target confirmed available and compiling (verified via toolchain probe) |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Polynomial mutation dispatch | Trait (RealValuedMutation) | Factory match arm | Trait method called from `Mutation::mutate()` — replaces `try_polynomial` |
| Cauchy mutation dispatch | Trait (RealValuedMutation) | Factory match arm | Trait method called from `Mutation::mutate()` — replaces `try_cauchy` |
| Levy Flight mutation dispatch | Trait (RealValuedMutation) | Factory match arm | Trait method called from `Mutation::mutate()` — replaces `try_levy` |
| Uniform mutation dispatch | Trait (RealValuedMutation) | Factory match arm | Trait method called from `Mutation::mutate()` — replaces `try_uniform` |
| SelfAdaptive Gaussian dispatch | Trait (RealValuedMutation) | Factory match arm | Trait method called from `Mutation::mutate()` — replaces `try_self_adaptive` |
| Error fallback for unsupported types | Trait default impl | — | Default methods return `Err(GaError::MutationError(...))` — matches current behavior |
| Downcast removal | mutation.rs cleanup | — | Remove `use std::any::Any`, `try_type!` macro, 5 `try_*` functions |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (no new dependencies) | — | — | This phase is a pure internal refactor; no new crates needed |

### Existing Dependencies Used
| Library | Version | Purpose | Relevance |
|---------|---------|---------|-----------|
| `rand` | 0.9.2 | RNG for mutation operators | Used by all 5 mutation implementations (polynomial, cauchy, levy_flight, uniform, self_adaptive) |
| `log` | 0.4.22 | Logging (feature-gated) | Used by `log_debug!` in mutation implementations |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `RealValuedMutation` trait with default impls | Enum-based dispatch (match on chromosome type) | Enum dispatch doesn't scale to user-defined types; trait dispatch is extensible |
| Separate traits per operator | Single `RealValuedMutation` trait (D-01) | Single trait is simpler; all 5 operators share the same chromosome support set |

**Installation:** No new packages to install.

## Package Legitimacy Audit

> No external packages are installed in this phase. This section is included for completeness.

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   Mutation::mutate()                         │
│              (src/operations/mutation.rs)                    │
│                                                              │
│  match mutation {                                            │
│    Polynomial { eta } ──────────► individual.polynomial_mutation(eta)   │
│    Cauchy { scale }   ──────────► individual.cauchy_mutation(scale)     │
│    LevyFlight { alpha } ─────────► individual.levy_flight_mutation(alpha)│
│    Uniform           ──────────► individual.uniform_mutation()          │
│    SelfAdaptiveGaussian { ... } ► individual.self_adaptive_gaussian_(..)│
│    Value / BitFlip / etc. ──────► (existing ValueMutable path — unchanged)│
│    Swap / Inversion / etc. ─────► (direct function call — unchanged)    │
│  }                                                         │
│                                                              │
│  where U: LinearChromosome + ValueMutable + RealValuedMutation + 'static  │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────────────────┐
│          RealValuedMutation trait (src/traits/real_valued_mutation.rs)  │
│                                                              │
│  fn polynomial_mutation(&mut self, eta_m: f64) -> Result<(), GaError>  │
│  fn cauchy_mutation(&mut self, scale: f64) -> Result<(), GaError>      │
│  fn levy_flight_mutation(&mut self, alpha: f64) -> Result<(), GaError>  │
│  fn uniform_mutation(&mut self) -> Result<(), GaError>                 │
│  fn self_adaptive_gaussian_mutation(                                   │
│    &mut self, tau, tau_prime, sigma_min, sigma_max                     │
│  ) -> Result<(), GaError>                                              │
│                                                              │
│  Default impls: return Err(GaError::MutationError("..."))    │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────────────────┐
│    Range<T> implements RealValuedMutation                    │
│    (src/types/chromosomes/range.rs)                          │
│                                                              │
│  Overrides all 5 methods to delegate to:                     │
│    polynomial::polynomial_mutation(self, eta_m)              │
│    cauchy::cauchy_mutation(self, scale)                      │
│    levy_flight::levy_flight_mutation(self, alpha)            │
│    uniform::uniform_mutation(self)                           │
│    self_adaptive_gaussian::self_adaptive_gaussian_mutation(self, ...) │
└──────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src/
├── traits/
│   ├── real_valued_mutation.rs       # NEW: RealValuedMutation trait
│   └── mod.rs (via traits.rs)        # Add `pub mod real_valued_mutation; pub use ...`
├── types/chromosomes/
│   └── range.rs                      # MODIFY: impl RealValuedMutation for Range<T>
├── operations/
│   └── mutation.rs                   # MODIFY: Remove try_* fns, downcast macro, Any import
└── lib.rs                            # MODIFY: Add re-export of RealValuedMutation
```

### Pattern 1: Trait with Default Error Implementations
**What:** Define a trait where each method has a default implementation that returns `Err(GaError::MutationError(...))`. Concrete types override only the methods they support.
**When to use:** When multiple chromosome types share a common set of operators, but only some types support some operators.
**Example (from existing codebase — `ValueMutable` pattern):**
```rust
// Source: src/operations/mutation.rs lines 186-238 (existing ValueMutable trait)
pub trait ValueMutable: LinearChromosome {
    fn value_mutate(&mut self) {
        crate::log_warn!("value_mutate() not overridden; falling back to swap mutation.");
        swap(self);
    }
    fn bit_flip_mutate(&mut self) {
        crate::log_warn!("bit_flip_mutate() not overridden; falling back to swap mutation.");
        swap(self);
    }
    // ... more methods with default impls
}
```

### Pattern 2: Trait Method Calling Into Existing Operator Functions
**What:** The trait method on `Range<T>` delegates directly to the existing standalone operator function (e.g., `polynomial::polynomial_mutation(self, eta_m)`). No logic duplication.
**When to use:** When refactoring from downcasting to trait dispatch — the operator implementations already exist and work.
**Example (planned pattern):**
```rust
// RealValuedMutation impl for Range<T>:
fn polynomial_mutation(&mut self, eta_m: f64) -> Result<(), GaError> {
    polynomial::polynomial_mutation(self, eta_m)
}
```

### Anti-Patterns to Avoid
- **Don't add `Self: Sized` bounds on default trait methods** — this prevents `dyn RealValuedMutation` use and conflicts with the existing `U: 'static` bound pattern.
- **Don't use `#[inline]` on default trait methods** — the existing `ValueMutable` trait doesn't use it, and these methods are not in the hot path (they're called once per mutation event).
- **Don't create blanket implementations** — `Range<T>` is the only concrete type that implements all 5 methods. A blanket impl would conflict with the `OperatorCompat` pattern (see `src/traits/operator_compat.rs` lines 34-41 for why).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Type-level dispatch for operators | Runtime downcasting via `Any` | Trait with default impls | Compile-time safety, no `unsafe`, extensible to user types |
| Error messages for unsupported ops | Custom error enum per operator | `GaError::MutationError(String)` | Existing pattern used everywhere in the crate |
| f64 conversion for numeric types | Manual `as f64` / `from_f64` | `GaussianConvertible` / `PolynomialConvertible` traits | Already exist in `gaussian.rs` and `polynomial.rs` |

**Key insight:** The `try_type!` macro pattern (`macro_rules! try_type { ($t:ty) => { ... } }`) exists solely to work around Rust's inability to genericize over concrete `Range<T>` types at the call site. The trait dispatch approach eliminates this entirely — `Range<T>` implements the trait for all `T: GaussianConvertible + PolynomialConvertible`, so the generic parameter is resolved at compile time.

## Common Pitfalls

### Pitfall 1: Forgetting to Add `RealValuedMutation` to the `where` Clause
**What goes wrong:** If the `MutateOperator::mutate()` signature doesn't include `RealValuedMutation` as a bound, the trait methods won't be callable on `individual`.
**Why it happens:** The current bound is `U: LinearChromosome + ValueMutable + 'static`. Adding `+ RealValuedMutation` is easy to miss.
**How to avoid:** Update the bound on `mutate()` (line 247), `factory()` (line 377), `factory_with_params()` (line 413), `factory_with_chromosome_length()` (line 458), and `factory_self_adaptive()` (line 490).
**Warning signs:** Compiler errors like "no method named `polynomial_mutation` found for type parameter `U`".

### Pitfall 2: Breaking the `factory_non_value` Function
**What goes wrong:** `factory_non_value()` (line 525) doesn't use `try_*` functions — it returns errors for value-dependent operators. Adding `RealValuedMutation` bound to its signature would break non-Range chromosome types.
**Why it happens:** `factory_non_value` is specifically for types that don't implement `ValueMutable`.
**How to avoid:** Do NOT add `RealValuedMutation` bound to `factory_non_value`. It has its own error messages that don't depend on downcasting.
**Warning signs:** Tests like `test_factory_non_value_value_returns_error` fail.

### Pitfall 3: Not Removing the `RangeChromosome` Import
**What goes wrong:** After removing the `try_*` functions, the `use crate::chromosomes::Range as RangeChromosome;` import (line 24) becomes unused, causing a compiler warning.
**Why it happens:** The import was only needed for the `downcast_mut::<RangeChromosome<$t>>()` calls.
**How to avoid:** Remove the import when removing the `try_*` functions. Also remove `use std::any::Any;` (line 27).
**Warning signs:** `cargo clippy` warns about unused imports.

### Pitfall 4: SelfAdaptive Gaussian Has Different Error Semantics
**What goes wrong:** `try_self_adaptive` currently returns `None` when the type doesn't match (line 165), and the error is generated by the `unwrap_or_else` in `mutate()` (line 342). The trait default should match this behavior.
**Why it happens:** The other 4 operators use `try_*.unwrap_or_else(|| Err(...))` where the `None` case produces the error. The `try_self_adaptive` also returns `None` for type mismatch.
**How to avoid:** The trait default for `self_adaptive_gaussian_mutation()` should return `Err(GaError::MutationError("SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive (RangeChromosome<T>)."))` — matching the existing error message at line 344.
**Warning signs:** Test `self_adaptive_gaussian_returns_error_for_non_self_adaptive` fails.

### Pitfall 5: The `try_polynomial` Function Returns `Result` Inside `Option`
**What goes wrong:** Unlike the other 4 `try_*` functions which return `Option<Result<(), GaError>>` where `None` means "type not supported", `try_polynomial` also returns `Option<Result<(), GaError>>` but the inner `Result` carries the `Err` from `polynomial_mutation` (e.g., negative eta). The trait default must preserve this two-level error semantics.
**Why it happens:** Polynomial mutation can fail for valid reasons (negative eta) in addition to type mismatch.
**How to avoid:** The trait's `polynomial_mutation()` default should return `Err(GaError::MutationError("Polynomial mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."))` — which is the message currently used when `try_polynomial` returns `None`. The inner `Result` from the actual implementation propagates through naturally.
**Warning signs:** Test `polynomial_mutation_negative_eta_returns_error` fails with a different error message.

## Code Examples

### RealValuedMutation Trait Definition
```rust
// NEW FILE: src/traits/real_valued_mutation.rs
use crate::error::GaError;
use crate::traits::LinearChromosome;

/// Opt-in trait for chromosomes that support real-valued mutation operators.
///
/// This trait groups the 5 mutation operators that require numeric gene values:
/// polynomial, Cauchy, Lévy Flight, uniform reset, and self-adaptive Gaussian.
///
/// Only [`Range<T>`](crate::chromosomes::Range) implements all 5 methods.
/// Other chromosome types inherit the default implementations that return
/// `Err(GaError::MutationError(...))`.
///
/// This trait replaces the previous runtime downcasting approach (`as_any().downcast_mut()`)
/// with compile-time trait dispatch.
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

### Range<T> Implementation
```rust
// ADD to src/types/chromosomes/range.rs (after existing SelfAdaptive impl)
impl<T: Sync + Send + Copy + Default + Debug + PartialOrd + 'static + GaussianConvertible + PolynomialConvertible>
    crate::traits::RealValuedMutation for Range<T>
{
    fn polynomial_mutation(&mut self, eta_m: f64) -> Result<(), GaError> {
        crate::operations::mutation::polynomial::polynomial_mutation(self, eta_m)
    }
    fn cauchy_mutation(&mut self, scale: f64) -> Result<(), GaError> {
        crate::operations::mutation::cauchy::cauchy_mutation(self, scale);
        Ok(())
    }
    fn levy_flight_mutation(&mut self, alpha: f64) -> Result<(), GaError> {
        crate::operations::mutation::levy_flight::levy_flight_mutation(self, alpha);
        Ok(())
    }
    fn uniform_mutation(&mut self) -> Result<(), GaError> {
        crate::operations::mutation::uniform::uniform_mutation(self);
        Ok(())
    }
    fn self_adaptive_gaussian_mutation(
        &mut self,
        tau: f64,
        tau_prime: f64,
        sigma_min: f64,
        sigma_max: Option<f64>,
    ) -> Result<(), GaError> {
        crate::operations::mutation::self_adaptive_gaussian::self_adaptive_gaussian_mutation(
            self, tau, tau_prime, sigma_min, sigma_max,
        )
    }
}
```

### Mutation::mutate() Updated Match Arms
```rust
// MODIFY in src/operations/mutation.rs — replace try_* calls with trait method calls
Mutation::Polynomial { eta } => {
    let eta_val = eta.unwrap_or(DEFAULT_POLYNOMIAL_ETA);
    return individual.polynomial_mutation(eta_val);
}
Mutation::Cauchy { scale } => {
    let s = scale.unwrap_or(1.0);
    return individual.cauchy_mutation(s);
}
Mutation::LevyFlight { alpha } => {
    let a = alpha.unwrap_or(1.5);
    return individual.levy_flight_mutation(a);
}
Mutation::Uniform => {
    return individual.uniform_mutation();
}
Mutation::SelfAdaptiveGaussian { tau, tau_prime, sigma_min, sigma_max } => {
    let n_hint = individual.dna().len().max(1);
    let effective_tau = tau.unwrap_or_else(|| 1.0 / (2.0 * n_hint as f64).sqrt());
    let effective_tau_prime = tau_prime.unwrap_or_else(|| 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt());
    let effective_sigma_min = sigma_min.unwrap_or(1e-5_f64);
    return individual.self_adaptive_gaussian_mutation(
        effective_tau, effective_tau_prime, effective_sigma_min, *sigma_max,
    );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Runtime `as_any().downcast_mut()` via `try_type!` macro | Trait method dispatch via `RealValuedMutation` | Phase 70 (this phase) | Compile-time safety, no `Any` import, extensible |

**Deprecated/outdated:**
- `try_polynomial`, `try_cauchy`, `try_levy`, `try_uniform`, `try_self_adaptive` functions: replaced by trait methods
- `try_type!` macro: no longer needed after trait dispatch
- `use std::any::Any;` import in `mutation.rs`: removed after downcast calls eliminated
- `use crate::chromosomes::Range as RangeChromosome;` import in `mutation.rs`: removed after downcast calls eliminated (the concrete type is only referenced in the trait impl, not in the dispatch code)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Phase 69 (dependency) does not exist yet — Phase 70 has no blocking upstream phases | Context review | LOW — Phase 70 is independent of Phase 69; no code coupling found |
| A2 | The `MutationOperator` trait bound change (adding `RealValuedMutation`) will not break downstream user code | Pitfall 1 | LOW — `MutationOperator` is internal; users call `factory()` which is generic |
| A3 | The `factory_non_value` function should NOT get the `RealValuedMutation` bound | Pitfall 2 | LOW — verified by reading the function; it's designed for non-ValueMutable types |
| A4 | `cauchy_mutation`, `levy_flight_mutation`, and `uniform_mutation` return `()` (not `Result`), so the trait impl wraps them in `Ok(())` | Code example | LOW — verified by reading the source files; all 3 return `()` |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **Should `RealValuedMutation` be re-exported from `src/lib.rs`?**
   - What we know: D-06 says yes. Existing pattern in `lib.rs` line 423 re-exports `LinearChromosome`, `RealValued`, `SelfAdaptive`.
   - What's unclear: Nothing — D-06 is explicit.
   - Recommendation: Re-export as `pub use traits::RealValuedMutation;` in `lib.rs`.

2. **Should the trait bound on `MutationOperator::mutate()` change?**
   - What we know: Current bound is `U: LinearChromosome + ValueMutable + 'static` (line 247).
   - What's unclear: Whether adding `+ RealValuedMutation` to this bound would break downstream `impl MutationOperator` users.
   - Recommendation: Yes, add it — all callers that pass `Range<T>` already satisfy it, and the trait default impls mean non-Range types work too.

3. **Should `#[inline]` be used on the `Range<T>` trait impl methods?**
   - What we know: The standalone operator functions (`polynomial_mutation`, etc.) are not `#[inline]`. The `ValueMutable` default methods don't use `#[inline]`.
   - What's unclear: Whether LLVM will inline through the trait dispatch.
   - Recommendation: Do not add `#[inline]` — follow existing patterns. The operator functions are not in the critical hot path (mutation is called once per individual per generation, not per gene).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Compilation | ✓ | 1.94.1 | — |
| wasm32-unknown-unknown target | Success criterion #4 | ✓ | (verified: `cargo check --target wasm32-unknown-unknown` passes) | — |
| cargo test | Validation | ✓ | 1.94.1 | — |

**Missing dependencies with no fallback:** None.

## Validation Architecture

> No `nyquist_validation` key in `.planning/config.json` — treat as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Built-in `cargo test` (rustc test harness) |
| Config file | none — standard Cargo test layout |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` (includes doc-tests) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| (no downcast calls) | All downcast calls removed from mutation.rs | compilation check | `cargo check` | ✅ |
| (all operators work) | Polynomial, Cauchy, LevyFlight, Uniform, SelfAdaptiveGaussian continue working | integration | `cargo test` | ✅ (267 tests) |
| (wasm32 compiles) | `cargo check --target wasm32-unknown-unknown` passes | compilation | `cargo check --target wasm32-unknown-unknown` | ✅ |

### Sampling Rate
- **Per task commit:** `cargo test` (267 tests, ~27s)
- **Per wave merge:** `cargo test` + `cargo clippy` + `cargo fmt --check`
- **Phase gate:** Full suite green + `cargo check --target wasm32-unknown-unknown`

### Wave 0 Gaps
- None — existing test infrastructure covers all phase requirements. The 12 mutation test files already test the operators via the factory path, which exercises the exact code being refactored.

## Security Domain

> No `security_enforcement` key in config — defaulting to enabled. This phase is a pure internal refactor with no external input handling, no authentication, no cryptography, and no data persistence. ASVS categories do not apply.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no | No new user-facing API |
| V6 Cryptography | no | — |

### Known Threat Patterns for Rust trait refactoring

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| (none applicable) | — | — |

## Sources

### Primary (HIGH confidence)
- `src/operations/mutation.rs` (lines 1-726) — full file read, all 5 `try_*` functions identified
- `src/traits/self_adaptive.rs` (lines 1-110) — `SelfAdaptive` trait pattern reference
- `src/operations/mutation.rs` (lines 186-238) — `ValueMutable` trait pattern reference
- `src/types/chromosomes/range.rs` (lines 1-218) — `Range<T>` implementation target
- `src/traits/operators.rs` (lines 125-143) — `MutationOperator` trait bound reference
- `src/traits/real_valued.rs` (lines 1-65) — existing `RealValued` marker trait (naming reference)

### Secondary (MEDIUM confidence)
- `tests/operations/test_mutation_polynomial.rs` — 8 tests exercising polynomial mutation
- `tests/operations/test_mutation_cauchy_levy_uniform.rs` — 16 tests across Cauchy/Levy/Uniform
- `tests/operations/test_mutation_self_adaptive.rs` — 7 tests for SelfAdaptive Gaussian
- `tests/operations/test_mutation.rs` — 22 tests for base mutation operators + factory paths

### Tertiary (LOW confidence)
- None — all findings verified via direct code reading

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — no new dependencies; pure refactor
- Architecture: HIGH — pattern directly follows existing `ValueMutable` trait in the same codebase
- Pitfalls: HIGH — all pitfalls identified by reading the actual source code and test files

**Research date:** 2026-06-18
**Valid until:** 2026-07-18 (stable — internal refactor, no external dependencies)
