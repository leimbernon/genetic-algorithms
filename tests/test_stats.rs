use genetic_algorithms::stats::GenerationStats;

// ─── Phase 60 foundation tests ────────────────────────────────────────────────

/// Verifies that from_fitness_values yields None for cache_hits and cache_misses
/// (no cache wired at construction time).
#[test]
fn cache_stats_default_none() {
    let stats = GenerationStats::from_fitness_values(0, &[1.0, 2.0, 3.0], false);
    assert_eq!(stats.cache_hits, None);
    assert_eq!(stats.cache_misses, None);

    // Also check the empty-population path
    let empty_stats = GenerationStats::from_fitness_values(0, &[], false);
    assert_eq!(empty_stats.cache_hits, None);
    assert_eq!(empty_stats.cache_misses, None);
}

/// Verifies that a JSON checkpoint without cache_hits/cache_misses deserializes
/// successfully (serde(default) backward-compat).
#[cfg(feature = "serde")]
#[test]
fn cache_stats_serde_compat_old_checkpoint() {
    let json = r#"{
        "generation": 3,
        "best_fitness": 0.5,
        "worst_fitness": 9.0,
        "avg_fitness": 4.0,
        "fitness_std_dev": 2.5,
        "population_size": 50,
        "diversity": 2.5
    }"#;
    let stats: GenerationStats =
        serde_json::from_str(json).expect("old checkpoint should deserialize");
    assert_eq!(stats.generation, 3);
    assert_eq!(stats.cache_hits, None);
    assert_eq!(stats.cache_misses, None);
    assert_eq!(stats.dynamic_mutation_probability, None);
    assert_eq!(stats.avg_node_count, 0.0);
    assert_eq!(stats.true_fitness_calls, None);
}

#[test]
fn test_stats_from_empty() {
    let stats = GenerationStats::from_fitness_values(0, &[], false);
    assert_eq!(stats.population_size, 0);
    assert_eq!(stats.avg_fitness, 0.0);
    assert_eq!(stats.diversity, 0.0);
}

#[test]
fn test_stats_maximization() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let stats = GenerationStats::from_fitness_values(1, &values, true);
    assert_eq!(stats.generation, 1);
    assert_eq!(stats.best_fitness, 5.0);
    assert_eq!(stats.worst_fitness, 1.0);
    assert!((stats.avg_fitness - 3.0).abs() < 1e-10);
    assert_eq!(stats.population_size, 5);
    assert!(stats.fitness_std_dev > 0.0);
    assert_eq!(stats.diversity, stats.fitness_std_dev);
    assert!(stats.diversity > 0.0);
}

#[test]
fn test_stats_minimization() {
    let values = vec![10.0, 20.0, 30.0];
    let stats = GenerationStats::from_fitness_values(5, &values, false);
    assert_eq!(stats.best_fitness, 10.0);
    assert_eq!(stats.worst_fitness, 30.0);
    assert!((stats.avg_fitness - 20.0).abs() < 1e-10);
    assert_eq!(stats.diversity, stats.fitness_std_dev);
}

#[test]
fn test_stats_single_value() {
    let values = vec![42.0];
    let stats = GenerationStats::from_fitness_values(0, &values, true);
    assert_eq!(stats.best_fitness, 42.0);
    assert_eq!(stats.worst_fitness, 42.0);
    assert_eq!(stats.avg_fitness, 42.0);
    assert_eq!(stats.fitness_std_dev, 0.0);
    assert_eq!(stats.population_size, 1);
    assert_eq!(stats.diversity, 0.0);
}

#[test]
fn test_stats_diversity_equals_std_dev() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let stats = GenerationStats::from_fitness_values(1, &values, true);
    assert_eq!(stats.diversity, stats.fitness_std_dev);
    assert!(stats.diversity > 0.0);
}

#[test]
fn test_stats_diversity_empty() {
    let stats = GenerationStats::from_fitness_values(0, &[], false);
    assert_eq!(stats.diversity, 0.0);
}

#[test]
fn test_stats_diversity_single() {
    let stats = GenerationStats::from_fitness_values(0, &[42.0], true);
    assert_eq!(stats.diversity, 0.0);
}

#[test]
fn test_stats_diversity_uniform_population() {
    let values = vec![5.0, 5.0, 5.0, 5.0];
    let stats = GenerationStats::from_fitness_values(0, &values, true);
    assert_eq!(stats.diversity, 0.0);
}
