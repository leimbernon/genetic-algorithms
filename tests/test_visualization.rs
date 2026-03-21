//! Integration tests for the visualization module.
//!
//! These tests are gated behind `#![cfg(feature = "visualization")]` and exercise
//! PNG/SVG chart generation and error cases for `plot_fitness`.
#![cfg(feature = "visualization")]

use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::visualization::{plot_fitness, VisualizationError};

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
