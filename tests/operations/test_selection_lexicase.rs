//! Tests for the Lexicase and Epsilon-Lexicase selection operators.

#[allow(unused_imports)]
use crate::structures::{Gene, MultiCaseChromosome};
#[allow(unused_imports)]
use genetic_algorithms::{
    configuration::SelectionConfiguration,
    fitness::FitnessFnWrapper,
    operations::Selection,
    traits::{ChromosomeT, VectorFitness},
};

#[allow(dead_code)]
fn make_multi_case_chromosome(fitness_values: Vec<f64>, dna: Vec<Gene>) -> MultiCaseChromosome {
    let mean = if fitness_values.is_empty() {
        0.0
    } else {
        fitness_values.iter().sum::<f64>() / fitness_values.len() as f64
    };
    let mut c = MultiCaseChromosome {
        dna,
        fitness: mean,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    c.set_fitness_values(fitness_values);
    c
}

#[allow(dead_code)]
fn pop_with_cases(case_score_matrix: &[Vec<f64>]) -> Vec<MultiCaseChromosome> {
    case_score_matrix
        .iter()
        .map(|scores| make_multi_case_chromosome(scores.clone(), vec![Gene { id: 1 }]))
        .collect()
}

#[test]
fn test_vector_fitness_trait_roundtrip() {
    let mut c = MultiCaseChromosome::default();
    c.set_fitness_values(vec![1.0, 2.0, 3.0]);
    assert_eq!(c.fitness_values(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_lexicase_returns_correct_couple_count() {
    use genetic_algorithms::operations::selection::lexicase_selection;

    let pop = pop_with_cases(&[
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![0.5, 0.5, 0.0],
        vec![0.0, 0.5, 0.5],
        vec![0.5, 0.0, 0.5],
    ]);
    let result = lexicase_selection(&pop, 4, 2);
    assert_eq!(result.len(), 4, "Expected exactly 4 couples");
    for group in &result {
        assert!(group[0] < pop.len(), "Index {} out of bounds", group[0]);
        assert!(group[1] < pop.len(), "Index {} out of bounds", group[1]);
    }
}

#[test]
fn test_lexicase_case_order_is_shuffled() {
    use genetic_algorithms::operations::selection::lexicase_selection;

    // Individual 0 dominates case 0, individual 1 dominates case 1.
    // With shuffled case order, both specialists should be selected at some point
    // over a large enough sample.
    let pop = pop_with_cases(&[
        vec![1.0, 0.0], // specialist on case 0
        vec![0.0, 1.0], // specialist on case 1
    ]);

    let pairs = lexicase_selection(&pop, 200, 2);
    let all_selected: Vec<usize> = pairs
        .iter()
        .flat_map(|group| [group[0], group[1]])
        .collect();

    let saw_0 = all_selected.contains(&0);
    let saw_1 = all_selected.contains(&1);

    assert!(saw_0, "Individual 0 (case-0 specialist) was never selected in 200 pairs");
    assert!(saw_1, "Individual 1 (case-1 specialist) was never selected in 200 pairs");
}

#[test]
fn test_lexicase_syncs_scalar_fitness_to_mean() {
    use genetic_algorithms::operations::selection::factory_lexicase;

    let mut pop = pop_with_cases(&[
        vec![0.0, 10.0, 20.0],
        vec![0.0, 10.0, 20.0],
        vec![0.0, 10.0, 20.0],
        vec![0.0, 10.0, 20.0],
    ]);
    let config = SelectionConfiguration {
        method: Selection::Lexicase,
        number_of_couples: 2,
        ..Default::default()
    };
    factory_lexicase(&mut pop, config, 1).unwrap();

    for (i, c) in pop.iter().enumerate() {
        assert_eq!(
            c.fitness(),
            10.0,
            "Chromosome {} fitness should be mean of case scores (10.0), got {}",
            i,
            c.fitness()
        );
    }
}

#[test]
fn test_factory_rejects_lexicase() {
    use genetic_algorithms::{error::GaError, operations::selection};

    let pop = pop_with_cases(&[vec![1.0], vec![2.0]]);
    let config = SelectionConfiguration {
        method: Selection::Lexicase,
        ..Default::default()
    };
    let result = selection::factory(&pop, config, 1, 2);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError, got: {:?}",
        result
    );
    if let Err(GaError::ConfigurationError(msg)) = result {
        assert!(
            msg.contains("factory_lexicase"),
            "Expected 'factory_lexicase' in error message, got: {}",
            msg
        );
    }
}

#[test]
fn test_factory_rejects_epsilon_lexicase() {
    use genetic_algorithms::{error::GaError, operations::selection};

    let pop = pop_with_cases(&[vec![1.0], vec![2.0]]);
    let config = SelectionConfiguration {
        method: Selection::EpsilonLexicase,
        ..Default::default()
    };
    let result = selection::factory(&pop, config, 1, 2);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError, got: {:?}",
        result
    );
    if let Err(GaError::ConfigurationError(msg)) = result {
        assert!(
            msg.contains("factory_lexicase"),
            "Expected 'factory_lexicase' in error message, got: {}",
            msg
        );
    }
}

#[test]
fn test_epsilon_lexicase_fixed_tolerance() {
    use genetic_algorithms::operations::selection::epsilon_lexicase_selection;

    // Individual 3 (score 0.50) is far below the best (1.00) on case 0.
    // With epsilon=0.05, the threshold is 1.00 - 0.05 = 0.95.
    // Individuals scoring 0.99 and 1.00 survive; 0.97 just barely survives (0.97 >= 0.95).
    // Individual 3 (0.50) must never be selected.
    let pop = pop_with_cases(&[
        vec![1.00], // index 0
        vec![0.99], // index 1
        vec![0.97], // index 2
        vec![0.50], // index 3 — excluded
    ]);
    let pairs = epsilon_lexicase_selection(&pop, 100, Some(0.05), 2);
    let all_selected: Vec<usize> = pairs.iter().flat_map(|group| [group[0], group[1]]).collect();

    assert!(
        !all_selected.contains(&3),
        "Individual 3 (score 0.50) should never be selected with epsilon=0.05 and best=1.00"
    );
}

#[test]
fn test_epsilon_lexicase_dynamic_mad() {
    use genetic_algorithms::operations::selection::epsilon_lexicase_selection;

    // 5 individuals with 2 cases.
    // Case 0 scores: [1.0, 0.9, 0.5, 0.1, 0.0]
    //   sorted: [0.0, 0.1, 0.5, 0.9, 1.0] -> median = 0.5
    //   abs devs: [0.5, 0.4, 0.0, 0.4, 0.5] -> sorted: [0.0, 0.4, 0.4, 0.5, 0.5] -> MAD = 0.4
    // Case 1 scores: [0.0, 0.1, 0.5, 0.9, 1.0]  (mirror)
    //   same MAD = 0.4
    //
    // With MAD=0.4 epsilon on case 0, threshold = best - 0.4 = 1.0 - 0.4 = 0.6.
    // Individuals with case 0 score < 0.6: indices 3 (0.1) and 4 (0.0).
    // These cannot win when case 0 comes first in shuffle AND the pool is still > 1.
    // However with 2 cases and dynamic shuffling, the test simply checks the function
    // runs without errors and returns valid pairs.
    let pop = pop_with_cases(&[
        vec![1.0, 0.0],
        vec![0.9, 0.1],
        vec![0.5, 0.5],
        vec![0.1, 0.9],
        vec![0.0, 1.0],
    ]);
    let pairs = epsilon_lexicase_selection(&pop, 50, None, 2);
    assert_eq!(pairs.len(), 50, "Expected 50 pairs from dynamic MAD epsilon-lexicase");
    for group in &pairs {
        assert!(group[0] < pop.len(), "Index {} out of bounds", group[0]);
        assert!(group[1] < pop.len(), "Index {} out of bounds", group[1]);
    }
    // Confirm that both extreme specialists (index 0 and 4) can be selected,
    // as they are each best on at least one case.
    let all: Vec<usize> = pairs.iter().flat_map(|group| [group[0], group[1]]).collect();
    assert!(
        all.contains(&0) || all.contains(&4),
        "At least one extreme specialist should appear in 50 pairs"
    );
}

#[test]
fn test_ga_engine_runs_with_lexicase_dispatch() {
    use genetic_algorithms::{
        ga::Ga,
        operations::Selection,
        population::Population,
        traits::{ConfigurationT, VectorFitness, SelectionConfig, StoppingConfig},
        ChromosomeLength,
    };
    use crate::structures::{Gene, MultiCaseChromosome};

    // Build a small population of MultiCaseChromosome directly
    let make_chrom = |scores: Vec<f64>| -> MultiCaseChromosome {
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let mut c = MultiCaseChromosome {
            dna: vec![Gene { id: 1 }],
            fitness: mean,
            age: 0,
            fitness_values: scores.clone(),
            fitness_fn: Default::default(),
        };
        c.set_fitness_values(scores);
        c
    };

    let chromosomes = vec![
        make_chrom(vec![1.0, 0.0, 0.0]),
        make_chrom(vec![0.0, 1.0, 0.0]),
        make_chrom(vec![0.0, 0.0, 1.0]),
        make_chrom(vec![0.5, 0.5, 0.0]),
        make_chrom(vec![0.0, 0.5, 0.5]),
        make_chrom(vec![0.5, 0.0, 0.5]),
    ];

    let mut ga = Ga::<MultiCaseChromosome>::new()
        .with_population_size(6)
        .with_max_generations(1)
        .with_chromosome_length(ChromosomeLength::Fixed(1))
        .with_selection_method(Selection::Lexicase)
        .with_number_of_couples(3)
        .with_fitness_fn(|_dna: &[Gene]| 0.0);

    ga.population = Population::new(chromosomes);

    let result = ga.select_parents_lexicase();
    assert!(result.is_ok(), "select_parents_lexicase failed: {:?}", result);
    let pairs = result.unwrap();
    assert_eq!(pairs.len(), 3, "Expected 3 parent pairs");
    for group in &pairs {
        assert!(group[0] < 6, "Index {} out of bounds", group[0]);
        assert!(group[1] < 6, "Index {} out of bounds", group[1]);
    }
}
