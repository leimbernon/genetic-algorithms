# Phase 65: v3.0.0 Migration Guide & Release Notes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 65-v3-0-0-migration-guide-release-notes
**Areas discussed:** MIGRATION.md gaps, Compiler error messages, Release gate scope, CHANGELOG date & [Unreleased]

---

## MIGRATION.md gaps

| Option | Description | Selected |
|--------|-------------|----------|
| Add all 3 + features (Recommended) | All 3 missing entries + parallel/logging hints | ✓ |
| Only DeGene→RealGene + features | SelectionOperator and Mutation are rare operator-author concerns | |
| All 3 but features go in CHANGELOG only | Feature changes already in CHANGELOG | |

**User's choice:** Add all 3 + features

---

| Option | Description | Selected |
|--------|-------------|----------|
| Own ## sections (Recommended) | parallel and logging each get their own ## section | ✓ |
| One combined 'Feature flags' section | Two subsections under one ## | |
| Addendum box at the end | Callout box for less prominent display | |

**User's choice:** Own ## sections

---

| Option | Description | Selected |
|--------|-------------|----------|
| Fold LinearChromosome bound into ChromosomeT split section | Prominent callout inside existing section | |
| Give it its own ## section | Harder to miss | |
| You decide | Follow whichever is cleaner | ✓ |

**User's choice:** Claude's discretion — fold into ChromosomeT split section as a callout

---

## Compiler error messages

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — add error[E...] blocks to each entry (Recommended) | Required by ROADMAP success criteria | ✓ |
| Yes, but only for the most confusing ones | ChromosomeT split, DeGene→RealGene, SelectionOperator | |
| No — before/after is enough | Errors change between rustc versions | |

**User's choice:** Yes — add error[E...] blocks to each entry

---

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated '### Compiler error' subsection (Recommended) | Fenced code block, consistent across all entries | ✓ |
| Inline as a blockquote after 'Before' block | Lighter, more compact | |

**User's choice:** Dedicated ### Compiler error subsection

---

## Release gate scope (Plan 65-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Full CI matrix | cargo test, serde, clippy, doc, wasm32 | ✓ |
| crates.io dry-run publish | catch Cargo.toml/file issues | ✓ |
| Smoke-test a v2 sample crate | minimal crate with v2 patterns, apply MIGRATION.md | ✓ |
| examples/ smoke run | cargo run --example X for all examples | ✓ |

**User's choice:** All four

---

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal crate with top 3 breaking patterns (Recommended) | ChromosomeT+LinearChromosome, Reporter, SelectionOperator | ✓ |
| Full working v2 GA | More confidence, heavy to build | |
| Just verify compiler errors match MIGRATION.md | Confirm error[E...] codes are correct | |

**User's choice:** Minimal crate with top 3 patterns

---

## CHANGELOG date & [Unreleased]

| Option | Description | Selected |
|--------|-------------|----------|
| Merge [Unreleased] into [3.0.0] and set today's date (Recommended) | Drop empty [Unreleased], promote [3.0.0] to 2026-06-17 | ✓ |
| Keep [Unreleased] empty, change [3.0.0] - Unreleased to [3.0.0] | Keep placeholder for future | |
| Leave both Unreleased | Date set only when git tag is cut | |

**User's choice:** Merge and set 2026-06-17

---

| Option | Description | Selected |
|--------|-------------|----------|
| All phases 47–69 summarized (Recommended) | Existing content + gaps for 64–69 | ✓ |
| Only phases with public API changes | Build-perf phases are internal | |
| Summarize by theme, not phase number | Reorganize into New Engines / API Changes / etc. | |

**User's choice:** All phases 47–69

---
