# Phase 77: Extend Fitness Cache to More Engines - Context

**Gathered:** 2026-06-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Audit which engines benefit from fitness caching (re-evaluations of unchanged DNA) and extend `FitnessCache` wiring beyond Ga and CmaEngine to PSO, EDA, and DE engines. The cache prevents redundant fitness evaluations when populations contain duplicate or near-duplicate chromosomes.

**Already wired — no action needed:**
- `Ga` — `with_fitness_cache_size()` builder method, wraps fitness_fn at build() time
- `CmaEngine` — `with_fitness_cache()` config method, wraps fitness_fn at engine construction

**In scope:**
- PSO engine: add `with_fitness_cache_size()` builder, wire cache, expose stats
- EDA engine: add `with_fitness_cache_size()` builder, wire cache, expose stats
- DE engine: add `with_fitness_cache_size()` builder, wire cache, expose stats
- Tests verifying cache hit behavior per engine
- WASM compatibility verification

**Out of scope:**
- Cache for other engines (ALPS, Cellular, HillClimb, Permutate, GP, Island, Scatter, Permutate)
- Cache invalidation strategies beyond LRU
- Distributed/remote caching
- Cache size auto-tuning

</domain>

<decisions>
## Implementation Decisions

### Engine wiring pattern
- **D-01:** Follow the Ga pattern: wrap fitness_fn inside `build()` via `wrap_with_cache()`. The builder gets a `with_fitness_cache_size(size: usize)` method. Cache is an internal detail — users opt-in via the builder.
- **D-02:** Cache is optional per-engine (default: no cache, zero overhead). `with_fitness_cache_size()` is the builder method, consistent with Ga's API.
- **D-03:** WASM compatibility is automatic — `FitnessCache` already uses no threads and no `std::time`. No cfg-gating needed in the new engines.

### Cache stats reporting
- **D-04:** Expose hit/miss stats per-generation via `GenerationStats`. Call `cache_snapshot()` each generation to populate `cache_hits` and `cache_misses` fields (already defined as `Option<u64>`).
- **D-05:** Stats remain `Option<u64>` — `None` when cache is disabled, `Some(hits/misses)` when enabled. Avoids confusion about whether 0 means "no cache" or "cache with zero hits".
- **D-06:** No separate `cache_stats()` method on engine Result types. Stats are only in GenerationStats — keeps API surface minimal.

### Engine-specific behavior
- **D-07:** Identical cache treatment for all three engines: same `with_fitness_cache_size()` builder, same `wrap_with_cache()` call, same stats exposure. Users decide if caching helps via the cache_size parameter.
- **D-08:** Cache wraps the primary fitness_fn call only. For PSO, this covers velocity-update evaluations; personal_best tracking uses the same fitness_fn and benefits automatically when DNA matches.

### Cache lifecycle
- **D-09:** LRU eviction only — no clearing between generations. Fitness values don't change between generations, so cached values remain valid. Hot entries stay warm across generations.
- **D-10:** Persistent cache across generations — same `Arc<Mutex<FitnessCache>>` instance for the entire run. Consistent with Ga's implementation.

### Claude's Discretion
- Whether to add a brief comment at each engine's cache wiring explaining the pattern
- Exact order of builder methods in each engine's impl block
- Whether to add a benchmark demonstrating cache benefit on a deterministic problem per engine

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Fitness cache implementation
- `src/fitness/cache.rs` — `FitnessCache` struct, `wrap_with_cache()` function, `hash_dna()` helper; the core caching mechanism to wire into new engines

### Ga engine (reference implementation)
- `src/engines/ga/mod.rs` — lines 327-334: `fitness_cache_size` and `fitness_cache` fields; lines 872-877: build()-time cache wrapping; line 1014: `with_fitness_cache_size()` builder method; line 1442: `cache_snapshot()` call for stats

### CMA engine (secondary reference)
- `src/engines/cma/configuration.rs` — line 83: `fitness_cache_size` field; line 199: `with_fitness_cache()` builder method
- `src/engines/cma/engine.rs` — line 344: `fitness_cache` field; lines 583-594: cache wrapping in engine construction

### Target engines (to modify)
- `src/engines/pso/mod.rs` — PSO engine entry point; add `fitness_cache_size` field, `with_fitness_cache_size()` builder, cache wiring
- `src/engines/pso/engine.rs` — PSO run loop; wrap fitness_fn with cache, call `cache_snapshot()` for stats
- `src/engines/eda/mod.rs` — EDA engine entry point; add cache fields and builder
- `src/engines/eda/engine.rs` — EDA run loop; wrap fitness_fn with cache, call `cache_snapshot()` for stats
- `src/engines/de/mod.rs` — DE engine entry point; add cache fields and builder
- `src/engines/de/engine.rs` — DE run loop; wrap fitness_fn with cache, call `cache_snapshot()` for stats

### Stats and generation tracking
- `src/stats.rs` — lines 71, 77: `cache_hits` and `cache_misses` fields in `GenerationStats`

### WASM compatibility
- `CLAUDE.md` — WASM rules; FitnessCache is already WASM-safe (no threads, no std::time)

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `FitnessCache` in `src/fitness/cache.rs` — ready to use, no modifications needed
- `wrap_with_cache()` in `src/fitness/cache.rs` — wraps any `Arc<FitnessFn<G>>` with LRU caching; returns `(Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)`
- `hash_dna()` in `src/fitness/cache.rs` — hashes DNA via Debug representation; works for all gene types
- `cache_snapshot()` in `src/fitness/cache.rs` — returns `(u64, u64)` of (hits, misses) for GenerationStats
- `GenerationStats` in `src/stats.rs` — already has `cache_hits: Option<u64>` and `cache_misses: Option<u64>` fields

### Established Patterns
- Ga build()-time wrapping: `if let Some(cache_size) = self.fitness_cache_size { if let Some(fitness_fn) = self.fitness_fn.take() { let (wrapped, handle) = wrap_with_cache(fitness_fn, cache_size); ... } }`
- Ga stats collection: `let (prev_hits, prev_misses) = cache_snapshot(&self.fitness_cache);` then `cache_hits: self.fitness_cache.as_ref().map(|c| c.lock().unwrap().hits() - prev_hits)`
- Builder method: `pub fn with_fitness_cache_size(mut self, size: usize) -> Self { self.fitness_cache_size = Some(size); self }`

### Integration Points
- Each engine's `build()` method — where fitness_fn is wrapped with cache
- Each engine's run loop — where `cache_snapshot()` is called for per-gen stats
- `GenerationStats` construction — where `cache_hits`/`cache_misses` are populated

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches following the Ga pattern.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 77-extend-fitness-cache-to-more-engines-issue-260*
*Context gathered: 2026-06-19*
