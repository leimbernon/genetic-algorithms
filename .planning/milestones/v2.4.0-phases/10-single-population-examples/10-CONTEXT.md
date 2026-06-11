# Phase 10: Single-population Examples - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Three self-contained runnable examples in `examples/` using `Ga<U>`:
- `rastrigin.rs` — continuous optimization (EX-01)
- `feature_selection.rs` — binary feature selection with adaptive GA (EX-05)
- `niching.rs` — multimodal optimization with fitness sharing (EX-06)

Creating, editing, or moving other examples is out of scope.

</domain>

<decisions>
## Implementation Decisions

### Code structure and style
- Follow `examples/onemax_binary.rs` as the reference template for all three examples
- Each example must have a `/*!` doc block at the top with: problem description, features demonstrated, and `cargo run --example <name>` command
- Constants section at the top of `main()` for all configurable parameters (pop size, generations, etc.)
- Progress via `run_with_callback` reporting every N generations (same pattern as onemax)
- Final result shown via `match result { Ok(...) => ..., Err(...) => ... }`

### Feature Selection setup
- Use a small embedded realistic dataset — Iris-style with hardcoded data (no external file or dependency)
- 20 features total, with a known subset of relevant ones baked into the fitness function
- Fitness function: count relevant features selected minus penalty for irrelevant ones selected
- Show the best binary feature mask in the output (which features are on/off)
- Adaptive GA enabled via `with_adaptive_enabled(true)` — document in the doc block that this auto-adjusts crossover/mutation probabilities

### Claude's Discretion
- Rastrigin: number of dimensions, specific operators (gaussian vs creep), convergence criterion
- Niching: which multimodal function to use, how to print multiple distinct peaks in output
- Exact dataset values for Feature Selection (Iris-style structure, values Claude's choice)
- Report interval N for each example

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing examples (style reference)
- `examples/onemax_binary.rs` — Reference template for structure, doc block, callback, result handling
- `examples/nqueens_range.rs` — Reference for Range<T> chromosome usage pattern

### Project requirements
- `.planning/REQUIREMENTS.md` — EX-01, EX-05, EX-06 acceptance criteria
- `.planning/ROADMAP.md` — Phase 10 success criteria (success criteria section)

### Library source
- `src/ga.rs` — Ga builder API and run_with_callback signature
- `src/configuration.rs` — with_adaptive_enabled and other builder methods
- `src/niching/` — Fitness sharing configuration and usage
- `src/chromosomes/` — Binary and Range chromosome types
- `src/initializers.rs` or `src/initializers/` — binary_random_initialization, range_random_initialization

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `binary_random_initialization` — direct use for Feature Selection and Niching (if using binary)
- `range_random_initialization` — direct use for Rastrigin
- `count_true` fitness helper — may not apply to new examples; custom fitness fns needed
- `run_with_callback` — established pattern for progress reporting

### Established Patterns
- Fitness fn signature: `|dna: &[GeneType]| -> f64`
- Builder chain ending with `.build().expect("...")`
- Callback signature: `|gen: &usize, pop: &Population<C>, stats: &GenerationStats, cause: &TerminationCause| -> ControlFlow<()>`
- `ProblemSolving::Minimization` for Rastrigin, `ProblemSolving::Maximization` for Feature Selection

### Integration Points
- New files go in `examples/` directory — no changes to `src/`
- No new dependencies needed — all operators already exist
- Niching config is set on `Ga` builder (check `with_niching*` methods)

</code_context>

<specifics>
## Specific Ideas

- Feature Selection dataset: Iris-style structure (4-5 real features, padded to 20 with noise) — values hardcoded, no file I/O
- The adaptive GA in Feature Selection should be the highlight of that example's doc block

</specifics>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 10-single-population-examples*
*Context gathered: 2026-03-22*
