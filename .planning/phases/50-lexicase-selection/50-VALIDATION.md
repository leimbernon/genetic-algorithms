---
phase: 50
slug: lexicase-selection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-22
---

# Phase 50 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test test_operations test_selection_lexicase` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test test_operations test_selection_lexicase`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 50-01-01 | 01 | 1 | TRAITS-01 | — | N/A | unit | `cargo test --test test_operations test_multi_case_fitness_trait_roundtrip` | ❌ W0 | ⬜ pending |
| 50-01-02 | 01 | 1 | SEL-02 | — | N/A | unit | `cargo test --test test_operations test_lexicase_case_order_is_shuffled` | ❌ W0 | ⬜ pending |
| 50-01-03 | 01 | 1 | SEL-03 | — | N/A | unit | `cargo test --test test_operations test_epsilon_lexicase_fixed_tolerance` | ❌ W0 | ⬜ pending |
| 50-02-01 | 02 | 2 | SEL-02 | — | N/A | integration | `cargo test --test test_operations test_lexicase_syncs_scalar_fitness_to_mean` | ❌ W0 | ⬜ pending |
| 50-02-02 | 02 | 2 | SEL-02 | — | N/A | integration | `cargo test --test test_operations test_lexicase_produces_more_specialists_than_tournament` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/operations/test_selection_lexicase.rs` — stubs for TRAITS-01, SEL-02, SEL-03
- [ ] `tests/operations/test_selection_lexicase_diversity.rs` — stub for SEL-02 specialist diversity test
- [ ] `tests/test_operations.rs` — add `mod test_selection_lexicase;` and `mod test_selection_lexicase_diversity;` declarations
- [ ] Existing `tests/` infrastructure covers framework (cargo test)

*Note: No new test framework installation needed — cargo test is already configured.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `cargo check --target wasm32-unknown-unknown` passes | SEL-02, SEL-03 | Requires wasm32 target toolchain | Run `cargo check --target wasm32-unknown-unknown` after all implementation tasks |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
