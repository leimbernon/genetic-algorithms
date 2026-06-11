# Phase 31: Selection & Survivor Diversity Operators - Pattern Map

**Mapped:** 2026-05-04
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/operations/selection/clearing.rs` | operator | request-response | `src/operations/selection/random.rs` + `src/operations/selection/tournament.rs` | role-match |
| `src/operations/survivor/deterministic_crowding.rs` | operator | request-response | `src/operations/survivor/mu_comma_lambda.rs` | exact |
| `src/operations.rs` | enum dispatch | config | `src/operations.rs` (existing `Selection` / `Survivor` enums) | exact |
| `src/operations/selection.rs` | dispatch | request-response | `src/operations/selection.rs` (existing match arms) | exact |
| `src/operations/survivor.rs` | dispatch | request-response | `src/operations/survivor.rs` (existing match arms) | exact |
| `src/configuration.rs` | config struct | config | `src/configuration.rs` `SelectionConfiguration` (add field) | exact |
| `src/traits/configuration.rs` | builder trait | config | `src/traits/configuration.rs` `SelectionConfig` (add method) | exact |
| `tests/operations/test_selection_clearing.rs` + `tests/operations/test_survivor_deterministic_crowding.rs` | test | CRUD | `tests/operations/test_selection.rs` + `tests/operations/test_survivor.rs` | exact |

---

## Pattern Assignments

### `src/operations/selection/clearing.rs` (operator, request-response)

**Analogs:** `src/operations/selection/random.rs` (pairing logic) and `src/operations/selection/tournament.rs` (structure, logging, clamping)

**Imports pattern** (random.rs lines 7-9 / tournament.rs lines 7-10):
```rust
use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;
```

**Function signature pattern** — mirror `random` (no `couples` param, returns index pairs) since eligible-pool pairing is random (D-01):
```rust
// From src/operations/selection/random.rs lines 18-19
pub fn clearing<U: ChromosomeT>(chromosomes: &[U], niche_radius: f64) -> Vec<(usize, usize)> {
```

**Core algorithm structure** (two-phase: filter then pair):

Phase 1 — identify niche winners and build eligible index set. Iterate over chromosomes sorted by fitness descending; mark the first individual in each niche as a winner, mark everyone else within `niche_radius` (|f_a - f_b| < niche_radius) of any winner as cleared. Eligible = winners + uncovered individuals.

Phase 2 — Fisher-Yates random pairing on the eligible index pool. Copy verbatim from `src/operations/selection/random.rs` lines 21-46:
```rust
// From src/operations/selection/random.rs lines 21-46
let n = eligible.len();          // eligible: Vec<usize> of original indices
let pair_count = n / 2;
let mut mating = Vec::with_capacity(pair_count);
let mut indexes: Vec<usize> = (0..n).collect();  // positions into `eligible`
let mut rng = crate::rng::make_rng();
let mut remaining = n;

while remaining >= 2 {
    let r1 = rng.random_range(0..remaining);
    let index_value_1 = indexes[r1];
    remaining -= 1;
    indexes.swap(r1, remaining);

    let r2 = rng.random_range(0..remaining);
    let index_value_2 = indexes[r2];
    remaining -= 1;
    indexes.swap(r2, remaining);

    mating.push((eligible[index_value_1], eligible[index_value_2]));
    trace!(target="selection_events", method="clearing"; "Mating index 1 {} with index 2 {}", eligible[index_value_1], eligible[index_value_2]);
}
mating
```

**Logging pattern** (tournament.rs lines 39, 77):
```rust
debug!(target="selection_events", method="clearing"; "Starting clearing selection");
// ... work ...
debug!(target="selection_events", method="clearing"; "Clearing selection finished");
```

**RNG pattern** — always use project RNG, never `rand::thread_rng()` (random.rs line 23 / tournament.rs line 54):
```rust
let mut rng = crate::rng::make_rng();
```

**Empty / too-small guard** — copy the couples-clamping pattern from tournament.rs lines 40-44, adapted for eligible pool:
```rust
if eligible.len() < 2 {
    debug!(target="selection_events", method="clearing"; "Clearing selection finished (eligible pool too small)");
    return vec![];
}
```

---

### `src/operations/survivor/deterministic_crowding.rs` (operator, request-response)

**Analog:** `src/operations/survivor/mu_comma_lambda.rs` (exact: in-place Vec mutation, age() usage, logging pattern)

**Imports pattern** (mu_comma_lambda.rs lines 8-11):
```rust
pub(crate) use crate::{
    configuration::LimitConfiguration,
    traits::ChromosomeT,
};
use log::{debug, trace};
```

Note: `LimitConfiguration` and `ProblemSolving` are NOT needed here — DeterministicCrowding's keep-the-fitter comparison is always "higher fitness wins" on the raw score (the caller's fitness already encodes direction). Do not import them unless needed.

**Function signature pattern** (fitness.rs line 26-30, mu_comma_lambda.rs line 24-28):
```rust
pub fn deterministic_crowding<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
```

**Offspring identification pattern** (mu_comma_lambda.rs line 32):
```rust
// Offspring are age == 0; parents are age > 0 (D-05)
// Separate into parents and offspring by iterating with index
let (parents, offspring): (Vec<usize>, Vec<usize>) = (0..chromosomes.len())
    .partition(|&i| chromosomes[i].age() > 0);
```

**Hamming distance helper** (D-07, D-08 — new logic, no existing analog):
```rust
fn hamming_distance<U: ChromosomeT>(a: &U, b: &U) -> usize {
    let len = a.dna().len().min(b.dna().len());
    a.dna()[..len]
        .iter()
        .zip(b.dna()[..len].iter())
        .filter(|(ga, gb)| ga.id() != gb.id())
        .count()
}
```

**Core DC loop** — for each offspring find its most similar parent; keep the fitter; mark the parent as used. Unpaired offspring survive unconditionally (D-06):
```rust
debug!(target="survivor_events", method="deterministic_crowding"; "Starting deterministic crowding survivor selection");

let mut used_parents: Vec<bool> = vec![false; parents.len()];
let mut survivors: Vec<usize> = Vec::new();  // indices into chromosomes

for &off_idx in &offspring {
    // Find closest unused parent
    let best = parents.iter().enumerate()
        .filter(|(pi, _)| !used_parents[*pi])
        .min_by_key(|(_, &par_idx)| hamming_distance(&chromosomes[off_idx], &chromosomes[par_idx]));

    match best {
        Some((pi, &par_idx)) => {
            used_parents[pi] = true;
            // Keep the fitter of the two
            if chromosomes[off_idx].fitness() >= chromosomes[par_idx].fitness() {
                survivors.push(off_idx);
            } else {
                survivors.push(par_idx);
            }
            trace!(target="survivor_events", method="deterministic_crowding";
                "Offspring {} vs parent {}: survivor chosen", off_idx, par_idx);
        }
        None => {
            // No available parent — offspring survives unconditionally (D-06)
            survivors.push(off_idx);
        }
    }
}

// Any remaining unmatched parents also survive
for (pi, &par_idx) in parents.iter().enumerate() {
    if !used_parents[pi] {
        survivors.push(par_idx);
    }
}
```

**In-place truncation pattern** (fitness.rs lines 50-65):
```rust
// Rebuild chromosomes in-place from survivor indices
// (sort survivors to allow stable drain; or swap-to-front and truncate)
survivors.sort_unstable();
let mut new_pop: Vec<U> = survivors.iter()
    .map(|&i| chromosomes[i].clone())
    .collect();
if new_pop.len() > population_size {
    // Truncate by fitness (maximization) as tiebreaker
    new_pop.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal));
    new_pop.truncate(population_size);
}
*chromosomes = new_pop;

debug!(target="survivor_events", method="deterministic_crowding"; "Deterministic crowding survivor selection finished");
```

---

### `src/operations.rs` — add `Selection::Clearing` and `Survivor::DeterministicCrowding`

**Enum variant pattern** (operations.rs lines 18-39 for Selection, lines 119-130 for Survivor):
```rust
// Selection enum — Copy, Clone, Debug, PartialEq, optional serde
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Selection {
    // ... existing variants ...
    /// Clearing selection: niche winners survive; individuals within `niche_radius`
    /// of a winner are ineligible. Eligible pool is paired randomly (see D-01 – D-04).
    Clearing,
}

// Survivor enum — same derives
pub enum Survivor {
    // ... existing variants ...
    /// Deterministic crowding: each offspring is matched to its most similar parent
    /// (Hamming distance on gene IDs); the fitter of the pair survives (see D-05 – D-08).
    DeterministicCrowding,
}
```

---

### `src/operations/selection.rs` — add `Clearing` match arm

**pub use pattern** (selection.rs lines 13-20):
```rust
pub use self::clearing::clearing;
// ...
pub mod clearing;
```

**Match arm pattern** (selection.rs lines 41-51 — `SelectionOperator for Selection`):
```rust
Selection::Clearing => clearing(
    chromosomes,
    configuration.niche_radius,   // passed via SelectionConfiguration
),
```

**factory() dispatch pattern** (selection.rs lines 85-96):
```rust
// In factory(), the Clearing variant needs niche_radius from configuration.
// Add to the existing match or delegate via method.select():
Selection::Clearing => clearing(chromosomes, configuration.niche_radius),
// OR handle in method.select() with niche_radius plumbed through SelectionOperator::select signature?
// NOTE: SelectionOperator::select() does NOT have a niche_radius param.
// Therefore handle Clearing specially in factory() (same pattern as Boltzmann on lines 86-95):
let pairs = match configuration.method {
    Selection::Boltzmann => boltzmann_selection(...),
    Selection::Clearing => clearing(chromosomes, configuration.niche_radius),
    _ => configuration.method.select(chromosomes, configuration.number_of_couples, number_of_threads),
};
```

---

### `src/operations/survivor.rs` — add `DeterministicCrowding` match arm

**pub use pattern** (survivor.rs lines 8-11):
```rust
pub use self::deterministic_crowding::deterministic_crowding;
// ...
pub mod deterministic_crowding;
```

**Match arm pattern in `SurvivorOperator for Survivor`** (survivor.rs lines 29-39):
```rust
Survivor::DeterministicCrowding => {
    deterministic_crowding(chromosomes, population_size, limit_configuration)
}
```

The outer `impl` block already returns `Ok(())` after the match — new arm follows the same pattern (no return value from the function itself, `Ok(())` is emitted by the `impl`).

---

### `src/configuration.rs` — add `niche_radius: f64` to `SelectionConfiguration`

**Struct field pattern** (configuration.rs lines 75-91 — `SelectionConfiguration`):
```rust
pub struct SelectionConfiguration {
    pub number_of_couples: usize,
    pub method: Selection,
    /// Temperature parameter for Boltzmann selection. ...
    pub boltzmann_temperature: f64,
    /// Niche radius for Clearing selection (fitness-space distance).
    /// Individuals within this radius of a niche winner are ineligible for pairing.
    /// Only used when `method` is `Selection::Clearing`. Default is `0.1`.
    pub niche_radius: f64,
}
impl Default for SelectionConfiguration {
    fn default() -> Self {
        SelectionConfiguration {
            number_of_couples: 0,
            method: Selection::Tournament,
            boltzmann_temperature: 1.0,
            niche_radius: 0.1,   // D-03
        }
    }
}
```

Pattern: field has doc comment explaining when it is active, default value matches documented default.

---

### `src/traits/configuration.rs` — add `with_niche_radius()` to `SelectionConfig`

**Builder method pattern** (traits/configuration.rs lines 12-17 — `SelectionConfig` trait):
```rust
pub trait SelectionConfig {
    fn with_number_of_couples(self, number_of_couples: usize) -> Self;
    fn with_selection_method(self, selection_method: Selection) -> Self;
    /// Sets the niche radius for Clearing selection.
    ///
    /// Distance is measured in fitness space: |f_a - f_b|. Only used when
    /// `selection_method` is `Selection::Clearing`. Default is `0.1`.
    fn with_niche_radius(self, niche_radius: f64) -> Self;
}
```

Implementation in `Ga` (wherever `SelectionConfig` is implemented — follow the same pattern as `with_boltzmann_temperature` if it exists, or any other `SelectionConfig` impl):
```rust
fn with_niche_radius(mut self, niche_radius: f64) -> Self {
    self.configuration.selection.niche_radius = niche_radius;
    self
}
```

---

### Tests — `tests/operations/test_selection_clearing.rs` and `tests/operations/test_survivor_deterministic_crowding.rs`

**File registration pattern** (test_operations.rs lines 1-28):
```rust
// In tests/test_operations.rs, add:
mod test_selection_clearing;
mod test_survivor_deterministic_crowding;
```

**Test file imports pattern** (test_selection.rs lines 1-8):
```rust
#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::selection::clearing,
    operations::Selection,
    configuration::SelectionConfiguration,
};
```

For survivor test (test_survivor.rs lines 1-9):
```rust
#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::{LimitConfiguration, ProblemSolving},
    fitness::FitnessFnWrapper,
    operations::survivor::deterministic_crowding,
    operations::Survivor,
    traits::ChromosomeT,
};
```

**Chromosome constructor pattern** (test_selection.rs lines 21-27 — reuse throughout):
```rust
Chromosome {
    dna: vec![Gene { id: 1 }, Gene { id: 2 }],
    fitness: 10.0,
    age: 0,
    fitness_fn: FitnessFnWrapper::default(),
}
```

**Key test cases for Clearing:**
- All individuals in one niche → only 1 winner eligible → 0 pairs
- Two distinct niches → 2 winners eligible → 1 pair
- All individuals spread apart (no one cleared) → same as random pairing
- Empty population → 0 pairs (no panic)
- Population of 1 → 0 pairs (no panic)
- `niche_radius = 0.0` → no clearing, full random pairing

**Key test cases for DeterministicCrowding:**
- All offspring (`age == 0`), no parents → all offspring survive unconditionally
- All parents (`age > 0`), no offspring → all parents survive (no DC matching needed)
- 2 parents + 2 offspring: each offspring matched to most similar parent; fitter wins
- Offspring with no remaining parent → survives unconditionally (D-06)
- Hamming distance tie → first found (deterministic on sorted order)
- Mixed DNA lengths → `min(len_a, len_b)` positions compared (D-08)
- Empty population → no panic, empty result
- population_size truncation — if survivors > population_size, truncate by fitness

**Direct-function call pattern** (test_selection.rs line 67 / test_survivor.rs line 117):
```rust
// Selection: call the free function directly
let pairs = clearing::clearing(&population, 0.1);
assert_eq!(pairs.len(), expected);

// Survivor: call the free function directly
deterministic_crowding::deterministic_crowding(&mut population, pop_size, LimitConfiguration::default());
assert_eq!(population.len(), expected_len);
```

---

## Shared Patterns

### RNG
**Source:** `src/rng.rs` via `crate::rng::make_rng()`
**Apply to:** `src/operations/selection/clearing.rs`
```rust
let mut rng = crate::rng::make_rng();
```
Never use `rand::thread_rng()`.

### Logging targets
**Source:** `src/operations/selection/tournament.rs` lines 39, 58, 73, 77
**Apply to:** Both new operator files

| File | `target=` | `method=` |
|------|-----------|-----------|
| `clearing.rs` | `"selection_events"` | `"clearing"` |
| `deterministic_crowding.rs` | `"survivor_events"` | `"deterministic_crowding"` |

Pattern:
```rust
debug!(target="selection_events", method="clearing"; "Starting clearing selection");
trace!(target="selection_events", method="clearing"; "...");
debug!(target="selection_events", method="clearing"; "Clearing selection finished");
```

### Serde derives on enum variants
**Source:** `src/operations.rs` lines 18-20 and 119-121
**Apply to:** New `Selection::Clearing` and `Survivor::DeterministicCrowding` variants
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
```
Both enums already carry these derives; new variants inherit them automatically — no per-variant annotation needed.

### In-place Vec mutation (survivor operators)
**Source:** `src/operations/survivor/fitness.rs` lines 49-65
**Apply to:** `src/operations/survivor/deterministic_crowding.rs`

The survivor operator receives `&mut Vec<U>` and modifies it in place. Use `truncate` / `drain` for bulk removal rather than one-by-one `Vec::remove`. If rebuilding from scratch (as DC does), assign `*chromosomes = new_pop` after constructing the new Vec.

### Default field value documentation style
**Source:** `src/configuration.rs` lines 79-81 (`boltzmann_temperature`)
**Apply to:** `niche_radius` field in `SelectionConfiguration`

Doc comment format:
```
/// <Description>. Only used when `method` is `Selection::<Variant>`. Default is `<value>`.
```

---

## No Analog Found

All files have analogs. No entries in this section.

---

## Metadata

**Analog search scope:** `src/operations/selection/`, `src/operations/survivor/`, `src/configuration.rs`, `src/traits/configuration.rs`, `src/operations.rs`, `tests/operations/`
**Files scanned:** 12 source files, 2 test files
**Pattern extraction date:** 2026-05-04
