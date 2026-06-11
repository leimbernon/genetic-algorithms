---
phase: 25-alternative-metaheuristics
plan: 03
tags: [restructure, engines-group]
completed: "2026-04-26"
---

# Plan 03: Move ga/island/nsga2 into src/engines/ and create placeholder stubs

**Result:** Complete — full suite green (test, serde, clippy, doc).

## What Was Done

- `src/ga.rs` → `src/engines/ga.rs`
- `src/island/` → `src/engines/island/`
- `src/nsga2/` → `src/engines/nsga2/`
- Created placeholder stubs (filesystem-only, not compiled): `src/engines/de/mod.rs`, `src/engines/scatter/mod.rs`, `src/engines/cellular/mod.rs`, `src/engines/alps/mod.rs`
- `src/lib.rs`: added `#[path]` for ga, island, nsga2
- Fixed `tests/test_observer.rs:340` hard-coded `include_str!("../src/ga.rs")` → `../src/engines/ga.rs`
- Fixed 4 pre-existing rustdoc redundant link warnings in ga.rs, island/mod.rs, nsga2/mod.rs
- Suppressed pre-existing `clippy::too_many_arguments` on `parent_crossover` with `#[allow]`

## Verification

- `cargo test`: 657 passed, 20 ignored
- `cargo test --features serde`: 687 passed, 20 ignored
- `cargo clippy`: 0 issues
- `cargo doc --no-deps`: 0 warnings
