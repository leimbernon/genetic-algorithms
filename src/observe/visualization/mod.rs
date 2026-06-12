//! Visualization — plotting utilities for GA statistics (visualization feature).
//!
//! This module provides chart-generating functions that render [`GenerationStats`]
//! data to PNG or SVG files. It is only available when the `visualization`
//! feature flag is enabled.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`plot_fitness`] | Generate fitness-over-generations chart |
//! | [`plot_diversity`] | Generate diversity-over-generations chart |
//!
//! # When to use
//! Enable the `visualization` feature and call these functions after a GA run
//! to generate charts for post-hoc analysis and debugging.
//!
//! # Example
//!
//! ```ignore
//! use genetic_algorithms::visualization::plot_fitness;
//! use genetic_algorithms::stats::GenerationStats;
//!
//! let stats: Vec<GenerationStats> = /* ... collect from ga.run() ... */ vec![];
//! plot_fitness(&stats, "fitness_chart.png").expect("chart failed");
//! ```

use std::fmt;
use std::path::Path;

use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;

use crate::stats::GenerationStats;

/// Error type for visualization operations.
///
/// Follows the [`crate::error::GaError`] style: plain enum, `Display` impl,
/// `std::error::Error` impl. No `thiserror` macro.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::visualization::VisualizationError;
///
/// let err = VisualizationError::UnsupportedFormat;
/// assert!(err.to_string().contains("png"));
/// ```
#[derive(Debug)]
pub enum VisualizationError {
    /// A plotters drawing backend error (PNG encode, SVG render, file write).
    DrawingError(String),
    /// An I/O error accessing the output path.
    IoError(String),
    /// The file extension is not `.png` or `.svg`.
    UnsupportedFormat,
    /// The input slice has too few data points to produce a meaningful chart.
    InsufficientData,
}

impl fmt::Display for VisualizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisualizationError::DrawingError(msg) => write!(f, "Drawing error: {}", msg),
            VisualizationError::IoError(msg) => write!(f, "I/O error: {}", msg),
            VisualizationError::UnsupportedFormat => {
                write!(f, "Unsupported format: path must end in .png or .svg")
            }
            VisualizationError::InsufficientData => {
                write!(f, "Insufficient data: at least 2 data points required")
            }
        }
    }
}

impl std::error::Error for VisualizationError {}

/// Compute the y-axis range across best, avg, and worst fitness values.
///
/// Returns `(y_min, y_max)` expanded by at least `1.0` when all values are equal,
/// preventing a degenerate (zero-span) axis range.
fn compute_y_range(stats: &[GenerationStats]) -> (f64, f64) {
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for s in stats {
        y_min = y_min
            .min(s.best_fitness)
            .min(s.avg_fitness)
            .min(s.worst_fitness);
        y_max = y_max
            .max(s.best_fitness)
            .max(s.avg_fitness)
            .max(s.worst_fitness);
    }
    if (y_max - y_min).abs() < f64::EPSILON {
        y_max = y_min + 1.0;
    }
    (y_min, y_max)
}

/// Draw the fitness chart body onto `root`.
///
/// Draws three line series (best, avg, worst) with a legend.
/// Errors are returned as `DrawingAreaErrorKind` and must be converted at the
/// call site.
fn draw_fitness_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    stats: &[GenerationStats],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    let max_gen = stats.last().map(|s| s.generation).unwrap_or(0);
    let x_max = if max_gen == 0 { 1 } else { max_gen };
    let (y_min, y_max) = compute_y_range(stats);

    let mut chart = ChartBuilder::on(root)
        .caption("Fitness over Generations", (FontFamily::SansSerif, 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0usize..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Generation")
        .y_desc("Fitness")
        .x_label_style((FontFamily::SansSerif, 12))
        .y_label_style((FontFamily::SansSerif, 12))
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            stats.iter().map(|s| (s.generation, s.best_fitness)),
            &BLUE,
        ))?
        .label("Best")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(LineSeries::new(
            stats.iter().map(|s| (s.generation, s.avg_fitness)),
            &GREEN,
        ))?
        .label("Average")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

    chart
        .draw_series(LineSeries::new(
            stats.iter().map(|s| (s.generation, s.worst_fitness)),
            &RED,
        ))?
        .label("Worst")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font((FontFamily::SansSerif, 12))
        .draw()?;

    Ok(())
}

/// Compute the y-axis range across diversity values.
///
/// Returns `(y_min, y_max)` expanded by at least `1.0` when all values are
/// equal, preventing a degenerate (zero-span) axis range.
fn compute_diversity_range(stats: &[GenerationStats]) -> (f64, f64) {
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for s in stats {
        y_min = y_min.min(s.diversity);
        y_max = y_max.max(s.diversity);
    }
    if (y_max - y_min).abs() < f64::EPSILON {
        y_max = y_min + 1.0;
    }
    (y_min, y_max)
}

/// Draw the diversity chart body onto `root`.
///
/// Draws a single `LineSeries` of diversity values over generations.
fn draw_diversity_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    stats: &[GenerationStats],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    let max_gen = stats.last().map(|s| s.generation).unwrap_or(0);
    let x_max = if max_gen == 0 { 1 } else { max_gen };
    let (y_min, y_max) = compute_diversity_range(stats);

    let mut chart = ChartBuilder::on(root)
        .caption("Population Diversity over Generations", (FontFamily::SansSerif, 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0usize..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Generation")
        .y_desc("Diversity")
        .x_label_style((FontFamily::SansSerif, 12))
        .y_label_style((FontFamily::SansSerif, 12))
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            stats.iter().map(|s| (s.generation, s.diversity)),
            &BLUE,
        ))?
        .label("Diversity")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font((FontFamily::SansSerif, 12))
        .draw()?;

    Ok(())
}

/// Draw the histogram chart body onto `root`.
///
/// Bins the `fitness_values` into `NUM_BINS` equal-width bins and draws
/// a vertical bar chart. Handles the degenerate case where all values are
/// identical (bin_width == 0) by placing all values in bin 0.
fn draw_histogram_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    fitness_values: &[f64],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    const NUM_BINS: u32 = 20;

    let min = fitness_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = fitness_values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let bin_width = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        (max - min) / NUM_BINS as f64
    };

    let max_count = fitness_values.len() as u32;

    let mut chart = ChartBuilder::on(root)
        .caption("Fitness Distribution", (FontFamily::SansSerif, 20))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(
            (0u32..NUM_BINS).into_segmented(),
            0u32..max_count,
        )?;

    chart
        .configure_mesh()
        .x_desc("Fitness bin")
        .y_desc("Count")
        .x_label_formatter(&|v| match v {
            SegmentValue::Exact(b) => {
                let val = min + (*b as f64) * bin_width;
                format!("{:.2}", val)
            }
            SegmentValue::CenterOf(b) => {
                let val = min + (*b as f64 + 0.5) * bin_width;
                format!("{:.2}", val)
            }
            SegmentValue::Last => format!("{:.2}", max),
        })
        .x_label_style((FontFamily::SansSerif, 10))
        .y_label_style((FontFamily::SansSerif, 12))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(BLUE.mix(0.5).filled())
            .margin(1)
            .data(fitness_values.iter().map(|&v| {
                let bin = ((v - min) / bin_width).min((NUM_BINS - 1) as f64).max(0.0) as u32;
                (bin, 1u32)
            })),
    )?;

    Ok(())
}

/// Plot fitness metrics over generations and write to a PNG or SVG file.
///
/// Draws three lines: best fitness (blue), average fitness (green), and
/// worst fitness (red), with a legend. The output format is inferred from the
/// file extension:
/// - `.png` → `BitMapBackend` (raster)
/// - `.svg` → `SVGBackend` (vector)
/// - Anything else → [`VisualizationError::UnsupportedFormat`]
///
/// # Errors
///
/// - [`VisualizationError::InsufficientData`] — if `stats.len() < 2`
/// - [`VisualizationError::UnsupportedFormat`] — if the path extension is not `.png` or `.svg`
/// - [`VisualizationError::DrawingError`] — if the plotters backend fails
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::visualization::plot_fitness;
/// plot_fitness(&stats, "output/fitness.png").unwrap();
/// ```
pub fn plot_fitness(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError> {
    if stats.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
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
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}

/// Plot population diversity over generations and write to a PNG or SVG file.
///
/// Draws a single line of `diversity` values from [`GenerationStats`] over
/// generation indices. The output format is inferred from the file extension:
/// - `.png` → `BitMapBackend` (raster)
/// - `.svg` → `SVGBackend` (vector)
/// - Anything else → [`VisualizationError::UnsupportedFormat`]
///
/// # Errors
///
/// - [`VisualizationError::InsufficientData`] — if `stats.len() < 2`
/// - [`VisualizationError::UnsupportedFormat`] — if the path extension is not `.png` or `.svg`
/// - [`VisualizationError::DrawingError`] — if the plotters backend fails
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::visualization::plot_diversity;
/// plot_diversity(&stats, "output/diversity.png").unwrap();
/// ```
pub fn plot_diversity(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError> {
    if stats.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_diversity_chart(&root, stats)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        Some("svg") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_diversity_chart(&root, stats)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}

/// Plot fitness distribution as a histogram and write to a PNG or SVG file.
///
/// Takes raw `fitness_values` (one entry per individual) and bins them into
/// 20 equal-width bins. Handles the degenerate case where all values are
/// identical without panicking. The output format is inferred from the file
/// extension:
/// - `.png` → `BitMapBackend` (raster)
/// - `.svg` → `SVGBackend` (vector)
/// - Anything else → [`VisualizationError::UnsupportedFormat`]
///
/// # Errors
///
/// - [`VisualizationError::InsufficientData`] — if `fitness_values` is empty
/// - [`VisualizationError::UnsupportedFormat`] — if the path extension is not `.png` or `.svg`
/// - [`VisualizationError::DrawingError`] — if the plotters backend fails
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::visualization::plot_histogram;
/// plot_histogram(&fitness_values, "output/histogram.png").unwrap();
/// ```
pub fn plot_histogram(fitness_values: &[f64], path: &str) -> Result<(), VisualizationError> {
    if fitness_values.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_histogram_chart(&root, fitness_values)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        Some("svg") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_histogram_chart(&root, fitness_values)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}

/// Compute the axis range for a single dimension of a Pareto front.
///
/// Iterates over all values tracking min and max. If the range is degenerate
/// (max ≈ min), expands max by 1.0 to prevent a zero-span axis panic in plotters.
fn compute_pareto_range(iter: impl Iterator<Item = f64>) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in iter {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }
    if (max - min).abs() < f64::EPSILON {
        max = min + 1.0;
    }
    (min, max)
}

/// Draw a 2-D Pareto scatter chart onto `root`.
///
/// Plots each point as a filled `Circle` of radius 3 in blue.
/// Axis labels are `f1` (x) and `f2` (y).
fn draw_pareto_2d_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    points: &[(f64, f64)],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    let (x_min, x_max) = compute_pareto_range(points.iter().map(|p| p.0));
    let (y_min, y_max) = compute_pareto_range(points.iter().map(|p| p.1));

    let mut chart = ChartBuilder::on(root)
        .caption("Pareto Front", (FontFamily::SansSerif, 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc("f1")
        .y_desc("f2")
        .x_label_style((FontFamily::SansSerif, 12))
        .y_label_style((FontFamily::SansSerif, 12))
        .draw()?;

    let mut sorted: Vec<(f64, f64)> = points.to_vec();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    chart.draw_series(LineSeries::new(
        sorted.iter().copied(),
        BLUE.mix(0.3).stroke_width(1),
    ))?;

    chart
        .draw_series(points.iter().map(|&(x, y)| Circle::new((x, y), 3, BLUE.filled())))?
        .label("Pareto front")
        .legend(|(x, y)| Circle::new((x + 10, y), 3, BLUE.filled()));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font((FontFamily::SansSerif, 12))
        .draw()?;

    Ok(())
}

/// Plot a 2-D Pareto front as a scatter chart and write to a PNG or SVG file.
///
/// Each point `(f1, f2)` is rendered as a blue circle. The output format is
/// inferred from the file extension:
/// - `.png` → `BitMapBackend` (raster, 800×600; not available on WASM)
/// - `.svg` → `SVGBackend` (vector, 800×600)
/// - Anything else → [`VisualizationError::UnsupportedFormat`]
///
/// # Errors
///
/// - [`VisualizationError::InsufficientData`] — if `points.len() < 2`
/// - [`VisualizationError::UnsupportedFormat`] — if the path extension is not `.png` or `.svg`,
///   or if called on WASM with `.png`
/// - [`VisualizationError::DrawingError`] — if the plotters backend fails
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::visualization::plot_pareto_front_2d;
/// let points = vec![(0.0_f64, 1.0), (0.5, 0.5), (1.0, 0.0)];
/// plot_pareto_front_2d(&points, "output/pareto2d.png").unwrap();
/// ```
pub fn plot_pareto_front_2d(points: &[(f64, f64)], path: &str) -> Result<(), VisualizationError> {
    if points.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_pareto_2d_chart(&root, points)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        Some("svg") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_pareto_2d_chart(&root, points)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}

/// Draw a 3-D Pareto front as three side-by-side scatter panels onto `root`.
///
/// Splits `root` into three equal panels (left: f1×f2, center: f1×f3, right: f2×f3).
/// Each panel renders blue circles of radius 3 at the appropriate coordinates.
fn draw_pareto_3d_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    points: &[(f64, f64, f64)],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    let panels = root.split_evenly((1, 3));

    // Panel 0: f1 vs f2 (indices 0, 1)
    // Panel 1: f1 vs f3 (indices 0, 2)
    // Panel 2: f2 vs f3 (indices 1, 2)
    let panel_axes: [(usize, usize, &str, &str, &str); 3] = [
        (0, 1, "f1 vs f2", "f1", "f2"),
        (0, 2, "f1 vs f3", "f1", "f3"),
        (1, 2, "f2 vs f3", "f2", "f3"),
    ];

    for i in 0..panels.len() {
        let (xi, yi, title, x_label, y_label) = panel_axes[i];

        let (x_min, x_max) = compute_pareto_range(points.iter().map(|p| match xi {
            0 => p.0,
            1 => p.1,
            _ => p.2,
        }));
        let (y_min, y_max) = compute_pareto_range(points.iter().map(|p| match yi {
            0 => p.0,
            1 => p.1,
            _ => p.2,
        }));

        let mut chart = ChartBuilder::on(&panels[i])
            .caption(title, (FontFamily::SansSerif, 16))
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(55)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc(x_label)
            .y_desc(y_label)
            .x_label_style((FontFamily::SansSerif, 11))
            .y_label_style((FontFamily::SansSerif, 11))
            .draw()?;

        chart.draw_series(points.iter().map(|p| {
            let px = match xi {
                0 => p.0,
                1 => p.1,
                _ => p.2,
            };
            let py = match yi {
                0 => p.0,
                1 => p.1,
                _ => p.2,
            };
            Circle::new((px, py), 3, BLUE.filled())
        }))?;
    }

    Ok(())
}

/// Plot a 3-D Pareto front as three side-by-side scatter panels and write to a PNG or SVG file.
///
/// The 1200×400 canvas is split into three equal panels:
/// - Left: f1 vs f2
/// - Center: f1 vs f3
/// - Right: f2 vs f3
///
/// The output format is inferred from the file extension:
/// - `.png` → `BitMapBackend` (raster, 1200×400; not available on WASM)
/// - `.svg` → `SVGBackend` (vector, 1200×400)
/// - Anything else → [`VisualizationError::UnsupportedFormat`]
///
/// # Errors
///
/// - [`VisualizationError::InsufficientData`] — if `points.len() < 2`
/// - [`VisualizationError::UnsupportedFormat`] — if the path extension is not `.png` or `.svg`,
///   or if called on WASM with `.png`
/// - [`VisualizationError::DrawingError`] — if the plotters backend fails
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::visualization::plot_pareto_front_3d;
/// let points = vec![(0.0_f64, 0.0, 1.0), (0.5, 0.5, 0.5), (1.0, 1.0, 0.0)];
/// plot_pareto_front_3d(&points, "output/pareto3d.png").unwrap();
/// ```
pub fn plot_pareto_front_3d(
    points: &[(f64, f64, f64)],
    path: &str,
) -> Result<(), VisualizationError> {
    if points.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = BitMapBackend::new(path, (1200, 400)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_pareto_3d_chart(&root, points)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        Some("svg") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = SVGBackend::new(path, (1200, 400)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_pareto_3d_chart(&root, points)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}

/// Draw a line chart of true fitness call counts over generations onto `root`.
///
/// Plots a magenta line series of `(generation, true_fitness_calls)` pairs.
fn draw_true_fitness_calls_chart<DB>(
    root: &DrawingArea<DB, Shift>,
    data: &[(usize, u64)],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
    DB::ErrorType: std::error::Error + Send + Sync,
{
    let max_gen = data.iter().map(|&(g, _)| g).max().unwrap_or(0);
    let x_max = if max_gen == 0 { 1 } else { max_gen };
    let (y_min, y_max) = compute_pareto_range(data.iter().map(|&(_, v)| v as f64));

    let mut chart = ChartBuilder::on(root)
        .caption("True Fitness Calls over Generations", (FontFamily::SansSerif, 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0usize..x_max + 1, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Generation")
        .y_desc("True fitness calls")
        .x_label_style((FontFamily::SansSerif, 12))
        .y_label_style((FontFamily::SansSerif, 12))
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|&(g, v)| (g, v as f64)),
            &MAGENTA,
        ))?
        .label("True fitness calls")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], MAGENTA));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font((FontFamily::SansSerif, 12))
        .draw()?;

    Ok(())
}

/// Plot the number of true fitness calls (post-surrogate-prescreening) over
/// generations and write to a PNG or SVG file.
///
/// Only generations where [`GenerationStats::true_fitness_calls`] is `Some` are
/// plotted. Returns [`VisualizationError::InsufficientData`] when fewer than
/// 2 such generations exist.
///
/// The output format is inferred from the file extension:
/// - `.png` → `BitMapBackend` (raster, 800×600; not available on WASM)
/// - `.svg` → `SVGBackend` (vector, 800×600)
/// - Anything else → [`VisualizationError::UnsupportedFormat`]
///
/// # Errors
///
/// - [`VisualizationError::InsufficientData`] — if fewer than 2 stats entries have `true_fitness_calls: Some(_)`
/// - [`VisualizationError::UnsupportedFormat`] — if the path extension is not `.png` or `.svg`,
///   or if called on WASM with `.png`
/// - [`VisualizationError::DrawingError`] — if the plotters backend fails
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::visualization::plot_true_fitness_calls;
/// plot_true_fitness_calls(&stats, "output/true_fitness_calls.png").unwrap();
/// ```
pub fn plot_true_fitness_calls(
    stats: &[GenerationStats],
    path: &str,
) -> Result<(), VisualizationError> {
    let data: Vec<(usize, u64)> = stats
        .iter()
        .filter_map(|s| s.true_fitness_calls.map(|v| (s.generation, v)))
        .collect();

    if data.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_true_fitness_calls_chart(&root, &data)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        Some("svg") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
                root.fill(&WHITE)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                draw_true_fitness_calls_chart(&root, &data)
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
                root.present()
                    .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            }
            #[cfg(target_arch = "wasm32")]
            {
                return Err(VisualizationError::UnsupportedFormat);
            }
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}
