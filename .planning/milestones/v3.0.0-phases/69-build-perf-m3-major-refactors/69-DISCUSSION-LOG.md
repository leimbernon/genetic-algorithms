# Phase 69: Build-perf M3 — major refactors - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-15
**Phase:** 69-build-perf-m3-major-refactors
**Areas discussed:** rayon gating scope, ga.rs submodule visibility, divan port fidelity, Clippy enforcement gate

---

## rayon gating scope

| Option | Description | Selected |
|--------|-------------|----------|
| All 27 files (full crate) | Gate every par_iter()/into_par_iter() site across all engines — nsga2, nsga3, moead, spea2, ibea, gp, island, cellular, alps, de, scatter, eda | ✓ |
| Standard-GA path only | Gate only ga.rs, population.rs, common.rs, tournament.rs; alt-engines remain rayon-unconditional | |
| All files, two commits | Wave A: standard-GA path; Wave B: alt-engines. Reduces blast radius while achieving full coverage | |

**User's choice:** All 27 files (full crate)
**Notes:** Matches BUILD-PERF.md §Action #3 "every... site" language.

### Follow-up: grep CI enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| Add regex CI check | grep-based check that fails if any rayon:: reference lacks the cfg gate | ✓ |
| Rely on feature-matrix CI | CC-2 feature-matrix already catches unconditional rayon:: at compile time; redundant | |

**User's choice:** "Lo que permita que el CI vaya más rápido" (what makes CI faster) → Initially leaned toward no extra check, but follow-up Clippy area discussion led to including a grep step. Decided: add grep CI step (fast, <1s) as enforcement; do NOT add clippy custom lint.

---

## ga.rs submodule visibility

| Option | Description | Selected |
|--------|-------------|----------|
| pub(crate) for cross-submodule items | Anything referenced outside its submodule gets pub(crate). Standard Rust idiom. | |
| pub(super) for tightly-coupled pairs | Items only shared between siblings get pub(super); wider sharing gets pub(crate). | |
| You decide (minimize pub surface) | Use pub(super) where possible, escalate to pub(crate) only when required. | ✓ |

**User's choice:** Claude's discretion — minimize pub surface.

### Follow-up: commit strategy

| Option | Description | Selected |
|--------|-------------|----------|
| One atomic commit | Entire ga.rs → ga/ split in a single commit. Simpler revert. | |
| Submodule-by-submodule commits | One commit per submodule extracted (~11 commits). Easier to review and bisect. | ✓ |

**User's choice:** Submodule-by-submodule commits.

---

## divan port fidelity

| Option | Description | Selected |
|--------|-------------|----------|
| Strict 1:1 port | Preserve all existing bench cases. Only change harness API. | |
| Allow light cleanup | Remove dead/duplicate cases, simplify complex setup where divan helps. ±3% tolerance maintained. | ✓ |

**User's choice:** Allow light cleanup.

### Follow-up: metrics_observer bench

| Option | Description | Selected |
|--------|-------------|----------|
| Keep separate CI step | cargo bench --bench metrics_observer --features observer-metrics as its own CI step. | ✓ |
| Fold into standard bench run | | |

**User's choice:** Keep separate CI step — mirrors existing pattern for `de` bench with `--features benchmarks`.

---

## Clippy enforcement gate

| Option | Description | Selected |
|--------|-------------|----------|
| Defer to later | Feature-matrix CI already catches this at compile time. Document in .planning/intel/parallel-feature.md instead. | |
| Include in Phase 69 | Add enforcement check now while rayon gating is fresh. | ✓ |

**User's choice:** Include in Phase 69.

### Follow-up: enforcement form

| Option | Description | Selected |
|--------|-------------|----------|
| grep CI step (fast) | Simple grep -r 'rayon::' src/ that fails if match lacks cfg gate. <1s. | ✓ |
| clippy forbid rule | #![deny(clippy::some_rule)] in lib.rs. More robust but more setup + CI compilation cost. | |

**User's choice:** grep CI step (fast).

---

## Claude's Discretion

- Exact grep regex for the enforcement step (D-13)
- Ordering of the 11 submodule extraction commits within plan 69-04 (extract in dependency order: low-level helpers first, then algorithm steps, orchestrator last)
- Visibility minimization: pub(super) preferred, pub(crate) as escalation path

## Deferred Ideas

None — discussion stayed within phase scope.
