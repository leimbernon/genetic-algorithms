# Phase 72: Audit and Fix Ignored Doctests - Context

**Gathered:** 2026-06-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Every rustdoc `# Examples` block in `src/` compiles and passes under `cargo test --doc` — zero `#[ignore]` or `# ignore` annotations on doctests.

**What this phase delivers:**
- Fix 1 failing doctest (missing `GaussianParams` import in `CreepParams` doc example)
- Remove all 29 `# ignore` annotations from doctests
- Doctests that need external resources (GPU, network, long runtime) converted to compile-only (`no_run`) with a comment explaining why
- All remaining doctests run fully and pass

**Out of scope:**
- New doctests for undocumented public items (separate initiative)
- Integration tests or unit tests
- Any source code logic changes

</domain>

<decisions>
## Implementation Decisions

### Doctest restoration strategy
- **D-01:** Doctests ignored because they require external resources (GPU, network, filesystem) or have long runtime are converted to `no_run` with a `// no_run: [reason]` comment. They compile but do not execute.
- **D-02:** Doctests that were ignored for no valid reason (laziness, convenience) are fully restored — remove the `# ignore` and make them run.
- **D-03:** The 1 failing doctest (`CreepParams` example at line 244 in `src/operations.rs`) is fixed by adding the missing `use genetic_algorithms::operations::GaussianParams;` import.

### Approach
- **D-04:** Process is mechanical — each ignored doctest is individually evaluated and fixed. No bulk transformations.
- **D-05:** Engine module-level examples (CMA, DE, GA, GP, IBEA, Island, NSGA2/3, MOEAD, SPEA2, SMS-EMOA) that demonstrate full algorithm runs are likely `no_run` candidates due to runtime. Evaluate case-by-case.

### Agent's Discretion
- Exact comment text for `no_run` annotations — be concise and actionable.
- Whether any engine examples can realistically run within test timeout (e.g., with very small population/generation counts).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary files to audit
- `src/engines/` — All engine module-level and struct-level doc examples (29 ignored doctests span this directory)
- `src/fitness/` — `batch.rs`, `cache.rs`, `surrogate.rs` doc examples
- `src/observe/observer/` — `composite.rs`, `log.rs` doc examples
- `src/traits/` — `configuration.rs`, `operator_compat.rs` doc examples
- `src/lib.rs` — Module-level doc example
- `src/rng.rs` — Module-level doc example
- `src/initializers/unique_initializer.rs` — Doc example

### Failing doctest
- `src/operations.rs` line 244 — `CreepParams` example missing `GaussianParams` import

### Test command
- `cargo test --doc` — the single validation command for this phase

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cargo test --doc` output lists every ignored test with exact file:line — use as the definitive checklist.

### Established Patterns
- Doctest ignore syntax is `# ignore` (inside the code block) or `#[ignore]` (attribute). In this codebase, ignored doctests use `# ignore`.

### Integration Points
- No source code changes — only doc comment modifications in existing files.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — mechanical audit, standard Rust doctest conventions.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 72-audit-ignored-doctests*
*Context gathered: 2026-06-18*
