---
phase: 48
slug: new-genotype-types
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-21
---

# Phase 48 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | None — standard Rust test discovery |
| **Quick run command** | `cargo test && cargo clippy` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test && cargo clippy`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 48-??-01 | 01 | 1 | GEN-01 | — | N/A | unit | `cargo test --test test_initializers` | ❌ W0 | ⬜ pending |
| 48-??-02 | 01 | 1 | GEN-01 | — | N/A | unit | `cargo test --test test_chromosomet_core` | partial | ⬜ pending |
| 48-??-03 | 01 | 1 | GEN-01 | — | N/A | unit | `cargo test --test test_engines` | partial | ⬜ pending |
| 48-??-04 | 01 | 1 | GEN-01 | — | N/A | unit | `cargo test --test test_crossover_pmx` | ✅ needs case | ⬜ pending |
| 48-??-05 | 02 | 1 | GEN-02 | — | N/A | smoke | `cargo run --example job_scheduling` | ✅ (post-migrate) | ⬜ pending |
| 48-??-06 | 03 | 2 | GEN-03 | — | N/A | unit | `cargo test --test test_initializers` | ❌ W0 | ⬜ pending |
| 48-??-07 | 03 | 2 | GEN-03 | — | N/A | unit | `cargo test --test test_mutation_creep_gaussian` | partial | ⬜ pending |
| 48-??-08 | 04 | 3 | GEN-04 | — | N/A | unit | `cargo test tests/types/chromosomes/test_multi_unique.rs` | ❌ W0 | ⬜ pending |
| 48-??-09 | 04 | 3 | GEN-04 | — | N/A | unit | `cargo test --test test_crossover_multi_group` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/types/chromosomes/test_unique.rs` — stubs for GEN-01 ChromosomeT/LinearChromosome impls
- [ ] `tests/types/chromosomes/test_multi_range.rs` — stubs for GEN-03
- [ ] `tests/types/chromosomes/test_multi_unique.rs` — stubs for GEN-04 group_ranges
- [ ] `tests/types/genotypes/test_unique.rs` — stubs for UniqueGenotype GeneT impl
- [ ] `tests/types/genotypes/test_multi_range.rs` — stubs for MultiRangeGenotype GeneT impl
- [ ] `tests/operations/test_crossover_multi_group_pmx.rs` — stubs for MultiGroupPmx correctness
- [ ] `tests/operations/test_crossover_multi_group_ox.rs` — stubs for MultiGroupOx correctness
- [ ] Add `unique_random_initialization` cases to `tests/initializers/test_initializers.rs`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `job_scheduling` example produces valid (non-duplicate) job sequences visually | GEN-02 | Output validation requires human inspection of permutation ordering | Run `cargo run --example job_scheduling` and verify output shows valid permutations |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
