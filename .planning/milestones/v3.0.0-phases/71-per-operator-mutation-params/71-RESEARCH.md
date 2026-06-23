# Phase 71: Per-Operator Mutation Parameters - Research

**Researched:** 2026-06-18
**Domain:** Rust enum refactoring — inline struct variant fields to named parameter structs
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Param structs use `Option<f64>` fields matching the current inline field pattern. `None` means "use the operator's documented default." Defaults remain at dispatch, not at construction.
- **D-02:** All param structs live in `src/operations.rs` alongside the `Mutation` enum — no new module.
- **D-03:** Parameterized variants switch to tuple form: `Mutation::Gaussian(GaussianParams)`, etc. Zero-param variants stay as unit variants.
- **D-04:** Only the 8 variants with existing inline fields get structs: `Creep`, `Gaussian`, `Polynomial`, `NonUniform`, `Differential`, `Cauchy`, `LevyFlight`, `SelfAdaptiveGaussian`.
- **D-05:** `factory_with_params(mutation, individual, _step, _sigma)` is removed entirely. Callers use `factory(mutation, individual)` directly. Intentional v3.0.0 breaking change.
- **D-06:** `factory_with_chromosome_length` is kept but simplified: `_step: Option<f64>` and `_sigma: Option<f64>` args removed. The 4 engine call sites drop the trailing `None, None` args.
- **D-07:** `ValueMutable` trait method signatures and all `RealValuedMutation` trait methods remain unchanged — raw `f64` params, not param structs.

### Claude's Discretion

- Exact derived trait impls for param structs (`#[derive(Debug, Clone, PartialEq)]` — follow `Mutation` enum's existing derive list).
- Whether to implement `Default` on param structs (returning `None` for all fields) — reasonable ergonomic addition.
- Serde `#[cfg_attr(feature = "serde", serde(default))]` annotations on struct fields — follow the pattern on the existing inline variant fields.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

## Summary

Phase 71 converts the `Mutation` enum's 8 inline-field struct variants into tuple variants that carry named parameter structs (e.g., `Mutation::Gaussian { sigma: Option<f64> }` becomes `Mutation::Gaussian(GaussianParams)`). This is a pure mechanical refactor: behavior is unchanged, all existing defaults remain, and the breaking changes are intentional under the v3.0.0 milestone.

The scope is precisely contained in two primary files (`src/operations.rs` and `src/operations/mutation.rs`) plus mechanical update cascades across engine call sites, tests, examples, and doc comments. No new logic is introduced. The `factory_with_params` function is removed; `factory_with_chromosome_length` drops its trailing `_step`/`_sigma` args.

The phase requires coordinated changes across approximately 8 files in `src/` and approximately 15 test/example files due to widespread use of the struct-field construction syntax (`Mutation::Gaussian { sigma: None }`), which must all become tuple construction syntax (`Mutation::Gaussian(GaussianParams { sigma: None })`).

**Primary recommendation:** Apply changes in two passes — (1) define structs and reshape enum in `src/operations.rs`, then (2) mechanically update all match arms, construction sites, and function signatures. Compile-error feedback drives completeness.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Param struct definitions | `src/operations.rs` | — | Enum and its data types co-locate |
| Dispatch / match arms | `src/operations/mutation.rs` | — | Factory owns dispatch logic |
| Engine call-site updates | `src/engines/ga/generation.rs`, `src/engines/island/mod.rs` | — | These own the 4 call sites being simplified |
| Test updates | `tests/operations/`, `tests/observe/test_serde.rs` | — | Tests exercise construction syntax |
| Example updates | `examples/*.rs` | — | Examples construct parameterized variants |

---

## Standard Stack

No external dependencies. This phase is pure Rust refactoring within the existing crate.

**Rust features in use:**

| Feature | Purpose | Notes |
|---------|---------|-------|
| Named tuple variant structs | Carry per-operator params | Standard Rust pattern |
| `#[derive(Debug, Clone, PartialEq)]` | Required for `Mutation` to keep its derives | Must mirror parent enum |
| `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` | Serde round-trip | Must be applied to each param struct, matching existing `Mutation` annotation |
| `#[cfg_attr(feature = "serde", serde(default))]` | Serde field defaults | Must appear on each `Option<f64>` field, matching existing inline field pattern |

---

## Architecture Patterns

### System Architecture Diagram

```
src/operations.rs
  Mutation enum (struct variants)
        │
        ▼ Phase 71 reshape
  Mutation enum (tuple variants)
  + CreepParams / GaussianParams / PolynomialParams /
    NonUniformParams / DifferentialParams / CauchyParams /
    LevyFlightParams / SelfAdaptiveGaussianParams
        │
        ▼ destructuring propagates to
  src/operations/mutation.rs   (MutationOperator::mutate match arms)
  src/engines/ga/generation.rs (Differential match arm, factory_with_chromosome_length calls)
  src/engines/island/mod.rs    (same 2 patterns × 2 call sites)
  src/engines/moead/mod.rs     (Differential { .. } guard — wildcard, survives reshape)
  src/engines/nsga2/mod.rs     (same wildcard guard)
  src/engines/nsga3/mod.rs     (same wildcard guard)
  src/engines/cellular/configuration.rs (Gaussian construction default)
  src/engines/alps/configuration.rs     (Gaussian construction default)
        │
        ▼ same propagation to
  tests/ (construction syntax at every factory() call)
  examples/ (construction syntax)
```

### Recommended Project Structure

No structural changes. All new types stay in `src/operations.rs` per D-02.

### Pattern 1: Param Struct Definition (with derives + serde)

**What:** Define a named struct for each of the 8 parameterized variants, then convert the variant to tuple form.

**When to use:** Exactly the 8 variants listed in D-04.

```rust
// Source: project CONTEXT.md / CLAUDE.md existing pattern
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaussianParams {
    /// Standard deviation of the Gaussian noise. Default: `0.1`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma: Option<f64>,
}

// SelfAdaptiveGaussian has 4 fields — preserve all
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfAdaptiveGaussianParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub tau: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub tau_prime: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma_min: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma_max: Option<f64>,
}
```

### Pattern 2: Variant Reshape

**What:** Convert inline struct variant to tuple variant.

```rust
// Before
Gaussian {
    #[cfg_attr(feature = "serde", serde(default))]
    sigma: Option<f64>,
},

// After
Gaussian(GaussianParams),
```

### Pattern 3: Match Arm Destructuring

**What:** Update every match arm that binds the old struct fields.

```rust
// Before (in MutationOperator::mutate)
Mutation::Gaussian { sigma } => {
    let s = sigma.unwrap_or(0.1);
    individual.gaussian_mutate(s);
}

// After
Mutation::Gaussian(GaussianParams { sigma }) => {
    let s = sigma.unwrap_or(0.1);
    individual.gaussian_mutate(s);
}

// Wildcard arms (Differential { .. } guard in moead/nsga2/nsga3) survive unchanged
// because { .. } works for tuple struct fields too:
Mutation::Differential { .. }  →  Mutation::Differential(..)
// NOTE: { .. } on a tuple variant is NOT valid Rust. Must change to (..)
```

### Pattern 4: Construction Site Updates

**What:** Every place that constructs a parameterized variant must switch to tuple-struct syntax.

```rust
// Before
Mutation::Gaussian { sigma: Some(0.1) }
Mutation::Gaussian { sigma: None }

// After
Mutation::Gaussian(GaussianParams { sigma: Some(0.1) })
Mutation::Gaussian(GaussianParams { sigma: None })
// OR with Default impl:
Mutation::Gaussian(GaussianParams::default())
```

### Pattern 5: factory_with_chromosome_length Simplification (D-06)

**What:** Remove `_step` and `_sigma` args from the public function and all 4 call sites.

```rust
// Before signature
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

**Call sites to update (4 total):**

- `src/engines/ga/generation.rs` lines ~278 and ~305: `factory_with_chromosome_length(m, ind, Some(cl), None, None)` → `factory_with_chromosome_length(m, ind, Some(cl))`
- `src/engines/island/mod.rs` lines ~592 and ~691: `factory_with_chromosome_length(m, child, None, None, None)` → `factory_with_chromosome_length(m, child, None)`

### Pattern 6: factory_with_params Removal (D-05)

**What:** Delete the function entirely.

**Impact scan:**
- `src/operations/mutation.rs` — the definition itself (delete)
- `tests/test_variable_length.rs:58` — one test calls `factory_with_params(Mutation::PermutationInsert, ...)` → migrate to `factory(Mutation::PermutationInsert, ...)`
- Doc comment in `src/operations.rs:329` references `factory_with_params` — update to `factory`

### Pattern 7: factory_self_adaptive Update

**What:** `factory_self_adaptive` constructs `Mutation::SelfAdaptiveGaussian { tau, tau_prime, sigma_min, sigma_max }` inline. Must become `Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams { tau, tau_prime, sigma_min, sigma_max })`.

### Pattern 8: Wildcard Guard Arms

**What:** Several engine files match `Mutation::Differential { .. }` or `Mutation::SelfAdaptiveGaussian { .. }` as a guard/rejection pattern. After the refactor, `{ .. }` on a tuple variant is invalid Rust syntax — must become `(..)`.

```rust
// Before (moead/mod.rs, nsga2/mod.rs, nsga3/mod.rs)
crate::operations::Mutation::Differential { .. }

// After
crate::operations::Mutation::Differential(..)
```

### Anti-Patterns to Avoid

- **Forgetting serde derives on param structs:** The `Mutation` enum has `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` — each param struct needs the same annotation, or serde compilation fails.
- **Using `{ .. }` wildcard on tuple variants:** `Mutation::Differential { .. }` is invalid after the reshape. Use `Mutation::Differential(..)`.
- **Partial field exposure:** All param struct fields should be `pub` so users constructing `GaussianParams { sigma: Some(0.1) }` outside the crate can access them.
- **Missing Default impl:** D-01 says defaults stay at dispatch, not construction — but implementing `Default` on param structs (all fields = `None`) is ergonomic and consistent; leaving it out means `GaussianParams::default()` is unavailable as a shorthand.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Serde for param structs | Custom `Serialize`/`Deserialize` impls | `#[derive]` + `serde(default)` on each field | Existing pattern is already correct and tested |
| Migration of construction syntax | Manual grep-and-edit | Let compiler errors guide completeness | Compiler finds every missed call site |
| Default values | New constant lookup table | `unwrap_or(default_value)` at the dispatch match arm (already there) | Default logic already exists in `MutationOperator::mutate` |

---

## Complete Change Inventory

This is a mechanical refactor. The full list of files that need changes:

### Primary (behavior changes)

| File | Change |
|------|--------|
| `src/operations.rs` | Add 8 param structs before `Mutation` enum; reshape 8 variants to tuple form; update doc examples |
| `src/operations/mutation.rs` | Update `MutationOperator::mutate` match arms (destructure via tuple); remove `factory_with_params`; simplify `factory_with_chromosome_length` signature; update `factory_self_adaptive`; update `factory_non_value` match arms |

### Engine call sites

| File | Change |
|------|--------|
| `src/engines/ga/generation.rs` | 2 calls: drop `None, None` from `factory_with_chromosome_length`; update `Differential { f }` arm to `Differential(DifferentialParams { f })` |
| `src/engines/island/mod.rs` | 2 calls: drop `None, None` from `factory_with_chromosome_length` |
| `src/engines/moead/mod.rs` | `Differential { .. }` → `Differential(..)` (2 occurrences — one doc, one match guard) |
| `src/engines/nsga2/mod.rs` | `Differential { .. }` → `Differential(..)` |
| `src/engines/nsga3/mod.rs` | `Differential { .. }` → `Differential(..)` |
| `src/engines/cellular/configuration.rs` | `Mutation::Gaussian { sigma: Some(0.1) }` → `Mutation::Gaussian(GaussianParams { sigma: Some(0.1) })` |
| `src/engines/alps/configuration.rs` | Same as above |

### Trait/doc files (doc example strings only)

| File | Change |
|------|--------|
| `src/traits/configuration.rs` | Doc examples for `with_mutation_method` using old struct syntax |
| `src/configuration.rs` | Doc example |
| `src/engines/cellular/engine.rs` | Doc comment example |
| `src/engines/alps/engine.rs` | Doc comment example |
| `src/lib.rs` | Module-level doc comment (minor) |

### Tests

| File | Change |
|------|--------|
| `tests/operations/test_mutation_creep_gaussian.rs` | ~20 construction sites for `Creep { step }` and `Gaussian { sigma }` |
| `tests/operations/test_mutation_cauchy_levy_uniform.rs` | Construction sites for `Cauchy { scale }` and `LevyFlight { alpha }` |
| `tests/operations/test_mutation_self_adaptive.rs` | Construction sites for `SelfAdaptiveGaussian { ... }` |
| `tests/operations/test_mutation.rs` | `Creep { .. }` and `Gaussian { .. }` in `factory_non_value` rejection tests |
| `tests/observe/test_serde.rs` | Full variant list for round-trip tests |
| `tests/test_multi_parent_integration.rs` | `SelfAdaptiveGaussian { ... }` and `Gaussian { sigma }` |
| `tests/test_surrogate.rs` | Multiple `Gaussian { sigma: None }` sites |
| `tests/test_variable_length.rs` | One `factory_with_params` call → migrate to `factory` |
| `tests/types/chromosomes/test_multi_range.rs` | `Gaussian { sigma }` sites |
| `tests/types/chromosomes/test_unique.rs` | `Gaussian { sigma: None }` |
| `tests/types/chromosomes/test_multi_unique.rs` | `Gaussian { sigma: None }` |
| `tests/engines/moead/test_moead.rs` | `Differential { f: None }` |

### Examples

| File | Change |
|------|--------|
| `examples/surrogate_rastrigin.rs` | `Gaussian { sigma: None }` |
| `examples/constrained_g1.rs` | `Gaussian { sigma: None }` |
| `examples/island_model.rs` | `Gaussian { sigma: None }` |
| `examples/rastrigin.rs` | `Gaussian { sigma: None }` |
| `examples/niching.rs` | `Gaussian { sigma: None }` |
| `examples/memetic_rastrigin.rs` | `Gaussian { sigma: None }` |

---

## Common Pitfalls

### Pitfall 1: Wildcard { .. } on Tuple Variants
**What goes wrong:** `Mutation::Differential { .. }` in match arms inside `moead`, `nsga2`, `nsga3` engines becomes a compile error after the reshape.
**Why it happens:** `{ .. }` is struct-variant wildcard syntax; tuple variants use `(..)`.
**How to avoid:** After reshaping the enum, search for `{ .. }` patterns and replace with `(..)`.
**Warning signs:** `error[E0769]: tuple variant used like a struct` from the compiler.

### Pitfall 2: Serde missing on param structs
**What goes wrong:** `cargo test --features serde` fails with "the trait `Serialize` is not implemented for `GaussianParams`".
**Why it happens:** The `Mutation` enum derives serde conditionally — but its referenced types must also derive it under the same cfg gate.
**How to avoid:** Add `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` to every param struct definition. Then verify with `cargo test --features serde`.
**Warning signs:** Compile error only visible when `--features serde` is passed.

### Pitfall 3: Non-pub fields block external construction
**What goes wrong:** Users outside the crate cannot write `GaussianParams { sigma: Some(0.1) }` if the `sigma` field is private.
**Why it happens:** Default Rust struct field visibility is private.
**How to avoid:** Declare all param struct fields `pub`.
**Warning signs:** `error[E0451]: field is private` in tests or examples that construct the struct directly.

### Pitfall 4: factory_self_adaptive still uses old syntax
**What goes wrong:** `factory_self_adaptive` constructs `Mutation::SelfAdaptiveGaussian { tau, tau_prime, sigma_min, sigma_max }` inline — this is a construction site that won't trigger a compile error on the enum reshape unless the function is also updated.
**Why it happens:** The function body constructs the variant directly; if the rename from struct-variant to tuple-variant is missed here, it compiles once the match arm is removed from `mutate()`, but the function itself won't compile.
**How to avoid:** Include `factory_self_adaptive` in the primary `src/operations/mutation.rs` edit pass.
**Warning signs:** Compile error inside `factory_self_adaptive`.

### Pitfall 5: Test uses factory_with_params (removal target)
**What goes wrong:** `tests/test_variable_length.rs:58` calls `factory_with_params(Mutation::PermutationInsert, ...)` — after the function is removed, this test fails to compile.
**Why it happens:** One test exercises the now-removed compatibility shim.
**How to avoid:** Migrate this call to `factory(Mutation::PermutationInsert, ...)` as part of the removal pass.
**Warning signs:** `error[E0425]: cannot find function 'factory_with_params'`.

### Pitfall 6: OperatorCompat valid_mutations lists contain only unit variants
**What goes wrong:** After the refactor, `UniqueChromosome::valid_mutations()` returns `&[Mutation::Insertion, Mutation::Swap, Mutation::Inversion]`. These are all unit variants — unaffected by the reshape. No action needed.
**Why it happens:** The restricted lists only include unit variants; no struct-variant entries exist in those lists.
**How to avoid:** Verify no `valid_mutations()` impl references a parameterized variant before assuming no action is needed.

---

## Code Examples

### Defining all 8 param structs

```rust
// Source: inferred from existing inline field pattern in src/operations.rs

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreepParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub step: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaussianParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PolynomialParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub eta: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NonUniformParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub b: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DifferentialParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub f: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CauchyParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub scale: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LevyFlightParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub alpha: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfAdaptiveGaussianParams {
    #[cfg_attr(feature = "serde", serde(default))]
    pub tau: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub tau_prime: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma_min: Option<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma_max: Option<f64>,
}
```

### Updated match arms in MutationOperator::mutate

```rust
// Source: inferred from existing src/operations/mutation.rs dispatch
Mutation::Creep(CreepParams { step }) => {
    let s = step.unwrap_or(0.01);
    individual.creep_mutate(s);
}
Mutation::Gaussian(GaussianParams { sigma }) => {
    let s = sigma.unwrap_or(0.1);
    individual.gaussian_mutate(s);
}
Mutation::Polynomial(PolynomialParams { eta }) => {
    let eta_val = eta.unwrap_or(DEFAULT_POLYNOMIAL_ETA);
    return individual.polynomial_mutation(eta_val);
}
Mutation::Differential(DifferentialParams { f }) => {
    return Err(GaError::MutationError("...".to_string()));
}
Mutation::Cauchy(CauchyParams { scale }) => {
    let s = scale.unwrap_or(1.0);
    return individual.cauchy_mutation(s);
}
Mutation::LevyFlight(LevyFlightParams { alpha }) => {
    let a = alpha.unwrap_or(1.5);
    return individual.levy_flight_mutation(a);
}
Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams { tau, tau_prime, sigma_min, sigma_max }) => {
    let n_hint = individual.dna().len().max(1);
    let effective_tau = tau.unwrap_or_else(|| 1.0 / (2.0 * n_hint as f64).sqrt());
    let effective_tau_prime = tau_prime.unwrap_or_else(|| 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt());
    let effective_sigma_min = sigma_min.unwrap_or(1e-5_f64);
    return individual.self_adaptive_gaussian_mutation(effective_tau, effective_tau_prime, effective_sigma_min, *sigma_max);
}
```

### Differential arm in generation.rs after refactor

```rust
// Source: src/engines/ga/generation.rs existing pattern, reshaped
Mutation::Differential(DifferentialParams { f }) => {
    let f_val = f.unwrap_or(0.5);
    crate::operations::mutation::differential::differential_mutation(
        &mut child_1,
        chromosomes,
        key,
        f_val,
    )?;
}
```

### Wildcard guards in moead/nsga2/nsga3

```rust
// Before
crate::operations::Mutation::Differential { .. }
// After
crate::operations::Mutation::Differential(..)
```

### Updated factory_self_adaptive

```rust
pub fn factory_self_adaptive<U: ...>(
    individual: &mut U,
    tau: Option<f64>,
    tau_prime: Option<f64>,
    sigma_min: Option<f64>,
    sigma_max: Option<f64>,
) -> Result<(), GaError> {
    let variant = Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
        tau,
        tau_prime,
        sigma_min,
        sigma_max,
    });
    variant.mutate(individual, &variant.clone())
}
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` + `cargo test --features serde` |
| Config file | `Cargo.toml` (no external config) |
| Quick run command | `cargo test 2>&1 \| tail -5` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy -- -D warnings && cargo doc --no-deps` |

### Phase Requirements → Test Map

This phase has no new functional requirements. All tests are regression — verify existing behavior is preserved after structural reshape.

| Behavior | Test Type | Automated Command |
|----------|-----------|-------------------|
| Mutation dispatch correctness | regression | `cargo test operations::test_mutation` |
| Serde round-trip for all variants | regression | `cargo test --features serde observe::test_serde` |
| Creep/Gaussian mutations work | regression | `cargo test operations::test_mutation_creep_gaussian` |
| Cauchy/Levy/Uniform work | regression | `cargo test operations::test_mutation_cauchy_levy_uniform` |
| SelfAdaptive dispatch | regression | `cargo test operations::test_mutation_self_adaptive` |
| Variable-length mutations (factory_with_params removal) | regression | `cargo test test_variable_length` |
| WASM compile target | regression | `cargo check --target wasm32-unknown-unknown` |

### Sampling Rate

- **Per task commit:** `cargo test 2>&1 | tail -20`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy -- -D warnings`
- **Phase gate:** Full suite + `cargo doc --no-deps` (zero warnings) + `cargo check --target wasm32-unknown-unknown`

### Wave 0 Gaps

None — existing test infrastructure covers all phase requirements. No new test files needed.

---

## Security Domain

This phase makes no changes to authentication, data handling, input validation, or external APIs. ASVS categories V2–V6 are not applicable. Security enforcement: no new attack surface introduced.

---

## Open Questions (RESOLVED)

1. **Default impl on param structs — RESOLVED**
   - Decision: Add `#[derive(Default)]` to all param structs. Costs nothing; enables `GaussianParams::default()` as construction shorthand. Plan 71-01 Task 1 implements this.

2. **Public re-export of param structs — RESOLVED**
   - Finding: `src/lib.rs` uses `pub mod operations;` at line 340 (not `pub use crate::operations::Mutation` by name). The module is publicly accessible via path (`genetic_algorithms::operations::GaussianParams`). No new re-exports needed in `lib.rs`. Plan 71-03 handles test imports by adding struct names to existing `use genetic_algorithms::operations::{...}` lines in each test file.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The 4 call sites for `factory_with_chromosome_length` are exactly at generation.rs:278,305 and island/mod.rs:592,691 | Complete Change Inventory | A missed call site is a compile error — low risk, compiler will catch |
| A2 | `Mutation::NonUniform` is never matched with field destructuring inside any engine (it always returns Err in `mutate()` and is dispatched directly by the GA engine internals) | Code Examples | If a direct match with `{ b }` exists somewhere, it would be missed; grep showed no engine-level dispatch of NonUniform |
| A3 | `test_variable_length.rs:58` is the only remaining `factory_with_params` call site in tests | Complete Change Inventory | A second call site would be a compile error caught by `cargo test` |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed. (Three low-risk assumptions logged above; all will be caught by compile errors if wrong.)

---

## Sources

### Primary (HIGH confidence)
- Direct codebase read: `src/operations.rs` lines 253–391 — [VERIFIED: codebase grep]
- Direct codebase read: `src/operations/mutation.rs` — [VERIFIED: codebase grep]
- Direct codebase read: `src/engines/ga/generation.rs` — [VERIFIED: codebase grep]
- Direct codebase read: `src/engines/island/mod.rs` — [VERIFIED: codebase grep]
- Direct codebase read: `tests/observe/test_serde.rs` — [VERIFIED: codebase grep]
- CONTEXT.md decisions D-01 through D-07 — [VERIFIED: CONTEXT.md read]

### Secondary (MEDIUM confidence)
- grep scan of all `Mutation::{ struct variant }` construction sites across `src/`, `tests/`, `examples/` — [VERIFIED: codebase grep] — completeness depends on grep coverage

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — pure Rust, no external deps
- Architecture: HIGH — all changes verified directly from source files
- Pitfalls: HIGH — derived from direct compiler knowledge and existing codebase patterns
- Change inventory: MEDIUM — grep-verified but completeness relies on grep coverage; compiler errors catch any gaps

**Research date:** 2026-06-18
**Valid until:** N/A — all findings are from a point-in-time codebase read; re-read primary files if >1 week elapses before planning
