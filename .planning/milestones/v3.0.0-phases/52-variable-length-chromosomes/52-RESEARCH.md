# Phase 52: Variable-Length Chromosomes - Research

**Researched:** 2026-05-24
**Domain:** Rust genetic algorithm engine — variable-length chromosome support, mutation rename, crossover enforcement, parsimony pressure
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `Mutation::Insertion` (permutation-move) renamed to `Mutation::PermutationInsert`. Breaking change, allowed in v3.0.0. All callsites, tests, and docs updated.
- **D-02:** `Mutation::Insertion` (new) = length-growing: add a gene at random position, clamped to `ChromosomeLength::Variable.max`. Returns `GaError::MutationError` if `Fixed`.
- **D-03:** `Mutation::Deletion` (new) = length-shrinking: remove a gene at random position, clamped to `ChromosomeLength::Variable.min`. Returns `GaError::MutationError` if `Fixed`.
- **D-04:** New gene for `Mutation::Insertion` sampled randomly from allele set (same as init). Consistent with `AlignmentStrategy::Pad` padding source.
- **D-05:** `AlignmentStrategy` is a public enum with `Trim` and `Pad`. Crossover variant is `Crossover::VariableLength(AlignmentStrategy)`.
- **D-06:** `Trim` — crossover point within `[0, min(len_a, len_b)]`; offspring length = `min(len_a, len_b)`.
- **D-07:** `Pad` — pad shorter parent with random genes from allele set up to `max(len_a, len_b)`; offspring length = `max(len_a, len_b)`.
- **D-08:** Both strategies use single-point crossover within aligned region. No inner crossover configurability. `AlignmentStrategy` is `Copy`.
- **D-09:** All 9 existing fixed-length crossover operators check `p1.dna().len() == p2.dna().len()` and return `GaError::CrossoverError(String)` (not a new variant). Use `CrossoverError(String)` with descriptive message.
- **D-10:** `length_penalty: Option<f64>` added to `SurvivorConfiguration` (i.e., embedded in `GaConfiguration` alongside `survivor: Survivor`). `None` = disabled.
- **D-11:** Parsimony applied by computing adjusted effective fitness for comparison only (stored fitness unchanged). Formula auto-adjusts for maximization/minimization.
- **D-12:** Parsimony applied in ALL survivor operators when `length_penalty` is set.
- **D-13:** Extension regrowth scans surviving population for `[min_len, max_len]` when `ChromosomeLength::Variable`.
- **D-14:** Sampled length passed as `genes_per_chromosome` to existing `init_fn(genes_per_chromosome, alleles)`. No signature change.
- **D-15:** Variable-length sampling only for `ChromosomeLength::Variable`; Fixed path unchanged.

### Claude's Discretion

None noted — all decisions are locked.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MUT-06 | User can configure `Mutation::Insertion` (add gene) and `Mutation::Deletion` (remove gene), only valid when `ChromosomeLength::Variable`. Lengths clamp to `[min, max]`. | D-01 through D-04; existing `insertion_mutation()` renamed; two new mutation files |
| CHR-01 | User can configure `ChromosomeLength::Variable { min, max }` — existing crossover operators return `GaError::CrossoverError` for unequal-length parents; `Crossover::VariableLength(AlignmentStrategy)` handles variable-length parents; `ExtensionOperator` samples length distribution from population | D-05 through D-09, D-13 through D-15; `AlignmentStrategy` enum + new crossover file; crossover guard helper; extension regrowth branch |
| CHR-02 | User can optionally apply parsimony pressure — `length_penalty: Option<f64>` in survivor configuration penalizes longer chromosomes | D-10 through D-12; `SurvivorConfiguration`-like field in `GaConfiguration`; adjusted-fitness computation in all survivor variants |
</phase_requirements>

---

## Summary

Phase 52 wires the already-defined `ChromosomeLength::Variable` enum into the standard `Ga<U>` engine, making variable-length chromosomes functional end-to-end. The work divides into five orthogonal clusters: (1) renaming the old permutation-move operator and adding two new length-mutating operators, (2) adding a length-equality guard to all existing fixed-length crossover operators via a shared helper, (3) implementing `Crossover::VariableLength(AlignmentStrategy)` as a new crossover file, (4) threading parsimony pressure through survivor selection, and (5) enabling length-aware regrowth in the extension operator.

All five clusters are mechanically well-understood from the existing codebase. The `Mutation::Insertion` rename touches the `Mutation` enum, the dispatch match in `mutation.rs`, the `factory_non_value` match, tests, and docs. The `AlignmentStrategy::Pad` path requires access to the allele set at crossover time, which is a new concern — the crossover function signature (`fn crossover<U>(&self, p1: &U, p2: &U) -> Result<Vec<U>, GaError>`) has no allele parameter, so padded genes must be sampled from a random existing gene (fallback) or from the chromosome's gene range. This is the single most significant design question the planner must resolve (see Open Questions).

The parsimony pressure design is clean: add `length_penalty: Option<f64>` as a flat field directly on `GaConfiguration` (like `elitism_count: usize`), then pass it through `survivor::factory()`. No `SurvivorConfiguration` struct currently exists — the survivor is stored as `survivor: Survivor` plus `limit_configuration: LimitConfiguration`. The planner should add `length_penalty` as a flat field on `GaConfiguration`, not in a new sub-struct.

**Primary recommendation:** Implement the five clusters in this order: (a) enum changes + rename, (b) crossover guard helper + guard all fixed operators, (c) `Crossover::VariableLength`, (d) `length_penalty` field + survivor integration, (e) extension regrowth + ga.rs `ChromosomeLength::Variable` unlock.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Mutation rename (`Insertion` → `PermutationInsert`) | `src/operations.rs` enum + `src/operations/mutation.rs` dispatch | All test files referencing `Mutation::Insertion` | Enum variant is the source of truth; dispatch match and tests follow |
| `Mutation::Insertion` (new, length-growing) | `src/operations/mutation/` (new file) | `src/operations/mutation.rs` dispatch | Follows existing operator-per-file pattern |
| `Mutation::Deletion` (new, length-shrinking) | `src/operations/mutation/` (new file) | `src/operations/mutation.rs` dispatch | Same pattern |
| `AlignmentStrategy` enum | `src/operations.rs` | — | Public enum lives beside operator enums |
| `Crossover::VariableLength(AlignmentStrategy)` | `src/operations.rs` enum + `src/operations/crossover/` (new file) | `src/operations/crossover.rs` dispatch | Enum variant + implementation file pattern |
| Fixed crossover length guard | `src/operations/crossover/*.rs` (10+ files) | Shared helper in `src/operations/crossover.rs` | Guard helper avoids copy-paste across 10 files |
| Parsimony pressure field | `src/configuration.rs` `GaConfiguration` | `src/traits/configuration.rs` `SurvivorConfig`-like trait | Flat field on `GaConfiguration` matching `elitism_count` pattern |
| Parsimony pressure application | `src/operations/survivor/*.rs` (all variants) | `src/operations/survivor.rs` factory | Each variant adjusts sort key when `length_penalty` is set |
| `ga.rs` Variable length unlock | `src/engines/ga.rs` (3 `match ChromosomeLength` blocks) | — | Currently returns `Err` for `Variable`; Phase 52 removes those errors |
| Extension variable-length regrowth | `src/engines/ga.rs` regrowth block | `src/operations/extension/mass_genesis.rs` | Length sampling replaces fixed `n` in the `deficit` loop |
| Validation | `src/validators/generic_validator.rs` | `src/engines/ga.rs` `run()` call chain | Validate `min <= max` and `min >= 1` for `Variable` |
| Builder method `with_length_penalty` | `src/engines/ga.rs` impl block | `src/traits/configuration.rs` | Follows existing builder method pattern |

---

## Standard Stack

### Core (No New Dependencies)

Phase 52 introduces no new external crates. All capabilities are implemented using:

| Item | Source | Purpose |
|------|--------|---------|
| `rand::Rng` via `crate::rng::make_rng()` | Already in Cargo.toml | Random position sampling in new mutation operators and padding |
| `std::borrow::Cow` | std | `set_dna(Cow::Owned(dna))` — existing zero-copy pattern |
| `log::debug!(target=...)` | Already in Cargo.toml | Operator event logging |
| `crate::error::GaError` | Internal | `MutationError(String)` / `CrossoverError(String)` for operator failures |
| `crate::chromosomes::ChromosomeLength` | `src/types/chromosomes/length.rs` | Already defined; phase wires it in |

[VERIFIED: codebase grep] No new `Cargo.toml` dependencies required.

### Package Legitimacy Audit

> No external packages are installed in this phase.

**Packages removed due to slopcheck:** none
**Packages flagged as suspicious:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User configures Ga<U>
  └── .with_chromosome_length(ChromosomeLength::Variable { min, max })
  └── .with_crossover_method(Crossover::VariableLength(AlignmentStrategy::Trim | Pad))
  └── .with_mutation_method(Mutation::Insertion | Deletion)
  └── .with_length_penalty(f64)               ← new builder method

ga.rs run() loop
  ├── initialize_random() / initialize_with_seeds()
  │     └── ChromosomeLength::Variable → sample length uniformly [min, max] per individual [NEW]
  │
  ├── crossover (per parent pair)
  │     ├── Crossover::VariableLength(Trim) → align to min_len, single-point [NEW]
  │     ├── Crossover::VariableLength(Pad)  → pad shorter to max_len, single-point [NEW]
  │     └── All fixed operators → check len_a == len_b → CrossoverError if not [NEW GUARD]
  │
  ├── mutation (per offspring)
  │     ├── Mutation::PermutationInsert → renamed insertion_mutation() [RENAMED]
  │     ├── Mutation::Insertion → add gene at random pos, clamp to max [NEW]
  │     └── Mutation::Deletion  → remove gene at random pos, clamp to min [NEW]
  │
  ├── survivor::factory(survivor, chromosomes, pop_size, limit_config, length_penalty)
  │     └── When length_penalty.is_some() → adjusted_fitness = fitness ∓ penalty×len [NEW]
  │
  └── extension regrowth (when population shrinks)
        ├── ChromosomeLength::Fixed(n)        → existing path unchanged
        └── ChromosomeLength::Variable{min,max}→ scan survivors for [min_len, max_len],
                                                sample per individual [NEW]
```

### Recommended Project Structure

```
src/operations/mutation/
├── insertion.rs          → RENAME function to permutation_insert_mutation(); content unchanged
├── deletion.rs           → NEW: deletion_mutation(individual, min_len) -> Result<(), GaError>
└── (insertion.rs for new)→ NOTE: file keeps the name; only the exported function + enum dispatch changes

src/operations/crossover/
└── variable_length.rs    → NEW: variable_length_crossover(p1, p2, strategy) -> Result<Vec<U>, GaError>
```

### Pattern 1: Mutation Enum Rename + New Variants

**What:** Rename `Mutation::Insertion` to `Mutation::PermutationInsert`; add `Mutation::Insertion` (new) and `Mutation::Deletion`.

**Key constraint:** `Mutation` is `#[derive(Copy, Clone, Debug, PartialEq)]`. Both new variants carry no data, satisfying `Copy`. [VERIFIED: codebase read of `src/operations.rs`]

**Dispatch locations that must ALL be updated:**
1. `src/operations.rs` — enum definition (rename + add)
2. `src/operations/mutation.rs` — `impl MutationOperator for Mutation` match block
3. `src/operations/mutation.rs` — `factory_non_value()` match block
4. All tests referencing `Mutation::Insertion` — currently in `tests/operations/test_mutation_insertion.rs` and `tests/operations/test_mutation.rs`
5. Doc comments in `src/operations.rs` module-level doc table

**Example — enum addition (src/operations.rs):**
```rust
// Source: codebase pattern from existing Mutation enum
pub enum Mutation {
    // ... existing variants ...

    /// Permutation-insert mutation (renamed from Insertion).
    /// Removes a gene from one position and re-inserts it adjacent to another.
    /// Preserves all alleles. Suited for permutation encodings.
    PermutationInsert,

    /// Length-growing insertion mutation. Adds a new gene at a random position,
    /// clamped to `ChromosomeLength::Variable.max`. Returns `GaError::MutationError`
    /// if `ChromosomeLength::Fixed` is configured.
    Insertion,

    /// Length-shrinking deletion mutation. Removes a gene at a random position,
    /// clamped to `ChromosomeLength::Variable.min`. Returns `GaError::MutationError`
    /// if `ChromosomeLength::Fixed` is configured.
    Deletion,
}
```

### Pattern 2: New Mutation Operator File — Deletion

**What:** New file `src/operations/mutation/deletion.rs` implementing `deletion_mutation()`.

**Example structure (modeled on `insertion.rs`):**
```rust
// Source: modeled on src/operations/mutation/insertion.rs pattern
use crate::error::GaError;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::borrow::Cow;

/// Length-shrinking deletion mutation.
///
/// Removes one gene at a random position. Clamped to `min_len`.
/// If DNA length is already <= `min_len`, returns `Ok(())` (no-op).
pub fn deletion_mutation<U: LinearChromosome>(
    individual: &mut U,
    min_len: usize,
) -> Result<(), GaError> {
    let len = individual.dna().len();
    if len <= min_len {
        debug!(target="mutation_events", method="deletion"; "DNA at min length {}, skipping deletion", min_len);
        return Ok(());
    }
    let mut rng = crate::rng::make_rng();
    let pos = rng.random_range(0..len);
    let mut dna = individual.dna().to_vec();
    dna.remove(pos);
    individual.set_dna(Cow::Owned(dna));
    debug!(target="mutation_events", method="deletion"; "Deletion mutation: removed gene at {}", pos);
    Ok(())
}
```

**Key:** `min_len` comes from `ChromosomeLength::Variable.min` — retrieved from `LimitConfiguration` in the dispatch match. The `MutationOperator::mutate` signature only has `step: Option<f64>` and `sigma: Option<f64>` — neither carries length bounds. The dispatch match in `mutation.rs` cannot access `LimitConfiguration` as currently designed.

**Critical design issue (see Open Questions #1):** The `MutationOperator::mutate` signature does not accept `ChromosomeLength`. The dispatch code in `ga.rs` calls `mutation::factory_with_params(method, individual, step, sigma)`. The `min`/`max` bounds must be extracted from `LimitConfiguration` and threaded through a new dedicated factory function (`factory_variable_length`) that takes `ChromosomeLength` as a parameter, or the dispatch must happen directly in `ga.rs` with a pre-check. The planner must choose one approach.

### Pattern 3: New Mutation File — Insertion (Length-Growing)

**What:** New function `insertion_add_mutation()` in a new or existing file. To avoid confusion with the renamed `insertion.rs`, the planner should either rename the existing `insertion.rs` to `permutation_insert.rs` or add the new function to a separate `insertion_add.rs` file. The simpler approach is to rename the existing file and add a new `insertion_add.rs`.

**Allele sampling for new gene (D-04):** The gene is sampled from `self.alleles`. The allele set is available in `ga.rs` but NOT in the operator function signature. Same constraint as deletion: needs a dedicated factory or direct dispatch in `ga.rs`.

**Fallback when alleles is empty:** Clone a random existing gene in the DNA. This is the degenerate case mentioned in CONTEXT.md specifics.

### Pattern 4: Crossover Length Guard Helper

**What:** Shared `check_compatible_length<U: LinearChromosome>(p1: &U, p2: &U) -> Result<(), GaError>` in `src/operations/crossover.rs`. Called at the entry of all fixed crossover functions.

**Example:**
```rust
// Source: src/operations/crossover/single_point.rs existing pattern, generalized
pub(crate) fn check_compatible_length<U: LinearChromosome>(
    p1: &U,
    p2: &U,
) -> Result<(), GaError> {
    if p1.dna().len() != p2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "IncompatibleChromosomeLength: parent 1 has {} genes, parent 2 has {} genes. \
             Use Crossover::VariableLength for variable-length populations.",
            p1.dna().len(),
            p2.dna().len()
        )));
    }
    Ok(())
}
```

**Files that need this guard added (all fixed-length operators):**
- `single_point.rs` — already has an ad-hoc guard; replace with helper call [VERIFIED: codebase read]
- `multipoint.rs`
- `uniform_crossover.rs`
- `cycle.rs`
- `order.rs`
- `pmx.rs`
- `sbx.rs`
- `blend_alpha.rs`
- `arithmetic.rs`
- `edge_recombination.rs`

`clone.rs` and `rejuvenate.rs` — these copy parents without recombining; the planner should decide whether to guard them too (defensive) or exempt them (they work on any-length parents by design). The CONTEXT.md says "all existing fixed-length crossover operators" so guard all 10. `Undx`, `Spx`, `Pcx` — these are already behind `factory_multi_parent` and return `Err` from the 2-parent path; no guard needed.

### Pattern 5: `Crossover::VariableLength` Implementation

**What:** New file `src/operations/crossover/variable_length.rs`.

**Trim strategy:**
```rust
// Source: based on src/operations/crossover/single_point.rs pattern
pub fn variable_length_trim<U: LinearChromosome>(
    p1: &U, p2: &U
) -> Result<Vec<U>, GaError> {
    let len = p1.dna().len().min(p2.dna().len());
    if len < 1 {
        return Err(GaError::CrossoverError(
            "VariableLength(Trim): both parents must have at least 1 gene".into()
        ));
    }
    let mut rng = crate::rng::make_rng();
    // Point uniformly in [0, len]; point=0 → child is all-p2; point=len → child is all-p1.
    // For len==1, point must be 0 or 1 to avoid empty offspring.
    let point = if len == 1 { rng.random_range(0..=1) } else { rng.random_range(0..len) };

    let d1 = p1.dna();
    let d2 = p2.dna();

    let mut c1_dna = Vec::with_capacity(len);
    let mut c2_dna = Vec::with_capacity(len);
    c1_dna.extend_from_slice(&d1[..point]);
    c1_dna.extend_from_slice(&d2[point..len]);
    c2_dna.extend_from_slice(&d2[..point]);
    c2_dna.extend_from_slice(&d1[point..len]);

    let mut c1 = U::new(); c1.set_dna(Cow::Owned(c1_dna));
    let mut c2 = U::new(); c2.set_dna(Cow::Owned(c2_dna));
    Ok(vec![c1, c2])
}
```

**Pad strategy — allele access problem:** `Pad` requires padding genes from the allele set, but the crossover function signature `fn crossover<U: LinearChromosome>(&self, p1: &U, p2: &U)` has no allele parameter. **Resolution:** The `Pad` variant must be dispatched from `ga.rs` directly (bypassing `CrossoverOperator::crossover`) or the crossover module must expose a `variable_length_pad(p1, p2, alleles: Option<&[U::Gene]>)` function that `ga.rs` calls explicitly when it detects `Crossover::VariableLength(Pad)`. The planner should document which approach is chosen. Recommended: add a second public function `variable_length_pad(p1, p2, alleles: Option<&[U::Gene]>)` that `ga.rs` calls directly via a pre-dispatch check (identical pattern to how `factory_multi_parent` bypasses `CrossoverOperator::crossover` for UNDX/SPX/PCX).

**Fallback for empty alleles during Pad:** Clone a random gene from the shorter parent's DNA.

### Pattern 6: Parsimony Pressure — Field Addition

**What:** Add `length_penalty: Option<f64>` as a flat field on `GaConfiguration` (same level as `elitism_count: usize`). There is no separate `SurvivorConfiguration` struct in the current codebase — survivor state is just `pub(crate) survivor: Survivor`. [VERIFIED: codebase read of `src/configuration.rs`]

**Changes required:**
1. `src/configuration.rs` — add `pub(crate) length_penalty: Option<f64>` to `GaConfiguration` struct; default `None`.
2. `src/configuration.rs` `GaConfiguration::default()` — initialize to `None`.
3. `src/traits/configuration.rs` — add `with_length_penalty(f64) -> Self` to `SurvivorConfig` trait (or `ConfigurationT` directly if no `SurvivorConfig` exists — check; currently only `SelectionConfig`, `CrossoverConfig`, `MutationConfig`, `StoppingConfig`, `ElitismConfig`, `ExtensionConfig`, `LocalSearchConfig`; no `SurvivorConfig` builder trait yet).
4. `src/engines/ga.rs` — implement `with_length_penalty(f64) -> Self` on `Ga<U>`.
5. `src/operations/survivor.rs` — `factory()` signature gains `length_penalty: Option<f64>`; all calls in `ga.rs` updated.

**Adjusted fitness computation (D-11):**
```rust
// Applied in each survivor variant when length_penalty.is_some()
fn effective_fitness(chromosome: &impl ChromosomeT, problem: ProblemSolving, penalty: f64) -> f64 {
    let raw = chromosome.fitness();
    let adjustment = penalty * chromosome.dna().len() as f64;
    match problem {
        ProblemSolving::Maximization => raw - adjustment,
        ProblemSolving::Minimization | ProblemSolving::FixedFitness => raw + adjustment,
    }
}
```

**Important:** This effective fitness is used ONLY for sort comparisons. The stored `chromosome.fitness()` value is never mutated (D-11).

### Pattern 7: Extension Variable-Length Regrowth

**What:** In `ga.rs`, the regrowth block after extension checks `ChromosomeLength`. Currently three locations return `Err` for `Variable`. Phase 52 replaces those errors with functional code.

**For `ChromosomeLength::Variable { min, max }`:**
```rust
// In the regrowth deficit loop — sample length per individual from [min_obs, max_obs]
let min_obs = self.population.chromosomes.iter()
    .map(|c| c.dna().len())
    .min()
    .unwrap_or(min);
let max_obs = self.population.chromosomes.iter()
    .map(|c| c.dna().len())
    .max()
    .unwrap_or(max);
// Then per individual:
let indiv_len = rng.random_range(min_obs..=max_obs);
let genes = init_fn(indiv_len, alleles_ref);
```

**Same pattern applies in `initialize_random()` and `initialize_with_seeds()`:** Sample individual length uniformly from `[min, max]` instead of using a fixed `n`.

### Anti-Patterns to Avoid

- **Mutating stored fitness for parsimony:** D-11 explicitly prohibits this. Sort-only adjustment must be computed inline.
- **Returning a new `GaError` variant for incompatible length:** D-09 says use `GaError::CrossoverError(String)`. Adding a new variant is a semver break that the discussion explicitly rejected.
- **Passing `ChromosomeLength` into the `MutationOperator::mutate` trait signature:** This would break the trait definition and all existing implementations. Use a dedicated factory function or direct dispatch.
- **Forgetting `factory_non_value()` when renaming `Mutation::Insertion`:** This match block in `mutation.rs` is separate from the `MutationOperator for Mutation` impl and must also be updated.
- **`#[cfg(not(target_arch = "wasm32"))]` gaps:** Any use of `rng.random_range()` in the init path is fine (rand supports wasm). The parallel init loops `(0..deficit).into_par_iter()` already have wasm cfg gates in `ga.rs` — the new variable-length regrowth code must fit the same pattern.
- **Empty offspring from Trim when both parents have length 1:** Crossover point range `[0, min_len]` with `min_len = 1` gives a valid 1-gene offspring. Guard against `len < 1` (both parents empty) not `len < 2`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random position sampling | Custom uniform sampler | `crate::rng::make_rng()` + `rng.random_range()` | Already seeded, WASM-compatible |
| Zero-copy DNA writes | `dna.clone()` everywhere | `Cow::Owned(dna)` via `set_dna()` | Existing performance pattern |
| Parallel regrowth | Custom thread pool | Existing `(0..n).into_par_iter()` with wasm cfg gate | Already in `ga.rs` regrowth block |
| Sorting with adjusted fitness | Second Vec allocation | Inline sort key closure capturing penalty | Avoids allocation in hot path |

---

## Common Pitfalls

### Pitfall 1: Mutation Operator Allele Access

**What goes wrong:** `Mutation::Insertion` (new) needs to sample a gene from the allele set, but `MutationOperator::mutate(individual, step, sigma)` has no access to alleles or `ChromosomeLength`. The operator function gets called deep in the mutation dispatch chain with no chromosome-level context.

**Why it happens:** The `MutationOperator` trait was designed for allele-independent operators. Insertion/Deletion are the first operators that need both the chromosome AND the configuration.

**How to avoid:** Add a separate `factory_variable_length` function in `mutation.rs` that takes `ChromosomeLength` and `alleles: Option<&[U::Gene]>`, called directly from `ga.rs` when `method == Mutation::Insertion || Mutation::Deletion`. The `MutationOperator::mutate` dispatch arm for these variants returns `Err(GaError::MutationError("use factory_variable_length"))` to prevent accidental mis-use via the trait.

**Warning signs:** Compilation error "alleles not found in scope" inside the mutation dispatch match, or runtime `GaError` bubbling up from the trait impl.

### Pitfall 2: Crossover Allele Access for Pad Strategy

**What goes wrong:** `AlignmentStrategy::Pad` needs to pad with random alleles. `CrossoverOperator::crossover(p1, p2)` has no allele parameter.

**Why it happens:** Same trait-design constraint as mutation allele access.

**How to avoid:** Dispatch `Crossover::VariableLength(Pad)` directly in `ga.rs` crossover call site (before the trait dispatch), passing `alleles_ref` explicitly to `variable_length_pad(p1, p2, alleles_ref)`. The `CrossoverOperator::crossover` match arm for `VariableLength(Pad)` returns `Err` to prevent accidental calls.

**Warning signs:** `AlignmentStrategy::Trim` works but `Pad` always produces zero-variation offspring (because it fell through to a fallback that clones existing genes from the shorter parent).

### Pitfall 3: Three ga.rs ChromosomeLength::Variable Error Sites

**What goes wrong:** Forgetting one of the three locations in `ga.rs` that return `Err` for `ChromosomeLength::Variable`:
1. `initialize_random()` — line ~1141-1148
2. `initialize_with_seeds()` — line ~1200-1207
3. Extension regrowth deficit loop — line ~1952-1959

**Why it happens:** They are not adjacent; each is inside a separate function or conditional block.

**How to avoid:** Search for `"not yet supported (Phase 52)"` — all three use this exact string as a marker. [VERIFIED: grep of engines/ga.rs]

**Warning signs:** Tests pass for initialization but fail at runtime when extension triggers; or seeds work but random init fails with `ConfigurationError`.

### Pitfall 4: Survivor Factory Signature Propagation

**What goes wrong:** Adding `length_penalty: Option<f64>` to `survivor::factory()` requires updating every call site in `ga.rs`. There are at least two call sites (regular survivor call + the `MuCommaLambda` path that swaps `survivor` temporarily).

**Why it happens:** `ga.rs` calls `survivor::factory()` in multiple places with different logic around it.

**How to avoid:** Search `survivor::factory(` in `ga.rs` — update all occurrences. Also check that `SurvivorOperator::select_survivors` trait method receives `length_penalty` (or that the adjustment is done in the `factory` wrapper before delegating to the trait).

**Warning signs:** Compiler error "expected N arguments, found N-1" at the second `survivor::factory` call site.

### Pitfall 5: `factory_non_value` Mutation Dispatch

**What goes wrong:** The `factory_non_value()` function in `mutation.rs` has a separate exhaustive match. Forgetting to add `Mutation::PermutationInsert`, `Mutation::Insertion`, and `Mutation::Deletion` arms causes a compile error (non-exhaustive match).

**Why it happens:** There are two independent match blocks: `impl MutationOperator for Mutation` and the `factory_non_value()` function.

**How to avoid:** After updating the enum, check both match blocks simultaneously.

**Warning signs:** `non-exhaustive patterns` compile error mentioning `PermutationInsert`, `Insertion`, or `Deletion`.

### Pitfall 6: WASM cfg Gaps in Variable-Length Init

**What goes wrong:** The new variable-length population init code inside `initialize_random()` must use `iter()` on WASM and `par_iter()` on native. Forgetting the cfg gate breaks the WASM target.

**Why it happens:** The mandate in `CLAUDE.md` — every `par_iter()` must be gated.

**How to avoid:** Follow the exact pattern already in the extension regrowth block:
```rust
#[cfg(not(target_arch = "wasm32"))]
let chromosomes: Vec<U> = (0..population_size).into_par_iter().map(...).collect();
#[cfg(target_arch = "wasm32")]
let chromosomes: Vec<U> = (0..population_size).map(...).collect();
```

---

## Code Examples

### Guard Helper (Crossover)
```rust
// Source: codebase pattern — models src/operations/crossover/single_point.rs existing guard
// Location: src/operations/crossover.rs (pub(crate))
pub(crate) fn check_compatible_length<U: LinearChromosome>(
    p1: &U,
    p2: &U,
) -> Result<(), GaError> {
    if p1.dna().len() != p2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "IncompatibleChromosomeLength: parent 1 has {} genes, parent 2 has {} genes. \
             Use Crossover::VariableLength for variable-length populations.",
            p1.dna().len(),
            p2.dna().len()
        )));
    }
    Ok(())
}
```

### Parsimony Adjusted Fitness (Survivor)
```rust
// Source: D-11 from CONTEXT.md, following existing fitness_based sort pattern
// Location: inline in each survivor variant or as a private helper in survivor.rs
fn effective_fitness<U: ChromosomeT>(
    c: &U,
    problem: ProblemSolving,
    penalty: f64,
) -> f64 {
    let raw = c.fitness();
    let adjust = penalty * c.dna().len() as f64;
    match problem {
        ProblemSolving::Maximization => raw - adjust,
        ProblemSolving::Minimization | ProblemSolving::FixedFitness => raw + adjust,
    }
}
```

### Extension Regrowth — Variable Length Sampling
```rust
// Source: D-13 from CONTEXT.md — length distribution from surviving population
// Location: src/engines/ga.rs regrowth block
ChromosomeLength::Variable { min, max } => {
    let min_obs = self.population.chromosomes.iter()
        .map(|c| c.dna().len())
        .min()
        .unwrap_or(*min);
    let max_obs = self.population.chromosomes.iter()
        .map(|c| c.dna().len())
        .max()
        .unwrap_or(*max);
    // Per-individual length sampling happens inside the closure:
    let mut rng = crate::rng::make_rng();
    let indiv_len = if min_obs == max_obs {
        min_obs
    } else {
        rng.random_range(min_obs..=max_obs)
    };
    init_fn(indiv_len, alleles_ref)
}
```

---

## Runtime State Inventory

> Not applicable — this is a greenfield feature addition, not a rename/refactor/migration phase.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|-----------------|--------------|--------|
| `ChromosomeLength::Variable` returns `Err` | `ChromosomeLength::Variable` fully supported | Phase 52 | Variable-length populations work end-to-end |
| `Mutation::Insertion` = permutation move | `Mutation::PermutationInsert` = permutation move; `Mutation::Insertion` = add gene | Phase 52 | Breaking rename (v3.0.0) |
| Fixed crossover silently truncates or panics on length mismatch | All fixed crossover operators return `GaError::CrossoverError` | Phase 52 | Safer; explicit error message guides users to `VariableLength` |

**Prior state confirmed by code:**
- `ChromosomeLength::Variable` already in `src/types/chromosomes/length.rs` with full serde support [VERIFIED: codebase read]
- `LimitConfiguration` already embeds `chromosome_length: ChromosomeLength` [VERIFIED: codebase read]
- `initialize_random()` / `initialize_with_seeds()` / extension regrowth all have `// Phase 52` placeholder errors for `Variable` [VERIFIED: grep of engines/ga.rs]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `AlignmentStrategy::Pad` dispatched from `ga.rs` directly (bypassing `CrossoverOperator::crossover` trait) to access alleles | Architecture Patterns §Pattern 5 | If dispatched through trait, alleles are inaccessible — Pad cannot sample random genes |
| A2 | `Mutation::Insertion` / `Deletion` dispatched via new `factory_variable_length()` function (not `MutationOperator::mutate`) to access `ChromosomeLength` and alleles | Architecture Patterns §Pattern 2/3 | If dispatched through trait, bounds and alleles are inaccessible |
| A3 | `length_penalty` is a flat field on `GaConfiguration` (not a new `SurvivorConfiguration` sub-struct) | Architecture Patterns §Pattern 6 | If planner adds a sub-struct, serde compatibility and `GaConfiguration::default()` need more changes |
| A4 | `clone.rs` and `rejuvenate.rs` get the length guard added (CONTEXT.md says "all existing fixed-length crossover operators") | Common Pitfalls §3, Anti-Patterns | If exempted, variable-length parents passed to Clone silently produce mismatched output |

---

## Open Questions

1. **Allele access for `Mutation::Insertion` and `Crossover::VariableLength(Pad)`**
   - What we know: Both operators need to sample from `self.alleles`. Neither `MutationOperator::mutate` nor `CrossoverOperator::crossover` accepts alleles as a parameter.
   - What's unclear: Whether to (a) add `alleles` to the trait signatures (breaking), (b) dispatch these specific variants directly from `ga.rs` before the trait call, or (c) expose standalone functions called only from `ga.rs`.
   - Recommendation: Option (c) — add `pub fn variable_length_pad(p1, p2, alleles: Option<&[U::Gene]>)` and `pub fn factory_variable_length(method, individual, chr_len, alleles)` — callable directly from `ga.rs`, never through the trait. The trait dispatch arms for these variants return `Err` with a clear message. This is the exact pattern used for `factory_multi_parent` (UNDX/SPX/PCX) and `factory_self_adaptive`.

2. **Survivor trait signature for `length_penalty`**
   - What we know: `SurvivorOperator::select_survivors(chromosomes, pop_size, limit_config)` exists as a trait. `factory()` in `survivor.rs` calls this.
   - What's unclear: Whether to add `length_penalty: Option<f64>` to the trait method (breaking) or keep it in `factory()` wrapper only (non-breaking, adjusts fitness before delegating).
   - Recommendation: Add to `factory()` only. Pre-compute a temporary `Vec<f64>` of adjusted fitness values and sort by those — the trait method never sees it. This is non-breaking for user-implemented custom survivor operators.

3. **`clone.rs` and `rejuvenate.rs` guard**
   - What we know: CONTEXT.md D-09 says "all existing fixed-length crossover operators." Technically Clone and Rejuvenate work on any-length parents.
   - What's unclear: Whether "fixed-length" implies "operators that assume equal length" (Clone/Rejuvenate don't) or "all non-VariableLength operators."
   - Recommendation: Add the guard to Clone and Rejuvenate anyway (defensive). A user who accidentally uses Clone on a variable-length population should get a clear error, not silent behavior.

---

## Environment Availability

> Step 2.6: SKIPPED — phase is pure Rust code/config changes; no external services, databases, or CLI tools beyond the existing Rust toolchain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All | Assumed present | — | — |
| `cargo check --target wasm32-unknown-unknown` | WASM compliance | Assumed present | — | — |

---

## Validation Architecture

> `workflow.nyquist_validation` absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Config file | `Cargo.toml` (test harness) |
| Quick run command | `cargo test --test operations -- variable_length --nocapture` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MUT-06 | `Mutation::PermutationInsert` works like old `Mutation::Insertion` | unit | `cargo test --test operations test_mutation_insertion` | Existing (rename needed) |
| MUT-06 | `Mutation::Insertion` adds a gene, clamped to max | unit | `cargo test --test operations test_mutation_variable_length_insertion` | Wave 0 |
| MUT-06 | `Mutation::Deletion` removes a gene, clamped to min | unit | `cargo test --test operations test_mutation_variable_length_deletion` | Wave 0 |
| MUT-06 | `Mutation::Insertion` on Fixed returns `MutationError` | unit | `cargo test --test operations test_mutation_insertion_on_fixed` | Wave 0 |
| MUT-06 | `Mutation::Deletion` on Fixed returns `MutationError` | unit | `cargo test --test operations test_mutation_deletion_on_fixed` | Wave 0 |
| CHR-01 | `Crossover::VariableLength(Trim)` produces offspring of `min(len_a, len_b)` | unit | `cargo test --test operations test_crossover_variable_length_trim` | Wave 0 |
| CHR-01 | `Crossover::VariableLength(Pad)` produces offspring of `max(len_a, len_b)` | unit | `cargo test --test operations test_crossover_variable_length_pad` | Wave 0 |
| CHR-01 | Fixed crossover (e.g., SinglePoint) returns `CrossoverError` for unequal lengths | unit | `cargo test --test operations test_crossover_incompatible_length` | Wave 0 (existing test updated) |
| CHR-01 | `ChromosomeLength::Variable` init samples individual lengths from `[min, max]` | integration | `cargo test test_variable_length_initialization` | Wave 0 |
| CHR-01 | Extension regrowth with `Variable` samples length from `[min_obs, max_obs]` | integration | `cargo test test_variable_length_extension_regrowth` | Wave 0 |
| CHR-02 | `length_penalty` reduces effective fitness for longer chromosomes (maximization) | unit | `cargo test --test operations test_survivor_parsimony_pressure` | Wave 0 |
| CHR-02 | `length_penalty` does NOT mutate stored fitness | unit | `cargo test test_parsimony_no_fitness_mutation` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test && cargo clippy`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite + `cargo check --target wasm32-unknown-unknown` before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/operations/test_mutation_variable_length.rs` — covers MUT-06 new operator behaviors
- [ ] `tests/operations/test_crossover_variable_length.rs` — covers CHR-01 VariableLength crossover
- [ ] `tests/operations/test_crossover_incompatible_length.rs` — covers CHR-01 guard on fixed operators (or extend existing test files with new test functions)
- [ ] `tests/operations/test_survivor_parsimony.rs` — covers CHR-02 parsimony pressure
- [ ] Integration tests for variable-length init and extension regrowth (can be added to `tests/test_engines.rs` or a new file)

---

## Security Domain

> `security_enforcement` absent from config — treated as enabled. However, this phase implements internal algorithm operators with no user input, no authentication, no network I/O, and no file I/O (except existing checkpoint path, unchanged). Standard ASVS categories do not apply to pure in-memory evolutionary operators.

| ASVS Category | Applies | Rationale |
|---------------|---------|-----------|
| V2 Authentication | No | No user identity involved |
| V3 Session Management | No | Stateless operators |
| V4 Access Control | No | Library-internal |
| V5 Input Validation | Partial | `min <= max` and `min >= 1` for `ChromosomeLength::Variable`; `length_penalty >= 0.0` guard in builder |
| V6 Cryptography | No | RNG is non-cryptographic (evolutionary use case) |

**Input validation to add:**
- `ChromosomeLength::Variable { min, max }`: validate `min >= 1` and `min <= max` in the generic validator.
- `length_penalty`: validate `>= 0.0` in the `with_length_penalty` builder method (negative penalty reverses the pressure direction confusingly).

---

## Sources

### Primary (HIGH confidence)

- Codebase: `src/types/chromosomes/length.rs` — `ChromosomeLength` enum definition
- Codebase: `src/operations/mutation/insertion.rs` — existing permutation-insert implementation
- Codebase: `src/operations/mutation.rs` — `Mutation` enum dispatch and `factory_non_value`
- Codebase: `src/operations/crossover.rs` + `single_point.rs` — crossover dispatch patterns
- Codebase: `src/operations/survivor.rs` + `fitness.rs` — survivor selection structure
- Codebase: `src/engines/ga.rs` — three `ChromosomeLength::Variable` placeholder errors; extension regrowth; allele access patterns
- Codebase: `src/configuration.rs` — `GaConfiguration` struct (flat fields, no `SurvivorConfiguration` sub-struct)
- Codebase: `src/operations/extension/mod.rs` + `mass_genesis.rs` — extension operator structure
- Codebase: `src/traits/operators.rs` — `MutationOperator`, `CrossoverOperator`, `SurvivorOperator` trait signatures
- `.planning/phases/52-variable-length-chromosomes/52-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)

- EA literature consensus: linear parsimony pressure `penalty × length` is the standard formula; typical coefficient range `1e-4` to `1e-2` [ASSUMED from training data — planner should document in rustdoc]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all Rust std + existing crate internals
- Architecture: HIGH — all files identified from codebase read; dispatch patterns verified
- Pitfalls: HIGH — derived from direct code inspection of existing patterns and constraint boundaries
- Parsimony formula: MEDIUM — standard EA literature; exact coefficient guidance is ASSUMED

**Research date:** 2026-05-24
**Valid until:** 2026-06-24 (stable Rust library, internal patterns unlikely to shift)
