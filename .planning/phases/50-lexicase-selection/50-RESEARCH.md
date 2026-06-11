# Phase 50: Lexicase Selection — Research

**Researched:** 2026-05-22
**Domain:** Rust trait extension, selection operator implementation, lexicase / epsilon-lexicase algorithm
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `MultiCaseFitness: ChromosomeT` provides exactly two methods: `case_fitness() -> &[f64]` and `set_case_fitness(Vec<f64>)`. Supertrait of `ChromosomeT` (not `LinearChromosome`) so `TreeChromosome` (Phase 53) can reuse it.
- **D-02:** Case fitness is populated inside the user's `calculate_fitness()` — they call `self.set_case_fitness(vec![...])` alongside setting scalar fitness. No second callback field.
- **D-03:** Case count derived from `chromosomes[0].case_fitness().len()` at runtime — no `num_cases` parameter on the selection function.
- **D-04:** After a candidate is selected, the lexicase function calls `chromosome.set_fitness(mean_case_score)` on **every individual in the population** before returning pairs. Syncs scalar fitness for survivor and stopping criteria.
- **D-05:** Add `Selection::Lexicase` and `Selection::EpsilonLexicase` to the `Selection` enum in `src/operations.rs`. Both variants carry no data — enum stays `Copy`.
- **D-06:** Separate `selection::factory_lexicase<U: ChromosomeT + MultiCaseFitness>()` handles lexicase dispatch. The standard `selection::factory<U: ChromosomeT>()` returns `GaError::ConfigurationError` for Lexicase/EpsilonLexicase variants.
- **D-07:** In `ga.rs` `run()`, per-generation selection adds if/else: `if Lexicase/EpsilonLexicase { factory_lexicase() } else { factory() }`. Dispatch per-generation, not at build time.
- **D-08:** `SelectionOperator` trait impl for `Lexicase`/`EpsilonLexicase` panics with clear message pointing to `factory_lexicase`. Guards island-model and NSGA-II paths.
- **D-09:** `epsilon: f64` added to `SelectionConfiguration` with sentinel default `0.0` (= use dynamic MAD). User calls `.with_epsilon_lexicase(0.05)` for a fixed value.
- **D-10:** When `epsilon == 0.0`, epsilon-lexicase computes MAD of case scores across the population per case, once before shuffling. O(n × num_cases) per selection event.
- **D-11:** Epsilon is a single scalar applied uniformly across all test cases. No per-case epsilon vector.
- **D-12:** New files: `src/operations/selection/lexicase.rs`. Both functions pub-exported from `src/operations/selection.rs`.
- **D-13:** `MultiCaseFitness` trait: `src/traits/multi_case_fitness.rs`, re-exported via `src/traits.rs`.

### Claude's Discretion

None — discussion stayed within phase scope.

### Deferred Ideas (OUT OF SCOPE)

None stated.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SEL-02 | User can configure `LexicaseSelection` on any chromosome implementing `MultiCaseFitness: ChromosomeT` — shuffles test cases randomly per event, filters to elites case by case; scalar `fitness()` set to mean case score | Algorithm verified via official lexicase.ai canonical description; implementation pattern from `clearing.rs` analog |
| SEL-03 | User can configure epsilon-lexicase for continuous-valued case scores — filter keeps all within epsilon of best per case; epsilon user-configurable with MAD default | MAD-epsilon procedure confirmed from La Cava et al. 2016 GECCO canonical source |
| TRAITS-01 | User can implement `MultiCaseFitness: ChromosomeT` with `case_fitness() -> &[f64]` and `set_case_fitness(Vec<f64>)` — enables `LexicaseSelection`; compatible with `TreeChromosome` | Trait structure analysis complete; no existing trait conflicts found |
</phase_requirements>

---

## Summary

Phase 50 adds two parent selection operators — `LexicaseSelection` and `EpsilonLexicaseSelection` — backed by a new opt-in `MultiCaseFitness: ChromosomeT` trait. The work spans four modules: a new trait file, a new selection implementation file, enum additions, and a dispatch branch in `ga.rs run()`.

The codebase already has the exact structural analog needed: `clearing.rs` + `factory()` dispatch in `selection.rs` demonstrates the pattern of an operator that requires operator-specific config params, a `factory_special<U: Bound>()` wrapper, and a panic-guard arm in the `SelectionOperator` trait impl. Every lexicase file follows this pattern precisely.

The lexicase algorithm itself is well-understood and academically settled. The MAD-epsilon default is the canonical choice from La Cava et al. 2016. The only non-trivial implementation challenge is computing a per-case MAD efficiently (O(n × c) sort, where n = population, c = num_cases) and ensuring correct use of the per-generation Fisher-Yates shuffle via `crate::rng::make_rng()` rather than `par_iter` (WASM compatibility is mandatory).

**Primary recommendation:** Follow the `clearing.rs` + `selection::factory()` structural pattern exactly. Implement `lexicase_selection` and `epsilon_lexicase_selection` as free functions in `src/operations/selection/lexicase.rs`, add the two enum variants, wire the `factory_lexicase` dispatch in `ga.rs`, and publish `MultiCaseFitness` from `src/traits/multi_case_fitness.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `MultiCaseFitness` trait definition | Traits layer | — | Opt-in contract; no data, no engine coupling |
| Case-fitness population | User's chromosome `calculate_fitness()` | — | Consistent with all existing chromosomes — user owns fitness computation |
| Lexicase/epsilon-lexicase algorithm | Selection operator (src/operations/selection/) | — | Pure selection concern; produces parent pairs from population |
| Per-case MAD computation | Selection operator (lexicase.rs) | — | Pre-selection setup step; belongs with the operator that needs it |
| Scalar fitness sync (mean case score) | Selection operator (lexicase.rs, post-selection) | — | D-04 explicitly scopes this to the selection call |
| `Selection` enum variants | src/operations.rs | — | Enum + factory pattern; all operator variants live here |
| `factory_lexicase` dispatch | src/operations/selection.rs | — | Mirror of `factory()` but with `MultiCaseFitness` bound |
| `SelectionOperator` trait panic arms | src/traits/operators.rs | — | Guards island/NSGA-II paths from silent misbehavior |
| Per-generation dispatch branch | src/engines/ga.rs `run()` | — | Engine dispatch is the only place that can check the variant and branch |
| `epsilon: f64` config field | src/configuration.rs `SelectionConfiguration` | — | Same slot as `boltzmann_temperature`, `niche_radius` |
| Builder method `with_epsilon_lexicase` | src/traits/configuration.rs `SelectionConfig` | src/engines/ga.rs | Fluent API; same pattern as `with_niche_radius` |

---

## Standard Stack

### Core — No new crate dependencies

Phase 50 introduces zero new external dependencies. All needs are met by existing crate internals.

| Asset | Location | Purpose | Already Present |
|-------|----------|---------|-----------------|
| `crate::rng::make_rng()` | `src/rng.rs` | Per-selection Fisher-Yates shuffle of case indices | Yes |
| `rand::Rng` | Cargo.toml `rand = "0.9.2"` | `rng.random_range(0..n)`, `rng.random_range(0..=i)` | Yes |
| `log::debug!` / `log::trace!` | `log = "0.4.22"` | `target="selection_events"` logging | Yes |
| `crate::error::GaError` | `src/error.rs` | `SelectionError`, `ConfigurationError` variants | Yes |
| `crate::traits::ChromosomeT` | `src/traits/chromosome.rs` | Supertrait for `MultiCaseFitness` | Yes |

`[VERIFIED: codebase]` — All assets confirmed present in the working tree.

### Package Legitimacy Audit

> Phase 50 installs **no new external packages**. This section is N/A.

No new crate dependencies are added. Existing `rand 0.9.2` and `log 0.4.22` are already used throughout the codebase.

---

## Architecture Patterns

### System Architecture Diagram

```
User's calculate_fitness()
  └─ self.set_case_fitness(vec![s1, s2, ..., sk])   ← populates per-case scores
  └─ self.set_fitness(mean_or_custom_scalar)          ← also sets scalar (or lexicase sync overwrites)

ga.rs run() per generation
  ├─ if selection.method == Lexicase | EpsilonLexicase
  │    └─ selection::factory_lexicase(&chromosomes, config, threads)
  │         ├─ guard: len >= 2, no NaN in case_fitness vectors
  │         ├─ [EpsilonLexicase, epsilon==0.0] compute per-case MAD over population
  │         ├─ for each couple needed:
  │         │    ├─ pool = all chromosome indices (0..n)
  │         │    ├─ shuffle case indices via Fisher-Yates (make_rng())
  │         │    ├─ for each case in shuffled order:
  │         │    │    ├─ best = max case_fitness[case] in pool
  │         │    │    ├─ keep pool where case_fitness[case] >= best (Lexicase)
  │         │    │    │  OR where case_fitness[case] >= best - epsilon (EpsilonLexicase)
  │         │    │    └─ if pool.len() == 1: break
  │         │    └─ if pool.len() > 1: pick randomly from pool
  │         ├─ sync scalar fitness: for each c in chromosomes { c.set_fitness(mean(case_fitness)) }
  │         └─ return Vec<(usize, usize)>
  └─ else
       └─ selection::factory(&chromosomes, config, threads)   ← existing path unchanged
```

### Recommended Project Structure (additions only)

```
src/
├── traits/
│   └── multi_case_fitness.rs        ← NEW: MultiCaseFitness trait definition
├── operations/
│   └── selection/
│       └── lexicase.rs              ← NEW: lexicase_selection + epsilon_lexicase_selection
```

Modified files:
- `src/traits.rs` — add `pub mod multi_case_fitness` + `pub use multi_case_fitness::MultiCaseFitness`
- `src/operations.rs` — add `Selection::Lexicase` and `Selection::EpsilonLexicase` variants
- `src/operations/selection.rs` — add `pub mod lexicase`, re-exports, `factory_lexicase`, and arm in `SelectionOperator` impl
- `src/configuration.rs` — add `epsilon: f64` field to `SelectionConfiguration`
- `src/traits/configuration.rs` — add `with_epsilon_lexicase(f64)` to `SelectionConfig` trait
- `src/engines/ga.rs` — add `with_epsilon_lexicase` impl + `factory_lexicase` dispatch branch in `run()`
- `src/lib.rs` — add `MultiCaseFitness` to public re-exports

### Pattern 1: MultiCaseFitness Trait Definition

**What:** New opt-in supertrait of `ChromosomeT` with two methods — getter and setter for the per-case score vector.
**When to use:** Users add this alongside `ChromosomeT` impl when configuring `Selection::Lexicase` or `Selection::EpsilonLexicase`.

```rust
// Source: design from CONTEXT.md D-01, consistent with ChromosomeT pattern in src/traits/chromosome.rs
// File: src/traits/multi_case_fitness.rs

use crate::traits::ChromosomeT;

/// Opt-in trait for multi-case fitness evaluation.
///
/// Implement this alongside [`ChromosomeT`] to enable [`Selection::Lexicase`] and
/// [`Selection::EpsilonLexicase`]. Call `set_case_fitness` inside your
/// `calculate_fitness()` implementation:
///
/// ```rust,ignore
/// fn calculate_fitness(&mut self) {
///     let scores = run_all_test_cases(self.dna());
///     let mean = scores.iter().sum::<f64>() / scores.len() as f64;
///     self.set_case_fitness(scores);
///     self.set_fitness(mean);
/// }
/// ```
pub trait MultiCaseFitness: ChromosomeT {
    /// Returns the per-case fitness scores set during `calculate_fitness`.
    fn case_fitness(&self) -> &[f64];

    /// Sets the per-case fitness scores. Called inside `calculate_fitness`.
    fn set_case_fitness(&mut self, scores: Vec<f64>);
}
```

`[VERIFIED: codebase]` — ChromosomeT trait pattern confirmed in `src/traits/chromosome.rs`.

### Pattern 2: Lexicase Selection Free Function

**What:** Free function `lexicase_selection<U: ChromosomeT + MultiCaseFitness>()` following the exact shape of `clearing_selection`. [ASSUMED]

```rust
// Source: structural analog from src/operations/selection/clearing.rs
// File: src/operations/selection/lexicase.rs

use crate::traits::{ChromosomeT, MultiCaseFitness};
use rand::Rng;
use log::{debug, trace};

pub fn lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
) -> Vec<(usize, usize)>
where
    U: ChromosomeT + MultiCaseFitness,
{
    debug!(target="selection_events", method="lexicase"; "Starting lexicase selection, pop={}", chromosomes.len());

    if chromosomes.is_empty() || chromosomes[0].case_fitness().is_empty() {
        return Vec::new();
    }

    let n = chromosomes.len();
    let num_cases = chromosomes[0].case_fitness().len();
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(number_of_couples);

    while mating.len() < number_of_couples {
        // Pool is indices into chromosomes
        let mut pool: Vec<usize> = (0..n).collect();

        // Shuffle case indices — Fisher-Yates (same pattern as unique_initializer.rs)
        let mut case_order: Vec<usize> = (0..num_cases).collect();
        for i in (1..case_order.len()).rev() {
            let j = rng.random_range(0..=i);
            case_order.swap(i, j);
        }

        for &case in &case_order {
            if pool.len() <= 1 { break; }
            // Find best score on this case among candidates in pool
            let best = pool.iter()
                .map(|&idx| chromosomes[idx].case_fitness()[case])
                .fold(f64::NEG_INFINITY, f64::max);
            pool.retain(|&idx| chromosomes[idx].case_fitness()[case] >= best);
        }

        // Pick randomly from survivors (or the sole survivor)
        let chosen = pool[rng.random_range(0..pool.len())];
        trace!(target="selection_events", method="lexicase"; "Selected individual {}", chosen);
        mating.push((chosen, chosen)); // placeholder — real pairing loop pairs successive selections
    }

    // NOTE: The above is a sketch — the real implementation selects pairs by
    // running the filter loop twice per couple, not reusing the same index.
    debug!(target="selection_events", method="lexicase"; "Lexicase selection finished: {} pairs", mating.len());
    mating
}
```

`[ASSUMED]` — Code sketch for illustration; precise pair selection strategy (two independent filter runs per couple) must be confirmed by planner.

### Pattern 3: MAD Computation for Epsilon-Lexicase

**What:** Compute per-case MAD once before the selection loop. O(n log n) per case due to sort.

```rust
// Source: MAD-epsilon lexicase procedure from La Cava, Spector, Danai (GECCO 2016) [CITED: dl.acm.org/doi/10.1145/2908812.2908898]
// and canonical description from lexicase.ai

/// Compute per-case epsilon as MAD (Median Absolute Deviation from median) across population.
/// Returns a Vec<f64> of length `num_cases`.
fn compute_mad_epsilons(chromosomes: &[impl MultiCaseFitness], num_cases: usize) -> Vec<f64> {
    let n = chromosomes.len();
    let mut epsilons = Vec::with_capacity(num_cases);
    for case in 0..num_cases {
        let mut scores: Vec<f64> = chromosomes.iter()
            .map(|c| c.case_fitness()[case])
            .collect();
        scores.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if n % 2 == 0 {
            (scores[n / 2 - 1] + scores[n / 2]) / 2.0
        } else {
            scores[n / 2]
        };
        let mut deviations: Vec<f64> = scores.iter().map(|&s| (s - median).abs()).collect();
        deviations.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = if n % 2 == 0 {
            (deviations[n / 2 - 1] + deviations[n / 2]) / 2.0
        } else {
            deviations[n / 2]
        };
        epsilons.push(mad);
    }
    epsilons
}
```

`[CITED: dl.acm.org/doi/10.1145/2908812.2908898]` — MAD-epsilon procedure confirmed from original GECCO 2016 paper (La Cava, Spector, Danai).

### Pattern 4: Factory Dispatch (factory_lexicase)

**What:** Mirrors `selection::factory()` with an added `MultiCaseFitness` bound.

```rust
// Source: structural analog from src/operations/selection.rs factory()
// File: src/operations/selection.rs (new function added alongside existing factory())

pub fn factory_lexicase<U>(
    chromosomes: &mut Vec<U>,   // mut needed for set_fitness sync (D-04)
    configuration: SelectionConfiguration,
    number_of_threads: usize,
) -> Result<Vec<(usize, usize)>, GaError>
where
    U: ChromosomeT + MultiCaseFitness + Sync + Send + 'static + Clone,
{
    if chromosomes.len() < 2 {
        return Err(GaError::SelectionError(format!(
            "Population size {} too small for lexicase selection (minimum 2)",
            chromosomes.len()
        )));
    }

    // Guard: validate case fitness populated
    if chromosomes[0].case_fitness().is_empty() {
        return Err(GaError::SelectionError(
            "case_fitness() is empty — implement MultiCaseFitness::set_case_fitness in calculate_fitness()".into()
        ));
    }

    let pairs = match configuration.method {
        Selection::Lexicase => lexicase_selection(chromosomes, configuration.number_of_couples),
        Selection::EpsilonLexicase => {
            let eps = if configuration.epsilon == 0.0 {
                // dynamic MAD
                None
            } else {
                Some(configuration.epsilon)
            };
            epsilon_lexicase_selection(chromosomes, configuration.number_of_couples, eps)
        }
        _ => return Err(GaError::ConfigurationError(
            "factory_lexicase called with non-lexicase selection method".into()
        )),
    };

    // D-04: sync scalar fitness to mean case score for all individuals
    for c in chromosomes.iter_mut() {
        let scores = c.case_fitness();
        if !scores.is_empty() {
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            c.set_fitness(mean);
        }
    }

    Ok(pairs)
}
```

`[ASSUMED]` — Sketch only; mut signature for `chromosomes` needs reconciliation with `ga.rs` borrow patterns.

### Pattern 5: ga.rs Dispatch Branch (D-07)

**What:** The single line in `run()` that currently calls `selection::factory()` becomes an if/else.

```rust
// Source: src/engines/ga.rs:1474 (existing factory call confirmed by codebase read)
// The Ga<U> struct bound is U: LinearChromosome; factory_lexicase needs U: LinearChromosome + MultiCaseFitness.
// This requires a where-clause approach at the call site, not a generic bound on Ga<U> itself.

// IMPORTANT NOTE: Ga<U: LinearChromosome> cannot call factory_lexicase<U: MultiCaseFitness> directly
// unless U also implements MultiCaseFitness. The if/else branch approach means the compiler must
// be satisfied that the call in the Lexicase arm only executes when U: MultiCaseFitness.
// This is a TYPE SYSTEM CONSTRAINT — see "Pitfall 1: Ga<U> Bound Conflict" below.
```

**Resolution (from CONTEXT.md D-07):** Dispatch stays in `run()` per-generation. However, `Ga<U: LinearChromosome>` calling `factory_lexicase<U: LinearChromosome + MultiCaseFitness>` requires either:
- A separate `GaLexicase<U: LinearChromosome + MultiCaseFitness>` engine type, OR
- An `if let` branch with a downcast/trait-object, OR
- The `run()` method body gaining the `MultiCaseFitness` bound via an additional `where U: MultiCaseFitness` on a separate `impl` block, OR
- Using `Any`/unsafe, OR
- **Most pragmatic (D-07 approach):** The `ga.rs` `run()` method is on `impl<U> Ga<U> where U: LinearChromosome`. A compile-time check at the `Selection::Lexicase` arm calls `factory_lexicase` which requires `U: MultiCaseFitness` — this won't compile unless `Ga<U>` also requires `U: MultiCaseFitness` for that impl block.

`[ASSUMED]` — The D-07 dispatch approach needs concrete resolution at planning time. See "Open Questions" section.

### Anti-Patterns to Avoid

- **Calling `par_iter()` in lexicase:** WASM-incompatible. Lexicase requires sequential case-by-case filtering where the pool shrinks progressively — parallelism does not apply to the inner loop. Use `.iter()` throughout.
- **Computing MAD inside the pair loop:** MAD is stable for a generation and must be computed once before the shuffle/filter loop, not per pair selected.
- **Using `f64::NAN` as an epsilon sentinel:** `0.0` is the sentinel for MAD (D-09). NaN comparisons are always false and would silently corrupt filtering.
- **Setting scalar fitness before returning pairs:** The `set_fitness(mean)` sync (D-04) must happen after all pairs are selected, on all chromosomes in the population — not just winners.
- **Forgetting to guard empty case_fitness:** If a user forgets to call `set_case_fitness()` in their `calculate_fitness()`, `case_fitness()` returns an empty slice. The factory must detect this and return `GaError::SelectionError` with a clear diagnostic.
- **Mutating `chromosomes` slice signature in `factory_lexicase`:** The existing `factory()` takes `&[U]`. The `set_fitness` sync (D-04) needs `&mut [U]` or interior mutability. This signature difference needs careful handling to avoid breaking the existing call site in `ga.rs`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random shuffling | Custom PRNG or index rotation | `crate::rng::make_rng()` + Fisher-Yates (as in `unique_initializer.rs`) | Codebase-consistent; supports seeding for reproducibility |
| Median computation | Sorting + indexing by hand | Inline sort + index (no external crate needed; small O(n) operation) | `std` sort_unstable_by suffices; no crate required for this operation |
| Parallel selection | `rayon::par_iter()` | Sequential `.iter()` | WASM-incompatible; lexicase inner loop is inherently sequential (shrinking pool) |
| NaN-guarding | Custom NaN check loop | Reuse the existing guard pattern from `selection::factory()` | Consistency; existing tests validate this guard |

**Key insight:** Lexicase selection is a sequential filter algorithm — parallelism belongs at the offspring generation step, not the parent selection step. Do not attempt to parallelize the case-by-case filtering.

---

## Common Pitfalls

### Pitfall 1: Ga<U: LinearChromosome> Bound Cannot Call factory_lexicase<U: MultiCaseFitness>

**What goes wrong:** `Ga<U>` is defined as `where U: LinearChromosome`. `factory_lexicase` requires `U: ChromosomeT + MultiCaseFitness`. Rust will reject the call inside `run()` because `LinearChromosome: ChromosomeT` but `LinearChromosome` does NOT imply `MultiCaseFitness`.

**Why it happens:** Rust's type system enforces bounds at definition time. A branch that is only "conceptually" taken when `U: MultiCaseFitness` doesn't satisfy the compiler — all branches must typecheck regardless of runtime condition.

**How to avoid:**
Option A — Add a separate `impl<U> Ga<U> where U: LinearChromosome + MultiCaseFitness` block with a `run_lexicase()` method (not ideal — API split).
Option B — Use `Any`-based downcast (fragile, unsafe feel).
Option C — **Preferred:** Add `factory_lexicase_dyn` that operates via a trait object or takes a closure extracting `case_fitness`, so the `factory_lexicase` call in `run()` does not require the `MultiCaseFitness` bound on `U` in the standard `run()` impl.
Option D — **Simplest compatible with D-07:** Scope the `if/else` dispatch branch to a separate `run()` impl block that requires both bounds: `impl<U: LinearChromosome + MultiCaseFitness> Ga<U>`. The standard `run()` (non-lexicase path) stays on `impl<U: LinearChromosome> Ga<U>`.

`[ASSUMED]` — The exact Rust type system resolution needs planner decision. Option D appears most consistent with D-07 intent but creates a `Ga<U>` with two `run()` methods if both `impl` blocks exist.

**Warning signs:** Compiler error `the trait MultiCaseFitness is not implemented for U` at the `factory_lexicase` call site.

### Pitfall 2: Scalar Fitness Sync Invalidates Fitness Caching

**What goes wrong:** `ga.rs` has an optional LRU fitness cache (`fitness_cache_size`). After lexicase syncs `set_fitness(mean)` on every chromosome, cached fitness values become stale or inconsistent with the cache key (DNA hash).

**Why it happens:** D-04 calls `set_fitness(mean_case_score)` externally, not through `calculate_fitness()`. The fitness cache is keyed by DNA content; the cache does not know fitness was modified externally.

**How to avoid:** Document that `Selection::Lexicase` is incompatible with the LRU fitness cache. Add a guard in `factory_lexicase` (or in `run()` before calling it) that returns `GaError::ConfigurationError` if `fitness_cache_size` is Some and the selection method is Lexicase/EpsilonLexicase.

**Warning signs:** `fitness()` returns stale values after lexicase selection; `calculate_fitness()` is never called but fitness changes between generations.

### Pitfall 3: Empty Pool After Filtering All Cases

**What goes wrong:** If every individual has the exact same case fitness vector, filtering by each case might produce a pool of size 0 (impossible in theory) or all individuals survive every case (pool stays full, random selection at the end). The extreme edge case is a single-individual population reaching the pool-exhaustion branch.

**Why it happens:** The filter condition `case_fitness[i] >= best` always keeps the individual that IS the best — so the pool can never go to zero (at least one individual always has the best score on each case). However, floating-point equality comparison is fragile.

**How to avoid:** Assert `pool.len() >= 1` after each case filter. If it would hit 0 (shouldn't happen), fall back to the last non-empty pool rather than panicking.

**Warning signs:** `index out of bounds: the len is 0` panic inside lexicase_selection.

### Pitfall 4: Case Index Shuffling Using rayon — WASM Break

**What goes wrong:** If a contributor later "optimizes" the selection by parallelizing the couples generation loop with `par_iter`, wasm32 builds break silently at runtime (no thread pool).

**Why it happens:** `rayon::par_iter()` requires OS threads, which `wasm32-unknown-unknown` does not provide.

**How to avoid:** Use `.iter()` (not `.par_iter()`) throughout lexicase.rs. Add a `// WASM: intentionally not par_iter — lexicase inner loop is sequential by design` comment.

**Warning signs:** `cargo check --target wasm32-unknown-unknown` failure after any future optimization attempt.

### Pitfall 5: Median Computation on Empty Slice

**What goes wrong:** If `num_cases == 0` (user forgot to call `set_case_fitness`), computing `n / 2` on empty slice panics.

**Why it happens:** Integer arithmetic on `0 / 2` is fine, but indexing `scores[0]` on an empty Vec panics.

**How to avoid:** Guard at the top of `factory_lexicase`: if `chromosomes[0].case_fitness().is_empty()` return `GaError::SelectionError`.

---

## Code Examples

### MultiCaseFitness Implementation by a User

```rust
// Source: pattern from src/traits/chromosome.rs ChromosomeT implementation convention
// src/traits/multi_case_fitness.rs

use genetic_algorithms::traits::{ChromosomeT, MultiCaseFitness};

struct MyChromosome {
    dna: Vec<f64>,
    fitness: f64,
    case_scores: Vec<f64>,
    age: usize,
}

impl ChromosomeT for MyChromosome {
    type Gene = ...; // user's gene type

    fn calculate_fitness(&mut self) {
        // Populate case scores here — e.g., one score per test case
        let scores = vec![
            evaluate_case_1(&self.dna),
            evaluate_case_2(&self.dna),
            evaluate_case_3(&self.dna),
        ];
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        self.set_case_fitness(scores);
        self.set_fitness(mean);
    }
    // ... fitness(), set_fitness(), age(), set_age()
}

impl MultiCaseFitness for MyChromosome {
    fn case_fitness(&self) -> &[f64] { &self.case_scores }
    fn set_case_fitness(&mut self, scores: Vec<f64>) { self.case_scores = scores; }
}
```

`[VERIFIED: codebase]` — ChromosomeT pattern from `src/traits/chromosome.rs`; trait structure from CONTEXT.md D-01.

### Configuring Lexicase Selection

```rust
// Source: pattern from clearing selection config in test_selection_clearing.rs
use genetic_algorithms::{
    configuration::SelectionConfiguration,
    operations::Selection,
};

// Standard Lexicase
let config = SelectionConfiguration {
    method: Selection::Lexicase,
    number_of_couples: 10,
    ..Default::default()
};

// Epsilon-Lexicase with fixed epsilon
let config_eps = SelectionConfiguration {
    method: Selection::EpsilonLexicase,
    number_of_couples: 10,
    epsilon: 0.05,         // <-- new field
    ..Default::default()
};

// Epsilon-Lexicase with dynamic MAD (epsilon == 0.0)
let config_mad = SelectionConfiguration {
    method: Selection::EpsilonLexicase,
    number_of_couples: 10,
    epsilon: 0.0,          // sentinel: compute MAD per generation
    ..Default::default()
};
```

`[VERIFIED: codebase]` — `SelectionConfiguration` struct confirmed in `src/configuration.rs:96–119`.

### SelectionOperator Trait Panic Arm (D-08)

```rust
// Source: structural analog from src/operations/selection.rs Clearing warning pattern (lines 52–64)
// Applied to Lexicase/EpsilonLexicase

Selection::Lexicase | Selection::EpsilonLexicase => {
    panic!(
        "Selection::Lexicase/EpsilonLexicase cannot be called through SelectionOperator \
         trait: use selection::factory_lexicase for Lexicase/EpsilonLexicase operators. \
         Island-model and NSGA-II paths do not support MultiCaseFitness."
    );
}
```

`[VERIFIED: codebase]` — Warning pattern confirmed at `src/operations/selection.rs:52–64`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Lexicase via global mean fitness | Per-case filtering with mean-sync | La Cava 2016 (ε-lexicase) | Preserves specialists scalar fitness cannot retain |
| Fixed epsilon | Dynamic MAD epsilon | Helmuth/La Cava 2016 | Auto-adapts to problem difficulty; standard default today |
| Separate num_cases param | Derived from `chromosomes[0].case_fitness().len()` | Phase 50 design D-03 | Simpler API; user doesn't repeat themselves |

**Deprecated / outdated:**
- **Fixed epsilon only:** MAD-dynamic epsilon is the current standard for continuous regression. Fixed epsilon (via `with_epsilon_lexicase(0.05)`) is valid for discrete-valued fitness.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `factory_lexicase` takes `&mut [U]` to call `set_fitness(mean)` (D-04) — or the sync is done differently | Architecture Patterns Pattern 4 | Signature conflict with ga.rs borrow checker; may need inner mutability or post-loop sync |
| A2 | Pairing is done by running two independent lexicase filter loops per couple (not reusing one winner index) | Patterns Pattern 2 sketch | If wrong, produces self-pairs or biased selection |
| A3 | D-07 dispatch in `run()` resolves via a separate `impl<U: LinearChromosome + MultiCaseFitness> Ga<U>` block | Pitfall 1 | Type system constraint means the plan must specify the exact Rust pattern |
| A4 | `compute_mad_epsilons` operates on a `&[U]` slice with lifetime that outlives the selection call | Patterns Pattern 3 | Borrow lifetime issue if `chromosomes` is also borrowed mutably elsewhere in the call |
| A5 | No fitness-cache incompatibility guard currently exists; one needs to be added | Pitfall 2 | Silent stale fitness values under caching + lexicase combination |

---

## Open Questions (RESOLVED)

1. **Type system: how does `Ga<U: LinearChromosome>` call `factory_lexicase<U: MultiCaseFitness>` in `run()`?**
   - What we know: D-07 says dispatch happens in `run()` per-generation. Rust requires all branches to typecheck at compile time, not runtime.
   - What's unclear: The exact Rust pattern — separate impl block? Trait object dispatch? Runtime Any downcast?
   - Recommendation: Plan Wave 0 should decide and document this as the first task. Option D (separate `impl<U: LinearChromosome + MultiCaseFitness> Ga<U>` block with `run()` that redirects lexicase arms) is the most idiomatic.
   - RESOLVED (Plan 02, Task 2): Option D — separate `impl<U: LinearChromosome + MultiCaseFitness> Ga<U>` block providing `select_parents_lexicase()`; with a compile-time fallback renaming to `run_with_lexicase` if duplicate `run()` definitions are rejected by the compiler.

2. **Pairing strategy: one filter run per individual, or two independent runs per couple?**
   - What we know: Lexicase produces one individual per filter run. A couple requires two.
   - What's unclear: Whether the same filter pass produces two different winners (by using two separate independent filter runs), or if one run picks a parent and another picks the other.
   - Recommendation: Two independent filter runs per couple (standard academic practice). `number_of_couples` filter runs yield 2 × `number_of_couples` parents, then paired as `(winners[0], winners[1])`, `(winners[2], winners[3])`, etc.
   - RESOLVED (Plan 02, Task 1): Two independent filter runs per couple. `factory_lexicase` runs the single-winner filter cascade twice independently per couple and pairs winners as `(winners[0], winners[1])`, `(winners[2], winners[3])`, etc.

3. **Behavioral diversity integration test (SEL-02 success criterion 4): what constitutes "measurably more specialists"?**
   - What we know: The success criterion requires a CI behavioral diversity test comparing lexicase vs tournament.
   - What's unclear: The exact diversity metric (Shannon entropy of fitness values? count of unique behavioral phenotypes?), the problem instance, and the statistical test.
   - Recommendation: Use a multi-case benchmark where each chromosome has a score vector (e.g., 10 distinct test cases), measure variance in case-specific performance, and assert that lexicase variance is statistically higher than tournament over N runs. Use seeded RNG for determinism.
   - RESOLVED (Plan 02, Task 3): Per-case variance comparison with a `>= 1.2x` margin, N=50 population, seeded RNG. Test name: `test_lexicase_produces_more_specialists_than_tournament`.

4. **`mut chromosomes` for D-04 scalar sync: does `factory_lexicase` take ownership, `&mut Vec<U>`, or `&mut [U]`?**
   - What we know: The existing `factory()` takes `&[U]` (immutable slice). D-04 requires mutating fitness values.
   - What's unclear: Whether `ga.rs` can provide a mutable reference at the selection call site (line 1474–1478).
   - Recommendation: `factory_lexicase(chromosomes: &mut [U], ...)` — mutable slice. The call site in `run()` already has `&mut self.population.chromosomes` available.
   - RESOLVED (Plan 02, Task 1): Signature is `factory_lexicase(chromosomes: &mut [U], configuration: SelectionConfiguration, number_of_threads: usize) -> Result<Vec<(usize, usize)>, GaError>`.

---

## Environment Availability

Step 2.6: Phase 50 is code-only — no external tools, services, or CLIs required beyond the existing Rust toolchain.

```
wasm32-unknown-unknown target: confirmed available (cargo check --target wasm32-unknown-unknown passes [VERIFIED: codebase])
rand 0.9.2: present in Cargo.toml [VERIFIED: codebase]
log 0.4.22: present in Cargo.toml [VERIFIED: codebase]
```

---

## Validation Architecture

> `workflow.nyquist_validation` is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Config file | `Cargo.toml` (test harness is standard) |
| Quick run command | `cargo test --test test_selection_lexicase` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRAITS-01 | `MultiCaseFitness` trait compiles on a custom chromosome | unit | `cargo test --test test_multi_case_fitness` | Wave 0 |
| TRAITS-01 | `case_fitness()` returns the values set by `set_case_fitness()` | unit | `cargo test --test test_multi_case_fitness::test_roundtrip` | Wave 0 |
| SEL-02 | `lexicase_selection` returns `number_of_couples` pairs with valid indices | unit | `cargo test --test test_selection_lexicase::test_returns_correct_count` | Wave 0 |
| SEL-02 | Lexicase case order is shuffled (not always same order) | unit | `cargo test --test test_selection_lexicase::test_case_shuffle` | Wave 0 |
| SEL-02 | Scalar fitness after lexicase equals mean case score | unit | `cargo test --test test_selection_lexicase::test_scalar_fitness_sync` | Wave 0 |
| SEL-02 | `selection::factory()` returns `GaError::ConfigurationError` for `Lexicase` variant | unit | `cargo test --test test_selection_lexicase::test_factory_rejects_lexicase` | Wave 0 |
| SEL-03 | `epsilon_lexicase_selection` with fixed epsilon keeps candidates within tolerance | unit | `cargo test --test test_selection_lexicase::test_epsilon_filter_tolerance` | Wave 0 |
| SEL-03 | `epsilon_lexicase_selection` with `epsilon=0.0` computes MAD and applies it | unit | `cargo test --test test_selection_lexicase::test_mad_epsilon_applied` | Wave 0 |
| SEL-02 | Behavioral diversity: lexicase produces more specialists than tournament on multi-case benchmark | integration | `cargo test --test test_selection_lexicase_diversity` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --test test_selection_lexicase`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo check --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/operations/test_selection_lexicase.rs` — unit tests for SEL-02, SEL-03 (lexicase + epsilon-lexicase operators)
- [ ] `tests/traits/test_multi_case_fitness.rs` — unit tests for TRAITS-01 (trait roundtrip, compilation)
- [ ] `tests/operations/test_selection_lexicase_diversity.rs` — integration / behavioral diversity test (success criterion 4)
- [ ] `tests/structures.rs` — extend with `MultiCaseChromosome` test fixture (adds `case_scores: Vec<f64>` field)

---

## Security Domain

> `security_enforcement` not explicitly set in `.planning/config.json` — treated as enabled. However, Phase 50 is a pure algorithmic addition (no I/O, no authentication, no network, no secrets). ASVS categories are not applicable.

| ASVS Category | Applies | Rationale |
|---------------|---------|-----------|
| V2 Authentication | No | No auth surface |
| V3 Session Management | No | No sessions |
| V4 Access Control | No | No access decisions |
| V5 Input Validation | Partial | Guard against empty `case_fitness()`, `NaN` scores in operator inputs |
| V6 Cryptography | No | RNG is for algorithmic stochasticity, not security |

**Applicable hardening:** Input validation on `case_fitness()` being non-empty and free of `NaN` before filtering. This follows the existing `factory()` pattern which already guards NaN in scalar fitness.

---

## Project Constraints (from CLAUDE.md)

| Directive | Impact on Phase 50 |
|-----------|-------------------|
| No breaking changes | `Selection` enum gains new variants; existing `Default::default()` configs unaffected. `SelectionConfiguration` gains `epsilon: f64` with default `0.0` — existing configs using `..Default::default()` are not broken. |
| WASM compatibility mandatory | `lexicase.rs` must use `.iter()` not `.par_iter()`. `rng::make_rng()` already WASM-safe. `case_fitness()` is a pure slice access — no thread concerns. Verify with `cargo check --target wasm32-unknown-unknown`. |
| Tests in `tests/` folder | All test files go under `tests/operations/` and `tests/traits/`. No inline `#[cfg(test)] mod tests` in implementation files. |
| Operator impl follows enum + factory pattern | `Selection::Lexicase` / `EpsilonLexicase` in `src/operations.rs`, `factory_lexicase` in `src/operations/selection.rs`. Exactly mirrors the `Clearing` + `factory()` extension. |
| Tests in same file rule | MEMORY.md confirms: "All unit tests must be in `tests/`, never inline with implementation code." |
| Logging target convention | Use `log::debug!(target="selection_events", method="lexicase"; ...)` — consistent with clearing.rs and tournament.rs. |
| `Copy` enum constraint | `Selection` enum must stay `Copy`. `Lexicase` and `EpsilonLexicase` carry no data. `SelectionConfiguration` must stay `Copy` — `epsilon: f64` is `Copy`. |

---

## Sources

### Primary (HIGH confidence)
- `src/operations/selection/clearing.rs` — structural analog for lexicase operator shape `[VERIFIED: codebase]`
- `src/operations/selection.rs` — `factory()` + `SelectionOperator` impl patterns `[VERIFIED: codebase]`
- `src/traits/chromosome.rs` — `ChromosomeT` supertrait pattern `[VERIFIED: codebase]`
- `src/configuration.rs` — `SelectionConfiguration` struct with Copy-safe f64 fields `[VERIFIED: codebase]`
- `src/rng.rs` — `make_rng()` for Fisher-Yates shuffling `[VERIFIED: codebase]`
- `tests/structures.rs` — test fixture chromosome pattern for planning test stubs `[VERIFIED: codebase]`
- [lexicase.ai canonical algorithm description](https://lexicase.ai/) — standard lexicase and epsilon-lexicase procedure `[CITED: lexicase.ai]`

### Secondary (MEDIUM confidence)
- [La Cava, Spector, Danai (GECCO 2016)](https://dl.acm.org/doi/10.1145/2908812.2908898) — epsilon-lexicase with MAD-epsilon default `[CITED: dl.acm.org/doi/10.1145/2908812.2908898]`
- [Helmuth et al. lexicase diversity analysis 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC9453780/) — specialist-preserving properties of lexicase `[CITED: pmc.ncbi.nlm.nih.gov]`

### Tertiary (LOW confidence — training knowledge)
- Standard Fisher-Yates shuffle procedure (identical to `unique_initializer.rs` implementation) `[ASSUMED]`

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero new dependencies; all assets verified in codebase
- Architecture: HIGH — CONTEXT.md decisions are locked and complete; clearing.rs analog is definitive
- Algorithm (lexicase): HIGH — well-established academic procedure confirmed from official source
- Pitfalls: HIGH — type system conflict (Pitfall 1) is a real Rust constraint; others derived from existing codebase patterns
- Open Questions: MEDIUM — Pitfall 1 / A3 (Rust bound resolution) is the main unresolved design question

**Research date:** 2026-05-22
**Valid until:** 2026-06-22 (stable domain — algorithm specification is fixed; Rust type system resolution is the only moving part)
