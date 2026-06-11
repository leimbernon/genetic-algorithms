---
plan: 30-03
phase: 30-observer-wiring-de-benchmark
status: complete
completed: 2026-05-02
---

# Phase 30 Plan 03: DE-vs-GA Convergence Benchmark Summary

Added `bench_de_vs_ga` criterion group to `benches/de.rs` comparing Differential Evolution and standard GA wall-time on sphere(5D) with identical max_generations=100.

## What Was Built

Extended `benches/de.rs` with a new `bench_de_vs_ga` function containing two benchmark sub-functions:
- `de_sphere_5d` — runs `DeEngine` with Rand1 strategy, pop=30, max_gen=100, Minimization on sphere(5D)
- `ga_sphere_5d` — runs `Ga<RangeChromosome<f64>>` with Tournament+Uniform+Gaussian+Fitness operators, pop=30, max_gen=100, Minimization on sphere(5D)

Both use `make_pop(30, 5)` for consistent population initialization. The GA benchmark uses `with_population()` to skip the initialization function (same pattern as `benches/ga_run.rs`), avoiding a borrow issue with `ga.run()` returning a reference into the owned `ga`.

## Key Files Modified

- `benches/de.rs` — added `bench_de_vs_ga` function with `de_sphere_5d` and `ga_sphere_5d` benchmarks, updated `criterion_group!` to include both groups

## Deviations from Plan

**1. [Rule 1 - Bug] Used `with_population()` instead of `with_initialization_fn()`**
- **Found during:** Task 1
- **Issue:** The plan template used `with_initialization_fn(|n, _, _| ...)` which returns `Vec<Gene>` for one chromosome, but the `ga.run()` method returns `&Population<U>` — a reference tied to the `ga` local — making it impossible to return from the `b.iter()` closure.
- **Fix:** Used `with_population(Population::new(make_pop(30, 5)))` instead, consistent with `benches/ga_run.rs` pattern. Also changed `ga.run().expect(...)` to `let _ = ga.run().expect(...)` to discard the reference.
- **Files modified:** `benches/de.rs`
- **Commit:** a8573c6

## Self-Check

- [x] `cargo bench --bench de -- --test` exits 0
- [x] `benches/de.rs` contains `fn bench_de_vs_ga(c: &mut Criterion)`
- [x] `criterion_group` includes `bench_de_vs_ga`
- [x] Both `de_sphere_5d` and `ga_sphere_5d` benchmark functions present

## Self-Check: PASSED
