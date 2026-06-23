---
phase: 68
slug: build-perf-m2-dependency-hygiene
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-15
---

# Phase 68 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test --all-features && cargo test --no-default-features && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test --all-features && cargo test --no-default-features && cargo check --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 68-01-01 | 01 | 0 | D-01 | — | N/A | compile | `cargo build` | ✅ | ⬜ pending |
| 68-01-02 | 01 | 0 | D-01 | — | N/A | compile | `cargo test` | ✅ | ⬜ pending |
| 68-01-03 | 01 | 0 | D-01 | — | N/A | integration | `cargo test --test test_no_logger_installed` | ❌ W0 | ⬜ pending |
| 68-02-01 | 02 | 1 | D-02/D-03/D-04 | — | N/A | compile | `cargo check --no-default-features` | ✅ | ⬜ pending |
| 68-02-02 | 02 | 1 | D-02/D-03/D-04 | — | N/A | compile | `cargo check --features logging` | ✅ | ⬜ pending |
| 68-02-03 | 02 | 1 | D-04 | — | N/A | compile | `cargo check --target wasm32-unknown-unknown` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_no_logger_installed.rs` — integration test asserting GA does not install a logger (referenced in task 68-01-03 as ❌ W0 — must be created as part of Plan 68-01)

*Existing `cargo test` infrastructure covers all other phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| build-perf-gate confirms ≥15% clean-build improvement | SC-7 | Timing-based, environment-dependent | Run `cargo clean && time cargo build` before and after; compare wall-clock |
| 12-15 fewer transitive crates | SC-7 | Count-based | Run `cargo tree --no-dedupe \| grep -c '^[a-z]'` before and after |
| CC-3 golden tests byte-identical | SC-8 | Byte comparison | Run `cargo test --test golden` and diff outputs |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
