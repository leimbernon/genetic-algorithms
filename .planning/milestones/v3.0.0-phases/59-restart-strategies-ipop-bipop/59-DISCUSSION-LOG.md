# Phase 59: Restart Strategies — IPOP / BIPOP - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-05
**Phase:** 59-restart-strategies-ipop-bipop
**Areas discussed:** Stagnation / restart trigger, RestartStrategy API shape, on_restart hook payload, CmaResult extension

---

## Stagnation / restart trigger

| Option | Description | Selected |
|--------|-------------|----------|
| No improvement for N generations | Restart when best fitness hasn't improved for N consecutive generations. `stagnation_threshold: usize` in the restart config. | ✓ |
| Sigma collapse only | Restart when sigma drops below a threshold (e.g., 1e-12). CMA-ES-specific convergence detection. | |
| Both (no improvement OR sigma collapse) | Either condition triggers a restart. | |

**User's choice:** No improvement for N generations

| Option | Description | Selected |
|--------|-------------|----------|
| In a RestartStrategy enum variant | `RestartStrategy::Ipop { population_scale, stagnation_threshold }`. Self-contained. | ✓ |
| As fields on CmaConfiguration | Add fields directly to existing config. | |
| As a separate CmaRestartConfig struct | `CmaConfiguration.restart: Option<CmaRestartConfig>` field. | |

**User's choice:** In a RestartStrategy enum variant

| Option | Description | Selected |
|--------|-------------|----------|
| max_restarts in the variant, stop normally when reached | Exit with best individual when max_restarts hit. | ✓ |
| Only bound by max_generations | Run indefinitely until total generations hits max_generations. | |
| You decide | Planner picks. | |

**User's choice:** max_restarts in the variant, stop normally when reached

---

## RestartStrategy API shape

| Option | Description | Selected |
|--------|-------------|----------|
| Alternate strictly every other restart | Odd = large (IPOP-style), even = small (fixed small_population_size). Simple, predictable. | ✓ |
| Budget-based (Hansen 2009 original) | Switch based on total function evaluation budget. Faithful to paper but complex. | |
| You decide | Planner picks. | |

**User's choice:** Alternate strictly every other restart

| Option | Description | Selected |
|--------|-------------|----------|
| Builder method on CmaConfiguration | `CmaConfiguration::with_restart_strategy(RestartStrategy)`. Consistent with cc/cs/c1/cmu pattern. | ✓ |
| Separate restart-aware constructor | `CmaEngine::with_restart(config, restart_strategy, ...)` alternative constructor. | |
| You decide | Planner picks. | |

**User's choice:** Builder method on CmaConfiguration

---

## on_restart hook payload

| Option | Description | Selected |
|--------|-------------|----------|
| RestartEvent struct | Stack-allocated struct with restart_number, population_size_before, population_size_after, generation. | ✓ |
| Flat parameters | `on_restart(restart_number: usize, new_population_size: usize)`. Simpler. | |
| You decide | Planner picks. | |

**User's choice:** RestartEvent struct

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — include RestartKind enum field | `RestartKind::Ipop / BipopLarge / BipopSmall` lets observers distinguish behavior. | ✓ |
| No — just sizes, kind not needed | Simpler, observers infer from population size. | |
| You decide | Planner decides. | |

**User's choice:** Yes — include RestartKind enum field

---

## CmaResult extension

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — add total_restarts: usize | Minimal addition. Backward-compatible. | ✓ |
| Yes — full restart history (Vec<RestartSummary>) | Richer but adds heap allocation. | |
| No — leave CmaResult unchanged | Observer-only restart metadata. | |

**User's choice:** Yes — add total_restarts: usize

| Option | Description | Selected |
|--------|-------------|----------|
| best is global best across ALL restarts | CmaResult.best holds best individual from the entire multi-restart search. | ✓ |
| best is from the final restart only | Simpler but can lose good solutions. | |
| You decide | Planner decides. | |

**User's choice:** best is global best across all restarts

---

## Claude's Discretion

- Default `population_scale` value in docs/examples (standard: 2.0 for IPOP)
- Default `stagnation_threshold` suggestion in docs (common: 100 or 10*n)
- Whether `small_population_size = 0` auto-computes to `floor(default_lambda / 5)` or similar
- Internal variable names and struct layout
- Whether `RestartStrategy`, `RestartEvent`, `RestartKind` live in `src/observe/observer/mod.rs` or `src/engines/cma/restart.rs`

## Deferred Ideas

- Sigma-collapse stagnation detection — future extension to RestartStrategy variants
- Budget-based BIPOP alternation (Hansen 2009 original) — strict alternation chosen for this phase
- Per-restart RestartSummary history in CmaResult — deferred to a future phase
- Restart strategies for PSO, EDA, or other engines — out of scope
