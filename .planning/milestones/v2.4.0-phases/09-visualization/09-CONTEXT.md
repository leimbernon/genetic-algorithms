# Phase 9: Visualization - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Add an optional `visualization` feature flag that lets users generate PNG or SVG charts from
`Vec<GenerationStats>` (or `&[GenerationStats]`) produced by a GA run. Three chart types:
fitness-over-generations (VIZ-01), diversity-over-generations (VIZ-02), and a fitness-distribution
histogram for a chosen generation (VIZ-03). All visualization code is absent from the compiled
binary unless the `visualization` feature flag is explicitly enabled (VIZ-04).

This phase does NOT add visualization hooks to the GA execution loop, change `GenerationStats`,
or expose any new public types outside the `visualization` module.

</domain>

<decisions>
## Implementation Decisions

### Charting library
- Use **`plotters`** — pure Rust, supports PNG and SVG natively, no system dependencies
- Registered as an optional dependency gated on the `visualization` feature (same pattern as `serde`)
- No other charting crates considered; `plotters` is the clear choice for a library crate

### API surface
- **Standalone module**: `genetic_algorithms::visualization` — not methods on `Ga`
- Avoids coupling visualization to the `Ga` struct; works equally for island/NSGA2 runs (user collects stats then calls functions)
- Three public functions:
  - `plot_fitness(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError>`
  - `plot_diversity(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError>`
  - `plot_histogram(fitness_values: &[f64], path: &str) -> Result<(), VisualizationError>`
- A new `VisualizationError` enum lives in `src/visualization/` — separate from `GaError` (which is for GA execution errors, not file I/O)
- `VisualizationError` variants at minimum: `DrawingError`, `IoError`, `UnsupportedFormat`, `InsufficientData`

### Format selection
- **Path extension auto-detects format**: `.png` → PNG (BitMapBackend), `.svg` → SVG (SVGBackend)
- Any other extension → `Err(VisualizationError::UnsupportedFormat)`
- Default chart dimensions: **800×600 pixels** (PNG: pixel dimensions; SVG: logical viewport)
- No user-configurable dimensions in this API version

### Fitness chart content (VIZ-01)
- Three lines on one chart: **best fitness**, **average fitness**, **worst fitness**
- Distinct colors per line, legend included
- X-axis: generation number; Y-axis: fitness value
- Chart returns an error if `stats` is empty or has fewer than 2 data points

### Histogram data source (VIZ-03)
- Signature takes **raw `&[f64]` fitness values** — not `&[GenerationStats]`
- User extracts fitness values themselves: `population.iter().map(|c| c.fitness()).collect()`
- Rationale: `GenerationStats` only stores aggregates (avg, std_dev) — using them to approximate a distribution would be misleading; real GA populations are rarely normally distributed
- `plot_histogram` returns an error if the slice is empty

### Claude's Discretion
- Exact `plotters` drawing API calls (series, axis ranges, grid lines, label formatting)
- Legend positioning and color palette for fitness chart lines
- Number of histogram bins (auto or fixed)
- Internal module structure within `src/visualization/` (e.g., separate files per chart type or all in `mod.rs`)
- Whether to add a `pub use visualization::*;` re-export in `src/lib.rs` or require explicit import
- `VisualizationError` Display impl wording

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Visualization — VIZ-01 through VIZ-04

### Existing patterns to follow
- `Cargo.toml` `[features]` and `[dependencies]` sections — follow the `serde` feature flag pattern for the `visualization` optional dep
- `src/lib.rs` line 70–71 — `#[cfg(feature = "serde")] pub mod checkpoint;` pattern; do the same for `visualization`
- `src/stats.rs` — `GenerationStats` struct fields: `generation`, `best_fitness`, `worst_fitness`, `avg_fitness`, `fitness_std_dev`, `diversity`, `population_size`
- `src/error.rs` — Existing `GaError` enum (for reference on error enum style; `VisualizationError` follows same pattern but is a separate type)

### No external design specs
No external ADRs or design docs — requirements and decisions are fully captured above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GenerationStats` (`src/stats.rs`) — the primary input type for VIZ-01 and VIZ-02; fields `generation`, `best_fitness`, `worst_fitness`, `avg_fitness`, `diversity` cover all three line charts
- `GaError` enum (`src/error.rs`) — style reference for `VisualizationError`; do not extend `GaError` itself

### Established Patterns
- Feature flag gating: `#[cfg(feature = "visualization")] pub mod visualization;` in `src/lib.rs`; `visualization = ["dep:plotters"]` in `Cargo.toml [features]`; `plotters = { version = "...", optional = true }` in `[dependencies]`
- Optional deps use `dep:` prefix in feature list (same as `dep:serde`, `dep:serde_json`)
- Modules exposed via `pub mod` in `src/lib.rs`; re-exports in module's own `mod.rs`
- Error types: simple enums with `Display` impl, no `thiserror` macro (not a current dependency)

### Integration Points
- `src/lib.rs` — add `#[cfg(feature = "visualization")] pub mod visualization;`
- `Cargo.toml` — add `plotters` as optional dep, add `visualization = ["dep:plotters"]` feature
- `src/visualization/mod.rs` — new module: public functions `plot_fitness`, `plot_diversity`, `plot_histogram`, and `VisualizationError` type
- No changes to `src/ga.rs`, `src/stats.rs`, or any existing types

</code_context>

<specifics>
## Specific Ideas

- `plot_histogram` takes `&[f64]` not `&[GenerationStats]` — this is deliberate to avoid misleading normal-distribution approximations from aggregates
- Format detection is extension-based (`.png` / `.svg`) with `UnsupportedFormat` error for anything else — no `Format` enum parameter, no separate `plot_*_png` / `plot_*_svg` functions
- All three functions follow identical calling convention: `(data, path) -> Result<(), VisualizationError>`

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 09-visualization*
*Context gathered: 2026-03-21*
