# Phase 9: Visualization - Research

**Researched:** 2026-03-21
**Domain:** Rust charting with `plotters` 0.3.7, optional Cargo feature flags
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Charting library: **`plotters`** — pure Rust, PNG and SVG natively, no system dependencies. Optional dep gated on `visualization` feature (same pattern as `serde`). No alternatives considered.
- API: standalone `genetic_algorithms::visualization` module — NOT methods on `Ga`. Three public functions:
  - `plot_fitness(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError>`
  - `plot_diversity(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError>`
  - `plot_histogram(fitness_values: &[f64], path: &str) -> Result<(), VisualizationError>`
- New `VisualizationError` enum lives in `src/visualization/` — separate from `GaError`. Minimum variants: `DrawingError`, `IoError`, `UnsupportedFormat`, `InsufficientData`.
- Format detection: path extension auto-detects — `.png` → `BitMapBackend`, `.svg` → `SVGBackend`, anything else → `Err(VisualizationError::UnsupportedFormat)`.
- Default chart dimensions: **800×600 pixels**. No user-configurable dimensions in this API version.
- Fitness chart (VIZ-01): three lines — best_fitness, avg_fitness, worst_fitness. Distinct colors, legend included. Error if fewer than 2 data points.
- Histogram input (VIZ-03): raw `&[f64]` fitness values, not `&[GenerationStats]`. Error if slice is empty.
- No changes to `src/ga.rs`, `src/stats.rs`, or any existing types.

### Claude's Discretion

- Exact `plotters` drawing API calls (series, axis ranges, grid lines, label formatting)
- Legend positioning and color palette for fitness chart lines
- Number of histogram bins (auto or fixed)
- Internal module structure within `src/visualization/` (separate files per chart type or all in `mod.rs`)
- Whether to add `pub use visualization::*;` re-export in `src/lib.rs` or require explicit import
- `VisualizationError` Display impl wording

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| VIZ-01 | User can plot fitness over generations (best, worst, average) to PNG/SVG | `LineSeries` with three series on a `Cartesian2d` chart via `ChartBuilder`; both `BitMapBackend` and `SVGBackend` confirmed in plotters 0.3.7 prelude |
| VIZ-02 | User can plot population diversity over generations to PNG/SVG | Same `LineSeries` pattern as VIZ-01; `GenerationStats.diversity` field confirmed present in `src/stats.rs` |
| VIZ-03 | User can plot fitness distribution at a given generation to PNG/SVG | `Histogram::vertical()` with `.into_segmented()` range confirmed in plotters 0.3.7; takes `&[f64]` input |
| VIZ-04 | Visualization is only available when the `visualization` feature flag is enabled | `#[cfg(feature = "visualization")]` gating pattern confirmed from existing `serde` feature in `src/lib.rs` line 70–71 and `Cargo.toml` |
</phase_requirements>

---

## Summary

Phase 9 adds an optional `visualization` feature that exposes three chart-generating functions over existing `GenerationStats` data. The charting library is **plotters 0.3.7** — the current published version as of 2026-03-21 per `cargo search`. Plotters is pure Rust with no native system dependencies, and both `BitMapBackend` (PNG) and `SVGBackend` (SVG) are included in its default feature set.

The implementation follows a two-part pattern already established in this codebase: (1) an optional Cargo dependency declared with `optional = true` and exposed via a named feature (`visualization = ["dep:plotters"]`), and (2) a module gated in `src/lib.rs` with `#[cfg(feature = "visualization")] pub mod visualization;`. This is identical to the existing `serde` feature and `checkpoint` module pattern.

The `plotters` API is well-suited to all three chart types. Line charts use `LineSeries` with `ChartBuilder::build_cartesian_2d`, the legend uses `configure_series_labels()` with `SeriesLabelPosition`, and histograms use `Histogram::vertical()` with a segmented coordinate range. The only non-obvious API detail is that the histogram requires `(0u32..N).into_segmented()` as the X coordinate type — raw `f64` values must be bucketed into integer bin indices before passing to `Histogram::vertical().data()`.

**Primary recommendation:** Implement `src/visualization/mod.rs` with all three functions and `VisualizationError`. Gate everything behind `#[cfg(feature = "visualization")]`. Add plotters 0.3.7 as an optional dep with `default-features = false, features = ["bitmap_encoder", "svg_backend", "histogram"]` to minimize compile-time impact for users who do not enable the feature.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| plotters | 0.3.7 | PNG/SVG chart rendering | Pure Rust, no system deps, supports both raster and vector output natively; locked decision |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| plotters `bitmap_encoder` feature | bundled | Enables `BitMapBackend` → PNG output | Required for `.png` path extension support |
| plotters `svg_backend` feature | bundled | Enables `SVGBackend` → SVG output | Required for `.svg` path extension support |
| plotters `histogram` feature | bundled | Enables `Histogram::vertical()` series | Required for VIZ-03 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| plotters | charming (Apache ECharts FFI) | Requires JavaScript runtime — unacceptable for a library crate |
| plotters | textplots | Terminal-only ASCII output — not PNG/SVG |

**Installation (Cargo.toml addition):**
```toml
plotters = { version = "0.3.7", default-features = false, features = ["bitmap_encoder", "svg_backend", "histogram"], optional = true }
```

**Feature declaration:**
```toml
[features]
visualization = ["dep:plotters"]
```

**Version verification:** Confirmed `plotters = "0.3.7"` via `cargo search plotters` on 2026-03-21.

---

## Architecture Patterns

### Recommended Module Structure

```
src/
└── visualization/
    └── mod.rs       # VisualizationError enum + all three public functions
```

Single-file is appropriate given the three functions share the same error type, backend-dispatch logic, and are tightly scoped. Splitting per chart type adds file overhead without benefit at this scope.

**Alternative (acceptable):** Split into `mod.rs` (re-exports + error) + `fitness.rs` + `diversity.rs` + `histogram.rs` if the implementer prefers it. Either is fine; the planner may choose.

### Pattern 1: Backend Dispatch by Path Extension

**What:** Detect output format from the file extension and select `BitMapBackend` vs `SVGBackend` at runtime. Because the two backends are different types, extract a shared drawing closure or use a macro/helper to avoid duplicating chart logic.

**When to use:** All three `plot_*` functions share this dispatch — factor it out.

**Example approach** (source: plotters 0.3.7 official docs + confirmed API):
```rust
// Source: https://docs.rs/plotters/0.3.7/plotters/backend/struct.SVGBackend.html
//         https://docs.rs/plotters/0.3.7/plotters/backend/struct.BitMapBackend.html
use std::path::Path;

fn detect_format(path: &str) -> Result<(), VisualizationError> {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => { /* use BitMapBackend */ Ok(()) }
        Some("svg") => { /* use SVGBackend */   Ok(()) }
        _           => Err(VisualizationError::UnsupportedFormat),
    }
}
```

Because `BitMapBackend` and `SVGBackend` are different concrete types (not `dyn DrawingBackend`), the cleanest pattern is a private macro or a pair of private `draw_fitness_chart<DB: DrawingBackend>` generics. See Pattern 2.

### Pattern 2: Generic Drawing Function Over `DrawingBackend`

**What:** Write the chart logic once as a generic function `fn draw_fitness<DB: DrawingBackend>(root: DrawingArea<DB, Shift>, stats: &[GenerationStats]) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>`, then dispatch from the public function.

**Why:** Avoids duplicating `ChartBuilder` calls for PNG vs SVG paths.

**Note:** `DrawingAreaErrorKind` is not `std::error::Error`, so conversion into `VisualizationError::DrawingError` requires wrapping via `.map_err(|e| VisualizationError::DrawingError(e.to_string()))`.

### Pattern 3: Line Chart (VIZ-01 and VIZ-02)

**Source:** plotters 0.3.7 official chart example (github.com/plotters-rs/plotters/blob/master/plotters/examples/chart.rs)

```rust
// Source: https://docs.rs/plotters/0.3.7/plotters/
use plotters::prelude::*;

// Assumes root: DrawingArea<DB, Shift> already created and filled WHITE
let max_gen = stats.last().map(|s| s.generation).unwrap_or(0);
let (y_min, y_max) = compute_y_range(stats); // scan stats for min/max fitness

let mut chart = ChartBuilder::on(&root)
    .caption("Fitness over Generations", ("sans-serif", 30))
    .margin(10)
    .x_label_area_size(35)
    .y_label_area_size(50)
    .build_cartesian_2d(0usize..max_gen, y_min..y_max)?;

chart.configure_mesh()
    .x_desc("Generation")
    .y_desc("Fitness")
    .draw()?;

chart.draw_series(LineSeries::new(
    stats.iter().map(|s| (s.generation, s.best_fitness)),
    &BLUE,
))?.label("Best").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

chart.draw_series(LineSeries::new(
    stats.iter().map(|s| (s.generation, s.avg_fitness)),
    &GREEN,
))?.label("Average").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

chart.draw_series(LineSeries::new(
    stats.iter().map(|s| (s.generation, s.worst_fitness)),
    &RED,
))?.label("Worst").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

chart.configure_series_labels()
    .position(SeriesLabelPosition::UpperRight)
    .background_style(WHITE.mix(0.8))
    .border_style(BLACK)
    .draw()?;

root.present()?;
```

### Pattern 4: Histogram (VIZ-03)

**Source:** plotters 0.3.7 histogram example (github.com/plotters-rs/plotters/blob/master/plotters/examples/histogram.rs)

The key constraint: `Histogram::vertical()` needs a **segmented** coordinate on the X axis — `(0u32..num_bins).into_segmented()`. Raw `f64` values must be mapped to bin indices.

```rust
// Source: https://docs.rs/plotters/0.3.7/plotters/series/struct.Histogram.html
use plotters::prelude::*;

const NUM_BINS: u32 = 20;

// Compute bin width
let min = fitness_values.iter().cloned().fold(f64::INFINITY, f64::min);
let max = fitness_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
let bin_width = (max - min) / NUM_BINS as f64;

let mut chart = ChartBuilder::on(&root)
    .caption("Fitness Distribution", ("sans-serif", 30))
    .margin(10)
    .x_label_area_size(35)
    .y_label_area_size(40)
    .build_cartesian_2d(
        (0u32..NUM_BINS).into_segmented(),
        0u32..(fitness_values.len() as u32),
    )?;

chart.configure_mesh()
    .x_desc("Fitness Bin")
    .y_desc("Count")
    .draw()?;

chart.draw_series(
    Histogram::vertical(&chart)
        .style(BLUE.mix(0.5).filled())
        .margin(1)
        .data(fitness_values.iter().map(|&v| {
            let bin = ((v - min) / bin_width).min((NUM_BINS - 1) as f64) as u32;
            (bin, 1u32)
        })),
)?;

root.present()?;
```

**Edge case:** If `max == min` (all fitness values identical), `bin_width` is 0.0. Guard with: if `bin_width == 0.0`, place all values in bin 0 or return `Err(VisualizationError::InsufficientData)`.

### Anti-Patterns to Avoid

- **Using `dyn DrawingBackend` as a return type:** The `DrawingBackend` trait is not object-safe in all configurations. Use generics or enum-dispatch instead.
- **Calling `root.present()` inside the generic helper:** The `?` error type varies per backend. Call `present()` in the caller after the generic function returns.
- **Omitting `root.fill(&WHITE)`:** Without filling the background, the output has a transparent or undefined background. Always call `.fill(&WHITE)?` immediately after `into_drawing_area()`.
- **Passing empty stats without guarding:** `build_cartesian_2d(0..0, 0.0..0.0)` will produce an empty chart or panic. The functions must return `Err(VisualizationError::InsufficientData)` when `stats.len() < 2` (line charts) or `fitness_values.is_empty()` (histogram).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rendering PNG raster output | Custom pixel writer | `plotters::BitMapBackend` | Handles PNG encoding, pixel layout, file writing |
| Rendering SVG vector output | Custom SVG string builder | `plotters::SVGBackend` | Handles SVG element tree, viewport, escaping |
| Drawing chart axes, grid, labels | Custom coordinate math | `ChartBuilder::build_cartesian_2d` + `configure_mesh()` | Handles axis ranging, tick mark placement, label formatting |
| Histogram bin accumulation | Manual bin counting loop | `Histogram::vertical().data()` | Accumulates `(position, count)` pairs internally |
| Legend rendering | Custom legend layout | `configure_series_labels()` + `SeriesLabelPosition` | Handles bounding, positioning, border, background |

**Key insight:** plotters handles all low-level rendering concerns. The implementation role is data mapping (stats fields → coordinate tuples) and error translation (`DrawingAreaErrorKind` → `VisualizationError`).

---

## Common Pitfalls

### Pitfall 1: Feature Flag Scope on plotters Items

**What goes wrong:** Code compiles without `visualization` feature but uses plotters types that are only available when the feature is enabled — or vice versa: the module compiles only under the feature but tests are run without it.

**Why it happens:** `#[cfg(feature = "visualization")]` on the module in `lib.rs` gates the public API, but test files in `tests/` that import `genetic_algorithms::visualization` will fail to compile without `--features visualization`.

**How to avoid:** All integration tests for visualization go in `tests/test_visualization.rs` with `#![cfg(feature = "visualization")]` at the top (identical pattern to `tests/test_serde.rs` which uses `#![cfg(feature = "serde")]`). Run with `cargo test --features visualization`.

**Warning signs:** `error[E0432]: unresolved import genetic_algorithms::visualization` when running `cargo test` without the feature flag.

### Pitfall 2: `DrawingAreaErrorKind` Is Not `std::error::Error`

**What goes wrong:** Attempting `map_err(|e| VisualizationError::DrawingError(e))` where `e` is a `DrawingAreaErrorKind<BE::ErrorType>` — this type does not implement `std::error::Error` in all backends, and its `to_string()` requires `Display`.

**Why it happens:** plotters error types are generic over the backend error type, making them awkward to store in an enum variant.

**How to avoid:** Convert to string at the boundary: `map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))`. Store a `String` in `DrawingError(String)`.

### Pitfall 3: Histogram X-Axis Type Mismatch

**What goes wrong:** Attempting `build_cartesian_2d(0f64..1f64, 0u32..max_count)` for a histogram and then calling `Histogram::vertical()` — the segmented range type `SegmentedCoord` is required, not a plain float range.

**Why it happens:** `Histogram::vertical()` uses the `Ranged` + `AsRangedCoord` system and requires a segmented coordinate. The compiler error message is opaque.

**How to avoid:** Always use `(0u32..NUM_BINS).into_segmented()` as the X spec when building a histogram chart. Map `f64` values to `u32` bin indices explicitly before passing to `.data()`.

### Pitfall 4: Y-Axis Range When All Fitness Values Are Equal

**What goes wrong:** `build_cartesian_2d(0..max_gen, y_min..y_max)` where `y_min == y_max` produces a degenerate range, causing a panic or blank chart.

**Why it happens:** A single-value range has zero span; plotters cannot compute tick marks.

**How to avoid:** After computing `y_min` and `y_max`, expand: `let y_max = if (y_max - y_min).abs() < f64::EPSILON { y_min + 1.0 } else { y_max };`.

### Pitfall 5: Forgetting `root.present()`

**What goes wrong:** The output file is created but contains partial or no data because the backend buffer was never flushed.

**Why it happens:** `BitMapBackend` and `SVGBackend` buffer internally; `present()` triggers the actual write.

**How to avoid:** Always call `root.present()?` as the final step before returning `Ok(())`.

---

## Code Examples

### Feature Flag Setup (Cargo.toml)

```toml
# Source: existing serde pattern in Cargo.toml + plotters feature flag docs
[features]
default = []
serde = ["dep:serde", "dep:serde_json"]
visualization = ["dep:plotters"]

[dependencies]
plotters = { version = "0.3.7", default-features = false, features = ["bitmap_encoder", "svg_backend", "histogram"], optional = true }
```

### Module Gating (src/lib.rs)

```rust
// Source: existing pattern at lib.rs line 70-71
#[cfg(feature = "serde")]
pub mod checkpoint;

// Add:
#[cfg(feature = "visualization")]
pub mod visualization;
```

### VisualizationError (following GaError style in src/error.rs)

```rust
// Source: src/error.rs — GaError style; no thiserror macro used in this codebase
use std::fmt;

#[derive(Debug)]
pub enum VisualizationError {
    /// Drawing backend error (file write, PNG encode, SVG render).
    DrawingError(String),
    /// I/O error accessing the output path.
    IoError(String),
    /// File extension is not `.png` or `.svg`.
    UnsupportedFormat,
    /// Input data has too few points to produce a meaningful chart.
    InsufficientData,
}

impl fmt::Display for VisualizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisualizationError::DrawingError(msg) => write!(f, "Drawing error: {}", msg),
            VisualizationError::IoError(msg)      => write!(f, "I/O error: {}", msg),
            VisualizationError::UnsupportedFormat => write!(f, "Unsupported format: path must end in .png or .svg"),
            VisualizationError::InsufficientData  => write!(f, "Insufficient data: at least 2 data points required"),
        }
    }
}

impl std::error::Error for VisualizationError {}
```

### Format Detection Skeleton

```rust
// Source: std::path::Path docs + plotters backend selection
use std::path::Path;
use plotters::prelude::*;

pub fn plot_fitness(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError> {
    if stats.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE).map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_fitness_chart(&root, stats)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present().map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        Some("svg") => {
            let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE).map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_fitness_chart(&root, stats)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present().map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }
    Ok(())
}
```

Note: `draw_fitness_chart` must be a generic `fn<DB: DrawingBackend>` — see Pattern 2 above. The exact lifetime/type bounds are verbose; the implementer should consult the plotters 0.3.7 `DrawingArea` type signature.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| plotters 0.3.x had `plotters-bitmap` as separate crate | Now `bitmap_encoder` feature within `plotters` crate | 0.3.x series | Single dep entry in Cargo.toml |
| Full default feature set (heavy compile) | `default-features = false` with explicit features | Available throughout 0.3.x | Reduces compile time for optional dep users |

**Deprecated/outdated:**
- `plotters-svg`: SVGBackend is now bundled in `plotters` itself via the `svg_backend` feature. No separate `plotters-svg` dep needed in 0.3.7.

---

## Open Questions

1. **Generic `DrawingBackend` lifetime bounds**
   - What we know: The private helper `draw_fitness_chart<DB: DrawingBackend>(root: &DrawingArea<DB, Shift>, ...)` approach is sound conceptually.
   - What's unclear: The exact lifetime and `where` clause needed — `DrawingArea` carries a lifetime tied to the backend, and `DrawingBackend::ErrorType` must be `Debug` for formatting.
   - Recommendation: Implementer should check the plotters 0.3.7 `DrawingArea` signature directly. Alternatively, duplicate the chart body in each `Some("png")` / `Some("svg")` match arm to avoid the generic complexity entirely (small duplication, simpler code).

2. **Histogram bin count strategy**
   - What we know: Fixed 20 bins is simple and confirmed working in the histogram example.
   - What's unclear: Whether Sturges' rule (k = ceil(log2(n) + 1)) would be better for variable population sizes.
   - Recommendation: Use fixed 20 bins for Phase 9 (discretion area). Note it in code; can be parameterized later.

---

## Validation Architecture

`workflow.nyquist_validation` is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none — cargo built-in |
| Quick run command | `cargo test --features visualization test_visualization` |
| Full suite command | `cargo test --features visualization && cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VIZ-01 | `plot_fitness` writes a valid PNG/SVG file when given >= 2 stats | integration | `cargo test --features visualization viz_fitness` | Wave 0 |
| VIZ-01 | `plot_fitness` returns `InsufficientData` for < 2 stats | unit | `cargo test --features visualization viz_fitness_insufficient` | Wave 0 |
| VIZ-01 | `plot_fitness` returns `UnsupportedFormat` for unknown extension | unit | `cargo test --features visualization viz_fitness_bad_ext` | Wave 0 |
| VIZ-02 | `plot_diversity` writes a valid file; uses `diversity` field | integration | `cargo test --features visualization viz_diversity` | Wave 0 |
| VIZ-02 | `plot_diversity` returns `InsufficientData` for < 2 stats | unit | `cargo test --features visualization viz_diversity_insufficient` | Wave 0 |
| VIZ-03 | `plot_histogram` writes a valid file for non-empty `&[f64]` | integration | `cargo test --features visualization viz_histogram` | Wave 0 |
| VIZ-03 | `plot_histogram` returns `InsufficientData` for empty slice | unit | `cargo test --features visualization viz_histogram_empty` | Wave 0 |
| VIZ-04 | `genetic_algorithms::visualization` module is absent without feature | compile-test | `cargo build` (no feature) must succeed without visualization module | Verify in Wave 0 setup |

**VIZ-04 testing strategy:** This is a compilation property, not a runtime assertion. Verify by running `cargo build` (default features only) and confirming it compiles cleanly, then confirming `use genetic_algorithms::visualization;` in a test file with `#![cfg(not(feature = "visualization"))]` produces a compile error. In practice: the `#![cfg(feature = "visualization")]` guard at the top of `tests/test_visualization.rs` is the standard project pattern (see `tests/test_serde.rs` line 5: `#![cfg(feature = "serde")]`).

**File existence test pattern:** For VIZ-01/02/03 integration tests, write the chart to a `tempfile` or a path under `std::env::temp_dir()`, assert `std::path::Path::new(&out_path).exists()`, then clean up with `std::fs::remove_file`. This avoids leaving artifacts in the repo while confirming the backend actually wrote bytes.

### Sampling Rate

- **Per task commit:** `cargo test --features visualization test_visualization`
- **Per wave merge:** `cargo test --features visualization && cargo test && cargo clippy`
- **Phase gate:** Full suite green (including `cargo test --features serde`) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_visualization.rs` — covers VIZ-01, VIZ-02, VIZ-03, VIZ-04 (all new; does not exist yet)
- [ ] `src/visualization/mod.rs` — new module (does not exist yet)

No missing framework or shared fixtures — cargo test harness requires no additional setup.

---

## Sources

### Primary (HIGH confidence)

- `cargo search plotters` — confirmed version 0.3.7 published 2026-03-21
- https://docs.rs/plotters/0.3.7/plotters/ — feature flags, backend list, prelude types
- https://docs.rs/plotters/0.3.7/plotters/backend/struct.SVGBackend.html — SVGBackend constructor, feature requirement
- https://docs.rs/plotters/0.3.7/plotters/series/struct.Histogram.html — Histogram API, `.data()`, `.style()`, `.margin()`
- https://docs.rs/plotters/0.3.7/plotters/chart/struct.ChartContext.html — `configure_series_labels()`, `SeriesLabelPosition`, `draw_series()` → `SeriesAnno`
- https://github.com/plotters-rs/plotters/blob/master/plotters/examples/chart.rs — verified line chart pattern with legend
- https://github.com/plotters-rs/plotters/blob/master/plotters/examples/histogram.rs — verified histogram pattern with segmented X axis
- `src/lib.rs` line 70–71 — existing `#[cfg(feature = "serde")]` pattern (HIGH — in-repo)
- `Cargo.toml` — existing feature/dep pattern for `serde` (HIGH — in-repo)
- `src/stats.rs` — `GenerationStats` fields confirmed: `generation`, `best_fitness`, `worst_fitness`, `avg_fitness`, `diversity` (HIGH — in-repo)
- `src/error.rs` — `GaError` style: plain enum + `Display` impl, no `thiserror` (HIGH — in-repo)
- `tests/test_serde.rs` line 5 — `#![cfg(feature = "serde")]` guard pattern (HIGH — in-repo)

### Secondary (MEDIUM confidence)

- Plotters prelude feature flag list (bitmap_encoder, svg_backend, histogram defaults) — fetched from docs.rs, cross-referenced with cargo search result

### Tertiary (LOW confidence)

None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — version confirmed via cargo search; library locked by user decision
- Architecture: HIGH — patterns taken from official plotters 0.3.7 docs and examples; existing project patterns confirmed from source
- Pitfalls: MEDIUM — derived from API structure (type system constraints) and established project patterns; some pitfalls are inferred from API design rather than observed failures

**Research date:** 2026-03-21
**Valid until:** 2026-04-20 (plotters 0.3.x is stable; unlikely to change)
