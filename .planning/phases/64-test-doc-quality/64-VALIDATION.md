---
phase: 64
slug: test-doc-quality
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 64 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`, `cargo llvm-cov`, `cargo clippy`, `cargo doc`) |
| **Config file** | `Cargo.toml`, `.github/workflows/` |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy -- -D warnings && cargo doc --no-deps` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green + `cargo llvm-cov --all-features --summary-only` shows ≥80%
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| baseline | 01 | 0 | D-05 | coverage | `cargo llvm-cov --all-features --summary-only` | ⬜ pending |
| ci-gate | 02 | 1 | D-01/D-02 | CI | `cargo llvm-cov --all-features` exits non-zero below 80% | ⬜ pending |
| allow-removal | 03 | 1 | D-06–D-10 | lint | `cargo clippy -- -D warnings` exits 0 | ⬜ pending |
| de-params | 04 | 1 | D-07 | unit | `cargo test de` | ⬜ pending |
| doc-examples | 05 | 2 | D-11–D-13 | doc | `cargo test --doc` | ⬜ pending |
| coverage-tests | 06 | 2 | D-14 | unit | `cargo llvm-cov --all-features --summary-only` ≥80% | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Run `cargo llvm-cov --all-features --summary-only` — capture per-module baseline
- [ ] Run `grep -rn "#\[allow(" src/ --include="*.rs"` — confirm suppression inventory matches RESEARCH.md
- [ ] Identify the 3–5 lowest-coverage modules in `src/engines/` and `src/operations/` from the baseline report

*Wave 0 is a data-gathering step: its outputs drive which tests are written in Wave 2.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CI coverage gate fails on regression | D-02 | Requires a PR that drops coverage intentionally | Temporarily comment out a test, verify CI step fails |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
