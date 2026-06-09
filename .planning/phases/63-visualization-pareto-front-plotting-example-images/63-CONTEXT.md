# Phase 63: Visualization — Pareto Front Plotting & Example Images - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 63 adds `plot_pareto_front_2d` and `plot_pareto_front_3d` to the existing `visualization` module, adds `--plot` flags to all 5 multi-objective examples plus `rastrigin.rs`, generates and commits 6 PNG images to `docs/images/`, and links them from the `Visualization` section of `README.md`. The `visualization` feature flag and `plotters` dependency already exist; `plot_fitness`, `plot_diversity`, and `plot_histogram` already work. This phase adds Pareto-specific charts and the "show, don't tell" docs assets.

**Out of scope:** 3D interactive/WebGL rendering, accepting axis labels as function arguments, single-objective example --plot for all examples (only rastrigin.rs), adding --plot to CmaEngine or IslandGa examples, custom color palettes, dominance-filtering inside the plot function.

</domain>

<decisions>
## Implementation Decisions

### plot_pareto_front API

- **D-01:** Two separate functions, not one dispatching on arity:
  - `plot_pareto_front_2d(points: &[(f64, f64)], path: &str) -> Result<(), VisualizationError>`
  - `plot_pareto_front_3d(points: &[(f64, f64, f64)], path: &str) -> Result<(), VisualizationError>`
  Both live in `src/observe/visualization/mod.rs` alongside the existing `plot_fitness` / `plot_diversity` / `plot_histogram`.

- **D-02:** First argument takes extracted fitness coordinates, not generic chromosome types. The caller is responsible for extracting `fitness_values` from the NSGA-II population (e.g., `population.iter().map(|c| (c.fitness_values()[0], c.fitness_values()[1])).collect()`). This keeps the function dependency-free (no `VectorFitness` bound in the visualization module).

- **D-03:** Functions plot only the points passed in — caller pre-filters the Pareto front. No dominance checking inside the visualization module.

- **D-04:** Axis labels are generic: `f1`, `f2`, `f3`. No label argument. Consistent with `plot_fitness` which doesn't expose axis labels.

- **D-05:** Add `plot_true_fitness_calls(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError>` — plots `true_fitness_calls: Option<u64>` from `GenerationStats` (Phase 62 addition). Renders a line chart of `Some(n)` values per generation; skips generations where the value is `None`. Lives in the same module.

### 3-Objective Rendering

- **D-06:** `plot_pareto_front_3d` renders a **three-panel image** (three side-by-side 2D scatter charts): f1×f2, f1×f3, f2×f3. Image dimensions: 1200×400 (landscape, 3 equal panels). Each panel is a scatter chart with labeled axes (`f1`/`f2`/`f3`). This is standard practice in multi-objective optimization papers and avoids any projection math.

### Examples — --plot Flag

- **D-07:** All 5 multi-objective examples get `--plot`:
  - `nsga2_zdt1.rs` → `docs/images/nsga2_zdt1.png` (2-obj, uses `plot_pareto_front_2d`)
  - `spea2_zdt1.rs` → `docs/images/spea2_zdt1.png` (2-obj)
  - `sms_emoa_zdt1.rs` → `docs/images/sms_emoa_zdt1.png` (2-obj)
  - `ibea_zdt1.rs` → `docs/images/ibea_zdt1.png` (2-obj)
  - `nsga3_dtlz2.rs` → `docs/images/nsga3_dtlz2.png` (3-obj, uses `plot_pareto_front_3d`)

- **D-08:** One single-objective example gets `--plot`:
  - `rastrigin.rs` → `docs/images/rastrigin.png` (uses existing `plot_fitness`)

- **D-09:** `--plot` is parsed with `std::env::args().any(|a| a == "--plot")` — no `clap` dependency. Consistent with `surrogate_rastrigin.rs` which uses `std::env::var` for configuration.

- **D-10:** Each `--plot` example creates `docs/images/` if it doesn't exist (`std::fs::create_dir_all("docs/images")`), then writes to `docs/images/[example_name].png`. Running from the repo root (standard `cargo run --example`) means the path resolves correctly.

- **D-11:** The `--plot` block in each example is gated with `#[cfg(feature = "visualization")]`:
  ```rust
  #[cfg(feature = "visualization")]
  if std::env::args().any(|a| a == "--plot") {
      // extract pareto points, call plot_pareto_front_2d / plot_pareto_front_3d
  }
  ```
  Without the `visualization` feature, `--plot` silently does nothing (or prints a note).

### README Image Links

- **D-12:** All 6 images are linked from the existing `### Visualization` section in `README.md` under a new sub-section `#### Multi-Objective Pareto Fronts`. A small gallery of `![alt](docs/images/name.png)` tags, one per algorithm, with a one-line caption per image. `rastrigin.png` goes under a `#### Single-Objective Fitness Progress` sub-section.

### Claude's Discretion

- Exact scatter plot marker style (circles vs dots, size), background color, grid lines — follow the existing module's aesthetic (white background, minimal grid, plotters defaults)
- Whether `plot_true_fitness_calls` filters out `None` silently or returns `InsufficientData` if all values are `None`
- Whether the three panels in `plot_pareto_front_3d` share a margin or have a small gap between them
- Caption text for README image alt attributes

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Visualization Module (extend, don't replace)
- `src/observe/visualization/mod.rs` — ALL existing code. New functions must follow the same structure: `draw_*_chart<DB>` private helper + public `plot_*` dispatcher. Same `VisualizationError` enum, same PNG/SVG extension dispatch pattern.
- `src/lib.rs` lines containing `#[cfg(feature = "visualization")]` and `pub mod visualization` — re-export pattern; any new public items must be re-exported from crate root the same way.

### GenerationStats (true_fitness_calls)
- `src/stats.rs` — `GenerationStats` struct. `true_fitness_calls: Option<u64>` field added in Phase 62 (line ~74). `plot_true_fitness_calls` reads this field.

### VectorFitness (how to extract Pareto coordinates)
- `src/traits/` — `VectorFitness` trait. Multi-objective chromosomes implement `fitness_values() -> &[f64]`. Caller extracts `(f1, f2)` tuples before calling `plot_pareto_front_2d`.

### Existing Multi-Objective Examples (add --plot to each)
- `examples/nsga2_zdt1.rs` — NSGA-II 2-obj ZDT1. Population has `fitness_values: Vec<f64>` with 2 entries per individual.
- `examples/spea2_zdt1.rs` — SPEA2 2-obj ZDT1.
- `examples/sms_emoa_zdt1.rs` — SMS-EMOA 2-obj ZDT1.
- `examples/ibea_zdt1.rs` — IBEA 2-obj ZDT1.
- `examples/nsga3_dtlz2.rs` — NSGA-III 3-obj DTLZ2. Population has 3 fitness values per individual.
- `examples/rastrigin.rs` — single-obj GA. Uses existing `plot_fitness(&[GenerationStats], path)`.

### Feature Flag
- `Cargo.toml` — `visualization = ["dep:plotters"]` and `plotters = { version = "0.3.7", ... }` already present. No changes to Cargo.toml needed unless new plotters features are required for scatter plots (check if `scatter` or `point_series` features are included).

### README (extend Visualization section)
- `README.md` — existing `### Visualization` section already describes `plot_fitness`, `plot_diversity`. New sub-sections `#### Multi-Objective Pareto Fronts` and `#### Single-Objective Fitness Progress` go inside it.

### WASM Compatibility Pattern
- `src/engines/ga.rs` — `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` rayon gates for reference. The visualization module must not use `par_iter` or `std::time::Instant`. `plotters` supports `wasm32-unknown-unknown` with `svg_backend` (not bitmap).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `draw_fitness_chart<DB>` / `draw_diversity_chart<DB>` private helpers in `src/observe/visualization/mod.rs` — exact pattern to replicate for `draw_pareto_2d_chart<DB>`, `draw_pareto_3d_chart<DB>`, and `draw_true_fitness_calls_chart<DB>`.
- `VisualizationError` enum — already has all needed variants (`DrawingError`, `IoError`, `UnsupportedFormat`, `InsufficientData`). No new variants expected.
- PNG/SVG dispatch block (match on extension) — copy verbatim into new functions.

### Established Patterns
- `#[cfg(feature = "visualization")]` gate on the entire `mod.rs` (it's only compiled when the feature is on). New functions inherit this automatically.
- `plotters::prelude::*` import already in scope. New functions can use `ScatterSeries` or individual `Circle` elements for scatter plots.
- `compute_y_range` helper pattern — write a `compute_pareto_range` equivalent for x/y axis bounds.
- `root.fill(&WHITE)` + `root.present()` frame wrapping every chart — mandatory for PNG flush.

### Integration Points
- `docs/images/` — new directory, created by examples at first `--plot` run. Must be `.gitignore`-exempt (add to git, not ignored).
- `README.md` `### Visualization` section — extend with two sub-sections; don't restructure existing content.
- `Cargo.toml` `[[example]]` entries — check if spea2/sms_emoa/ibea/nsga3 examples already have entries; `nsga2_zdt1` and `surrogate_rastrigin` have explicit entries as a precedent.

</code_context>

<specifics>
## Specific Ideas

- **Three-panel layout for 3-obj:** `root.split_evenly((1, 3))` in plotters to get three equal `DrawingArea` regions for f1×f2, f1×f3, f2×f3. Image size 1200×400 (landscape).
- **Scatter points:** use `Circle` elements with radius 3 in `BLUE` — consistent with minimal aesthetic of existing charts.
- **`plot_true_fitness_calls`:** collect only generations where `true_fitness_calls.is_some()`, plot as a `LineSeries` in `MAGENTA` or another distinct color not used by `plot_fitness`. Y-axis label: "True fitness calls". If all values are `None`, return `VisualizationError::InsufficientData`.
- **`--plot` gate note in example:** add a `// requires --features visualization` comment alongside the `#[cfg(feature = "visualization")]` block for discoverability.
- **ROADMAP success criterion check:** after generating `docs/images/nsga2_zdt1.png` via `cargo run --example nsga2_zdt1 --features visualization -- --plot`, assert the file exists in the CI-equivalent test (a `cargo run` in a test or doc-verified step).

</specifics>

<deferred>
## Deferred Ideas

- **--plot for CMA-ES / island model examples** — These use `Ga` or `CmaEngine` and could demo `plot_fitness`. Deferred to Phase 64 or later.
- **Optional axis label arguments** — `plot_pareto_front_2d(points, labels: Option<[&str; 2]>, path)` for user-named objectives. User preferred generic labels for now; this is a future ergonomics enhancement.
- **Color gradient for f3 in 3-obj** — f1×f2 with f3 encoded as color. More visually rich alternative to three-panel. Deferred.
- **True fitness calls chart in multi-obj observer** — NSGA-II / SPEA2 don't expose `true_fitness_calls` yet (Phase 62 only added it to `Ga`). A future surrogate-for-multi-obj phase would enable this.

</deferred>

---

*Phase: 63-visualization-pareto-front-plotting-example-images*
*Context gathered: 2026-06-09*
