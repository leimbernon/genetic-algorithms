---
phase: 25-alternative-metaheuristics
plan: 01
tags: [restructure, types-group]
completed: "2026-04-26"
---

# Plan 01: Move chromosomes and genotypes into src/types/

**Result:** Complete — zero test failures, zero warnings.

## What Was Done

- `src/chromosomes.rs` → `src/types/chromosomes/mod.rs` (converted to directory module so submodule resolution works)
- `src/chromosomes/` → `src/types/chromosomes/`
- `src/genotypes.rs` → `src/types/genotypes/mod.rs`
- `src/genotypes/` → `src/types/genotypes/`
- `src/lib.rs`: replaced bare `pub mod chromosomes;` and `pub mod genotypes;` with `#[path = "types/chromosomes/mod.rs"]` and `#[path = "types/genotypes/mod.rs"]`

## Key Deviation

Plan specified `#[path = "types/chromosomes.rs"]` (flat file), but Rust resolves submodules relative to the file's parent directory. With a flat `.rs` file in `src/types/`, submodules would be sought at `src/types/binary.rs` — wrong. Fixed by converting to directory module form (`mod.rs` inside the directory), then using `#[path = "types/chromosomes/mod.rs"]`.

## Verification

- `cargo test`: 657 passed, 20 ignored
- `cargo clippy`: 0 issues
