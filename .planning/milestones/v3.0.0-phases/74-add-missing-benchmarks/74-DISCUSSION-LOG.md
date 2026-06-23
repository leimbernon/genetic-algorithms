# Phase 74: Add Missing Engine and Feature Benchmarks - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-18
**Phase:** 74-add-missing-benchmarks
**Areas discussed:** AOS/Surrogate/Batch structure, GP benchmark design, Problem selection for PSO/CMA-ES/EDA

---

## AOS / Surrogate / Batch Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Separate file per feature | benches/aos.rs, benches/surrogate.rs, benches/batch_fitness.rs — mirrors metrics_observer.rs | ✓ |
| Single combined file | benches/features.rs with groups for each | |
| Skip feature benchmarks | Only add engine benchmarks (PSO, CMA-ES, EDA, GP) | |

**User's choice:** Separate file per feature

---

### AOS benchmark focus

| Option | Description | Selected |
|--------|-------------|----------|
| AOS on vs off overhead | Two groups: GA with AOS enabled vs without, same problem/pop | ✓ |
| Per-strategy comparison | Bench each AOS strategy against each other | |
| You decide | Leave to implementer's discretion | |

**User's choice:** AOS on vs off overhead

---

### Surrogate benchmark focus

| Option | Description | Selected |
|--------|-------------|----------|
| Surrogate-assisted vs full eval throughput | Surrogate GA vs plain GA — measures speedup potential | ✓ |
| Surrogate accuracy vs evaluation budget | Vary fraction of real evals used | |
| You decide | Leave to implementer | |

**User's choice:** Surrogate-assisted vs full eval throughput

---

### Batch fitness benchmark focus

| Option | Description | Selected |
|--------|-------------|----------|
| Batch vs individual evaluation throughput | Batch evaluator vs individual FitnessFnWrapper — measures parallelism gain | ✓ |
| Batch size scaling | Vary batch size (10, 100, 1000 chromosomes) | |
| You decide | Leave to implementer | |

**User's choice:** Batch vs individual evaluation throughput

---

## GP Benchmark Design

| Option | Description | Selected |
|--------|-------------|----------|
| Symbolic regression | Evolve tree approximating f(x) = x^2 + x + 1 — standard GP benchmark | ✓ |
| Boolean formula / truth table | Evolve boolean expression matching target truth table | |
| You decide | Leave to implementer's discretion | |

**User's choice:** Symbolic regression

---

### GP benchmark axis

| Option | Description | Selected |
|--------|-------------|----------|
| Population size (pop_50, pop_200, pop_500) | Varies number of individuals — natural GP scaling axis | ✓ |
| Max tree depth (depth_3, depth_5, depth_8) | Varies tree complexity cap | |
| Both pop size and depth | Cross-product of both axes | |

**User's choice:** Population size (pop_50, pop_200, pop_500)

---

## Problem Selection for PSO / CMA-ES / EDA

| Option | Description | Selected |
|--------|-------------|----------|
| Same as existing: sphere + rastrigin | Both problems, enables cross-engine comparison | ✓ |
| Sphere only | Simpler, lower noise in timings | |
| Rastrigin only | More representative, less comparable | |

**User's choice:** Same as existing: sphere + rastrigin

---

### Dimension groups

| Option | Description | Selected |
|--------|-------------|----------|
| dims_10, dims_30, dims_100 | Standard continuous optimization dimensions, CI-friendly | ✓ |
| dims_10, dims_100, dims_1000 | Wider range; 1000D may be too slow for CI | |
| Single configuration | One fixed dimension (e.g. 30D) | |

**User's choice:** dims_10, dims_30, dims_100

---

## Claude's Discretion

- Exact population size and generation count per engine (keep CI-friendly)
- Whether new feature bench entries need `required-features` in Cargo.toml
- Exact ordering of new `[[bench]]` entries in Cargo.toml
- Exact `Cargo.toml` `[[bench]]` entry ordering

## Deferred Ideas

None — discussion stayed within phase scope.
