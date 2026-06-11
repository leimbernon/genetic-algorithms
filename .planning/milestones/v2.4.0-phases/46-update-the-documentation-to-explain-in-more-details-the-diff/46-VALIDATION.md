---
phase: 46
slug: update-the-documentation-to-explain-in-more-details-the-diff
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 46 — Validation Strategy

> Documentation validation: build-time verification + manual inspection checklists.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo doc + cargo test (doc-tests) |
| **Quick run command** | `cargo doc --no-deps 2>&1 | grep -E "warning\|error"; test $? -eq 1` |
| **Full suite command** | `cargo doc --no-deps && cargo test --doc 2>&1` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo doc --no-deps` for zero warnings
- **After every plan wave:** Run full `cargo doc --no-deps && cargo test --doc`
- **Before verify-work:** Full suite must be green, manual D-01..D-11 checklist complete
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Description | Test Type | Automated Command | Status |
|---------|------|------|-------------|-------------|-----------|-------------------|--------|
| 46-01-01 | 01 | 1 | D-02 | lib.rs crate-level docs rewrite | build | `cargo doc --no-deps 2>&1 | grep -i "error\|warning"; test $? -eq 1` | ⬜ pending |
| 46-01-02 | 01 | 1 | D-05 | README engine table (11 engines) | manual | grep | ⬜ pending |
| 46-01-03 | 01 | 1 | D-06 | README examples table (19 entries) | manual | grep | ⬜ pending |
| 46-02-01 | 02 | 2 | D-01, D-03, D-04 | Per-engine guide docs in docs/ | manual | file existence check | ⬜ pending |
| 46-02-02 | 02 | 2 | D-07 | New engine docs (NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA) | manual | file existence check | ⬜ pending |
| 46-03-01 | 03 | 2 | D-08 | Example inline doc comments | build | `cargo doc --no-deps` | ⬜ pending |
| 46-04-01 | 04 | 3 | D-09 | rustdoc on all public items | build | `cargo doc --no-deps` | ⬜ pending |
| 46-04-02 | 04 | 3 | D-10 | Module-level //! docs expansion | build | `cargo doc --no-deps` | ⬜ pending |
| 46-05-01 | 05 | 3 | D-11 | AI-ready precision verification | manual | checklist | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Enable `#![warn(missing_docs)]` in src/lib.rs during Wave 1 to enforce D-09
- [ ] Add cargo doc to pre-commit hook or CI if not already present
- [ ] Define AI-readability rubric per D-11

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| All 11 engines listed in README table | D-05 | Cannot grep for count correctness | Count engine rows in README table |
| All 19 examples listed with cargo run commands | D-06 | Cannot grep for count correctness | Count example rows in README table |
| Engine guides follow "ficha técnica" template | D-04 | Structural validation | Spot-check 3 engine guides for all template sections |
| AI-ready precision level | D-11 | Subjective quality check | Sample 3 doc blocks: parameter tables, decision guidance, complete examples |
| Cross-references between similar engines | D-04 | Structural validation | Check 3 engine guides for "When to choose" or cross-ref section |

---

## Validation Sign-Off

- [ ] All tasks have automated or manual verification defined
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] `cargo doc --no-deps` produces zero warnings
- [ ] All D-01 through D-11 checkboxes verified at phase gate
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
