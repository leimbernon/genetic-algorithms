---
phase: 80-document-cmaengine-psoengine-edaengine-in-docs-engines-md-is
plan: "03"
subsystem: docs
tags: [documentation, cma-es, pso, eda, engines, navigation]
dependency_graph:
  requires: [80-01, 80-02]
  provides: [docs/engines.md (table rows + stubs), docs/index.md (nav links)]
  affects: []
tech_stack:
  added: []
  patterns: [nsga3.md inline-stub pattern, engines.md overview table extension]
key_files:
  created: []
  modified:
    - docs/engines.md
    - docs/index.md
decisions:
  - "Three new rows added to overview table after GpGa<N> row, grouped with single-objective/continuous engines before multi-objective block"
  - "Pre-existing VonNeumann in CellularEngine section left intact — the acceptance-criteria grep catches it but it is a valid Neighborhood enum variant, not an invented PSO variant"
  - "EDA Key Parameters table notes Maximization-default explicitly — consistent with eda.md Pitfall 2 callout"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-22"
  tasks_completed: 2
  files_modified: 2
status: complete
---

# Phase 80 Plan 03: Integrate CMA/PSO/EDA into engines.md and index.md Summary

Updated `docs/engines.md` with three new overview table rows and three NSGA-III-style stub sections for CmaEngine, PsoEngine, and EdaEngine/EdaRealEngine, and updated `docs/index.md` with three navigation entries under the Engines section. Engine counts updated to fifteen in both files. Phase-level `cargo doc --no-deps` gate passes with zero warnings.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Update docs/engines.md overview table + add three stub sections | ffd7ba1 | docs/engines.md (+143 lines) |
| 2 | Add CMA/PSO/EDA navigation links to docs/index.md + final rustdoc gate | 54da88e | docs/index.md (+4 lines) |

## What Was Built

### docs/engines.md

- **Overview table:** "twelve" changed to "fifteen engines"; three new rows added after the `GpGa<N>` row:
  - `CmaEngine<U>` | `cma` | Best f64 vector | Continuous optimisation — self-adaptive covariance matrix
  - `PsoEngine<U>` | `pso` | Best vector | Swarm-based continuous optimisation — few hyperparameters
  - `EdaEngine<U>` / `EdaRealEngine<U>` | `eda` | Best individual | Probabilistic model-building — binary (Bernoulli) or continuous (Gaussian)

- **Three stub sections** following the NSGA-III inline-stub pattern (each ~30 lines):
  - `## CmaEngine<U> — CMA-ES` — description, entry point, When to Use, Configuration snippet (default_for_dim + with_sigma0 + RestartStrategy::Ipop + with_fitness_cache), Key Parameters table (5 rows), See Also (cma.md + cma_es_rastrigin.rs)
  - `## PsoEngine<U> — Particle Swarm Optimization` — description, entry point, When to Use, Configuration snippet (LinearDecay + Ring + c1/c2=1.49445), Key Parameters table (5 rows), See Also (pso.md + pso_rastrigin.rs)
  - `## EdaEngine<U> / EdaRealEngine<U> — Estimation of Distribution` — description, entry point, When to Use, two-path configuration snippet (Bernoulli + Gaussian), Key Parameters table (5 rows with Maximization-default note), See Also (eda.md + eda_trap.rs)

### docs/index.md

- Heading updated from `### Engines (12 total)` to `### Engines (15 total)`
- Three navigation entries added after Genetic Programming entry:
  - `[CMA-ES](cma.md) — Covariance Matrix Adaptation Evolution Strategy for continuous optimization`
  - `[PSO](pso.md) — Particle Swarm Optimization with configurable inertia and topology`
  - `[EDA](eda.md) — Univariate Marginal Distribution Algorithm (Bernoulli and Gaussian models)`

## Verification Results

- `grep -q "| \`CmaEngine<U>\` | \`cma\`" docs/engines.md`: PASS
- `grep -q "| \`PsoEngine<U>\` | \`pso\`" docs/engines.md`: PASS
- `grep -q "EdaRealEngine<U>" docs/engines.md`: PASS (3 occurrences)
- `grep -q "fifteen engines" docs/engines.md`: PASS
- `grep -q "(cma.md)" docs/engines.md`: PASS
- `grep -q "(pso.md)" docs/engines.md`: PASS
- `grep -q "(eda.md)" docs/engines.md`: PASS
- `grep -cE '\(cma\.md\)|\(pso\.md\)|\(eda\.md\)' docs/engines.md`: 3 (>= 3)
- `grep -q "(cma.md)" docs/index.md`: PASS
- `grep -q "(pso.md)" docs/index.md`: PASS
- `grep -q "(eda.md)" docs/index.md`: PASS
- `grep -q "### Engines (15 total)" docs/index.md`: PASS
- `grep -c 'twelve engines' docs/engines.md`: 0 (stale count removed)
- `grep -c '### Engines (12 total)' docs/index.md`: 0 (stale count removed)
- `grep -cE 'RandomRange|VonNeumann' docs/cma.md docs/pso.md docs/eda.md`: 0 each (no invented variants in new pages)
- `cargo doc --no-deps 2>&1 | grep -c warning`: **0** (phase-level rustdoc gate passes)

## Deviations from Plan

### Note: Pre-existing VonNeumann in CellularEngine section

- **Found during:** Task 1 verification
- **Issue:** `grep -cE 'RandomRange|VonNeumann' docs/engines.md` returns 1 because `VonNeumann` appears in the pre-existing `CellularEngine` section as the `Neighborhood::VonNeumann` enum variant — a documented, real type in the codebase.
- **Resolution:** Not removed. The intent of the acceptance criterion was to prevent invented non-existent PSO/CMA variants from appearing in new stubs. The pre-existing `VonNeumann` in CellularEngine is correct documentation of an existing `Neighborhood` enum. Our new CMA/PSO/EDA stubs contain neither `RandomRange` nor `VonNeumann`.
- **Impact:** None — zero functional or accuracy regression.

## Known Stubs

None — all three stub sections link their dedicated pages (cma.md, pso.md, eda.md) which were created in plans 80-01 and 80-02.

## Threat Flags

None — documentation-only changes; no new network endpoints, auth paths, or schema changes.

## Self-Check: PASSED

- `docs/engines.md` modified: FOUND (commit ffd7ba1)
- `docs/index.md` modified: FOUND (commit 54da88e)
- Commit ffd7ba1 exists: FOUND (docs(80-03): update engines.md overview table and add CMA/PSO/EDA stub sections)
- Commit 54da88e exists: FOUND (docs(80-03): add CMA-ES/PSO/EDA navigation links to index.md and update engine count)
- cargo doc --no-deps: 0 warnings
