# Phase 77: Extend Fitness Cache to More Engines - Research

**Researched:** 2026-06-19
**Domain:** Rust genetic algorithms library — fitness caching extension
**Confidence:** HIGH

## Summary

Phase 77 extends the existing `FitnessCache` LRU caching mechanism from `Ga` and `CmaEngine` to three additional engines: PSO, EDA (Bernoulli + Gaussian variants), and DE. The cache prevents redundant fitness evaluations when populations contain duplicate or near-duplicate chromosomes.

The implementation is mechanical: each engine needs the same three-part wiring pattern already proven in `Ga` (struct fields + builder method + `run()`-time wrapping + per-generation stats). No new abstractions or dependencies are needed. `FitnessCache` already satisfies WASM compatibility (no threads, no `std::time`).

**Primary recommendation:** Follow the Ga/CMA pattern exactly — add `fitness_cache_size: Option<usize>` and `fitness_cache: Option<Arc<Mutex<FitnessCache>>>` fields to each engine, add a `with_fitness_cache_size()` builder method, wrap `fitness_fn` at `run()` start, and collect per-generation delta stats. Three engines × ~5 changes each = mechanical, low-risk work.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Follow the Ga pattern: wrap fitness_fn inside `build()` (or `run()` for CMA-style engines) via `wrap_with_cache()`. The builder gets a `with_fitness_cache_size(size: usize)` method. Cache is an internal detail — users opt-in via the builder.
- **D-02:** Cache is optional per-engine (default: no cache, zero overhead). `with_fitness_cache_size()` is the builder method, consistent with Ga's API.
- **D-03:** WASM compatibility is automatic — `FitnessCache` already uses no threads and no `std::time`. No cfg-gating needed in the new engines.
- **D-04:** Expose hit/miss stats per-generation via `GenerationStats`. Call `cache_snapshot()` each generation to populate `cache_hits` and `cache_misses` fields.
- **D-05:** Stats remain `Option<u64>` — `None` when cache is disabled, `Some(hits/misses)` when enabled.
- **D-06:** No separate `cache_stats()` method on engine Result types. Stats are only in GenerationStats.
- **D-07:** Identical cache treatment for all three engines.
- **D-08:** Cache wraps the primary fitness_fn call only. For PSO, velocity-update evaluations benefit automatically.
- **D-09:** LRU eviction only — no clearing between generations.
- **D-10:** Persistent cache across generations — same `Arc<Mutex<FitnessCache>>` instance for the entire run.

### the agent's Discretion
- Whether to add a brief comment at each engine's cache wiring explaining the pattern
- Exact order of builder methods in each engine's impl block
- Whether to add a benchmark demonstrating cache benefit on a deterministic problem per engine

### Deferred Ideas (OUT OF SCOPE)
- Cache for other engines (ALPS, Cellular, HillClimb, Permutate, GP, Island, Scatter, Permutate)
- Cache invalidation strategies beyond LRU
- Distributed/remote caching
- Cache size auto-tuning
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| (none — performance, closes GitHub issue #260) | Audit which engines benefit from fitness caching and extend FitnessCache wiring beyond Ga and CmaEngine to PSO, EDA, and DE engines | Well-understood: FitnessCache is ready to use, Ga/CMA patterns are reference implementations, all three target engines store `Arc<FitnessFn<U::Gene>>` that can be wrapped identically |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cache LRU eviction + storage | API/Backend (`FitnessCache`) | — | Cache struct is self-contained; no tier boundary issues |
| Cache wrapping of fitness_fn | Engine (PSO/EDA/DE) | — | Each engine owns its fitness evaluation; wrapping happens at engine construction |
| Per-generation cache stats | Engine run loop | GenerationStats | Stats collection happens after fitness evaluations; stats struct carries the data |
| WASM compatibility | Engine | — | Cache uses no threads, no `std::time`; engines already have WASM gates |

## Standard Stack

### Core (no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::sync::Arc` | — | Shared cache handle across threads | Already used by every engine |
| `std::sync::Mutex` | — | Thread-safe cache access | Already used by Ga/CMA cache wiring |
| `crate::fitness::cache::FitnessCache` | — | LRU cache implementation | Ready to use, no modifications needed |
| `crate::fitness::cache::wrap_with_cache` | — | Wraps Arc<FitnessFn> with caching | Ready to use |
| `crate::fitness::cache::cache_snapshot` | — | Snapshots hit/miss counters for delta computation | Ready to use (or inline equivalent) |

**Installation:** No new crates. All dependencies are already in `Cargo.toml`.

## Package Legitimacy Audit

> No external packages installed in this phase. Skip.

## Architecture Patterns

### System Architecture Diagram

```
User Code
    │
    ▼
PsoEngine::new(config, init_fn, fitness_fn)    ← stores Arc<FitnessFn>
    │
    ▼
engine.with_fitness_cache_size(1024)            ← sets config field
    │
    ▼
engine.run()
    │
    ├─ If cache configured:
    │   fitness_fn = wrap_with_cache(fitness_fn, 1024)
    │   → (Arc<FitnessFn>, Arc<Mutex<FitnessCache>>)
    │
    ├─ Per generation:
    │   prev_hits, prev_misses = cache_snapshot()
    │   ...
    │   (self.fitness_fn)(ind.dna())   ← cache intercepts identical DNA
    │   ...
    │   gen_stats.cache_hits = current_hits - prev_hits
    │   gen_stats.cache_misses = current_misses - prev_misses
    │
    └─ Returns PsoResult with generation stats
```

### Reference Implementations

#### Ga Engine (canonical pattern)
```
src/engines/ga/mod.rs:
  - Struct fields: fitness_cache_size (Option<usize>), fitness_cache (Option<Arc<Mutex<FitnessCache>>>)
  - Default: both None
  - Builder: with_fitness_cache_size(size: usize) → sets fitness_cache_size
  - build(): wraps fitness_fn if cache_size configured
  - run(): cache_snapshot() → fitness evaluations → cache_fill_stats()
```

#### CMA Engine (secondary pattern)
```
src/engines/cma/configuration.rs:
  - CmaConfiguration: fitness_cache_size field
  - with_fitness_cache(size) builder method

src/engines/cma/engine.rs:
  - CmaEngine struct: fitness_cache field
  - run(): wraps fitness_fn at start (not at construction)
  - Per-generation: inline snapshot → delta computation
```

### Pattern to Apply Per Engine

Each target engine needs these exact changes:

1. **Struct fields** — add `fitness_cache_size: Option<usize>` and `fitness_cache: Option<Arc<Mutex<FitnessCache>>>`
2. **Default impl** — both fields `None`
3. **Builder method** — `with_fitness_cache_size(size: usize) -> Self`
4. **`run()` start** — wrap fitness_fn if configured (same as CMA pattern)
5. **`run()` per-generation** — snapshot before, compute delta after

### Anti-Patterns to Avoid
- **Wrapping at construction time for non-Ga engines:** PSO/EDA/DE don't have a `build()` method. Wrap at `run()` start (CMA pattern), not in `new()`.
- **Double-wrapping:** Guard with `if self.fitness_cache.is_none()` before wrapping (CMA line 584 pattern).
- **Clearing cache between generations:** D-10 says no. Hot entries stay warm.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LRU cache with O(1) lookup | Custom HashMap + LinkedList | `FitnessCache` (existing) | Already tested, already WASM-safe |
| Hash function for DNA | Custom hasher | `hash_dna()` (existing) | Works for all gene types via Debug repr |
| Thread-safe shared cache | Custom RwLock or sharded cache | `Arc<Mutex<FitnessCache>>` (existing) | Proven pattern in Ga/CMA |
| Per-gen stats delta | Manual hit/miss tracking | `cache_snapshot()` pattern (existing) | Prevents off-by-one errors |

**Key insight:** `FitnessCache` uses `HashMap<u64, f64>` + `VecDeque<u64>` for LRU. The Mutex is held briefly for get/put. For non-parallel engines (PSO, DE inner loop), contention is zero. For EDA's `par_iter` path, contention is brief (cache lookup is O(1)).

## Common Pitfalls

### Pitfall 1: Wrapping fitness_fn in wrong lifecycle phase
**What goes wrong:** If fitness_fn is wrapped in `new()` or `build()`, it cannot be un-wrapped. PSO/EDA/DE don't have `build()`.
**Why it happens:** Ga uses `build()` because it has a complex builder chain. PSO/EDA/DE construct directly.
**How to avoid:** Follow CMA pattern: wrap at `run()` start, guard with `if self.fitness_cache.is_none()`.
**Warning signs:** If you find yourself calling `wrap_with_cache` in `new()`, you're doing it wrong.

### Pitfall 2: Stats not collected per-generation
**What goes wrong:** `GenerationStats` has `cache_hits: None` for all generations, defeating observability.
**Why it happens:** Forgetting to snapshot before and compute delta after each generation.
**How to avoid:** Copy exact pattern from Ga lines 1442 + 1910-1914 or CMA lines 721-727 + 920-924.
**Warning signs:** All stats show `None` for cache fields.

### Pitfall 3: EDA parallel path cache lock contention
**What goes wrong:** `par_iter` calls `fitness_fn` concurrently; each call locks the Mutex briefly.
**Why it happens:** The cache uses `Arc<Mutex<FitnessCache>>` — concurrent access is safe but serialized.
**How to avoid:** This is fine. The cache lookup is O(1) and the lock is held for microseconds. For expensive fitness functions (the use case), serialization overhead is negligible. Do not implement a sharded cache.
**Warning signs:** Benchmark showing cache slower than no-cache (should not happen for expensive fitness).

### Pitfall 4: DE inner-loop stats missing
**What goes wrong:** DE evaluates fitness inside the per-individual loop (`for i in 0..pop_size`), not as a batch. Stats must be computed once after the full loop, not per-individual.
**Why it happens:** DE's structure is different from PSO/EDA — fitness is called per individual within the generation loop, but stats are only meaningful at generation boundary.
**How to avoid:** Snapshot before the `for i in 0..pop_size` loop; compute delta after the loop ends (before `generations += 1`).
**Warning signs:** Stats showing hits/misses that don't match population size.

### Pitfall 5: PSO fitness evaluated in inner loop
**What goes wrong:** PSO evaluates fitness inside `for i in 0..pop.len()` — the per-particle loop. Same as DE.
**Why it happens:** PSO's velocity-update-then-evaluate pattern puts fitness evaluation inside the particle loop.
**How to avoid:** Same as DE: snapshot before the outer generation loop; compute delta after the particle loop + gbest update complete.
**Warning signs:** Duplicate or incorrect stats.

## Code Examples

### Ga Engine — Reference Cache Wiring

```rust
// Source: src/engines/ga/mod.rs lines 871-878 (build-time wrapping)
if let Some(cache_size) = self.fitness_cache_size {
    if let Some(fitness_fn) = self.fitness_fn.take() {
        let (wrapped, cache_handle) =
            crate::fitness::cache::wrap_with_cache(fitness_fn, cache_size);
        self.fitness_fn = Some(wrapped);
        self.fitness_cache = Some(cache_handle);
    }
}
```

```rust
// Source: src/engines/ga/mod.rs lines 1441-1442 (per-gen snapshot)
let (prev_cache_hits, prev_cache_misses) = cache::cache_snapshot(&self.fitness_cache);
```

```rust
// Source: src/engines/ga/mod.rs lines 1909-1915 (per-gen delta fill)
cache::cache_fill_stats(
    &self.fitness_cache,
    &mut gen_stats,
    prev_cache_hits,
    prev_cache_misses,
);
```

### CMA Engine — Run-Time Wrapping Pattern

```rust
// Source: src/engines/cma/engine.rs lines 582-598 (run-time wrapping)
if let Some(size) = self.config.fitness_cache_size {
    if self.fitness_cache.is_none() {
        if self.batch_evaluator.is_none() {
            let (wrapped_fn, cache_handle) =
                crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
            self.fitness_fn = wrapped_fn;
            self.fitness_cache = Some(cache_handle);
        } else {
            self.fitness_cache = Some(Arc::new(Mutex::new(
                crate::fitness::cache::FitnessCache::new(size),
            )));
        }
    }
}
```

```rust
// Source: src/engines/cma/engine.rs lines 920-924 (per-gen delta — inline pattern)
if let Some(ref ch) = self.fitness_cache {
    let c = ch.lock().expect("fitness cache lock poisoned");
    stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
    stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
}
```

### PSO Engine — Fitness Call Site

```rust
// Source: src/engines/pso/engine.rs line 415 (per-particle fitness eval)
// This is the call that benefits from caching.
// No change needed — self.fitness_fn is already the wrapped version.
let new_fit = (self.fitness_fn)(pop[i].dna());
```

### EDA Engine — Fitness Call Sites

```rust
// Source: src/engines/eda/engine.rs lines 350-354 (parallel path, Bernoulli)
let fitness_fn = Arc::clone(&self.fitness_fn);
let fitnesses: Vec<f64> = new_pop
    .par_iter()
    .map(|ind| fitness_fn(ind.dna()))
    .collect();
```

```rust
// Source: src/engines/eda/engine.rs lines 360-364 (sequential path, WASM)
for ind in &mut new_pop {
    let f = (self.fitness_fn)(ind.dna());
    ind.set_fitness(f);
}
```

### DE Engine — Fitness Call Site

```rust
// Source: src/engines/de/engine.rs line 167 (per-individual trial eval)
let trial_fitness = (self.fitness_fn)(&trial_dna);
```

### New Builder Method Pattern

```rust
/// Enables an LRU fitness cache with the given capacity.
///
/// When enabled, fitness evaluations are cached by DNA hash. Chromosomes
/// with identical genes will reuse cached fitness values, avoiding
/// redundant (and potentially expensive) fitness function calls.
///
/// The cache is shared across all chromosomes and threads.
///
/// # Arguments
///
/// * `size` — Maximum number of entries in the cache. A typical value
///   is 2-10x the population size.
pub fn with_fitness_cache_size(mut self, size: usize) -> Self {
    self.fitness_cache_size = Some(size);
    self
}
```

### New Struct Fields Pattern

```rust
/// Optional LRU fitness cache size. When set, fitness evaluations are
/// cached to avoid re-evaluating chromosomes with identical DNA.
fitness_cache_size: Option<usize>,

/// Shared handle to the active LRU fitness cache.
///
/// Set during `run()` when `fitness_cache_size` is configured. `None`
/// when no cache is in use.
fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Only Ga + CMA had fitness caching | PSO, EDA, DE also support caching | Phase 77 | Users of these engines can now benefit from duplicate-chromosome elimination |

**Already implemented:**
- `FitnessCache` LRU struct in `src/fitness/cache.rs` — ready to use
- `wrap_with_cache()` — wraps any `Arc<FitnessFn<G>>` where `G: GeneT + Debug + 'static`
- `hash_dna()` — hashes DNA via Debug representation
- `cache_snapshot()` — returns `(hits, misses)` for delta computation
- `GenerationStats.cache_hits` / `cache_misses` — already defined as `Option<u64>`

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | All three target engines' gene types satisfy `GeneT + Debug + 'static` bounds required by `wrap_with_cache` | Standard Stack | Low — PSO/DE use `RealGene: GeneT + Debug`, EDA uses `BinaryGene: GeneT + Debug`; verified in source |
| A2 | `cache_snapshot()` function in `src/engines/ga/cache.rs` is `pub(crate)` and accessible from sibling engine modules | Common Pitfalls | Low — can inline the snapshot logic (match + lock + hits/misses) if needed, same as CMA does |

## Open Questions

1. **Should `cache_snapshot` be extracted to `src/fitness/cache.rs` as a free function?**
   - What we know: Currently it lives in `src/engines/ga/cache.rs` as `pub(crate)`. CMA inlines the same logic.
   - What's unclear: Whether D-07 (identical cache treatment) implies sharing the function or just sharing the pattern.
   - Recommendation: Inline the snapshot logic in each engine (same as CMA does). This avoids changing the existing Ga module structure and keeps each engine self-contained. The pattern is 5 lines — not worth a shared utility.

## Environment Availability

> Step 2.6: SKIPPED — no external dependencies required. This phase is purely code changes within the existing crate.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Built-in `#[test]` (no external framework) |
| Config file | none |
| Quick run command | `cargo test` |
| Full suite command | `cargo test --all-targets && cargo test --doc` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| (issue #260) | PSO cache hit behavior | unit | `cargo test test_pso_cache` | ❌ Wave 0 |
| (issue #260) | EDA (Bernoulli) cache hit behavior | unit | `cargo test test_eda_cache` | ❌ Wave 0 |
| (issue #260) | EDA (Gaussian) cache hit behavior | unit | `cargo test test_eda_real_cache` | ❌ Wave 0 |
| (issue #260) | DE cache hit behavior | unit | `cargo test test_de_cache` | ❌ Wave 0 |
| (issue #260) | Cache disabled = no overhead | unit | `cargo test test_*_no_cache` | ❌ Wave 0 |
| (issue #260) | Per-gen stats populated when cache enabled | unit | within above tests | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test --all-targets && cargo test --doc`
- **Phase gate:** `cargo fmt --check && cargo clippy && cargo test --all-targets && cargo test --doc && cargo check --target wasm32-unknown-unknown`

### Wave 0 Gaps
- [ ] `tests/engines/pso/test_pso.rs` — add cache behavior tests (2 tests minimum)
- [ ] `tests/engines/eda/test_eda.rs` — add cache behavior tests (2 tests minimum, one per model)
- [ ] `tests/engines/de/test_de.rs` — add cache behavior tests (2 tests minimum)
- [ ] WASM check: `cargo check --target wasm32-unknown-unknown` (verify FitnessCache still compiles)

## Security Domain

> Omitted — `security_enforcement` not applicable. This phase adds opt-in caching with no security surface.

## Sources

### Primary (HIGH confidence)
- `src/fitness/cache.rs` — FitnessCache implementation, wrap_with_cache, hash_dna, cache_snapshot (read in session)
- `src/engines/ga/mod.rs` — Ga reference implementation for cache wiring (lines 327-334, 871-878, 1014-1017, 1441-1442, 1909-1915)
- `src/engines/ga/cache.rs` — cache_snapshot and cache_fill_stats helpers (read in session)
- `src/engines/cma/engine.rs` — CMA secondary reference (lines 582-598, 721-727, 920-924)
- `src/engines/cma/configuration.rs` — CMA cache config pattern (line 83, line 199)
- `src/stats.rs` — GenerationStats cache_hits/cache_misses fields (lines 71, 77)
- `src/engines/pso/engine.rs` — PSO fitness call site (line 415)
- `src/engines/eda/engine.rs` — EDA fitness call sites (lines 350-354, 360-364, 678-682, 689-693)
- `src/engines/de/engine.rs` — DE fitness call site (line 167)

### Secondary (MEDIUM confidence)
- `AGENTS.md` — project conventions, test requirements, WASM rules
- `CLAUDE.md` — WASM compatibility rules, code style, development workflow

### Tertiary (LOW confidence)
- None — all findings verified from source code in session

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — all dependencies already exist; FitnessCache is proven in Ga/CMA
- Architecture: HIGH — pattern is mechanical repetition of established Ga/CMA wiring
- Pitfalls: HIGH — all pitfalls identified from reading existing code patterns

**Research date:** 2026-06-19
**Valid until:** 2026-07-19 (stable — no external dependencies, internal pattern change only)
