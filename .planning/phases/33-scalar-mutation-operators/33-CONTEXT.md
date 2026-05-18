# Phase 33: Scalar Mutation Operators - Context

**Gathered:** 2026-05-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Add three new real-valued mutation operators to the standard `Mutation` enum:
- `Mutation::Cauchy` — heavy-tailed perturbation using the Cauchy (Lorentzian) distribution with configurable scale (γ); produces occasional large jumps compared to Gaussian
- `Mutation::LevyFlight` — long-range jump using Mantegna's Lévy step algorithm with configurable stability index (α); extreme outlier steps are the defining behavior
- `Mutation::Uniform` — full gene reset: picks a new value uniformly at random within the gene's declared `[lo, hi]` range

All three operators:
- Target `Range<T>` chromosomes only; return `GaError::MutationError` for Binary/List types
- Mutate exactly ONE randomly selected gene per `mutate()` call (consistent with Gaussian/Creep/Value)
- Follow the existing enum + factory delegation pattern

No new traits. No changes to `MutationOperator::mutate` signature. All existing operators unaffected.

</domain>

<decisions>
## Implementation Decisions

### Chromosome Type Scope

- **D-01:** All three operators are `Range<T>`-only. Return `GaError::MutationError` with a clear message for Binary and List chromosomes. Consistent with Cauchy/Lévy (perturbation requires numeric range clamping) and decided to keep Uniform consistent rather than special-casing it.

### Mutation Scope (per-call behavior)

- **D-02:** Each operator mutates **one randomly selected gene** per `mutate()` call — identical to Gaussian, Creep, and Value. The GA engine's mutation probability already controls call frequency; users get fine-grained control through that mechanism.

### Uniform Semantics

- **D-03:** Uniform mutation = **full reset** — picks a new gene value uniformly at random within the gene's declared `[lo, hi]` range. Equivalent to re-initializing that gene. No new config parameter needed (uses gene's own range boundaries).
- **D-04:** When a gene has multiple declared ranges (as `Range<T>` allows), Uniform picks a **random range** and resets within it — mirrors the `gaussian.rs` `range_idx` selection pattern exactly.

### Configuration Parameters

- **D-05:** Add `cauchy_scale: Option<f64>` to `MutationConfiguration`. Default: `1.0` when `None`. Follows the `polynomial_eta` / `non_uniform_b` / `differential_f` naming pattern — one field per operator-specific parameter. Add `with_cauchy_scale(scale: f64)` builder method to `ConfigurationT`.
- **D-06:** Add `levy_alpha: Option<f64>` to `MutationConfiguration`. Default: `1.5` when `None` (standard stability index for most GA applications). Add `with_levy_alpha(alpha: f64)` builder method to `ConfigurationT`.
- **D-07:** Uniform needs no new config field — uses gene's declared range directly.
- **D-08:** Do NOT reuse `sigma` for Cauchy scale — `sigma` is semantically Gaussian's standard deviation. Separate named fields preserve clear intent and discoverability.

### Lévy Step Algorithm

- **D-09:** Use **Mantegna's algorithm** to generate Lévy-distributed steps. Standard approach cited in GA/swarm literature (Yang 2010, Firefly Algorithm, Cuckoo Search). Two normal samples combined to approximate a Lévy-stable distribution — produces correct heavy-tail behavior.
  - Mantegna formula: `step = σ_u * u / |v|^(1/α)` where `u ~ N(0, σ_u²)`, `v ~ N(0, 1)`, `σ_u = (Γ(1+α) * sin(πα/2) / (Γ((1+α)/2) * α * 2^((α-1)/2)))^(1/α)`
  - The `gaussian.rs` Box-Muller implementation already exists — reuse it for the two normal samples.

### Cauchy Step Generation

- **D-10 (Claude discretion):** Cauchy perturbation: `noise = cauchy_scale * tan(π * (u - 0.5))` where `u ~ Uniform(0, 1)`. Standard inverse-CDF method. Result clamped to `[lo, hi]`.

### Serde / Test Coverage

- **D-11:** Add `Mutation::Cauchy`, `Mutation::LevyFlight`, `Mutation::Uniform` to the serde round-trip test array in `tests/observe/test_serde.rs` (Phase 32 CR-01 lesson — mandatory for all new enum variants).

### Claude's Discretion

- Exact Gamma function implementation for Mantegna's σ_u (can use the Lanczos approximation or inline the constant for α=1.5: σ_u ≈ 0.6966)
- Whether to precompute Mantegna's σ_u once in the function body or compute inline each call
- Log target names: follow existing patterns (`mutation_events`)
- Internal helper function names and file structure within `src/operations/mutation/`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Operator Infrastructure

- `src/operations.rs` — `Mutation` enum (add `Cauchy`, `LevyFlight`, `Uniform` variants)
- `src/operations/mutation.rs` — `MutationOperator for Mutation` impl + `factory_with_params`; add match arms for all three new variants
- `src/traits/operators.rs` — `MutationOperator` trait definition (interface contract)

### Reference Implementations (Mutation)

- `src/operations/mutation/gaussian.rs` — canonical `Range<T>` mutation pattern: `try_type!()` macro, `GaussianConvertible`, Box-Muller normal sample, `rng.random_range()`, single-gene selection, value clamping to `(lo, hi)`, multi-range handling with `range_idx`. **Primary reference for all three new operators.**
- `src/operations/mutation/differential.rs` — secondary reference: `try_type!()` macro expansion for multiple concrete types, error path for non-Range chromosomes
- `src/operations/mutation/creep.rs` — single-gene perturbation pattern with step parameter (Cauchy is similar in structure)

### Configuration

- `src/configuration.rs` — `MutationConfiguration` struct (add `cauchy_scale: Option<f64>` and `levy_alpha: Option<f64>`)
- `src/traits/configuration.rs` — builder trait methods (add `with_cauchy_scale(f64)` and `with_levy_alpha(f64)`)

### Chromosome Traits

- `src/traits/chromosome.rs` — `ChromosomeT`: `dna()`, `dna_mut()`, `set_gene()`
- `src/traits/gene.rs` — `GeneT`: `id() -> i32`

### Tests

- `tests/observe/test_serde.rs` — serde round-trip test; add all three new `Mutation` variants (mandatory — Phase 32 CR-01 lesson)

### Requirements

- `.planning/REQUIREMENTS.md` §MUT-01 (Cauchy), §MUT-02 (Lévy Flight), §MUT-03 (Uniform) — exact acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `src/operations/mutation/gaussian.rs::gaussian_mutation()` — directly reusable structure: single-gene selection, `range_idx` for multi-range genes, Box-Muller implementation, value clamping. All three new operators follow this same structure, replacing only the noise generation.
- `crate::rng::make_rng()` — RNG factory used by all operators; produces a seeded or random RNG consistent with the library's seed configuration.
- `GaussianConvertible` trait (defined in `gaussian.rs`) — provides `to_f64` / `from_f64` conversion for `f64`, `f32`, `i32`, `i64`. All three new operators need this for gene value manipulation. Import from `crate::operations::mutation::gaussian::GaussianConvertible`.
- `try_type!()` macro pattern (in `differential.rs`) — dispatches to concrete `Range<T>` implementations for supported types; returns `GaError::MutationError` for unsupported types. Reuse this pattern for type dispatch in all three new operators.

### Established Patterns

- Enum variant derives: `Copy + Clone + Debug + PartialEq` + `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`
- Factory delegation: `src/operations/mutation.rs` match arm → free function in `src/operations/mutation/<name>.rs`
- Config field naming: `<operator_name>_<param_name>: Option<f64>` with `unwrap_or(<default>)` at call site in factory
- Error path: `GaError::MutationError(format!("..."))` with descriptive message
- Log: `debug!(target: "mutation_events", "...")`

### Integration Points

- `src/operations.rs` — new `Mutation` variants land here
- `src/operations/mutation.rs` — match arm dispatch + `factory_with_params` passthrough (config fields passed as args)
- `src/configuration.rs` — two new `Option<f64>` fields on `MutationConfiguration`
- `src/traits/configuration.rs` — two new builder methods
- `tests/observe/test_serde.rs` — serde round-trip test array

</code_context>

<specifics>
## Specific Ideas

- Lévy step applied to gene: `gene_value += levy_step * (hi - lo)` — scale the step by the gene's range width so that α and the step magnitude are range-independent. Or apply directly (clamp handles overshoot). To be decided by planner based on what makes the stability index more intuitive across different gene scales.
- Mantegna's σ_u for α=1.5 ≈ 0.6966 (can be precomputed as a constant if α is the default, with full computation for other values).
- Box-Muller is already manually implemented in `gaussian.rs` (lines 52-55) — the same two-uniform-sample pattern can be reused for the two normal samples in Mantegna's algorithm.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 33-Scalar Mutation Operators*
*Context gathered: 2026-05-06*
