---
phase: 25
slug: alternative-metaheuristics
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-26
---

# Phase 25 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in + criterion benchmarks) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd-verify-work`:** Full suite must be green (including `cargo doc --no-deps`)
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 25-01-01 | 01 | 1 | STRUCT-02 | — | N/A | filesystem+compile | `ls src/types/chromosomes.rs src/types/genotypes.rs && test ! -f src/chromosomes.rs && cargo test 2>&1 \| tail -5` | ✅ | ⬜ pending |
| 25-01-02 | 01 | 1 | STRUCT-02 | — | N/A | compile | `cargo test && cargo clippy` | ✅ | ⬜ pending |
| 25-02-01 | 02 | 2 | STRUCT-03 | — | N/A | filesystem+compile | `ls src/observe/observer/mod.rs src/observe/checkpoint.rs && test ! -d src/observer && cargo test 2>&1 \| tail -5` | ✅ | ⬜ pending |
| 25-02-02 | 02 | 2 | STRUCT-03 | — | N/A | compile | `cargo test && cargo test --features serde && cargo clippy` | ✅ | ⬜ pending |
| 25-03-01 | 03 | 3 | STRUCT-01 | — | N/A | filesystem+compile | `ls src/engines/ga.rs src/engines/island/mod.rs src/engines/de/mod.rs && test ! -f src/ga.rs && cargo test 2>&1 \| tail -5` | ✅ | ⬜ pending |
| 25-03-02 | 03 | 3 | STRUCT-01, STRUCT-04 | — | N/A | regression | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The validation for this phase is compilation + existing test pass — no new test files needed (STRUCT-04 passes if `cargo test` is green post-restructure).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Downstream user path compatibility | STRUCT-01/02/03 | Requires building a consuming crate | Create a minimal crate with `genetic_algorithms` as dependency; verify `use genetic_algorithms::...` paths compile |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
