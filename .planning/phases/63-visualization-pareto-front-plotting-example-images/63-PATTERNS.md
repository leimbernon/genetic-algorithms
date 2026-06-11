# Phase 63: Visualization — Pareto Front Plotting & Example Images - Pattern Map

**Mapped:** 2026-06-09
**Files analyzed:** 11 new/modified files
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/observe/visualization/mod.rs` (add 3 functions) | utility | file-I/O | `src/observe/visualization/mod.rs` (existing functions) | exact |
| `Cargo.toml` (add `point_series` feature) | config | — | `Cargo.toml` plotters line | exact |
| `tests/observe/visualization/test_visualization.rs` (extend) | test | file-I/O | same file (existing tests) | exact |
| `.github/workflows/wasm-check.yml` (add visualization step) | config | — | same file (existing steps) | exact |
| `examples/nsga2_zdt1.rs` (add `--plot`) | utility | request-response | `examples/nsga2_zdt1.rs` (existing `main`) | exact |
| `examples/spea2_zdt1.rs` (add `--plot`) | utility | request-response | `examples/nsga2_zdt1.rs` | role-match |
| `examples/sms_emoa_zdt1.rs` (add `--plot`) | utility | request-response | `examples/nsga2_zdt1.rs` | role-match |
| `examples/ibea_zdt1.rs` (add `--plot`) | utility | request-response | `examples/nsga2_zdt1.rs` | role-match |
| `examples/nsga3_dtlz2.rs` (add `--plot`) | utility | request-response | `examples/nsga2_zdt1.rs` | role-match |
| `examples/rastrigin.rs` (add `--plot`) | utility | request-response | `examples/rastrigin.rs` (existing `main`) | exact |
| `README.md` (extend Visualization section) | config | — | `README.md` (existing `### Visualization` section) | exact |

---

## Pattern Assignments

### `src/observe/visualization/mod.rs` — three new functions

**Analog:** `src/observe/visualization/mod.rs` (existing `plot_fitness`, `plot_diversity`, `plot_histogram`)

The entire file was read. All new functions must follow the established two-layer pattern exactly.

---

#### Layer 1: Range helper (copy `compute_y_range` / `compute_diversity_range` pattern)

**Range helper pattern** (lines 71–91 and 142–153):
```rust
fn compute_y_range(stats: &[GenerationStats]) -> (f64, f64) {
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for s in stats {
        y_min = y_min.min(s.best_fitness).min(s.avg_fitness).min(s.worst_fitness);
        y_max = y_max.max(s.best_fitness).max(s.avg_fitness).max(s.worst_fitness);
    }
    if (y_max - y_min).abs() < f64::EPSILON {
        y_max = y_min + 1.0;
    }
    (y_min, y_max)
}
```

New helper to write for pareto: `compute_pareto_range(iter: impl Iterator<Item = f64>) -> (f64, f64)` — apply same degenerate guard `if (max - min).abs() < f64::EPSILON { max = min + 1.0; }`. Call once per axis.

---

#### Layer 2a: Private `draw_*_chart<DB>` helper (copy `draw_fitness_chart` / `draw_diversity_chart` pattern)

**Private draw helper signature** (lines 98–105):
```rust
fn draw_fitness_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    stats: &[GenerationStats],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
```

All three new private helpers — `draw_pareto_2d_chart<DB>`, `draw_pareto_3d_chart<DB>`, `draw_true_fitness_calls_chart<DB>` — must use this identical signature shape (generic over `DB`, borrow `root`, return `DrawingAreaErrorKind<DB::ErrorType>`).

**ChartBuilder pattern** (lines 109–115):
```rust
let mut chart = ChartBuilder::on(root)
    .margin(10)
    .x_label_area_size(0)
    .y_label_area_size(0)
    .build_cartesian_2d(0usize..max_gen, y_min..y_max)?;

chart.configure_mesh().disable_mesh().draw()?;
```

For pareto 2d/3d, use `x_label_area_size(30)` and `y_label_area_size(30)` (non-zero — axis needs labels `f1`/`f2`/`f3`). Add `.x_desc("f1").y_desc("f2")` to `configure_mesh()` call.

**LineSeries draw pattern** (lines 118–133):
```rust
chart.draw_series(LineSeries::new(
    stats.iter().map(|s| (s.generation, s.best_fitness)),
    &BLUE,
))?;
```

For scatter (pareto functions), replace `LineSeries` with individual `Circle` elements:
```rust
chart.draw_series(
    points.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())),
)?;
```
Requires `"point_series"` feature in Cargo.toml plotters dep (see Cargo.toml section below).

**Three-panel layout for `draw_pareto_3d_chart`** — use `root.split_evenly((1, 3))`. Returns `Vec<DrawingArea<DB, Shift>>` with 3 elements (row-major: index 0=left, 1=center, 2=right). Iterate by index, not by destructuring (no `Copy`).

---

#### Layer 2b: Public dispatcher (copy `plot_fitness` / `plot_diversity` pattern verbatim)

**Public dispatcher pattern** (lines 256–284):
```rust
pub fn plot_fitness(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError> {
    if stats.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_fitness_chart(&root, stats)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present()
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        Some("svg") => {
            let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_fitness_chart(&root, stats)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present()
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}
```

Copy this three-step frame (`fill` → `draw_*_chart` → `present`) verbatim into all three new public functions. The only differences:
- `plot_pareto_front_2d`: image size `(800, 600)`, input guard `points.len() < 2`
- `plot_pareto_front_3d`: image size `(1200, 400)` (landscape, three panels), input guard `points.len() < 2`
- `plot_true_fitness_calls`: collect `data: Vec<(usize, u64)>` from `stats` first filtering `true_fitness_calls.is_some()`, then guard `data.len() < 2` returning `InsufficientData`

---

### `Cargo.toml` — add `"point_series"` to plotters features

**Analog:** `Cargo.toml` line 50 (current plotters dep)

**Current line** (line 50):
```toml
plotters = { version = "0.3.7", default-features = false, features = ["bitmap_backend", "bitmap_encoder", "svg_backend", "line_series", "histogram", "ab_glyph"], optional = true }
```

**Required change** — insert `"point_series"` after `"line_series"`:
```toml
plotters = { version = "0.3.7", default-features = false, features = ["bitmap_backend", "bitmap_encoder", "svg_backend", "line_series", "point_series", "histogram", "ab_glyph"], optional = true }
```

This must be done before any test compilation touching scatter plots or `Circle` elements.

---

### `tests/observe/visualization/test_visualization.rs` — extend with pareto tests

**Analog:** `tests/observe/visualization/test_visualization.rs` (existing test file, full read above)

**File header and feature gate** (lines 1–11):
```rust
//! Integration tests for the visualization module.
#![cfg(feature = "visualization")]

use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::visualization::{
    plot_diversity, plot_fitness, plot_histogram, VisualizationError,
};
```

New imports to add alongside existing ones:
```rust
use genetic_algorithms::visualization::{
    plot_diversity, plot_fitness, plot_histogram,
    plot_pareto_front_2d, plot_pareto_front_3d, plot_true_fitness_calls,
    VisualizationError,
};
```

**PNG happy-path test pattern** (lines 29–51):
```rust
#[test]
fn test_plot_fitness_png() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_fitness.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_fitness(&stats, path_str);
    assert!(result.is_ok(), "plot_fitness PNG failed: {:?}", result.err());
    assert!(path.exists(), "PNG file was not created");
    assert!(std::fs::metadata(&path).unwrap().len() > 0, "PNG file is empty");

    let _ = std::fs::remove_file(&path);
}
```

Copy this pattern for `test_plot_pareto_front_2d_png`, `test_plot_pareto_front_3d_png`, `test_plot_true_fitness_calls_png`. Fixtures:
- For pareto 2d: `let points: Vec<(f64,f64)> = (0..10).map(|i| (i as f64 * 0.1, 1.0 - i as f64 * 0.1)).collect();`
- For pareto 3d: `let points: Vec<(f64,f64,f64)> = (0..10).map(|i| (i as f64*0.1, i as f64*0.1, 1.0-i as f64*0.1)).collect();`
- For true_fitness_calls: reuse `make_stats` but set `true_fitness_calls: Some(50 + i as u64)` on each entry (requires a separate `make_stats_with_surrogate` helper)

**InsufficientData pattern** (lines 77–89):
```rust
#[test]
fn test_plot_fitness_insufficient_empty() {
    let stats: Vec<GenerationStats> = vec![];
    let path = std::env::temp_dir().join("test_viz_empty.png");
    let path_str = path.to_str().unwrap();

    let result = plot_fitness(&stats, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}
```

Copy for: empty points vec, single-point vec, all-None `true_fitness_calls`.

**`make_stats` helper** (lines 13–27) — note the `GenerationStats` struct literal requires ALL fields. The existing helper omits `true_fitness_calls` (added in Phase 62). Check that `make_stats` compiles after `true_fitness_calls` was added: it likely uses struct-literal syntax without `true_fitness_calls`, which requires the field to have a default or the helper to be updated. Add `true_fitness_calls: None` to the existing `make_stats` helper if needed, and add a new `make_stats_with_surrogate` helper that sets `true_fitness_calls: Some(n)`.

---

### `.github/workflows/wasm-check.yml` — add visualization step

**Analog:** `.github/workflows/wasm-check.yml` (existing file, full read above)

**Existing step pattern** (lines 31–32):
```yaml
- name: cargo check (serde feature)
  run: cargo check --target wasm32-unknown-unknown --lib --features serde
```

**New step to append** — same structure, different feature:
```yaml
- name: cargo check (visualization feature)
  run: cargo check --target wasm32-unknown-unknown --lib --features visualization
```

This step requires the PNG dispatch branches in `mod.rs` to be gated with `#[cfg(not(target_arch = "wasm32"))]` first (see Shared Patterns section below). Without that gate, this step will fail to compile.

---

### `examples/nsga2_zdt1.rs` — add `--plot` block

**Analog:** `examples/nsga2_zdt1.rs` (full read above)

**Return value and objective access** (lines 142–165): `nsga2.run()` returns `Ok(mut front)` where `front.individuals` is `Vec<Individual>` and each `ind.objectives[0]`, `ind.objectives[1]` are the two fitness values.

**`--plot` block to append** inside the `Ok(mut front)` arm, after the existing print loop:
```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features visualization
    let points: Vec<(f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_2d(
        &points,
        "docs/images/nsga2_zdt1.png",
    )
    .expect("plot failed");
    println!("Pareto front plot saved to docs/images/nsga2_zdt1.png");
}
```

**Placement:** After line 165 (inside `Ok(mut front)` match arm), before the closing brace.

---

### `examples/spea2_zdt1.rs` — add `--plot` block

**Analog:** `examples/nsga2_zdt1.rs` `--plot` block above (role-match)

Identical pattern to `nsga2_zdt1.rs`. Verified (grep results): `front.individuals[i].objectives[0]`, `front.individuals[i].objectives[1]` are the access pattern. Output path: `"docs/images/spea2_zdt1.png"`.

```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features visualization
    let points: Vec<(f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_2d(
        &points,
        "docs/images/spea2_zdt1.png",
    )
    .expect("plot failed");
    println!("Pareto front plot saved to docs/images/spea2_zdt1.png");
}
```

---

### `examples/sms_emoa_zdt1.rs` — add `--plot` block

**Analog:** `examples/nsga2_zdt1.rs` `--plot` block (role-match)

Same objectives access pattern confirmed (grep: `front.individuals[i].objectives[0]`, line 151). Output path: `"docs/images/sms_emoa_zdt1.png"`.

**Important:** This example has `required-features = ["benchmarks"]` in `Cargo.toml` (line 112). Running `--plot` requires `--features "visualization,benchmarks"`. Add a `// requires --features "visualization,benchmarks"` comment in the `--plot` block.

```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features "visualization,benchmarks"
    let points: Vec<(f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_2d(
        &points,
        "docs/images/sms_emoa_zdt1.png",
    )
    .expect("plot failed");
    println!("Pareto front plot saved to docs/images/sms_emoa_zdt1.png");
}
```

---

### `examples/ibea_zdt1.rs` — add `--plot` block

**Analog:** `examples/nsga2_zdt1.rs` `--plot` block (role-match)

Same objectives access pattern confirmed (grep: `front.individuals[i].objectives[0]`, line 151). Output path: `"docs/images/ibea_zdt1.png"`.

**Important:** Has `required-features = ["benchmarks"]` (Cargo.toml line 116). Same two-feature comment as sms_emoa.

```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features "visualization,benchmarks"
    let points: Vec<(f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_2d(
        &points,
        "docs/images/ibea_zdt1.png",
    )
    .expect("plot failed");
    println!("Pareto front plot saved to docs/images/ibea_zdt1.png");
}
```

---

### `examples/nsga3_dtlz2.rs` — add `--plot` block (3-obj)

**Analog:** `examples/nsga2_zdt1.rs` `--plot` block (role-match, 3-obj variant)

Objectives access confirmed (grep: `ind.objectives[0]`, `ind.objectives[1]`, `ind.objectives[2]`, lines 151–153). Uses `plot_pareto_front_3d` not `plot_pareto_front_2d`. Output path: `"docs/images/nsga3_dtlz2.png"`.

```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features visualization
    let points: Vec<(f64, f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1], ind.objectives[2]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_3d(
        &points,
        "docs/images/nsga3_dtlz2.png",
    )
    .expect("plot failed");
    println!("Pareto front plot saved to docs/images/nsga3_dtlz2.png");
}
```

---

### `examples/rastrigin.rs` — add `--plot` block

**Analog:** `examples/rastrigin.rs` (existing `main`, full read above)

**Critical difference from multi-obj examples:** `ga.run_with_callback(...)` returns `Ok(population)` — not `Vec<GenerationStats>`. Stats are passed per-generation to the callback. Must accumulate stats in a `Vec` captured by the callback.

**Callback accumulation pattern** (based on existing callback at lines 102–119 and RESEARCH.md Pitfall 4):

Replace the existing anonymous callback with a stats-accumulating closure using `Arc<Mutex<Vec<GenerationStats>>>`:

```rust
#[cfg(feature = "visualization")]
let plot_stats: std::sync::Arc<std::sync::Mutex<Vec<genetic_algorithms::stats::GenerationStats>>>
    = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
#[cfg(feature = "visualization")]
let plot_stats_clone = plot_stats.clone();
```

Then pass a callback that appends `_stats.clone()` to `plot_stats_clone`:
```rust
let result = ga.run_with_callback(
    Some(
        move |gen: &usize,
              pop: &Population<RangeChromosome<f64>>,
              _stats: &GenerationStats,
              _cause: &TerminationCause|
              -> std::ops::ControlFlow<()> {
            // existing print logic...
            #[cfg(feature = "visualization")]
            plot_stats_clone.lock().unwrap().push(_stats.clone());
            std::ops::ControlFlow::Continue(())
        },
    ),
    report_interval,
);
```

After `run_with_callback` returns, in the `Ok(population)` arm:
```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features visualization
    let stats = plot_stats.lock().unwrap();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_fitness(
        &stats,
        "docs/images/rastrigin.png",
    )
    .expect("plot failed");
    println!("Fitness plot saved to docs/images/rastrigin.png");
}
```

**Note:** The callback currently uses `_stats` (prefixed with `_` to suppress unused warning). Rename to `stats_ref` when capturing is needed, or add the `push` inside the existing callback body. The `GenerationStats` type must impl `Clone` (it does — it's `#[derive(Clone)]` in `src/stats.rs`).

---

### `README.md` — extend Visualization section

**Analog:** `README.md` existing `### Visualization` section

**Pattern:** Existing section describes `plot_fitness`, `plot_diversity` with prose. New sub-sections `#### Multi-Objective Pareto Fronts` and `#### Single-Objective Fitness Progress` are added inside `### Visualization`. Each image linked as:
```markdown
#### Multi-Objective Pareto Fronts

| Algorithm | Benchmark | Plot |
|-----------|-----------|------|
| NSGA-II | ZDT1 (2-obj) | ![NSGA-II ZDT1 Pareto front](docs/images/nsga2_zdt1.png) |
| SPEA2 | ZDT1 (2-obj) | ![SPEA2 ZDT1 Pareto front](docs/images/spea2_zdt1.png) |
| SMS-EMOA | ZDT1 (2-obj) | ![SMS-EMOA ZDT1 Pareto front](docs/images/sms_emoa_zdt1.png) |
| IBEA | ZDT1 (2-obj) | ![IBEA ZDT1 Pareto front](docs/images/ibea_zdt1.png) |
| NSGA-III | DTLZ2 (3-obj, three-panel) | ![NSGA-III DTLZ2 Pareto front](docs/images/nsga3_dtlz2.png) |

#### Single-Objective Fitness Progress

![Rastrigin fitness over generations](docs/images/rastrigin.png)
```

---

## Shared Patterns

### WASM Bitmap Gate
**Source:** `src/observe/visualization/mod.rs` (currently missing — must be added)
**Apply to:** All `Some("png")` match arms in every `plot_*` function (both existing and new)

The `BitMapBackend` type is not available on `wasm32-unknown-unknown`. Gate every PNG branch:
```rust
Some("png") => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        draw_fitness_chart(&root, stats)
            .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        root.present()
            .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        return Err(VisualizationError::UnsupportedFormat);
    }
}
```

This gate must be applied to all 3 existing functions (`plot_fitness`, `plot_diversity`, `plot_histogram`) AND all 3 new functions. It is a prerequisite for the `wasm-check.yml` new step to pass.

### Error Conversion Pattern
**Source:** `src/observe/visualization/mod.rs` lines 265–269
**Apply to:** All new `draw_*` call sites in public dispatchers
```rust
.map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
```

### `root.present()` Mandatory Flush
**Source:** `src/observe/visualization/mod.rs` lines 268–269
**Apply to:** All new public `plot_*` functions, both PNG and SVG branches
```rust
root.present()
    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
```

Missing this call produces a 0-byte or corrupt PNG. It is mandatory after every `draw_*_chart` call.

### Feature Flag Guard in Examples
**Source:** `src/engines/ga.rs` (WASM cfg gate reference from CONTEXT.md), D-11
**Apply to:** All 6 example `--plot` blocks
```rust
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // ...
}
```

### Degenerate Axis Range Guard
**Source:** `src/observe/visualization/mod.rs` lines 87–89
**Apply to:** `draw_pareto_2d_chart` and each panel in `draw_pareto_3d_chart`
```rust
if (y_max - y_min).abs() < f64::EPSILON {
    y_max = y_min + 1.0;
}
```
Apply the same guard to both x and y axes in pareto charts.

---

## No Analog Found

All files in scope have strong existing analogs. No files require patterns sourced from RESEARCH.md alone.

---

## Metadata

**Analog search scope:** `src/observe/visualization/`, `examples/`, `tests/observe/visualization/`, `Cargo.toml`, `.github/workflows/`
**Files scanned:** 11 (full reads: `mod.rs`, `nsga2_zdt1.rs`, `rastrigin.rs`, `test_visualization.rs`, `wasm-check.yml`, `stats.rs` partial, `lib.rs` partial; grep scans on remaining examples)
**Pattern extraction date:** 2026-06-09
