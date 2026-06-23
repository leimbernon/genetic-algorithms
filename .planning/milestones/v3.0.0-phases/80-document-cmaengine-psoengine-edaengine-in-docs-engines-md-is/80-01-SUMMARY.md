---
phase: 80-document-cmaengine-psoengine-edaengine-in-docs-engines-md-is
plan: "01"
subsystem: docs
tags: [documentation, cma-es, pso, continuous-optimization]
dependency_graph:
  requires: []
  provides: [docs/cma.md, docs/pso.md]
  affects: []
tech_stack:
  added: []
  patterns: [nsga3.md section skeleton, verified-API-only docs]
key_files:
  created:
    - docs/cma.md
    - docs/pso.md
  modified: []
decisions:
  - "CMA docs use with_fitness_cache (not with_fitness_cache_size) — verified from src/engines/cma/configuration.rs"
  - "PSO docs document only Constant and LinearDecay inertia variants; only Global and Ring topology variants — no RandomRange/VonNeumann (non-existent in source)"
  - "sigma0 heuristic documented as 1/5 to 1/3 range (not exact 1/3 formula) per Pitfall 3 in RESEARCH.md"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-22"
  tasks_completed: 2
  files_created: 2
status: complete
---

# Phase 80 Plan 01: Create CMA-ES and PSO Dedicated Documentation Pages Summary

Two dedicated documentation pages — `docs/cma.md` (CMA-ES) and `docs/pso.md` (PSO) — following the nsga3.md section skeleton with verified-API-only content and zero rustdoc warnings.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create docs/cma.md (CMA-ES dedicated page) | c564725 | docs/cma.md (105 lines) |
| 2 | Create docs/pso.md (PSO dedicated page) | 09c764f | docs/pso.md (113 lines) |

## What Was Built

### docs/cma.md (105 lines)

Dedicated CMA-ES guide following the nsga3.md section skeleton:
- Description explaining covariance matrix adaptation and Jacobi eigendecomposition (WASM-compatible)
- When to Use bullets covering problem type, variable type, key strength (non-separable landscapes), key weakness (O(n²) limit ~40 dims)
- Quick Reference table with all 11 CmaConfiguration fields and verified defaults
- RestartStrategy documentation with both Ipop and Bipop variants and their real fields
- Complete Example using `default_for_dim`, `with_sigma0`, `with_restart_strategy(RestartStrategy::Ipop {...})`, and `with_fitness_cache` (correct spelling)
- Configuration Tips covering sigma0 heuristic (1/5 to 1/3 range), auto-lambda, IPOP restarts
- Comparison table vs PSO (mechanism / hyperparameters / dimensionality / landscape)
- References: Hansen & Ostermeier (2001), Auger & Hansen (2005)
- See Also linking engines.md, pso.md, cma_es_rastrigin.rs example, docs.rs

### docs/pso.md (113 lines)

Dedicated PSO guide following the nsga3.md section skeleton:
- Description explaining velocity update (inertia + cognitive + social terms)
- When to Use bullets covering continuous optimization, few hyperparameters, higher-dimensional than CMA-ES
- Quick Reference table with all 9 PsoConfiguration fields and verified defaults
- Inertia Strategies subsection: Constant(f64) and LinearDecay { w_start, w_end } only
- Topologies subsection: Global and Ring { neighborhood_size } only
- Complete Example using D-05 prescribed snippet: LinearDecay + Ring + Clerc c1/c2=1.49445 + `with_fitness_cache_size`
- Configuration Tips covering Global vs Ring topology tradeoffs and Clerc's constriction
- Comparison table vs CMA-ES (mechanism / hyperparameters / dimensionality / correlation / convergence)
- References: Kennedy & Eberhart (1995), Shi & Eberhart (1998)
- See Also linking engines.md, cma.md, pso_rastrigin.rs example, docs.rs

## Verification Results

- `docs/cma.md` exists: yes (105 lines, >= 90)
- `docs/pso.md` exists: yes (113 lines, >= 90)
- Both pages contain all required sections (## Description, ## When to Use, ## Quick Reference, ## Complete Example, ## See Also): confirmed
- `grep -cE "RandomRange|VonNeumann" docs/cma.md docs/pso.md`: 0 for both
- `grep -c "with_fitness_cache_size" docs/cma.md`: 0 (uses `with_fitness_cache` — correct)
- `grep -c "default_for_dim" docs/cma.md`: present
- `grep -c "RestartStrategy::Ipop" docs/cma.md`: present
- `grep -c "PsoInertia::LinearDecay" docs/pso.md`: present
- `grep -c "PsoTopology::Ring" docs/pso.md`: present
- `grep -c "cma_es_rastrigin" docs/cma.md`: present
- `grep -c "pso_rastrigin" docs/pso.md`: present
- `cargo doc --no-deps 2>&1 | grep -c warning`: **0** (zero rustdoc warnings)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — both pages are complete with all required sections and links.

## Threat Flags

None — documentation-only changes; no new network endpoints, auth paths, or schema changes.

## Self-Check: PASSED

- `docs/cma.md` exists: FOUND
- `docs/pso.md` exists: FOUND
- Commit c564725 exists: FOUND (docs(80-01): add CMA-ES dedicated documentation page)
- Commit 09c764f exists: FOUND (docs(80-01): add PSO dedicated documentation page)
- cargo doc --no-deps: 0 warnings
