# Phase 79: Add Runnable Examples for GP, DE, Scatter, Cellular, ALPS Engines - Context

**Gathered:** 2026-06-22
**Status:** Already shipped — context written post-hoc (commit 4f28d45)

<domain>
## Phase Boundary

Add five standalone runnable examples in `examples/` — one per major non-GA engine (GP, DE, Scatter Search, Cellular GA, ALPS) — and register each in the smoke-test list and README.

</domain>

<decisions>
## Implementation Decisions

### Delivery status
- **D-01:** All five examples were delivered in commit `4f28d45` on `feat/phase-79` branch and merged.
- **D-02:** All examples registered in `tests/test_examples.rs` (build + run smoke tests).
- **D-03:** All examples documented in README `## Examples` table at line ~576.

### Claude's Discretion
All implementation choices (example structure, benchmark problem, parameter values) were made by Claude during execution — no user discussion needed.

</decisions>

<canonical_refs>
## Canonical References

No external specs — phase was pure documentation/examples work. Requirements fully captured in ROADMAP.md phase 79.

### Delivered files
- `examples/gp_symbolic_regression.rs` — GP symbolic regression with `GpGa<N>` and `MathNode`
- `examples/de_rastrigin.rs` — Differential Evolution (L-SHADE) on Rastrigin
- `examples/scatter_search.rs` — Scatter Search on continuous benchmark
- `examples/cellular_ga.rs` — Cellular GA with Moore neighborhood
- `examples/alps.rs` — ALPS with age-layered evolution
- `tests/test_examples.rs` — smoke tests for all five examples

</canonical_refs>

<code_context>
## Existing Code Insights

### Established Patterns
- All examples follow the existing pattern: minimal setup, short generation count, `println!` result at the end.
- Registered in `tests/test_examples.rs` via `cargo_build_example` + `cargo_run_example` helpers.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — examples implemented following existing example conventions.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 79-add-runnable-examples-for-gp-de-scatter-cellular-alps-engine*
*Context gathered: 2026-06-22*
