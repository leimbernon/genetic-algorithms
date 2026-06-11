# Phase 22: Survivor & Extension Optimization - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Internal performance optimizations across four specific code points — no public API changes:

1. **`reinsert_elite` (src/ga.rs)** — O(n) elite reinsertion replacing full sort (ALGO-05)
2. **`mass_genesis` (src/operations/extension/mass_genesis.rs)** — O(n) single-pass top-2 scan replacing full sort (ALGO-06)
3. **`src/rng.rs`** — Relax atomic ordering from SeqCst to minimum correct (CONC-01)
4. **Extension regrow loop (src/ga.rs)** — Parallelize with rayon (CONC-02)

Observer notification points in ga.rs must be preserved. No public API changes.

</domain>

<decisions>
## Implementation Decisions

### Mass genesis scan (ALGO-06)
- **Single O(n) pass** maintaining best and second-best simultaneously — not two explicit passes
- One loop, two tracked candidates; simpler and equally correct
- After identifying top-2 indices: `chromosomes.swap(0, best_idx)` then `chromosomes.swap(1, second_idx)` then `chromosomes.truncate(2)` — pure in-place, no extra allocation

### RNG atomic ordering (CONC-01)
- `make_rng()`: `SEED.load(Ordering::Acquire)` and `COUNTER.fetch_add(1, Ordering::Relaxed)`
- `set_seed()`: `SEED.store(..., Ordering::Release)` and `COUNTER.store(0, Ordering::Release)` — update set_seed too (Release pairs correctly with Acquire in make_rng)
- Both stores in set_seed change from SeqCst to Release

### Extension regrow parallelization (CONC-02)
- **Collect-then-extend pattern** using rayon:
  ```rust
  let new_chromosomes: Vec<U> = (0..deficit)
      .into_par_iter()
      .map(|_| { /* create + evaluate chromosome */ })
      .collect();
  self.population.chromosomes.extend(new_chromosomes);
  ```
- `init_fn` and `fitness_fn` are already `Arc`'d — thread-safe as-is
- **Preserve existing error behavior** — no new error handling added; if `calculate_fitness()` panics, rayon propagates it to the calling thread (consistent with current behavior)

### reinsert_elite replacement (ALGO-05)
- `select_nth_unstable_by(k - 1, worst_first_cmp)` partitions the slice so the k worst are at indices `0..k` (unordered)
- Overwrite those k slots with elite individuals in-place
- **Comparator**: reverse the existing fitness comparator (handles Maximization/Minimization already) — use `.reverse()` wrapper, no duplication
- Pattern:
  ```rust
  chromosomes.select_nth_unstable_by(k - 1, |a, b| existing_cmp(a, b).reverse());
  for (i, e) in elite.into_iter().enumerate() {
      chromosomes[i] = e; // overwrite worst slot
  }
  ```

### Claude's Discretion
- Exact variable naming and code structure within the above patterns
- Whether to extract the worst-first comparator as a closure or inline it

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — ALGO-05, ALGO-06, CONC-01, CONC-02 acceptance criteria and success criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reinsert_elite` in `src/ga.rs` (~line 1397): current full-sort + replace-last-k pattern to replace
- `extract_elite` in `src/ga.rs` (~line 1367): already uses `select_nth_unstable_by` — consistent pattern to follow for reinsert_elite
- `mass_genesis` in `src/operations/extension/mass_genesis.rs`: full sort + truncate(2) to replace with single-pass top-2 scan
- `src/rng.rs`: `make_rng()` and `set_seed()` — all atomic orderings are currently SeqCst

### Established Patterns
- Parallel crossover/mutation in `src/ga.rs` uses `par_iter` + `collect` + `extend` — CONC-02 regrow follows same pattern
- `extract_elite` already uses `select_nth_unstable_by` for O(n) best selection — `reinsert_elite` should mirror this for worst selection
- `Arc::clone(ff)` inside the regrow loop for fitness_fn — this pattern must be preserved in the parallel closure

### Integration Points
- Extension regrow block in `src/ga.rs` (~line 970–1010): sequential `for _ in 0..deficit` loop to parallelize
- Observer call `on_extension_triggered` fires before regrow — must remain in place unchanged
- Post-regrow NaN recalculation block (for MassDegeneration) is sequential and should remain sequential — it runs after regrow completes

</code_context>

<specifics>
## Specific Ideas

No specific references — open to idiomatic Rust approaches within the patterns above.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 22-survivor-extension-optimization*
*Context gathered: 2026-03-31*
