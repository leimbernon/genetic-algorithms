---
phase: 53-tree-chromosome-gpga-engine
plan: 02
subsystem: engines
tags: [genetic-programming, gp, crossover, mutation, tree, bloat-control]

# Dependency graph
requires:
  - phase: 53-01
    provides: GpNode trait, Node<N> enum, GpChromosome<N>, TreeChromosome, GaError::TreeDepthExceeded/TreeSizeExceeded, grow_tree/check_limits in node.rs

provides:
  - GpCrossover::SubtreeCrossover with max_depth/max_node_count bloat enforcement
  - GpMutation::SubtreeMutation replacing random subtrees with grow_tree output
  - GpMutation::PointMutation preserving tree shape (same-arity node value replacement)
  - GpMutation::HoistMutation always shrinking trees (descendant replaces ancestor)
  - Debug impls for Node<N> and GpChromosome<N>
  - pub mod crossover and pub mod mutation in gp/mod.rs

affects: [53-03, 53-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Stateful traversal struct (NodeReplacer/Replacer) for in-place subtree replacement without borrow checker conflicts"
    - "Pre-order index addressing for node selection in tree traversal"
    - "p_per_node probabilistic per-node mutation with same-arity filtering"
    - "HoistMutation: pick Function node, pick random descendant, replace ancestor with descendant"

key-files:
  created:
    - src/engines/gp/crossover.rs
    - src/engines/gp/mutation.rs
  modified:
    - src/engines/gp/mod.rs
    - src/engines/gp/node.rs
    - src/engines/gp/chromosome.rs

key-decisions:
  - "GpCrossover and GpMutation are NOT wired into global factory dispatch; they use direct impl methods dispatched by GpGa engine"
  - "PointMutation silently skips nodes when no same-arity alternatives exist (no panic, no error)"
  - "HoistMutation on terminal root returns Ok(()) — no-op"
  - "NodeReplacer stateful struct pattern avoids Rust borrow checker conflicts when replacing boxed nodes in a loop"
  - "Debug added to Node<N> via derive; GpChromosome<N> uses manual Debug impl (Arc<dyn Fn> doesn't impl Debug)"
  - "No par_iter() anywhere in operators — WASM compatible; called per-pair from engine loop"

patterns-established:
  - "Pattern: stateful NodeReplacer/Replacer struct for single in-place tree mutation without consuming replacement in loop"
  - "Pattern: pre-order index selection for crossover and subtree mutation points"

requirements-completed: [CHR-05]

# Metrics
duration: 25min
completed: 2026-05-25
---

# Phase 53 Plan 02: GP Operators (Wave 1) Summary

**SubtreeCrossover, PointMutation, HoistMutation, and SubtreeMutation implemented with bloat enforcement — all 5 Wave 1 tests pass**

## Performance

- **Duration:** 25 min
- **Started:** 2026-05-25T00:00:00Z
- **Completed:** 2026-05-25T00:25:00Z
- **Tasks:** 2
- **Files modified:** 5 (plus 2 created)

## Accomplishments

- `GpCrossover::SubtreeCrossover` swaps random subtrees between two parents using pre-order index addressing, enforcing max_depth and max_node_count via `check_limits`
- `GpMutation` with three variants: `SubtreeMutation` (replace subtree with `grow_tree` output, enforce limits), `PointMutation` (replace node values with same-arity alternatives, tree shape preserved), `HoistMutation` (replace Function subtree with descendant, tree always shrinks)
- All 12 non-ignored tests in `tests/gp.rs` pass; 3 are properly ignored for Wave 2/3

## Task Commits

1. **Task 1: SubtreeCrossover + Task 2: GpMutation (combined)** — `e433aa7` (feat)

## Files Created/Modified

- `src/engines/gp/crossover.rs` — GpCrossover enum with SubtreeCrossover; Replacer stateful struct for in-place subtree swap
- `src/engines/gp/mutation.rs` — GpMutation enum with SubtreeMutation/PointMutation/HoistMutation; NodeReplacer stateful struct
- `src/engines/gp/mod.rs` — added `pub mod crossover`, `pub mod mutation`, exports GpCrossover and GpMutation
- `src/engines/gp/node.rs` — added `#[derive(Debug)]` to Node<N>
- `src/engines/gp/chromosome.rs` — added manual `Debug` impl for GpChromosome<N> (Arc<dyn Fn> has no Debug)

## Decisions Made

- GpCrossover and GpMutation use direct `impl` methods, NOT factory functions — GP operators are dispatched by GpGa engine (Wave 2), not the global Crossover/Mutation enum chain. A factory function would be dead code.
- PointMutation silently skips any node where `N::all_functions()` returns no same-arity alternative — this is correct GP behavior (not an error).
- HoistMutation on a terminal root is a no-op returning `Ok(())` — no error, no panic.
- Stateful `NodeReplacer` struct pattern: Rust's borrow checker prevents consuming `replacement: Box<Node<N>>` inside a `for child in children.iter_mut()` loop, so the replacement is held in an `Option` field and `take()`n when the target index is found.
- `Node<N>` gets `#[derive(Debug)]` (requires `N: Debug` at the call site). `GpChromosome<N>` gets a manual `Debug` impl using `finish_non_exhaustive()` to skip the `Arc<dyn Fn>` field.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added Debug impls for Node<N> and GpChromosome<N>**
- **Found during:** Task 1 (first test run)
- **Issue:** Test asserts use `{:?}` formatting on `Result<(GpChromosome<N>, GpChromosome<N>), GaError>` — `GpChromosome` lacked `Debug`; `Node<N>` also lacked it
- **Fix:** Added `#[derive(Debug)]` to `Node<N>`; added manual `Debug` impl to `GpChromosome<N>` using `finish_non_exhaustive` to skip the non-Debug `Arc<dyn Fn>` field
- **Files modified:** `src/engines/gp/node.rs`, `src/engines/gp/chromosome.rs`
- **Verification:** Tests compile and pass; `cargo clippy -- -D warnings` clean
- **Committed in:** `e433aa7` (Task 1+2 combined commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 — missing Debug impl required by test assertions)
**Impact on plan:** Necessary for test compilation. No scope creep.

## Issues Encountered

None beyond the Debug impl deviation above.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. All operators work on in-memory tree structures only. Threat model items T-53-04 through T-53-SC are all addressed:
- `grow_tree(max_depth <= 1)` returns Terminal immediately (T-53-04)
- `PointMutation` filters `all_functions()` in O(n) and picks one (T-53-05)
- `HoistMutation` only shrinks — no injection vector (T-53-06)
- No new Cargo dependencies added (T-53-SC)

## Known Stubs

None. All Wave 1 functionality is fully wired.

## Self-Check: PASSED

- `src/engines/gp/crossover.rs` — exists
- `src/engines/gp/mutation.rs` — exists
- Commit `e433aa7` — exists in git log
- `cargo test --test gp` — 12 passed, 3 ignored
- `cargo clippy -- -D warnings` — clean

## Next Phase Readiness

- Wave 2 (Plan 53-03) can import `GpCrossover` and `GpMutation` directly for use in the `GpGa` engine loop
- `grow_tree` is pub(crate) in node.rs — available for ramped half-and-half initializer (Wave 2)
- No blockers

---
*Phase: 53-tree-chromosome-gpga-engine*
*Completed: 2026-05-25*
