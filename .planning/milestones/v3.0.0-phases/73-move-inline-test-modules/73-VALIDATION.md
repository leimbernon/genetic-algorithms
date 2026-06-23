---
phase: 73
slug: move-inline-test-modules
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-18
---

# Phase 73 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` |
| **Config file** | `Cargo.toml` (no explicit `[[test]]` entries needed for this phase) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo test --features benchmarks --tests` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo test --features benchmarks --tests`
- **Before `/gsd-verify-work`:** Full suite must be green + `grep -rn '#\[cfg(test)\]' src/` returns zero matches
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | SC-1 (grep clean) | — | N/A | shell | `grep -rn '#\[cfg(test)\]' src/ \|\| echo "CLEAN"` | ✅ | ⬜ pending |
| TBD | TBD | TBD | SC-2 (tests pass) | — | N/A | integration | `cargo test` | ✅ | ⬜ pending |
| TBD | TBD | TBD | SC-3 (no regression) | — | N/A | integration | `cargo test -- --list 2>&1 \| wc -l` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_benchmarks.rs` — new harness file needed before benchmark test files can be used
- [ ] `tests/benchmarks/dtlz.rs` — new benchmark test file
- [ ] `tests/benchmarks/single_objective.rs` — new benchmark test file
- [ ] `tests/benchmarks/zdt.rs` — new benchmark test file
- [ ] `tests/operations/test_mutation_levy_flight.rs` — new levy_flight test file (rewritten public-API tests)

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
