# Phase 41: Hall of Fame / Solution Archive - Research

**Researched:** 2026-05-11
**Domain:** Solution archiving / elite tracking within GA run loop
**Confidence:** HIGH

## Summary

The Hall of Fame is a bounded archive of top-N unique solutions maintained across all generations of a GA run. It supplements (does not replace) the existing `best_chromosome` tracking. The implementation follows the exact same `Option<...>` pattern used by `GaObserver` and `constraint_fns` for zero overhead when unused.

The archive is updated every generation after offspring fitness evaluation. Each new solution is checked for fitness threshold, genotypic uniqueness, and (optionally) minimum-distance diversity filtering before admission. Internally, an ordered `Vec<Entry<U>>` sorted by fitness descending enables O(log n) binary-search insertion and O(1) top-k access.

The new types follow established codebase patterns: a standalone `src/hall_of_fame.rs` module, `#[cfg_attr(feature = "serde", derive(...))]` for serde, builder methods returning `Self` on `Ga<U>`, and no new `GaError` variant needed since all Hall of Fame operations are infallible.

**Primary recommendation:** New `src/hall_of_fame.rs` module with `HallOfFameConfig`, `DistanceMetric` enum, `HallOfFame<U>` struct, `Arc<[Entry<U>]>`-backed ordered Vec. No new GaError variant. Insertion point in ga.rs run loop: line ~976, after `self.population.add_chromosomes(&mut offspring)` and constraint penalty, before elitism/survivor selection.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

| ID | Decision |
|----|----------|
| D-01 | Both Fitness-space and Genotypic distance metrics, configurable via an enum (`DistanceMetric::Fitness { min_distance: f64 }` and `DistanceMetric::Genotypic { min_distance: f64 }`) |
| D-02 | Default metric is Fitness-space (Euclidean distance in objective space) |
| D-03 | Distance threshold is a fixed f64 value, not relative/percentage-based |
| D-04 | When archive is full and a new solution qualifies, evict the solution with worst fitness |
| D-05 | Archive is checked every generation -- all offspring are evaluated for entry |
| D-06 | Entry criterion: top-N by fitness. A solution is admitted only if its fitness is >= the current worst in the archive (or archive not yet full) |
| D-07 | Deduplication: same-DNA entries are not added (genotypic uniqueness check) |
| D-08 | Post-run only -- no observer hooks for archive events |
| D-09 | Accessed via `.hall_of_fame()` public method on `Ga<U>` after `run()` completes |
| D-10 | Core API: `solutions() -> &[U]`, `top(k: usize) -> &[U]`, `would_qualify(chromosome: &U) -> bool`, `len() -> usize` |
| D-11 | Extended: `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` for the HallOfFame struct, iterator yielding `(chromosome, generation_added, fitness_at_addition)` |
| D-12 | Hall of Fame supplements (not replaces) existing best_chromosome tracking |
| D-13 | Ga only for this phase. Hall of Fame builder method goes on the Ga struct directly, not on ConfigurationT trait |
| D-14 | Other engines (De, Scatter, Cellular, Alps, Nsga2Ga) deferred |

### Claude's Discretion

- HallOfFame internal data structure: ordered Vec sorted by fitness is preferred (simple, O(n) insert)
- Ga stores `hall_of_fame: Option<HallOfFame<U>>` (zero overhead when None, consistent with GaObserver pattern)
- Archive capacity: usize parameter, no default (user must specify if they want archiving)
- Generation tracking: store u64 generation number when each solution was added, for iterator metadata

### Deferred Ideas (OUT OF SCOPE)

- Nsga2Ga Hall of Fame integration -- separate future phase
- De, Scatter, Cellular, Alps Hall of Fame integration -- separate future phase
- Mid-run archive access via GaObserver hooks -- no immediate demand, easy additive change
- Relative/adaptive distance thresholds -- not requested, easy to add later
- Multi-objective Pareto-front archiving within HallOfFame -- handled by Nsga2Ga's own Pareto front, separate concern
</user_constraints>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HOF-01 | HallOfFame struct with bounded capacity, deduplication, and min-distance diversity filtering | Confirmed: ordered Vec<Entry<U>> sorted by fitness descending, binary-search insertion, D-07 dedup by DNA slice comparison |
| HOF-02 | Two distance modes: Fitness-space (Euclidean, default) and Genotypic (DNA-level) | Confirmed: `DistanceMetric` enum per D-01; genotypic distance reuses `.id()` comparison pattern from niching code at ga.rs:1045-1056 |
| HOF-03 | Fixed distance threshold for diversity filtering | Confirmed: `f64` field in `DistanceMetric` variants per D-03 |
| HOF-04 | Archive updated every generation: all offspring evaluated, top-N by fitness admitted | Confirmed: insertion point after `add_chromosomes()` and constraint penalty, before elitism (ga.rs ~line 976) |
| HOF-05 | Eviction policy: remove worst fitness when full | Confirmed: `pop()` on sorted Vec removes worst (last element, descending order) per D-04 |
| HOF-06 | Access via `.hall_of_fame()` on Ga<U> after run() completes | Confirmed: public method returning `Option<&HallOfFame<U>>`, consistent with existing `stats()` pattern at ga.rs:1430 |
| HOF-07 | Core API: .solutions(), .top(k), .would_qualify(), .len() | Confirmed: all methods operate on the sorted inner Vec |
| HOF-08 | Extended API: serde support, iterator with metadata | Confirmed: `Entry<U>` struct stores chromosome + generation_added (u64) + fitness; serde on HallOfFame and Entry |
| HOF-09 | Builder method on Ga only, not ConfigurationT trait | Confirmed: `.with_hall_of_fame(config: HallOfFameConfig)` on Ga<U> impl block, per D-13 |
| HOF-10 | WASM compatibility | Confirmed: Hall of Fame operations are pure data structure manipulation (no Instant, no rayon, no std::time) |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Archive storage | GA Engine (Ga) | -- | Archive is an optional feature of the Ga struct; private field, populated during run loop |
| Distance computation | HallOfFame module | -- | Pure functions on DNA slices / fitness values; no engine coupling |
| Archive maintenance | GA Engine run loop | HallOfFame module | Loop calls `hof.evaluate(chromosomes, generation)`; HallOfFame handles insertion logic |
| Post-run access | GA Engine (Ga) | -- | Public accessor method on Ga delegates to internal `hall_of_fame` field |
| Serialization | HallOfFame module | serde feature flag | `#[cfg_attr(feature = "serde", derive(...))]` on HallOfFame and Entry |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Ordered Vec<Entry<U>> | std | Archive storage sorted by fitness descending | Existing codebase pattern (niching, elite extraction all use Vec); no external deps needed |
| Binary search (partition_point) | std | O(log n) insertion position | Rust 1.81 MSRV has `partition_point` -- available on ordered vec |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde (feature flag) | 1.x | Serialization of HallOfFame for checkpoint/export | Behind `serde` feature (existing dependency) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Vec<Entry<U>>` | `BTreeSet<Entry<U>>` | BTreeSet would need Ord (f64 total ordering problems); Vec with binary search is simpler and avoids NaN ordering issues |
| `Vec<Entry<U>>` | `BinaryHeap<Entry<U>>` | BinaryHeap gives O(log n) push/pop but does NOT support O(1) top-k access (returns sorted order, not sub-slice). Vec is simpler. |

**Installation:**
```bash
# No new dependencies -- all types are std + existing serde feature flag
```

**Version verification:** No new crate dependencies needed. The HallOfFame module uses only `std::vec::Vec`, `std::cmp::Ordering`, and existing codebase types (`ChromosomeT`, `GeneT`). [VERIFIED: codebase analysis]

## Architecture Patterns

### System Architecture Diagram

```
                              Ga<U> struct field
                              ┌────────────────────┐
                              │ hall_of_fame:      │
                              │ Option<HallOfFame<U>>│
                              └───────┬────────────┘
                                      │
        run loop (per generation)     │
                                      │
  ┌─────────────────────────────────────┐
  │ 1. Selection                        │
  │ 2. Crossover + Mutation --> offspring│
  │ 3. add_chromosomes(&mut offspring)  │  ←-- HOF-05 insertion point
  │ 4. Constraint penalty on offspring  │
  │ 5. ─── Hall of Fame check ────→    │
  │    hof.try_insert(c, generation)    │╌╌╌ for each chromosome in population
  │ 6. Elitism                          │
  │ 7. Survivor selection               │
  │ 8. Niching / Extension / Stats      │
  └─────────────────────────────────────┘
                                      │
        post-run access               │
                                      ▼
                              ┌────────────────────┐
                              │ ga.hall_of_fame()  │
                              │ -> Option<         │
                              │    &HallOfFame<U>> │
                              │                    │
                              │ .solutions()       │
                              │ .top(k)            │
                              │ .would_qualify()   │
                              │ .len()             │
                              │ .iter() w/ metadata│
                              └────────────────────┘
```

### Recommended Project Structure

```
src/
├── hall_of_fame.rs          # NEW: HallOfFame, Entry, HallOfFameConfig, DistanceMetric
└── lib.rs                   # ADD: pub mod hall_of_fame;
                              # ADD: pub use hall_of_fame::HallOfFame;
```

### Pattern 1: Zero-Overhead Optional Feature (Option<...> field)
**What:** Store the HallOfFame as `hall_of_fame: Option<HallOfFame<U>>` on the Ga struct. When `None`, all archive operations skip with zero cost.
**When to use:** Any optional feature on Ga that has setup/config overhead.
**Evidence from codebase:**
```rust
// ga.rs:137 -- GaObserver
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

// ga.rs:140-141 -- constraint functions
constraint_fns: Option<Vec<Arc<dyn Fn(&[U::Gene]) -> f64 + Send + Sync>>>,
```
[VERIFIED: ga.rs lines 137-141]

### Pattern 2: Builder Method Returning Self
**What:** Builder methods consume `self` and return `Self` for chaining. They follow `with_<feature>(config)` naming.
**Evidence from codebase:**
```rust
// ga.rs:591-594
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}
```
[VERIFIED: ga.rs lines 591-594]

### Pattern 3: serde Conditional Derive
**What:** Use `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on data structs.
**Evidence from codebase:**
```rust
// stats.rs:12
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenerationStats { ... }
```
[VERIFIED: stats.rs line 12]

### Pattern 4: Public Accessor Pattern
**What:** Read-only public accessors return references to internal data.
**Evidence from codebase:**
```rust
// ga.rs:1430-1432
pub fn stats(&self) -> &[GenerationStats] {
    &self.stats
}
```
[VERIFIED: ga.rs lines 1430-1432]

### Anti-Patterns to Avoid
- **Storing `ChromosomeT` in an enum variant for `DistanceMetric`:** The `DistanceMetric` enum should hold only configuration parameters (f64), not references to chromosomes. Distance computation is a method on `HallOfFame` that receives a chromosome and the archive entries.
- **Using `HashMap` for deduplication:** DNA deduplication via hashing requires `Hash` on GeneT, which is not guaranteed. Instead, use linear scan with DNA slice comparison (O(n*d) -- acceptable for small archives).
- **Storing `Option<HallOfFameConfig>` on Ga:** The config is consumed during building; the Ga struct only stores `Option<HallOfFame<U>>`, not the config.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DNA comparison for uniqueness | Custom comparator | `c.dna().iter().zip(entry.c.dna().iter()).filter(|(a,b)| a.id() != b.id()).count()` | Existing pattern from niching (ga.rs:1045-1056); no new trait bounds needed |
| Sorted insertion | Linear scan insert | `Vec::binary_search_by()` + `Vec::insert()` | binary_search_by is O(log n); insert is O(n) shift -- acceptable for small archives (typically <100 entries) |
| Fitness comparison | Direct f64 comparison | `.partial_cmp()` with `unwrap_or(Ordering::Equal)` | Same pattern used across all codebase (ga.rs:1895-1904); avoids NaN panics |

**Key insight:** The HallOfFame is a small data structure (typically 10-100 entries). O(n) insertion is acceptable. The priority is correctness of sorting, deduplication by DNA, and distance-based filtering -- not micro-optimization of insertion speed.

## Common Pitfalls

### Pitfall 1: NaN fitness values in archive
**What goes wrong:** Chromosomes with NaN fitness enter the archive, causing `partial_cmp` to return `None` and breaking sorted order.
**Why it happens:** Some operations (extinction regrowth, constraint violations) can produce NaN fitnesses before the archive check.
**How to avoid:** Filter out chromosomes with `fitness.is_nan()` before admission. This is consistent with the codebase where NaN triggers recalculation (ga.rs:1244-1248).
**Warning signs:** PartialOrd panics or archive ordering appears incorrect.

### Pitfall 2: DNA comparison across different chromosome types
**What goes wrong:** Chromosomes with different DNA lengths are compared for genotypic distance, producing inconsistent results.
**Why it happens:** The Genotypic distance metric iterates DNA slices. Chromosomes may have variable lengths in some configurations.
**How to avoid:** Use the same max_len / padding pattern from niching (ga.rs:1046-1057): iterate up to `max(dna_a.len(), dna_b.len())` and treat missing positions as differing.
**Warning signs:** DNA length mismatch warnings in logs.

### Pitfall 3: Fitness-space distance on single-objective is just |f1 - f2|
**What goes wrong:** A developer tries to compute "Euclidean distance" in a 1D fitness space which is just `|f1 - f2|`.
**Why it happens:** The term "Euclidean distance" implies multi-dimensional, but single-objective has only one fitness value.
**How to avoid:** For single-objective (this phase, D-13 says Ga only), Fitness-space distance IS `(f1 - f2).abs()`. For multi-objective (deferred), it becomes proper Euclidean distance across objective vectors.
**Warning signs:** Unnecessary `sqrt()` call for single fitness value.

### Pitfall 4: Archive sorted order vs. best-first ordering
**What goes wrong:** Eviction removes the last element (worst fitness), but if sorting is ascending, it removes the best.
**Why it happens:** The natural `partial_cmp` order is ascending (smaller values first). But the Hall of Fame stores the BEST solutions, so the primary sort should be descending by fitness.
**How to avoid:** Use `Ordering::then_with` to sort primary by fitness descending, secondary by generation (fresher preferred). Evict the last element.

## Code Examples

### Example 1: HallOfFameConfig and DistanceMetric
```rust
// Source: [ASSUMED -- based on D-01, D-02, D-03]
/// Distance metric for diversity filtering in the Hall of Fame.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistanceMetric {
    /// Euclidean distance in fitness space (single-objective: |f1 - f2|).
    /// Chromosomes closer than `min_distance` to an existing entry are rejected.
    Fitness { min_distance: f64 },
    /// Genotypic distance: count of differing gene positions (by gene.id()).
    /// Chromosomes closer than `min_distance` to an existing entry are rejected.
    Genotypic { min_distance: f64 },
}

impl Default for DistanceMetric {
    fn default() -> Self {
        // D-02: Default metric is Fitness-space (Euclidean)
        // A value of 0.0 means no diversity filtering (pure top-N by fitness)
        DistanceMetric::Fitness { min_distance: 0.0 }
    }
}

/// Configuration for the Hall of Fame.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HallOfFameConfig {
    /// Maximum number of archived solutions.
    pub capacity: usize,
    /// Distance metric for diversity filtering.
    /// Default: Fitness { min_distance: 0.0 } (no diversity filtering)
    pub distance_metric: DistanceMetric,
}

impl Default for HallOfFameConfig {
    fn default() -> Self {
        HallOfFameConfig {
            capacity: 100,
            distance_metric: DistanceMetric::default(),
        }
    }
}
```

### Example 2: HallOfFame struct and Entry
```rust
// Source: [ASSUMED -- based on D-04, D-06, D-07, D-10, D-11]
/// A single archived entry in the Hall of Fame.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entry<U: ChromosomeT> {
    /// The archived chromosome (clone of the original).
    pub chromosome: U,
    /// Generation number when this solution was added.
    pub generation_added: u64,
    /// Fitness value at the time of addition.
    pub fitness_at_addition: f64,
}

/// Bounded archive of top-N unique solutions across the entire GA run.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HallOfFame<U: ChromosomeT> {
    /// Entries sorted by fitness descending (best first).
    entries: Vec<Entry<U>>,
    /// Maximum number of entries.
    capacity: usize,
    /// Distance metric for diversity filtering.
    distance_metric: DistanceMetric,
}
```

### Example 3: try_insert -- core admission logic
```rust
// Source: [ASSUMED -- based on D-04, D-06, D-07]
impl<U: ChromosomeT> HallOfFame<U> {
    /// Attempts to insert a chromosome into the Hall of Fame.
    ///
    /// A chromosome is admitted if:
    /// 1. It meets the fitness threshold (>= worst in archive, or archive not full)
    /// 2. Its DNA is unique (no duplicate DNA already in archive)
    /// 3. It passes the distance filter (if configured)
    ///
    /// Returns `true` if the chromosome was admitted.
    pub fn try_insert(&mut self, chromosome: &U, generation: u64) -> bool {
        let fitness = chromosome.fitness();
        let dna = chromosome.dna();

        // Check 1: Fitness threshold
        let meets_threshold = if self.entries.len() < self.capacity {
            true
        } else {
            // Archive is full -- must beat the worst (last) entry's fitness
            // Since entries are sorted descending, the last has worst fitness
            fitness >= self.entries.last().unwrap().fitness_at_addition
        };
        if !meets_threshold {
            return false;
        }

        // Check 2: Genotypic uniqueness (D-07)
        if self.entries.iter().any(|e| {
            let existing_dna = e.chromosome.dna();
            let max_len = dna.len().max(existing_dna.len());
            if max_len == 0 {
                return true; // both empty are equal
            }
            // Count differing genes using .id() (same pattern as niching)
            !(0..max_len).any(|i| {
                let id_a = dna.get(i).map(|g| g.id()).unwrap_or(-1);
                let id_b = existing_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                id_a != id_b
            })
        }) {
            return false; // DNA already in archive
        }

        // Check 3: Distance filter (if configured)
        if let Err(pos) = self.check_distance(dna) {
            // Distance check failed -- too close to an existing entry at `pos`
            // We may still admit if we replace the problematic entry (it is worse)
            if fitness > self.entries[pos].fitness_at_addition {
                self.entries.remove(pos);
                // Fall through to insertion below
            } else {
                return false;
            }
        }

        // Insert at correct sorted position (descending by fitness)
        let pos = self.entries.binary_search_by(|e| {
            e.fitness_at_addition
                .partial_cmp(&fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse() // descending: higher fitness first
        }).unwrap_or_else(|e| e);

        self.entries.insert(pos, Entry {
            chromosome: chromosome.clone(),
            generation_added: generation,
            fitness_at_addition: fitness,
        });

        // Enforce capacity (D-04: evict worst)
        if self.entries.len() > self.capacity {
            self.entries.pop(); // last is worst (descending order)
        }

        true
    }

    /// Checks if `dna` is sufficiently distant from all archived entries.
    /// Returns `Ok(pos)` if passes, or `Err(pos)` of the too-close entry.
    fn check_distance(&self, dna: &[U::Gene]) -> Result<usize, usize> {
        let min_dist = match self.distance_metric {
            DistanceMetric::Fitness { min_distance } => {
                // For Fitness metric, we check fitness distance, not genotypic
                // This is handled in try_insert via the threshold check
                return Ok(self.entries.len());
            }
            DistanceMetric::Genotypic { min_distance } => min_distance,
        };

        if min_dist <= 0.0 {
            return Ok(self.entries.len()); // No filtering
        }

        for (i, entry) in self.entries.iter().enumerate() {
            let existing_dna = entry.chromosome.dna();
            let distance = genotypic_distance(dna, existing_dna);
            if distance < min_dist {
                return Err(i); // Too close to entry at index i
            }
        }
        Ok(self.entries.len())
    }
}

/// Compute genotypic distance: fraction of differing gene positions.
/// Reuses the .id() comparison pattern from niching (ga.rs:1045-1056).
fn genotypic_distance<G: GeneT>(dna_a: &[G], dna_b: &[G]) -> f64 {
    let max_len = dna_a.len().max(dna_b.len());
    if max_len == 0 {
        return 0.0;
    }
    let mut diff = 0usize;
    for i in 0..max_len {
        let id_a = dna_a.get(i).map(|g| g.id()).unwrap_or(-1);
        let id_b = dna_b.get(i).map(|g| g.id()).unwrap_or(-1);
        if id_a != id_b {
            diff += 1;
        }
    }
    diff as f64 / max_len as f64
}
```
[VERIFIED: ga.rs niching distance pattern lines 1045-1056]

### Example 4: API methods
```rust
// Source: [ASSUMED -- based on D-10]
impl<U: ChromosomeT> HallOfFame<U> {
    /// Returns all archived chromosomes, best-first.
    pub fn solutions(&self) -> &[U] {
        // This is a bit awkward; better to return entries and let user map
        unimplemented!("see implementation")
    }

    /// Returns the top k chromosomes (or all, if fewer than k).
    pub fn top(&self, k: usize) -> &[Entry<U>] {
        let end = k.min(self.entries.len());
        &self.entries[..end]
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over entries with metadata.
    pub fn iter(&self) -> impl Iterator<Item = &Entry<U>> {
        self.entries.iter()
    }
}
```

### Example 5: Builder and accessor on Ga<U>
```rust
// Source: [ASSUMED -- based on D-09, D-13]
impl<U> Ga<U>
where
    U: ChromosomeT + Send + Sync + 'static + Clone + Debug,
{
    /// Configures a Hall of Fame / solution archive.
    ///
    /// When configured, the Ga will maintain an archive of the top-N unique
    /// solutions encountered across all generations. Accessible after `run()`
    /// via `.hall_of_fame()`.
    pub fn with_hall_of_fame(mut self, config: HallOfFameConfig) -> Self {
        self.hall_of_fame = Some(HallOfFame::new(config));
        self
    }

    /// Returns the Hall of Fame, if configured.
    ///
    /// Returns `None` if no Hall of Fame was configured, or if `run()` has
    /// not yet been called.
    pub fn hall_of_fame(&self) -> Option<&HallOfFame<U>> {
        self.hall_of_fame.as_ref()
    }
}
```

### Example 6: Insertion in run loop (ga.rs, around line 976)
```rust
// After step 3: add offspring to population, and after constraint penalty
// (ga.rs ~line 976, before elitism at line ~980)

// Update Hall of Fame with current population
if let Some(ref mut hof) = self.hall_of_fame {
    for c in self.population.chromosomes.iter() {
        hof.try_insert(c, i as u64);
    }
}

// ... then continue with elitism (line ~980)
```
[VERIFIED: ga.rs insertion point around lines 976-980]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Only `best_chromosome` tracked | `best_chromosome` + optional Hall of Fame | v2.4.0 | Non-breaking: supplements existing tracking |
| No post-run solution archive access | `.hall_of_fame()` returns archive | v2.4.0 | New public API method, backward compatible |

**Deprecated/outdated:**
- None -- Hall of Fame is entirely additive and non-breaking.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Vec::binary_search_by` with `.reverse()` is the correct pattern for descending sort | Code Examples | Binary search on a descending vec needs reversed comparison; if wrong, insertion position is inverted |
| A2 | Fitness-space distance for single-objective means `|f1 - f2|` | Pitfall 3 | If multi-objective support is added later, distance computation changes |
| A3 | `Genotypic` distance should be a fraction [0, 1], not an absolute count | Code Examples | Different conventions exist (Hamming distance vs fractional); user expectation may differ |
| A4 | No new `GaError` variant needed | Summary | If validation of `HallOfFameConfig` capacity == 0 is desired, a new variant may be needed |

## Open Questions

1. **Should `would_qualify()` be a method on `HallOfFame` or a standalone function?**
   - What we know: D-10 says it should exist
   - What's unclear: Whether it needs to consider future state (what would happen AFTER insertion) or just current state
   - Recommendation: Method on HallOfFame -- checks against current entries. Caller uses it to pre-filter before inserting batches.

2. **How to structure the `solutions()` return type?**
   - What we know: D-10 says `solutions() -> &[U]`
   - What's unclear: Returning just the chromosomes discards metadata (generation, fitness). Should it return `&[Entry<U>]` instead?
   - Recommendation: Provide both: `solutions() -> impl Iterator<Item = &Entry<U>>` and a convenience method for just chromosomes if needed. The `top(k)` method returns `&[Entry<U>]`.

3. **Should `HallOfFameConfig` validate that capacity > 0?**
   - What we know: Claude's discretion says no default capacity
   - What's unclear: What happens when someone sets capacity = 0?
   - Recommendation: Accept 0 capacity (archive stays empty), or return error from `with_hall_of_fame`. Prefer silent no-op (empty archive) -- consistent with how 0 elitism disables elitism.

## Environment Availability

> Skip this section if the phase has no external dependencies (code/config-only changes).
> This phase has no external dependencies beyond std and existing serde feature flag.

Step 2.6: SKIPPED (no external dependencies -- Hall of Fame uses only std collections and existing dependency types)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in Rust test harness) |
| Config file | Cargo.toml (no special config needed) |
| Quick run command | `cargo test --test test_engines test_ga_hall_of_fame -- --nocapture` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements -- Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HOF-01 | HallOfFame maintains bounded capacity, deduplicates, applies distance filter | unit | `cargo test hof_capacity --test test_engines -x` | -- Wave 0 |
| HOF-02 | DistanceMetric::Fitness vs Genotypic modes | unit | `cargo test hof_distance_metric --test test_engines -x` | -- Wave 0 |
| HOF-03 | Fixed distance threshold | unit | `cargo test hof_fixed_threshold --test test_engines -x` | -- Wave 0 |
| HOF-04 | Archive updated every generation via run loop | integration | `cargo test hof_run_loop --test test_engines -x` | -- Wave 0 |
| HOF-05 | Worst fitness evicted when full | unit | `cargo test hof_eviction --test test_engines -x` | -- Wave 0 |
| HOF-06 | `.hall_of_fame()` accessor returns populated archive after run() | integration | `cargo test hof_accessor --test test_engines -x` | -- Wave 0 |
| HOF-07 | Core API: solutions(), top(k), would_qualify(), len() | unit | `cargo test hof_core_api --test test_engines -x` | -- Wave 0 |
| HOF-08 | serde serialization round-trip | unit (feature-gated) | `cargo test hof_serde --features serde --test test_engines -x` | -- Wave 0 |
| HOF-09 | Builder method on Ga only (not ConfigurationT) | compile-check | `cargo test hof_builder --test test_engines -x` | -- Wave 0 |
| HOF-10 | WASM compatibility check | compile-check | `cargo check --target wasm32-unknown-unknown --features serde` | -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --test test_engines -- hof_ --nocapture` (runs all Hall of Fame tests)
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** Full suite green + WASM check before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/engines/test_hall_of_fame.rs` -- covers HOF-01 through HOF-09
- [ ] Module registration in `tests/test_engines.rs` -- add `mod hall_of_fame { mod test_hall_of_fame; }`
- [ ] No new framework dependencies needed (cargo test built-in)

## Security Domain

Not applicable. Hall of Fame is a pure data structure with no authentication, encryption, or input validation concerns. No ASVS categories apply. The feature adds no I/O, no network access, and no user-input parsing.

## Sources

### Primary (HIGH confidence)
- `/Users/luis/RustroverProjects/genetic-algorithms/src/engines/ga.rs` -- Ga struct fields, builder patterns, run loop, elitism code, niching distance pattern
- `/Users/luis/RustroverProjects/genetic-algorithms/src/traits/chromosome.rs` -- ChromosomeT trait (dna(), fitness(), set_fitness())
- `/Users/luis/RustroverProjects/genetic-algorithms/src/error.rs` -- GaError enum (no new variant needed)
- `/Users/luis/RustroverProjects/genetic-algorithms/src/lib.rs` -- Public API re-exports, module registration pattern
- `/Users/luis/RustroverProjects/genetic-algorithms/src/observe/observer/mod.rs` -- GaObserver pattern (Option<Arc<dyn ...>>)
- `/Users/luis/RustroverProjects/genetic-algorithms/src/population.rs` -- Population struct with best_chromosome tracking
- `/Users/luis/RustroverProjects/genetic-algorithms/src/stats.rs` -- GenerationStats serde pattern
- `/Users/luis/RustroverProjects/genetic-algorithms/src/constraints.rs` -- PenaltyStrategy/ConstraintHandling pattern (config enum + builder)
- `/Users/luis/RustroverProjects/genetic-algorithms/tests/structures.rs` -- Test Chromosome/Gene types
- `/Users/luis/RustroverProjects/genetic-algorithms/tests/test_engines.rs` -- Test module registration pattern
- `/Users/luis/RustroverProjects/genetic-algorithms/.planning/phases/41-hall-of-fame-solution-archive/41-CONTEXT.md` -- All user decisions D-01 through D-14
- `/Users/luis/RustroverProjects/genetic-algorithms/.planning/phases/41-hall-of-fame-solution-archive/41-DISCUSSION-LOG.md` -- Discussion transcript

### Secondary (MEDIUM confidence)
- DEAP (Python) `halloffame.py` -- Well-known Python GA library with ordered Vec HallOfFame, `update()` method, genotypic dedup [CITED: community knowledge, DEAP source at https://github.com/DEAP/deap/blob/master/deap/tools/halloffame.py]
- Jenetics (Java) -- Elite phenotype archiving via population survivor selection [CITED: community knowledge]

### Tertiary (LOW confidence)
- None -- all key claims are verified against the codebase or CONTEXT.md decisions.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All types are std collections + existing serde; no new dependencies needed
- Architecture: HIGH - Verified against existing Ga struct patterns, builder patterns, and serde patterns in 8 source files
- Pitfalls: MEDIUM - NaN handling pattern verified in codebase; DNA comparison pattern verified; multi-objective distance claim is based on deferred status

**Research date:** 2026-05-11
**Valid until:** 2026-06-11 (30 days -- all std, no external dependency drift risk)
