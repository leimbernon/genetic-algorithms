---
phase: 63-visualization-pareto-front-plotting-example-images
plan: "02"
subsystem: examples
tags: [visualization, examples, pareto, multi-objective, single-objective]
dependency_graph:
  requires:
    - plot_pareto_front_2d (63-01)
    - plot_pareto_front_3d (63-01)
    - plot_fitness (63-01)
  provides:
    - nsga2_zdt1 --plot block -> docs/images/nsga2_zdt1.png
    - spea2_zdt1 --plot block -> docs/images/spea2_zdt1.png
    - sms_emoa_zdt1 --plot block -> docs/images/sms_emoa_zdt1.png
    - ibea_zdt1 --plot block -> docs/images/ibea_zdt1.png
    - nsga3_dtlz2 --plot block -> docs/images/nsga3_dtlz2.png
    - rastrigin --plot block -> docs/images/rastrigin.png
  affects:
    - examples/nsga2_zdt1.rs
    - examples/spea2_zdt1.rs
    - examples/sms_emoa_zdt1.rs
    - examples/ibea_zdt1.rs
    - examples/nsga3_dtlz2.rs
    - examples/rastrigin.rs
tech_stack:
  added: []
  patterns:
    - cfg(feature = "visualization") gated --plot block in match arm
    - Arc<Mutex<Vec<GenerationStats>>> accumulator with move closure for rastrigin
    - fully-qualified genetic_algorithms::visualization::* calls (no new use imports)
key_files:
  modified:
    - examples/nsga2_zdt1.rs
    - examples/spea2_zdt1.rs
    - examples/sms_emoa_zdt1.rs
    - examples/ibea_zdt1.rs
    - examples/nsga3_dtlz2.rs
    - examples/rastrigin.rs
decisions:
  - "Use fully-qualified paths for visualization calls — no new use imports in examples to keep changes minimal and additive"
  - "rastrigin stats accumulator uses Arc<Mutex<Vec<GenerationStats>>> declared with cfg gates so the accumulator and clone bindings compile away cleanly when visualization feature is off"
  - "move keyword added unconditionally to rastrigin callback closure — existing closure body captures nothing by reference so the move is safe"
metrics:
  duration: "~5 minutes"
  completed_date: "2026-06-10"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 6
---

# Phase 63 Plan 02: Example --plot Blocks Summary

Added cfg-gated `--plot` blocks to six example files so users can generate Pareto front or fitness-progress PNGs by re-running examples with `--features visualization -- --plot`. All six examples continue to compile and behave identically without the visualization feature.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add --plot blocks to four 2-objective NSGA-II-family examples | 9ce0c29 | nsga2_zdt1.rs, spea2_zdt1.rs, sms_emoa_zdt1.rs, ibea_zdt1.rs |
| 2 | Add --plot block to nsga3_dtlz2 (3-objective) and rastrigin (single-objective with stats accumulator) | f11adf4 | nsga3_dtlz2.rs, rastrigin.rs |

## What Was Built

**Four 2-objective examples** (`nsga2_zdt1.rs`, `spea2_zdt1.rs`, `sms_emoa_zdt1.rs`, `ibea_zdt1.rs`):

Each received an identical additive block appended after the existing per-individual print loop inside the `Ok(mut front)` match arm:

```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features visualization (sms_emoa/ibea: --features "visualization,benchmarks")
    let points: Vec<(f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_2d(
        &points,
        "docs/images/<example>.png",
    )
    .expect("plot failed");
    println!("Pareto front plot saved to docs/images/<example>.png");
}
```

**nsga3_dtlz2.rs** — 3-objective variant using `plot_pareto_front_3d` with `Vec<(f64,f64,f64)>` points. Block placed after the 10-individual print loop in the `Ok(mut front)` arm.

**rastrigin.rs** — Single-objective with per-generation stats accumulation:
- Two `#[cfg(feature = "visualization")]`-gated declarations before `run_with_callback`: `plot_stats: Arc<Mutex<Vec<GenerationStats>>>` and `plot_stats_clone`
- `move` keyword added to the callback closure (safe — existing closure body captured nothing by reference)
- `#[cfg(feature = "visualization")] plot_stats_clone.lock().unwrap().push(_stats.clone())` appended inside callback before `ControlFlow::Continue(())`
- `--plot` block in the `Ok(population)` arm uses `plot_stats.lock().unwrap()` to pass accumulated stats to `plot_fitness`

## Deviations from Plan

None — plan executed exactly as written.

## Threat Model Coverage

| Threat ID | Mitigation Applied |
|-----------|--------------------|
| T-63-05 | `.expect("failed to create docs/images")` and `.expect("plot failed")` produce actionable panic messages — accepted per threat register |
| T-63-06 | All writes use relative path `docs/images/<name>.png` — accepted per threat register |

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build --example nsga2_zdt1 --features visualization` | PASS |
| `cargo build --example spea2_zdt1 --features visualization` | PASS |
| `cargo build --example sms_emoa_zdt1 --features "visualization,benchmarks"` | PASS |
| `cargo build --example ibea_zdt1 --features "visualization,benchmarks"` | PASS |
| `cargo build --example nsga3_dtlz2 --features visualization` | PASS |
| `cargo build --example rastrigin --features visualization` | PASS |
| All six examples build without --features visualization | PASS |
| `cargo clippy --example nsga3_dtlz2 --features visualization -- -D warnings` | PASS |
| `cargo clippy --example rastrigin --features visualization -- -D warnings` | PASS |

## Known Stubs

None. All six --plot blocks are fully wired to the visualization API from Plan 01.

## Self-Check: PASSED

- examples/nsga2_zdt1.rs contains `docs/images/nsga2_zdt1.png` and `plot_pareto_front_2d`
- examples/spea2_zdt1.rs contains `docs/images/spea2_zdt1.png` and `plot_pareto_front_2d`
- examples/sms_emoa_zdt1.rs contains `docs/images/sms_emoa_zdt1.png` and `plot_pareto_front_2d`
- examples/ibea_zdt1.rs contains `docs/images/ibea_zdt1.png` and `plot_pareto_front_2d`
- examples/nsga3_dtlz2.rs contains `docs/images/nsga3_dtlz2.png` and `plot_pareto_front_3d`
- examples/rastrigin.rs contains `docs/images/rastrigin.png` and `plot_fitness`
- Commits 9ce0c29 (Task 1) and f11adf4 (Task 2) exist in git log
