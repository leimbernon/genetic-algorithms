# Phase 82: Per-Engine Convergence Integration Tests (Issue #284) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-22
**Phase:** 82-per-engine-convergence-integration-tests-issue-284
**Areas discussed:** Benchmark function choice, Convergence thresholds, Budget parameters, CMA restart testing

---

## Benchmark function choice

### DeEngine

| Option | Description | Selected |
|--------|-------------|----------|
| Sphere (Recommended) | Matches existing DE tests. Simpler, faster, still validates convergence. | ✓ |
| Rastrigin | Harder multimodal test. Would prove DE escapes local optima. | |
| Both | Two separate convergence tests — Sphere for basic, Rastrigin for multimodal. | |

**User's choice:** Sphere (Recommended)
**Notes:** Consistent with existing `test_de.rs` helper functions.

### CellularEngine

| Option | Description | Selected |
|--------|-------------|----------|
| Sphere (Recommended) | Consistent with other engines. Cellular GA's spatial structure already tested in unit tests. | ✓ |
| Rastrigin | Would test if cellular structure helps escape local optima. Harder convergence target. | |
| Both | Two tests — Sphere for basic, Rastrigin for multimodal. | |

**User's choice:** Sphere (Recommended)
**Notes:** Consistent with other engines.

### AlpsEngine

| Option | Description | Selected |
|--------|-------------|----------|
| Sphere (Recommended) | Consistent with existing tests. ALPS age-layering is the differentiator, not multimodal escape. | ✓ |
| Rastrigin | ALPS age layers might help escape local optima on Rastrigin. More rigorous test. | |
| Both | Two tests — Sphere for basic, Rastrigin for multimodal. | |

**User's choice:** Sphere (Recommended)
**Notes:** Consistent with other engines.

### PsoEngine

| Option | Description | Selected |
|--------|-------------|----------|
| Sphere (Recommended) | Matches existing PSO tests. PSO convergence on Sphere is well-studied. | ✓ |
| Rastrigin | PSO can struggle with multimodal — would be a harder convergence test. | |
| Both | Two tests — Sphere for basic, Rastrigin for multimodal. | |

**User's choice:** Sphere (Recommended)
**Notes:** Consistent with other engines.

### ScatterEngine

| Option | Description | Selected |
|--------|-------------|----------|
| Sphere (Recommended) | Same as other engines. Scatter already uses Sphere in existing test_scatter.rs. Consistent. | ✓ |
| Rastrigin | Harder test for Scatter's diversification + reference set combination. | |
| Ackley | Another common continuous benchmark. More complex landscape than Sphere. | |

**User's choice:** Sphere (Recommended)
**Notes:** All engines use Sphere for consistency.

---

## Convergence thresholds

| Option | Description | Selected |
|--------|-------------|----------|
| Uniform threshold (Recommended) | All engines use same tolerance (e.g., best_fitness < 1.0 on 5-dim sphere). Simple, consistent. | ✓ |
| Engine-specific | Each engine gets its own threshold based on its dynamics. More accurate but harder to maintain. | |
| Tiered | Easy ( < 10.0), Medium ( < 1.0), Hard ( < 0.1) — each engine targets a tier. | |

**User's choice:** Uniform threshold (Recommended)
**Notes:** Simplicity and consistency preferred.

### Threshold value

| Option | Description | Selected |
|--------|-------------|----------|
| < 1.0 (Recommended) | Matches existing CMA/PSO tests. Tight enough to prove convergence, loose enough for stochastic engines. | ✓ |
| < 0.1 | Very tight. May require more generations for some engines. Proves strong convergence. | |
| < 5.0 | Loose. Matches existing DE test. Quick to converge but less rigorous. | |
| < 0.01 | Extremely tight. May fail for some engines without large budgets. | |

**User's choice:** < 1.0 (Recommended)
**Notes:** Matches existing CMA/PSO test patterns.

---

## Budget parameters

### Dimension

| Option | Description | Selected |
|--------|-------------|----------|
| 5 dimensions (Recommended) | Matches existing tests. Fast, still meaningful. 5 variables = 5 search dimensions. | ✓ |
| 10 dimensions | Harder. More representative of real problems. Takes longer. | |
| 3 dimensions | Easiest. Very fast convergence. May not stress engines enough. | |

**User's choice:** 5 dimensions (Recommended)
**Notes:** Matches existing tests.

### Population size

| Option | Description | Selected |
|--------|-------------|----------|
| 30 (Recommended) | Matches existing DE/Scatter tests. Good balance of speed and convergence ability. | ✓ |
| 50 | Larger population. More diversity, better convergence, but slower tests. | |
| 20 | Smaller population. Faster tests, but may struggle to converge. | |

**User's choice:** 30 (Recommended)
**Notes:** Matches existing tests.

### Max generations/iterations

| Option | Description | Selected |
|--------|-------------|----------|
| 300 (Recommended) | Matches existing DE test. Generous budget for convergence. Tests run in < 1s. | ✓ |
| 500 | Extra generous. Ensures convergence even for slower engines. Slightly slower tests. | |
| 100 | Tight budget. Tests fail fast if engine doesn't converge. May be too strict. | |

**User's choice:** 300 (Recommended)
**Notes:** Matches existing DE test.

---

## CMA restart testing

| Option | Description | Selected |
|--------|-------------|----------|
| Separate restart test (Recommended) | One test for basic convergence (no restart), one test specifically for restart path. Clear separation of concerns. | ✓ |
| Single test with restart | One test that configures IPOP/BIPOP and asserts convergence. Simpler but less targeted. | |
| Restart as secondary assertion | Main convergence test checks fitness; if restart fires, also assert it happened. Combined check. | |

**User's choice:** Separate restart test (Recommended)
**Notes:** Clear separation of concerns.

### Restart strategy

| Option | Description | Selected |
|--------|-------------|----------|
| IPOP only (Recommended) | Simpler restart strategy. Doubles population on stagnation. Well-tested in existing CMA tests. | ✓ |
| BIPOP only | More complex. Alternates between small and large populations. More thorough test. | |
| Both IPOP and BIPOP | Two separate restart tests. Most thorough but more test code. | |

**User's choice:** IPOP only (Recommended)
**Notes:** Simpler, well-tested.

---

## Agent's Discretion

- Exact test function names and internal structure
- Whether to extract shared `sphere` helper to a common module (currently duplicated)
- Specific assertion messages and error context

## Deferred Ideas

None — discussion stayed within phase scope.
