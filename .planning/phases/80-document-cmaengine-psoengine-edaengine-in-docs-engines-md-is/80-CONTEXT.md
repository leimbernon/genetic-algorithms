# Phase 80: Document CmaEngine, PsoEngine, EdaEngine in docs/engines.md - Context

**Gathered:** 2026-06-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Add comprehensive documentation for the three currently undocumented engines: CMA-ES (`CmaEngine`), PSO (`PsoEngine`), and EDA (`EdaEngine`/`EdaRealEngine`). All three are fully implemented in `src/engines/` and have runnable examples — but have zero coverage in `docs/engines.md` or `docs/index.md`.

**In scope:**
- Create `docs/cma.md`, `docs/pso.md`, `docs/eda.md` — dedicated pages following the `nsga3.md`/`moead.md` pattern
- Add short stub sections (when-to-use + link) to `docs/engines.md` for all three engines
- Update `docs/index.md` to link the three new pages
- Update the engine decision matrix in `docs/engines.md` overview table to include CMA/PSO/EDA
- Zero rustdoc warnings (`cargo doc --no-deps`)

**Out of scope:**
- Changing any Rust source code
- Docs for other engines not mentioned
- Adding new runnable examples (already exist: `cma_es_rastrigin.rs`, `pso_rastrigin.rs`, `eda_trap.rs`)

</domain>

<decisions>
## Implementation Decisions

### Page structure
- **D-01:** CMA-ES, PSO, and EDA each get a **dedicated docs page** (`docs/cma.md`, `docs/pso.md`, `docs/eda.md`) following the `nsga3.md`/`moead.md` pattern (~130–150 lines each).
- **D-02:** `docs/engines.md` gets a **short stub section** per engine (when-to-use bullet list + key params + link to dedicated page), NOT a full inline section. Same pattern as NSGA-III inline section in engines.md that links to `nsga3.md`.
- **D-03:** `docs/index.md` gets entries linking the three new pages under the "Engines" section.
- **D-04:** The engine overview table at the top of `docs/engines.md` is updated to add rows for `CmaEngine`, `PsoEngine`, and `EdaEngine`/`EdaRealEngine`.

### Snippet style & depth
- **D-05:** Code snippets in dedicated pages cover **key differentiating config only** — not minimal boilerplate, not a full tutorial:
  - CMA: show `sigma0` heuristic (1/3 of search range), `RestartStrategy::Ipop` for multimodal, and `lambda = 0` (auto-compute)
  - PSO: show `PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 }` + `PsoTopology::Ring { neighborhood_size: 2 }` as the typical recommended config
  - EDA: show the Bernoulli model path (`EdaEngine` for binary) vs Gaussian model path (`EdaRealEngine` for continuous) as two contrasting snippets

### Claude's Discretion
- Parameter table formatting: follow existing `nsga3.md` / `moead.md` Markdown table style
- "When PSO beats GA" / "When EDA beats crossover-based GAs" content: derive from config docs and engine logic in `src/engines/`
- Section ordering within each page: Description → When to Use → Configuration (param table) → Key Snippets → See Also (links to example + engines.md)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Reference patterns (existing dedicated pages to match)
- `docs/nsga3.md` — structure template: Description, When to Use, Configuration table, code snippet, See Also
- `docs/moead.md` — second reference for structure and tone
- `docs/spea2.md` — third reference for structure and tone

### Engine implementations (source of truth for params and behavior)
- `src/engines/cma/configuration.rs` — `CmaConfiguration` fields with doc comments (sigma0, lambda, restart_strategy, cc/cs/c1/cmu)
- `src/engines/cma/restart.rs` — `RestartStrategy` enum (Ipop, Bipop) with doc comments
- `src/engines/pso/configuration.rs` — `PsoConfiguration`, `PsoInertia` enum, `PsoTopology` enum with doc comments
- `src/engines/eda/configuration.rs` — `EdaConfiguration` and `EdaRealConfiguration` fields

### Existing examples (link targets in See Also sections)
- `examples/cma_es_rastrigin.rs`
- `examples/pso_rastrigin.rs`
- `examples/eda_trap.rs`

### Docs to update
- `docs/engines.md` — add stub sections + update overview table
- `docs/index.md` — add links to three new pages

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docs/nsga3.md`, `docs/moead.md`, `docs/spea2.md`: direct structural templates for new pages (150 lines, Description + When to Use + Configuration table + snippet + See Also)
- `docs/engines.md` §`Nsga3Ga<U>`: inline stub pattern to replicate for CMA/PSO/EDA (59 lines with link to dedicated page)

### Established Patterns
- Param tables use `| Field | Type | Default | Description |` header row (see `engines.md` DE/Scatter sections)
- "When to Use" is a bullet list with: Problem type, Variable type, Key strength, Key weakness
- Code snippets do NOT import `use genetic_algorithms::*` — they use precise module paths
- Each dedicated page ends with a `## See Also` section linking back to `engines.md` and the runnable example

### Integration Points
- `docs/index.md` "Engines" section: add three entries following the existing format
- `docs/engines.md` overview table: add rows for the three engines (between existing entries — sort by use-case category)

</code_context>

<specifics>
## Specific Ideas

- CMA sigma0 heuristic to document: "set sigma0 ≈ 1/3 of the expected search range per dimension"
- PSO recommended defaults to highlight: Clerc's values `c1 = c2 = 1.49445`, `w = 0.729` with Constant inertia; or `w_start=0.9, w_end=0.4` with LinearDecay
- EDA key distinction: `EdaEngine` uses Bernoulli model (binary genes, UMDA), `EdaRealEngine` uses Gaussian model (continuous genes) — this duality should be clear in the page title and opening paragraph

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 80-document-cmaengine-psoengine-edaengine-in-docs-engines-md-is*
*Context gathered: 2026-06-22*
