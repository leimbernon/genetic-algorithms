---
phase: 60
slug: batch-fitness-fitness-cache-extension
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-07
audited: 2026-06-10
---

# Phase 60 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test (`cargo test`) |
| **Config file** | none — standard Cargo test infrastructure |
| **Quick run command** | `cargo test --lib 2>&1 | tail -20` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 60-01-01 | 01 | 1 | SC-1 | — | N/A | unit | `cargo test batch_evaluator` | ✅ tests/engines/test_ga.rs | ✅ green |
| 60-01-02 | 01 | 1 | SC-1 | — | N/A | unit | `cargo test wrap_with_cache` | ✅ tests/fitness/test_cache.rs | ✅ green |
| 60-02-01 | 02 | 1 | SC-2 | — | N/A | unit | `cargo test cache_stats` | ✅ tests/test_stats.rs | ✅ green |
| 60-02-02 | 02 | 1 | SC-2/3 | — | N/A | unit | `cargo test --features serde cache_stats_serde_compat` | ✅ tests/test_stats.rs | ✅ green |
| 60-03-01 | 03 | 2 | SC-1/2/4 | — | N/A | integration | `cargo test batch_and_cache && cargo clippy -- -D warnings` | ✅ tests/engines/cma/test_cma.rs | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/engines/test_ga.rs::batch_evaluator_tests` — 8 active tests for `BatchFitnessEvaluator` trait and `Ga` wiring
- [x] `tests/fitness/test_cache.rs` — `wrap_with_cache_returns_handle` + full FitnessCache unit tests
- [x] `tests/test_stats.rs` — `cache_stats_default_none` + `cache_stats_serde_compat_old_checkpoint`
- [x] `tests/engines/cma/test_cma.rs::batch_and_cache_tests` — 5 active tests for `CmaEngine` batch eval wiring

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WASM compile check | SC-3 | No WASM test harness in CI | `cargo check --target wasm32-unknown-unknown` |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** 2026-06-10

---

## Validation Audit 2026-06-10

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

**Gap resolved:** `cache_stats_serde_compat_old_checkpoint` — was `#[ignore = "Wave 0 stub"]`, implemented as active `#[cfg(feature = "serde")]` test verifying backward-compat deserialization of checkpoints missing `cache_hits`/`cache_misses` fields.
