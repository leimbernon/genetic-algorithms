# Phase 75: Reduce Clones in Generation Loop — Reusable Offspring Buffers - Context

**Gathered:** 2026-06-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Profile and eliminate redundant `.clone()` calls in the per-generation offspring path, and convert `parent_crossover()` to use a caller-owned output buffer. Current baseline: 8 actual clone call sites in `src/engines/ga/generation.rs` and 11 in `src/engines/ga/mod.rs` (19 total). Target: reduce by ≥50% (≥10 eliminated), with ≥2% wall-time improvement on `cargo bench --bench rastrigin` at population 500.

**Not in scope:**
- Observer snapshot clones (`mod.rs:2037-2040`) — labeled "justified" in ROADMAP, kept as-is
- Clones in engines other than `src/engines/ga/`
- Full object pooling with chromosome slot reuse (too complex; output buffer pattern is sufficient)
- Surrogate filtering clone (`mod.rs:1542`) or best-chromosome tracking clones (`mod.rs:1863,1876`) — defer unless the ≥50% target requires them

</domain>

<decisions>
## Implementation Decisions

### Mutation enum → Copy derive
- **D-01:** Add `#[derive(Copy)]` to all 8 `*Params` structs: `CreepParams`, `GaussianParams`, `PolynomialParams`, `NonUniformParams`, `DifferentialParams`, `CauchyParams`, `LevyFlightParams`, `SelfAdaptiveGaussianParams` (all have only `Option<f64>` fields — Copy-compatible). Then add `Copy` to the `Mutation` enum itself.
- **D-02:** Also add `Copy` to `MutationConfiguration` (consistent with `CrossoverConfiguration` and `LimitConfiguration` which already derive Copy). `MutationConfiguration` fields are `Option<f64>`, `bool`, and `Mutation` — all Copy-compatible once D-01 lands.
- **D-03:** This eliminates `mutation_method.clone()` at `generation.rs:264,279,304` and `configuration.mutation_configuration.method.clone()` at `mod.rs:1301` — ~4 clones at zero runtime cost.

### No-crossover passthrough
- **D-04:** When crossover probability is not met, **skip the pair entirely** — produce no offspring for that parent pair. Do NOT clone parents into offspring. The current `child_1 = parent_1.clone(); child_2 = parent_2.clone()` lines are removed.
- **D-05:** This is a behavioral change (offspring count = `crossed_pairs * 2`, not `all_pairs * 2`), which is acceptable — this is a v3.0.0 milestone (major semver break allowed). Document in PR. No compatibility flag needed.
- **D-06:** The 1-child fallback clone (`child_2 = children.pop().unwrap_or_else(|| parent_1.clone())` at `generation.rs:254`) — when multi-parent crossover returns only 1 child, use `parent_2` as child_2 rather than `parent_1`. This avoids an asymmetry and eliminates the clone.

### Offspring buffer lifecycle
- **D-07:** Change `parent_crossover()` signature from `-> Result<Vec<U>, GaError>` to accept `&mut Vec<U>` as an output buffer parameter. The function clears the buffer at entry and pushes offspring into it. `parent_crossover` is `pub(crate)` — breaking its signature is fine.
- **D-08:** At the generation loop call site in `mod.rs`, allocate `let mut offspring_buf: Vec<U> = Vec::with_capacity(configuration.limit_configuration.population_size * 2)` once before the loop. Pass `&mut offspring_buf` each generation. This eliminates the per-generation `Vec<U>` heap allocation.
- **D-09:** The buffer is `clear()`ed at the start of each `parent_crossover()` call (inside the function), not outside. This keeps the clearing logic co-located with the buffer ownership.

### Elite clone refactor
- **D-10:** Refactor `extract_elite()` to return `Vec<usize>` (indices into `chromosomes`) instead of `Vec<U>`. Update `reinsert_elite()` to accept `Vec<usize>` and clone from the population at reinsertion time. This is a code-clarity win (the clone is deferred and made explicit); the clone count is the same but the extract phase is now allocation-free.

### Claude's Discretion
- Whether `mod.rs:1688` (`offspring[idx].clone()` in the local-search path) is worth eliminating — tackle it if the ≥50% target is not yet met after D-01 through D-10
- Exact threshold for "≥2% improvement" measurement methodology (single run vs. divan multi-sample average)
- Whether to add a `// ponytail: observer snapshot clone, kept intentional` comment at `mod.rs:2037-2040` to document the justified clones

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core generation loop files
- `src/engines/ga/generation.rs` — `parent_crossover()`, `extract_elite()`, `reinsert_elite()`; all clone sites in this phase's primary target file
- `src/engines/ga/mod.rs` — generation loop call site (lines ~1490-1710), observer notification clones, best-chromosome tracking clones

### Type definitions to modify
- `src/operations.rs` — `Mutation` enum (line 336) and all `*Params` structs (lines 254–432); add `Copy` per D-01
- `src/configuration.rs` — `MutationConfiguration` (line 222); add `Copy` per D-02

### Benchmarks (success criterion measurement)
- `benches/rastrigin.rs` — primary benchmark; "≥2% wall-time improvement on population size 500 vs Phase 61 baseline"
- `Cargo.toml` — `[[bench]]` entries and divan dev-dependency

### Existing pattern reference
- `src/engines/ga/lifecycle.rs` — contains other pub(crate) functions with similar signature conventions; follow the same style for the `parent_crossover` signature change
- `src/traits/chromosome.rs` — `ChromosomeT` trait; `dna_mut()` / `set_gene()` preferred over `dna().to_vec()` per CLAUDE.md performance patterns

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Clone sites targeted (generation.rs — 8 actual)
- Line 167: `portfolio[op_idx].clone()` — AOS mutation portfolio; eliminated by D-01 (Mutation: Copy)
- Line 254: `parent_1.clone()` fallback for 1-child crossover → use `parent_2` per D-06
- Lines 256–257: `parent_1.clone()` / `parent_2.clone()` when crossover skipped → removed per D-04
- Line 264: `configuration.mutation_configuration.method.clone()` → eliminated by D-01/D-02
- Lines 279, 304: `mutation_method.clone()` in Insertion/Deletion match arms → eliminated by D-01
- Line 427: `chromosomes[i].clone()` in `extract_elite` → refactored to index return per D-10

### Clone sites targeted (mod.rs — 11 actual)
- Line 1301: `self.configuration.mutation_configuration.method.clone()` → eliminated by D-02
- Line 1542: `offspring[idx].clone()` in surrogate filtering → kept (out of scope unless needed for target)
- Line 1688: `offspring[idx].clone()` in local-search path → at discretion
- Line 1709: `orig_dna.clone()` in variable-length DNA backup → kept (functional, not hot)
- Lines 1863, 1876: `chromosomes[best_idx].clone()` for best tracking → kept (out of scope)
- Lines 2025, 2037–2040: stats/population/configuration/stats observer snapshots → kept (justified)

### Established Patterns
- `Cow<[Gene]>` for zero-copy DNA already in place (Phase 61 baseline) — don't regress
- `pub(crate)` functions in `ga/` can have signatures changed freely — no public API impact
- WASM cfg-gate: any new rayon usage needs `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`

### Integration Points
- `parent_crossover()` is called once per generation in `mod.rs` — single call site to update for D-07/D-08
- `extract_elite()` / `reinsert_elite()` are called in the elitism block in `mod.rs` — update both call sites when refactoring to index-based API

</code_context>

<specifics>
## Specific Ideas

- The `offspring_buf` pre-allocation (D-08) should use `Vec::with_capacity(population_size * 2)` — this is the maximum offspring count (2 children per parent pair, all pairs crossed).
- For D-06 (1-child fallback): `child_2 = children.pop().unwrap_or_else(|| parent_2.clone())` — use `parent_2` not `parent_1` to avoid asymmetry where both children could be identical to `parent_1`.

</specifics>

<deferred>
## Deferred Ideas

- Full object pool with chromosome slot reuse — too invasive for Phase 75; consider for a dedicated performance phase if profiling shows it's worth it
- Parallelizing survivor selection — Phase 76 (Issue #259)
- Observer snapshot clone elimination — would require `GaObserver` trait signature change (pass `&Population` instead of `Population`); track as a future observer improvement

None of the cross_reference_todos step matched — no todos folded or deferred.

</deferred>

---

*Phase: 75-reduce-clones-in-generation-loop-reusable-offspring-buffers*
*Context gathered: 2026-06-19*
