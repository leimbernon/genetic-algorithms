---
phase: 53
slug: tree-chromosome-gpga-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-25
---

# Phase 53 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test gp` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test gp`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green + `cargo check --target wasm32-unknown-unknown`
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 53-01-01 | 01 | 0 | CHR-03 | — | N/A | unit | `cargo test --test gp test_gp_node_trait` | ❌ W0 | ⬜ pending |
| 53-01-02 | 01 | 0 | CHR-03 | — | N/A | unit | `cargo test --test gp test_tree_chromosome_not_linear` | ❌ W0 | ⬜ pending |
| 53-01-03 | 01 | 0 | CHR-03 | — | N/A | unit | `cargo test --test gp test_node_drop_iterative` | ❌ W0 | ⬜ pending |
| 53-01-04 | 01 | 0 | CHR-07 | — | N/A | unit | `cargo test --test gp test_display_prefix_sexpr` | ❌ W0 | ⬜ pending |
| 53-01-05 | 01 | 0 | CHR-04 | — | N/A | unit | `cargo test --test gp test_math_node_gp_node_impl` | ❌ W0 | ⬜ pending |
| 53-01-06 | 01 | 0 | CHR-04 | — | N/A | unit | `cargo test --test gp test_bool_node_gp_node_impl` | ❌ W0 | ⬜ pending |
| 53-02-01 | 02 | 1 | CHR-03 | — | N/A | unit | `cargo test --test gp test_subtree_crossover` | ❌ W0 | ⬜ pending |
| 53-02-02 | 02 | 1 | CHR-03 | — | N/A | unit | `cargo test --test gp test_point_mutation` | ❌ W0 | ⬜ pending |
| 53-02-03 | 02 | 1 | CHR-03 | — | N/A | unit | `cargo test --test gp test_hoist_mutation` | ❌ W0 | ⬜ pending |
| 53-02-04 | 02 | 1 | CHR-05 | — | N/A | unit | `cargo test --test gp test_bloat_limit_crossover` | ❌ W0 | ⬜ pending |
| 53-02-05 | 02 | 1 | CHR-05 | — | N/A | unit | `cargo test --test gp test_bloat_limit_mutation` | ❌ W0 | ⬜ pending |
| 53-03-01 | 03 | 2 | CHR-04 | — | N/A | integration | `cargo test --test gp test_gpga_ramp_half_and_half` | ❌ W0 | ⬜ pending |
| 53-03-02 | 03 | 2 | CHR-04 | — | N/A | integration | `cargo test --test gp test_gpga_run_symbolic_regression` | ❌ W0 | ⬜ pending |
| 53-03-03 | 03 | 2 | CHR-05 | — | N/A | unit | `cargo test --test gp test_generation_stats_avg_node_count` | ❌ W0 | ⬜ pending |
| 53-04-01 | 04 | 3 | CHR-06 | — | N/A | unit | `cargo test --features serde --test gp test_serde_deep_tree` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/gp.rs` — stub test file with all test function signatures from the map above
- [ ] Wave 0 stubs compile with `cargo test --test gp` (tests may fail, but must compile)

*Existing cargo test infrastructure covers all phase requirements — no new test framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Compile-error (not runtime panic) when GpChromosome used with linear operator | CHR-03 | Compile-time type check — cannot be tested with cargo test | Add `GpChromosome` as generic arg to `Crossover::SinglePoint.crossover(...)` — should fail to compile with clear LinearChromosome bound error |
| WASM compatibility | All | Requires cross-compilation target | `cargo check --target wasm32-unknown-unknown` and `cargo check --target wasm32-unknown-unknown --features serde` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
