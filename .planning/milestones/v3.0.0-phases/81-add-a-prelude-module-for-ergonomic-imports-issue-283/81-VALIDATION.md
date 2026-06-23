---
phase: 81
slug: add-a-prelude-module-for-ergonomic-imports-issue-283
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-22
---

# Phase 81 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Built-in `cargo test` + `cargo test --doc` |
| **Config file** | `Cargo.toml` `[dev-dependencies]` |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --doc && cargo bench --no-run` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test && cargo clippy`
- **After every plan wave:** Run `cargo test && cargo test --doc && cargo bench --no-run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 81-01-01 | 01 | 1 | SC-1 | — | N/A | compile-check | `cargo check` | ❌ W0 | ⬜ pending |
| 81-01-02 | 01 | 1 | SC-3 | — | N/A | integration | `cargo test test_prelude` | ❌ W0 | ⬜ pending |
| 81-01-03 | 01 | 1 | SC-4 | — | N/A | compile-check | `cargo check` with prelude glob | ❌ W0 | ⬜ pending |
| 81-01-04 | 01 | 1 | SC-7 | — | N/A | doc-build | `cargo doc --no-deps` | ✅ (CI) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_prelude.rs` — compile-check test that `use genetic_algorithms::prelude::*;` succeeds and key types are accessible
- [ ] `tests/test_prelude_minimal_ga.rs` — integration test that a minimal GA can be built and run using only prelude imports + concrete types

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No glob-import name collisions in fresh file | SC-4 | Compile-check is sufficient; no runtime behavior | Create a test file using `prelude::*` only, run `cargo check` |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** {pending / approved 2026-06-22}
