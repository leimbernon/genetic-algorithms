# Phase 23: Memory Layout - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Five targeted internal memory/performance optimizations — no public API behavioral changes beyond field type and dead field removal:

1. **MEM-01**: `Range<T>.ranges` migrated from `Vec<(T,T)>` to `Arc<[(T,T)]>` — shared slice per chromosome instead of per-gene heap allocation
2. **MEM-02**: `Range::value()` returns by value for `Copy` types without calling `.clone()`
3. **MEM-03**: Dead `generation_numbers: Vec<usize>` field removed from `Population` struct and its serde impls
4. **MEM-04**: `FitnessFnWrapper::call()` annotated with `#[inline]` — zero logic change
5. **MEM-05**: `MassDeduplication` replaces per-chromosome `Vec<i32>` key with incremental `DefaultHasher` — two-pass collision-safe approach

No new features. No public API signature changes to methods. No benchmarks (deferred to Phase 25).

</domain>

<decisions>
## Implementation Decisions

### MEM-01: ranges field type and visibility
- `pub ranges: Vec<(T, T)>` → `pub ranges: Arc<[(T, T)]>` — field stays public, type changes
- Internal operators access `.ranges[i]`, `.ranges.len()`, `.ranges.is_empty()` via Deref — all work identically on `Arc<[...]>`, no access site changes needed
- `Range::new(id, ranges: Vec<(T,T)>, value)` constructor signature stays unchanged — converts Vec to Arc internally via `.into_boxed_slice().into()`
- `Hash` impl in `range.rs` currently iterates `&self.ranges` — still works on `Arc<[(T,T)]>` via Deref, no change needed
- External code that constructs `Range { ranges: vec![] }` via struct literal breaks — accepted (no such pattern in codebase)

### MEM-02: value() for Copy types
- `Range::value(&self) -> T` currently calls `self.value.clone()` — add a specialized impl for `T: Copy` that returns `self.value` directly
- This is the `impl<T: Copy> Range<T>` + `value(&self) -> T { self.value }` pattern (no `.clone()` call)
- The non-Copy impl (`impl<T: Clone> Range<T>`) may remain for non-Copy T or be replaced by a single `T: Copy` impl depending on planner's read of usage
- Claude decides the exact trait bound split

### MEM-03: generation_numbers removal
- Remove `pub generation_numbers: Vec<usize>` field from `Population<U>` struct
- Remove both struct initialization sites in `new_empty()` and `new()` (lines ~54, ~66)
- Remove `generation_numbers: self.generation_numbers.clone()` from the Clone impl (~line 209)
- Remove `serialize_field("generation_numbers", ...)` from the Serialize impl (~line 302)
- Remove the `generation_numbers` field handling from the Deserializer: the local `Option<Vec<usize>>` variable (~line 341), match arm (~line 357), struct construction use (~lines 375-376), and the field-name string in the fields list (~line 387)
- **Serde break is acceptable** — old checkpoint files will fail to deserialize; field was always `vec![]` and never populated with real data

### MEM-04: FitnessFnWrapper::call() inline
- Add `#[inline]` attribute directly above `pub fn call(&self, dna: &[G]) -> f64` in `src/fitness/fitness_fn_wrapper.rs`
- No logic change — one-line annotation only

### MEM-05: MassDeduplication hash approach
- Replace `HashSet<Vec<i32>>` with incremental `DefaultHasher` to avoid per-chromosome `Vec<i32>` allocation
- **Two-pass collision-safe approach**: hash gene IDs incrementally to get `u64`, use that as the common-case key; on hash collision, fall back to exact comparison so no false duplicate removal occurs
- **Hash content**: User wants gene IDs + values in the hash (richer semantics — chromosomes with same gene structure but different values are NOT considered duplicates). **Constraint**: `GeneT` does not bound `Hash` — planner must determine feasibility:
  - Option A: Hash only gene IDs (via `g.id()`) — preserves exact current dedup semantics, no trait bound change
  - Option B: Add `Hash` to `GeneT` trait bounds — enables hashing values, but is a breaking change (contradicts milestone policy)
  - **Planner should choose Option A (IDs only) unless a non-breaking path to include values exists**
- Dedup key semantics: sequence-ordered (gene at position 0 hashes before gene at position 1 — distinguishes [1,2,3] from [3,2,1])

### Claude's Discretion
- Exact trait bound split for MEM-02 (single `Copy` impl vs separate `Clone`/`Copy` impls)
- Implementation structure for the MEM-05 two-pass collision check (HashMap<u64, Vec<i32>> for collision buckets, or similar)
- Whether to add `Cow<[Gene]>` deref compatibility for the Arc field (not needed — all access sites already use slicing)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — MEM-01 through MEM-05 acceptance criteria and success criteria

### Files to modify
- `src/genotypes/range.rs` — MEM-01 (Arc field, new() conversion), MEM-02 (Copy value accessor)
- `src/population.rs` — MEM-03 (remove generation_numbers from struct, Clone, Serialize, Deserialize)
- `src/fitness/fitness_fn_wrapper.rs` — MEM-04 (#[inline] on call())
- `src/operations/extension/mass_deduplication.rs` — MEM-05 (DefaultHasher two-pass approach)

### Files to verify (no changes, but read to understand access patterns)
- `src/operations/mutation/non_uniform.rs` — accesses `gene.ranges[i]`, `gene.ranges.len()`, `gene.ranges.is_empty()`
- `src/operations/mutation/gaussian.rs` — same access pattern
- `src/operations/mutation/polynomial.rs` — same access pattern
- `src/operations/mutation/value.rs` — same access pattern
- `src/operations/mutation/creep.rs` — same access pattern
- `src/operations/crossover/sbx.rs` — accesses `dna[i].ranges[0]`
- `src/operations/crossover/arithmetic.rs` — accesses `dna[i].ranges[0]`
- `src/operations/crossover/blend_alpha.rs` — accesses `dna[i].ranges[0]`
- `src/initializers/range_initializer.rs` — accesses `allele.ranges[rng.random_range(0..allele.ranges.len())]`
- `tests/operations/test_crossover_sbx.rs` — `gene.ranges[0]` access in tests
- `tests/operations/test_mutation_creep_gaussian.rs` — `gene.ranges[0]` access in tests

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Arc::from(vec.into_boxed_slice())` or `vec.into_iter().collect::<Arc<[_]>>()` — convert Vec to Arc<[T]> in `Range::new()`
- `std::hash::{DefaultHasher, Hash, Hasher}` — available in std, no dependencies needed for MEM-05
- `Population::new_empty()` and `Population::new()` — both initialize `generation_numbers: vec![]`, both need the field removed

### Established Patterns
- `Arc` already used throughout the codebase for fitness functions, config, observers — Arc-wrapping a slice is consistent with existing patterns
- `#[inline]` already used elsewhere (e.g., `partition_point` is inlined in std) — adding it to a thin wrapper function is idiomatic
- `mass_genesis.rs` uses in-place swap+truncate pattern (Phase 22) — `mass_deduplication.rs` can follow a similar single-pass approach

### Integration Points
- All mutation/crossover operators access `.ranges` as a slice (via Deref on Arc<[...]>) — zero changes needed to those files after the Arc migration
- `MassDeduplication` is called from `src/ga.rs` extension block — no changes to the call site for MEM-05
- `population.rs` serde Serialize/Deserialize are hand-written (no derive macros) — both impls need surgical removal of the generation_numbers handling

</code_context>

<specifics>
## Specific Ideas

- For MEM-01, `new()` conversion: `Arc::from(ranges.into_boxed_slice())` is one way; `ranges.into_iter().collect::<Arc<[_]>>()` is another — planner picks idiomatic choice
- For MEM-03, the population.rs Deserialize impl has a `FIELDS` array (~line 387) listing field names — `"generation_numbers"` must be removed from it to avoid silently ignoring the removed field as an unknown
- For MEM-05, the two-pass collision approach: `HashMap<u64, Vec<i32>>` (hash → gene ID vec of first chromosome with that hash) means: common path has no Vec<i32> alloc; only hash-colliding entries get one Vec<i32> — rare and acceptable

</specifics>

<deferred>
## Deferred Ideas

- Adding `Hash` to `GeneT` trait to enable gene-value-inclusive dedup — would be breaking; defer to a future milestone that accepts API changes
- Criterion benchmarks for Phase 23 changes — deferred to Phase 25

</deferred>

---

*Phase: 23-memory-layout*
*Context gathered: 2026-04-03*
