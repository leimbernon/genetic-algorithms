# Phase 20: Crossover Algorithm Optimization - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the remaining O(n²) position scan in PMX Crossover with an O(n) hash-based position map. OX is already O(n) (fixed in commit ca5bb76, Task 4.8) — no changes needed to order.rs. Phase 20 is PMX-only.

No public API changes. No new features. No benchmarks (deferred to Phase 25).

</domain>

<decisions>
## Implementation Decisions

### OX scope (ALGO-01)
- OX is already O(n). Commit `ca5bb76` replaced `Vec::contains` with `HashSet::contains` in Task 4.8.
- ALGO-01 is satisfied by that prior commit — mark it complete in REQUIREMENTS.md with a note pointing to ca5bb76.
- No code changes to `src/operations/crossover/order.rs`.

### PMX position map (ALGO-02)
- Replace `other.iter().position(|g| g.id() == mapped_id)` (line 99, O(n) per chain step) with a pre-built `HashMap<i32, usize>` mapping gene ID → index in `other`.
- Build `pos_in_other: HashMap<i32, usize>` once before the chain loop (O(n) build, O(1) lookup).
- Consistent with `segment_ids: HashMap<i32, usize>` already in the file.
- Do NOT use an index array — gene IDs are `i32` and not guaranteed to be densely packed.

### PMX child construction — replace Vec<Option<G>> with direct Vec<G>
- Pre-fill child from `other.dna()` upfront: `let mut child = other.dna().to_vec()`.
- Overwrite the segment with donor values: `child[start..=end] = donor.dna()[start..=end].clone()`.
- Fix displaced genes using the position map: for each `other.dna()[i]` in the segment that is NOT in `donor_segment_ids`, follow the chain via `pos_in_other` and overwrite `child[chain_dest]` in-place.
- No `Vec<Option<G>>` or `.unwrap()` dance. No `unsafe`. Same O(n) total work.
- Apply to both child constructions (child_1 and child_2).

### Claude's Discretion
- Exact variable naming inside `pmx_build_child`
- Whether to extract the position-map build into a helper or keep it inline
- How to structure the chain-following loop now that it overwrites pre-filled values

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — ALGO-01 (mark satisfied, no code change) and ALGO-02 (PMX fix) acceptance criteria

### Files to modify
- `src/operations/crossover/pmx.rs` — only file needing code changes this phase
  - `pmx_build_child`: add pre-fill from `other`, add `pos_in_other: HashMap<i32, usize>`, replace `other.iter().position()` with map lookup, remove `Vec<Option<G>>`

### Files to verify (no changes)
- `src/operations/crossover/order.rs` — confirm no `.position()` call, document why ALGO-01 is satisfied
- `tests/` — run existing PMX correctness tests with seeded RNG to verify identical outputs post-refactor

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `segment_ids: HashMap<i32, usize>` already in `pmx_build_child` — same pattern to replicate for `pos_in_other`
- `crate::rng::make_rng()` used in `pmx()` for crossover point selection — seeded for test reproducibility

### Established Patterns
- Both OX and PMX use `U::new()` + `set_dna(Cow::Owned(...))` for child construction (Phase 19) — keep this
- `pmx_build_child` currently returns `Vec<G>` (after unwrapping Options at end) — return type stays the same
- `segment_ids` HashMap is already built correctly — reuse this for the `donor_segment_ids` membership check in the displaced-gene loop

### Integration Points
- `pmx()` calls `pmx_build_child(parent_1.dna(), parent_2.dna(), start, end)` twice — the function signature `fn pmx_build_child<G: GeneT>(donor: &[G], other: &[G], start: usize, end: usize) -> Vec<G>` stays unchanged
- `tests/` covers PMX with seeded RNG — output must be bit-identical before and after the refactor

</code_context>

<specifics>
## Specific Ideas

- The pre-fill approach (`child = other.dna().to_vec()`) means that positions outside the segment that are NOT displaced already have their correct values. The chain loop only needs to overwrite positions that currently hold "orphaned" values (genes that appear twice after the segment copy). PMX's algorithm guarantees the chain terminates at exactly those positions.
- ALGO-01 should be marked `[x]` in REQUIREMENTS.md with an inline note: "Satisfied by commit ca5bb76 (Task 4.8) — HashSet membership check replaces Vec::contains, making OX O(n). No Phase 20 code change."

</specifics>

<deferred>
## Deferred Ideas

- `Vec<Option<G>>` cleanup in OX — OX doesn't use this pattern; not applicable.
- Phase 23 memory layout cleanup may revisit child construction patterns across all crossover operators.
- Criterion benchmarks — deferred to Phase 25 (after Phase 24) to measure cumulative v2.2.1 gains.

</deferred>

---

*Phase: 20-crossover-algorithm-optimization*
*Context gathered: 2026-03-30*
