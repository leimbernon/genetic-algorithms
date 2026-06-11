---
phase: 63-visualization-pareto-front-plotting-example-images
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - src/observe/visualization/mod.rs
  - tests/observe/visualization/test_visualization.rs
  - .github/workflows/wasm-check.yml
  - examples/nsga2_zdt1.rs
  - examples/spea2_zdt1.rs
  - examples/sms_emoa_zdt1.rs
  - examples/ibea_zdt1.rs
  - examples/nsga3_dtlz2.rs
  - examples/rastrigin.rs
  - Cargo.toml
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 63: Code Review Report

**Reviewed:** 2026-06-10T00:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

This phase delivers visualization utilities (`plot_fitness`, `plot_diversity`, `plot_histogram`, `plot_pareto_front_2d`, `plot_pareto_front_3d`, `plot_true_fitness_calls`) and wires them into five example programs. The module-level logic is mostly sound, but there are two blockers: a logical defect in the x-axis range for line/diversity/fitness charts that causes a zero-span axis crash with single-generation data that passes the `>= 2` guard, and an incorrect return value used in the Das-Dennis reference-point count formula displayed to the user (cosmetic for the formula but exposes an off-by-one). Below these there are four warnings covering dead variables, a WASM gap, non-idiomatic panic patterns, and an error-variant misuse.

---

## Critical Issues

### CR-01: Zero-span x-axis not guarded — plotters panics when all `generation` values are equal

**File:** `src/observe/visualization/mod.rs:106-113`

The `draw_fitness_chart` function builds `0usize..max_gen` as the x-axis range. When `stats` contains two or more entries but every entry has `generation == 0` (which is valid when all stats come from generation 0 in a degenerate run, or when the caller passes repeat-zero data), `max_gen` is `0` and the range `0usize..0` is empty. Plotters does not handle empty cartesian ranges gracefully — it panics with a "degenerate range" assertion. The same defect exists in `draw_diversity_chart` (line 166) and `draw_true_fitness_calls_chart` (line 664). The `compute_y_range` helper correctly widens a degenerate y range, but there is no equivalent protection on the x axis.

The x-axis guard applied to y should also apply to x:

```rust
// current — can produce empty range:
let max_gen = stats.last().map(|s| s.generation).unwrap_or(0);
// ...
.build_cartesian_2d(0usize..max_gen, y_min..y_max)?;

// fix — ensure x range is never empty:
let max_gen = stats.last().map(|s| s.generation).unwrap_or(0);
let x_max = if max_gen == 0 { 1 } else { max_gen };
// ...
.build_cartesian_2d(0usize..x_max, y_min..y_max)?;
```

Apply the same guard in `draw_diversity_chart` (line 166–173) and `draw_true_fitness_calls_chart` (line 664):

```rust
// draw_true_fitness_calls_chart — line 657-664:
let max_gen = data.iter().map(|&(g, _)| g).max().unwrap_or(0);
let x_max = if max_gen == 0 { 1 } else { max_gen };
// ...
.build_cartesian_2d(0usize..x_max + 1, y_min..y_max)?;
```

---

### CR-02: WASM builds with the `visualization` feature will fail — `SVGBackend` writes to the filesystem via `std::fs`

**File:** `src/observe/visualization/mod.rs:279-285`, `335-342`, `394-401`, `502-510`, `631-639`, `731-739`

The WASM CI step in `.github/workflows/wasm-check.yml` (line 35) runs `cargo check --target wasm32-unknown-unknown --lib --features visualization`. The SVG path in all six public functions calls `SVGBackend::new(path, ...)` unconditionally (not gated behind `#[cfg(not(target_arch = "wasm32"))]`). `SVGBackend` ultimately writes to disk via `std::fs::File`, which does not exist on `wasm32-unknown-unknown`. This will produce a compile error in CI and violates the project's mandatory WASM compatibility requirement stated in `CLAUDE.md`.

The PNG path is already correctly gated. The SVG path must receive the same treatment:

```rust
Some("svg") => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
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

Apply to all six functions: `plot_fitness`, `plot_diversity`, `plot_histogram`, `plot_pareto_front_2d`, `plot_pareto_front_3d`, `plot_true_fitness_calls`.

---

## Warnings

### WR-01: `_x_label` and `_y_label` are dead — unused axis label strings allocated on every panel iteration

**File:** `src/observe/visualization/mod.rs:534-538`, `541`

`panel_axes` stores `(&str, &str)` label pairs, but they are destructured as `_x_label` and `_y_label` (underscore-prefixed) and never used. The chart is built with `x_label_area_size(0)` and `y_label_area_size(0)`, so labels cannot appear. This is silent dead code: the data is in the struct and the caller may expect axis labels to display, but they are suppressed and the stored values serve no purpose. Either remove the label fields and the `x_label_area_size(0)` / `y_label_area_size(0)` restrictions and actually draw the labels (which would improve usability), or remove the labels from `panel_axes` entirely:

```rust
// simplified — remove unused label strings:
let panel_axes: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];
for i in 0..panels.len() {
    let (xi, yi) = panel_axes[i];
    // ...
}
```

---

### WR-02: `partial_cmp(...).unwrap()` panics on NaN fitness in examples

**File:** `examples/nsga2_zdt1.rs:151`, `examples/spea2_zdt1.rs:157`, `examples/sms_emoa_zdt1.rs:142`, `examples/ibea_zdt1.rs:142`

All four ZDT1 examples sort the Pareto front with:

```rust
front.individuals.sort_by(|a, b| a.objectives[0].partial_cmp(&b.objectives[0]).unwrap());
```

`partial_cmp` on `f64` returns `None` when either value is `NaN`. `.unwrap()` on `None` panics at runtime. While ZDT1 is unlikely to produce NaN in practice, the fitness formula `g * (1.0 - (x0 / g).sqrt())` involves a square root and a division; if `g` were ever zero or negative through floating-point underflow, `NaN` would propagate and crash the sort. Use `unwrap_or` with a stable fallback:

```rust
front.individuals.sort_by(|a, b| {
    a.objectives[0]
        .partial_cmp(&b.objectives[0])
        .unwrap_or(std::cmp::Ordering::Equal)
});
```

The NSGA-III example (`examples/nsga3_dtlz2.rs:142-146`) already uses this safe pattern — the ZDT1 examples should be updated to match.

---

### WR-03: `plot_histogram` accepts a single value — `InsufficientData` guard inconsistent with other functions

**File:** `src/observe/visualization/mod.rs:373-374`

`plot_fitness`, `plot_diversity`, `plot_pareto_front_2d`, and `plot_pareto_front_3d` all require `len() >= 2`. `plot_histogram` only requires `!is_empty()`, accepting `len() == 1`. A histogram with one data point produces a chart with a single bar in bin 0, which is arguably valid, but the inconsistency is surprising. More critically, the `draw_histogram_chart` function casts `fitness_values.len() as u32` for the y-axis upper bound (line 217); with exactly one value this is `1u32`, which is a valid range. However, the chart's y-axis will always start at 0, and `0u32..1u32` is a valid range, so there is no crash. The warning is about the API contract inconsistency: either document that one value is intentionally supported, or raise the guard to `< 2` to match all other functions.

---

### WR-04: `rastrigin.rs` callback captures stats only when `#[cfg(feature = "visualization")]` but uses `_stats` unconditionally

**File:** `examples/rastrigin.rs:114-128`

The callback parameter is named `_stats` (with underscore) in the non-visualization path, suppressing the unused-variable warning. When `visualization` is enabled, `plot_stats_clone.lock().unwrap().push(_stats.clone())` is invoked (line 124). The `.unwrap()` on the mutex lock will panic if the mutex is poisoned (e.g., if a previous callback panicked). This is a low-probability but real failure path. Use `if let Ok(mut guard) = plot_stats_clone.lock() { guard.push(_stats.clone()); }` or handle the poison error explicitly.

---

## Info

### IN-01: Das-Dennis reference-point count formula has an off-by-one in the display

**File:** `examples/nsga3_dtlz2.rs:133`

The number of Das-Dennis reference points for `M=3` objectives with `p` divisions is `C(p + M - 1, M - 1) = C(p+2, 2)`. The code displays:

```rust
(DAS_DENNIS_P + 2) * (DAS_DENNIS_P + 1) / 2
```

With `DAS_DENNIS_P = 12`, this gives `14 * 13 / 2 = 91`, which is `C(14, 2) = 91`. This is correct for `M=3`. However the formula is hardcoded for 3 objectives — it silently gives a wrong count if `num_objectives` is ever changed without updating `DAS_DENNIS_P`. This is purely informational output; it does not affect the algorithm. Consider extracting it to a helper or a doc comment that makes the `M=3` assumption explicit.

---

### IN-02: `sms_emoa_zdt1.rs` and `ibea_zdt1.rs` doc comments say `--features "visualization,benchmarks"` but only `benchmarks` is actually required for building the example

**File:** `examples/sms_emoa_zdt1.rs:158`, `examples/ibea_zdt1.rs:158`

The inline comments read `// requires --features "visualization,benchmarks"`. The `visualization` feature is gated behind `#[cfg(feature = "visualization")]` and is optional at runtime (the `--plot` flag path). The example binary itself only requires `benchmarks` to compile (as specified in `Cargo.toml` lines 111-114). The comment is misleading for users who want to run without plotting. Change to: `// requires --features benchmarks; add visualization for --plot`.

---

### IN-03: Test file tests PNG output on non-wasm but does not assert file content validity

**File:** `tests/observe/visualization/test_visualization.rs:66-68`

Tests check `path.exists()` and `len() > 0`, but do not assert the file is a valid PNG (magic bytes `\x89PNG`). A malformed file that is non-empty would pass. This is acceptable for a first integration test pass but note that the `SVGBackend` tests have the same gap — an SVG that is truncated or empty (but non-zero byte count due to partial writes) would pass. This is an info-level observation; the existing checks are useful and the gap is minor.

---

_Reviewed: 2026-06-10T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
