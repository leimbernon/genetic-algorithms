---
phase: 09-visualization
plan: 01
subsystem: visualization
tags: [plotters, png, svg, charting, feature-flag]

# Dependency graph
requires:
  - phase: 06-stats
    provides: GenerationStats struct with best_fitness, avg_fitness, worst_fitness, diversity fields
provides:
  - visualization feature flag gating plotters 0.3.7 optional dependency
  - src/visualization/mod.rs with VisualizationError enum and plot_fitness function
  - plot_fitness writes PNG or SVG fitness charts from &[GenerationStats]
  - Error variants: DrawingError, IoError, UnsupportedFormat, InsufficientData
affects: [09-02-plan, future VIZ-02 diversity chart, future VIZ-03 histogram]

# Tech tracking
tech-stack:
  added: [plotters 0.3.7 (bitmap_backend, bitmap_encoder, svg_backend, line_series, histogram, ab_glyph)]
  patterns:
    - Optional dep gated on named feature flag (same pattern as serde/checkpoint)
    - cfg-gated pub mod in lib.rs mirrors existing #[cfg(feature = serde)] checkpoint pattern
    - VisualizationError as standalone enum (separate from GaError) — plain enum + Display + Error
    - Generic draw_fitness_chart<DB: DrawingBackend> avoids duplicating chart body across backends
    - Y-axis degenerate range guard: expand by 1.0 when y_max - y_min < EPSILON

key-files:
  created:
    - src/visualization/mod.rs
    - tests/test_visualization.rs
  modified:
    - Cargo.toml
    - src/lib.rs

key-decisions:
  - "plotters features: bitmap_backend + bitmap_encoder required for BitMapBackend::new (file path constructor); bitmap_backend alone only provides with_buffer"
  - "ab_glyph feature added to enable pure-Rust font rendering without system deps; text labels omitted from chart to avoid font registration requirement"
  - "line_series feature required explicitly when default-features = false (LineSeries is not included by default)"
  - "Generic draw_fitness_chart<DB: DrawingBackend> over duplicating chart body in each match arm — chosen for DRY; works with where DB::ErrorType: std::error::Error + Send + Sync"

patterns-established:
  - "Plotters backend dispatch: match path extension -> Some('png') BitMapBackend, Some('svg') SVGBackend, _ UnsupportedFormat"
  - "DrawingAreaErrorKind error conversion: .map_err(|e| VisualizationError::DrawingError(format!(\"{:?}\", e)))"
  - "Feature-gated integration tests: #![cfg(feature = \"visualization\")] at top of tests/test_visualization.rs"

requirements-completed: [VIZ-04, VIZ-01]

# Metrics
duration: 6min
completed: 2026-03-21
---

# Phase 09 Plan 01: Visualization Foundation Summary

**plotters 0.3.7 visualization module with PNG/SVG fitness chart, VisualizationError enum, and feature-gated module isolation**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-21T17:48:52Z
- **Completed:** 2026-03-21T17:54:58Z
- **Tasks:** 1 (TDD: RED + GREEN)
- **Files modified:** 4

## Accomplishments

- Added `visualization` feature flag with plotters 0.3.7 optional dependency to Cargo.toml
- Created `src/visualization/mod.rs` with `VisualizationError` (4 variants) and `plot_fitness` function
- `plot_fitness` dispatches to `BitMapBackend` (PNG) or `SVGBackend` (SVG) by file extension, draws best/avg/worst fitness lines as `LineSeries`
- All 6 tests pass: PNG output, SVG output, InsufficientData (0 entries), InsufficientData (1 entry), UnsupportedFormat (.txt), UnsupportedFormat (no extension)
- `cargo build` without features compiles cleanly — VIZ-04 isolation confirmed

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Failing tests for plot_fitness** - `1dc8d55` (test)
2. **Task 1 GREEN: Implement plot_fitness** - `37670df` (feat)

_Note: TDD task — RED commit first (tests fail, no feature exists), GREEN commit adds implementation._

## Files Created/Modified

- `Cargo.toml` - Added `visualization = ["dep:plotters"]` feature and plotters 0.3.7 optional dep
- `src/lib.rs` - Added `#[cfg(feature = "visualization")] pub mod visualization;`
- `src/visualization/mod.rs` - VisualizationError enum, compute_y_range helper, draw_fitness_chart generic, plot_fitness public function
- `tests/test_visualization.rs` - 6 feature-gated integration tests for PNG/SVG/error cases

## Decisions Made

- **plotters feature set:** `bitmap_backend + bitmap_encoder` required for `BitMapBackend::new(path, dims)` (file-path constructor). `bitmap_backend` alone only provides `with_buffer`. Added `bitmap_encoder` after compile error confirmed this.
- **`line_series` feature required explicitly:** With `default-features = false`, `LineSeries` is not included unless `line_series` feature is enabled.
- **Text labels omitted from chart:** `ab_glyph` (pure-Rust font) added as dep but requires calling `register_font` with static font bytes before drawing text. Rather than bundling a font file, text elements (caption, axis labels, legend) are omitted — chart draws colored lines on white background. This is within "Claude's discretion" per research.
- **Generic `draw_fitness_chart<DB: DrawingBackend>`:** Chosen over duplicating chart body in each match arm. Requires `DB::ErrorType: std::error::Error + Send + Sync` bound.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] plotters feature name correction: `bitmap_encoder` vs `bitmap_backend`**
- **Found during:** Task 1 GREEN (compile error)
- **Issue:** Research specified `features = ["bitmap_encoder", "svg_backend", "histogram"]` but actual plotters feature for enabling `BitMapBackend` type is `bitmap_backend`; `bitmap_encoder` enables PNG file encoding on top of it
- **Fix:** Added both `bitmap_backend` and `bitmap_encoder`; also added `line_series` (required for `LineSeries` with `default-features = false`)
- **Files modified:** Cargo.toml
- **Verification:** `cargo build --features visualization` succeeds
- **Committed in:** 37670df (Task 1 GREEN commit)

**2. [Rule 1 - Bug] Removed text rendering from bitmap chart to avoid font panic**
- **Found during:** Task 1 GREEN (test failure: naive font panics on `draw_text`)
- **Issue:** plotters naive font backend panics when attempting to draw text (`caption`, `x_desc`, `y_desc`, `configure_series_labels`). Without a registered font (requires static font bytes), text drawing fails.
- **Fix:** Removed `.caption()`, `.x_desc()`, `.y_desc()`, and `configure_series_labels()` calls. Chart renders lines and axes without text labels. Added `ab_glyph` feature for future font registration capability.
- **Files modified:** src/visualization/mod.rs
- **Verification:** All 6 tests pass including `test_plot_fitness_png`
- **Committed in:** 37670df (Task 1 GREEN commit)

---

**Total deviations:** 2 auto-fixed (1 blocking — wrong feature names; 1 bug — font panic)
**Impact on plan:** Both fixes necessary for compilation and test passage. Chart lines render correctly; text labels deferred pending font bundling decision. No scope creep.

## Issues Encountered

- Initial `bitmap_encoder` feature name (from research) was incorrect — actual feature is `bitmap_backend` + `bitmap_encoder` as separate features
- SVG backend works without font registration (SVG embeds font-family as CSS), PNG requires actual glyph rendering

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Foundation established: feature flag, module structure, error type, backend dispatch pattern
- Plan 02 can add `plot_diversity` and `plot_histogram` following the same patterns established here
- Font bundling (for axis labels/legend on PNG charts) can be addressed in Plan 02 or a future enhancement

## Self-Check: PASSED

- src/visualization/mod.rs: FOUND
- tests/test_visualization.rs: FOUND
- 09-01-SUMMARY.md: FOUND
- commit 1dc8d55 (RED tests): FOUND
- commit 37670df (GREEN implementation): FOUND
- commit 0a3ebc5 (metadata): FOUND

---
*Phase: 09-visualization*
*Completed: 2026-03-21*
