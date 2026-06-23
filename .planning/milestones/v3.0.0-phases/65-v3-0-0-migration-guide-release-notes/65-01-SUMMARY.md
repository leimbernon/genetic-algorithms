---
phase: 65
plan: "01"
subsystem: documentation
tags:
  - documentation
  - migration-guide
  - v3.0.0
  - release

dependency_graph:
  requires: []
  provides:
    - "MIGRATION.md — complete v3.0.0 migration guide (13 sections, 11 compiler error blocks)"
  affects:
    - "MIGRATION.md"

tech_stack:
  added: []
  patterns:
    - "Before/After/Compiler error pattern for each breaking change"
    - "blockquote callout for LinearChromosome bound requirement"

key_files:
  created: []
  modified:
    - path: MIGRATION.md
      role: "v3.0.0 migration guide — complete with all breaking changes and compiler error examples"

decisions:
  - "D-03: LinearChromosome bound callout folded into existing Trait split section as > **Note:** blockquote"
  - "D-04: Every breaking-change section has ### Compiler error subsection with real error[E...] block"
  - "Intro updated from 'seven breaking changes' to 'every breaking change' (future-proof)"
  - "Parallel and logging feature sections have no ### Compiler error (opt-in/opt-out, no v2 compile errors)"

metrics:
  duration: "~15 minutes"
  completed: "2026-06-17"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 1
---

# Phase 65 Plan 01: MIGRATION.md — Complete v3.0.0 Migration Guide Summary

Complete v3.0.0 MIGRATION.md with 13 top-level sections, 11 compiler error blocks, and D-03 LinearChromosome bound callout — covering every breaking change from v2.x to v3.0.0.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add `### Compiler error` to 8 existing sections + D-03 Note callout + intro update | 90da926 | MIGRATION.md |
| 2 | Author 3 missing breaking-change sections + 2 feature-flag sections | 90da926 | MIGRATION.md |

## What Was Built

Both tasks were executed in a single atomic write to `MIGRATION.md` (same file, same logical operation). The complete file now contains:

**Task 1 additions to existing 8 sections:**
1. `## Trait split: ChromosomeT + LinearChromosome` → `error[E0277]` + `> **Note:**` D-03 callout
2. `## LinearChromosome: default() renamed to reset()` → `error[E0599]`
3. `## Reporter removed — use GaObserver` → `error[E0432]` + `error[E0599]` (import + builder)
4. `## ChromosomeLength replaces genes_per_chromosome` → `error[E0599]`
5. `## Flat stopping builders replace StoppingCriteria struct` → `error[E0432]`
6. `## LimitConfiguration field removals` → `error[E0559]`
7. `## GaConfiguration field access → accessor methods` → `error[E0616]`
8. `## Logger setup (v2 auto-init → v3 explicit)` → `error[E0432]` + `error[E0599]`

Introduction updated from "seven breaking changes" to "every breaking change".

**Task 2 new sections:**
9. `## DeGene → RealGene rename` → `error[E0412]` with `help: similar name: RealGene`
10. `## SelectionOperator::select — new num_parents parameter` → `error[E0053]` (incompatible type)
11. `## Mutation enum variant parameter changes` → `error[E0599]` (PermutationInsert) + `error[E0308]` (struct variant)
12. `## parallel feature — rayon is now optional` → Opting-out section (no compiler error — feature is default-on)
13. `## logging feature — log crate is now optional` → Opting-out section (no compiler error — feature is default-on)

## Verification Results

```
Section count (expect 13):     OK — 13 ## sections
Compiler error count (expect 11): OK — 11 ### Compiler error subsections
cargo doc --no-deps:           OK — 0 warnings
README banner link:            OK — MIGRATION.md linked
Note callout (D-03):           OK — > **Note:** in Trait split section (line 102)
"seven breaking changes":      OK — 0 occurrences (removed)
error[E0277]:                  1 occurrence
error[E0599]:                  5 occurrences
error[E0432]:                  3 occurrences
error[E0616]:                  1 occurrence
error[E0412]:                  1 occurrence
error[E0053]:                  1 occurrence
```

## Deviations from Plan

None — plan executed exactly as written. Both tasks were implemented in a single atomic write since they target the same file; a single commit covers both.

## Known Stubs

None. All compiler error blocks are populated with realistic rustc output following the format documented in 65-RESEARCH.md. Exact error message wording may differ slightly from real rustc 1.81.0 output — this is an accepted risk (T-65-01) that Plan 65-03 will reconcile by building the v2 smoke-test crate and capturing actual compiler output.

## Threat Flags

None. MIGRATION.md contains no network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- MIGRATION.md exists and has been verified
- Commit 90da926 exists
- All 13 section counts correct
- All 11 compiler error subsection counts correct
- cargo doc --no-deps clean
