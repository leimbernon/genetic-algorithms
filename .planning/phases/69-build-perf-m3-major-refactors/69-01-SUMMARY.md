---
phase: 69-build-perf-m3-major-refactors
plan: "01"
subsystem: bench-harness
tags: [divan, criterion, benchmarks, build-perf]
dependency_graph:
  requires: []
  provides: [divan-bench-harness]
  affects: [Cargo.toml, benches/]
tech_stack:
  added: [divan 0.1.21]
  patterns: [bench-per-fn, args-tuple, with_inputs+bench_values, module-grouping]
key_files:
  created:
    - .planning/intel/bench-harness.md
  modified:
    - Cargo.toml
    - benches/metrics_observer.rs
    - benches/de.rs
    - benches/scatter.rs
    - benches/alps.rs
    - benches/cellular.rs
    - benches/crossover.rs
    - benches/nsga2.rs
    - benches/selection.rs
    - benches/rastrigin.rs
    - benches/survivor.rs
    - benches/mutation.rs
    - benches/ga_run.rs
    - benches/island_ga.rs
    - docs/benchmarks.md
    - CHANGELOG.md
decisions:
  - "bench_values (not bench) is required when using with_inputs — divan API distinction"
  - "Tuple args require Display; use module-level grouping for enum-variant loops"
  - ".planning/ is gitignored; intel files require git add -f"
metrics:
  duration: "~35 minutes"
  completed: "2026-06-16"
  tasks_completed: 2
  files_modified: 16
---

# Phase 69 Plan 01: Criterion → Divan Bench Harness Port Summary

All 13 benchmark files ported from criterion to divan 0.1.21; criterion removed from dev-dependencies.

## Task 1: Add divan + port 9 LOW/MED bench files

| Bench File | Complexity | Commit | Key Pattern |
|---|---|---|---|
| Cargo.toml (divan add) | — | f4cbf10 | add divan alongside criterion |
| benches/metrics_observer.rs | LOW | 7842d8b | simple bencher.bench() |
| benches/de.rs | LOW | 2fd7cb2 | module grouping for enum variants |
| benches/scatter.rs | LOW | e3732a7 | module grouping, sample_count=10 |
| benches/alps.rs | LOW | 19c0752 | 2 modules, sample_count=10 |
| benches/cellular.rs | LOW | 48e543f | 2 modules, sample_count=10 |
| benches/crossover.rs | MED | a995cbd | args=[10,100,1000], no PlotConfig |
| benches/nsga2.rs | MED | c5cb9db | args tuples, no Throughput |
| benches/selection.rs | MED | 9da62a7 | args tuples, no Throughput |
| benches/rastrigin.rs | MED | 44f5044 | with_inputs+bench_values |

## Task 2: Port 4 HIGH bench files, remove criterion, write docs

| Bench File | Complexity | Commit | Key Pattern |
|---|---|---|---|
| benches/survivor.rs | HIGH | 7fb4f64 | with_inputs+bench_values, args |
| benches/mutation.rs | HIGH | 77d89af | 7 mutation fns, with_inputs+bench_values |
| benches/ga_run.rs | HIGH | bcc11ff | args 5-tuple, with_inputs+bench_values |
| benches/island_ga.rs | HIGHEST | b3a273c | args 5-tuple, builder setup fn preserved |
| Cargo.toml (criterion remove) | — | 47f9cf5 | zero criterion references confirmed |
| docs/benchmarks.md + intel + CHANGELOG | — | a250ced | divan invocation examples |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] divan with_inputs API uses bench_values not bench**
- **Found during:** Task 1 (rastrigin.rs port)
- **Issue:** PATTERNS.md showed `.bench(|_b, mut ga| ...)` but divan's actual API for `with_inputs()` is `.bench_values(|mut ga| ...)` — the method name differs
- **Fix:** All `with_inputs()` calls use `bench_values` not `bench`
- **Files modified:** rastrigin.rs, survivor.rs, mutation.rs, ga_run.rs, island_ga.rs

**2. [Rule 1 - Bug] Tuple args require Display — enum variants can't be in args**
- **Found during:** Task 1 (de.rs port)
- **Issue:** PATTERNS.md suggested `args = [("rand1", DeMutationStrategy::Rand1), ...]` but divan requires `args` items to implement `Display`; `(&str, DeMutationStrategy)` does not
- **Fix:** Converted to module-level grouping with one `#[divan::bench(sample_count=10)]` fn per variant — preserves same benchmark coverage
- **Files modified:** de.rs

**3. [Rule 3 - Blocking] Worktree branched from main before feat/64-04-rustdoc-examples commits**
- **Found during:** Task 1 start
- **Issue:** Worktree had 12 bench files; plan requires 13 including rastrigin.rs which was added in feat/64-04-rustdoc-examples (phase 64)
- **Fix:** Merged feat/64-04-rustdoc-examples into worktree branch before starting port — brings rastrigin.rs and correct Cargo.toml into scope
- **Impact:** Non-breaking; merge auto-committed cleanly

**4. [Rule 2 - Missing] .planning is gitignored — intel files need git add -f**
- **Found during:** Task 2 docs commit
- **Issue:** .gitignore excludes `.planning/` so `.planning/intel/bench-harness.md` required `git add -f`
- **Fix:** Used `git add -f .planning/intel/bench-harness.md`

## Revert Plan

- Revert commit a250ced (docs)
- Revert commit 47f9cf5 (criterion removal: restore `criterion = "0.8.2"` in dev-dependencies)
- Revert per-bench commits b3a273c, bcc11ff, 77d89af, 7fb4f64, 44f5044, 9da62a7, c5cb9db, a995cbd, 48e543f, 19c0752, e3732a7, 2fd7cb2, 7842d8b (13 file ports)
- Revert commit f4cbf10 (drop divan from dev-dependencies)

## Self-Check: PASSED

Files verified:
- benches/ count: 13 confirmed (ls benches/*.rs | wc -l = 13)
- zero criterion references: grep -rn criterion benches/ Cargo.toml returned exit 1 (no matches)
- all bench executables: cargo bench --no-run --all-features produced 13 Executable lines
- tests passed: 1661 passed (all-features), 1536 passed (logging-only)
- wasm clean: cargo check --target wasm32-unknown-unknown --lib — 0 errors
- rustdoc clean: cargo doc --no-deps — 0 warnings
- .planning/intel/bench-harness.md: exists, contains Why divan, Canonical patterns, Do Not Reintroduce sections
- CHANGELOG.md: grep -c 'divan' = 1
