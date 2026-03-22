# Phase 12: Documentation - Research

**Researched:** 2026-03-22
**Domain:** README Markdown editing — GitHub-flavored Markdown, Rust project conventions
**Confidence:** HIGH

## Summary

Phase 12 is a pure documentation task: add an `## Examples` section to README.md that covers all 10 runnable examples, update the Table of Contents to include the new section, and remove the now-redundant `### Run Examples` block under `## Development`. No source code changes are involved.

All decisions about format, placement, and content have been locked in CONTEXT.md. The research task is therefore primarily fact-gathering: confirm the exact current state of README.md (line numbers, ToC entries, existing `### Run Examples` content) and establish the correct domain labels for all 10 examples by reading their doc blocks.

**Primary recommendation:** The planner needs one task — edit README.md in three coordinated spots (ToC insertion, section insertion, subsection removal). Content for the table is fully determined and verified below.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- New top-level `## Examples` section (same level as `## Features`, `## Quick Example`, `## Usage`)
- Positioned: after `## Full Example (Range)` and before `## Usage`
- Added to the Table of Contents at the top of README.md
- The existing `### Run Examples` under `## Development` is removed — one authoritative place for all examples
- Table format with three columns: `Example` | `Domain` | `Command`
- Domain column uses short labels only (no separate description column)
- Example column uses inline code formatting (`` `rastrigin` ``)
- Command column shows the exact `cargo run --example <name>` invocation
- All 10 examples included (6 new + 4 existing)
- Domain labels for the 6 new ones: continuous, multi-objective, parallel, permutation, binary, multimodal (from roadmap success criteria)

### Claude's Discretion
- Domain label wording for the 4 existing examples
- Section intro sentence (if any) before the table
- Exact table column header capitalization and spacing

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DOC-01 | README documents all available examples with a brief purpose description and the corresponding `cargo run --example <name>` command | All 10 example names, domain labels, and commands confirmed below; exact insertion points in README.md identified |
</phase_requirements>

## Standard Stack

This phase involves only Markdown editing. No library dependencies, no new packages, no compilation.

| Artifact | Current State | Change Required |
|----------|--------------|----------------|
| `README.md` | 289 lines, GFM with `##` sections and ToC | Three edits: ToC entry, section insertion, subsection removal |
| `examples/*.rs` | 10 files, doc blocks read | Source for domain labels (read-only) |

## Architecture Patterns

### README.md Current Structure (verified)

```
Line 18–37  ## Table of Contents
Line 33       [Full Example (Range)](#full-example-range)   ← insert ToC entry AFTER this
Line 34       [Usage](#usage)
Line 35       [Development](#development)

Line 185    ## Full Example (Range)          ← new ## Examples goes AFTER this section ends
Line 229    ## Usage                         ← new ## Examples goes BEFORE this

Line 241    ## Development
Line 268    ### Run Examples                 ← REMOVE this subsection (lines 268-273)
```

### Pattern: Three-Edit Atomic Change

All three edits must be applied together to keep the README consistent:

1. **ToC insertion** — add `  - [Examples](#examples)` after the `[Full Example (Range)]` line
2. **Section insertion** — insert `## Examples` block between line 229 (`## Usage`) and the end of `## Full Example (Range)`
3. **Subsection removal** — delete the `### Run Examples` block under `## Development`

### GitHub-Flavored Markdown conventions (verified in README.md)

- Top-level sections use `##`
- Subsections use `###`
- ToC uses anchor links: `[Section Name](#section-name)` — lowercase, spaces become hyphens
- Code in table cells uses backtick inline code
- Code blocks use triple-backtick with `bash` language tag
- Table rows use `|` delimiters with header separator row `|---|---|---|`

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Anchor link format | Custom anchor logic | Follow existing ToC pattern exactly | GitHub normalizes anchors: lowercase, spaces → hyphens, special chars stripped |

**Key insight:** The anchor for `## Examples` is `#examples` — no special characters, straightforward.

## Common Pitfalls

### Pitfall 1: Anchor Link Mismatch
**What goes wrong:** ToC entry links to wrong anchor (e.g., `#example` instead of `#examples`).
**Why it happens:** Typo or forgetting GitHub's normalization rules.
**How to avoid:** Anchor = section heading lowercased, spaces replaced with hyphens. `## Examples` → `#examples`.
**Warning signs:** ToC link renders but clicking takes user to wrong place or nowhere.

### Pitfall 2: Incomplete Removal of `### Run Examples`
**What goes wrong:** Leaving a blank line or partial content from the removed subsection, creating an orphaned gap.
**Why it happens:** Off-by-one in line range when editing.
**How to avoid:** Remove the heading line AND all code block lines up to and including the closing ` ``` `. Verify the `### Code Quality` heading immediately follows `### Run Examples` removal.

### Pitfall 3: Table Alignment
**What goes wrong:** Misaligned `|` delimiters cause broken table rendering on GitHub.
**How to avoid:** Markdown tables do not require alignment — any consistent `|` placement works. Keep it simple.

## Code Examples

### Approved Table Content (from CONTEXT.md `<specifics>`)

```markdown
## Examples

Run any example directly with `cargo run`:

| Example | Domain | Command |
|---------|--------|---------|
| `rastrigin` | Continuous optimization | `cargo run --example rastrigin` |
| `nsga2_zdt1` | Multi-objective (NSGA-II) | `cargo run --example nsga2_zdt1` |
| `island_model` | Parallel / island model | `cargo run --example island_model` |
| `job_scheduling` | Permutation / scheduling | `cargo run --example job_scheduling` |
| `feature_selection` | Binary / adaptive GA | `cargo run --example feature_selection` |
| `niching` | Multimodal / niching | `cargo run --example niching` |
| `knapsack_binary` | Binary / combinatorial | `cargo run --example knapsack_binary` |
| `nqueens_range` | Constraint satisfaction | `cargo run --example nqueens_range` |
| `onemax_binary` | Binary / baseline | `cargo run --example onemax_binary` |
| `onemax_extension` | Binary / diversity control | `cargo run --example onemax_extension` |
```

**Notes on domain labels for the 4 existing examples (Claude's discretion):**

| Example | Doc block evidence | Recommended label |
|---------|-------------------|-------------------|
| `knapsack_binary` | "Knapsack problem using Binary chromosomes" (README `### Run Examples`) | Binary / combinatorial |
| `nqueens_range` | "N-Queens problem using Range<i32> chromosomes" (README `### Run Examples`) | Constraint satisfaction |
| `onemax_binary` | "Hello World for Genetic Algorithms — maximize true bits in binary chromosome" | Binary / baseline |
| `onemax_extension` | "Population Diversity Control — OneMax with MassDeduplication extension" | Binary / diversity control |

### ToC Entry to Insert

```markdown
  - [Examples](#examples)
```

Insert after:
```markdown
  - [Full Example (Range)](#full-example-range)
```

### `### Run Examples` Block to Remove

Current content (lines 268-273):
```markdown
### Run Examples
```bash
cargo run --example knapsack_binary    # 0/1 Knapsack problem using Binary chromosomes
cargo run --example nqueens_range      # N-Queens problem using Range<i32> chromosomes
cargo run --example onemax_extension   # OneMax with MassDeduplication extension strategy
```
```

Remove this entire subsection including heading and code block. The section immediately after is `### Code Quality`.

## State of the Art

This is a documentation-only phase. No library API changes. No "old vs. new" considerations.

| Observation | Detail |
|-------------|--------|
| README already uses GFM tables | Pattern confirmed in `## Features` section |
| Existing `### Run Examples` only covers 3 of 10 examples | Confirms the need for this phase |
| All 10 `cargo run --example <name>` commands verified | Confirmed by checking each `.rs` file exists in `examples/` |

## Open Questions

None. All decisions are locked. All content is verified from source files.

## Validation Architecture

`workflow.nyquist_validation` is not present in `.planning/config.json` — treated as enabled.

### Test Framework

This phase has no code changes. Validation is manual document review only.

| Property | Value |
|----------|-------|
| Framework | Manual review — no automated test framework applies |
| Config file | N/A |
| Quick run command | `cargo build` (confirms examples still compile) |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOC-01 | README contains Examples section with all 10 examples, domain labels, and run commands | manual | Review README.md after edit | N/A — doc review |

### Sampling Rate

- **Per task commit:** `cargo build` — confirms no accidental source edits
- **Per wave merge:** Review README.md in browser or `cat README.md`
- **Phase gate:** Confirm three criteria from SUCCESS CRITERIA are visually satisfied in README.md

### Wave 0 Gaps

None — no test infrastructure needed for a README-only change.

## Sources

### Primary (HIGH confidence)
- `/Users/luis/RustroverProjects/genetic-algorithms/README.md` — Full file read; confirmed ToC structure, section ordering, line positions of insertion points, and `### Run Examples` content
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/rastrigin.rs` — Doc block: "Rastrigin Continuous Optimization Example"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/nsga2_zdt1.rs` — Doc block: "NSGA-II Multi-Objective Optimization (ZDT1 Benchmark)"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/island_model.rs` — Doc block: "Island Model Parallel Evolution (Rastrigin 20D)"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/job_scheduling.rs` — Doc block: "Job Scheduling -- Permutation-Based Makespan Minimization"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/feature_selection.rs` — Doc block: "Feature Selection with Adaptive GA"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/niching.rs` — Doc block: "Niching / Fitness Sharing Example — Multimodal Optimization"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/onemax_binary.rs` — Doc block: "OneMax Binary Example — Hello World for Genetic Algorithms"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/onemax_extension.rs` — Doc block: "OneMax with Extension Strategies — Population Diversity Control"
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/knapsack_binary.rs` — No doc block; referenced as "0/1 Knapsack problem using Binary chromosomes" in existing README
- `/Users/luis/RustroverProjects/genetic-algorithms/examples/nqueens_range.rs` — No doc block; referenced as "N-Queens problem using Range<i32> chromosomes" in existing README
- `.planning/phases/12-documentation/12-CONTEXT.md` — All locked decisions, table preview

### Secondary (MEDIUM confidence)
- GitHub-Flavored Markdown anchor rules — standard behavior verified against existing README ToC anchors

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Content (table rows): HIGH — read directly from source files and CONTEXT.md
- Insertion points: HIGH — confirmed by reading README.md line by line
- Domain labels for existing examples: MEDIUM — inferred from doc comments and existing README descriptions; Claude's discretion per CONTEXT.md
- Anchor link format: HIGH — verified against existing ToC patterns in README.md

**Research date:** 2026-03-22
**Valid until:** Stable — README editing conventions do not change
