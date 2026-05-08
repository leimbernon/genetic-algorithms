# Phase 35: NSGA-III for Many-Objective Optimization - Research

**Researched:** 2026-05-07
**Domain:** Many-objective genetic algorithm (NSGA-III), Rust library extension
**Confidence:** HIGH — codebase is fully verified; algorithm is from locked peer-reviewed source

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Extract `non_dominated_sort.rs` and `pareto.rs` from `src/engines/nsga2/` to a new `src/engines/multi_objective/` module.
- **D-02:** Expose as `pub mod multi_objective` in `src/lib.rs` via `#[path = "engines/multi_objective/mod.rs"]` pattern.
- **D-03:** `nsga2` keeps `pub use crate::multi_objective::pareto::*` and `pub use crate::multi_objective::non_dominated_sort::*` re-exports — zero breaking change.
- **D-04:** `Nsga3Configuration::with_reference_points_auto(p: usize)` — Das-Dennis lattice with subdivision `p`, generating `C(p+M-1, M-1)` points.
- **D-05:** `Nsga3Configuration::with_reference_points(Vec<Vec<f64>>)` — user-supplied custom points.
- **D-06:** If neither auto nor custom is configured, `validate()` returns `GaError::InvalidNsga3Configuration` — mirrors the existing `InvalidNsga2Configuration` pattern in `src/error.rs`. (Updated 2026-05-08: original CONTEXT.md said `GaError::ConfigurationError`; revised to align with the implemented per-engine variant pattern. See Open Questions (RESOLVED) below.)
- **D-07:** Auto and custom are mutually exclusive; last builder call wins.
- **D-08:** `Nsga3Observer<U>` sub-trait in `src/observe/observer/mod.rs`, mirrors `Nsga2Observer<U>`. Hooks: `on_pareto_front_assigned(gen, front_count, pop_size)` and `on_non_dominated_sort_complete(gen, duration_ms)`. No `on_reference_association` hook. All default no-op.
- **D-09:** `Nsga3Ga<U>` stores `Option<Arc<dyn Nsga3Observer<U> + Send + Sync>>` with `with_observer()` + `notify()` pattern.
- **D-10:** `AllObserver<U>` NOT updated in Phase 35 — deferred.
- **D-11:** `run()` returns `Result<ParetoFront<U>, GaError>`.
- **D-13:** `Nsga3Ga<U>` carries `Nsga3Observer<U>` only (no separate `GaObserver<U>` field) — matches Nsga2Ga.
- **D-14:** `LogObserver` (`src/observe/observer/log.rs`) implements `Nsga3Observer<U>` with debug-level messages on the `nsga3_events` target — mirrors the existing `impl Nsga2Observer<U> for LogObserver` block.

**Note on former D-12:** D-12 (`on_new_best` tracking on Nsga3Ga) was removed from the locked decision set on 2026-05-08 and moved to CONTEXT.md `<deferred>`. See Open Questions (RESOLVED) for the rationale.

### Claude's Discretion

- Das-Dennis generator implementation details (recursive vs iterative, internal function name).
- Reference point normalization: store raw (on unit hyperplane already) or normalize on construction.
- WASM cfg-gating: apply `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` to all `Instant::now()` and `par_iter()` call sites.

### Deferred Ideas (OUT OF SCOPE)

- Two-layer Das-Dennis reference points for M > 5 objectives.
- Constraint handling for NSGA-III.
- Updating `AllObserver<U>` to include `Nsga3Observer<U>`.
- Adaptive normalization (online ideal/nadir estimation).
- `on_new_best` tracking on `Nsga3Ga` (formerly D-12). See Open Questions (RESOLVED) for rationale.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOO-01 | User can run NSGA-III on problems with 3+ objectives; reference points are auto-generated (Das-Dennis simplex lattice) or user-supplied, and the algorithm selects survivors via reference-point association rather than crowding distance | D-01–D-11, D-13, D-14 cover all implementation decisions; algorithm procedure documented in Architecture Patterns section |
</phase_requirements>

---

## Summary

Phase 35 adds a new `Nsga3Ga<U>` engine following the exact structural pattern established by `Nsga2Ga<U>`. The primary algorithmic difference is in the survivor selection step: NSGA-II uses crowding distance, while NSGA-III uses reference-point association (Das-Dennis lattice points on the unit hyperplane, perpendicular-distance based niche selection).

The phase also introduces a refactoring: the shared primitives `non_dominated_sort.rs` and `pareto.rs` move from `src/engines/nsga2/` into a new `src/engines/multi_objective/` shared module, which both `nsga2` and `nsga3` import. This is a non-breaking structural change using the established `#[path]` pattern in `lib.rs` plus `pub use` re-exports in `nsga2`.

The algorithm itself is well-documented in Deb & Jain 2014 (IEEE-TEC) and is deterministic — the only discretion area is efficient implementation of the normalization + association step within the generation loop.

**Primary recommendation:** Copy `Nsga2Ga<U>` structure verbatim, swap crowding-distance assignment for reference-point association, and inline the Das-Dennis generator as a pure function in `src/engines/nsga3/das_dennis.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Shared multi-objective primitives (sort, pareto types) | `src/engines/multi_objective/` | — | Reused by nsga2, nsga3, and future phases 36-38 |
| NSGA-III engine orchestration | `src/engines/nsga3/mod.rs` | — | Engine owns generation loop |
| Reference point generation (Das-Dennis) | `src/engines/nsga3/das_dennis.rs` | — | Pure function, no state |
| Reference point association & niching | `src/engines/nsga3/mod.rs` (inline) | — | Called once per generation during env. selection |
| NSGA-III configuration | `src/engines/nsga3/configuration.rs` | — | Builder pattern, mirrors Nsga2Configuration |
| Observer hook dispatch | `src/observe/observer/mod.rs` | `src/engines/nsga3/mod.rs` | Trait defined in observer; called by engine |
| LogObserver Nsga3Observer impl | `src/observe/observer/log.rs` | — | Mirrors existing Nsga2Observer impl on LogObserver (D-14) |
| NSGA-II backward compatibility | `src/engines/nsga2/mod.rs` | — | `pub use` re-exports from multi_objective |
| Public API re-exports | `src/lib.rs` | — | Adds `pub mod multi_objective`, `pub mod nsga3`, `Nsga3Observer` |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` | 0.9.2 [VERIFIED: Cargo.lock] | RNG for crossover/mutation | Already in crate |
| `rayon` | 1.10 [VERIFIED: Cargo.lock] | Parallel objective evaluation | Already in crate, cfg-gated for WASM |
| `std::sync::Arc` | std | Observer storage | Same pattern as Nsga2Ga |
| `std::time::Instant` | std | Observer timing | cfg-gated, same as Nsga2Ga |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` | feature-gated | Config serialization | `#[cfg_attr(feature = "serde", derive(...))]` on Nsga3Configuration |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Inline Das-Dennis generator | External `ndarray` crate | External dep unnecessary; algorithm is 20 lines |
| Perpendicular distance via nalgebra | Manual dot-product formula | Nalgebra not in deps; formula is trivial |

**Installation:** No new dependencies required.

---

## Architecture Patterns

### System Architecture Diagram

```
User code
  │
  ▼
Nsga3Ga<U>::run()
  │
  ├─── initialize_population()  ──────────────────► parallel objective eval (rayon / iter cfg-gated)
  │                                                   returns Vec<ParetoIndividual<U>>
  │
  └─── for gen in 0..max_gens:
        │
        ├─── non_dominated_sort_with_directions()   ◄── from multi_objective module
        │       returns Vec<Vec<usize>> (fronts)
        │       observer: on_non_dominated_sort_complete
        │
        ├─── assign_ranks()                         ◄── from multi_objective module
        │
        ├─── create_offspring()   ──────────────────► binary tournament + crossover + mutation
        │                                              parallel objective eval
        │
        ├─── combine parent + offspring population
        │
        ├─── non_dominated_sort (combined)
        │
        ├─── assign_ranks (combined)
        │
        ├─── nsga3_environmental_selection():
        │       ├── collect all fronts that fit entirely  (fronts 0..L-1)
        │       ├── identify last (splitting) front Fl
        │       ├── normalize objectives:
        │       │     - compute ideal point z* (per-objective min across St)
        │       │     - translate: f'_i = f_i - z*_i
        │       │     - find extreme points via ASF (min scalarization per objective)
        │       │     - compute intercepts a_i (hyperplane through M extreme points)
        │       │     - normalize: f''_i = f'_i / a_i
        │       ├── associate each individual in St to nearest reference point
        │       │     (perpendicular distance to reference line)
        │       ├── count niche counts ρ_j per reference point (over already-selected)
        │       └── niche-selection from Fl until |Pt+1| == pop_size:
        │             - pick ref point j* with min ρ_j
        │             - if ref point has candidate in Fl: pick min-distance (ρ_j==0) or random
        │             - increment ρ_j*
        │
        └─── observer: on_pareto_front_assigned
        │
        (note: per-generation "best" tracking via on_new_best is intentionally NOT
         performed — see Open Questions (RESOLVED) for the rationale.)
  │
  ▼
filter rank==0 → ParetoFront<U>
```

### Recommended Project Structure

```
src/engines/
├── multi_objective/          # NEW — extracted from nsga2
│   ├── mod.rs                # pub mod non_dominated_sort; pub mod pareto;
│   ├── non_dominated_sort.rs # moved verbatim from nsga2
│   └── pareto.rs             # moved verbatim from nsga2
├── nsga2/
│   ├── mod.rs                # adds pub use crate::multi_objective::pareto::*;
│   │                         # adds pub use crate::multi_objective::non_dominated_sort::*;
│   ├── configuration.rs      # unchanged
│   ├── crowding_distance.rs  # unchanged
│   └── [pareto.rs removed, non_dominated_sort.rs removed]
└── nsga3/                    # NEW
    ├── mod.rs                # Nsga3Ga<U> engine
    ├── configuration.rs      # Nsga3Configuration + reference point enum
    └── das_dennis.rs         # Das-Dennis generator pure function

src/observe/observer/mod.rs   # add Nsga3Observer<U> trait (after Nsga2Observer)
src/observe/observer/log.rs   # add impl Nsga3Observer<U> for LogObserver (after Nsga2Observer impl)
src/lib.rs                    # add pub mod multi_objective, pub mod nsga3, pub use Nsga3Observer

tests/engines/
└── nsga3/                    # NEW
    ├── test_nsga3.rs          # engine integration tests
    ├── test_nsga3_configuration.rs
    └── test_das_dennis.rs     # reference point generator unit tests
tests/test_engines.rs         # add nsga3 mod block

tests/engines/
└── multi_objective/           # NEW (if split from nsga2)
    ├── test_non_dominated_sort.rs  # moved from nsga2 (or keep in nsga2)
    └── test_pareto.rs              # moved from nsga2 (or keep in nsga2)

examples/
└── nsga3_dtlz2.rs             # NEW
```

### Pattern 1: Module Extraction with Backward-Compatible Re-exports

**What:** Move files to new location, add `pub use` in old location.
**When to use:** Structural refactor without API breakage.

```rust
// src/engines/nsga2/mod.rs (after extraction)
// Source: established in v2.3.0, verified in src/lib.rs [VERIFIED: codebase]
pub use crate::multi_objective::pareto::*;
pub use crate::multi_objective::non_dominated_sort::*;

// src/lib.rs additions
#[path = "engines/multi_objective/mod.rs"]
pub mod multi_objective;
#[path = "engines/nsga3/mod.rs"]
pub mod nsga3;
pub use observer::Nsga3Observer;
```

### Pattern 2: Das-Dennis Simplex Lattice Generator

**What:** Generate `C(p+M-1, M-1)` uniformly-spaced points on the unit (M-1)-simplex.
**When to use:** Called once at construction when `with_reference_points_auto(p)` is configured.

```rust
// Source: Deb & Jain 2014, Das & Dennis 1998 [CITED: IEEE-TEC 2014]
// Algorithm: enumerate all non-negative integer vectors (n_1,...,n_M) with sum = p
// then normalize each to (n_1/p, ..., n_M/p).
// Produces exactly C(p+M-1, M-1) points.

pub fn generate_das_dennis(num_objectives: usize, p: usize) -> Vec<Vec<f64>> {
    let mut result = Vec::new();
    let mut current = vec![0usize; num_objectives];
    enumerate_partitions(p, num_objectives, 0, p, &mut current, &mut result);
    result
}

fn enumerate_partitions(
    total: usize,
    m: usize,
    dim: usize,
    remaining: usize,
    current: &mut Vec<usize>,
    result: &mut Vec<Vec<f64>>,
) {
    if dim == m - 1 {
        current[dim] = remaining;
        result.push(current.iter().map(|&n| n as f64 / total as f64).collect());
        return;
    }
    for v in 0..=remaining {
        current[dim] = v;
        enumerate_partitions(total, m, dim + 1, remaining - v, current, result);
    }
}
```

**Point count formula:** For M=3, p=4: C(4+3-1, 3-1) = C(6,2) = 15 points. [CITED: Das & Dennis 1998]

### Pattern 3: Reference Point Association (Normalization + Perpendicular Distance)

**What:** Normalize objective vectors to unit hyperplane, then associate each individual to nearest reference point by perpendicular distance.
**When to use:** Every generation, in `nsga3_environmental_selection()`.

```rust
// Step 1: Ideal point (per-objective minimum over combined population St)
// [CITED: Deb & Jain 2014, Procedure 1]
let ideal: Vec<f64> = (0..M).map(|m| population.iter().map(|ind| ind.objectives[m]).fold(f64::INFINITY, f64::min)).collect();

// Step 2: Translate objectives
// f'_im = f_im - z*_m

// Step 3: Extreme points via ASF (Achievement Scalarizing Function)
// ASF(x, w) = max_m { f'_im / w_m }  where w_m=1 for target axis, else 1e-6
// extreme_point[m] = argmin_x ASF(x, e_m)

// Step 4: Intercepts — fit hyperplane through M extreme points
// a_m = x-intercept of hyperplane (when other objectives = 0)
// If degenerate (extreme points nearly identical), fall back to nadir = max per-objective

// Step 5: Normalize
// f''_im = f'_im / a_m  (clamp to small epsilon if a_m ≈ 0)

// Step 6: Perpendicular distance from point f'' to reference line r
// d_perp(f'', r) = ||f'' - (f''·r / ||r||²) * r||
// [VERIFIED: moo-rs docs perpendicular distance formula]

// Step 7: Associate each individual to nearest ref point (min d_perp)
```

### Pattern 4: Niche-Preservation Operator (Last Front Selection)

**What:** Select `k = pop_size - |St \ Fl|` individuals from the splitting front Fl using niche counts.
**When to use:** Inside `nsga3_environmental_selection()`, after association step.

```rust
// [CITED: Deb & Jain 2014, Procedure 2]
// niche_count[j] = number of already-selected individuals associated with ref point j
// Loop until k individuals selected from Fl:
//   1. Find ref points with minimum niche_count (ties broken by selecting all tied refs)
//   2. Pick one j* at random from tied refs
//   3. If niche_count[j*] == 0:
//        Select individual in Fl with minimum d_perp to ref j* (if tie, pick randomly)
//      Else (niche_count[j*] > 0):
//        Select individual in Fl associated with j* uniformly at random
//   4. Add selected individual to Pt+1; remove from Fl; increment niche_count[j*]
```

### Anti-Patterns to Avoid

- **Re-implementing non_dominated_sort:** Use the shared function from `multi_objective` — it is already tested and direction-aware.
- **Using `sort()` instead of partial sort for niche min:** Finding minimum niche count should use `iter().min_by_key()`, not a full sort.
- **Storing normalized objectives in `ParetoIndividual`:** Normalization is transient per generation (ideal/intercept change). Store normalized objectives in a local `Vec<Vec<f64>>`, not in the struct.
- **Forgetting to clamp intercepts:** If all extreme points are identical or the hyperplane is degenerate, intercepts can be 0 or negative. Clamp to `f64::EPSILON` before dividing.
- **Calling `Instant::now()` unconditionally:** Always wrap in `#[cfg(not(target_arch = "wasm32"))]` block — CLAUDE.md mandatory constraint.
- **Using `par_iter()` unconditionally:** Duplicate iterator call with cfg gates — CLAUDE.md mandatory constraint.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Non-dominated sorting | Custom sort | `multi_objective::non_dominated_sort_with_directions()` | Already tested, direction-aware, O(N²M) correct implementation |
| Binary tournament selection | Custom tournament | Copy `binary_tournament()` from `Nsga2Ga` verbatim | Same semantics (rank-based, feasibility-aware) |
| Parallel objective evaluation | Custom thread pool | `rayon::par_iter()` (cfg-gated) | Already in codebase, correct WASM fallback pattern |
| Population initialization | Custom initializer | `crate::traits::initialize_chromosomes()` | Used by Nsga2Ga, tested |
| RNG seeding | Custom seed | `crate::rng::make_rng()` / `crate::rng::set_seed()` | Handles optional seeding consistently |

**Key insight:** NSGA-III's unique logic is confined to two functions: `generate_das_dennis()` (~25 lines) and `nsga3_environmental_selection()` (~80 lines). Everything else is a direct copy of the NSGA-II pattern.

---

## Common Pitfalls

### Pitfall 1: Degenerate Normalization (Zero or Negative Intercept)

**What goes wrong:** When the population has very low diversity (especially in early generations), extreme points across objectives may be nearly identical. The hyperplane intercept computation produces `a_m ≈ 0`, causing division-by-zero in normalization.

**Why it happens:** ASF minimization returns the same individual for multiple objectives when all individuals cluster near one point.

**How to avoid:** After computing intercepts, clamp each `a_m = a_m.max(f64::EPSILON)`. The original Deb & Jain paper notes this fallback: "use the nadir point estimate (per-objective maximum of St) when intercepts are not well-defined." [CITED: Deb & Jain 2014]

**Warning signs:** `f64::NaN` or `f64::INFINITY` values in normalized objectives; all individuals associating with a single reference point.

### Pitfall 2: Non_dominated_sort Import Path Changes

**What goes wrong:** After extracting `non_dominated_sort.rs` to `multi_objective`, the old `use crate::nsga2::non_dominated_sort::*` path in `nsga2/mod.rs` and in tests must be updated.

**Why it happens:** The extraction is a non-trivial rename affecting imports in `nsga2/mod.rs`, `tests/engines/nsga2/test_non_dominated_sort.rs`, `tests/engines/nsga2/test_pareto.rs`.

**How to avoid:** Add `pub use crate::multi_objective::non_dominated_sort::*` to `nsga2/mod.rs` immediately after extraction. This preserves `genetic_algorithms::nsga2::non_dominated_sort::*` public path. Test imports can keep using this path or switch to `multi_objective` — both work.

**Warning signs:** `cargo test` failing with "unresolved import" after extraction step.

### Pitfall 3: ParetoIndividual Has `crowding_distance` Field — NSGA-III Doesn't Use It

**What goes wrong:** `ParetoIndividual<U>` has a `crowding_distance: f64` field used by NSGA-II. After moving to `multi_objective`, this field remains but is meaningless in NSGA-III. Planner must not attempt to add a `niche_distance` field to `ParetoIndividual` — that breaks NSGA-II.

**Why it happens:** Shared struct, algorithm-specific meaning.

**How to avoid:** NSGA-III tracks `(reference_point_idx, perpendicular_distance)` as local variables inside `nsga3_environmental_selection()`, not stored on the struct. The `crowding_distance` field is left at 0.0 for NSGA-III individuals.

**Warning signs:** Plan task adding fields to `ParetoIndividual` for NSGA-III purposes.

### Pitfall 4: `ObjectiveFn<G>` Type Alias Lives in nsga2/mod.rs

**What goes wrong:** `Nsga3Ga<U>` needs `ObjectiveFn<G>`. The type alias currently lives in `nsga2/mod.rs`. If the plan task for `nsga3/mod.rs` tries to define a second `ObjectiveFn<G>` type alias, there will be duplication.

**Why it happens:** CONTEXT.md §code_context says "move `ObjectiveFn<G>` type alias to `multi_objective` as shared type alias." This is the correct resolution.

**How to avoid:** Move `pub type ObjectiveFn<G> = ...` from `nsga2/mod.rs` into `multi_objective/mod.rs`. Both `nsga2` and `nsga3` import it from there. This is part of the extraction plan task (Wave 1).

### Pitfall 5: `AllObserver<U>` Supertrait Not Updated

**What goes wrong:** `AllObserver<U>` is defined as `GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U>`. After adding `Nsga3Observer<U>`, the trait is NOT in `AllObserver` (deferred per D-10). Any user who implements `AllObserver<U>` will NOT automatically get `Nsga3Observer` hooks — this is intentional and documented.

**Why it happens:** Adding `Nsga3Observer<U>` to `AllObserver<U>` would be a breaking change for existing implementors.

**How to avoid:** Document clearly in `Nsga3Observer` rustdoc that `AllObserver` does not include it in Phase 35. The `with_observer()` method on `Nsga3Ga<U>` accepts `Arc<dyn Nsga3Observer<U>>` independently.

### Pitfall 6: WASM cfg-Gating Forgotten in New Files

**What goes wrong:** `das_dennis.rs` is pure math — no WASM issues. But `nsga3/mod.rs` calls `Instant::now()` (for observer timing) and `par_iter()` (for objective evaluation). Missing cfg gates cause CI failures.

**Why it happens:** Copy-paste from Nsga2Ga tends to preserve the gates, but new code additions may omit them.

**How to avoid:** Copy `initialize_population()` and `create_offspring()` VERBATIM from `Nsga2Ga` (they already have correct cfg gates). For observer timing blocks, copy the exact pattern:
```rust
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
```

### Pitfall 7: LogObserver Cast in nsga3 Example Without Nsga3Observer Impl

**What goes wrong:** The `examples/nsga3_dtlz2.rs` casts `Arc::new(LogObserver)` as `Arc<dyn Nsga3Observer<RangeChromosome<f64>> + Send + Sync>`. If `LogObserver` does not yet `impl Nsga3Observer<U>`, the cast fails to compile.

**Why it happens:** `LogObserver` already implements `GaObserver`, `IslandGaObserver`, and `Nsga2Observer`. Phase 35 adds the missing `Nsga3Observer` impl in `src/observe/observer/log.rs` (per D-14).

**How to avoid:** Plan 02 includes the `impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver` block in `src/observe/observer/log.rs`, mirroring the existing `Nsga2Observer` impl on the same type. Verify with `grep -c 'impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver' src/observe/observer/log.rs` returning 1.

---

## Code Examples

### Das-Dennis Point Count Verification

```rust
// Source: combinatorial formula C(p+M-1, M-1) [CITED: Das & Dennis 1998]
// M=3 objectives:
// p=2: C(4,2) = 6 points
// p=4: C(6,2) = 15 points
// p=6: C(8,2) = 28 points
// M=5 objectives, p=6: C(10,4) = 210 points
//
// Rule of thumb from Deb & Jain 2014: choose p such that
// C(p+M-1, M-1) >= population_size (ideally ≈ population_size)
// For M=3, pop=100: p=12 gives C(14,2)=91 ≈ 100 ✓
```

### Nsga3Configuration Builder Pattern

```rust
// Source: mirrors Nsga2Configuration [VERIFIED: src/engines/nsga2/configuration.rs]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nsga3Configuration {
    pub num_objectives: usize,
    pub population_size: usize,
    pub max_generations: usize,
    pub objective_directions: Vec<ObjectiveDirection>,
    // Internal: one of these is Some after builder call
    reference_points_auto_p: Option<usize>,
    reference_points_custom: Option<Vec<Vec<f64>>>,
}

impl Nsga3Configuration {
    pub fn with_reference_points_auto(mut self, p: usize) -> Self {
        self.reference_points_auto_p = Some(p);
        self.reference_points_custom = None; // last call wins
        self
    }
    pub fn with_reference_points(mut self, points: Vec<Vec<f64>>) -> Self {
        self.reference_points_custom = Some(points);
        self.reference_points_auto_p = None; // last call wins
        self
    }
    // Called during validate() to materialise the points
    pub fn effective_reference_points(&self) -> Option<Vec<Vec<f64>>> { ... }
}
```

### Nsga3Observer Trait Definition

```rust
// Source: mirrors Nsga2Observer [VERIFIED: src/observe/observer/mod.rs lines 154-167]
pub trait Nsga3Observer<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {}
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}
```

### LogObserver Nsga3Observer Impl (per D-14)

```rust
// Source: mirrors `impl Nsga2Observer<U> for LogObserver` at src/observe/observer/log.rs lines 190-206 [VERIFIED]
impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver {
    fn on_pareto_front_assigned(
        &self,
        generation: usize,
        front_count: usize,
        population_size: usize,
    ) {
        log::debug!(target: "nsga3_events",
            "Generation {} complete, population size = {}, fronts = {}",
            generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "nsga3_events",
            "Non-dominated sort complete at generation {} ({:.2}ms)",
            generation, duration_ms);
    }
}
```

### DTLZ2 Objective Functions (Example)

```rust
// Source: Deb et al. 2002 DTLZ test suite [CITED: standard benchmark]
// 3-objective sphere: f1²+f2²+f3² = 1
// Variables: x = (x_1,...,x_n) in [0,1]
// f_1 = cos(x_1 * π/2) * cos(x_2 * π/2) * g(x)
// f_2 = cos(x_1 * π/2) * sin(x_2 * π/2) * g(x)
// f_3 = sin(x_1 * π/2) * g(x)
// g(x) = 1 + 9/(n-M) * sum(x_{M},...,x_n)  [standard form, M=3]
// Pareto-optimal: g(x)=1 (x_3,...,x_n = 0), front lies on unit sphere
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Crowding distance (NSGA-II) | Reference-point association (NSGA-III) | Deb & Jain 2014 | Better coverage on 3+ objectives; crowding distance degrades beyond 3 |
| nsga2-specific pareto/sort | Shared `multi_objective` module | Phase 35 | Enables MOEA/D, SPEA2 (phases 36-37) to reuse without duplication |
| N/A | `ObjectiveFn<G>` in `multi_objective` | Phase 35 | Type alias available to all multi-objective engines |

**Deprecated/outdated:**
- NSGA-II crowding distance for >3 objectives: not deprecated as a library feature, but NSGA-III is the preferred algorithm for many-objective (3+) problems per Deb & Jain 2014.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `non_dominated_sort.rs` uses `super::pareto::*` imports that will need updating after file move | Architecture Patterns (extraction pitfall) | Compile error during extraction — low risk, caught by `cargo check` |
| A2 | DTLZ2 example can use `RangeChromosome<f64>` with `range_random_initialization` as in ZDT1 example | Code Examples | Wrong chromosome type needed — low risk, ZDT1 uses the same type |

All other claims were verified directly from the codebase or the locked CONTEXT.md decisions.

---

## Open Questions (RESOLVED)

1. **Should test files for `non_dominated_sort` and `pareto` move to `tests/engines/multi_objective/`?**
   - What we know: tests are currently under `tests/engines/nsga2/test_non_dominated_sort.rs` and `test_pareto.rs`.
   - What's unclear: Whether to move them or leave them under nsga2 (both compile fine; nsga2 still re-exports).
   - **RESOLVED (2026-05-08):** Leave existing tests under `tests/engines/nsga2/`. They continue to compile via the `pub use crate::multi_objective::*` re-exports added by Plan 01, so no test code changes are needed. A new `tests/engines/multi_objective/` directory is not created in Phase 35; nsga3-specific utility tests live under `tests/engines/nsga3/`.

2. **Does `Nsga3Configuration` need `rng_seed` or does it inherit from `GaConfiguration`?**
   - What we know: `Nsga2Ga<U>` takes both `Nsga2Configuration` and `GaConfiguration`. `GaConfiguration` has `rng_seed`.
   - What's unclear: Whether the Phase 35 plan should keep the same two-arg constructor.
   - **RESOLVED (2026-05-08):** Mirror Nsga2Ga exactly — `Nsga3Ga::new(nsga3_config: Nsga3Configuration, ga_config: GaConfiguration)`. The engine reads `rng_seed` from the inherited `GaConfiguration` (via `crate::rng::set_seed(self.ga_config.rng_seed)` at the top of `run()`). `Nsga3Configuration` does NOT add a redundant `rng_seed` field.

3. **Should D-12 (`on_new_best` tracking on Nsga3Ga) be implemented in Phase 35?**
   - What we know: D-12 originally specified that `Nsga3Ga::run()` would track the individual in front 0 with the smallest objective-0 value as the "best" and forward it via an `on_new_best` hook (mirroring `GaObserver::on_new_best`).
   - What's unclear: How to surface this hook given that (a) `Nsga3Observer` deliberately does NOT include `on_new_best` (D-08 limits it to NSGA-III lifecycle hooks); (b) `Nsga3Ga` deliberately does NOT carry a separate `GaObserver<U>` field (D-13); (c) `AllObserver<U>` is intentionally NOT extended in this phase (D-10); and (d) Deb & Jain 2014 do not define a single "best" individual in many-objective settings — picking obj-0 as a stand-in is arbitrary and may mislead users (e.g., on DTLZ2 it picks the individual with smallest f_1, which has nothing to do with overall solution quality).
   - **RESOLVED (2026-05-08):** Defer D-12 to a future phase. Removed from the locked decision set; moved to CONTEXT.md `<deferred>`. Rationale: implementing it now would either (i) widen `Nsga3Observer` (violates D-08), (ii) re-introduce a `GaObserver<U>` field on `Nsga3Ga` (violates D-13), or (iii) extend `AllObserver` (violates D-10). A future phase can introduce a richer concept (e.g., hypervolume-based "best" or a reference-point-weighted score) and decide where the hook lives. Until then, users that need per-generation summary data can rely on `on_pareto_front_assigned` and inspect the returned `ParetoFront<U>` after `run()`.

4. **Should D-06 say `GaError::ConfigurationError` or `GaError::InvalidNsga3Configuration`?**
   - What we know: The original CONTEXT.md D-06 said `GaError::ConfigurationError`. The existing `src/error.rs` has a per-engine variant `InvalidNsga2Configuration(String)` for NSGA-II validation failures, which is a tighter, more discoverable error type.
   - What's unclear: Which variant to use for NSGA-III validation errors.
   - **RESOLVED (2026-05-08):** Use `GaError::InvalidNsga3Configuration(String)`, mirroring the existing `InvalidNsga2Configuration` pattern. CONTEXT.md D-06 was updated on 2026-05-08 to say `GaError::InvalidNsga3Configuration`. The plans (35-02 Task 2.1, 35-03 Task 3.1) were already correctly using `InvalidNsga3Configuration` against the NSGA-II precedent — D-06 wording now matches the implementation.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 35 is purely code changes within an existing Rust workspace. No external tools, services, or runtimes beyond what is already in the project.

**Pre-existing test failure:** `test_reporter_on_new_best_fires` in `tests/observe/reporter/test_reporter.rs` fails independently of Phase 35 work. [VERIFIED: cargo test run 2026-05-07] This is a pre-existing issue, not introduced by this phase.

**WASM baseline:** `cargo check --target wasm32-unknown-unknown --lib` passes (3 errors are getrandom backend issues in the getrandom crate itself when no `js` feature is specified — this is a known pre-existing state unrelated to Phase 35). CI uses `--lib` which avoids the getrandom integration issue. [VERIFIED: .github/workflows/wasm-check.yml]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — standard Cargo integration |
| Quick run command | `cargo test nsga3` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOO-01 | Das-Dennis generates correct point count C(p+M-1,M-1) | unit | `cargo test test_das_dennis` | ❌ Wave 0 |
| MOO-01 | Das-Dennis points sum to 1.0 per vector | unit | `cargo test test_das_dennis` | ❌ Wave 0 |
| MOO-01 | `Nsga3Configuration` validates missing reference points | unit | `cargo test test_nsga3_configuration` | ❌ Wave 0 |
| MOO-01 | `Nsga3Configuration` validates wrong reference point dimension | unit | `cargo test test_nsga3_configuration` | ❌ Wave 0 |
| MOO-01 | `Nsga3Ga::run()` produces non-empty ParetoFront on 3-obj problem | integration | `cargo test test_nsga3` | ❌ Wave 0 |
| MOO-01 | `Nsga3Ga::run()` distributes solutions across reference points | integration | `cargo test test_nsga3` | ❌ Wave 0 |
| MOO-01 | `nsga2::pareto::ParetoIndividual` still accessible after extraction | regression | `cargo test test_pareto` | ✅ (existing) |
| MOO-01 | `nsga2::non_dominated_sort::non_dominated_sort` still accessible | regression | `cargo test test_non_dominated_sort` | ✅ (existing) |
| MOO-01 | WASM compile check passes | build | `cargo check --target wasm32-unknown-unknown --lib` | ✅ (CI) |
| MOO-01 (D-14) | `LogObserver` is castable to `Arc<dyn Nsga3Observer<U>>` | integration | `cargo test test_nsga3` (covered by example build) | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test nsga3 && cargo check --target wasm32-unknown-unknown --lib`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green (excluding pre-existing `test_reporter_on_new_best_fires` failure) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/engines/nsga3/test_das_dennis.rs` — covers Das-Dennis correctness (MOO-01)
- [ ] `tests/engines/nsga3/test_nsga3_configuration.rs` — covers config validation (MOO-01)
- [ ] `tests/engines/nsga3/test_nsga3.rs` — covers engine integration (MOO-01)
- [ ] `tests/test_engines.rs` — add `mod nsga3 { mod test_das_dennis; mod test_nsga3_configuration; mod test_nsga3; }` block

---

## Security Domain

Security enforcement is not applicable to this phase. NSGA-III is a numerical algorithm with no external inputs, authentication, sessions, or cryptography. All data is in-process f64 values from user-provided objective functions.

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: src/engines/nsga2/mod.rs] — Nsga2Ga engine structure, observer pattern, WASM cfg-gating, binary tournament
- [VERIFIED: src/engines/nsga2/configuration.rs] — Nsga2Configuration builder pattern to mirror
- [VERIFIED: src/engines/nsga2/pareto.rs] — ParetoIndividual, ParetoFront types
- [VERIFIED: src/engines/nsga2/non_dominated_sort.rs] — non_dominated_sort functions to reuse
- [VERIFIED: src/observe/observer/mod.rs] — Nsga2Observer trait pattern (lines 154-167), AllObserver supertrait
- [VERIFIED: src/observe/observer/log.rs] — `impl Nsga2Observer<U> for LogObserver` block (lines 190-206) — exact pattern for the new `impl Nsga3Observer<U> for LogObserver` (D-14)
- [VERIFIED: src/lib.rs] — `#[path]` re-export pattern, existing pub module list
- [VERIFIED: src/error.rs] — GaError enum; `InvalidNsga2Configuration` variant as model for `InvalidNsga3Configuration`
- [VERIFIED: tests/test_engines.rs] — test module structure to extend
- [VERIFIED: .github/workflows/wasm-check.yml] — CI uses `--lib` flag for WASM check

### Secondary (MEDIUM confidence)
- [CITED: Deb & Jain 2014, IEEE-TEC 18(4):577-601] — NSGA-III algorithm (non-dominated sorting + reference-point association + niche operator)
- [CITED: Das & Dennis 1998] — Simplex lattice reference point generation formula
- [CITED: Deb et al. 2002 DTLZ] — DTLZ2 3-objective benchmark problem definition
- [moo-rs documentation](https://andresliszt.github.io/moo-rs/user_guide/algorithms/nsga3.html) — perpendicular distance formula d_perp(f'', r) confirmed

### Tertiary (LOW confidence)
- None required — all implementation decisions are locked in CONTEXT.md and algorithm is from established source

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps already in Cargo.toml; no new dependencies
- Architecture: HIGH — extraction pattern proven in v2.3.0; engine pattern proven by Nsga2Ga
- Algorithm correctness (Das-Dennis, niche selection): HIGH — from Deb & Jain 2014 paper; details in CONTEXT.md §specifics
- Pitfalls: HIGH — degenerate normalization is documented in follow-up Deb papers; import path changes are deterministic

**Research date:** 2026-05-07
**Last revised:** 2026-05-08 — Open Questions resolved; D-06 reworded to use `GaError::InvalidNsga3Configuration`; D-12 deferred; D-14 added (LogObserver Nsga3Observer impl).
**Valid until:** 2026-06-07 (stable algorithm domain; no external dependencies to expire)
