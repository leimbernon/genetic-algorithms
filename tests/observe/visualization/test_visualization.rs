//! Integration tests for the visualization module.
//!
//! These tests are gated behind `#![cfg(feature = "visualization")]` and exercise
//! PNG/SVG chart generation and error cases for `plot_fitness`, `plot_diversity`,
//! and `plot_histogram`.
#![cfg(feature = "visualization")]

use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::visualization::{
    plot_diversity, plot_fitness, plot_histogram, plot_pareto_front_2d, plot_pareto_front_3d,
    plot_true_fitness_calls, VisualizationError,
};

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
            dynamic_mutation_probability: None,
            avg_node_count: 0.0,
            cache_hits: None,
            cache_misses: None,
            true_fitness_calls: None,
        })
        .collect()
}

fn make_stats_with_surrogate(n: usize) -> Vec<GenerationStats> {
    (0..n)
        .map(|i| GenerationStats {
            generation: i,
            best_fitness: 100.0 - i as f64,
            worst_fitness: i as f64,
            avg_fitness: 50.0,
            fitness_std_dev: 10.0,
            population_size: 100,
            diversity: 10.0,
            dynamic_mutation_probability: None,
            avg_node_count: 0.0,
            cache_hits: None,
            cache_misses: None,
            true_fitness_calls: Some(50 + i as u64),
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
    assert!(
        result.is_ok(),
        "plot_fitness PNG failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "plot_fitness SVG failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "plot_diversity PNG failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "plot_diversity SVG failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "plot_histogram PNG failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "plot_histogram SVG failed: {:?}",
        result.err()
    );
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
    assert!(
        result.is_ok(),
        "plot_histogram identical values failed: {:?}",
        result.err()
    );
    assert!(
        path.exists(),
        "PNG file was not created for identical values"
    );

    let _ = std::fs::remove_file(&path);
}

// ── plot_pareto_front_2d tests ────────────────────────────────────────────────

#[test]
fn test_plot_pareto_front_2d_png() {
    let points: Vec<(f64, f64)> = (0..10)
        .map(|i| (i as f64 * 0.1, 1.0 - i as f64 * 0.1))
        .collect();
    let path = std::env::temp_dir().join("test_viz_pareto2d.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_pareto_front_2d(&points, path_str);
    assert!(
        result.is_ok(),
        "plot_pareto_front_2d PNG failed: {:?}",
        result.err()
    );
    assert!(path.exists(), "PNG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "PNG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_pareto_2d_insufficient_empty() {
    let p: Vec<(f64, f64)> = vec![];
    let path = std::env::temp_dir().join("test_viz_pareto2d_empty.png");
    let path_str = path.to_str().unwrap();

    let result = plot_pareto_front_2d(&p, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

#[test]
fn test_pareto_2d_insufficient_single() {
    let p: Vec<(f64, f64)> = vec![(0.0, 0.0)];
    let path = std::env::temp_dir().join("test_viz_pareto2d_single.png");
    let path_str = path.to_str().unwrap();

    let result = plot_pareto_front_2d(&p, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

// ── plot_pareto_front_3d tests ────────────────────────────────────────────────

#[test]
fn test_plot_pareto_front_3d_png() {
    let points: Vec<(f64, f64, f64)> = (0..10)
        .map(|i| (i as f64 * 0.1, i as f64 * 0.1, 1.0 - i as f64 * 0.1))
        .collect();
    let path = std::env::temp_dir().join("test_viz_pareto3d.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_pareto_front_3d(&points, path_str);
    assert!(
        result.is_ok(),
        "plot_pareto_front_3d PNG failed: {:?}",
        result.err()
    );
    assert!(path.exists(), "PNG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "PNG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_pareto_3d_insufficient_empty() {
    let p: Vec<(f64, f64, f64)> = vec![];
    let path = std::env::temp_dir().join("test_viz_pareto3d_empty.png");
    let path_str = path.to_str().unwrap();

    let result = plot_pareto_front_3d(&p, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}

// ── plot_true_fitness_calls tests ─────────────────────────────────────────────

#[test]
fn test_plot_true_fitness_calls_png() {
    let stats = make_stats_with_surrogate(5);
    let path = std::env::temp_dir().join("test_viz_tfc.png");
    let path_str = path.to_str().unwrap();

    let _ = std::fs::remove_file(&path);

    let result = plot_true_fitness_calls(&stats, path_str);
    assert!(
        result.is_ok(),
        "plot_true_fitness_calls PNG failed: {:?}",
        result.err()
    );
    assert!(path.exists(), "PNG file was not created");
    assert!(
        std::fs::metadata(&path).unwrap().len() > 0,
        "PNG file is empty"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_true_fitness_calls_insufficient_all_none() {
    let stats = make_stats(5);
    let path = std::env::temp_dir().join("test_viz_tfc_none.png");
    let path_str = path.to_str().unwrap();

    let result = plot_true_fitness_calls(&stats, path_str);
    assert!(
        matches!(result, Err(VisualizationError::InsufficientData)),
        "Expected InsufficientData, got: {:?}",
        result
    );
}
