# Phase 6: Diversity Estimation - Research

**Researched:** 2026-03-20
**Domain:** Rust genetic algorithms — diversity metric, stats, adaptive subsystems
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DIV-01 | User can read a diversity metric from per-generation statistics | Add `diversity: f64` field to `GenerationStats`; compute it in `from_fitness_values` or via a dedicated function called in `ga.rs` |
| DIV-02 | Extension strategies use the diversity metric to determine when to trigger | Replace the inline fitness-std-dev computation in `ga.rs` with the `GenerationStats.diversity` value; compare against `ExtensionConfiguration.diversity_threshold` |
| DIV-03 | Dynamic mutation probability uses the diversity metric for adjustment decisions | Replace `compute_cardinality` call in `ga.rs` with a single diversity value drawn from per-generation stats |
</phase_requirements>

---

## Summary

Phase 6 is entirely about making population diversity a first-class `f64` value that lives in `GenerationStats` and flows through the two existing adaptive subsystems (extension trigger and dynamic mutation probability). There is no new operator, no new configuration struct, and no new trait — the work is a targeted surgical edit to three source files.

**Current situation:** Two separate, inconsistent diversity signals exist. The extension trigger computes fitness standard deviation inline in `ga.rs` (lines 807–820) and compares it to `ExtensionConfiguration.diversity_threshold`. Dynamic mutation calls `mutation::compute_cardinality` (unique fitness ratio) on the same chromosome slice. Neither value is exposed to users. Both computations happen inside the hot loop with their own local bindings — they do not share data.

**After phase 6:** A single `diversity: f64` field in `GenerationStats` is computed once per generation. Both adaptive subsystems read it from the stats struct. The callback receives the value via `gen_stats`. The user can read it from `ga.stats()`.

**Primary recommendation:** Use fitness standard deviation as the diversity metric. It is already the signal used by the extension trigger, it matches the existing `diversity_threshold` field semantics, it has zero new dependencies, and it requires no `Hash`/`Eq` bounds on `GeneT`. Cardinality (unique-fitness ratio) should be retired from dynamic mutation and replaced with this same signal for consistency.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std | — | `f64` arithmetic, iterators | No external dep needed for std-dev computation |
| (no new deps) | — | — | All math is integer/float operations on existing `Vec<f64>` |

No new Cargo dependencies are required for this phase.

---

## Architecture Patterns

### Current Code Paths

#### Extension trigger (ga.rs lines 804–884)
```rust
// Current: ad-hoc inline computation
let fitness_vals: Vec<f64> = self.population.chromosomes.iter().map(|c| c.fitness()).collect();
let n = fitness_vals.len() as f64;
if n > 1.0 {
    let avg = fitness_vals.iter().sum::<f64>() / n;
    let variance = fitness_vals.iter().map(|f| (f - avg).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    if std_dev < ext_config.diversity_threshold { /* trigger */ }
}
```

#### Dynamic mutation (ga.rs lines 762–802)
```rust
// Current: cardinality = unique-fitness-count / population-size
let cardinality = mutation::compute_cardinality(&self.population.chromosomes);
self.dynamic_mutation_probability = mutation::dynamic_probability(
    self.dynamic_mutation_probability, cardinality, target, step, p_max, p_min,
);
```

#### Stats collection (ga.rs lines 974–983)
```rust
// Current: fitness values collected separately — a second pass over chromosomes
let fitness_values: Vec<f64> = self.population.chromosomes.iter().map(|c| c.fitness()).collect();
let gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);
```

### Recommended Project Structure (no change)
```
src/
├── stats.rs          # ADD: diversity field + computation
├── ga.rs             # CHANGE: compute stats first, use stats.diversity downstream
└── operations/
    └── mutation.rs   # RETAIN: compute_cardinality still exists but ga.rs switches away from it
```

### Pattern 1: Stats-first ordering
**What:** Move the stats collection block to BEFORE the extension trigger and dynamic mutation blocks so that `gen_stats.diversity` is available when those blocks run.

**Current order (ga.rs, inside the generation loop):**
1. Selection
2. Crossover + Mutation
3. Population merge
4. Elitism
5. Survivor selection
6. Adaptive GA update
7. **Dynamic mutation update** (uses cardinality)
8. **Extension trigger** (recomputes std-dev)
9. Niching
10. Best chromosome update
11. **Stats collection** (computes std-dev again — third pass!)
12. Checkpoint
13. Callback
14. Stopping criteria

**Recommended order:**
1–10 unchanged
11. **Stats collection** ← move to here (computes diversity once)
12. **Dynamic mutation update** ← read from `gen_stats.diversity`
13. **Extension trigger** ← read from `gen_stats.diversity`
14. Niching (unchanged position)
15. Best chromosome update (unchanged)
16. Checkpoint
17. Callback (already receives `gen_stats`)
18. Stopping criteria (already uses `gen_stats.fitness_std_dev`)

Wait — the stopping criteria `ConvergenceReached` already reads `gen_stats.fitness_std_dev` (line 1061). Stats must be collected before stopping criteria, which is already the case. The only change needed is to reorder stats before dynamic mutation and extension. There is a subtlety: best chromosome update (step 5 above) is currently before stats. That is fine — best chromosome update does not depend on stats.

**Concrete reordering summary:**
- Move stats collection to immediately after best chromosome update and BEFORE dynamic mutation update.
- Derive `diversity` inside `GenerationStats::from_fitness_values` (or pass it in alongside fitness values).

### Pattern 2: Diversity field in GenerationStats
**What:** Add `pub diversity: f64` to `GenerationStats`. Populate it from `from_fitness_values`.

**The metric:** fitness standard deviation (population variance sqrt). This is:
- Already computed inside `from_fitness_values` (it produces `fitness_std_dev`)
- Identical to the extension trigger's existing computation
- Unit-consistent with `diversity_threshold` in `ExtensionConfiguration`
- Generic — works for any chromosome type without `Hash`/`Eq` bounds on `GeneT`

**Implementation:** `diversity` equals `fitness_std_dev`. No second pass needed.

```rust
// src/stats.rs — after change
pub struct GenerationStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub avg_fitness: f64,
    pub fitness_std_dev: f64,
    pub population_size: usize,
    /// Population diversity metric: standard deviation of fitness values.
    /// Higher values indicate more diverse populations.
    pub diversity: f64,
}
```

Inside `from_fitness_values`, set `diversity: std_dev` (same value as `fitness_std_dev`). The two fields have different conceptual roles: `fitness_std_dev` is the raw statistical measure; `diversity` is the semantic name the user reads and the subsystems act on. Starting with them equal is consistent and can be evolved later.

### Pattern 3: Dynamic mutation reads diversity, not cardinality
**What:** The `dynamic_probability` function signature stays unchanged. The caller in `ga.rs` passes `gen_stats.diversity` instead of `compute_cardinality(...)`.

**Tradeoff:** `compute_cardinality` counts unique fitness values; `fitness_std_dev` measures spread. They move in the same direction (both low when the population converges) but have different scales. The `target_cardinality` config field will need to be re-interpreted as a "target diversity" floor — or the field can be renamed or joined to a shared `target_diversity` concept. Because no breaking changes are allowed, the cleanest approach is to keep `target_cardinality` but document it as target diversity threshold when dynamic mutation reads std-dev. The builder method `with_mutation_target_cardinality` is still valid API; only the internal signal changes.

### Anti-Patterns to Avoid
- **Recomputing fitness values three times per generation:** The current code does this. After the change, fitness values are collected once (for stats), and diversity is read from stats.
- **Storing diversity on Population:** Population already has `f_avg` and `f_max` (for adaptive GA). Adding `diversity` there would duplicate the stats struct responsibility. Keep it in `GenerationStats`.
- **HyperLogLog or hash-based approaches:** These require `Hash` on `GeneT`, which is not a current bound. Fitness std-dev avoids touching the trait bounds.
- **Gene-level Hamming distance:** O(n²) for n chromosomes × L genes. Scales poorly for large populations or long chromosomes. Fitness std-dev is O(n).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Standard deviation | Custom rolling variance | Existing `from_fitness_values` already computes it | Already in codebase, already correct |
| Serde for new field | Custom Serialize/Deserialize | `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on `GenerationStats` (already present) | Adding a field to a derived struct auto-includes it |

**Key insight:** The entire phase is wiring, not algorithm work. The math is already present; the task is to expose and route the value correctly.

---

## Common Pitfalls

### Pitfall 1: Serde round-trip breakage for existing checkpoints
**What goes wrong:** Adding a new `pub diversity: f64` field to `GenerationStats` — which already carries `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` — means any saved checkpoint JSON files that do not have the `diversity` key will fail to deserialize.
**Why it happens:** serde derives require all fields present by default.
**How to avoid:** Add `#[serde(default)]` on the `diversity` field. This makes deserialization of old checkpoints succeed with `diversity = 0.0`. The `test_serde.rs::serde_generation_stats` test (line 234) already accesses fields by name — it will compile fine once the field is added, but must be updated to also assert `diversity`.
**Warning signs:** `cargo test --features serde` fails with "missing field `diversity`".

### Pitfall 2: Loop ordering — stats before or after subsystems?
**What goes wrong:** If stats are collected after dynamic mutation and extension, those subsystems still compute their own signals inline and `gen_stats.diversity` is stale (zero or wrong) when passed to the callback.
**Why it happens:** The current loop puts stats collection at step 11 (after dynamic mutation at step 7 and extension at step 8).
**How to avoid:** Collect stats immediately after best-chromosome update (before dynamic mutation and extension). The stopping-criteria blocks at the bottom already read from `gen_stats` — this is already correct.
**Warning signs:** Callback receives `gen_stats.diversity = 0.0` for the first generation.

### Pitfall 3: `fitness_std_dev` vs `diversity` — two fields, same value
**What goes wrong:** Confusion about why `fitness_std_dev == diversity`. Someone assumes diversity is a richer metric and is disappointed.
**How to avoid:** Clear rustdoc on both fields explaining that `diversity` is the semantic alias for population-level diversity and `fitness_std_dev` is retained for backward compatibility. They are intentionally equal in v2.2.
**Warning signs:** PR review confusion about duplication.

### Pitfall 4: Dynamic mutation target scale mismatch
**What goes wrong:** `target_cardinality` was calibrated as a ratio in `[0.0, 1.0]` (e.g., `0.5` = half the population has distinct fitness). Fitness std-dev is unbounded and problem-specific. A `target_cardinality = 0.5` makes no sense as a diversity threshold in std-dev units.
**Why it happens:** The two signals have different scales.
**How to avoid:** Document the behavior change. The `target_cardinality` field now acts as a "minimum acceptable diversity (std-dev)" floor. Users who previously set `target_cardinality = 0.5` will need to recalibrate. Add a doc note to `with_mutation_target_cardinality` explaining this. Alternatively, expose `target_diversity` as a new builder method that sets the same underlying field. This is the preferred approach since it gives new users a well-named API without breaking the old name (it just remains as an alias).

### Pitfall 5: Forgetting the `from_fitness_values(generation, &[], ...)` empty-population branch
**What goes wrong:** The empty-population early return in `from_fitness_values` must also set `diversity: 0.0`.
**Why it happens:** Adding a new field requires updating all construction sites.
**Warning signs:** Compiler error "missing field `diversity` in struct initializer" — the compiler catches this automatically. Make sure to fix both `GenerationStats { ... }` literal sites in `from_fitness_values`.

---

## Code Examples

### Adding diversity to GenerationStats
```rust
// src/stats.rs
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenerationStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub avg_fitness: f64,
    pub fitness_std_dev: f64,
    pub population_size: usize,
    /// Population diversity: standard deviation of fitness values.
    /// Equal to `fitness_std_dev` in v2.2. Higher = more diverse.
    #[cfg_attr(feature = "serde", serde(default))]
    pub diversity: f64,
}

// In from_fitness_values, set:
//   diversity: std_dev,
//   fitness_std_dev: std_dev,
// (both from the same computed value)
```

### Stats-first ordering in ga.rs
```rust
// BEFORE dynamic mutation and extension blocks:
let fitness_values: Vec<f64> = self.population.chromosomes.iter().map(|c| c.fitness()).collect();
let gen_stats = GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);
self.stats.push(gen_stats.clone());

// Dynamic mutation — use gen_stats.diversity instead of compute_cardinality:
if self.configuration.mutation_configuration.dynamic_mutation {
    let target = self.configuration.mutation_configuration.target_cardinality.unwrap_or(0.5);
    let step   = self.configuration.mutation_configuration.probability_step.unwrap_or(0.01);
    let p_max  = self.configuration.mutation_configuration.probability_max.unwrap_or(1.0);
    let p_min  = self.configuration.mutation_configuration.probability_min.unwrap_or(0.0);
    self.dynamic_mutation_probability = mutation::dynamic_probability(
        self.dynamic_mutation_probability,
        gen_stats.diversity,  // was: compute_cardinality(&self.population.chromosomes)
        target,
        step,
        p_max,
        p_min,
    );
}

// Extension trigger — use gen_stats.diversity instead of inline computation:
if let Some(ref ext_config) = self.configuration.extension_configuration {
    if ext_config.method != Extension::Noop && gen_stats.diversity < ext_config.diversity_threshold {
        // ... trigger extension
    }
}
```

### Serde backward-compat for old checkpoints
```rust
// The #[serde(default)] attribute on the diversity field ensures that
// checkpoint JSON files written before this change (which lack the field)
// deserialize successfully with diversity = 0.0.
#[cfg_attr(feature = "serde", serde(default))]
pub diversity: f64,
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No diversity in stats | `diversity: f64` in `GenerationStats` | Phase 6 | User can log and chart diversity |
| Extension trigger recomputes std-dev inline | Reads `gen_stats.diversity` | Phase 6 | Single computation, consistent signal |
| Dynamic mutation uses cardinality ratio | Uses `gen_stats.diversity` (std-dev) | Phase 6 | Consistent signal across subsystems |
| `compute_cardinality` called in ga.rs | No longer called in ga.rs hot loop | Phase 6 | Function stays in codebase; ga.rs stops calling it |

**Deprecated/outdated (after this phase):**
- Inline extension std-dev computation (lines 807–820 in ga.rs): replaced by reading `gen_stats.diversity`.
- `compute_cardinality` call in ga.rs dynamic mutation block: replaced by reading `gen_stats.diversity`. The `compute_cardinality` function itself is NOT deleted — it remains in the public API and is tested.

---

## Open Questions

1. **Should `diversity` be `fitness_std_dev` or something richer?**
   - What we know: fitness std-dev is already computed, generic, O(n), and matches the existing extension threshold semantics.
   - What's unclear: Whether users would prefer a normalized metric (e.g., coefficient of variation: std-dev / mean) or a gene-level metric (Hamming distance-based) for better intuition.
   - Recommendation: Ship fitness std-dev in v2.2. The field is public and the computation is isolated in `stats.rs` — it can be replaced or extended in a later phase without breaking the field name.

2. **Should `diversity` equal `fitness_std_dev`, or should `fitness_std_dev` be removed?**
   - What we know: `fitness_std_dev` is tested in `test_stats.rs` and in `test_serde.rs`. Removing it would be a breaking change.
   - What's unclear: Long-term, the two fields with identical values create maintenance confusion.
   - Recommendation: Keep both for v2.2. They are intentionally equal. Add rustdoc cross-reference. Plan removal of `fitness_std_dev` for a future breaking-change milestone.

3. **Should `compute_cardinality` be deprecated or removed from the public API?**
   - What we know: It is `pub` in `operations::mutation` and tested in `test_mutation_dynamic.rs`.
   - Recommendation: Retain it as-is — it is a useful utility. Simply stop calling it in `ga.rs`. No deprecation annotation needed for v2.2.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` |
| Config file | none — `cargo test` |
| Quick run command | `cargo test test_stats` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DIV-01 | `gen_stats.diversity` is a non-negative `f64` set from fitness std-dev | unit | `cargo test test_stats` | ✅ `tests/test_stats.rs` (needs new assertions) |
| DIV-01 | `diversity` serializes/deserializes round-trip with `#[serde(default)]` | unit | `cargo test --features serde serde_generation_stats` | ✅ `tests/test_serde.rs` (needs update) |
| DIV-02 | Extension triggers when `gen_stats.diversity < threshold`, not when inline std-dev triggers | integration | `cargo test test_extension` | ✅ `tests/test_extension.rs` (needs new case) |
| DIV-03 | Dynamic mutation reads `gen_stats.diversity` (std-dev), not cardinality ratio | unit | `cargo test test_mutation_dynamic` | ✅ `tests/operations/test_mutation_dynamic.rs` (existing tests remain valid) |
| DIV-01 | `ga.stats()` returns stats with `diversity > 0.0` after a multi-generation run | integration | `cargo test test_ga` | ✅ `tests/test_ga.rs` (needs new assertion) |

### Sampling Rate
- **Per task commit:** `cargo test test_stats test_ga test_extension test_mutation_dynamic`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers all phase requirements. No new test files needed; existing files need targeted additions.

---

## Sources

### Primary (HIGH confidence)
- Direct source read: `src/stats.rs` — `GenerationStats` struct and `from_fitness_values` constructor
- Direct source read: `src/ga.rs` — full generation loop, stats collection (line 974), dynamic mutation block (line 762), extension trigger block (line 804)
- Direct source read: `src/operations/mutation.rs` — `compute_cardinality`, `dynamic_probability`
- Direct source read: `src/extension/configuration.rs` — `ExtensionConfiguration.diversity_threshold`
- Direct source read: `src/configuration.rs` — `MutationConfiguration.target_cardinality`
- Direct source read: `src/checkpoint.rs` — `Checkpoint` holds `Vec<GenerationStats>` — serde impact confirmed
- Direct source read: `tests/test_serde.rs` — `serde_generation_stats` test at line 234 — must be updated

### Secondary (MEDIUM confidence)
- None required — all findings are direct code reads.

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all math already in codebase
- Architecture: HIGH — full source read of all affected files
- Pitfalls: HIGH — serde backward-compat and loop-ordering issues verified directly from source
- Test map: HIGH — all test files read and confirmed to exist

**Research date:** 2026-03-20
**Valid until:** 2026-09-20 (stable codebase; no fast-moving dependencies)
