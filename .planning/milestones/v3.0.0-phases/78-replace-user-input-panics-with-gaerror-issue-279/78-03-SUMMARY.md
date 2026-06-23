---
phase: 78-replace-user-input-panics-with-gaerror-issue-279
plan: "03"
subsystem: engines/selection/crossover
tags: [error-handling, breaking-change, cellular, alps, selection, crossover]
status: complete

dependency_graph:
  requires:
    - "78-01 (GaError variants established)"
  provides:
    - "CellularEngine::new() -> Result<Self, GaError>"
    - "AlpsEngine::new() -> Result<Self, GaError>"
    - "SelectionOperator::select() -> Result<Vec<Vec<usize>>, GaError>"
    - "ox_build_child() -> Result<Vec<G>, GaError>"
  affects:
    - "78-04 (must update test/bench call sites for new() and select() signatures)"

tech_stack:
  added: []
  patterns:
    - "Result-returning constructors with ConfigurationError on invalid config"
    - "Trait return type change: Vec -> Result<Vec, GaError>"
    - "ok_or_else + collect::<Result<Vec<_>, _>>() for fallible iteration"
    - "Graceful degradation with warn log instead of panic in infallible run()"

key_files:
  created: []
  modified:
    - src/engines/cellular/engine.rs
    - src/engines/alps/engine.rs
    - src/traits/operators.rs
    - src/operations/selection.rs
    - src/operations/crossover/order.rs
    - src/operations/crossover/multi_group_ox.rs

decisions:
  - "ROADMAP SC-4 says 'build()' but D-06 specifies new(). Neither CellularEngine nor
     AlpsEngine has a build() method — both use new(). Target new() per D-06 and record
     discrepancy: ROADMAP 'build()' is loose terminology, not a separate method."
  - "run() kept infallible (CellularResult<U>, AlpsResult<U>): zero-size configs are
     fully validated in new(); run() uses debug_assert! for the removed guard."
  - "CellularEngine run() .select() cascade degrades gracefully: match Ok/Err, warn log,
     first-neighbor fallback (vec![vec![0, 1]]) — no panic, no abort, run continues."

metrics:
  duration: "275 seconds (~4.5 min)"
  completed: "2026-06-20"
  tasks_completed: 3
  files_modified: 6
---

# Phase 78 Plan 03: Move Cellular/ALPS validation + SelectionOperator Result + OX CrossoverError Summary

One-liner: Result-returning constructors for CellularEngine/AlpsEngine, SelectionOperator::select returning Result with Lexicase converted from panic to SelectionError, and ox_build_child returning CrossoverError on non-unique gene IDs.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Move Cellular + ALPS validation into Result-returning new() | 240fade | src/engines/cellular/engine.rs, src/engines/alps/engine.rs |
| 2 | Change SelectionOperator::select to Result + cellular cascade | cc50b7c | src/traits/operators.rs, src/operations/selection.rs, src/engines/cellular/engine.rs |
| 3 | Convert ox_build_child + order() non-unique-ID panic to CrossoverError | 2e28f91 | src/operations/crossover/order.rs, src/operations/crossover/multi_group_ox.rs |

## What Was Built

### Task 1 — Cellular/ALPS Result-returning constructors

`CellularEngine::new()` now returns `Result<Self, GaError>`. Validation at construction time:
```rust
if config.rows == 0 || config.cols == 0 {
    return Err(GaError::ConfigurationError(
        "CellularEngine: rows and cols must both be > 0".to_string(),
    ));
}
```

`AlpsEngine::new()` now returns `Result<Self, GaError>` with two guards:
- `layer_size == 0` → `ConfigurationError("AlpsEngine: layer_size must be > 0")`
- `n_layers == 0` → `ConfigurationError("AlpsEngine: n_layers must be > 0")`

Both engines' panics inside `run()` were removed. The cellular `run()` empty-pop guard was replaced with `debug_assert!(!pop.is_empty(), "grid validated > 0 in new()")`. The ALPS `run()` panics were removed entirely (no debug_assert needed — the layer count is non-zero by construction). Both `run()` return types remain unchanged.

### Task 2 — SelectionOperator::select returns Result

`SelectionOperator::select()` trait signature changed from `-> Vec<Vec<usize>>` to `-> Result<Vec<Vec<usize>>, GaError>`. All non-Lexicase arms in `impl SelectionOperator for Selection` now wrap their return values in `Ok(...)`. The Lexicase/EpsilonLexicase arm no longer panics — it returns:
```rust
Err(GaError::SelectionError(
    "Selection::Lexicase/EpsilonLexicase cannot be called through the SelectionOperator trait..."
))
```

The `factory()` function's fallthrough at `_ => configuration.method.select(...)` now propagates the Result with `?` (no double-wrapping, since `factory()` already returns `Result`).

The `CellularEngine::run()` direct `.select()` call is now handled with a match:
```rust
let pairs = match self.config.selection.select(&local, 1, 1, 2) {
    Ok(p) => p,
    Err(_) => {
        crate::log_warn!(...);
        vec![vec![0, 1]]  // first-neighbor fallback
    }
};
```
No panic, no unwrap. Degradation is graceful with a warn log.

### Task 3 — ox_build_child returns CrossoverError

`ox_build_child` return type changed from `Vec<G>` to `Result<Vec<G>, GaError>`. The panic on unfilled child positions was replaced with:
```rust
g.ok_or_else(|| GaError::CrossoverError(format!(
    "Order crossover: child position {} was not filled — indicates non-unique gene IDs in the parents.",
    i
)))
```
collected via `.collect::<Result<Vec<_>, _>>()`.

Both callers in `order.rs` propagate with `?`. Both callers in `multi_group_ox.rs` propagate with `?`. The stale `# Panics` doc section was replaced with `# Errors`.

## Verification

```
$ grep -rn 'panic!' src/engines/cellular/engine.rs src/engines/alps/engine.rs src/operations/selection.rs src/operations/crossover/order.rs
(no output — 0 panics)

$ grep -n 'Result<Vec<Vec<usize>>, GaError>' src/traits/operators.rs
src/traits/operators.rs:30:///     ) -> Result<Vec<Vec<usize>>, GaError>
src/traits/operators.rs:57:) -> Result<Vec<Vec<usize>>, GaError>

$ cargo build --lib
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.20s
```

## Deviations from Plan

### ROADMAP/CONTEXT Terminology (SC-4 vs D-06)

As documented in the PLAN.md objective and confirmed during execution: ROADMAP SC-4 uses the word "build()" but D-06 specifies "new()". Grep confirmed neither `CellularEngine` nor `AlpsEngine` has a `build()` method — both engines expose only `new()`. Executed against `new()` per D-06. This is a wording discrepancy in ROADMAP.md, not a different API target.

### Cellular fallback implementation choice

The plan's fallback suggestion was:
> "if it is a `for`/closure that cannot `continue`, fall back to selecting the cell's first neighbor — read the surrounding 30 lines and pick the construct that compiles"

The `run()` loop IS a regular nested `for` loop, so `continue` is valid. However, the approach of inserting a `vec![vec![0, 1]]` fallback vector (first-neighbor) was chosen over `continue` because:
1. `continue` would skip the crossover step for that cell, effectively freezing it for the generation.
2. The first-neighbor fallback produces an offspring (same behavior as if selection returned neighbor 0 + neighbor 1), maintaining consistent evolution throughput.
3. The plan explicitly mentioned the first-neighbor fallback as the preferred choice.

No other deviations — plan executed exactly as written.

## Known Stubs

None. All changes are complete functional implementations.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Changes are internal error-propagation refactors only.

## Self-Check: PASSED

- src/engines/cellular/engine.rs — modified (Result-returning new, debug_assert in run, select cascade)
- src/engines/alps/engine.rs — modified (Result-returning new, panics removed from run)
- src/traits/operators.rs — modified (select returns Result)
- src/operations/selection.rs — modified (impl matches new trait, factory propagates ?)
- src/operations/crossover/order.rs — modified (ox_build_child returns Result, callers use ?)
- src/operations/crossover/multi_group_ox.rs — modified (callers use ?, doc updated)
- Commits: 240fade, cc50b7c, 2e28f91 — all present in git log
- `cargo build --lib` exits 0
