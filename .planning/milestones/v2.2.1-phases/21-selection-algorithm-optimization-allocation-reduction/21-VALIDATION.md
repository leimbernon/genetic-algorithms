---
phase: 21
slug: selection-algorithm-optimization-allocation-reduction
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-31
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test selection && cargo test niching` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test selection && cargo test niching`
- **After every plan wave:** Run `cargo test && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 1 | ALGO-03 | unit | `cargo test test_selection_rank` | ✅ | ⬜ pending |
| 21-01-02 | 01 | 1 | ALGO-03 | unit | `cargo test test_rank_selection_favors_higher_fitness` | ✅ | ⬜ pending |
| 21-02-01 | 02 | 1 | ALGO-04 | unit | `cargo test test_selection_boltzmann` | ✅ | ⬜ pending |
| 21-02-02 | 02 | 1 | ALGO-04 | unit | `cargo test test_boltzmann_selection_high_temperature_approaches_uniform` | ✅ | ⬜ pending |
| 21-03-01 | 03 | 2 | ALLOC-02 | unit | `cargo test test_apply_fitness_sharing_with_dna` | ❌ W0 | ⬜ pending |
| 21-03-02 | 03 | 2 | ALLOC-02 | unit | `cargo test test_niching` | ✅ | ⬜ pending |
| 21-04-01 | 04 | 3 | ALLOC-01 | integration | `cargo test test_ga` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/niching/test_niching_sharing.rs` — add `test_apply_fitness_sharing_with_dna_matches_matrix_version` test that calls both the old matrix path and the new on-the-fly `apply_fitness_sharing_with_dna` on identical data and asserts equal fitness outputs (covers ALLOC-02 correctness)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Rank Selection source inspection | ALGO-03 | Structural code property — verify `partition_point` call exists | `grep -n "partition_point" src/operations/selection/rank.rs` — must show a match |
| Boltzmann Selection source inspection | ALGO-04 | Structural code property — verify no O(M×N) loop remains | `grep -n "partition_point" src/operations/selection/boltzmann.rs` — must show a match; `grep -n "\.position(" src/operations/selection/boltzmann.rs` — must show no match |
| fitness_values single collection in ga.rs | ALLOC-01 | Structural code property — verify single `fitness_values` Vec before niching | `grep -n "fitness_values" src/ga.rs` — must show one `let mut fitness_values` declaration, no duplicate collections inside the loop |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
