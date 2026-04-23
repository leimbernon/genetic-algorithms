# Phase 24: Minor Improvements - Research

**Researched:** 2026-04-04
**Domain:** Rust allocation reduction — ownership moves, O(n) partitioning, Arc sharing
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **MISC-01 (stats push)**: Push-last pattern — use `gen_stats` throughout the generation loop, move it via `self.stats.push(gen_stats)` at the end. Set `gen_stats.dynamic_mutation_probability = Some(new_p)` before pushing (eliminates `stats.last_mut()`). Observer call uses `self.stats.last().unwrap()`.
- **MISC-02 (truncation)**: Replace `indexed.sort_by(...)` with `select_nth_unstable_by()`. Elite member trace log drops the rank number — logs `"Elite member -> index {} fitness {}"` without rank.
- **MISC-03 (best-chromosome scan)**: `fitness_calculation()` already finds `best_idx` internally; planner should ensure that result propagates without a second scan. No user preference on exact approach.
- **MISC-04 (island migration sort)**: Replace `indices.sort_by(...)` in `select_best()` and `replace_worst()` with `select_nth_unstable_by()`.
- **MISC-05 (migrant sharing)**: Use `Arc<Vec<U>>` to share migrant data across neighbors. `replace_worst()` signature changed to accept `&[U]` — callers deref the Arc. Function stays Arc-agnostic.

### Claude's Discretion

- Variable naming and code structure within the above patterns
- MISC-03 exact implementation (already well-understood from requirements)
- Exact reborrow patterns within the stats push restructure (as long as push-last is used)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MISC-01 | `GenerationStats` moved (not cloned) when pushed to stats vec | Push-last ownership pattern; `gen_stats` is used as value throughout, moved on `push`, then borrowed via `self.stats.last().unwrap()` for remaining uses |
| MISC-02 | Truncation Selection uses `select_nth_unstable()` for O(n) partitioning | Exact same `select_nth_unstable_by` pattern used in `extract_elite` at ga.rs:1397 — direct transplant |
| MISC-03 | Best chromosome scan deduplicated — `fitness_calculation()` result reused, no redundant rescan | `fitness_values` Vec already computed at ga.rs:830; step-5 best-chromosome block at ga.rs:883 can use `fitness_values` instead of re-scanning chromosomes |
| MISC-04 | Island migration selection/replacement uses `select_nth_unstable()` instead of O(n log n) sort | `select_best()` and `replace_worst()` in migration.rs each have an `indices.sort_by(...)` — pattern mirrors extract_elite |
| MISC-05 | Island migration avoids cloning migrant vectors per neighbor topology | `all_migrants` collection loop → wrap each `Vec<U>` in `Arc`; inner distribution loop does `Arc::clone` instead of `.clone()` on the Vec |
</phase_requirements>

---

## Summary

Phase 24 completes the v2.2.1 optimization milestone by eliminating five residual allocations and unnecessary sorts across stats, truncation selection, and island migration. All five changes are pure internal refactors with no observable behavior change and no public API impact.

The techniques are all already proven in this codebase. `select_nth_unstable_by` is already used for elitism (ga.rs:1397, 1419). `Arc` sharing is already used for `FitnessFnWrapper`. The push-last ownership pattern is standard Rust. MISC-03 is the most surgical — it replaces a second linear scan with a derivation from the `fitness_values` Vec that already exists in scope. The changes are independent of each other and can be planned as separate tasks.

**Primary recommendation:** Implement each requirement as an isolated task in a single plan. No new dependencies required. The elitism code in `ga.rs` is the canonical reference for `select_nth_unstable_by` — all MISC-02 and MISC-04 implementations should mirror that pattern exactly.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::sync::Arc` | stdlib | Shared-ownership reference counting | Already used for `FitnessFnWrapper`; zero-cost ref-count bump vs Vec clone |
| `slice::select_nth_unstable_by` | stdlib | O(n) partial sort — partitions best k to front | Already used in elitism path; proven correct in this codebase |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::borrow` | stdlib | `Cow` and reborrow utilities | Already in use; push-last requires no new borrowing machinery |

**Installation:** No new dependencies required. All tools are in `std`.

---

## Architecture Patterns

### Recommended Project Structure

No structural changes. All modifications are in-place within existing files:
```
src/
├── ga.rs                          # MISC-01 (stats push), MISC-03 (best scan)
├── operations/selection/
│   └── truncation.rs              # MISC-02 (O(n) partitioning)
└── island/
    └── migration.rs               # MISC-04 (sort elimination), MISC-05 (Arc sharing)
```

### Pattern 1: Push-Last Ownership (MISC-01)

**What:** Use a value throughout a scope, move it into a collection at the end, borrow from the collection for the remaining reference.
**When to use:** When a value must be both consumed (pushed into a Vec) and borrowed (for observer notification) within the same scope, without `.clone()`.

**Current code (lines 909–911, 946–948, 1029):**
```rust
// ga.rs — CURRENT (two clones, last_mut patch-back)
let gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);
self.stats.push(gen_stats.clone());           // clone #1 — keeps gen_stats live

// ... later in dynamic_mutation block ...
if let Some(last) = self.stats.last_mut() {
    last.dynamic_mutation_probability = Some(self.dynamic_mutation_probability);
}

// ... later ...
let notify_stats = self.stats.last().cloned().unwrap_or(gen_stats.clone()); // clone #2
self.notify(|obs| obs.on_generation_end(&notify_stats));
```

**Target pattern:**
```rust
// ga.rs — TARGET (push-last, no clones)
let mut gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);

// ... dynamic_mutation block sets field directly before push ...
if self.configuration.mutation_configuration.dynamic_mutation {
    // ... compute new_p ...
    self.dynamic_mutation_probability = new_p;
    gen_stats.dynamic_mutation_probability = Some(self.dynamic_mutation_probability);
}

// ... extension and reporter blocks use gen_stats by reference as before ...

// At end of loop: move gen_stats into the vec
self.stats.push(gen_stats);

// Borrow from the vec for observer notification
self.notify(|obs| obs.on_generation_end(self.stats.last().unwrap()));
```

**Key constraint:** All existing uses of `gen_stats` (diversity check for extension trigger at line 954, reporter at line 1026, stopping criteria at lines 1116/1119/1130) must happen before the `push` call. These are all reads — compatible with owning `gen_stats` until the end.

### Pattern 2: `select_nth_unstable_by` Partial Sort (MISC-02, MISC-04)

**What:** O(n) partial sort that guarantees the k-th element is in its sorted position and all elements before it are `<=` it in the comparator order — but within those k elements, order is unspecified.
**When to use:** When only the top-k (or worst-k) elements are needed, not a fully ordered sequence.

**Canonical reference (ga.rs:1385–1399):**
```rust
// extract_elite — PROVEN pattern for "best k" selection
let mut indices: Vec<usize> = (0..chromosomes.len()).collect();
let cmp_fn = |a: &usize, b: &usize| {
    let cmp = chromosomes[*a]
        .fitness()
        .partial_cmp(&chromosomes[*b].fitness())
        .unwrap_or(std::cmp::Ordering::Equal);
    match problem_solving {
        ProblemSolving::Maximization => cmp.reverse(),
        _ => cmp,
    }
};
indices.select_nth_unstable_by(k - 1, cmp_fn);
indices.truncate(k);
// indices[0..k] are the k best indices (unordered among themselves)
```

**For `select_best()` in migration.rs — MISC-04:**
Same pattern. Comparator puts best first: Minimization = natural order (lower = better), Maximization = reversed (higher = better = `cmp.reverse()`).

**For `replace_worst()` in migration.rs — MISC-04:**
Same pattern with worst-first comparator (flip the order). Mirrors `reinsert_elite` at ga.rs:1419.

**For truncation.rs — MISC-02:**
Same pattern on `indexed: Vec<(usize, f64)>`. Comparator on the `f64` fitness component, descending. After `select_nth_unstable_by(truncation_size - 1, ...)`, slice `[..truncation_size]` contains the top-half indices (unordered). The per-rank trace log must drop rank numbering since order within the partition is undefined.

**Critical invariant:** `select_nth_unstable_by(k-1, ...)` requires `k >= 1`. All call sites already guard with `k = count.min(n)` and early-return on `n < 2`. This constraint is already satisfied.

### Pattern 3: Arc Migrant Sharing (MISC-05)

**What:** Wrap each island's migrant `Vec<U>` in `Arc<Vec<U>>` once. Each neighbor receives an `Arc::clone` (pointer bump) rather than a full `Vec<U>` clone.
**When to use:** When multiple consumers (neighbor islands) read the same data without mutating it.

**Current code (migration.rs:59–99):**
```rust
let mut all_migrants: Vec<Vec<U>> = Vec::with_capacity(num_islands);
// ...
all_migrants.push(migrants);           // Vec<U> per island

for (source_idx, source_migrants) in all_migrants.iter().enumerate() {
    for &dest_idx in &neighbors {
        let migrants = source_migrants.clone(); // full Vec<U> clone per neighbor
        replace_worst(&mut islands[dest_idx], &migrants, problem_solving);
    }
}
```

**Target pattern:**
```rust
use std::sync::Arc;

let mut all_migrants: Vec<Arc<Vec<U>>> = Vec::with_capacity(num_islands);
// ...
all_migrants.push(Arc::new(migrants));  // one allocation

for (source_idx, source_migrants) in all_migrants.iter().enumerate() {
    for &dest_idx in &neighbors {
        // Arc::clone bumps the ref-count — no Vec allocation
        let migrants_arc = Arc::clone(source_migrants);
        replace_worst(&mut islands[dest_idx], &*migrants_arc, problem_solving);
        // Debug log uses migrants_arc.len() instead of migrants.len()
    }
}
```

`replace_worst` signature changes from `migrants: &[U]` to `migrants: &[U]` — no change. The `&*migrants_arc` deref coerces `Arc<Vec<U>>` → `&Vec<U>` → `&[U]`. The function remains Arc-agnostic.

**Note:** `migrate_pareto` has the same `source_migrants.clone()` pattern at line 286. If MISC-05 scope includes Pareto migration, apply the same Arc wrapping. If scope is limited to `migrate()`, leave `migrate_pareto` unchanged. CONTEXT.md does not explicitly call out Pareto — planner should scope to `migrate()` only unless requirements say otherwise. MISC-05 in REQUIREMENTS.md says "island migration" without Pareto qualifier — conservative interpretation: only `migrate()`.

### Pattern 4: Scan-from-fitness_values (MISC-03)

**What:** Derive `best_idx` from the already-computed `fitness_values: Vec<f64>` instead of re-scanning `population.chromosomes` via `best_chromosome_index()`.
**When to use:** When fitness values have already been collected from chromosomes into a Vec and the best index is needed.

**Current flow (ga.rs:830–907):**
```
fitness_values collected (line 830)
  → niching uses fitness_values
  → step 5: best_chromosome_index(&chromosomes, ps) — second scan
  → stats: GenerationStats::from_fitness_values(i, &fitness_values, ...)
```

**Target approach:** Replace the `best_chromosome_index` call at step 5 with a scan over `fitness_values`. Since `fitness_values` is indexed 1:1 with `chromosomes`, `argmax`/`argmin` over the f64 slice gives the same `best_idx`. Eliminates the repeated chromosome iterator.

```rust
// MISC-03 target — scan fitness_values for best_idx
let best_idx = {
    let ps = self.configuration.limit_configuration.problem_solving;
    fitness_values.iter().enumerate().fold(
        None::<(usize, f64)>,
        |acc, (i, &f)| match acc {
            None => Some((i, f)),
            Some((_, best_f)) => {
                let is_better = match ps {
                    ProblemSolving::Maximization | ProblemSolving::FixedFitness => f > best_f,
                    ProblemSolving::Minimization => f < best_f,
                };
                if is_better { Some((i, f)) } else { acc }
            }
        }
    ).map(|(i, _)| i)
};
// Then use best_idx exactly as the current best_chromosome_index result is used
```

Alternatively (and simpler): keep `best_chromosome_index` but call it once inside `fitness_calculation()` which already does the work, and return or propagate that index. However, the fitness_calculation approach runs during initial setup, not in the per-generation loop. The simplest correct approach is the inline fold above, directly replacing the `best_chromosome_index` call.

### Anti-Patterns to Avoid

- **Sorting before partial use:** Calling `.sort_by()` then `.take(k)` is O(n log n) for an O(n) problem — use `select_nth_unstable_by(k-1)` then `truncate(k)`.
- **Cloning a Vec for read-only sharing:** When multiple destinations read the same data, `Arc<Vec<T>>` is O(1) per share vs O(n) per clone.
- **Patching a pushed value via `last_mut()`:** If you need to set a field after computing it but before sharing the struct, set it before the push — the `last_mut()` round-trip allocates nothing extra but adds indirection and requires the value already be moved.
- **Cloning only to defer a move:** `gen_stats.clone()` before `push` exists solely to keep the variable live. Restructuring scope to push last removes the reason.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Top-k selection from unsorted slice | Custom O(n log n) sort + take | `slice::select_nth_unstable_by` | stdlib O(n) partial sort; proven in this codebase |
| Shared read-only data across loop | Clone per iteration | `Arc::clone` | O(1) ref-count bump; no heap allocation |
| Best-index from fitness Vec | Re-scan chromosomes | Inline fold over `fitness_values` | Already-computed data; no second traversal of `Vec<U>` |

**Key insight:** All three patterns are O(1) or O(n) improvements over the current code. The complexity savings come from using the right stdlib primitive, not from algorithmic invention.

---

## Common Pitfalls

### Pitfall 1: select_nth_unstable_by with k=0 panics

**What goes wrong:** Calling `vec.select_nth_unstable_by(0, ...)` on a length-1 vec works, but the call itself requires `k < len`. If `len == 0`, it panics.
**Why it happens:** The current sort-then-take code handles empty slices implicitly (sort of empty vec is a no-op). `select_nth_unstable_by` panics on out-of-bounds index.
**How to avoid:** Guard with `if n < 2 { return ... }` (already present in truncation.rs) and `k = count.min(n)` with a `k == 0` early return (already in extract_elite). Verify these guards exist before removing the sort.
**Warning signs:** Panic on migration with `migration_count == 0` or empty island.

### Pitfall 2: Arc wrap in migrate() but not in migrate_pareto()

**What goes wrong:** MISC-05 applied to `migrate()` but `migrate_pareto()` still has `source_migrants.clone()`. No regression, but the optimization is incomplete. Tests don't cover this.
**Why it happens:** CONTEXT.md and REQUIREMENTS.md don't mention Pareto explicitly. Conservative scope is correct — don't touch `migrate_pareto()` unless the plan explicitly includes it.
**How to avoid:** Plan MISC-05 task to touch only `migrate()`. Add a note in the task comment if Pareto is deferred.

### Pitfall 3: gen_stats borrow order with the observer

**What goes wrong:** After `self.stats.push(gen_stats)`, calling `self.notify(|obs| obs.on_generation_end(self.stats.last().unwrap()))` borrows `self.stats` inside a closure that also has `&mut self`. This creates a simultaneous mutable + immutable borrow.
**Why it happens:** `self.notify` takes `&mut self` (it holds `&mut self.observer`). `self.stats.last().unwrap()` borrows `self.stats`. If `notify` is defined as `fn notify(&mut self, f: impl FnOnce(&mut dyn GaObserver))`, the borrow checker sees `&mut self` (for notify) plus `&self.stats` (for last()) in the same call.
**How to avoid:** Snapshot the reference before the call: `let stats_ref = self.stats.last().unwrap();` then pass `stats_ref` to the observer. Or clone only the stats ref. Check how `on_generation_end` is currently called at line 1030 — the existing `notify_stats` local variable pattern already works around this.
**Warning signs:** Borrow checker error: "cannot borrow `self` as mutable because it is also borrowed as immutable".

### Pitfall 4: Truncation selection correctness after unordered partition

**What goes wrong:** After `select_nth_unstable_by(truncation_size - 1, ...)`, elements at `indexed[..truncation_size]` are the top-k but in an arbitrary order. The existing test `test_truncation_selection_selects_only_from_top_half` verifies that only top-half original indices appear in results — this test must still pass. If the elite pool index slice is wrong, lower-half individuals could be selected.
**Why it happens:** Misreading `select_nth_unstable_by` semantics — the pivot element at position k is in its correct sorted position; everything before position k has comparator-value `<=` the pivot (i.e., are also "good enough"). The top-k are guaranteed to be in `[0..k]`.
**How to avoid:** After `select_nth_unstable_by(truncation_size - 1, cmp)`, verify `elite = &indexed[..truncation_size]` contains only original indices with fitness in the top half. The existing test suite covers this.
**Warning signs:** Test `test_truncation_selection_selects_only_from_top_half` fails.

### Pitfall 5: Dynamic mutation probability set on wrong gen_stats

**What goes wrong:** If `gen_stats` is moved into `self.stats` before `dynamic_mutation_probability` is set, the pushed entry has `None` for that field. The observer receives stale stats.
**Why it happens:** Setting the field on `gen_stats` must happen before `self.stats.push(gen_stats)`. With push-last, the dynamic mutation block must run before the push.
**How to avoid:** The push must be the last statement in the generation loop body (before loop iteration). All mutations to `gen_stats` happen before the push. The CONTEXT.md decision is explicit about this ordering.

---

## Code Examples

Verified patterns from this codebase:

### select_nth_unstable_by — best-k extraction (ga.rs:1385–1401)
```rust
// Source: src/ga.rs extract_elite()
let mut indices: Vec<usize> = (0..chromosomes.len()).collect();
let cmp_fn = |a: &usize, b: &usize| {
    let cmp = chromosomes[*a]
        .fitness()
        .partial_cmp(&chromosomes[*b].fitness())
        .unwrap_or(std::cmp::Ordering::Equal);
    match problem_solving {
        ProblemSolving::Maximization => cmp.reverse(),
        _ => cmp,
    }
};
indices.select_nth_unstable_by(k - 1, cmp_fn);
indices.truncate(k);
indices.iter().map(|&i| chromosomes[i].clone()).collect()
```

### select_nth_unstable_by — worst-k extraction (ga.rs:1419–1428)
```rust
// Source: src/ga.rs reinsert_elite()
chromosomes.select_nth_unstable_by(k - 1, |a, b| {
    let cmp = a
        .fitness()
        .partial_cmp(&b.fitness())
        .unwrap_or(std::cmp::Ordering::Equal);
    match problem_solving {
        ProblemSolving::Maximization => cmp,      // natural order = worst first
        _ => cmp.reverse(),                        // reversed = worst first for minimization
    }
});
// chromosomes[0..k] are the k worst (unordered)
```

### Arc sharing — FitnessFnWrapper pattern (ga.rs:988)
```rust
// Source: src/ga.rs extension regrow
let ff = self.fitness_fn.as_ref().map(Arc::clone);
// ...
if let Some(ref ff) = ff {
    let ff_clone = Arc::clone(ff);
    // ...
}
```

### Push-last with field mutation (MISC-01 target pseudocode)
```rust
let mut gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);

// set dynamic mutation field directly on gen_stats before push
if self.configuration.mutation_configuration.dynamic_mutation {
    // compute self.dynamic_mutation_probability ...
    gen_stats.dynamic_mutation_probability = Some(self.dynamic_mutation_probability);
}

// all other reads of gen_stats happen here (extension, reporter, stopping criteria)

// move gen_stats into the history vec — this is the only push
self.stats.push(gen_stats);

// borrow from vec for observer
self.notify(|obs| {
    let stats_ref = self.stats.last().unwrap();
    obs.on_generation_end(stats_ref);
});
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `sort_by` + `take` for top-k | `select_nth_unstable_by` + `truncate` | Phase 22 (elitism) | O(n log n) → O(n) |
| `Vec<U>` clone per neighbor | `Arc<Vec<U>>` + `Arc::clone` | Phase 24 (MISC-05) | O(n) clone → O(1) ref bump |
| `gen_stats.clone()` before push | move + borrow-from-vec | Phase 24 (MISC-01) | Eliminates 2 clones of GenerationStats |

**Deprecated/outdated:**
- `indices.sort_by(...)` in `select_best()` and `replace_worst()`: replaced by `select_nth_unstable_by`
- `stats.last_mut()` patch-back: replaced by setting field before push
- `stats.last().cloned()` for notify: replaced by `stats.last().unwrap()` borrow

---

## Open Questions

1. **MISC-05 scope: does it include `migrate_pareto()`?**
   - What we know: `migrate_pareto()` has the same `source_migrants.clone()` pattern (line 286)
   - What's unclear: REQUIREMENTS.md says "island migration" without qualifier; CONTEXT.md doesn't mention Pareto
   - Recommendation: Scope MISC-05 to `migrate()` only. Add a code comment noting `migrate_pareto()` as a follow-up candidate.

2. **MISC-01 borrow conflict in `notify` closure**
   - What we know: The current workaround uses a `notify_stats` local (clone). Push-last enables borrowing instead of cloning.
   - What's unclear: Whether Rust's borrow checker allows `self.stats.last().unwrap()` inside a `self.notify(...)` closure, given that `notify` takes `&mut self`.
   - Recommendation: Snapshot `let stats_ref = self.stats.last().unwrap()` before calling `self.notify`, then pass `stats_ref` into the closure. This avoids the nested `self` borrow.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none — Cargo.toml features control test gates |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MISC-01 | GenerationStats not cloned on push; dynamic_mutation_probability set before push | unit (structural) | `cargo test test_stats` | Behavioral coverage via `tests/test_ga.rs` (dynamic_mutation test at line 2022) |
| MISC-02 | Truncation selection pairs still come from top half; no sort | unit | `cargo test test_selection_truncation` | `tests/operations/test_selection_truncation.rs` ✅ |
| MISC-03 | Best chromosome scan uses fitness_values; no second pass | unit | `cargo test test_population` | `tests/test_population.rs` ✅ |
| MISC-04 | Migration selects correct best/worst individuals; no sort | unit | `cargo test test_island_migration` | `tests/island/test_island_migration.rs` ✅ |
| MISC-05 | Migrant data shared via Arc; replace_worst accepts &[U] | unit | `cargo test test_island_migration` | `tests/island/test_island_migration.rs` ✅ |

**Note on MISC-01:** There is no dedicated unit test for "GenerationStats is moved not cloned". The behavioral correctness is covered by the dynamic_mutation and full GA tests. A new test asserting `stats.last().dynamic_mutation_probability.is_some()` after a run with dynamic mutation enabled would strengthen the wave gate. This is a Wave 0 gap.

**Note on MISC-02:** `test_truncation_selection_selects_only_from_top_half` is the key regression test. It runs 200 random trials and must pass after the sort removal.

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/test_ga.rs` — add test asserting `ga.stats.last().unwrap().dynamic_mutation_probability.is_some()` after a dynamic-mutation run (covers MISC-01 field-before-push invariant)

---

## Sources

### Primary (HIGH confidence)
- `src/ga.rs` — direct code inspection of gen_stats push path (lines 909–1031), fitness_values collection (line 830), best_chromosome_index call (line 887), select_nth_unstable_by usage (lines 1397, 1419)
- `src/operations/selection/truncation.rs` — full file read; sort_by at line 51 is the MISC-02 target
- `src/island/migration.rs` — full file read; sort_by in select_best (line 114) and replace_worst (line 196) are the MISC-04 targets; Vec clone at line 79 and inner clone at line 86 are the MISC-05 targets
- `src/population.rs` — fitness_calculation and find_best_index (lines 98–192); already handles best chromosome internally
- `src/stats.rs` — GenerationStats struct and from_fitness_values (derives dynamic_mutation_probability = None by default)
- `tests/operations/test_selection_truncation.rs` — existing test coverage for MISC-02
- `tests/island/test_island_migration.rs` — existing test coverage for MISC-04 and MISC-05

### Secondary (MEDIUM confidence)
- `.planning/phases/24-minor-improvements/24-CONTEXT.md` — locked decisions from user discussion

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all tools are stdlib, already used in this codebase
- Architecture: HIGH — all patterns verified by direct code inspection
- Pitfalls: HIGH — identified from actual code paths, not speculation

**Research date:** 2026-04-04
**Valid until:** 2026-05-04 (stable — stdlib patterns don't change)
