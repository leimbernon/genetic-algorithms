# Phase 70: Replace Operator Downcasting - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Eliminate all `as_any().downcast_mut()` calls in the mutation operator layer by introducing a `RealValuedMutation` trait that chromosomes implement directly, routing operators to the correct chromosome type via trait dispatch rather than runtime downcasting.

**What this phase delivers:**
- New `RealValuedMutation` trait with optional methods for Polynomial, Cauchy, LevyFlight, Uniform, and SelfAdaptive Gaussian mutation
- `Range<T>` implements all 5 methods (the only chromosome type that supports them)
- All 5 `try_*` functions in `src/operations/mutation.rs` replaced with trait method calls
- `std::any::Any` import removed from `src/operations/mutation.rs`
- Zero behavioral change — all existing tests pass identically

</domain>

<decisions>
## Implementation Decisions

### Dispatch mechanism
- **D-01:** Create a new trait `RealValuedMutation` with optional methods: `polynomial_mutation()`, `cauchy_mutation()`, `levy_flight_mutation()`, `uniform_mutation()`, `self_adaptive_gaussian_mutation()`. Each method takes the operator-specific parameters (eta, scale, alpha, tau/tau_prime/sigma_min/sigma_max) and returns `Result<(), GaError>`.
- **D-02:** Default trait implementations return `Err(GaError::MutationError(...))` with a clear message stating the chromosome doesn't support the operator. This matches the current error behavior when downcasting fails.
- **D-03:** The trait is separate from `ValueMutable` — it groups only the 5 operators that currently require `Range<T>` downcasting. `ValueMutable` continues to handle `value_mutate`, `bit_flip_mutate`, `creep_mutate`, `gaussian_mutate` as before.

### Operator signatures
- **D-04:** Phase 70 keeps all existing parameter signatures unchanged. The `try_*` functions become trait method calls but the factory match arms in `Mutation::mutate()` keep their current parameter extraction logic (eta, scale, alpha, tau, etc.). Phase 71 will clean up signatures with per-operator parameter structs.

### Error behavior
- **D-05:** When a chromosome type doesn't support an operator, return `Err(GaError::MutationError(...))` with the same error messages currently used. This is the existing behavior and downstream code already handles it.

### Trait placement
- **D-06:** `RealValuedMutation` goes in `src/traits/` (like `ValueMutable`, `LinearChromosome`, etc.) and is re-exported from `src/lib.rs`. Implementations go in `src/types/chromosomes/range.rs` (where `ValueMutable` is already implemented for `Range<T>`).

### Agent's Discretion
- Exact error message strings for default trait method implementations — use clear, actionable messages matching the current style.
- Whether to use `#[inline]` on trait default methods — follow existing trait patterns in the codebase.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `src/operations/mutation.rs` — Current downcasting code (lines 50-166: 5 try_* functions with 11 downcast calls). This is the primary file to refactor.
- `src/traits/` — Existing trait definitions (LinearChromosome, ValueMutable, ChromosomeT). New trait follows this pattern.
- `src/types/chromosomes/range.rs` — Where RealValuedMutation will be implemented for Range<T>.
- `src/operations/mutation/self_adaptive_gaussian.rs` — SelfAdaptiveGaussian mutation implementation (called from try_self_adaptive).

### Prior phases
- `src/traits/linear_chromosome.rs` — LinearChromosome supertrait (Phase 47). RealValuedMutation methods require `LinearChromosome` bound.
- `src/operations/mutation.rs` lines 186-238 — ValueMutable trait definition. Pattern reference for default-impl-with-fallback style.

### GitHub issue
- https://github.com/leimbernon/genetic-algorithms/issues/247 — Original issue: "Replace operator runtime downcasting with representation-typed dispatch"

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ValueMutable` trait (src/operations/mutation.rs:186-238): Pattern for trait with default implementations that return errors/fallbacks. RealValuedMutation follows the same pattern.
- `RangeChromosome<T>` (src/types/chromosomes/range.rs): Already implements ValueMutable. Will also implement RealValuedMutation.
- `try_type!` macro (src/operations/mutation.rs:58-64): Currently used for downcasting. Will be replaced by direct trait method calls.

### Established Patterns
- Traits live in `src/traits/` with re-exports in `src/lib.rs`
- Chromosome implementations live in `src/types/chromosomes/<type>.rs`
- Error type is `GaError::MutationError(String)`
- All mutation operators are behind `MutationOperator` trait (src/traits/mutation_operator.rs)

### Integration Points
- `Mutation::mutate()` factory (src/operations/mutation.rs:240-330): The match arm dispatch that calls try_* functions. Will call trait methods instead.
- `src/operations/mutation/mod.rs` (if exists): Module declarations for sub-modules.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard Rust trait patterns.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 70-replace-operator-downcasting*
*Context gathered: 2026-06-17*
