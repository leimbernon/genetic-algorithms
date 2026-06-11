---
phase: 07
slug: list-genotype
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + `cargo test` |
| **Config file** | none (standard Cargo test runner) |
| **Quick run command** | `cargo test list` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test list`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | LIST-01 | unit | `cargo test list_gene` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | LIST-01 | unit | `cargo test list_gene_validation` | ❌ W0 | ⬜ pending |
| 07-01-03 | 01 | 1 | LIST-01 | unit | `cargo test list_gene_set_id` | ❌ W0 | ⬜ pending |
| 07-01-04 | 01 | 1 | LIST-02 | unit | `cargo test list_chromosome_new` | ❌ W0 | ⬜ pending |
| 07-01-05 | 01 | 1 | LIST-02 | unit | `cargo test list_chromosome_trait` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 2 | LIST-03 | unit | `cargo test list_chromosome_swap` | ❌ W0 | ⬜ pending |
| 07-02-02 | 02 | 2 | LIST-03 | unit | `cargo test list_value_mutation` | ❌ W0 | ⬜ pending |
| 07-02-03 | 02 | 2 | LIST-03 | unit | `cargo test list_value_mutation_different` | ❌ W0 | ⬜ pending |
| 07-02-04 | 02 | 2 | LIST-04 | unit | `cargo test list_initializer` | ❌ W0 | ⬜ pending |
| 07-02-05 | 02 | 2 | LIST-04 | unit | `cargo test list_initializer_id` | ❌ W0 | ⬜ pending |
| 07-02-06 | 02 | 2 | LIST-04 | unit | `cargo test list_initializer_no_rep` | ❌ W0 | ⬜ pending |
| 07-02-07 | 02 | 2 | LIST-02 | unit | `cargo test --features serde list_serde` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/genotypes/list.rs` — `#[cfg(test)] mod tests` with LIST-01 unit tests
- [ ] `src/chromosomes/list.rs` — `#[cfg(test)] mod tests` with LIST-02 unit tests
- [ ] `src/operations/mutation/list_value.rs` — `#[cfg(test)] mod tests` with LIST-03 mutation tests
- [ ] `src/initializers/list_initializer.rs` — `#[cfg(test)] mod tests` with LIST-04 initializer tests
- [ ] `tests/chromosomes/test_list.rs` — integration tests for LIST-02 and LIST-03 (operator integration)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `ListChromosome` works in full GA run | LIST-03 | E2E behavior | Run a short GA with `ListChromosome` using all 4 operator types; confirm it completes without panic |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
