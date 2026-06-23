# Phase 75: Reduce Clones in Generation Loop — Reusable Offspring Buffers - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-19
**Phase:** 75-reduce-clones-in-generation-loop-reusable-offspring-buffers
**Areas discussed:** Mutation → Copy derive, No-crossover passthrough, Offspring buffer lifecycle, Elite clone scope

---

## Mutation → Copy derive

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — add Copy | Eliminates mutation_method.clone() at lines 264, 279, 304 in generation.rs and line 1301 in mod.rs. Zero runtime cost. Locks out non-Copy fields in future. | ✓ |
| No — keep Clone only | Preserves freedom to add non-Copy fields to Mutation variants later. | |
| You decide | Let the implementer choose based on what they find when verifying Copy compatibility. | |

**User's choice:** Yes — add Copy to Mutation + all *Params structs

**Follow-up — MutationConfiguration:**

| Option | Description | Selected |
|--------|-------------|----------|
| Also add Copy to MutationConfiguration | Consistent with CrossoverConfiguration and LimitConfiguration which already derive Copy. Eliminates configuration.mutation_configuration.method.clone() pattern entirely. | ✓ |
| Leave MutationConfiguration as Clone-only | Safer — config structs can get new heap-allocating fields later. | |

**User's choice:** Also add Copy to MutationConfiguration

**Notes:** All params structs confirmed to have only `Option<f64>` fields — Copy-safe. MutationConfiguration fields are `Option<f64>`, `bool`, `Mutation` — all Copy-compatible once Mutation derives Copy.

---

## No-crossover passthrough

| Option | Description | Selected |
|--------|-------------|----------|
| Skip the pair — no offspring for uncrossed pairs | Eliminates both parent.clone() calls. Offspring pool smaller when crossover rate low but survivor selection handles it. | ✓ |
| Keep cloning parents through | Preserves current behavior: every selected pair always contributes exactly 2 offspring. | |
| Use mutation-only path | When crossover doesn't fire, produce offspring via mutation of a single parent. 1 clone per skipped pair. | |

**User's choice:** Skip the pair — no offspring for uncrossed pairs

**Follow-up — behavioral change acceptability:**

| Option | Description | Selected |
|--------|-------------|----------|
| Acceptable — just change it | v3.0.0 milestone (major semver break allowed). Document in PR. No flag needed. | ✓ |
| Add a config flag for old behavior | Add CrossoverConfiguration::passthrough_on_miss: bool. | |
| Only skip if crossover_probability = 1.0 | Conservative: only change when crossover is always-on. | |

**User's choice:** Acceptable — just change it

**Notes:** v3.0.0 major semver break covers this behavioral change. No compatibility flag needed.

---

## Offspring buffer lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Output buffer (&mut Vec<U>) — clear + refill | Allocate Vec::with_capacity(pop_size * 2) once before generation loop. Pass as &mut Vec<U> to parent_crossover(). Each generation: clear() + push. | ✓ |
| Full object pool — slot reuse | Pre-allocate fixed pool of chromosome slots. Much more complex. | |
| Keep current allocation | Don't change Vec lifecycle. Focus only on eliminating .clone() calls. | |

**User's choice:** Output buffer (&mut Vec<U>) — clear + refill

**Follow-up — signature change handling:**

| Option | Description | Selected |
|--------|-------------|----------|
| Change signature directly | parent_crossover() is pub(crate) — breaking the signature is fine. Update the single call site in mod.rs. | ✓ |
| Keep return type, allocate buffer externally | Keep parent_crossover() returning Vec<U>. Use std::mem::swap or assignment. | |
| You decide | Let the implementer pick based on what's cleanest. | |

**User's choice:** Change signature directly

**Notes:** `parent_crossover` is `pub(crate)` — not public API. Single call site in `mod.rs` to update.

---

## Elite clone scope

| Option | Description | Selected |
|--------|-------------|----------|
| In scope — eliminate if clean | Refactor extract_elite to return Vec<usize>. Counts toward ≥50% clone reduction target. | ✓ |
| Out of scope — skip it | ROADMAP only names parent.clone() and offspring[idx].clone(). Elite set is tiny (1-5 items). | |

**User's choice:** In scope — eliminate if clean

**Follow-up — what "clean" means for extract_elite:**

| Option | Description | Selected |
|--------|-------------|----------|
| Defer clone to reinsert — indices in, clone at reinsertion | extract_elite returns Vec<usize>. reinsert_elite clones from population by index at reinsertion. Code-clarity win; clone count unchanged. | ✓ |
| Eliminate with swap-based elite preservation | Mark elite slots (BitSet), have survivor selection skip them. Zero clones. More invasive. | |
| Skip elite refactor — not worth it | Elite set is tiny. Leave extract_elite as-is. | |

**User's choice:** Defer clone to reinsert — indices in, clone at reinsertion

**Notes:** Extract phase becomes allocation-free. The clone is made explicit and localized to reinsertion.

---

## Claude's Discretion

- Whether `mod.rs:1688` (`offspring[idx].clone()` in local-search path) is worth eliminating — tackle if ≥50% target not yet met
- Exact ≥2% improvement measurement methodology (single run vs. divan multi-sample average)
- Whether to add a comment at `mod.rs:2037-2040` documenting the intentional observer snapshot clones

## Deferred Ideas

- Full object pool with chromosome slot reuse (too invasive, defer to dedicated performance phase)
- Phase 76: Parallelize survivor selection and non-dominated sorting (Issue #259)
- Observer snapshot clone elimination — would require `GaObserver` trait signature change
