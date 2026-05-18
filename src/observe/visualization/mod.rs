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
    let (y_min, y_max) = compute_y_range(stats);

    let mut chart = ChartBuilder::on(root)
        .margin(10)
        .x_label_area_size(0)
        .y_label_area_size(0)
        .build_cartesian_2d(0usize..max_gen, y_min..y_max)?;

    chart.configure_mesh().disable_mesh().draw()?;

    // Best fitness — blue
    chart.draw_series(LineSeries::new(
        stats.iter().map(|s| (s.generation, s.best_fitness)),
        &BLUE,
    ))?;

    // Average fitness — green
    chart.draw_series(LineSeries::new(
        stats.iter().map(|s| (s.generation, s.avg_fitness)),
        &GREEN,
    ))?;

    // Worst fitness — red
    chart.draw_series(LineSeries::new(
        stats.iter().map(|s| (s.generation, s.worst_fitness)),
        &RED,
    ))?;

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
    let (y_min, y_max) = compute_diversity_range(stats);

    let mut chart = ChartBuilder::on(root)
        .margin(10)
        .x_label_area_size(0)
        .y_label_area_size(0)
        .build_cartesian_2d(0usize..max_gen, y_min..y_max)?;

    chart.configure_mesh().disable_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        stats.iter().map(|s| (s.generation, s.diversity)),
        &BLUE,
    ))?;

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

    let mut chart = ChartBuilder::on(root)
        .margin(10)
        .x_label_area_size(0)
        .y_label_area_size(0)
        .build_cartesian_2d(
            (0u32..NUM_BINS).into_segmented(),
            0u32..fitness_values.len() as u32,
        )?;

    chart.configure_mesh().disable_mesh().draw()?;

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
/// # Example
///
/// ```ignore
/// use genetic_algorithms::visualization::plot_fitness;
/// plot_fitness(&stats, "output/fitness.png").unwrap();
/// ```
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
/// # Example
///
/// ```ignore
/// use genetic_algorithms::visualization::plot_diversity;
/// plot_diversity(&stats, "output/diversity.png").unwrap();
/// ```
pub fn plot_diversity(stats: &[GenerationStats], path: &str) -> Result<(), VisualizationError> {
    if stats.len() < 2 {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_diversity_chart(&root, stats)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present()
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        Some("svg") => {
            let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_diversity_chart(&root, stats)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present()
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
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
/// # Example
///
/// ```ignore
/// use genetic_algorithms::visualization::plot_histogram;
/// plot_histogram(&fitness_values, "output/histogram.png").unwrap();
/// ```
pub fn plot_histogram(fitness_values: &[f64], path: &str) -> Result<(), VisualizationError> {
    if fitness_values.is_empty() {
        return Err(VisualizationError::InsufficientData);
    }

    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("png") => {
            let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_histogram_chart(&root, fitness_values)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present()
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        Some("svg") => {
            let root = SVGBackend::new(path, (800, 600)).into_drawing_area();
            root.fill(&WHITE)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            draw_histogram_chart(&root, fitness_values)
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
            root.present()
                .map_err(|e| VisualizationError::DrawingError(format!("{:?}", e)))?;
        }
        _ => return Err(VisualizationError::UnsupportedFormat),
    }

    Ok(())
}
