# Phase 30: Observer Wiring & DE Benchmark - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-27
**Phase:** 30-observer-wiring-de-benchmark
**Areas discussed:** Hook depth, DE-vs-GA benchmark, ALPS layer stats

---

## Hook Depth

| Option | Description | Selected |
|--------|-------------|----------|
| 5 required only | Ship exactly what OBS-01–04 mandate. Operator-timing hooks can be added later per-engine if needed. | ✓ |
| Add where semantic match exists | DE: on_mutation_complete for trial vector generation. Cellular: on_selection_complete for local tournament. Skip nonsensical mappings. | |
| Full 12 on all engines | Wire all available hooks, even if some have no natural operator counterpart. | |

**User's choice:** 5 required only (Recommended)
**Notes:** None — straightforward choice, operator-timing hooks deferred to a later phase.

---

## DE-vs-GA Benchmark

**File placement:**

| Option | Description | Selected |
|--------|-------------|----------|
| Extend benches/de.rs | Add GA run alongside existing DE benchmarks. No new file or Cargo.toml entry needed. | ✓ |
| New benches/convergence.rs | Separate file for cross-algorithm comparisons. Needs new [[bench]] entry. | |

**User's choice:** Extend benches/de.rs

**Comparison format:**

| Option | Description | Selected |
|--------|-------------|----------|
| Same max_generations | Both DE and GA run for same number of generations. Easy direct comparison in criterion output. | ✓ |
| Same fitness evaluations | Normalize by total evaluations (pop_size × generations). More rigorous but more complex. | |

**User's choice:** Same max_generations
**Notes:** Use sphere(5D) — same problem as existing DE benchmarks. sample_size(10) to match convention.

---

## ALPS Layer Stats

**GenerationStats representation:**

| Option | Description | Selected |
|--------|-------------|----------|
| Merged across all layers | Flatten all layer populations, compute a single GenerationStats. Matches single-population engine contract. | ✓ |
| Best (youngest) layer only | Emit stats for layer 0 only. Simpler but loses visibility into elder layers. | |
| Per-layer stats | Fire on_generation_end once per layer — N times per generation, unusual contract. | |

**User's choice:** Merged across all layers

**on_new_best scope:**

| Option | Description | Selected |
|--------|-------------|----------|
| Global best only | Fires when global best across all layers improves. Consistent with all other engines. | ✓ |
| Per-layer best | Track best per layer, fire when any layer improves. More granular but diverges from standard contract. | |

**User's choice:** Global best only
**Notes:** Consistency with other engines was the deciding factor for both ALPS decisions.

---

## Claude's Discretion

- `with_observer()` placement: either on the engine struct directly or on the configuration struct — follow whatever pattern is cleanest for each engine's existing API.
- `sample_size(10)` on the DE-vs-GA benchmark group.

## Deferred Ideas

- Operator-timing hooks for new engines — noted for a future phase
- Per-layer observer stats for ALPS — noted; merged chosen for now
