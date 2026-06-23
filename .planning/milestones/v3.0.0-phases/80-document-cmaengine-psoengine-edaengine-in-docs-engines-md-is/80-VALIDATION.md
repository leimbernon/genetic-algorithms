---
phase: 80
slug: document-cmaengine-psoengine-edaengine-in-docs-engines-md-is
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-22
---

# Phase 80 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo doc (rustdoc) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo doc --no-deps 2>&1 \| grep -c "^warning" && echo "0 expected"` |
| **Full suite command** | `cargo doc --no-deps` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo doc --no-deps 2>&1 | grep "^warning"` (must be empty)
- **After every plan wave:** Run `cargo doc --no-deps` (full, must exit 0 with 0 warnings)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 80-01-01 | 01 | 1 | docs/cma.md | — | N/A | manual | `test -f docs/cma.md` | ❌ W0 | ⬜ pending |
| 80-01-02 | 01 | 1 | docs/pso.md | — | N/A | manual | `test -f docs/pso.md` | ❌ W0 | ⬜ pending |
| 80-01-03 | 01 | 1 | docs/eda.md | — | N/A | manual | `test -f docs/eda.md` | ❌ W0 | ⬜ pending |
| 80-02-01 | 02 | 2 | docs/engines.md updated | — | N/A | manual | `grep -q "CmaEngine" docs/engines.md` | ✅ | ⬜ pending |
| 80-02-02 | 02 | 2 | docs/index.md updated | — | N/A | manual | `grep -q "cma.md" docs/index.md` | ✅ | ⬜ pending |
| 80-03-01 | 03 | 3 | Zero rustdoc warnings | — | N/A | automated | `cargo doc --no-deps 2>&1 \| grep "^warning" \| wc -l \| grep -q "^0$"` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `docs/cma.md` — new file (does not exist yet)
- [ ] `docs/pso.md` — new file (does not exist yet)
- [ ] `docs/eda.md` — new file (does not exist yet)

*All other files exist; only the three new pages are Wave 0 creates.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CMA docs accurate (sigma0, lambda, restart) | Phase goal SC-1 | Content accuracy, not automatable | Read docs/cma.md; verify sigma0 heuristic, lambda=0 auto, IPOP/BIPOP are documented |
| PSO docs accurate (inertia, topology, coefficients) | Phase goal SC-2 | Content accuracy | Read docs/pso.md; verify only Constant/LinearDecay inertia and Global/Ring topology documented |
| EDA docs accurate (Bernoulli vs Gaussian split) | Phase goal SC-3 | Content accuracy | Read docs/eda.md; verify Bernoulli/EdaEngine vs Gaussian/EdaRealEngine duality is clear |
| Overview table in engines.md has CMA/PSO/EDA rows | Phase goal SC-4 | Visual review | Check engines.md overview table includes three new engine rows |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
