# Phase 37: SPEA2 — Strength Pareto Evolutionary Algorithm 2 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 37-spea2-strength-pareto-evolutionary-algorithm
**Areas discussed:** Archive sizing, Density k & truncation, Observer hooks, Example benchmark

---

## Archive Sizing

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable with population default | `.with_archive_size(N)`, default = population_size, validate rejects > population or 0 | ✓ |
| Always equal to population | Archive size always equals population — no configuration, exact SPEA2 paper match | |

**User's choice:** Configurable with population default (Recommended)
**Notes:** Standard SPEA2 sets archive = population. User wants flexibility to experiment with smaller archives.

---

## Density k Parameter

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-calculate k | k = floor(sqrt(N_pop + N_archive)), always. Zero config, matches SPEA2 paper. | ✓ |
| Auto with override | Auto-calculate by default, allow `.with_density_k(N)` override | |

**User's choice:** Auto-calculate k (Recommended)
**Notes:** k = sqrt(pop + archive) is the canonical formula — no user demand for customization.

---

## Truncation Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Exact SPEA2 truncation only | Iterative nearest-neighbour removal, always. No alternatives. | ✓ |
| Configurable strategy enum | `Spea2Truncation` enum with NearestNeighbor, Random, etc. | |

**User's choice:** Exact SPEA2 truncation only (Recommended)
**Notes:** Matches prior engines' single-strategy approach (NSGA-II crowding, NSGA-III reference-point association). Proven in literature.

---

## Observer Hooks

| Option | Description | Selected |
|--------|-------------|----------|
| Two hooks: fitness + archive | `on_fitness_assigned` + `on_archive_updated` — mirrors NSGA3/MOEAD 2-hook pattern | ✓ |
| Single generation hook | `on_generation_complete` only — simpler but less granular | |

**User's choice:** Two hooks: fitness + archive (Recommended)
**Notes:** SPEA2 has two distinct phases (fitness assignment, environmental selection) that are natural hook points. Matches existing observer granularity.

---

## Example Benchmark

| Option | Description | Selected |
|--------|-------------|----------|
| ZDT1 | 2-objective, 30 variables — canonical SPEA2 benchmark from Zitzler et al. 2001 | ✓ |
| DTLZ2 | 3-objective, sphere Pareto front — matches nsga3/moead examples | |

**User's choice:** ZDT1 (Recommended)
**Notes:** ZDT1 is *the* benchmark from the original SPEA2 paper. Mirrors `examples/nsga2_zdt1.rs` structure. Provides diversity across examples (NSGA-II/ZDT1, NSGA-III/DLTZ2, MOEA/D/DLTZ2, SPEA2/ZDT1).

---

## Claude's Discretion

- Internal archive management: maintain archive alongside population; copy non-dominated after fitness, truncate if over capacity
- Binary tournament selection from archive (standard SPEA2)
- SPEA2 fitness: combine pop+archive, compute strength → raw fitness → density → final fitness
- WASM cfg-gating throughout (mandatory per CLAUDE.md)
- `run()` returns `Result<ParetoFront<U>, GaError>` (uniform multi-objective API)
- Engine at `src/engines/spea2/`, `#[path]` re-export in `lib.rs`

## Deferred Ideas

- Archive size adaptation (dynamic sizing)
- Alternative truncation strategies
- `AllObserver<U>` update to include `Spea2Observer<U>`
- Alternative density estimators
- DTLZ2 or other 3-objective example
