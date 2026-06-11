---
phase: 37
slug: spea2-strength-pareto-evolutionary-algorithm
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 37 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test --features serde`
- **Before `/gsd-verify-work`:** Full suite must be green: `cargo test --features serde && cargo clippy && cargo doc --no-deps`
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 37-01-01 | 01 | 1 | MOO-03 | T-37-01 / — | Validate config (archive_size > 0, archive_size <= pop_size) | unit | `cargo test --lib spea2` | ✅ | ⬜ pending |
| 37-01-02 | 01 | 1 | MOO-03 | T-37-02 / — | No panics from bad config values | integration | `cargo test engines::spea2` | ✅ | ⬜ pending |
| 37-01-03 | 01 | 1 | MOO-03 | — / N/A | N/A | integration | `cargo test engines::spea2` | ✅ | ⬜ pending |
| 37-01-04 | 01 | 1 | MOO-03 | — / N/A | N/A | integration | `cargo test engines::spea2` | ✅ | ⬜ pending |
| 37-01-05 | 01 | 1 | MOO-03 | — / N/A | N/A | unit | `cargo test --lib spea2` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/engines/spea2/` — integration test directory with ZDT1 test
- [ ] `cargo check --target wasm32-unknown-unknown` — WASM compatibility verified in CI

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ZDT1 Pareto front convergence | MOO-03 | Visual assessment of convergence quality | Run example, plot archive front against theoretical f2 = 1 - sqrt(f1) |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
