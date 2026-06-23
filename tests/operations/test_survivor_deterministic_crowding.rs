//! Tests for the Deterministic Crowding survivor selection operator.

#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::survivor::deterministic_crowding::deterministic_crowding, traits::ChromosomeT,
};

fn gene(id: i32) -> Gene {
    Gene { id }
}

fn make_chromosome(fitness: f64, age: usize, dna: Vec<Gene>) -> Chromosome {
    Chromosome {
        dna,
        fitness,
        age,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    }
}

// ---- Basic correctness ----

#[test]
fn test_dc_empty_population_is_no_op() {
    let mut pop: Vec<Chromosome> = Vec::new();
    deterministic_crowding(&mut pop);
    assert!(pop.is_empty());
}

#[test]
fn test_dc_all_parents_no_offspring_unchanged() {
    // No offspring (age > 0 for all) -> no pairings -> all survive.
    let mut pop = vec![
        make_chromosome(5.0, 1, vec![gene(1), gene(2)]),
        make_chromosome(3.0, 2, vec![gene(3), gene(4)]),
    ];
    deterministic_crowding(&mut pop);
    assert_eq!(pop.len(), 2);
}

#[test]
fn test_dc_all_offspring_no_parents_all_survive_unconditionally() {
    // No parents available -> every offspring survives unconditionally (D-06).
    let mut pop = vec![
        make_chromosome(5.0, 0, vec![gene(1)]),
        make_chromosome(3.0, 0, vec![gene(2)]),
    ];
    deterministic_crowding(&mut pop);
    assert_eq!(pop.len(), 2);
}

// ---- Replacement semantics ----

#[test]
fn test_dc_fitter_offspring_replaces_parent() {
    // Offspring (age=0, fit=10) vs parent (age=1, fit=5): offspring wins.
    // Both have same DNA so Hamming distance = 0.
    let dna = vec![gene(1), gene(2)];
    let mut pop = vec![
        make_chromosome(5.0, 1, dna.clone()),  // parent
        make_chromosome(10.0, 0, dna.clone()), // offspring
    ];
    deterministic_crowding(&mut pop);
    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].fitness(), 10.0, "Fitter offspring should survive");
    assert_eq!(pop[0].age(), 0);
}

#[test]
fn test_dc_fitter_parent_replaces_offspring() {
    // Parent (age=1, fit=10) vs offspring (age=0, fit=5): parent wins.
    let dna = vec![gene(1), gene(2)];
    let mut pop = vec![
        make_chromosome(10.0, 1, dna.clone()), // parent
        make_chromosome(5.0, 0, dna.clone()),  // offspring
    ];
    deterministic_crowding(&mut pop);
    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].fitness(), 10.0, "Fitter parent should survive");
    assert_eq!(pop[0].age(), 1);
}

#[test]
fn test_dc_equal_fitness_keeps_offspring() {
    // Ties resolved by >= condition: offspring wins with equal fitness.
    let dna = vec![gene(1)];
    let mut pop = vec![
        make_chromosome(5.0, 1, dna.clone()), // parent
        make_chromosome(5.0, 0, dna.clone()), // offspring
    ];
    deterministic_crowding(&mut pop);
    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].fitness(), 5.0);
    assert_eq!(
        pop[0].age(),
        0,
        "Offspring should win on tie (>= condition)"
    );
}

// ---- Most-similar parent matching ----

#[test]
fn test_dc_offspring_matches_most_similar_parent_by_hamming() {
    // Two parents and one offspring.
    // Parent A: [1, 2, 3] — Hamming distance from offspring [1, 2, 9] = 1 (position 2 differs)
    // Parent B: [7, 8, 9] — Hamming distance from offspring [1, 2, 9] = 2 (positions 0,1 differ)
    // Most similar parent = A.
    // Offspring fitness=5, parent A fitness=3 -> offspring wins over A.
    // Parent B has no pair -> survives unconditionally.
    let parent_a = make_chromosome(3.0, 1, vec![gene(1), gene(2), gene(3)]);
    let parent_b = make_chromosome(8.0, 2, vec![gene(7), gene(8), gene(9)]);
    let offspring = make_chromosome(5.0, 0, vec![gene(1), gene(2), gene(9)]);

    let mut pop = vec![parent_a, parent_b, offspring];
    deterministic_crowding(&mut pop);

    // Expected survivors: offspring (wins over A) + parent B (unpaired)
    assert_eq!(pop.len(), 2);
    let fitnesses: std::collections::HashSet<i64> =
        pop.iter().map(|c| c.fitness() as i64).collect();
    assert!(fitnesses.contains(&5), "Offspring (5.0) should survive");
    assert!(fitnesses.contains(&8), "Parent B (8.0) should survive");
}

#[test]
fn test_dc_offspring_pairs_with_closest_parent_different_lengths() {
    // DNA lengths differ; comparison uses min(len_a, len_b) (D-08).
    // Parent: [1, 2] (len 2), Offspring: [1, 2, 3] (len 3)
    // Comparison: first 2 positions -> distance = 0 (identical prefix).
    // Offspring fit=10 > parent fit=5 -> offspring survives.
    let parent = make_chromosome(5.0, 1, vec![gene(1), gene(2)]);
    let offspring = make_chromosome(10.0, 0, vec![gene(1), gene(2), gene(3)]);
    let mut pop = vec![parent, offspring];
    deterministic_crowding(&mut pop);
    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].fitness(), 10.0);
}

// ---- Multiple pairs ----

#[test]
fn test_dc_multiple_offspring_each_paired_with_most_similar_parent() {
    // 2 parents, 2 offspring.
    // Parent A [1, 1] fit=3, age=1
    // Parent B [9, 9] fit=7, age=1
    // Offspring X [1, 2] fit=5, age=0  -> closest to A (dist=1 vs dist=2 to B)
    // Offspring Y [9, 8] fit=2, age=0  -> closest to B (dist=1 vs dist=2 to A)
    //
    // X vs A: X wins (5 > 3)
    // Y vs B: B wins (7 > 2)
    // Survivors: X (fit=5) + B (fit=7)
    let parent_a = make_chromosome(3.0, 1, vec![gene(1), gene(1)]);
    let parent_b = make_chromosome(7.0, 1, vec![gene(9), gene(9)]);
    let offspring_x = make_chromosome(5.0, 0, vec![gene(1), gene(2)]);
    let offspring_y = make_chromosome(2.0, 0, vec![gene(9), gene(8)]);

    let mut pop = vec![parent_a, parent_b, offspring_x, offspring_y];
    deterministic_crowding(&mut pop);

    assert_eq!(pop.len(), 2);
    let fitnesses: Vec<f64> = {
        let mut v: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v
    };
    assert_eq!(fitnesses, vec![5.0, 7.0]);
}

// ---- Factory dispatch ----

#[test]
fn test_dc_via_survivor_enum_dispatch() {
    use genetic_algorithms::{
        configuration::{LimitConfiguration, ProblemSolving},
        operations::Survivor,
        traits::SurvivorOperator,
    };

    let dna = vec![gene(1)];
    let mut pop = vec![
        make_chromosome(5.0, 1, dna.clone()),  // parent
        make_chromosome(10.0, 0, dna.clone()), // offspring wins
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    Survivor::DeterministicCrowding
        .select_survivors(&mut pop, 2, config)
        .unwrap();

    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].fitness(), 10.0);
}
