---
phase: 17-compositeobserver-metricsobserver
plan: "01"
subsystem: observer
tags: [observer, composite, fan-out, AllObserver, CompositeObserver]
dependency_graph:
  requires: []
  provides: [AllObserver<U>, CompositeObserver<U>]
  affects: [src/observer/mod.rs, src/observer/composite.rs, src/lib.rs]
tech_stack:
  added: []
  patterns: [supertrait marker, blanket impl, Arc fan-out, builder pattern]
key_files:
  created:
    - src/observer/composite.rs
  modified:
    - src/observer/mod.rs
    - src/lib.rs
decisions:
  - AllObserver<U> is a pure supertrait marker (no methods) — object-safe, enabling dyn AllObserver<U>
  - CompositeObserver uses Vec<Arc<dyn AllObserver<U>>> so inner observers can be shared across engines
  - Clone impl clones Arcs cheaply — enables attaching same composite to multiple GA engines
  - GaObserver also added to lib.rs re-exports (was missing from top-level despite being fundamental)
metrics:
  duration_seconds: 180
  tasks_completed: 2
  files_modified: 3
  completed_date: "2026-03-27"
---

# Phase 17 Plan 01: AllObserver Supertrait + CompositeObserver Summary

**One-liner:** AllObserver<U> supertrait + CompositeObserver<U> fan-out dispatching all 19 hooks across GaObserver, IslandGaObserver, and Nsga2Observer via Arc-stored inner observers.

## What Was Built

`AllObserver<U>` — a pure supertrait marker combining `GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync`. A blanket impl automatically satisfies the supertrait for any type implementing all three. The trait is object-safe: `dyn AllObserver<U>` compiles.

`CompositeObserver<U>` — a fan-out observer struct that stores `Vec<Arc<dyn AllObserver<U>>>` and dispatches all 19 hooks to every inner observer in insertion order. Supports a fluent `.add()` builder, `Clone` (cheap Arc clones), and `Default`.

Both are re-exported from `src/lib.rs` as top-level public API. `GaObserver` is also now re-exported (was absent).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | AllObserver supertrait + CompositeObserver implementation | e6800a2 | src/observer/mod.rs, src/observer/composite.rs |
| 2 | Re-export AllObserver and CompositeObserver from src/lib.rs | 7911ada | src/lib.rs |

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo build`: clean (zero errors, zero unused warnings)
- `cargo test`: 22 passed, 0 failed
- `cargo clippy`: clean
- `cargo doc --no-deps`: 4 pre-existing warnings in ga.rs/island/nsga2 (redundant explicit link targets), zero warnings for new items

## Self-Check: PASSED

Files created:
- FOUND: src/observer/composite.rs
- FOUND: .planning/phases/17-compositeobserver-metricsobserver/17-01-SUMMARY.md

Commits:
- FOUND: e6800a2
- FOUND: 7911ada
