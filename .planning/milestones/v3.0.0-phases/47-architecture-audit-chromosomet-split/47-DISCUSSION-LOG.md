# Phase 47: Architecture Audit & ChromosomeT Split - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-19
**Phase:** 47-architecture-audit-chromosomet-split
**Areas discussed:** LinearChromosome defaults, Initialization gap, PR staging, MIGRATION.md breadth

---

## LinearChromosome defaults

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, carry all defaults | LinearChromosome provides default impls for `new_gene()`, `set_gene()`, and reset helper | ✓ |
| Only set_gene default | Move `set_gene()` default only, not `new_gene()` or `default()` | |
| No defaults in LinearChromosome | Pure interface contract, implementors write all methods | |

**User's choice:** Yes, carry all defaults
**Notes:** Minimizes migration cost for existing BinaryChromosome / RangeChromosome implementors.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Rename to `reset() -> &mut Self` | Removes shadowing ambiguity with Default trait, builder-style return | ✓ |
| Keep as `default(mut self) -> Self` | Preserves current contract, no migration for existing impls | |
| Remove it entirely | Callers reset manually | |

**User's choice:** Rename to `reset() -> &mut Self`
**Notes:** Cleaner API. Builder-style `&mut Self` return is consistent with other setter methods.

---

| Option | Description | Selected |
|--------|-------------|----------|
| ChromosomeT exposes only `calculate_fitness()` | No fitness fn installation in core trait | ✓ |
| ChromosomeT gets a boxed fitness fn | `set_fitness_fn_boxed(Box<dyn Fn(...)>)` on ChromosomeT | |
| You decide | Defer to planner | |

**User's choice:** ChromosomeT exposes only `calculate_fitness()`
**Notes:** Trees implement `calculate_fitness()` however they like. `set_fitness_fn<F>()` is a LinearChromosome concern (it references `Self::Gene` in its closure signature).

---

## Initialization gap

| Option | Description | Selected |
|--------|-------------|----------|
| Initializers simply drop those checks | No bridge; Phase 48 UniqueChromosome handles uniqueness at type level | ✓ |
| Move flags to InitializationConfig | New struct, Phase 48 removes them from there | |
| You decide | Defer to planner | |

**User's choice:** Initializers simply drop those checks
**Notes:** No bridge needed. The type-level approach in Phase 48 is the right long-term solution.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone type, re-exported from lib.rs | `src/chromosomes/length.rs` or `src/types/length.rs`; first-class public type | ✓ |
| Defined inline in configuration.rs | No new file; fits alongside other config types | |

**User's choice:** Standalone type, re-exported from lib.rs
**Notes:** Makes `ChromosomeLength` a first-class public type at the same level as `ChromosomeT`.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Remove the struct entirely | 3 fields flatten into GaConfiguration as pub(crate) Options | ✓ |
| Keep as pub(crate) internal struct | Hidden from public API, nested serde output | |
| You decide | Defer to planner | |

**User's choice:** Remove entirely — flat fields on GaConfiguration
**Notes:** User asked a clarifying question about where the state would live. After explanation (fields move directly to GaConfiguration as pub(crate) Option fields, serde serializes them flat), confirmed removal.

---

## PR staging

| Option | Description | Selected |
|--------|-------------|----------|
| 3 staged PRs | PR1=ChromosomeT split, PR2=Config cleanup, PR3=Reporter removal + examples | ✓ |
| One mega-PR | All 7 requirements in a single branch | |
| 7 independent PRs | One PR per requirement | |

**User's choice:** 3 staged PRs
**Notes:** Highest-risk phase in v3.0.0. Staged approach makes each PR independently reviewable and CI-verifiable.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Manual with sed/cargo check loop | `sed` replaces bounds, `cargo check` catches misses | ✓ |
| Full manual edit per file | Open each of ~30 files by hand | |
| You decide | Executor decides | |

**User's choice:** Manual with sed/cargo check loop
**Notes:** Fast and auditable. `cargo check` is the safety net.

---

| Option | Description | Selected |
|--------|-------------|----------|
| New workflow file: examples-smoke.yml | Separate job, runs on milestone branch pushes only | ✓ |
| Added to existing workflow | New step in test.yml or clippy.yml | |

**User's choice:** New workflow file: examples-smoke.yml
**Notes:** Keeps existing test workflow clean. Runs only on milestone branch to avoid unnecessary CI load.

---

## MIGRATION.md breadth

| Option | Description | Selected |
|--------|-------------|----------|
| Full v3.0.0 breaking changes guide | Covers all Phase 47 breaking changes | ✓ |
| Reporter only (ARCH-03 minimum) | Just `with_reporter()` → `with_observer()` | |
| Per-change docs in each module | Rustdoc `# Migration` sections, no standalone file | |

**User's choice:** Full v3.0.0 breaking changes guide
**Notes:** More valuable for library users than a narrow note.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Crate root alongside README.md | Highest visibility, ships with crate | ✓ |
| docs/MIGRATION.md | Standard location for larger projects | |
| CHANGELOG.md section | No separate file | |

**User's choice:** Crate root alongside README.md
**Notes:** Include in `Cargo.toml` `include` list. Link from README.

---

## Claude's Discretion

None — all areas had explicit user selections.

## Deferred Ideas

None — discussion stayed within phase scope.
