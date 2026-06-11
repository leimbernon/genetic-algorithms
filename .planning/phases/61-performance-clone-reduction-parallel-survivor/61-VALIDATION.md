---
phase: 61
slug: performance-clone-reduction-parallel-survivor
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-08
---

# Phase 61 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + Criterion benchmarks |
| **Config file** | `Cargo.toml` (bench section) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 61-01-01 | 01 | 1 | D-10/D-11/D-12 | — | N/A | bench | `cargo bench --bench rastrigin` | ✅ (created by this task) | ⬜ pending |
| 61-02-01 | 02 | 1 | D-06/D-07/D-08/D-09 | — | N/A | unit | `cargo test && cargo check --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| 61-03-01 | 03 | 1 | D-01/D-03/D-04/D-05 | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |
| 61-04-01 | 04 | 2 | D-13 | — | N/A | bench+ci | `cargo bench --bench rastrigin && cargo test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.* The `benches/rastrigin.rs` file is itself a Plan 01 deliverable (D-10), not an external Wave 0 prerequisite — Plan 01 creates the harness in Wave 1 and Plan 04 (Wave 2) consumes it. No prerequisite scaffold work is required before Wave 1 begins.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ≥10% wall-time reduction on rastrigin bench at pop=500 | D-13 / Success Criterion 1 | Requires before/after bench runs with `cargo bench` | Run `cargo bench --bench rastrigin` on base commit, apply changes, run again; compare mean times |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (no Wave 0 work needed — bench file is a Plan 01 in-phase deliverable)
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved
