# Phase 54: N-ary Selection + Per-Operator Mutation Params - Context

**Gathered:** 2026-05-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 54 delivers two breaking-change API cleanups to the operator layer:

1. **N-ary selection** (#248): `SelectionOperator::select` and `selection::factory` return `Vec<Vec<usize>>` (groups of N indices) instead of `Vec<(usize, usize)>`. Standard 2-parent selection returns groups of 2; multi-parent crossover (UNDX/SPX/PCX) receives groups of N. The GA loop dispatches to `factory` (N=2) or `factory_multi_parent` (N>2) based on `group.len()` — unifying the N-ary and multi-parent paths.

2. **Per-operator mutation params** (#249): `Mutation` enum variants carry their own parameters (`Mutation::Gaussian { sigma: f64 }`, `Mutation::Creep { step: f64 }`, etc.). The `MutationOperator` trait signature changes to `fn mutate<U>(&self, individual: &mut U, mutation: &Mutation)`. `MutationConfiguration.step` and `.sigma` generic fields are removed. The enum loses `Copy` — accepted as a v3.0.0 breaking change.

Both changes are documented in `MIGRATION.md` (Phase 65).

</domain>

<decisions>
## Implementation Decisions

### Execution Order

- **D-01:** Wave 1 = N-ary selection; Wave 2 = Mutation params. The two changes are independent; N-ary selection touches the GA loop's parent-consumption path, mutation params touches the mutation dispatch path. Separating them reduces conflict surface.

### N-ary Selection API

- **D-02:** `SelectionOperator::select` returns `Vec<Vec<usize>>`. The trait signature becomes:
  ```rust
  fn select<U>(
      &self,
      chromosomes: &[U],
      number_of_couples: usize,
      number_of_threads: usize,
      num_parents: usize,   // NEW: 2 for standard, N for multi-parent
  ) -> Vec<Vec<usize>>
  ```
  This is a breaking change for all custom `SelectionOperator` implementations — documented in MIGRATION.md.

- **D-03:** `selection::factory` signature extends to include `num_parents` read from `CrossoverConfiguration`. If `CrossoverConfiguration.num_parents` is `None` or `2`, groups of 2 are generated (backward-compatible behavior). All built-in selection operators use `num_parents` to build their groups.

- **D-04:** The GA loop's `parent_crossover` function changes its `parents` parameter from `&[(usize, usize)]` to `&[Vec<usize>]`. Inside, `group.len()` drives dispatch: `len() == 2` → `crossover::factory`, `len() > 2` → `crossover::factory_multi_parent`. This eliminates the need for a separate multi-parent selection path.

- **D-05:** `factory_multi_parent` from Phase 51 is kept as an internal helper but is now called from the unified `parent_crossover` function rather than from a separate path in the GA loop. No public API removal — it may still be useful for downstream use.

### Mutation Params API

- **D-06:** `Mutation` enum variants gain inline parameters for the fields they own. Examples:
  - `Mutation::Gaussian { sigma: Option<f64> }` (None → default 0.1)
  - `Mutation::Creep { step: Option<f64> }` (None → default 0.01)
  - `Mutation::Polynomial { eta: Option<f64> }`
  - `Mutation::Cauchy { scale: Option<f64> }`
  - `Mutation::LevyFlight { alpha: Option<f64> }`
  - `Mutation::SelfAdaptiveGaussian { tau: Option<f64>, tau_prime: Option<f64>, sigma_min: Option<f64>, sigma_max: Option<f64> }`
  - `Mutation::Insertion` / `Mutation::Deletion` — no inline params (length bounds come from `LimitConfiguration`)
  - Unit variants (`Swap`, `Inversion`, `Scramble`, `BitFlip`, `Value`, `NonUniform`, `Differential`, `PermutationInsert`) — no params needed, remain unit variants
  
  `Mutation` enum loses `Copy`; derives `Clone` instead. All places using `.copy()` semantics (AOS portfolio `Vec<Mutation>`, configuration clones) change to `.clone()`.

- **D-07:** `MutationOperator` trait signature changes to:
  ```rust
  fn mutate<U>(&self, individual: &mut U, mutation: &Mutation) -> Result<(), GaError>
  where U: LinearChromosome + ValueMutable + 'static;
  ```
  Operators extract their own params from the `mutation` variant reference. Custom `MutationOperator` impls must update their signature — breaking change documented in MIGRATION.md.

- **D-08:** `MutationConfiguration.step` and `MutationConfiguration.sigma` generic fields are removed. Operator-specific fields that already exist (`cauchy_scale`, `levy_alpha`, `polynomial_eta`, `self_adaptive_tau`, etc.) are also removed from `MutationConfiguration` — they move into the enum variants. `MutationConfiguration` retains only the fields that are operator-agnostic: `probability`, `probability_max`, `dynamic_mutation`, `probability_step`.

- **D-09:** The GA loop's big if/else chain in `parent_crossover` (currently dispatching with `mutation::factory_with_params`, `factory_self_adaptive`, etc.) is replaced by a single `mutation.mutate(&mut child, &mutation_method)` call through the trait. The operator struct reads its own params from the variant.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — no specific CHR/SEL/MUT requirements for Phase 54 beyond the breaking change described in the roadmap summary; the scope is mechanical migration of the operator API

### Architecture
- `.planning/codebase/ARCHITECTURE.md` — SelectionOperator / MutationOperator trait signatures (pre-Phase 54 state)

### Prior Phase Decisions (architectural constraints)
- `.planning/phases/51-multi-parent-crossover-self-adaptive-mutation/51-CONTEXT.md` — D-01: `factory_multi_parent` dispatch pattern; D-06: `SelfAdaptiveGaussian` sigma param handling; these must be preserved in the unified N-ary path
- `.planning/phases/47-architecture-audit-chromosomet-split/47-CONTEXT.md` — breaking-change policy and MIGRATION.md publication pattern
- `.planning/STATE.md` — v3.0.0 accumulated decisions

### Key Source Files
- `src/traits/operators.rs` — `SelectionOperator`, `MutationOperator` trait definitions (primary change targets)
- `src/operations/selection.rs` — `factory()` and `factory_lexicase()` dispatch functions
- `src/operations/mutation.rs` (or `src/operations/mutation/mod.rs`) — `factory_with_params`, `factory_self_adaptive`, `factory_with_chromosome_length` dispatch functions
- `src/engines/ga.rs` — GA loop: `parent_crossover` function (lines ~2513+); selection call site (line ~1581); mutation if/else dispatch block (lines ~2748+)
- `src/configuration.rs` — `MutationConfiguration` struct; `CrossoverConfiguration.num_parents` field

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crossover::factory_multi_parent<U>()` (`src/operations/crossover/`) — Phase 51 multi-parent dispatch; called from unified GA loop when `group.len() > 2`
- `mutation::factory_self_adaptive()` — Phase 51 self-adaptive dispatch; merges into the new trait-based call
- `selection::factory_lexicase()` — separate dispatch for lexicase; keeps its own path (not affected by N-ary change, since lexicase already returns `Vec<(usize, usize)>` internally and must be updated too)

### Established Patterns
- `Selection` enum uses the same enum+factory pattern — `Mutation` following suit (inline params) is consistent
- AOS portfolio (`Vec<Mutation>` in configuration) will change to `Vec<Mutation>` clone-only; same as `Vec<Crossover>` which is already non-Copy in some variants
- WASM cfg-gate pattern: no `Instant::now()` or `par_iter()` without `#[cfg(not(target_arch = "wasm32"))]` — nothing in this phase needs new gates but the existing ones must not be broken

### Integration Points
- `parent_crossover` function is the central integration point: takes parents, dispatches crossover + mutation for each group
- `selection::factory` is called from `Ga::run()` at line ~1581; its signature change propagates into the call site
- `GpGa` engine in `src/engines/gp/` uses its own crossover/mutation dispatch — does NOT use `factory_with_params`; not affected by mutation params change
- `LexicaseSelection` in `src/operations/selection/lexicase.rs` implements `SelectionOperator`; must be updated to new `num_parents` signature

</code_context>

<specifics>
## Specific Ideas

- The unified GA loop dispatch: `if group.len() == 2 { crossover::factory(...) } else { crossover::factory_multi_parent(...) }` — simple branch, no new abstraction
- `Mutation` variant params use `Option<f64>` (not `f64`) for all numeric params; `None` means "use operator default". This preserves zero-config ergonomics while making the param source explicit.
- `MutationConfiguration` retains: `probability`, `probability_max`, `dynamic_mutation`, `probability_step` — everything else moves to enum variants

</specifics>

<deferred>
## Deferred Ideas

- GP-specific observer hooks (`on_bloat_detected`) — deferred from Phase 53, still out of scope here
- `SelectionOperator` supporting non-uniform group sizes in a single call (e.g., some groups of 2, some of 3) — not needed for any current use case, deferred
- Making `Crossover` variants also carry inline params (parallel to Mutation change) — possible future phase if same overloading problem arises

</deferred>

---

*Phase: 54-n-ary-selection-per-operator-mutation-params*
*Context gathered: 2026-05-28*
