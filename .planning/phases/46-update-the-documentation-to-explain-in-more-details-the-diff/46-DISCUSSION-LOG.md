# Phase 46: Documentation Refactor - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 46-Documentation Refactor
**Areas discussed:** Doc structure & format, Engine doc depth, README scope, Examples & module completeness

---

## Doc Structure & Format

| Option | Description | Selected |
|--------|-------------|----------|
| Rich lib.rs + module docs | Rewrite src/lib.rs as comprehensive crate-level guide (docs.rs as SSOT) | |
| Hybrid: lib.rs + docs/ guide | Crate-level overview + docs/ directory with per-engine guides | ✓ |
| Minimal lib.rs, rich README | Keep lib.rs thin, put docs in README and docs/ | |

**User's choice:** Hybrid (lib.rs + docs/ guide)
**Notes:** User explicitly wants both formats. lib.rs for docs.rs (AI consumption) and docs/ guide for GitHub (human reading). Single source of truth approach.

---

## Engine Documentation Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Ficha técnica completa | Full per-engine spec: algorithm description, when to use, parameter table, complete example, cross-references | ✓ |
| Detalle moderado | Description, example, key parameters only | |

**User's choice:** Ficha técnica completa
**Notes:** "AI-ready" level of detail required. Every engine needs explicit decision guidance.

---

## README Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Catálogo completo | All 19 examples, 11 engines, all features listed with links | ✓ |
| Destacados con enlaces | Only main engines and representative examples | |

**User's choice:** Catálogo completo

---

## Examples & Module Completeness

| Option | Description | Selected |
|--------|-------------|----------|
| Todos los 19 ejemplos documentados | Each example with detailed comments explaining problem and configuration | |
| Priorizar módulos públicos | Focus on rustdoc on all public items | |
| Ambos en paralelo | Both examples and modules simultaneously | ✓ |

**User's choice:** Ambos en paralelo

---

## Claude's Discretion

- File naming and organization within `docs/` directory
- Exact structure of per-engine guide templates
- Which examples need the most attention for inline documentation

## Deferred Ideas

None — discussion stayed within phase scope.

---

*Discussion completed: 2026-05-14*
