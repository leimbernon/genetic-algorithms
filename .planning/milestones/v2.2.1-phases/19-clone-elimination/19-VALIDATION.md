---
phase: 19
slug: clone-elimination
status: compliant
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-05
---

# Phase 19 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test crossover` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 19-01-01 | 01 | 1 | CLONE-01 | integration | `cargo test` | ✅ | ✅ green |
| 19-01-02 | 01 | 1 | CLONE-04 | unit | `cargo test mutation` | ✅ | ✅ green |
| 19-02-01 | 02 | 1 | CLONE-03 | unit | `cargo test mutation` | ✅ | ✅ green |
| 19-03-01 | 03 | 1 | CLONE-02 | unit | `cargo test crossover` | ✅ | ✅ green |
| 19-03-02 | 03 | 1 | CLONE-02 | unit (fresh metadata) | `cargo test crossover_children_start_with_fresh` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-05

---

## Requirement Coverage

| Requirement | Description | Test File(s) | Status |
|-------------|-------------|--------------|--------|
| CLONE-01 | GA engine defers parent clones to fallback branch only | `tests/test_ga.rs` (integration), borrow-checker | COVERED |
| CLONE-02 | Crossover children built via `U::new()` — fresh state, no parent metadata inherited | `tests/operations/test_crossover_single_point.rs` (fresh metadata test), `tests/operations/test_crossover_arithmetic.rs` (fresh metadata test), all crossover DNA-correctness tests | COVERED |
| CLONE-03 | Numeric mutations use `set_gene()` not `dna().to_vec()` | `tests/operations/test_mutation_range_value.rs`, `tests/operations/test_mutation_creep_gaussian.rs`, `tests/operations/test_mutation_polynomial.rs`, `tests/operations/test_mutation_non_uniform.rs` | COVERED |
| CLONE-04 | Swap/Inversion/Scramble use in-place `dna_mut()` ops | `tests/operations/test_mutation.rs` (lines 10–380+) | COVERED |

---

## Validation Audit 2026-04-05

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

**Gap resolved:** CLONE-02 (PARTIAL → COVERED) — Added `single_point_crossover_children_start_with_fresh_metadata` and `arithmetic_crossover_children_start_with_fresh_metadata` tests. Both assert `child.age == 0` when parents have `age > 0`, directly verifying that `U::new()` / `RangeChromosome::<T>::new()` is used instead of `parent.clone()`.
