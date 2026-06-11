# Phase 63: Visualization — Pareto Front Plotting & Example Images - Research

**Researched:** 2026-06-09
**Domain:** Rust `plotters` 0.3.7 — scatter plots, multi-panel layouts, feature flags, WASM constraints
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Two separate functions: `plot_pareto_front_2d(points: &[(f64, f64)], path: &str)` and `plot_pareto_front_3d(points: &[(f64, f64, f64)], path: &str)`, both `-> Result<(), VisualizationError>`. Both live in `src/observe/visualization/mod.rs`.
- **D-02:** First argument takes extracted fitness coordinates. Caller extracts from NSGA-II population (`c.fitness_values()[0]`, `c.fitness_values()[1]`). No generic chromosome bound in the visualization module.
- **D-03:** Functions plot only the points passed in. No dominance checking inside visualization module.
- **D-04:** Axis labels are generic: `f1`, `f2`, `f3`. No label argument.
- **D-05:** Add `plot_true_fitness_calls(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError>` — plots `true_fitness_calls: Option<u64>` from `GenerationStats`. Renders a line chart of `Some(n)` values; skips `None`. Lives in same module.
- **D-06:** `plot_pareto_front_3d` renders a three-panel image (three side-by-side 2D scatter charts): f1×f2, f1×f3, f2×f3. Image dimensions: 1200×400.
- **D-07:** All 5 multi-objective examples get `--plot`: `nsga2_zdt1`, `spea2_zdt1`, `sms_emoa_zdt1`, `ibea_zdt1`, `nsga3_dtlz2`.
- **D-08:** `rastrigin.rs` gets `--plot` using existing `plot_fitness`.
- **D-09:** `--plot` parsed with `std::env::args().any(|a| a == "--plot")`. No `clap`.
- **D-10:** Each example creates `docs/images/` with `std::fs::create_dir_all("docs/images")`, writes to `docs/images/[name].png`.
- **D-11:** `--plot` block gated with `#[cfg(feature = "visualization")]`.
- **D-12:** All 6 images linked from `README.md` `### Visualization` section, under new sub-sections `#### Multi-Objective Pareto Fronts` and `#### Single-Objective Fitness Progress`.

### Claude's Discretion

- Exact scatter plot marker style (circles vs dots, size), background color, grid lines — follow existing module's aesthetic
- Whether `plot_true_fitness_calls` returns `InsufficientData` if all values are `None`
- Whether three panels in `plot_pareto_front_3d` share a margin or have a small gap
- Caption text for README image alt attributes

### Deferred Ideas (OUT OF SCOPE)

- `--plot` for CMA-ES / island model examples
- Optional axis label arguments
- Color gradient for f3 in 3-obj
- True fitness calls chart in multi-obj observer
</user_constraints>

---

## Summary

Phase 63 extends the already-functional `visualization` module in `src/observe/visualization/mod.rs` with three new functions: `plot_pareto_front_2d`, `plot_pareto_front_3d`, and `plot_true_fitness_calls`. The existing module infrastructure (PNG/SVG dispatch, `VisualizationError`, `plotters::prelude::*` import) is fully reusable. The `draw_*_chart<DB>` private helper + public `plot_*` dispatcher pattern is established and must be followed exactly.

The critical discovery for this phase: the current plotters dependency line does **not** include `"point_series"` in its features list. Scatter plots using `PointSeries` or drawing `Circle` elements via `draw_series` with individual elements requires this feature. The planner must add `"point_series"` to the plotters features in Cargo.toml as part of this phase.

WASM compatibility is the other key constraint. `BitMapBackend` relies on `plotters-bitmap` which is not available on `wasm32-unknown-unknown`. The existing module uses `BitMapBackend` and `SVGBackend` without WASM gates — this means the `visualization` feature is currently **not** tested in the wasm-check CI workflow (which only runs `--lib` without `--features visualization`). The CONTEXT.md says "Feature compiles and links on WASM (plotters supports wasm32)" but `BitMapBackend` is not supported on WASM. The resolution: the PNG branch of the dispatch must be gated with `#[cfg(not(target_arch = "wasm32"))]`, with SVG as the WASM path. This pattern does not yet exist in the current module — it must be added consistently to all functions (both existing and new) when the wasm-check CI is updated to include the visualization feature.

All six example output images are committed to `docs/images/` (new directory). The `docs/` directory already exists (contains all the documentation markdown files). `docs/images/` is new.

**Primary recommendation:** Add `"point_series"` to Cargo.toml plotters features, implement the three new functions following the established `draw_*_chart<DB>` pattern, add `--plot` blocks to 6 examples, create `docs/images/`, commit images, and extend README.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Pareto front rendering | Library (visualization module) | — | Pure in-process plotting; no external service boundary |
| Objective value extraction | Caller (example code) | — | D-02: examples extract `(f1, f2)` tuples before calling |
| File system write | Library (visualization functions) | — | `BitMapBackend`/`SVGBackend` write to path argument |
| `--plot` flag parsing | Example binary | — | Each example parses `std::env::args` independently |
| Image directory creation | Example binary | — | `std::fs::create_dir_all` in example before calling plot fn |
| WASM compatibility gating | Library module | — | `#[cfg(not(target_arch = "wasm32"))]` on bitmap branches |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `plotters` | 0.3.7 (already in Cargo.toml) | Chart rendering backend | Already adopted; `visualization` feature uses it |

**Required feature addition to Cargo.toml plotters dep:**

The current plotters features line is:
```toml
plotters = { version = "0.3.7", default-features = false, features = ["bitmap_backend", "bitmap_encoder", "svg_backend", "line_series", "histogram", "ab_glyph"], optional = true }
```

Must add `"point_series"` to the features list for `PointSeries<Circle>` / individual `Circle` element drawing to work:

```toml
plotters = { version = "0.3.7", default-features = false, features = ["bitmap_backend", "bitmap_encoder", "svg_backend", "line_series", "point_series", "histogram", "ab_glyph"], optional = true }
```

[VERIFIED: docs.rs/plotters/0.3.7] — `point_series` is a named feature in plotters 0.3.7.

### Supporting

No additional crate dependencies. All capabilities come from the existing `plotters` dep.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `PointSeries<Circle>` | Individual `Circle` elements via `draw_series(iter)` | Both work; `PointSeries` is the idiomatic plotters approach and requires `point_series` feature either way |
| Three-panel `split_evenly((1,3))` | Single chart with color-coded axis pairs | CONTEXT.md locks three-panel; `split_evenly` is the correct plotters API [VERIFIED: docs.rs] |

**Installation:** No new crates. Modify existing Cargo.toml line only.

---

## Package Legitimacy Audit

No new packages are installed in this phase. The only change is adding `"point_series"` to the features list of the already-present `plotters` 0.3.7 dependency.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
Example binary (e.g. nsga2_zdt1.rs)
    │
    ├── ga.run() ──────────────────────────────► ParetoFront { individuals: Vec<Individual> }
    │                                                   │
    │   #[cfg(feature = "visualization")]               │ .objectives[0..n]
    │   if std::env::args().any(|a| a == "--plot") {    │
    │       let points: Vec<(f64,f64)> = ◄─────────────┘
    │           front.individuals.iter()
    │               .map(|ind| (ind.objectives[0], ind.objectives[1]))
    │               .collect();
    │       std::fs::create_dir_all("docs/images")
    │       plot_pareto_front_2d(&points, "docs/images/nsga2_zdt1.png")
    │   }
    │
    └── returns to shell

genetic_algorithms::visualization::plot_pareto_front_2d(points, path)
    │
    ├── draw_pareto_2d_chart<BitMapBackend>  ──► docs/images/nsga2_zdt1.png (PNG)
    └── draw_pareto_2d_chart<SVGBackend>    ──► *.svg (SVG)

genetic_algorithms::visualization::plot_pareto_front_3d(points, path)
    │
    └── root.split_evenly((1, 3)) ──► [area_f1f2, area_f1f3, area_f2f3]
             │          │          │
             ▼          ▼          ▼
         f1×f2      f1×f3      f2×f3
         scatter    scatter    scatter
```

### Recommended Project Structure

No new directories in `src/`. Only additions:

```
src/observe/visualization/mod.rs   ← add 3 new functions + draw helpers
docs/images/                       ← new directory (created by examples at first --plot run)
  nsga2_zdt1.png
  spea2_zdt1.png
  sms_emoa_zdt1.png
  ibea_zdt1.png
  nsga3_dtlz2.png
  rastrigin.png
examples/nsga2_zdt1.rs             ← add --plot block
examples/spea2_zdt1.rs             ← add --plot block
examples/sms_emoa_zdt1.rs          ← add --plot block
examples/ibea_zdt1.rs              ← add --plot block
examples/nsga3_dtlz2.rs            ← add --plot block
examples/rastrigin.rs              ← add --plot block
README.md                          ← extend Visualization section
```

### Pattern 1: `draw_pareto_2d_chart<DB>` Private Helper

Follow the exact pattern of `draw_fitness_chart<DB>` already in the module.

```rust
// Source: src/observe/visualization/mod.rs (established pattern)
fn draw_pareto_2d_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    points: &[(f64, f64)],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    // compute x and y ranges (with degenerate-case expansion)
    let x_min = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let x_max = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let y_min = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let y_max = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    // expand degenerate ranges by 1.0

    let mut chart = ChartBuilder::on(root)
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart.configure_mesh().disable_mesh()
        .x_desc("f1")
        .y_desc("f2")
        .draw()?;

    chart.draw_series(
        points.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())),
    )?;

    Ok(())
}
```

[ASSUMED] — exact axis label configuration method name (`.x_desc`/`.y_desc`) — verified pattern via docs.rs but not confirmed via Context7.

### Pattern 2: `draw_pareto_3d_chart<DB>` — Three-Panel Layout

```rust
// Source: docs.rs/plotters/0.3.7 DrawingArea::split_evenly [VERIFIED]
fn draw_pareto_3d_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    points: &[(f64, f64, f64)],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    let panels = root.split_evenly((1, 3));
    // panels[0] = f1×f2, panels[1] = f1×f3, panels[2] = f2×f3
    let pairs: [(&DrawingArea<DB, Shift>, usize, usize, &str, &str); 3] = [
        (&panels[0], 0, 1, "f1", "f2"),
        (&panels[1], 0, 2, "f1", "f3"),
        (&panels[2], 1, 2, "f2", "f3"),
    ];
    for (panel, xi, yi, x_label, y_label) in &pairs {
        // build chart on panel, draw circles
        // ...
    }
    Ok(())
}
```

[VERIFIED: docs.rs/plotters] — `split_evenly((1, 3))` returns `Vec<DrawingArea<DB, Shift>>` with 3 elements for a 1-row, 3-column grid.

### Pattern 3: Public Dispatcher (PNG/SVG)

Copy verbatim from `plot_fitness` and `plot_diversity`:

```rust
// Source: src/observe/visualization/mod.rs (existing pattern — verified by reading file)
pub fn plot_pareto_front_2d(
    points: &[(f64, f64)],
    path: &str,
) -> Result<(), VisualizationError> {
    if points.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE).map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_pareto_2d_chart(&root, points)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present().map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        Some("svg") => { /* same with SVGBackend */ }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }
    Ok(())
}
```

### Pattern 4: `--plot` Block in Example

```rust
// Source: D-11 from CONTEXT.md
#[cfg(feature = "visualization")]
if std::env::args().any(|a| a == "--plot") {
    // requires --features visualization
    let points: Vec<(f64, f64)> = front.individuals.iter()
        .map(|ind| (ind.objectives[0], ind.objectives[1]))
        .collect();
    std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
    genetic_algorithms::visualization::plot_pareto_front_2d(&points, "docs/images/nsga2_zdt1.png")
        .expect("plot failed");
    println!("Pareto front plot saved to docs/images/nsga2_zdt1.png");
}
```

For `rastrigin.rs`, the `ga.run_with_callback()` call returns a `Population` but does NOT return `Vec<GenerationStats>`. Stats must be collected inside the callback by appending to a `Vec<GenerationStats>` captured via `RefCell` or by using `Arc<Mutex<Vec<GenerationStats>>>`, then passed to `plot_fitness` after the run.

[VERIFIED: reading examples/rastrigin.rs] — `run_with_callback` signature receives `_stats: &GenerationStats` per generation; the caller must accumulate them manually.

### Pattern 5: `plot_true_fitness_calls`

```rust
pub fn plot_true_fitness_calls(
    stats: &[GenerationStats],
    path: &str,
) -> Result<(), VisualizationError> {
    let data: Vec<(usize, u64)> = stats.iter()
        .filter_map(|s| s.true_fitness_calls.map(|v| (s.generation, v)))
        .collect();
    if data.is_empty() {
        return Err(VisualizationError::InsufficientData);
    }
    // dispatch PNG/SVG, draw LineSeries in MAGENTA
}
```

### Anti-Patterns to Avoid

- **Using `par_iter` in visualization helpers:** violates WASM constraint. All iteration in draw helpers must use `.iter()`.
- **Calling `std::time::Instant::now()` in visualization:** not needed, but flag for completeness.
- **Checking `docs/images/` existence before writing:** let `std::fs::create_dir_all` handle idempotently.
- **Omitting `root.present()` after drawing:** PNG backend buffers writes; missing `.present()` produces an empty/corrupt file. This is documented in existing functions and must not be skipped.
- **Using `plotters::series::PointSeries` without `"point_series"` feature:** compile error. Add feature first.
- **Borrow conflict with `split_evenly` return:** `split_evenly` returns `Vec<DrawingArea<DB, Shift>>` — cannot iterate with `for &panel` (no Copy). Iterate by index or by reference.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Scatter point rendering | Custom pixel-level drawing | `Circle` element from `plotters::element` | Handles coordinate mapping, clipping, anti-aliasing |
| Multi-panel layout | Manual coordinate arithmetic | `root.split_evenly((1, 3))` | plotters handles DrawingArea subdivision |
| Range axis computation | Ad-hoc min/max loops | Follow `compute_y_range` pattern already in module | Degenerate (zero-span) case already handled there |
| PNG file flush | Assuming write-on-drop | `root.present()` | BitMapBackend buffers; absent call = empty file |

**Key insight:** The `DrawingArea` abstraction eliminates all coordinate math — `ChartBuilder` handles axis scaling, label placement, and clipping automatically.

---

## Common Pitfalls

### Pitfall 1: Missing `"point_series"` Feature

**What goes wrong:** Compile error `use of undeclared crate or module 'PointSeries'` or `Circle` element not found via `draw_series` iterator.
**Why it happens:** plotters uses features to gate series/element types. The current Cargo.toml line for plotters does not include `"point_series"`.
**How to avoid:** Add `"point_series"` to the `features = [...]` list for `plotters` in Cargo.toml as the first task of this phase.
**Warning signs:** Compile errors mentioning `PointSeries` or related items not in scope.

### Pitfall 2: Missing `root.present()` Call

**What goes wrong:** PNG file is written as 0 bytes or a PNG header with no pixel data.
**Why it happens:** `BitMapBackend` buffers all draw calls and only flushes to disk on `.present()`. Drop does not flush.
**How to avoid:** Every public `plot_*` function already calls `root.present()`. New functions must follow the same three-step: `fill` → `draw_*_chart` → `present`.
**Warning signs:** Generated PNG opens as blank or fails to open in image viewer.

### Pitfall 3: `sms_emoa_zdt1` and `ibea_zdt1` Require `--features benchmarks`

**What goes wrong:** `cargo run --example sms_emoa_zdt1 --features visualization -- --plot` fails to compile because the example is declared with `required-features = ["benchmarks"]` in Cargo.toml.
**Why it happens:** Both examples have `required-features = ["benchmarks"]` in their `[[example]]` entries. The `--plot` block is also gated with `#[cfg(feature = "visualization")]`.
**How to avoid:** Run these examples with both features: `cargo run --example sms_emoa_zdt1 --features "visualization,benchmarks" -- --plot`. Document this in the example docstring.
**Warning signs:** `error: target 'sms_emoa_zdt1' in package ... requires the features: benchmarks`.

### Pitfall 4: `rastrigin.rs` Doesn't Return Stats Directly

**What goes wrong:** `plot_fitness` requires `&[GenerationStats]` but `run_with_callback` does not return accumulated stats — it passes a `&GenerationStats` per-generation to the callback.
**Why it happens:** The API was designed for real-time observation, not post-hoc collection.
**How to avoid:** Accumulate stats inside the callback using a shared `Arc<Mutex<Vec<GenerationStats>>>` or a `std::cell::RefCell<Vec<_>>` captured via closure. After `run_with_callback` returns, read the accumulated vec and pass to `plot_fitness`.
**Warning signs:** Compiler error about `stats` not being in scope after the run call.

### Pitfall 5: Degenerate Axis Range for Near-Converged Fronts

**What goes wrong:** `build_cartesian_2d(x_min..x_max, y_min..y_max)` panics or produces a blank chart when all Pareto points are clustered (x_min ≈ x_max).
**Why it happens:** plotters panics on zero-width axis ranges.
**How to avoid:** Apply the same `if (max - min).abs() < f64::EPSILON { max = min + 1.0 }` guard used in `compute_y_range`. Write a `compute_pareto_range` helper for both x and y.
**Warning signs:** Runtime panic during chart construction on degenerate populations.

### Pitfall 6: `nsga2_zdt1` run() Returns `ParetoFront`, Not `Vec<GenerationStats>`

**What goes wrong:** Attempting to call `plot_pareto_front_2d` with `front.chromosomes` or similar field that does not exist.
**Why it happens:** `Nsga2Ga::run()` returns a custom `ParetoFront` type with `.individuals: Vec<Individual>`, each with an `.objectives: Vec<f64>` field — not `.fitness_values()`.
**How to avoid:** Extract via `ind.objectives[0]`, `ind.objectives[1]` (confirmed in `nsga2_zdt1.rs` and `spea2_zdt1.rs` by reading the examples).
**Warning signs:** Compile error on field access in `--plot` block.

---

## Code Examples

### Verified: `Circle` element scatter draw pattern

```rust
// Source: docs.rs/plotters/0.3.7/plotters/element/struct.Circle.html [VERIFIED]
chart.draw_series(
    points.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())),
)?;
```

### Verified: `split_evenly` for three-panel layout

```rust
// Source: docs.rs/plotters/0.3.7/plotters/drawing/struct.DrawingArea.html [VERIFIED]
// Returns Vec<DrawingArea<DB, Shift>> with rows*cols elements, row-major order.
let panels = root.split_evenly((1, 3));
// panels[0] = left panel, panels[1] = center, panels[2] = right
```

### Verified: Objective extraction from multi-obj examples

```rust
// Source: reading examples/nsga2_zdt1.rs, spea2_zdt1.rs, sms_emoa_zdt1.rs [VERIFIED: codebase]
// All multi-obj engines return a ParetoFront with .individuals: Vec<Individual>
// Each Individual has .objectives: Vec<f64>
let points: Vec<(f64, f64)> = front.individuals.iter()
    .map(|ind| (ind.objectives[0], ind.objectives[1]))
    .collect();

// For nsga3_dtlz2 (3-obj):
let points: Vec<(f64, f64, f64)> = front.individuals.iter()
    .map(|ind| (ind.objectives[0], ind.objectives[1], ind.objectives[2]))
    .collect();
```

### Verified: `GenerationStats.true_fitness_calls` field

```rust
// Source: reading src/stats.rs line 74 [VERIFIED: codebase]
pub true_fitness_calls: Option<u64>,
```

### Verified: `docs/` directory structure

```
docs/
├── images/          ← NEW (created by examples at first --plot run)
├── ARCHITECTURE.md
├── DEVELOPMENT.md
├── index.md
├── engines.md
└── ... (other .md files)
```

`docs/images/` does not currently exist. [VERIFIED: codebase ls]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ScatterSeries` (old plotters) | `PointSeries<Circle>` or individual `Circle` elements | plotters 0.3.x | `point_series` feature must be enabled |
| Global feature defaults | `default-features = false` with explicit features | plotters 0.3.x | Project already uses this correctly |

**Deprecated/outdated:**
- `BitMapBackend` on WASM: not supported on `wasm32-unknown-unknown`. Must be gated with `#[cfg(not(target_arch = "wasm32"))]`. [VERIFIED: docs.rs plotters — only `CanvasBackend` (via `plotters-canvas`) works on WASM; `bitmap_backend` depends on `plotters-bitmap` which is excluded from WASM target deps]

---

## Open Questions

1. **WASM CI gate for `visualization` feature**
   - What we know: The wasm-check.yml CI runs `cargo check --target wasm32-unknown-unknown --lib` and `--lib --features serde` but NOT `--lib --features visualization`. The current visualization module uses `BitMapBackend` unconditionally in PNG branches — this would fail WASM compilation if tested.
   - What's unclear: Whether the planner should add `cargo check --target wasm32-unknown-unknown --lib --features visualization` to wasm-check.yml, or whether gating `BitMapBackend` with `#[cfg(not(target_arch = "wasm32"))]` is sufficient without updating CI.
   - Recommendation: Add the WASM cfg gates to all PNG dispatch branches (both existing functions and new ones) AND add a `--features visualization` step to wasm-check.yml. Both are needed for ARCH-07 compliance. This is a small wave 0 task.

2. **`plot_true_fitness_calls` behavior when `data.len() == 1`**
   - What we know: `InsufficientData` is returned when `data` is empty (all `None`). The existing `plot_fitness` requires `stats.len() >= 2`.
   - What's unclear: Should single-generation `true_fitness_calls` data (exactly one `Some` value) return `InsufficientData` or render a degenerate one-point line?
   - Recommendation: Apply the same `< 2` guard as `plot_fitness` for consistency. Return `InsufficientData` if `data.len() < 2`.

---

## Environment Availability

Step 2.6: SKIPPED — This phase is purely code changes within the existing Rust project. No external tools, services, or CLIs beyond the standard Rust toolchain are required. The `cargo` toolchain and `plotters` 0.3.7 (already in Cargo.toml) are the only dependencies.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (no `pytest.ini` equivalent) |
| Quick run command | `cargo test --features visualization` |
| Full suite command | `cargo test --features visualization && cargo test --features "visualization,serde"` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | `plot_pareto_front_2d` produces PNG for 2-obj | unit | `cargo test --features visualization test_plot_pareto_front_2d` | Wave 0 |
| SC-1 | `plot_pareto_front_3d` produces PNG for 3-obj | unit | `cargo test --features visualization test_plot_pareto_front_3d` | Wave 0 |
| SC-1 | `plot_true_fitness_calls` produces PNG | unit | `cargo test --features visualization test_plot_true_fitness_calls` | Wave 0 |
| SC-1 | `InsufficientData` on empty/all-None input | unit | `cargo test --features visualization test_pareto_error_cases` | Wave 0 |
| SC-2 | `--plot` produces `docs/images/nsga2_zdt1.png` | smoke/integration | `cargo run --example nsga2_zdt1 --features visualization -- --plot` | examples/nsga2_zdt1.rs (existing, modified) |
| SC-4 | WASM compilation | compile check | `cargo check --target wasm32-unknown-unknown --lib --features visualization` | CI: wasm-check.yml (needs update) |

### Sampling Rate

- **Per task commit:** `cargo test --features visualization`
- **Per wave merge:** `cargo test --features visualization && cargo clippy --features visualization`
- **Phase gate:** Full suite + smoke examples + wasm check before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/visualization_pareto.rs` — covers SC-1 (pareto 2d/3d/true_fitness_calls unit tests)
- [ ] `"point_series"` added to plotters features in Cargo.toml before any test compilation
- [ ] `#[cfg(not(target_arch = "wasm32"))]` gates on PNG branches in ALL visualization functions (including existing `plot_fitness`, `plot_diversity`, `plot_histogram` — they currently lack WASM gates)
- [ ] wasm-check.yml: add `cargo check --target wasm32-unknown-unknown --lib --features visualization`

---

## Security Domain

Step 2.6 security: Not applicable. This phase writes PNG/SVG files to a path argument under `docs/images/`. No network access, no authentication, no user-controlled input beyond the pre-computed `f64` tuples passed by the caller. No ASVS categories apply.

---

## Sources

### Primary (HIGH confidence)
- `src/observe/visualization/mod.rs` — full file read; established patterns confirmed in codebase
- `examples/nsga2_zdt1.rs`, `spea2_zdt1.rs`, `nsga3_dtlz2.rs`, `sms_emoa_zdt1.rs`, `rastrigin.rs` — objective extraction patterns and run() return types confirmed
- `src/stats.rs` lines 1-75 — `GenerationStats.true_fitness_calls: Option<u64>` confirmed
- `Cargo.toml` — plotters features list (missing `point_series`) confirmed
- [docs.rs/plotters/0.3.7 DrawingArea](https://docs.rs/plotters/0.3.7/plotters/drawing/struct.DrawingArea.html) — `split_evenly((rows, cols))` API confirmed
- [docs.rs/plotters/0.3.7 Circle](https://docs.rs/plotters/0.3.7/plotters/element/struct.Circle.html) — `Circle::new(coord, size, style)` confirmed
- [plotters GitHub Cargo.toml](https://github.com/plotters-rs/plotters/blob/master/plotters/Cargo.toml) — `point_series` feature name confirmed; WASM excludes bitmap_backend dep

### Secondary (MEDIUM confidence)
- [plotters-rs/plotters GitHub](https://github.com/plotters-rs/plotters) — WASM support via CanvasBackend only; BitMapBackend not available on wasm32

### Tertiary (LOW confidence)
- None

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Axis label config in `ChartBuilder` uses `.x_desc("f1")` / `.y_desc("f2")` method names | Code Examples | Compile error if method name differs; check docs.rs at implementation time |

**Note:** All other claims were verified by reading source files or official docs.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — plotters 0.3.7 already in Cargo.toml, features confirmed via docs.rs
- Architecture: HIGH — existing module pattern read directly, all example files read directly
- Pitfalls: HIGH — missing `point_series` feature confirmed by reading Cargo.toml; `objectives[]` access confirmed by reading example files; `root.present()` confirmed by existing module

**Research date:** 2026-06-09
**Valid until:** 2026-07-09 (plotters 0.3.x is stable; no fast-moving dependencies)
