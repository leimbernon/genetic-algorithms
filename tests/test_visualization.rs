//! Integration tests for the visualization module.
//!
//! These tests are gated behind `#![cfg(feature = "visualization")]` and exercise
//! PNG/SVG chart generation and error cases for `plot_fitness`, `plot_diversity`,
//! and `plot_histogram`.
#![cfg(feature = "visualization")]

use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::visualization::{plot_diversity, plot_fitness, plot_histogram, VisualizationError};

fn make_stats(n: usize) -> Vec<GenerationStats> {
    (0..n)
        .map(|i| GenerationStats {
            generation: i,
            best_fitness: 100.0 - i as f64,
            worst_fitness: i as f64,
            avg_fitness: 50.0,
            fitness_std_dev: 10.0,
            population_size: 100,
            diversity: 10.0,
        })
        .collect()
}

#[test]
fn test_plot_fitness_png() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_fitness.png");
    let path_str = path.to_str().unwrap();

    // Clean up any leftover file first
    let _ = std::fs::remove_file(&path);

    let result = plot_fitness(&stats, path_str);
    assert!(result.is_ok(), "plot_fitness PNG failed: {:?}", result.err());
    assert!(path.exists(), "PNG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "PNG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_plot_fitness_svg() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_fitness.svg");
    let path_str = path.to_str().unwrap();

    // Clean up any leftover file first
    let _ = std::fs::remove_file(&path);

    let result = plot_fitness(&stats, path_str);
    assert!(result.is_ok(), "plot_fitness SVG failed: {:?}", result.err());
    assert!(path.exists(), "SVG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "SVG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

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

#[test]
fn test_plot_fitness_insufficient_one() {
    let stats = make_stats(1);
    let path = std::env::temp_dir().join("test_viz_one.png");
    let path_str = path.to_str().unwrap();

    let result = plot_fitness(&stats, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

#[test]
fn test_plot_fitness_unsupported_format() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_fitness.txt");
    let path_str = path.to_str().unwrap();

    let result = plot_fitness(&stats, path_str);
    assert!(
        matches!(result, Err(VisualizationError::UnsupportedFormat)),
        "Expected UnsupportedFormat, got: {:?}",
        result
    );
}

#[test]
fn test_plot_fitness_no_extension() {
    let stats = make_stats(5);
    let path = "output_no_ext";

    let result = plot_fitness(&stats, path);
    assert!(
        matches!(result, Err(VisualizationError::UnsupportedFormat)),
        "Expected UnsupportedFormat, got: {:?}",
        result
    );
}

// ── plot_diversity tests ──────────────────────────────────────────────────────

#[test]
fn test_plot_diversity_png() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_diversity.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_diversity(&stats, path_str);
    assert!(result.is_ok(), "plot_diversity PNG failed: {:?}", result.err());
    assert!(path.exists(), "PNG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "PNG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_plot_diversity_svg() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_diversity.svg");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_diversity(&stats, path_str);
    assert!(result.is_ok(), "plot_diversity SVG failed: {:?}", result.err());
    assert!(path.exists(), "SVG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "SVG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_plot_diversity_insufficient_empty() {
    let stats: Vec<GenerationStats> = vec![];
    let path = std::env::temp_dir().join("test_viz_diversity_empty.png");
    let path_str = path.to_str().unwrap();

    let result = plot_diversity(&stats, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

#[test]
fn test_plot_diversity_insufficient_one() {
    let stats = make_stats(1);
    let path = std::env::temp_dir().join("test_viz_diversity_one.png");
    let path_str = path.to_str().unwrap();

    let result = plot_diversity(&stats, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

// ── plot_histogram tests ──────────────────────────────────────────────────────

#[test]
fn test_plot_histogram_png() {
    let values: Vec<f64> = (1..=20).map(|i| i as f64).collect();
    let path = std::env::temp_dir().join("test_viz_histogram.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_histogram(&values, path_str);
    assert!(result.is_ok(), "plot_histogram PNG failed: {:?}", result.err());
    assert!(path.exists(), "PNG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "PNG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_plot_histogram_svg() {
    let values: Vec<f64> = (1..=20).map(|i| i as f64).collect();
    let path = std::env::temp_dir().join("test_viz_histogram.svg");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_histogram(&values, path_str);
    assert!(result.is_ok(), "plot_histogram SVG failed: {:?}", result.err());
    assert!(path.exists(), "SVG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "SVG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_plot_histogram_empty() {
    let values: Vec<f64> = vec![];
    let path = std::env::temp_dir().join("test_viz_histogram_empty.png");
    let path_str = path.to_str().unwrap();

    let result = plot_histogram(&values, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

#[test]
fn test_plot_histogram_identical_values() {
    let values = vec![5.0f64; 10];
    let path = std::env::temp_dir().join("test_viz_histogram_identical.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    // Must NOT panic even when all values are identical (bin_width == 0 edge case)
    let result = plot_histogram(&values, path_str);
    assert!(result.is_ok(), "plot_histogram identical values failed: {:?}", result.err());
    assert!(path.exists(), "PNG file was not created for identical values");

    let _ = std::fs::remove_file(&path);
}
