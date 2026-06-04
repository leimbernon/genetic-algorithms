---
phase: 47
slug: architecture-audit-chromosomet-split
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-19
---

# Phase 47 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo check --all-features` |
| **Full suite command** | `cargo test --all-features && cargo test --features serde && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --all-features`
- **After every plan wave:** Run `cargo test --all-features && cargo test --features serde && cargo check --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green + all 10 examples compile
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 47-01-01 | 01 | 1 | ARCH-01 | — | N/A | compile | `cargo check --all-features` | ❌ W0 | ⬜ pending |
| 47-01-02 | 01 | 1 | ARCH-02 | — | N/A | compile | `cargo check --all-features` | ❌ W0 | ⬜ pending |
| 47-02-01 | 02 | 2 | ARCH-04 | — | N/A | compile | `cargo check --all-features` | ❌ W0 | ⬜ pending |
| 47-02-02 | 02 | 2 | ARCH-05 | — | N/A | compile | `cargo check --all-features` | ❌ W0 | ⬜ pending |
| 47-02-03 | 02 | 2 | ARCH-06 | — | N/A | compile | `cargo check --all-features` | ❌ W0 | ⬜ pending |
| 47-03-01 | 03 | 3 | ARCH-03 | — | N/A | compile | `cargo check --all-features` | ❌ W0 | ⬜ pending |
| 47-03-02 | 03 | 3 | ARCH-07 | — | N/A | compile | `cargo build --examples` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `cargo check --all-features` passes on baseline (before changes) — confirm zero pre-existing errors
- [ ] `cargo check --target wasm32-unknown-unknown` passes on baseline
- [ ] `cargo build --examples` compiles all 10 examples on baseline

*Existing infrastructure (cargo test) covers all phase requirements — no new test framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Compiler error message for `Reporter<U>` users is clear and points to `GaObserver<U>` | ARCH-03 | Error message quality is subjective | Temporarily add `use genetic_algorithms::Reporter;` to a test file; verify the compiler error names `GaObserver` as the replacement |
| `MIGRATION.md` before/after examples are accurate | ARCH-03 | Prose correctness | Read `MIGRATION.md`; verify each before/after code block compiles (before → old API error, after → green) |
| CI `examples-smoke.yml` correctly limits generations | ARCH-07 | CI behavior requires push/PR trigger | Inspect `.github/workflows/examples-smoke.yml`; confirm generation-count override flags or env vars are set |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
