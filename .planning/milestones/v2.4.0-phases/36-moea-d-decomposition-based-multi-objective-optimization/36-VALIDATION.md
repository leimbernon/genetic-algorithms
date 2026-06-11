---
phase: 36
slug: moea-d-decomposition-based-multi-objective-optimization
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-09
---

# Phase 36 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test moead 2>&1 \| tail -20` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test moead 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 36-01-01 | 01 | 1 | MOO-02 | — | N/A | unit | `cargo test engines::moead::configuration` | ❌ W0 | ⬜ pending |
| 36-01-02 | 01 | 1 | MOO-02 | — | N/A | unit | `cargo test engines::moead` | ❌ W0 | ⬜ pending |
| 36-02-01 | 02 | 2 | MOO-02 | — | N/A | integration | `cargo test --test moead` | ❌ W0 | ⬜ pending |
| 36-03-01 | 03 | 3 | MOO-02 | — | N/A | integration | `cargo test --test moead` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/engines/moead/mod.rs` — integration test stubs for MoeaDGa<U> engine
- [ ] `src/engines/moead/mod.rs` — module skeleton to prevent compilation failures during incremental implementation

*Existing infrastructure (cargo test, clippy) covers build validation across all tasks.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| DTLZ2 example produces visually spread Pareto front | MOO-02 | Pareto quality is subjective / no automated spread metric | Run `cargo run --example moead_dtlz2` and inspect output |
| WASM build succeeds | CLAUDE.md | No WASM test runner in CI | Run `cargo check --target wasm32-unknown-unknown` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
