---
phase: 69
slug: build-perf-m3-major-refactors
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-15
---

# Phase 69 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (nextest) + cargo check |
| **Config file** | `Cargo.toml`, `.cargo/config.toml` |
| **Quick run command** | `cargo test --all-features` |
| **Full suite command** | `cargo test --all-features && cargo test --no-default-features --features logging && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~60–90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --all-features`
- **After every plan wave:** Run full suite command above
- **Before `/gsd:verify-work`:** Full suite must be green + wasm-check green + golden tests byte-identical
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 69-01-* | 01 | 0 | — | — | N/A | compile+test | `cargo test --all-features && cargo bench --no-run` | ✅ | ⬜ pending |
| 69-02-* | 02 | 1 | — | — | N/A | compile+wasm | `cargo check --target wasm32-unknown-unknown && cargo test --no-default-features --features logging` | ✅ | ⬜ pending |
| 69-03-* | 03 | 1 | — | — | N/A | compile+golden | `cargo test --all-features && cargo test --no-default-features --features logging` | ✅ | ⬜ pending |
| 69-04-* | 04 | 2 | — | — | N/A | compile+expand | `cargo test --all-features && cargo expand > /tmp/after.txt` | ✅ | ⬜ pending |
| 69-05-* | 05 | 2 | — | — | N/A | doc+gate | `cargo doc --no-deps && bash bench/build_perf.sh` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing test infrastructure (cargo test) covers all phase requirements.
- No new test framework installation needed.
- `cargo-expand` and `cargo-public-api` installation is a Wave 2 prerequisite task in plan 69-04.

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Bench median ±3% tolerance | Success Criteria 1 | Requires same machine + same load | Run `bash bench/build_perf.sh` before and after port, compare output |
| build-perf-gate ≥10% improvement | Success Criteria 4 | Requires timing measurement | Run `bash bench/build_perf.sh` on `--no-default-features --features logging` before and after Phase 69 |
| `cargo expand` symbol diff zero | Success Criteria 3 | Visual diff of expanded output | Run `cargo expand > before.txt` before split, `cargo expand > after.txt` after, `diff before.txt after.txt` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
