# Phase 62: Surrogate-Assisted Evaluation - Research

**Researched:** 2026-06-09
**Domain:** Rust trait design, GA hot-path integration, WASM-compatible offspring filtering
**Confidence:** HIGH

## Summary

Phase 62 adds a `SurrogateModel<U>` trait that acts as a pre-screener on the offspring batch each generation, reducing the number of true fitness calls on expensive black-box problems. The surrogate ranks all newly generated offspring by predicted fitness; only the top `prescreening_fraction` survive to the real evaluation path (FitnessCache check then BatchFitnessEvaluator / scalar `fitness_fn`). Rejected offspring are dropped entirely — the surrogate is a filter, not a fitness predictor.

All decisions are locked in CONTEXT.md (D-01 through D-11). The implementation is narrow: one new trait file, one new field on `Ga<U>`, one new builder method, one new `GenerationStats` field, one re-export, and tests. No new external crates are required.

The primary risk is inserting the prescreening step in exactly the right location in the `ga.rs` hot path — between `parent_crossover()` output and the existing batch-evaluate / scalar fitness branches — without disturbing existing pipeline ordering (repair, constraints, local search, cache delta tracking).

**Primary recommendation:** Model every new piece directly on the existing Phase 60 `BatchFitnessEvaluator` / `cache_hits` pattern; the diff is small and fully constrained by the locked decisions.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `SurrogateModel<U>` trait with exactly one method `fn predict(&self, chromosome: &U) -> f64`. Training is user-managed; no `train()` / `update()` hooks.
- **D-02:** Trait lives in `src/fitness/surrogate.rs`. `Send + Sync` required for `Arc<dyn SurrogateModel<U>>` across rayon threads.
- **D-03:** Builder method `.with_surrogate(model: Arc<dyn SurrogateModel<U> + Send + Sync>, prescreening_fraction: f64) -> Self` on `Ga`.
- **D-04:** Rejected offspring (bottom `1 - prescreening_fraction`) are dropped entirely — they never enter the fitness evaluation path.
- **D-05:** Minimum floor: `max(1, floor(n * prescreening_fraction))` offspring always pass through.
- **D-06:** Prescreening applies to offspring only; existing population is never re-screened.
- **D-07:** Surrogate support added to `Ga` only. `CmaEngine` and `IslandGa` are out of scope.
- **D-08:** Pipeline order: surrogate prescreens offspring → FitnessCache check → BatchFitnessEvaluator or scalar `fitness_fn`.
- **D-09:** Surrogate and `BatchFitnessEvaluator` are compatible and compose cleanly.
- **D-10:** `GenerationStats` gains `true_fitness_calls: Option<u64>`. `None` when no surrogate; `Some(n)` is post-prescreening offspring count. Follows `cache_hits` / `cache_misses` pattern.
- **D-11:** `GaObserver` receives `true_fitness_calls` via existing `on_generation_complete(&self, stats: &GenerationStats)` — no new observer method needed.

### Claude's Discretion

- Internal variable names for the prescreened offspring sub-slice
- Whether `prescreening_fraction` is stored as a field in `GaConfiguration` or inline in the surrogate builder tuple
- Whether `SurrogateModel` is re-exported from `src/lib.rs` at crate root (follow `BatchFitnessEvaluator` re-export pattern)
- How the prescreening sort handles NaN surrogate predictions (treat as worst score)
- Whether to add a `with_surrogate` validation step to `src/validators/`

### Deferred Ideas (OUT OF SCOPE)

- CmaEngine surrogate support
- IslandGa surrogate support
- Online surrogate learning (`update` hook)
- Surrogate for initial population screening
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Surrogate trait definition | Library (src/fitness/) | — | Mirrors BatchFitnessEvaluator placement; user-facing API |
| Offspring prescreening sort | GA engine (src/engines/ga.rs) | — | Hot-path step between offspring generation and fitness evaluation |
| true_fitness_calls counting | GA engine (src/engines/ga.rs) | stats.rs | Count taken at prescreening site; written into GenerationStats |
| Re-export to public API | src/lib.rs | — | Consistent with all other user-facing traits |
| Validation of prescreening_fraction | GA engine build() | src/validators/ (optional) | Pattern matches how batch_evaluator mutual exclusivity is validated in build() |

---

## Standard Stack

### Core (no new dependencies)

Phase 62 introduces zero new external crates. All required capabilities (sorting, Arc, Send+Sync, cfg gating) are available from `std` and existing project infrastructure. [VERIFIED: codebase inspection]

| Capability | Source | Note |
|-----------|--------|------|
| `Arc<dyn Trait + Send + Sync>` | std | Established pattern for batch_evaluator and observer fields |
| Offspring sort by predicted score | std `sort_unstable_by` | Sequential; offspring batch is small; no rayon needed |
| NaN handling in sort | std `f64::total_cmp` or `partial_cmp().unwrap_or(Less)` | NaN treated as worst score per discretion |
| `#[cfg_attr(feature = "serde", ...)]` | existing serde feature | Same gating on new GenerationStats field |

**Installation:** none — no new packages.

## Package Legitimacy Audit

No external packages are introduced in this phase.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
Each generation:

  parent_crossover() output
         │
         ▼
  ┌──────────────────────────────┐
  │  Surrogate prescreening      │  ← NEW (when surrogate is Some)
  │  - predict() each offspring  │
  │  - sort desc by score        │
  │  - keep top max(1, floor(n*f)│
  │  - count survivors → u64     │
  └───────────┬──────────────────┘
              │  surviving offspring slice
              ▼
  ┌──────────────────────────────┐
  │  FitnessCache check          │  ← existing (Phase 60)
  │  (LRU hit/miss partition)    │
  └───────────┬──────────────────┘
              │
              ▼
  ┌──────────────────────────────┐
  │  BatchFitnessEvaluator       │  ← existing (Phase 60)
  │  OR scalar fitness_fn        │
  └───────────┬──────────────────┘
              │
              ▼
  repair / constraint penalty / local search
              │
              ▼
  population merge → survivor selection → stats collection
              │
              ▼
  GenerationStats { true_fitness_calls: Some(n), cache_hits, ... }
              │
              ▼
  GaObserver::on_generation_complete(&stats)
```

### Recommended Project Structure

```
src/fitness/
├── batch.rs            # BatchFitnessEvaluator<U> — existing
├── cache.rs            # FitnessCache — existing
├── count_true.rs       # existing
├── fitness_fn_wrapper.rs # existing
└── surrogate.rs        # NEW: SurrogateModel<U> trait

src/engines/
└── ga.rs               # add surrogate field, with_surrogate(), prescreening step

src/stats.rs            # add true_fitness_calls: Option<u64>
src/lib.rs              # add pub use fitness::SurrogateModel

tests/
└── test_surrogate.rs   # NEW: all SC-1*/SC-2*/SC-3 tests (flat path — no subdirectory)
```

### Pattern 1: SurrogateModel Trait Definition

Directly mirrors `BatchFitnessEvaluator` in `src/fitness/batch.rs`. [VERIFIED: codebase inspection]

```rust
// Source: src/fitness/batch.rs (model) → new src/fitness/surrogate.rs
use crate::traits::ChromosomeT;

pub trait SurrogateModel<U: ChromosomeT>: Send + Sync {
    fn predict(&self, chromosome: &U) -> f64;
}
```

### Pattern 2: Ga<U> Field and Builder Method

Mirrors `batch_evaluator` field and `with_batch_evaluator()` builder. [VERIFIED: codebase inspection of ga.rs lines 289, 950-956]

```rust
// In Ga<U> struct:
surrogate: Option<(Arc<dyn SurrogateModel<U> + Send + Sync>, f64)>,
//                                                            ^^^^ prescreening_fraction

// Builder method:
pub fn with_surrogate(
    mut self,
    model: Arc<dyn SurrogateModel<U> + Send + Sync>,
    prescreening_fraction: f64,
) -> Self {
    self.surrogate = Some((model, prescreening_fraction));
    self
}
```

Storing the fraction inline in the tuple avoids adding a field to `GaConfiguration` (the fraction is only meaningful when a surrogate is present). This is within Claude's discretion.

### Pattern 3: Prescreening Step Insertion Point

Inserted in the generation loop in `src/engines/ga.rs`, immediately after `parent_crossover()` returns `offspring` and before the existing batch-evaluate block at line 1712. [VERIFIED: codebase inspection]

```rust
// After: let mut offspring = parent_crossover(...)?;
// Before: if let Some(eval) = self.batch_evaluator.as_ref().map(Arc::clone) { ... }

let true_fitness_calls: Option<u64> = if let Some((ref surrogate, fraction)) = self.surrogate {
    let n = offspring.len();
    if n > 0 {
        // Compute predicted scores; treat NaN as worst (f64::NEG_INFINITY)
        let mut scores: Vec<(usize, f64)> = offspring
            .iter()
            .enumerate()
            .map(|(i, c)| (i, {
                let s = surrogate.predict(c);
                if s.is_nan() { f64::NEG_INFINITY } else { s }
            }))
            .collect();

        // Sort descending by predicted score (highest = most promising)
        scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let keep = ((n as f64 * fraction).floor() as usize).max(1);
        scores.truncate(keep);

        // Rebuild offspring in original order (stable relative order)
        scores.sort_unstable_by_key(|&(idx, _)| idx);
        let surviving: Vec<U> = scores.into_iter().map(|(idx, _)| offspring[idx].clone()).collect();
        offspring = surviving;
    }
    Some(offspring.len() as u64)
} else {
    None
};
```

Note: Sequential sort only — no `par_iter`. Offspring count is typically small (< population_size). This is unconditionally WASM-safe. [VERIFIED: codebase inspection of WASM gates in ga.rs]

### Pattern 4: GenerationStats Field Addition

Follows the `cache_hits` / `cache_misses` pattern exactly. [VERIFIED: codebase inspection of stats.rs lines 57-65]

```rust
// In GenerationStats struct:
/// Number of offspring that reached true fitness evaluation in this generation.
///
/// `None` when no surrogate is configured. When set, equals the count of
/// offspring that survived surrogate prescreening and proceeded to actual
/// fitness evaluation (post-prescreening offspring count).
#[cfg_attr(feature = "serde", serde(default))]
pub true_fitness_calls: Option<u64>,
```

The `serde(default)` attribute is mandatory for backward-compatible deserialization of existing checkpoints. [VERIFIED: existing pattern in stats.rs]

Must also update `GenerationStats::from_fitness_values()` to initialize the field as `None` (same as `cache_hits` / `cache_misses`). [VERIFIED: stats.rs line 90-91]

### Pattern 5: Assigning true_fitness_calls to gen_stats

The count is captured at the prescreening step (into `true_fitness_calls: Option<u64>`) and written into `gen_stats` after `from_fitness_values()` constructs it, mirroring the cache delta pattern at lines 2080-2085:

```rust
// After: let mut gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);
gen_stats.true_fitness_calls = true_fitness_calls;
```

### Pattern 6: lib.rs Re-export

```rust
// In src/lib.rs, after: pub use fitness::BatchFitnessEvaluator;
pub use fitness::SurrogateModel;
```

### Pattern 7: build() Validation

Validate `prescreening_fraction` in `build()` inline with the existing batch/fitness mutual-exclusivity check at lines 778-784. [VERIFIED: codebase inspection]

```rust
if let Some((_, fraction)) = &self.surrogate {
    if *fraction <= 0.0 || *fraction > 1.0 {
        return Err(GaError::ConfigurationError(
            "prescreening_fraction must be in (0.0, 1.0]".to_string(),
        ));
    }
}
```

### Anti-Patterns to Avoid

- **Using `par_iter` for the prescreening sort:** Offspring batch is small; adding a WASM cfg gate for a micro-optimization adds complexity for no measurable benefit. Sort sequentially.
- **Counting `true_fitness_calls` from within `batch_evaluate()`:** The count belongs at the prescreening boundary (post-truncation offspring length), not inside the batch evaluator. The evaluator may not run at all if every survivor is a cache hit.
- **Re-screening the existing population:** D-06 is explicit — only offspring are prescreened. Filtering `self.population.chromosomes` would break the semantics.
- **Storing `prescreening_fraction` in `GaConfiguration`:** The fraction is only meaningful when a surrogate exists. Storing it inside the surrogate tuple keeps the configuration clean and avoids a default-value decision for non-surrogate runs.
- **Panicking on NaN surrogate predictions:** User models may produce NaN for degenerate inputs. Silent treatment as worst score is more robust than panic.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| NaN-safe descending sort | Custom comparator with explicit NaN branches | `f64::NEG_INFINITY` substitution + `partial_cmp().unwrap()` | After replacing NaN, `partial_cmp` is always `Some`; no panic path |
| Thread-safe surrogate storage | `Mutex<Box<dyn ...>>` | `Arc<dyn SurrogateModel<U> + Send + Sync>` | Same pattern as `batch_evaluator` and `observer`; zero overhead when `None` |
| Prescreening fraction validation | Runtime assertion in hot path | `build()` validation at construction time | Follows existing validation pattern; fails fast before any generation runs |

**Key insight:** The surrogate integration is deliberately thin — it is a filter on an existing slice, not a new subsystem. All the hard infrastructure (batch evaluation, cache, observer notification, stats) already exists from Phase 60.

---

## Common Pitfalls

### Pitfall 1: Inserting Prescreening After Repair/Constraints

**What goes wrong:** If surrogate prescreening fires after repair or constraint penalty is applied, the surrogate sees post-repair DNA (good), but the repair operator and constraint logic have already run on offspring that will be dropped — wasted compute.

**Why it happens:** The natural reading of the ga.rs hot path is "do everything to offspring then filter"; but the design intent (D-08) is to filter first so repair/constraints run on fewer individuals.

**How to avoid:** Insert the prescreening block immediately after `parent_crossover()` returns `offspring` and before the batch-evaluate block (line 1711). Repair (line 1728) and constraints (line 1736) naturally follow.

**Warning signs:** Test that repair count drops when surrogate fraction is 0.5 — if repair runs the same number of times regardless of fraction, the insertion point is wrong.

### Pitfall 2: Counting true_fitness_calls from Cache Misses

**What goes wrong:** Counting only cache misses as "true fitness calls" gives a different semantic from what D-10 specifies. D-10 says: count of offspring that reached the evaluation path (post-prescreening), not just cache misses.

**Why it happens:** The cache delta tracking code in ga.rs is nearby; it is tempting to derive `true_fitness_calls` from it.

**How to avoid:** Capture `offspring.len()` immediately after truncation, before the cache check. This gives the "offspring that could have been evaluated" count (the ones that weren't rejected), which is the intended metric.

### Pitfall 3: Forgetting `serde(default)` on the New GenerationStats Field

**What goes wrong:** Old checkpoints deserialized with serde fail because the new field is missing from the JSON.

**Why it happens:** Easy to forget when adding a new optional field.

**How to avoid:** Check `src/stats.rs` — every optional or new field since Phase 60 carries `#[cfg_attr(feature = "serde", serde(default))]`. Apply the same attribute.

### Pitfall 4: fitness module re-export missing

**What goes wrong:** Users cannot write `use genetic_algorithms::SurrogateModel` and get a confusing path error.

**Why it happens:** New trait files in `src/fitness/` must be explicitly re-exported; the module is `pub mod fitness` but individual items require `pub use`.

**How to avoid:** Add `pub use fitness::SurrogateModel;` to `src/lib.rs` immediately after the existing `pub use fitness::BatchFitnessEvaluator;` line.

### Pitfall 5: `Default` impl for Ga<U> omits new field

**What goes wrong:** The `Default` impl for `Ga<U>` (ga.rs line 361) does not include the new `surrogate` field, causing a compile error or incorrect behavior.

**Why it happens:** `Default` lists each field explicitly; adding a struct field without updating `Default` is a compile error in Rust.

**How to avoid:** Add `surrogate: None` to the `Default` impl alongside `batch_evaluator: None` and `observer: None`.

---

## Code Examples

### Minimal SurrogateModel User Implementation

```rust
// Source: pattern from src/fitness/batch.rs (user-side usage)
use genetic_algorithms::{SurrogateModel, chromosomes::Range as RangeChromosome};
use std::sync::Arc;

struct LinearSurrogate { coeffs: Vec<f64> }

impl SurrogateModel<RangeChromosome<f64>> for LinearSurrogate {
    fn predict(&self, chromosome: &RangeChromosome<f64>) -> f64 {
        chromosome.dna()
            .iter()
            .zip(&self.coeffs)
            .map(|(g, c)| g.value() * c)
            .sum()
    }
}

let ga = Ga::new()
    .with_fitness_fn(|dna: &[_]| expensive_sim(dna))
    .with_surrogate(Arc::new(LinearSurrogate { coeffs: vec![1.0; 10] }), 0.5)
    // ... rest of builder
    .build()?;
```

### Prescreening Sort with NaN Safety

```rust
// Source: std (sort_unstable_by, f64)
let mut scores: Vec<(usize, f64)> = offspring
    .iter()
    .enumerate()
    .map(|(i, c)| {
        let s = surrogate.predict(c);
        (i, if s.is_nan() { f64::NEG_INFINITY } else { s })
    })
    .collect();
scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
let keep = ((offspring.len() as f64 * fraction).floor() as usize).max(1);
scores.truncate(keep);
scores.sort_unstable_by_key(|&(idx, _)| idx); // restore original order
let offspring: Vec<U> = scores.into_iter().map(|(idx, _)| offspring[idx].clone()).collect();
```

### GenerationStats Field Addition (serde pattern)

```rust
// Source: src/stats.rs lines 57-65 (cache_hits/cache_misses pattern)
#[cfg_attr(feature = "serde", serde(default))]
pub true_fitness_calls: Option<u64>,
```

And in `from_fitness_values()`:
```rust
// In GenerationStats { ... } constructor block — add alongside cache_hits: None
true_fitness_calls: None,
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All offspring unconditionally evaluated | Surrogate pre-screens; top fraction proceeds | Phase 62 (this phase) | Reduces true fitness calls proportional to `1 - prescreening_fraction` |
| `GenerationStats` had no fitness call count | `true_fitness_calls: Option<u64>` added | Phase 62 (this phase) | Users can track surrogate efficiency per generation via GaObserver |

**Deprecated/outdated:** Nothing deprecated. Phase 62 is additive only.

---

## Open Questions (RESOLVED)

1. **Ordering of survivors when prescreening_fraction produces float rounding** — RESOLVED: tie-breaking is non-deterministic by design (sort_unstable_by). Documented in trait rustdoc and Plan 01 Task 1 action.
   - What we know: `max(1, floor(n * fraction))` is deterministic per D-05.
   - What's unclear: Whether the planner should document the tie-breaking behavior (sort is unstable; equal surrogate scores may reorder between runs).
   - Recommendation: Use `sort_unstable_by` consistently and note in doc comments that tie-breaking is not guaranteed. This is acceptable — surrogate scores are continuous.

2. **fitness/mod.rs public re-export of surrogate.rs** — RESOLVED: there is no src/fitness/mod.rs — confirmed by codebase inspection in Plan 01 context. `src/fitness.rs` is the facade module with explicit `pub mod ...; pub use ...::...;` per file. Add `pub mod surrogate;` and `pub use surrogate::SurrogateModel;` there.
   - What we know: `src/fitness/` contains `batch.rs`, `cache.rs`, etc. There may be a `mod.rs` controlling visibility.
   - What's unclear: Whether `surrogate.rs` needs to be declared in a `fitness/mod.rs` first or if the flat `pub mod fitness` in `lib.rs` covers all files.
   - Recommendation: Check `src/fitness/mod.rs` (or equivalent) during Wave 0. If a `mod.rs` exists, add `pub mod surrogate;` to it. If no `mod.rs` (all files declared individually in `lib.rs`), add `pub mod surrogate;` under `pub mod fitness` or adjust the `#[path]` pattern as needed.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 62 is a pure code/config change within the existing Rust project. No external tools, services, CLIs, runtimes, or databases beyond the project's existing toolchain are required.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | Cargo.toml (no separate test config) |
| Quick run command | `cargo test surrogate` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1a | `SurrogateModel::predict` trait is implementable and callable | unit | `cargo test --test test_surrogate test_predict_called` | ❌ Wave 0 |
| SC-1b | `.with_surrogate(model, 0.5)` wires model onto Ga and runs | integration | `cargo test --test test_surrogate ga_with_surrogate_runs` | ❌ Wave 0 |
| SC-1c | Only top fraction of offspring reach fitness evaluation | integration | `cargo test --test test_surrogate prescreening_fraction_reduces_evaluations` | ❌ Wave 0 |
| SC-1d | `max(1, floor(n * fraction))` minimum floor holds | unit | `cargo test --test test_surrogate test_prescreening_floor` | ❌ Wave 0 |
| SC-1e | prescreening_fraction=0.0 rejected at build() | unit | `cargo test --test test_surrogate invalid_fraction_zero_rejected` | ❌ Wave 0 |
| SC-1f | prescreening_fraction>1.0 rejected at build() | unit | `cargo test --test test_surrogate invalid_fraction_over_one_rejected` | ❌ Wave 0 |
| SC-1g | NaN surrogate prediction treated as worst score (not panic) | unit | `cargo test --test test_surrogate test_nan_prediction_treated_as_worst` | ❌ Wave 0 |
| SC-2a | `GenerationStats.true_fitness_calls` is Some(n) when surrogate configured | integration | `cargo test --test test_surrogate true_fitness_calls_populated_in_stats` | ❌ Wave 0 |
| SC-2b | `GenerationStats.true_fitness_calls` is None when no surrogate | integration | `cargo test --test test_surrogate true_fitness_calls_none_without_surrogate` | ❌ Wave 0 |
| SC-2c | `true_fitness_calls` deserializes with serde default (backward compat) | unit | `cargo test --test test_surrogate --features serde stats_serde_default` | ❌ Wave 0 |
| SC-3 | Surrogate composes with BatchFitnessEvaluator (D-09) | integration | `cargo test --test test_surrogate surrogate_with_batch_evaluator_composes` | ❌ Wave 0 |
| SC-3w | `cargo check --target wasm32-unknown-unknown` passes | build check | `cargo check --target wasm32-unknown-unknown` | ❌ Wave 0 CI gate |

### Sampling Rate

- **Per task commit:** `cargo test surrogate`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green + `cargo check --target wasm32-unknown-unknown` before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_surrogate.rs` (flat path — Cargo discovers tests at `tests/*.rs` per existing project layout; no subdirectory) — covers all of SC-1a through SC-3:
  - SC-1a, SC-1d, SC-1g, SC-2c implementable in Plan 01 (trait-only, pure-math, serde-only — pass immediately)
  - SC-1b, SC-1c, SC-1e, SC-1f, SC-2a, SC-2b, SC-3 stubbed in Plan 01 with `#[ignore]`, activated in Plan 02 once `Ga::with_surrogate` exists
- [ ] `cargo check --target wasm32-unknown-unknown` — CI gate; verify locally before Phase gate

---

## Security Domain

Security enforcement is not relevant for this phase. Phase 62 is an internal performance optimization (offspring filtering) with no authentication, session management, access control, input from external sources, or cryptographic operations. All inputs are internal GA data structures (`&U` chromosome references). No ASVS categories apply.

---

## Sources

### Primary (HIGH confidence)

- Codebase: `src/fitness/batch.rs` — `BatchFitnessEvaluator<U>` — direct model for `SurrogateModel<U>` trait definition [VERIFIED: codebase inspection]
- Codebase: `src/stats.rs` lines 57-65 — `cache_hits`/`cache_misses` pattern for `true_fitness_calls` field [VERIFIED: codebase inspection]
- Codebase: `src/engines/ga.rs` lines 1688-1714 — offspring collection and batch-evaluate insertion point [VERIFIED: codebase inspection]
- Codebase: `src/engines/ga.rs` lines 2080-2085 — cache delta tracking pattern for gen_stats [VERIFIED: codebase inspection]
- Codebase: `src/engines/ga.rs` lines 778-784 — build() validation pattern [VERIFIED: codebase inspection]
- Codebase: `src/lib.rs` line 342 — BatchFitnessEvaluator re-export pattern [VERIFIED: codebase inspection]
- Context: `62-CONTEXT.md` — All locked decisions D-01 through D-11 [VERIFIED: document read]

### Secondary (MEDIUM confidence)

- None required — all decisions are locked and codebase is the authoritative reference.

### Tertiary (LOW confidence)

- None.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `src/fitness/` files are declared via a module mechanism that will accept `surrogate.rs` without a separate `mod.rs` entry | Open Questions #2 | Compile error if a `mod.rs` exists that requires explicit `pub mod surrogate;` |

**All other claims in this research were verified by direct codebase inspection.**

---

## Metadata

**Confidence breakdown:**
- Trait definition and placement: HIGH — direct model in same module exists
- Hot-path insertion point: HIGH — exact lines identified in ga.rs
- GenerationStats field pattern: HIGH — exact pattern in stats.rs verified
- Test structure: HIGH — existing Phase 60 tests provide direct model
- Validation pattern: HIGH — build() mutation-exclusivity check is the model

**Research date:** 2026-06-09
**Valid until:** Until ga.rs hot-path or stats.rs structure changes (stable; 90 days)
