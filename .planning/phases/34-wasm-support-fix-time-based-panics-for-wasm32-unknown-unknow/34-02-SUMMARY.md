---
phase: 34-wasm-support-fix-time-based-panics-for-wasm32-unknown-unknow
plan: "02"
subsystem: engines/ga
tags: [wasm, cfg-target-arch, ga-engine, rayon, instant]
dependency_graph:
  requires: []
  provides: [cfg-gated-instant-ga, cfg-gated-rayon-ga]
  affects: [src/engines/ga.rs]
tech_stack:
  added: []
  patterns: [cfg-target-arch-wasm32-gate, par_iter-sequential-fallback]
key_files:
  created: []
  modified:
    - src/engines/ga.rs
decisions:
  - "Applied cfg-split pattern (cfg-not-wasm32 / cfg-wasm32) for observer-timer sites rather than nested Option to keep the Instant type fully absent on wasm32 at compile time"
  - "Duplicated par_iter and into_par_iter closures verbatim for wasm32 sequential fallback — no shared helper needed since the closures are already at the call site"
metrics:
  duration: "~8 minutes"
  completed: "2026-05-07"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 34 Plan 02: cfg-gate Instant and rayon in ga.rs Summary

cfg-gate all 4 Instant::now() call sites and both rayon par_iter sites in src/engines/ga.rs for wasm32-unknown-unknown compatibility, with sequential iterator fallback and a one-time log::warn! when max_duration_secs is configured on wasm32.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | cfg-gate Instant imports, all 4 Instant::now() sites, and max_duration_secs check | 69b13ea | src/engines/ga.rs |
| 2 | cfg-gate rayon usage in ga.rs (import + 2 par_iter call sites) | 69b13ea | src/engines/ga.rs |

(Tasks 1 and 2 committed together since both modify the same file and were verified jointly.)

## What Was Built

Modified `src/engines/ga.rs` to compile on `wasm32-unknown-unknown` by:

1. **Instant import gate** — `use std::time::Instant` wrapped in `#[cfg(not(target_arch = "wasm32"))]`
2. **start_time gate** — `let start_time = Instant::now()` at run entry wrapped; on wasm32, emits `log::warn!` instead when `max_duration_secs` is `Some`
3. **Observer-timer gates** — `t_sel`, `t_cx`, `t_surv` each use a cfg-split inside the `if self.observer.is_some()` block: native returns `Some(Instant::now())`, wasm32 returns `None`
4. **max_duration_secs check gate** — the entire time-limit `if let Some(max_secs)` block wrapped in `#[cfg(not(target_arch = "wasm32"))]`
5. **rayon import gate** — `use rayon::prelude::*` wrapped in `#[cfg(not(target_arch = "wasm32"))]`
6. **into_par_iter fallback** — extension/regrow path duplicated: native uses `.into_par_iter()`, wasm32 uses `.map()` on range directly
7. **par_iter fallback** — `parent_crossover` function duplicated: native uses `.par_iter()`, wasm32 uses `.iter()`, closure body identical

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo build` — passed
- `cargo test --lib` — passed (2 tests)
- `cargo clippy --all-targets -- -D warnings` — passed (no issues)

## Known Stubs

None.

## Threat Flags

No new security surface introduced. All changes are compile-time cfg gates on existing code paths.

## Self-Check: PASSED

- `src/engines/ga.rs` modified and exists: confirmed
- Commit `69b13ea` exists: confirmed
- No unconditional `use std::time::Instant` remains in file
- No unconditional `use rayon::prelude::*` remains in file
- `max_duration_secs is not supported on wasm32` warn present: 1 match
- `#[cfg(target_arch = "wasm32")]` count: 6 (warning emit + 3 observer-timer None branches + 2 sequential fallback branches)
- `#[cfg(not(target_arch = "wasm32"))]` count: 7 (rayon import + Instant import + start_time + 3 observer-timer native branches + max_duration check)
