# Phase 12: Documentation - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Update README.md to add a top-level `## Examples` section that documents all available runnable examples with their domain and `cargo run --example <name>` command. No changes to source code or example files.

</domain>

<decisions>
## Implementation Decisions

### Section placement
- New top-level `## Examples` section (same level as `## Features`, `## Quick Example`, `## Usage`)
- Positioned: after `## Full Example (Range)` and before `## Usage`
- Added to the Table of Contents at the top of README.md
- The existing `### Run Examples` under `## Development` is removed — one authoritative place for all examples

### Content format
- Table format with three columns: `Example` | `Domain` | `Command`
- Domain column uses short labels only (no separate description column)
- Example column uses inline code formatting (`` `rastrigin` ``)
- Command column shows the exact `cargo run --example <name>` invocation

### Scope
- All 10 examples included (6 new + 4 existing)
- 6 new (from Phases 10-11): `rastrigin`, `nsga2_zdt1`, `island_model`, `job_scheduling`, `feature_selection`, `niching`
- 4 existing: `knapsack_binary`, `nqueens_range`, `onemax_binary`, `onemax_extension`
- Domain labels for the 6 new ones (from roadmap success criteria): continuous, multi-objective, parallel, permutation, binary, multimodal
- Domain labels for existing ones: Claude's discretion (e.g., binary, range/permutation, binary extension)

### Claude's Discretion
- Domain label wording for the 4 existing examples
- Section intro sentence (if any) before the table
- Exact table column header capitalization and spacing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### README structure
- `README.md` — Current document; read the full file to understand ToC, section ordering, and where to insert the new section

### Existing examples (for domain label reference)
- `examples/onemax_binary.rs` — Read doc block for domain
- `examples/onemax_extension.rs` — Read doc block for domain
- `examples/knapsack_binary.rs` — Read doc block for domain
- `examples/nqueens_range.rs` — Read doc block for domain
- `examples/rastrigin.rs` — Read doc block for domain (continuous optimization)
- `examples/nsga2_zdt1.rs` — Read doc block for domain (multi-objective)
- `examples/island_model.rs` — Read doc block for domain (parallel / island model)
- `examples/job_scheduling.rs` — Read doc block for domain (permutation / scheduling)
- `examples/feature_selection.rs` — Read doc block for domain (binary / adaptive GA)
- `examples/niching.rs` — Read doc block for domain (multimodal / niching)

### Requirements
- `.planning/REQUIREMENTS.md` — DOC-01 acceptance criteria
- `.planning/ROADMAP.md` — Phase 12 success criteria (three items)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — this phase is README editing only

### Established Patterns
- README uses GitHub-flavored Markdown with `##` top-level sections and `###` subsections
- ToC uses anchor links: `[Examples](#examples)` → `## Examples`
- Code blocks use triple backtick with `bash` language tag

### Integration Points
- `## Examples` inserted between `## Full Example (Range)` (line ~185) and `## Usage` (line ~230)
- ToC entry inserted after `[Full Example (Range)](#full-example-range)` entry
- `### Run Examples` block under `## Development` removed (lines ~268-272)

</code_context>

<specifics>
## Specific Ideas

- Table preview discussed and approved during context capture:
  ```
  | Example | Domain | Command |
  |---------|--------|---------|
  | `rastrigin` | Continuous optimization | `cargo run --example rastrigin` |
  | `nsga2_zdt1` | Multi-objective (NSGA-II) | `cargo run --example nsga2_zdt1` |
  | `island_model` | Parallel / island model | `cargo run --example island_model` |
  | `job_scheduling` | Permutation / scheduling | `cargo run --example job_scheduling` |
  | `feature_selection` | Binary / adaptive GA | `cargo run --example feature_selection` |
  | `niching` | Multimodal / niching | `cargo run --example niching` |
  ```

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 12-documentation*
*Context gathered: 2026-03-22*
