---
phase: 09
slug: visualization
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none — cargo built-in |
| **Quick run command** | `cargo test --features visualization test_visualization` |
| **Full suite command** | `cargo test --features visualization && cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features visualization test_visualization`
- **After every plan wave:** Run `cargo test --features visualization && cargo test && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green (including `cargo test --features serde`)
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | VIZ-04 | compile | `cargo build` (no feature) succeeds | ❌ Wave 0 | ⬜ pending |
| 09-01-02 | 01 | 1 | VIZ-01 | integration | `cargo test --features visualization viz_fitness` | ❌ Wave 0 | ⬜ pending |
| 09-01-03 | 01 | 1 | VIZ-01 | unit | `cargo test --features visualization viz_fitness_insufficient` | ❌ Wave 0 | ⬜ pending |
| 09-01-04 | 01 | 1 | VIZ-01 | unit | `cargo test --features visualization viz_fitness_bad_ext` | ❌ Wave 0 | ⬜ pending |
| 09-02-01 | 02 | 2 | VIZ-02 | integration | `cargo test --features visualization viz_diversity` | ❌ Wave 0 | ⬜ pending |
| 09-02-02 | 02 | 2 | VIZ-02 | unit | `cargo test --features visualization viz_diversity_insufficient` | ❌ Wave 0 | ⬜ pending |
| 09-02-03 | 02 | 2 | VIZ-03 | integration | `cargo test --features visualization viz_histogram` | ❌ Wave 0 | ⬜ pending |
| 09-02-04 | 02 | 2 | VIZ-03 | unit | `cargo test --features visualization viz_histogram_empty` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_visualization.rs` — integration + unit tests covering VIZ-01 through VIZ-04 (file does not exist yet; must be created with `#![cfg(feature = "visualization")]` guard at top, following `tests/test_serde.rs` line 5 pattern)
- [ ] `src/visualization/mod.rs` — new module with public API stubs (does not exist yet; stub must compile under `cargo test --features visualization`)

**Test file pattern for VIZ-01/02/03:** Write chart to `std::env::temp_dir()` path, assert `Path::new(&out_path).exists()`, then `std::fs::remove_file` — avoids repo artifacts while confirming the backend wrote bytes.

**VIZ-04 compile test:** `cargo build` (default features, no `visualization`) must succeed and must NOT expose `genetic_algorithms::visualization`. Confirmed by the `#![cfg(feature = "visualization")]` guard in `test_visualization.rs`.

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
