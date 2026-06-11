# Phase 50: Lexicase Selection - Context

**Gathered:** 2026-05-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 50 delivers two parent selection operators — `LexicaseSelection` and `EpsilonLexicaseSelection` — backed by the new `MultiCaseFitness: ChromosomeT` opt-in trait. Lexicase shuffles test cases randomly per selection event, filters candidates case-by-case to the elite subset, and syncs scalar `fitness()` to the mean case score for survivor/stopping compatibility. Epsilon-lexicase extends this with a tolerance band: candidates within epsilon of the best on each case remain eligible. Also includes integration of both operators into the `Selection` enum and `ga.rs` dispatch.

</domain>

<decisions>
## Implementation Decisions

### MultiCaseFitness Trait & Fitness Evaluation Flow

- **D-01:** `MultiCaseFitness: ChromosomeT` provides exactly two methods: `case_fitness() -> &[f64]` and `set_case_fitness(Vec<f64>)`. The trait is a supertrait of `ChromosomeT` (not `LinearChromosome`) so it can be implemented by `TreeChromosome` in Phase 53 for GP program synthesis.
- **D-02:** Case fitness is populated by the user in their `calculate_fitness()` implementation — they call `self.set_case_fitness(vec![s1, s2, ...])` alongside setting the scalar fitness. No second callback field is needed. This is consistent with how all existing chromosomes work (single calculate_fitness() handles all fitness state).
- **D-03:** The lexicase selection function reads `chromosomes[0].case_fitness().len()` to determine the number of test cases, then shuffles indices `0..num_cases`. The case count is derived from the chromosome at runtime — no `num_cases` parameter on the selection function.
- **D-04:** After case-by-case filtering selects a candidate, the lexicase selection function calls `chromosome.set_fitness(mean_case_score)` on each individual in the population before returning pairs. This syncs the scalar `fitness()` so survivor selection and stopping criteria see an up-to-date value. (Per SEL-02: "scalar `fitness()` is set to the mean case score for survivor/stopping compatibility".)

### Selection Enum & Factory Dispatch

- **D-05:** Add `Selection::Lexicase` and `Selection::EpsilonLexicase` to the `Selection` enum in `src/operations.rs`. The enum remains `Copy` (both variants carry no data; `epsilon` and any params live in `SelectionConfiguration`).
- **D-06:** A separate `selection::factory_lexicase<U: ChromosomeT + MultiCaseFitness>(chromosomes: &[U], configuration: SelectionConfiguration, number_of_threads: usize) -> Result<Vec<(usize, usize)>, GaError>` function handles lexicase dispatch. The standard `selection::factory<U: ChromosomeT>()` returns `GaError::ConfigurationError` for `Lexicase`/`EpsilonLexicase` variants — callers that don't carry the `MultiCaseFitness` bound cannot use these operators.
- **D-07:** In `ga.rs` `run()`, the per-generation selection call gains an if/else branch: `if config.selection.method is Lexicase/EpsilonLexicase { factory_lexicase(chromosomes, config) } else { factory(chromosomes, config) }`. Dispatch happens per-generation in `run()`, not at `build()` time. Minimal disruption to the `run()` structure.
- **D-08:** The `SelectionOperator` trait impl for `Selection::Lexicase` and `Selection::EpsilonLexicase` panics with a clear message: `"Use factory_lexicase for Lexicase/EpsilonLexicase selection; SelectionOperator trait path does not support MultiCaseFitness"`. This guards the island-model and NSGA-II paths (which go through the trait) from silently misbehaving.

### Epsilon-Lexicase Configuration

- **D-09:** `epsilon: f64` is added to `SelectionConfiguration` with a sentinel default of `0.0`, which signals "use dynamic MAD". When the user calls `.with_epsilon_lexicase(0.05)`, that fixed value is used instead. This keeps `SelectionConfiguration` `Copy`-able (no `Vec`).
- **D-10:** When `epsilon == 0.0` (the default), epsilon-lexicase computes the median absolute deviation (MAD) of case scores across the population per case, once at the start of the selection call (before shuffling/filtering). O(n × num_cases) per selection event. The MAD is stable during the filtering cascade.
- **D-11:** Epsilon is a single scalar applied uniformly to all test cases. No per-case epsilon vector — keeps `SelectionConfiguration` simple and `Copy`.

### Module Layout

- **D-12:** New files: `src/operations/selection/lexicase.rs` (contains `lexicase_selection<U: ChromosomeT + MultiCaseFitness>()` and `epsilon_lexicase_selection<U: ChromosomeT + MultiCaseFitness>()`). Both functions are pub-exported from `src/operations/selection.rs`.
- **D-13:** `MultiCaseFitness` trait: `src/traits/multi_case_fitness.rs`, re-exported via `src/traits.rs` alongside `ChromosomeT` and `LinearChromosome`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §SEL (SEL-02, SEL-03) and §TRAITS (TRAITS-01) — authoritative scope for this phase

### Selection Operator Patterns (structural pattern to follow)
- `src/operations/selection.rs` — `Selection` enum, `SelectionOperator` impl, `factory()` function — add Lexicase/EpsilonLexicase to enum and new `factory_lexicase()` here
- `src/operations/selection/tournament.rs` — canonical selection impl pattern (free fn, `U: ChromosomeT + Send + Sync + 'static + Clone`)
- `src/operations/selection/clearing.rs` — example of operator with operator-specific config params; the `factory()` special-case dispatch pattern; also shows how the SelectionOperator trait path has a warning when called without config access

### Configuration Patterns
- `src/configuration.rs` — `SelectionConfiguration` struct: add `epsilon: f64` field with default `0.0` here; follow same pattern as `boltzmann_temperature`, `niche_radius`
- `src/operations.rs` — `Selection` enum definition: add `Lexicase` and `EpsilonLexicase` variants here

### Trait Architecture
- `src/traits/chromosome.rs` — `ChromosomeT`: `MultiCaseFitness` supertrait of this (not `LinearChromosome`)
- `src/traits/operators.rs` — `SelectionOperator` trait: add `Lexicase`/`EpsilonLexicase` arms to the `impl SelectionOperator for Selection` match block (with panic message)

### Engine Dispatch (where factory_lexicase call goes)
- `src/engines/ga.rs` — `run()` method: add if/else branch for Lexicase dispatch per-generation here

### Prior Phase Context
- `.planning/STATE.md` — v3.0.0 decisions: `MultiCaseFitness: ChromosomeT` locked in Phase 50 for reuse in Phase 53 GP synthesis
- `.planning/phases/47-architecture-audit-chromosomet-split/47-CONTEXT.md` — ChromosomeT/LinearChromosome split decisions (MultiCaseFitness is ChromosomeT, not LinearChromosome)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/operations/selection/clearing.rs` `clearing_selection()` — best structural analog: takes `chromosomes: &[U]` + operator-specific params, returns `Vec<(usize, usize)>`. The lexicase functions follow this exact signature shape.
- `src/operations/selection.rs` `factory()` — the dispatch function to mirror; `factory_lexicase()` follows the same structure but with `U: ChromosomeT + MultiCaseFitness` bound.
- `src/rng::make_rng()` — needed for case index shuffling (per-selection random shuffle of `0..num_cases`).

### Established Patterns
- Operator-specific config params in `SelectionConfiguration` — `epsilon: f64` with default `0.0` (sentinel for dynamic MAD), same as `boltzmann_temperature` and `niche_radius`.
- `log::debug!(target="selection_events", ...)` — established logging target for all selection operators.
- `#[cfg(not(target_arch = "wasm32"))]` gates on any `par_iter()` — lexicase must use `.iter()`, not `.par_iter()`, for WASM compatibility.
- `SelectionConfiguration` is `Copy` — epsilon must stay `f64`, not `Vec<f64>`.

### Integration Points
- `src/operations/selection.rs` — add `pub mod lexicase` and re-exports for `lexicase_selection`, `epsilon_lexicase_selection`, and `factory_lexicase`
- `src/operations.rs` — add `Lexicase` and `EpsilonLexicase` to the `Selection` enum (both `Copy`-safe, carry no data)
- `src/configuration.rs` — add `epsilon: f64` field to `SelectionConfiguration`, default `0.0`
- `src/traits/multi_case_fitness.rs` — new file; re-export from `src/traits.rs`
- `src/engines/ga.rs` `run()` — add if/else branch for lexicase dispatch
- `src/lib.rs` — ensure `MultiCaseFitness` is publicly exported from crate root

</code_context>

<specifics>
## Specific Ideas

- Dynamic MAD default is the standard academic default for ε-lexicase (Helmuth et al. 2016). Compute once per selection call from the full population before the case shuffle — stable epsilon during the filtering cascade.
- Epsilon sentinel `0.0` = use dynamic MAD. Any `epsilon > 0.0` set by the user overrides MAD. This avoids an `Option<f64>` wrapper while keeping `SelectionConfiguration` `Copy`.
- Lexicase filtering algorithm: shuffle case indices → for each case in shuffled order → keep only candidates where `case_fitness[i] == max_in_pool[i]` (standard) or `case_fitness[i] >= max_in_pool[i] - epsilon` (ε-variant) → if pool reduces to 1, return it; if all cases exhausted, pick randomly from survivors.
- After candidate selection, sync `set_fitness(mean_case_score)` on all chromosomes in the POPULATION (not just winners) so survivor/stopping sees consistent scalar values.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 50-lexicase-selection*
*Context gathered: 2026-05-22*
