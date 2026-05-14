# Phase 46: Documentation Refactor - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver comprehensive, production-quality documentation that serves as a Single Source of Truth (SSOT) for both human developers and AI models. Every engine, operator, configuration option, and example must be documented with sufficient precision that an AI can read the docs and correctly implement the appropriate algorithm for any given problem.

**In scope:**
- Rewrite `src/lib.rs` as a comprehensive crate-level documentation entry point (docs.rs)
- Create a `docs/` guide directory with per-engine algorithm guides
- Full rustdoc (`///`) coverage on all public items (structs, traits, enums, functions)
- README.md as complete catalog: all 11 engines, 19 examples, all features
- Per-engine "ficha técnica": algorithm description, when to use (problem type, objectives, variable type), parameter tables (mandatory/optional), complete compilable examples, cross-references
- All 19 examples documented (inline comments + README table)
- Module-level `//!` docs for operations, configuration, traits, constraints, HOF, AOS, niching, extension, initializers, benchmarks

**Out of scope:**
- API changes or refactors — documentation only, no code changes
- Interactive documentation (no mdBook or similar tooling unless explicitly requested)
- Translation to other languages — Spanish/English docs are for planning only
- Performance benchmarking documentation (covered in other phases)
</domain>

<decisions>
## Implementation Decisions

### Documentation Structure
- **D-01:** Hybrid approach: comprehensive `src/lib.rs` crate-level docs (for docs.rs / AI consumption) + `docs/` guide directory (for GitHub / human reading)
- **D-02:** `src/lib.rs` rewritten to be the primary SSOT — full crate overview, engine catalog with cross-references, quickstart, feature flags
- **D-03:** `docs/` directory contains per-engine guides as individual markdown files, plus an index

### Engine Documentation Depth
- **D-04:** Each engine gets a "ficha técnica completa" covering:
  - Algorithm description and mathematical context
  - When to use: problem type, number of objectives, variable type constraints
  - All parameters: mandatory vs optional with defaults
  - Complete compilable example (not abbreviated)
  - Cross-references to similar engines (when to choose which)
  - Configuration tips and common pitfalls

### README Scope
- **D-05:** README as complete catalog listing all 19 examples, all 11 engines, all features with short descriptions and links to detailed docs
- **D-06:** Examples table in README expanded from 10 to 19 entries
- **D-07:** Engines table expanded from 7 to 11 entries (add NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA)

### Examples & Module Coverage
- **D-08:** All 19 examples get inline doc comments explaining problem domain, configuration choices, and demonstrated pattern
- **D-09:** All public items across all modules get full `///` rustdoc: parameters, preconditions, return values, panic conditions
- **D-10:** Module-level `//!` docs expanded for: operations, configuration, traits, constraints, hall_of_fame, aos, niching, extension, initializers, benchmarks, error
- **D-11:** All documentation must include the "AI-ready" level of precision: explicit parameter tables, decision guidance, complete examples

### Claude's Discretion
- File naming and organization within `docs/` directory
- Exact structure of per-engine guide templates
- Which examples need the most attention for inline documentation
- Whether to use `#[doc]` attributes for AI-consumable metadata
- Table of contents and navigation within `docs/` index
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Current Documentation State
- `src/lib.rs` — Crate root; needs full rewrite as comprehensive doc entry point (currently 154 lines, minimal, references 7 engines only)
- `README.md` — User-facing project README; needs expansion to cover all 11 engines and 19 examples (currently lists 10 of 19 examples)

### Engine Modules (Doc Targets)
- `src/engines/ga.rs` — Standard GA (2626 lines) — `//!` doc exists, needs "ficha técnica" expansion
- `src/engines/de/engine.rs` — Differential Evolution (291 lines) — `//!` doc exists
- `src/engines/scatter/engine.rs` — Scatter Search (326 lines) — `//!` doc exists
- `src/engines/cellular/engine.rs` — Cellular GA (385 lines) — `//!` doc exists
- `src/engines/alps/engine.rs` — ALPS (421 lines) — `//!` doc exists
- `src/engines/nsga2/mod.rs` — NSGA-II (571 lines) — `//!` doc exists
- `src/engines/nsga3/mod.rs` — NSGA-III (800 lines) — `//!` doc exists
- `src/engines/moead/mod.rs` — MOEA/D (602 lines) — `//!` doc exists
- `src/engines/spea2/mod.rs` — SPEA2 (592 lines) — `//!` doc exists
- `src/engines/sms_emoa/mod.rs` — SMS-EMOA (402 lines) — `//!` doc exists
- `src/engines/ibea/mod.rs` — IBEA — `//!` doc exists

### Module Docs Needing Expansion
- `src/operations.rs` + `src/operations/*/` — Operator enums and dispatchers
- `src/configuration.rs` — GaConfiguration and sub-configs
- `src/traits.rs` + `src/traits/*` — Core trait definitions
- `src/constraints.rs` — Constraint handling
- `src/hall_of_fame.rs` — Solution archive
- `src/aos.rs` — Adaptive operator selection
- `src/niching/` — Fitness sharing
- `src/extension/` — Extension configuration
- `src/initializers.rs` + `src/initializers/*` — Population initialization
- `src/error.rs` — GaError enum
- `src/benchmarks/` — Benchmark functions (behind `benchmarks` feature)
- `src/observe/` — Observer and reporter traits

### Feature Flags Reference
- `CLAUDE.md` §Feature Flags — Documentation pattern for feature-gated items
- `Cargo.toml` — Feature flag definitions

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 46 — Goal: "Update the documentation to explain in more details the different algorithms. A refactor of the documentation can happen if needed"
- `.planning/PROJECT.md` — Project value proposition and constraints

### Prior Phase Documentation Decisions
- `.planning/phases/12-documentation/` — Phase 12 was the last documentation phase (focused on README Examples table only)
- `.planning/codebase/STRUCTURE.md` — Codebase structure map (outdated: March 2026, doesn't reflect restructured src/)
- `.planning/codebase/CONVENTIONS.md` — Code conventions including documentation requirements
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Engine module docs** — All 11 engines already have basic `//!` module-level docs; these serve as the foundation for "ficha técnica" expansion
- **README examples table** — Current table structure (markdown) can be expanded with 9 more entries following the same pattern
- **lib.rs quickstart** — Existing 15-line example in `//!` doc shows the pattern; needs updating and expansion

### Established Patterns
- Rustdoc convention: `//!` for module docs, `///` for public items
- Crate documentation at `src/lib.rs` via `//!` doc comment
- README.md generated via `gsd-doc-writer` (has `<!-- generated-by: gsd-doc-writer -->` marker)
- README already has consistent section structure: Installation → Quick Start → Features → Engines → Observer → Configuration → Visualization → Examples → Development

### Integration Points
- `src/lib.rs` — Rewrite the `//!` block as comprehensive crate documentation; module re-exports stay unchanged
- `README.md` — Expand Examples table (10→19), Engines table (7→11), Features sections
- `docs/` directory — New; must be created and linked from README and lib.rs
- Per-engine `//!` docs — Expanded in their respective files
- All public items — Add `///` docs where missing

### Creative Options
- The "ficha técnica" per-engine docs can follow a template pattern for consistency (use one template markdown file in docs/)
- The AI-friendly documentation could use explicit parameter tables compatible with LLM reading patterns
- Decision trees or "when to use" matrices could supplement textual descriptions
</code_context>

<specifics>
## Specific Ideas

### AI-First Documentation Design
All documentation should be structured so that an LLM reading the docs (via docs.rs HTML, GitHub markdown, or crates.io) can:
1. Identify the correct algorithm for a given problem description
2. Understand all mandatory and optional configuration parameters
3. See a complete, compilable usage example
4. Know which engines are comparable and when to choose alternatives

This means:
- Explicit "When to use" sections (not implicit)
- Complete parameter tables with required/optional/default annotations
- No abbreviated or `// ...` truncated examples in public docs
- Cross-reference links between similar engines

### Template for Per-Engine Guide (docs/ directory)
Each engine guide should follow a consistent structure:
```
# Engine Name
## Description (algorithm + when to use)
## Quick Reference (parameters table)
## Complete Example
## Configuration Tips
## When to Choose This vs [Similar Engine]
## References
```

### User Vision (verbatim)
"Que tanto desarrolladores como modelos de IA sepan a la perfección y de forma muy precisa cómo y en qué circunstancias implementar cada uno de los métodos de la librería. Que la documentación sea simplemente perfecta, detallada y profunda."
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 46-Documentation Refactor*
*Context gathered: 2026-05-14*
