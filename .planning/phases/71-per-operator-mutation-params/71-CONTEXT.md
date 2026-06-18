# Phase 71: Per-Operator Mutation Parameters - Context

**Gathered:** 2026-06-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the legacy overloaded `factory_with_params(mutation, individual, _step, _sigma)` signature with per-operator typed parameter structs, and remove dead parameter plumbing from the mutation factory layer.

**What this phase delivers:**
- Named parameter structs for all parameterized `Mutation` enum variants: `CreepParams`, `GaussianParams`, `PolynomialParams`, `NonUniformParams`, `DifferentialParams`, `CauchyParams`, `LevyFlightParams`, `SelfAdaptiveGaussianParams`
- `Mutation` enum variants converted from inline struct fields to tuple variants: `Mutation::Gaussian(GaussianParams)` replaces `Mutation::Gaussian { sigma: Option<f64> }`
- `factory_with_params(mutation, individual, _step, _sigma)` removed entirely (redundant with `factory`)
- `factory_with_chromosome_length` simplified: `_step` and `_sigma` args removed; 4 engine call sites updated
- Zero behavioral change — all existing tests pass; trait method signatures (`creep_mutate(step)`, `polynomial_mutation(eta)`, etc.) untouched

**Out of scope:**
- `ValueMutable` trait method signatures (`creep_mutate`, `gaussian_mutate`) — stay as raw `f64`
- `RealValuedMutation` trait method signatures (`polynomial_mutation`, `cauchy_mutation`, etc.) — stay as raw `f64`
- Any new mutation operators

</domain>

<decisions>
## Implementation Decisions

### Parameter struct format
- **D-01:** Param structs use `Option<f64>` fields matching the current enum inline field pattern. `None` means "use the operator's documented default." Example: `GaussianParams { sigma: Option<f64> }`. Defaults remain at dispatch, not at construction.
- **D-02:** All param structs live in `src/operations.rs` alongside the `Mutation` enum — no new module.

### Mutation enum variant shape
- **D-03:** Parameterized variants switch to tuple form: `Mutation::Gaussian(GaussianParams)`, `Mutation::Polynomial(PolynomialParams)`, etc. Zero-param variants (Swap, Inversion, Scramble, Value, BitFlip, ListValue, PermutationInsert, Uniform, Insertion, Deletion) stay as unit variants.
- **D-04:** Only the 8 variants with existing inline fields get structs: `Creep`, `Gaussian`, `Polynomial`, `NonUniform`, `Differential`, `Cauchy`, `LevyFlight`, `SelfAdaptiveGaussian`.

### factory_with_params removal
- **D-05:** `factory_with_params(mutation, individual, _step, _sigma)` is removed entirely. Callers use `factory(mutation, individual)` directly. This is a v3.0.0 breaking change and is intentional.
- **D-06:** `factory_with_chromosome_length` is kept but simplified: `_step: Option<f64>` and `_sigma: Option<f64>` args removed. The 4 engine call sites (2 in `src/engines/ga/generation.rs`, 2 in `src/engines/island/mod.rs`) drop the trailing `None, None` args.

### Trait method signatures
- **D-07:** `ValueMutable::creep_mutate(step: f64)`, `gaussian_mutate(sigma: f64)`, and all `RealValuedMutation` trait methods (`polynomial_mutation(eta: f64)`, etc.) remain unchanged — these are internal dispatch hooks, not user-facing API. Param structs are only on the `Mutation` enum.

### Agent's Discretion
- Exact derived trait impls for param structs (`#[derive(Debug, Clone, PartialEq)]` — follow `Mutation` enum's existing derive list).
- Whether to implement `Default` on param structs (returning `None` for all fields) — reasonable ergonomic addition.
- Serde `#[cfg_attr(feature = "serde", serde(default))]` annotations on struct fields — follow the pattern on the existing inline variant fields.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary files to change
- `src/operations.rs` — `Mutation` enum definition (lines 255–391). Add param structs here; change variant shapes here.
- `src/operations/mutation.rs` — `MutationOperator::mutate` match arms, `factory_with_params` (to remove), `factory_with_chromosome_length` (to simplify), `ValueMutable` trait.

### Engine call sites that need updating
- `src/engines/ga/generation.rs` — 2 calls to `mutation::factory_with_chromosome_length(m, ind, Some(cl), None, None)`
- `src/engines/island/mod.rs` — 2 calls to `mutation::factory_with_chromosome_length(m, ind, cl, None, None)` (lines ~592, ~691)

### Trait definitions (read-only — no changes in Phase 71)
- `src/traits/real_valued_mutation.rs` — `RealValuedMutation` trait (raw f64 params — unchanged)
- `src/traits/operators.rs` line 126 — `MutationOperator::mutate` signature (unchanged)

### Prior phase context
- `.planning/phases/70-replace-operator-downcasting/70-CONTEXT.md` — Phase 70 D-04 explicitly deferred this work here.
- `src/types/chromosomes/range.rs` — `Range<T>` implements `RealValuedMutation`; match arms in its impl will need to destructure tuple variants.

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Current `Mutation` enum serde annotations (`#[cfg_attr(feature = "serde", serde(default))]`) — copy pattern to param struct fields.
- Existing `#[derive(Debug, Clone, PartialEq)]` on `Mutation` — param structs should carry the same derives.

### Established Patterns
- Inline `Option<f64>` fields with `unwrap_or(default)` at dispatch — preserve this exact pattern, just moved into named structs.
- `factory_with_params` already marks `step`/`sigma` as `_step`/`_sigma` (ignored since v3.0.0) — removal is a clean-up, not a semantic change.

### Integration Points
- `Mutation` enum pattern-match in `generation.rs` lines 267–304: `Mutation::Differential { f }` and `Mutation::Insertion | Mutation::Deletion` — these match arms will need tuple destructuring after the change.
- `engines/cellular/configuration.rs:112` and `engines/alps/configuration.rs:95` construct `Mutation::Gaussian { sigma: Some(0.1) }` — will become `Mutation::Gaussian(GaussianParams { sigma: Some(0.1) })`.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard Rust tuple-variant + named struct pattern.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 71-per-operator-mutation-params*
*Context gathered: 2026-06-18*
