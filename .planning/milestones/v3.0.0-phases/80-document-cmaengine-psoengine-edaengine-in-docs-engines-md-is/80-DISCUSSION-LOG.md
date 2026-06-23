# Phase 80: Document CmaEngine, PsoEngine, EdaEngine in docs/engines.md - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-22
**Phase:** 80-document-cmaengine-psoengine-edaengine-in-docs-engines-md-is
**Areas discussed:** Page structure, Snippet style & depth

---

## Page structure

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated pages + stubs | cma.md / pso.md / eda.md follow nsga3.md/moead.md pattern; engines.md gets short stub linking to them | ✓ |
| Inline sections only | All content inline in engines.md like DE/Scatter; no new files; docs/index.md gets anchor links | |

**User's choice:** Dedicated pages + stubs (Recommended)
**Notes:** Success criteria mention "docs/index.md links the new pages" confirming dedicated-page intent. The nsga3.md/moead.md pattern is the right template.

---

## Snippet style & depth

| Option | Description | Selected |
|--------|-------------|----------|
| Key differentiators only | CMA shows RestartStrategy + sigma0, PSO shows LinearDecay + Ring topology, EDA shows Bernoulli vs Gaussian contrast | ✓ |
| Minimal boilerplate | Same short snippet style as engines.md inline sections — default values only | |

**User's choice:** Key differentiators only (Recommended)
**Notes:** CMA/PSO/EDA have meaningful config enums (RestartStrategy, PsoInertia, PsoTopology) that distinguish them from simpler engines. Showing these in snippets is the main value of the documentation.

---

## Claude's Discretion

- Parameter table formatting: follow nsga3.md / moead.md Markdown style
- "When X beats Y" content: derive from engine source and doc comments
- Section ordering within dedicated pages: Description → When to Use → Configuration → Snippets → See Also

## Deferred Ideas

None — discussion stayed within phase scope.
