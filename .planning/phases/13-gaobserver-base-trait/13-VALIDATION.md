---
phase: 13
slug: gaobserver-base-trait
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — uses Cargo.toml test configuration |
| **Quick run command** | `cargo test test_observer` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test test_observer`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 0 | OBS-01, OBS-02, OBS-04 | unit/integration | `cargo test test_observer` | ❌ W0 | ⬜ pending |
| 13-xx-xx | TBD | 1 | OBS-01 | integration | `cargo test test_observer_hook_fire_counts` | ❌ W0 | ⬜ pending |
| 13-xx-xx | TBD | 1 | OBS-01 | integration | `cargo test test_observer_stagnation_fires` | ❌ W0 | ⬜ pending |
| 13-xx-xx | TBD | 1 | OBS-01 | integration | `cargo test test_observer_extension_triggered` | ❌ W0 | ⬜ pending |
| 13-xx-xx | TBD | 1 | OBS-02 | compile test | `cargo test test_observer_partial_impl_compiles` | ❌ W0 | ⬜ pending |
| 13-xx-xx | TBD | 1 | OBS-03 | bench | `cargo bench ga_run` | ✅ existing | ⬜ pending |
| 13-xx-xx | TBD | 1 | OBS-04 | compile test | `cargo test test_observer_is_object_safe` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_observer.rs` — stubs for OBS-01 (hook fire counts, stagnation, extension), OBS-02 (partial impl compile), OBS-04 (object safety + Send+Sync enforcement)
- [ ] `#[allow(deprecated)]` added to `tests/test_reporter.rs` and `src/reporter/mod.rs` test module — required once `#[deprecated]` is added to `Reporter<U>`

*Existing infrastructure: `benches/ga_run.rs` covers OBS-03 without modification.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Zero-overhead confirmed by benchmark comparison | OBS-03 | Benchmark requires human review of criterion output to compare with/without observer | Run `cargo bench ga_run`, compare ns/iter with and without observer field populated |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
