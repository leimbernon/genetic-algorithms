# Phase 32: Crossover & Differential Mutation - Research

**Researched:** 2026-05-04
**Domain:** Genetic operator implementation — Edge Recombination Crossover (ERX) and DE-style Differential Mutation for Rust GA library
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** The GA engine (`src/engines/ga.rs`) detects `Mutation::Differential` **before** the standard per-offspring mutation loop and calls a population-aware function directly, bypassing `factory_with_params`. No trait change to `MutationOperator::mutate`. Non-Differential operators are completely unaffected.
- **D-02:** Differential mutation is `Range<T>` chromosomes only — requires `ValueMutable` (same constraint as Gaussian/Creep). If used with Binary or List chromosomes, return a clear `GaError::MutationError`.
- **D-03:** When population is too small to draw 3 distinct members other than the target (`population_size < 4`), return `GaError::MutationError` with a clear message.
- **D-04:** Add `differential_f: Option<f64>` to `MutationConfiguration` with a default of `0.5` when `None`. Matches existing `polynomial_eta` / `non_uniform_b` pattern.
- **D-05:** Add a corresponding `with_differential_f(f: f64)` builder method to the `ConfigurationT` builder trait, following the same pattern as `with_mutation_sigma`.
- **D-06:** ERX adjacency-exhaustion fallback: randomly pick any remaining unvisited gene (canonical Whitley 1989 ERX algorithm).
- **D-07:** Minimum chromosome length for ERX is `len >= 2` — error with `GaError::CrossoverError` for shorter chromosomes.
- **D-08:** Validate gene uniqueness at factory time using an O(n) HashSet on gene IDs; if either parent contains duplicate gene IDs, return `GaError::CrossoverError`.

### Claude's Discretion

- Exact adjacency-list data structure (`HashMap<gene_id, HashSet<gene_id>>` or Vec-based)
- Tie-breaking when multiple neighbors have equal smallest remaining-neighbor count (any consistent policy)
- Whether ERX produces 1 child or 2 (producing 2 is the norm; use parent2's start gene for the second)
- Log target names: follow existing patterns (`crossover_events`, `mutation_events`)
- Internal helper function names and loop structure

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRS-01 | User can configure Edge Recombination crossover for permutation chromosomes, preserving adjacency relationships from both parents | ERX algorithm documented below; `order.rs` and `pmx.rs` patterns verified as reference implementations |
| MUT-04 | User can configure Differential mutation (DE-style) in the standard GA, using three random population members to generate a mutant vector with configurable F scale factor | DE mutation algorithm documented below; engine dispatch pattern verified in `ga.rs`; `gaussian.rs` Range<T> clamping pattern identified |
</phase_requirements>

---

## Summary

Phase 32 adds two new genetic operators to the library: `Crossover::EdgeRecombination` and `Mutation::Differential`. Both follow the established enum + factory pattern without any new traits or breaking changes to existing APIs.

Edge Recombination Crossover (ERX) is the canonical permutation operator for problems where adjacency matters more than position (TSP, scheduling). It builds a union adjacency list from both parents and grows offspring by always extending to the neighbor with the fewest remaining connections. Degenerate cases (exhausted adjacency lists) fall back to random selection of any unvisited gene.

Differential mutation is DE-style: the mutant vector is `x_r1 + F * (x_r2 - x_r3)` computed from three distinct random population members, clamped to each gene's range. Because this operator needs access to the entire population (not just the individual being mutated), the engine detects `Mutation::Differential` before the standard `factory_with_params` loop and calls a dedicated population-aware free function. The `MutationOperator::mutate` trait signature is unchanged.

**Primary recommendation:** Implement ERX in `src/operations/crossover/edge_recombination.rs` following the `pmx.rs` adjacency-mapping pattern. Implement Differential in `src/operations/mutation/differential.rs` following the `gaussian.rs` Range<T> clamping pattern. Wire the engine branch in `parent_crossover()` in `src/engines/ga.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ERX crossover logic | Operations (crossover module) | — | Operator implementation belongs in `src/operations/crossover/` per existing pattern |
| Differential mutation logic | Operations (mutation module) | Engine (ga.rs) | Core algorithm in `src/operations/mutation/`; population access dispatched from engine |
| F scale factor configuration | Configuration (MutationConfiguration) | Trait (MutationConfig) | Matches existing `polynomial_eta`/`non_uniform_b` pattern |
| Engine dispatch for Differential | Engine (ga.rs `parent_crossover`) | — | Only Differential needs population context; intercepted before standard loop |
| Enum variants + serde | Operations (operations.rs) | Serde test | Variants declared in `src/operations.rs`; serde round-trips verified in `tests/observe/test_serde.rs` |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::collections::HashMap` | stdlib | ERX adjacency map (gene_id → neighbor set) | No external dep; HashMap<i32, HashSet<i32>> is the right data structure for O(1) lookup |
| `std::collections::HashSet` | stdlib | Visited-gene tracking in ERX; duplicate-ID validation | Already used in `order.rs` for segment dedup |
| `rand` | workspace | RNG for random fallback in ERX and index sampling in Differential | Project standard via `crate::rng::make_rng()` |
| `std::any::Any` | stdlib | Downcast generic U to RangeChromosome<T> for Differential (same as SBX/BLX pattern) | Required for Range<T>-specific operations without trait changes |
| `log` | workspace | `debug!` logging with `target="crossover_events"` / `target="mutation_events"` | Project standard |

**Version verification:** All dependencies are workspace-managed — no new Cargo.toml additions required. [VERIFIED: read Cargo.toml patterns in source files]

---

## Architecture Patterns

### System Architecture Diagram

```
User config
    │  .with_crossover_method(Crossover::EdgeRecombination)
    │  .with_mutation_method(Mutation::Differential)
    │  .with_differential_f(0.8)
    ▼
GaConfiguration
    ├── CrossoverConfiguration { method: EdgeRecombination, ... }
    └── MutationConfiguration { method: Differential, differential_f: Some(0.8), ... }
    ▼
Ga::run() → parent_crossover(&parents, &chromosomes, &config, ...)
    │
    ├── [crossover phase]
    │   └── crossover::factory(p1, p2, config.crossover_configuration)
    │       └── CrossoverOperator for CrossoverConfiguration
    │           └── Crossover::EdgeRecombination → edge_recombination::erx(p1, p2)
    │               ├── validate lengths equal, len >= 2
    │               ├── validate gene uniqueness (HashSet on IDs)
    │               ├── build adjacency map from both parents
    │               ├── grow child_1 starting from p1[0]
    │               ├── grow child_2 starting from p2[0]
    │               └── return Vec<U> with 2 children
    │
    └── [mutation phase — per offspring]
        ├── if config.mutation_configuration.method == Mutation::Differential:
        │   └── differential::differential_mutation(&mut child, &chromosomes, f)
        │       ├── downcast child to RangeChromosome<T>
        │       ├── sample r1, r2, r3 ≠ child_index from chromosomes
        │       ├── compute mutant[i] = x_r1[i] + F * (x_r2[i] - x_r3[i])
        │       ├── clamp to gene ranges
        │       └── set_dna(Cow::Owned(mutant))
        └── else:
            └── mutation::factory_with_params(method, &mut child, step, sigma)
```

### Recommended Project Structure

```
src/operations/crossover/
├── edge_recombination.rs    # NEW: ERX implementation
├── order.rs                 # reference: visited-gene tracking pattern
├── pmx.rs                   # reference: gene-ID adjacency mapping pattern
└── ...

src/operations/mutation/
├── differential.rs          # NEW: DE-style differential mutation
├── gaussian.rs              # reference: Range<T> value clamping + GaussianConvertible
└── ...
```

### Pattern 1: ERX Adjacency List Construction

**What:** Build a `HashMap<i32, HashSet<i32>>` from both parents where each gene ID maps to the union of its left/right neighbors in both chromosomes.
**When to use:** Called once per ERX invocation; O(n) construction, O(1) neighbor lookup.

```rust
// Source: algorithm analysis + pmx.rs gene-ID patterns [VERIFIED: read pmx.rs]
fn build_adjacency_map<G: GeneT>(p1: &[G], p2: &[G]) -> HashMap<i32, HashSet<i32>> {
    let len = p1.len();
    let mut adj: HashMap<i32, HashSet<i32>> = HashMap::with_capacity(len);
    // Initialize all gene IDs from p1
    for g in p1 {
        adj.entry(g.id()).or_default();
    }
    // Add neighbors from both parents (circular: index wraps)
    for parent in [p1, p2] {
        for i in 0..len {
            let curr = parent[i].id();
            let left = parent[(i + len - 1) % len].id();
            let right = parent[(i + 1) % len].id();
            adj.entry(curr).or_default().insert(left);
            adj.entry(curr).or_default().insert(right);
        }
    }
    adj
}
```

### Pattern 2: ERX Child Construction

**What:** Grow a child by iteratively selecting the neighbor with fewest remaining (unvisited) neighbors; fall back to random unvisited gene if all neighbors are exhausted (D-06).
**When to use:** Called once per child in `erx()`.

```rust
// Source: Whitley et al. 1989 ERX algorithm + order.rs visited tracking [VERIFIED: read order.rs]
fn erx_build_child<G: GeneT>(
    start: i32,
    adj: &mut HashMap<i32, HashSet<i32>>,  // consumed (neighbor sets shrink as genes are visited)
    all_ids: &[i32],
    rng: &mut impl Rng,
) -> Vec<i32> {
    let n = all_ids.len();
    let mut child_ids = Vec::with_capacity(n);
    let mut visited: HashSet<i32> = HashSet::with_capacity(n);
    let mut current = start;

    for _ in 0..n {
        child_ids.push(current);
        visited.insert(current);

        // Remove current from all neighbor sets (it is now visited)
        // (Only need to update the adjacency sets of current's neighbors)

        // Find next gene: neighbor of current with fewest remaining neighbors
        let neighbors = adj.remove(&current).unwrap_or_default();
        let unvisited_neighbors: Vec<i32> = neighbors
            .into_iter()
            .filter(|id| !visited.contains(id))
            .collect();

        current = if unvisited_neighbors.is_empty() {
            // Fallback: pick any remaining unvisited gene (D-06)
            let remaining: Vec<i32> = all_ids
                .iter()
                .copied()
                .filter(|id| !visited.contains(id))
                .collect();
            if remaining.is_empty() { break; }
            remaining[rng.random_range(0..remaining.len())]
        } else {
            // Pick neighbor with fewest remaining unvisited neighbors
            *unvisited_neighbors
                .iter()
                .min_by_key(|id| adj.get(id).map(|s| s.iter().filter(|n| !visited.contains(n)).count()).unwrap_or(0))
                .unwrap()
        };
    }
    child_ids
}
```

### Pattern 3: Differential Mutation Engine Dispatch

**What:** In `parent_crossover()`, before calling `mutation::factory_with_params`, check if the mutation method is `Mutation::Differential` and branch to a population-aware function.
**When to use:** Applied to each offspring after crossover in `ga.rs::parent_crossover`.

```rust
// Source: ga.rs parent_crossover pattern [VERIFIED: read src/engines/ga.rs lines 1389-1406]
// BEFORE (existing):
if mutation_probability < effective_mutation_prob {
    mutation::factory_with_params(
        configuration.mutation_configuration.method,
        &mut child_1,
        configuration.mutation_configuration.step,
        configuration.mutation_configuration.sigma,
    )?;
}

// AFTER (with Differential branch):
if mutation_probability < effective_mutation_prob {
    if configuration.mutation_configuration.method == Mutation::Differential {
        let f = configuration.mutation_configuration.differential_f.unwrap_or(0.5);
        differential::differential_mutation(&mut child_1, chromosomes, f)?;
    } else {
        mutation::factory_with_params(
            configuration.mutation_configuration.method,
            &mut child_1,
            configuration.mutation_configuration.step,
            configuration.mutation_configuration.sigma,
        )?;
    }
}
```

### Pattern 4: Range<T> Downcast for Differential Mutation

**What:** Differential mutation is `Range<T>` only. Use `std::any::Any` downcasting — identical pattern to `try_sbx` / `try_blend_alpha` in `crossover.rs`.
**When to use:** Inside `differential_mutation()`.

```rust
// Source: src/operations/crossover.rs try_sbx pattern [VERIFIED: read crossover.rs lines 41-72]
macro_rules! try_type {
    ($t:ty) => {
        if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
            // ... operate on concrete type
            return Some(Ok(()));
        }
    };
}
try_type!(f64);
try_type!(f32);
try_type!(i32);
try_type!(i64);
// None → return GaError::MutationError (D-02)
```

### Pattern 5: MutationConfiguration Field Addition

**What:** Add `differential_f: Option<f64>` to `MutationConfiguration` and `with_differential_f(f: f64)` to the `MutationConfig` trait and its `Ga<U>` impl.
**When to use:** Follows the `polynomial_eta` / `non_uniform_b` pattern exactly.

```rust
// Source: src/configuration.rs MutationConfiguration [VERIFIED: read configuration.rs lines 140-181]
// Add to MutationConfiguration struct:
/// F scale factor for Differential mutation. Controls perturbation magnitude.
/// Typical range: 0.4–1.0. Default is 0.5.
pub differential_f: Option<f64>,

// Add to Default impl:
differential_f: None,

// Add to MutationConfig trait (src/traits/configuration.rs):
fn with_differential_f(self, f: f64) -> Self;

// Add to Ga<U> impl of MutationConfig:
fn with_differential_f(mut self, f: f64) -> Self {
    self.configuration.mutation_configuration.differential_f = Some(f);
    self
}
```

### Anti-Patterns to Avoid

- **Modifying `MutationOperator::mutate` signature:** D-01 is explicit — no trait changes. The Differential mutation arm in `Mutation::mutate()` should return `GaError::MutationError("...use engine dispatch...")` as a safety net (same approach as `Mutation::NonUniform`).
- **Using `dna().to_vec()` then `set_dna()` when `set_gene()` suffices:** For Differential mutation, all gene values must be computed before setting (because mutant depends on r1/r2/r3), so `set_dna(Cow::Owned(new_dna))` is correct here.
- **Modifying the adjacency map in-place during child construction without removing visited genes from neighbor sets:** This causes incorrect tie-breaking. Remove visited genes from all neighbor sets as the construction progresses.
- **Forgetting to add `Rejuvenate` / `ListValue` to the `factory_non_value` match arms:** The new `Differential` variant must be handled in `factory_non_value` in `mutation.rs` with an appropriate `GaError::MutationError`.
- **Skipping `MutationConfiguration.differential_f` in the serde `serde_ga_configuration_with_values` test:** The field must be present in the struct literal in `tests/observe/test_serde.rs` or the test will fail to compile after the field is added.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| RNG | Custom RNG | `crate::rng::make_rng()` | Project standard; supports seeding for reproducibility |
| f64 ↔ T conversion | Custom trait | `GaussianConvertible` from `gaussian.rs` | Already implements `f64`, `f32`, `i32`, `i64` — reuse for Differential gene arithmetic |
| HashMap/HashSet | Custom adjacency structure | `std::collections::{HashMap, HashSet}` | Already used in `pmx.rs` and `order.rs`; no dependency overhead |
| Downcast to Range<T> | Alternative type dispatch | `std::any::Any` + macro pattern from `crossover.rs` | Established pattern; handles all 4 supported numeric types |

**Key insight:** The `GaussianConvertible` trait in `gaussian.rs` provides exactly the `to_f64` / `from_f64` interface needed for Differential mutation's arithmetic. Reuse it — don't duplicate.

---

## Common Pitfalls

### Pitfall 1: Adjacency Map Neighbor Removal During ERX Construction

**What goes wrong:** If visited genes are not removed from neighbors' adjacency lists during construction, the tie-breaking heuristic (fewest remaining neighbors) gives wrong counts and produces suboptimal offspring.
**Why it happens:** The adjacency map is built once and then needs to be updated as genes are consumed.
**How to avoid:** As each gene is added to the child, iterate over its neighbors in the map and remove the consumed gene from their sets. (This is an O(degree) operation per step, acceptable for typical chromosome lengths.)
**Warning signs:** ERX children that don't minimize the tie-breaking count properly will still be valid permutations but won't preserve adjacency as well as expected.

### Pitfall 2: Differential Mutation Population Index vs. Offspring Target Index

**What goes wrong:** The three random indices r1, r2, r3 must be distinct from each other AND from the target chromosome's index in the population. If the target chromosome is a newly created child (not in `chromosomes`), use the parent index (the `key` variable in the `par_iter` closure) as the exclusion target.
**Why it happens:** The target in classic DE is the current individual. During the engine's crossover loop, child_1 corresponds to parent_1 at index `key`.
**How to avoid:** Pass `*key` (parent_1's index) as the exclusion index to `differential_mutation()`. Sample r1, r2, r3 from `0..chromosomes.len()` excluding `key`.
**Warning signs:** If this is wrong, occasionally two of the three basis vectors will be the same, producing weaker mutations.

### Pitfall 3: Serde Compilation Break on MutationConfiguration Struct Literals

**What goes wrong:** `tests/observe/test_serde.rs::serde_ga_configuration_with_values` constructs `MutationConfiguration { ... }` as a struct literal. Adding `differential_f: Option<f64>` to the struct makes the literal fail to compile (missing field).
**Why it happens:** Rust struct literals require all fields unless `..Default::default()` spread syntax is used.
**How to avoid:** Add `differential_f: None` to the struct literal in `serde_ga_configuration_with_values` at the same time as the field is added. This is the "CR-01 lesson" referenced in CONTEXT.md.
**Warning signs:** `cargo test --features serde` compilation error mentioning missing field in struct expression.

### Pitfall 4: ERX with Non-Permutation Chromosomes (Duplicate Gene IDs)

**What goes wrong:** ERX adjacency semantics are undefined for chromosomes with duplicate gene IDs. Without a guard, the algorithm will silently produce malformed output.
**Why it happens:** The adjacency map key is `gene.id()`. Duplicate IDs cause collisions and incorrect adjacency tracking.
**How to avoid:** Per D-08, perform an O(n) duplicate check using a `HashSet<i32>` on gene IDs of both parents before building the adjacency map. Return `GaError::CrossoverError` immediately if duplicates are detected.
**Warning signs:** ERX producing offspring shorter than expected, or panicking on `unwrap()` in the adjacency lookup.

### Pitfall 5: Population Size Guard for Differential Mutation

**What goes wrong:** Sampling 3 distinct indices r1, r2, r3 all different from `target_idx` requires at least 4 chromosomes in the population (target + 3 others).
**Why it happens:** When `chromosomes.len() < 4`, the deduplication loop either panics or never terminates.
**How to avoid:** Per D-03, check `chromosomes.len() < 4` at the top of `differential_mutation()` and return `GaError::MutationError` immediately.
**Warning signs:** Infinite loop in the random-sampling loop, or index panics on very small test populations.

---

## Code Examples

Verified patterns from the codebase:

### ERX: Permutation validation (reuse from pmx.rs)
```rust
// Source: src/operations/crossover/pmx.rs [VERIFIED: read pmx.rs]
if len < 2 {
    return Err(GaError::CrossoverError(
        "PMX crossover requires DNA of length >= 2".to_string(),
    ));
}
```

### ERX: Gene uniqueness check (D-08)
```rust
// Source: derived from order.rs segment_ids pattern [VERIFIED: read order.rs lines 73-74]
let ids_p1: std::collections::HashSet<i32> = parent_1.dna().iter().map(|g| g.id()).collect();
if ids_p1.len() != parent_1.dna().len() {
    return Err(GaError::CrossoverError(
        "EdgeRecombination crossover requires unique gene IDs in each parent (permutation chromosomes only)".to_string(),
    ));
}
```

### Differential: Range clamping (mirror of gaussian.rs)
```rust
// Source: src/operations/mutation/gaussian.rs lines 48-58 [VERIFIED: read gaussian.rs]
let (lo, hi) = gene.ranges[range_idx];
let lo_f64 = T::to_f64(lo);
let hi_f64 = T::to_f64(hi);
let new_val_f64 = (mutant_val).clamp(lo_f64, hi_f64);
gene.value = T::from_f64(new_val_f64);
```

### Differential: Three-index sampling
```rust
// Source: derived from existing RNG patterns [VERIFIED: make_rng() used in gaussian.rs]
let mut rng = crate::rng::make_rng();
let pop_len = chromosomes.len();
// chromosomes.len() >= 4 guaranteed by D-03 check above
let mut r1 = rng.random_range(0..pop_len);
while r1 == target_idx { r1 = rng.random_range(0..pop_len); }
let mut r2 = rng.random_range(0..pop_len);
while r2 == target_idx || r2 == r1 { r2 = rng.random_range(0..pop_len); }
let mut r3 = rng.random_range(0..pop_len);
while r3 == target_idx || r3 == r1 || r3 == r2 { r3 = rng.random_range(0..pop_len); }
```

### Mutation trait arm for Differential (safety net)
```rust
// Source: pattern from Mutation::NonUniform arm [VERIFIED: read mutation.rs lines 157-162]
Mutation::Differential => {
    return Err(GaError::MutationError(
        "Mutation::Differential requires population context. \
         It is applied automatically by the GA engine when configured — \
         do not call factory_with_params() directly.".to_string(),
    ));
}
```

### factory_non_value arm for Differential
```rust
// Source: pattern from factory_non_value other restricted arms [VERIFIED: read mutation.rs]
Mutation::Differential => Err(GaError::MutationError(
    "Mutation::Differential requires Range<T> chromosomes and population context. \
     Use Swap, Inversion, or Scramble instead.".to_string(),
)),
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| ERX: position-based crossover for permutations | ERX: adjacency-preserving via neighbor union map | Whitley 1989 | ERX produces offspring with fewer missing edges vs. OX/PMX for TSP |
| DE mutation: separate DE engine only | DE-style mutation in standard GA engine | This phase | Users get DE's perturbation strategy with GA's selection/survival operators |

**Deprecated/outdated:**

- None relevant to this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `GaussianConvertible` from `gaussian.rs` can be reused by `differential.rs` without visibility issues (currently `pub`) | Code Examples | If `pub(crate)` or module-gated, will need re-export or duplication — LOW risk, trait is `pub` |
| A2 | The `target_idx` to exclude in Differential sampling is `*key` (parent_1's population index) — child_1 maps to parent_1 | Architecture Patterns | If child_1 is treated as a new individual with no population index, the exclusion logic changes — LOW risk, child_1 is derived from parent_1 |

**Notes:** A1 is LOW risk — `GaussianConvertible` is declared `pub` in `gaussian.rs`. A2 is LOW risk — the intent of D-01 is confirmed by the engine dispatch design.

---

## Open Questions

1. **ERX: circular vs. linear adjacency**
   - What we know: The original Whitley 1989 ERX uses circular neighbors (index wraps around). Most TSP formulations treat chromosomes as tours (circular).
   - What's unclear: The CONTEXT.md description says "left and right neighbors in the chromosome order" — this is compatible with either circular or linear interpretation.
   - Recommendation: Use circular (wrap-around) adjacency — this is the canonical ERX definition and matches TSP tour semantics. Document the behavior in the function docstring.

2. **Differential mutation: does `parent_crossover()` need `chromosomes` to be `Arc<>`-wrapped for rayon safety?**
   - What we know: `chromosomes: &[U]` is already passed by reference to `parent_crossover()`. The `par_iter()` closure borrows it immutably.
   - What's unclear: Whether `differential_mutation(&mut child, chromosomes, f)` can hold `&[U]` while the rayon closure also holds `&[U]`.
   - Recommendation: Both are immutable borrows of `chromosomes` from the outer scope — should compile fine. No Arc needed.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`cargo test`) |
| Config file | `Cargo.toml` (no separate config) |
| Quick run command | `cargo test --test test_crossover_edge_recombination 2>/dev/null` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRS-01 | ERX produces 2 valid permutations preserving parent adjacency | unit | `cargo test --test test_crossover_edge_recombination` | ❌ Wave 0 |
| CRS-01 | ERX error on chromosome length < 2 | unit | `cargo test --test test_crossover_edge_recombination -- erx_error_too_short` | ❌ Wave 0 |
| CRS-01 | ERX error on duplicate gene IDs (D-08) | unit | `cargo test --test test_crossover_edge_recombination -- erx_error_duplicate_ids` | ❌ Wave 0 |
| CRS-01 | ERX fallback when adjacency list exhausted mid-construction (D-06) | unit | `cargo test --test test_crossover_edge_recombination -- erx_fallback_exhausted_neighbors` | ❌ Wave 0 |
| MUT-04 | Differential mutation produces mutant within gene ranges | unit | `cargo test --test test_mutation_differential` | ❌ Wave 0 |
| MUT-04 | Differential error when population < 4 (D-03) | unit | `cargo test --test test_mutation_differential -- differential_error_small_population` | ❌ Wave 0 |
| MUT-04 | Differential error on non-Range chromosome type (D-02) | unit | `cargo test --test test_mutation_differential -- differential_error_non_range` | ❌ Wave 0 |
| MUT-04 | Differential F parameter configurable via builder | unit | `cargo test --test test_mutation_differential -- differential_f_parameter` | ❌ Wave 0 |
| CRS-01 + MUT-04 | Serde round-trip for new enum variants | unit | `cargo test --features serde --test test_serde` | ✅ (must be updated) |

### Sampling Rate

- **Per task commit:** `cargo test 2>&1 | tail -5`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/operations/test_crossover_edge_recombination.rs` — covers CRS-01 ERX behavior + edge cases
- [ ] `tests/operations/test_mutation_differential.rs` — covers MUT-04 Differential behavior + error conditions
- [ ] Update `tests/observe/test_serde.rs` — add `Crossover::EdgeRecombination` to `serde_crossover_enum`, add `Mutation::Differential` to `serde_mutation_enum`, add `differential_f: None` to `serde_ga_configuration_with_values` struct literal

---

## Integration Checklist (all files that need changes)

This section summarizes every file the planner must assign tasks to. No file should be missed.

| File | Change Type | What Changes |
|------|-------------|--------------|
| `src/operations.rs` | Add variants | `Crossover::EdgeRecombination`, `Mutation::Differential` (with `#[cfg_attr(feature = "serde", ...)]`) |
| `src/operations/crossover.rs` | Add arm + module | `pub mod edge_recombination;`, match arm in `CrossoverOperator for Crossover` + `CrossoverOperator for CrossoverConfiguration` |
| `src/operations/crossover/edge_recombination.rs` | NEW | `pub fn erx<U: ChromosomeT>(p1, p2) -> Result<Vec<U>, GaError>` |
| `src/operations/mutation.rs` | Add arm + module | `pub mod differential;`, `Mutation::Differential` arm in `MutationOperator::mutate` (error arm, D-01), `Mutation::Differential` arm in `factory_non_value` (error arm) |
| `src/operations/mutation/differential.rs` | NEW | `pub fn differential_mutation<U: ChromosomeT>(individual, chromosomes, target_idx, f)` |
| `src/configuration.rs` | Add field | `differential_f: Option<f64>` in `MutationConfiguration` + `None` in `Default` |
| `src/traits/configuration.rs` | Add method | `fn with_differential_f(self, f: f64) -> Self;` in `MutationConfig` trait |
| `src/engines/ga.rs` | Add engine branch | Detect `Mutation::Differential` before `factory_with_params` in `parent_crossover()`; also add `with_differential_f` impl on `Ga<U>` |
| `tests/operations/test_crossover_edge_recombination.rs` | NEW | ERX tests |
| `tests/operations/test_mutation_differential.rs` | NEW | Differential mutation tests |
| `tests/observe/test_serde.rs` | Update | Add new variants to enum arrays; add `differential_f: None` to struct literal |

---

## Security Domain

This phase adds pure algorithmic operators with no I/O, no authentication, no external services, and no user input beyond f64 configuration parameters. ASVS categories V2, V3, V4, V6 do not apply. V5 input validation is addressed by D-03 (population size guard), D-07 (minimum chromosome length), and D-08 (gene uniqueness check) — all validated with explicit `GaError` returns.

---

## Sources

### Primary (HIGH confidence)

- `src/operations/crossover/order.rs` — visited-gene tracking with HashSet, Cow::Owned DNA pattern, crossover logging pattern [VERIFIED: read]
- `src/operations/crossover/pmx.rs` — gene-ID adjacency mapping with HashMap, permutation validation, two-child construction [VERIFIED: read]
- `src/operations/mutation/gaussian.rs` — Range<T> value clamping, GaussianConvertible trait, Box-Muller not needed here but clamping pattern is [VERIFIED: read]
- `src/operations/crossover.rs` — `try_sbx`/`try_blend_alpha` downcast pattern for Range<T> [VERIFIED: read]
- `src/operations/mutation.rs` — `factory_with_params`, `factory_non_value`, existing mutation arms, `NonUniform` error-arm pattern [VERIFIED: read]
- `src/engines/ga.rs` — `parent_crossover()` function, mutation dispatch at lines 1389–1406 [VERIFIED: read]
- `src/configuration.rs` — `MutationConfiguration` struct, existing `Optional<f64>` fields pattern [VERIFIED: read]
- `src/traits/configuration.rs` — `MutationConfig` trait methods [VERIFIED: read]
- `tests/observe/test_serde.rs` — serde round-trip test structure, struct literal that needs updating [VERIFIED: read]
- `src/types/chromosomes/range.rs` — `Range<T>` struct layout [VERIFIED: read]

### Secondary (MEDIUM confidence)

- Whitley, D. (1989). "Genetic algorithms and the traveling salesman problem." — ERX canonical algorithm (circular adjacency, smallest-neighbor tie-breaking, random fallback) [CITED: Whitley 1989, knowledge of standard ERX algorithm]
- Storn, R. & Price, K. (1997). "Differential Evolution — A Simple and Efficient Heuristic for Global Optimization over Continuous Spaces." — DE/rand/1 formula `x_r1 + F * (x_r2 - x_r3)` [CITED: Storn & Price 1997]

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all dependencies are already in the workspace; no new crates needed
- Architecture: HIGH — all integration points verified by reading the actual source files
- Algorithm correctness: MEDIUM — ERX and DE formulas are canonical; the circular-vs-linear adjacency question is LOW risk
- Pitfalls: HIGH — all identified from direct code inspection (serde struct literal, factory_non_value arms, adjacency removal, population size guard)

**Research date:** 2026-05-04
**Valid until:** 2026-06-04 (stable library — operator APIs are not changing)
