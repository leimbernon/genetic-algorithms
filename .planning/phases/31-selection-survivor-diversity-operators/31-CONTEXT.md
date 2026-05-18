# Phase 31: Selection & Survivor Diversity Operators - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Add two diversity-promoting operators to the library:
- `Selection::Clearing` — after identifying niche winners (best individual in each niche), individuals within `niche_radius` of a winner (measured in fitness space) are cleared from the selection pool; eligible individuals are then paired randomly to form parent couples.
- `Survivor::DeterministicCrowding` — for each offspring (identified by `age() == 0`), find the most similar parent (via Hamming distance on gene IDs up to `min(len_a, len_b)`) and keep the fitter of the two; unpaired offspring survive unconditionally.

Both operators follow the existing enum + factory pattern. No new trait definitions. No new engine wiring. All existing operators remain unaffected.

</domain>

<decisions>
## Implementation Decisions

### Clearing: Internal Pairing

- **D-01:** After filtering out cleared individuals, form parent pairs by **random pairing** on the eligible pool. This is consistent with the existing `Selection::Random` operator and keeps selective pressure at the clearing layer only.
- **D-02:** **Standard clearing semantics**: one niche winner per niche survives (the best individual in that radius); everyone else within that winner's radius is ineligible. The eligible pool = niche winners + anyone not within any winner's niche radius.
- **D-03:** `niche_radius: f64` lives in **`SelectionConfiguration`** (add a new field with default `0.1`). Consistent with how `BoltzmannConfiguration`-style params are co-located — one config struct for all selection params, no new configuration struct.

### Clearing: Niche Distance Metric

- **D-04:** Niche radius is measured in **fitness space**: distance between individuals A and B = `|f_a - f_b|`. Generic across all chromosome types, no gene-type constraints, and the `niche_radius` value is on the same scale as the fitness function.

### DeterministicCrowding: Offspring Identification

- **D-05:** Offspring are identified by **`chromosome.age() == 0`**. Fresh chromosomes produced by crossover+mutation start at age 0; parents have been incremented and are age > 0. No API changes needed — `ChromosomeT::age()` is already available.
- **D-06:** When an offspring has no available parent to pair with (e.g., all parents already matched), the offspring **survives unconditionally**. No chromosomes are silently discarded.

### DeterministicCrowding: Distance Metric

- **D-07:** "Most similar parent" is determined by **Hamming distance on gene IDs**: count positions where `gene_a.id() != gene_b.id()`, using `GeneT::id() -> i32`. Measures structural genotype similarity across all chromosome types.
- **D-08:** When parent and offspring DNA lengths differ, compare up to `min(len_a, len_b)` — extra positions are ignored. Safe for any chromosome length without special-casing.

### Claude's Discretion

- Exact implementation structure within `src/operations/selection/clearing.rs` and `src/operations/survivor/deterministic_crowding.rs` (helper functions, loop structure, etc.)
- Whether niche winner identification iterates sorted-by-fitness or not (either is fine as long as the semantics of D-02 are met)
- Log target naming: follow existing patterns (`selection_events`, `survivor_events`)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Operator Infrastructure

- `src/operations.rs` — `Selection` enum (add `Clearing` variant here) and `Survivor` enum (add `DeterministicCrowding` variant here)
- `src/operations/selection.rs` — `SelectionOperator for Selection` impl + factory function; add `Clearing` match arm here
- `src/operations/survivor.rs` — `SurvivorOperator for Survivor` impl + factory function; add `DeterministicCrowding` match arm here
- `src/traits/operators.rs` — `SelectionOperator` and `SurvivorOperator` trait definitions (interface contracts)

### Reference Implementations

- `src/operations/selection/tournament.rs` — canonical selection operator pattern: function signature, rayon usage, logging, pair-building
- `src/operations/survivor/fitness.rs` — canonical survivor operator pattern: in-place Vec mutation, logging, LimitConfiguration usage

### Configuration

- `src/configuration.rs` lines 75–90 — `SelectionConfiguration` struct (add `niche_radius: f64` field with default)
- `src/traits/configuration.rs` — builder trait methods for configuration (add `with_niche_radius()` builder here)

### Chromosome Traits

- `src/traits/chromosome.rs` — `ChromosomeT` trait: `fitness() -> f64`, `age() -> usize`, `dna() -> &[Self::Gene]`
- `src/traits/gene.rs` — `GeneT` trait: `id() -> i32` (used for Hamming distance in DeterministicCrowding)

### Requirements

- `.planning/REQUIREMENTS.md` §SEL-01 (Clearing) and §SRV-01 (DeterministicCrowding) — exact acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `src/operations/selection/random.rs::random()` — random pairing logic; reuse or mirror for Clearing's eligible-pool pairing step
- `crate::rng::make_rng()` — RNG factory used by all operators for reproducibility; use this, not `rand::thread_rng()`
- `ChromosomeT::fitness()` — available on every chromosome; used for Clearing's fitness-space distance and DC's fitter-of-two comparison
- `ChromosomeT::age()` — zero-cost field access; the offspring signal for DeterministicCrowding
- `GeneT::id() -> i32` — available on every gene; the Hamming distance building block

### Established Patterns

- Enum variants use `Copy + Clone + Debug + PartialEq` + serde derives behind `#[cfg_attr(feature = "serde", ...)]` — replicate for new variants
- Selection match arm in `src/operations/selection.rs::SelectionOperator for Selection`: delegate to a free function in the operator's own module file
- Survivor match arm in `src/operations/survivor.rs::SurvivorOperator for Survivor`: same pattern, `Ok(())` returned by the outer match after calling the function
- Logging: `debug!(target="selection_events", method="clearing"; ...)` / `debug!(target="survivor_events", method="deterministic_crowding"; ...)`
- Tests go in `tests/` — not inline; follow the pattern of existing operator tests

### Integration Points

- `src/operations.rs` — both new enum variants land here
- `src/operations/selection.rs` and `src/operations/survivor.rs` — match arm dispatch
- `src/configuration.rs` — `niche_radius` field on `SelectionConfiguration`
- `src/traits/configuration.rs` — `with_niche_radius()` builder method
- `src/lib.rs` — verify `Selection` and `Survivor` are already re-exported (they should be via `pub use crate::operations::{Selection, Survivor}`)

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond the algorithm semantics documented in the decisions above.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 31-Selection & Survivor Diversity Operators*
*Context gathered: 2026-05-04*
