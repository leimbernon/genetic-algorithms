# Phase 42: Warm Starting & Population Seeding - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 42-Warm Starting & Population Seeding
**Areas discussed:** Seeding API model, Checkpoint resumption API, Generation counter on resume, Fitness trust for seeds

---

## Seeding API Model

| Option | Description | Selected |
|--------|-------------|----------|
| `with_seeds(Vec<U>)` | Builder method that takes known solutions and fills remaining slots randomly via the existing initialization_fn | ✓ |
| `SeedConfig` struct | A config struct with explicit control over seeds and fill ratio | |
| Extend `with_population()` | Extend existing method to support seeds plus random fill | |

**User's choice:** `with_seeds(Vec<U>)`
**Notes:** User also chose that random fill should deduplicate against seed DNA (genotypic uniqueness check).

---

## Checkpoint Resumption API

| Option | Description | Selected |
|--------|-------------|----------|
| `resume_from()` method on Ga | Separate Ga method that loads checkpoint and skips initialization | |
| `with_checkpoint()` on the builder | Builder method; checkpoint fields override builder defaults | ✓ |

**User's choice:** `with_checkpoint()` on the builder
**Notes:** User chose hybrid config override — operator config (selection, crossover, mutation) from builder wins; state fields (population, generation, stats) from checkpoint.

---

## Generation Counter on Resume

| Option | Description | Selected |
|--------|-------------|----------|
| Absolute (checkpoint gen + offset) | Loop from checkpoint.generation; max_generations = "run N more" | ✓ |
| Reset to 0 | Simple — treat resumed run as generation 0 | |

**User's choice:** Absolute (checkpoint gen + offset)

---

## Fitness Trust for Seeds

| Option | Description | Selected |
|--------|-------------|----------|
| Re-evaluate all (simpler) | Seeds treated same as random fill — all get evaluated | |
| Trust seed fitness (skip re-eval) | Seeds skip re-evaluation; user provides correct fitness | ✓ |

**User's choice:** Trust seed fitness (skip re-eval)

---

## Claude's Discretion

- Internal validation details (seeds exceeding population_size, mutual exclusivity of seeds and checkpoint)
- Seed injection timing (seeds first, fill second)
- Dedup algorithm (genotypic DNA comparison, same approach as Hall of Fame)
- Checkpoint loading timing (build-time vs run-time)

## Deferred Ideas

None — discussion stayed within phase scope.
