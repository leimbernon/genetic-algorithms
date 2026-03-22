use genetic_algorithms::stats::GenerationStats;

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
