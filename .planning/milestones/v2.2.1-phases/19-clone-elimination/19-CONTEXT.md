# Phase 19: Clone Elimination - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Remove redundant heap allocations in the hot crossover and mutation paths by deferring or eliminating unnecessary clones. This phase covers:
- `ga.rs`: defer parent cloning until it's actually needed (fallback branch only)
- All crossover operators: build children via `U::new()` + `set_dna()` instead of `parent.clone()` + `set_dna()`
- All 5 numeric mutation operators: use `set_gene()` instead of `dna().to_vec()`
- Swap, Inversion, Scramble: use `dna_mut()` + `slice::swap()` for in-place operations

No public API changes. No new features.

</domain>

<decisions>
## Implementation Decisions

### Child construction (CLONE-02)
- Children must be built from `U::new()` + `set_dna(Cow::Owned(dna))` — NOT `parent.clone()` + `set_dna()`
- Children start with default state: `fitness = 0.0`, `age = 0`
- This is correct because fitness is always re-evaluated before selection; age=0 is correct for new offspring
- Children should NOT inherit any parent metadata beyond DNA

### Operator breadth (CLONE-02 scope)
- Fix ALL crossover operators that do `parent.clone()` + `set_dna()`, not just the 4 named in CLONE-02
- This includes: MultiPoint, Uniform, Cycle, SinglePoint, SBX, BlendAlpha, Arithmetic, Rejuvenate, Order, PMX
- Order and PMX will also receive algorithmic fixes in Phase 20 — Phase 19 fixes only the child construction pattern

### In-place mutation for Swap/Inversion/Scramble (CLONE-04)
- Use `dna_mut()` + `slice::swap(i, j)` for all index-based swaps
- Single call, idiomatic Rust, works for any `GeneT` regardless of `Copy` bounds
- No intermediate cloned variables

### Numeric mutation operators (CLONE-03)
- Fix all 5: `value.rs`, `creep.rs`, `gaussian.rs`, `polynomial.rs`, `non_uniform.rs`
- Replace `individual.dna().to_vec()` + clone dance with `set_gene(idx, new_gene)` directly
- All 5 operators use the same pattern — fix all in one phase

### Parent clone deferral (CLONE-01)
- Move the `parent.clone()` calls in `parent_crossover()` inside the fallback branch only
- In the crossover branch, parents are passed as `&U` references to the operator — no clone needed
- In the fallback branch (crossover probability not met), clone as-is (children are copies of parents)

### Benchmarks
- No benchmarks in Phase 19
- A dedicated benchmark phase will be added at the end of the milestone to measure cumulative v2.2.1 gains

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — CLONE-01, CLONE-02, CLONE-03, CLONE-04 acceptance criteria

### Core trait
- `src/traits/chromosome.rs` — `ChromosomeT` trait: `new()`, `dna()`, `dna_mut()`, `set_dna()`, `set_gene()` — all methods needed for this phase

### Files to modify
- `src/ga.rs` (lines ~1270-1285) — `parent_crossover()` where unconditional clones happen before probability roll
- `src/operations/crossover/multipoint.rs` — `parent.clone()` at lines 26-27
- `src/operations/crossover/uniform_crossover.rs` — `parent.clone()` at lines 33-34
- `src/operations/crossover/single_point.rs` — `parent.clone()` at lines 49-50
- `src/operations/crossover/cycle.rs` — DNA vec construction from parent
- `src/operations/crossover/sbx.rs` — likely same pattern
- `src/operations/crossover/blend_alpha.rs` — likely same pattern
- `src/operations/crossover/arithmetic.rs` — likely same pattern
- `src/operations/crossover/rejuvenate.rs` — likely same pattern
- `src/operations/crossover/order.rs` — Phase 20 will also touch this file (algo fix)
- `src/operations/crossover/pmx.rs` — Phase 20 will also touch this file (algo fix)
- `src/operations/mutation/value.rs` — `dna().to_vec()` pattern
- `src/operations/mutation/creep.rs` — same pattern
- `src/operations/mutation/gaussian.rs` — same pattern
- `src/operations/mutation/polynomial.rs` — same pattern
- `src/operations/mutation/non_uniform.rs` — same pattern
- `src/operations/mutation/swap.rs` — replace two `set_gene()` calls with `dna_mut().swap(i, j)`
- `src/operations/mutation/inversion.rs` — replace per-gene `.clone()` in loop with `dna_mut().swap()`
- `src/operations/mutation/scramble.rs` — replace per-gene `.clone()` in loop with `dna_mut()` operations

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ChromosomeT::new()` — already exists, returns blank chromosome with empty DNA; safe to use for child construction
- `ChromosomeT::set_gene(idx, gene)` — already exists; use for single-gene mutations instead of full DNA clone
- `ChromosomeT::dna_mut()` — already exists; use for in-place slice operations (swap, inversion, scramble)
- `ChromosomeT::set_dna(Cow::Owned(dna))` — already exists; use after `U::new()` for child construction

### Established Patterns
- `single_point.rs` and `uniform_crossover.rs` already build `child_dna_1` via `Vec::with_capacity()` + slice ops, then call `set_dna(Cow::Owned(...))` — the DNA construction is already efficient; only the `parent.clone()` to initialize the child struct needs to change to `U::new()`
- `cycle.rs` already builds DNA directly as `Vec` without cloning parents' DNA; just needs child initialization changed from `parent_1.clone()` to `U::new()`
- Observer hooks (`on_mutation_complete`, `on_crossover_complete`) are called in `ga.rs` at a higher level — operator internals can change freely without affecting observer notifications

### Integration Points
- `parent_crossover()` in `src/ga.rs` is the entry point for all crossover+mutation — CLONE-01 fix goes here
- Each crossover operator receives `&U` references; child construction is entirely internal to the operator
- Tests in `tests/` cover crossover and mutation correctness — must pass unchanged after this phase

</code_context>

<specifics>
## Specific Ideas

- For `swap.rs`, inversion, scramble: `dna_mut().swap(i, j)` is a single-line replacement for the current 4-line pattern (two clones + two set_gene calls)
- For numeric mutations: the `value.rs` pattern `dna().to_vec()` → modify → `set_dna(Cow::Owned(dna))` is a 3-step dance that becomes `set_gene(idx, new_gene)` — one call

</specifics>

<deferred>
## Deferred Ideas

- **Criterion benchmarks** — User wants a dedicated benchmark phase at the end of the milestone (after Phase 24) to measure cumulative v2.2.1 gains. Add Phase 25 via `/gsd:add-phase` after Phase 24 is complete.

</deferred>

---

*Phase: 19-clone-elimination*
*Context gathered: 2026-03-29*
