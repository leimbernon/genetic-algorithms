# Phase 51: Multi-parent Crossover + Self-Adaptive Mutation - Context

**Gathered:** 2026-05-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 51 delivers three multi-parent crossover operators (UNDX, SPX, PCX) restricted to real-valued chromosomes via a new `RealValued` marker trait, plus a `SelfAdaptive: ChromosomeT` opt-in trait and `Mutation::SelfAdaptiveGaussian` operator for strategy-parameter co-evolution. A built-in `SelfAdaptive` implementation is shipped on `RangeChromosome<T>`.

Scope:
- `pub trait RealValued: LinearChromosome {}` — marker trait in `src/traits/`; implemented on `RangeChromosome<T>` and `MultiRangeChromosome<T>`
- `Crossover::Undx { num_parents }`, `Crossover::Spx { num_parents }`, `Crossover::Pcx { num_parents }` — enum variants; dispatch via new `factory_multi_parent<U: LinearChromosome + RealValued>()`
- `pub trait SelfAdaptive: ChromosomeT` — opt-in trait in `src/traits/`; `strategy_params() -> &[f64]`, `set_strategy_params(Vec<f64>)`, `adapt_strategy_params(tau: f64, tau_prime: f64)`
- `Mutation::SelfAdaptiveGaussian` — new enum variant; dispatch via existing `factory_with_params`; mutation-only (no crossover-phase sigma blending)
- Built-in `SelfAdaptive` impl on `RangeChromosome<T>`: lazy-init sigma Vec to `vec![1.0; n]` on first `strategy_params()` call; sigmas included in serde serialization

</domain>

<decisions>
## Implementation Decisions

### Multi-parent Dispatch Architecture

- **D-01:** Multi-parent crossover uses a new `factory_multi_parent<U: LinearChromosome + RealValued>(parents: &[&U], config: CrossoverConfiguration) -> Result<Vec<U>, GaError>` function, parallel to `factory_lexicase` precedent. The standard `CrossoverOperator` trait is NOT modified — it remains a 2-parent interface.
- **D-02:** `ga.rs run()` adds an if/else branch: `if config.crossover.method is Undx/Spx/Pcx { factory_multi_parent(...) } else { factory(pair, config) }`. This mirrors the lexicase dispatch pattern.
- **D-03:** Parent collection for multi-parent call: the primary pair `(i, j)` comes from selection as usual. The engine then picks `(num_parents - 2)` additional random indices from the population to fill out the parent slice. No changes to `SelectionOperator`.
- **D-04:** Each call to `factory_multi_parent()` produces **1 offspring**. The engine loops over selection pairs and calls the operator once per pair, maintaining the same total offspring count as the 2-parent path.

### RealValued Marker Trait

- **D-05:** `pub trait RealValued: LinearChromosome {}` — empty marker trait in `src/traits/real_valued.rs`, re-exported from `src/traits.rs`. This provides compile-time protection: `factory_multi_parent<U: LinearChromosome + RealValued>()` rejects Binary/Unique chromosomes at compile time.
- **D-06:** Built-in implementations: `impl RealValued for RangeChromosome<T>` and `impl RealValued for MultiRangeChromosome<T>`. Users can also impl `RealValued` on custom real-valued chromosomes. (Note: `MultiRangeChromosome` is Phase 48; add its `RealValued` impl in this phase as a forward stub.)
- **D-07:** The existing SBX/BLX/Arithmetic downcast pattern is NOT changed — they continue to use runtime `try_sbx()` etc. Only the new UNDX/SPX/PCX operators use the `RealValued` bound.

### Sigma Inheritance (Self-Adaptive Mutation)

- **D-08:** Sigma inheritance is **mutation-only** — no sigma blending in crossover operators or in `ga.rs`. When offspring are created via crossover, the child inherits the primary parent's sigma vector via chromosome clone (implicit). `SelfAdaptiveGaussian::mutate()` then applies the log-normal update.
- **D-09:** Log-normal sigma update formula: `σ'_i = σ_i × exp(τ' × N(0,1) + τ × N_i(0,1))` for all i. Both `τ` and `τ'` default to standard ES heuristics: `τ = 1 / sqrt(2 * n)`, `τ' = 1 / sqrt(2 * sqrt(n))` where `n = strategy_params().len()`. User can override via `MutationConfiguration::self_adaptive_tau` and `MutationConfiguration::self_adaptive_tau_prime`.
- **D-10:** All sigmas in the vector are updated on every `mutate()` call (full log-normal update). After sigma update, **one randomly-selected gene** is mutated using its updated sigma. Sigma lower bound `sigma_min` is enforced after each update (default `1e-5`); configurable via `MutationConfiguration::sigma_min`.
- **D-11:** `SelfAdaptiveGaussian::mutate()` downcasts via `Any` (same pattern as SBX) to check if `U: SelfAdaptive`. If not a `SelfAdaptive` chromosome, returns `GaError::MutationError("SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive")`.

### SelfAdaptive Trait Design

- **D-12:** `pub trait SelfAdaptive: ChromosomeT` — supertrait of `ChromosomeT` (not `LinearChromosome`), parallel to `MultiCaseFitness`. Trait methods: `fn strategy_params(&self) -> &[f64]`, `fn set_strategy_params(&mut self, params: Vec<f64>)`, `fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64, sigma_min: f64)`. The `adapt_strategy_params` default impl is provided by the trait (calls the log-normal formula using `strategy_params` and `set_strategy_params`, then clamps each sigma to `sigma_min`). **[Amended 2026-05-23: sigma_min added as 3rd parameter so the trait enforces the lower bound directly; sigma_min value comes from MutationConfiguration and is passed by the operator at call time — per RESEARCH.md open question #1 resolution.]**
- **D-13:** Built-in impl on `RangeChromosome<T>`: adds `strategy_params: Vec<f64>` field. Lazy init: `strategy_params()` returns `&self.strategy_params`; if empty, auto-initializes to `vec![1.0; self.dna().len()]` on first call. `set_strategy_params` replaces the vector. `adapt_strategy_params` delegates to the default trait impl.
- **D-14:** Serde: `strategy_params` field included in `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` on `RangeChromosome<T>`. Sigmas survive checkpoint save/restore — evolved strategy parameters are preserved.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §CRS (CRS-02, CRS-03, CRS-04), §MUT (MUT-05), §TRAITS (TRAITS-02) — authoritative scope for this phase

### Multi-parent Crossover Patterns (structural patterns to follow)
- `src/operations/crossover.rs` — `CrossoverOperator` impl for `Crossover`, `CrossoverConfiguration`, and the `factory()` function; add `factory_multi_parent()` alongside `factory()` here
- `src/operations/crossover/sbx.rs` — canonical real-valued crossover: `try_sbx()` downcast pattern, `SbxConvertible` trait, gene value access via `.value`, range access via `.ranges[0]`
- `src/operations/crossover/blend_alpha.rs` — second real-valued crossover; shows same f64 arithmetic pattern
- `src/operations/selection.rs` — `factory_lexicase()` precedent: separate factory function for type-restricted operator dispatch (the model for `factory_multi_parent()`)

### Self-Adaptive Mutation Patterns
- `src/operations/mutation/gaussian.rs` — `gaussian_mutation()` for single-gene Gaussian perturbation; `GaussianConvertible` trait for f64 arithmetic
- `src/operations/mutation.rs` — `MutationOperator` impl for `Mutation` enum: add `Mutation::SelfAdaptiveGaussian` arm here; follow `Mutation::Cauchy` / `Mutation::LevyFlight` arm pattern
- `src/operations/mutation/cauchy.rs` or `src/operations/mutation/levy_flight.rs` — downcast-to-`RangeChromosome` pattern for real-valued mutation operators

### Configuration (where new fields go)
- `src/configuration.rs` — `CrossoverConfiguration`: add `num_parents: Option<usize>` for UNDX/SPX/PCX; `MutationConfiguration`: add `self_adaptive_tau: Option<f64>`, `self_adaptive_tau_prime: Option<f64>`, `sigma_min: Option<f64>`
- `src/operations.rs` — `Crossover` enum: add `Undx`, `Spx`, `Pcx` variants (each carries `num_parents: usize`); `Mutation` enum: add `SelfAdaptiveGaussian`

### Trait Architecture
- `src/traits/chromosome.rs` — `ChromosomeT`: `SelfAdaptive` and `RealValued` are supertraits of `ChromosomeT` (not `LinearChromosome`); follow `MultiCaseFitness` supertrait pattern
- `src/traits/multi_case_fitness.rs` — canonical opt-in supertrait pattern; `SelfAdaptive` follows the same structure
- `src/traits/operators.rs` — `CrossoverOperator` trait: `crossover(&self, p1: &U, p2: &U)` stays 2-parent; do NOT modify this trait
- `src/traits.rs` — re-export both new traits alongside `ChromosomeT`, `LinearChromosome`, `MultiCaseFitness`

### Engine Dispatch (where ga.rs changes go)
- `src/engines/ga.rs` — `run()` method: add if/else branch for multi-parent dispatch; mirror the lexicase selection if/else branch pattern; also handle parent collection (random extras from population)

### Chromosome Types (built-in impls)
- `src/chromosomes/range.rs` — `RangeChromosome<T>`: add `strategy_params: Vec<f64>` field, `RealValued` impl, `SelfAdaptive` impl (lazy-init to 1.0)
- `src/chromosomes/multi_range.rs` — `MultiRangeChromosome<T>`: add `RealValued` impl only (no `SelfAdaptive` — Phase 48 scope)

### Prior Phase Context
- `.planning/STATE.md` — v3.0.0 decisions (esp. `ChromosomeT` split pattern, `MultiCaseFitness` supertrait precedent)
- `.planning/phases/50-lexicase-selection/50-CONTEXT.md` — `factory_lexicase` dispatch pattern (D-06/D-07); supertrait of `ChromosomeT` decision

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/operations/crossover/sbx.rs` `sbx()` — best structural analog for UNDX/SPX/PCX: takes `parents: &[RangeChromosome<T>]`, does gene-level f64 arithmetic, clamps to ranges, returns offspring via `Cow::Owned`
- `src/operations/selection.rs` `factory_lexicase()` — exact template for `factory_multi_parent()`: separate pub fn, type-restricted generic, called from `ga.rs` if/else branch
- `src/rng::make_rng()` — needed for random extra parent selection and for UNDX/SPX/PCX sampling

### Established Patterns
- `log::debug!(target="crossover_events", method="undx"; ...)` — established logging pattern for crossover operators
- `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` — serde gating on chromosome structs
- `Crossover` enum variants that carry data (e.g., `MultiPoint` carries nothing, `CrossoverConfiguration.number_of_points` carries config): new `Undx { num_parents }` etc. can carry `num_parents: usize` directly in the enum variant since `#[derive(Copy)]` is still satisfied
- `Option<f64>` fields in `MutationConfiguration` with `None` = use default: `sigma_min: Option<f64>` follows `cauchy_scale: Option<f64>` pattern exactly

### Integration Points
- `src/operations.rs` — `Crossover` enum: add `Undx { num_parents: usize }`, `Spx { num_parents: usize }`, `Pcx { num_parents: usize }`. Note: `Crossover` is `#[derive(Copy, Clone, PartialEq)]` — `usize` carries through fine
- `src/operations/crossover.rs` — add `pub mod undx`, `pub mod spx`, `pub mod pcx`, new `try_undx()` (operating on `RealValued` bound) and `factory_multi_parent()`
- `src/operations/mutation.rs` — add `Mutation::SelfAdaptiveGaussian` arm in the `impl MutationOperator for Mutation` match block; add `pub mod self_adaptive_gaussian`
- `src/traits/operators.rs` — `CrossoverOperator` trait untouched; `MutationOperator` untouched
- `src/engines/ga.rs` `run()` — add if/else for multi-parent crossover path (mirrors lexicase if/else); collect `num_parents - 2` random extra parents

</code_context>

<specifics>
## Specific Ideas

- UNDX: offspring centered at centroid of all parents, normally distributed along inter-parent direction (primary axis) and orthogonal directions. Standard sigma parameters: `sigma_xi = 0.35 / sqrt(n_parents - 1)` for orthogonal, `sigma_eta = 0.35` for principal. Gene bounds enforced post-crossover via clamp.
- SPX: parents define a simplex in R^n. Expand simplex by epsilon factor (default `sqrt(n_parents + 2)`), then sample offspring uniformly from interior.
- PCX: offspring centered around primary parent (index 0), perturbed along directions from other parents. More exploitative than UNDX/SPX.
- The `adapt_strategy_params` default trait method should be provided in the trait body (not abstract) — it encapsulates the log-normal formula so all `SelfAdaptive` implementors get it for free. Only `strategy_params()` and `set_strategy_params()` are required abstract methods.
- For the lazy-init sigma on `RangeChromosome`, the `strategy_params()` method must handle the case where `dna()` returns an empty slice (len=0) — return empty `&[]` in that case.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 51-multi-parent-crossover-self-adaptive-mutation*
*Context gathered: 2026-05-23*
