#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::{LimitConfiguration, ProblemSolving},
    fitness::FitnessFnWrapper,
    operations::survivor::mu_comma_lambda::mu_comma_lambda,
    traits::ChromosomeT,
};

fn make_chromosome(fitness: f64, age: usize) -> Chromosome {
    Chromosome {
        dna: vec![Gene { id: 0 }],
        fitness,
        age,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    }
}

#[test]
fn test_mu_comma_lambda_only_keeps_offspring() {
    let mut chromosomes = vec![
        make_chromosome(10.0, 3), // parent -- should be discarded
        make_chromosome(5.0, 0),  // offspring
        make_chromosome(8.0, 1),  // parent -- should be discarded
        make_chromosome(3.0, 0),  // offspring
        make_chromosome(7.0, 0),  // offspring
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_comma_lambda(&mut chromosomes, 2, config);

    assert_eq!(chromosomes.len(), 2);
    // Best two offspring by maximization: 7.0, 5.0
    assert_eq!(chromosomes[0].fitness(), 7.0);
    assert_eq!(chromosomes[1].fitness(), 5.0);
    // Verify the parent with fitness 10.0 was discarded
    assert!(chromosomes.iter().all(|c| c.age() == 0));
}

#[test]
fn test_mu_comma_lambda_minimization() {
    let mut chromosomes = vec![
        make_chromosome(1.0, 2),  // parent -- discarded even though best
        make_chromosome(10.0, 0), // offspring
        make_chromosome(4.0, 0),  // offspring
        make_chromosome(6.0, 0),  // offspring
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Minimization,
        ..Default::default()
    };

    mu_comma_lambda(&mut chromosomes, 2, config);

    assert_eq!(chromosomes.len(), 2);
    // Best two offspring by minimization: 4.0, 6.0 (unordered check)
    let mut fitnesses: Vec<f64> = chromosomes.iter().map(|c| c.fitness()).collect();
    fitnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(fitnesses, vec![4.0, 6.0]);
}

#[test]
fn test_mu_comma_lambda_empty_population() {
    let mut chromosomes: Vec<Chromosome> = vec![];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_comma_lambda(&mut chromosomes, 5, config);

    assert!(chromosomes.is_empty());
}

#[test]
fn test_mu_comma_lambda_fewer_offspring_than_target() {
    let mut chromosomes = vec![
        make_chromosome(10.0, 5), // parent -- discarded
        make_chromosome(3.0, 0),  // offspring
        make_chromosome(8.0, 2),  // parent -- discarded
        make_chromosome(6.0, 0),  // offspring
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_comma_lambda(&mut chromosomes, 5, config);

    // Only 2 offspring exist, target is 5 -- keep all offspring
    assert_eq!(chromosomes.len(), 2);
    assert!(chromosomes.iter().all(|c| c.age() == 0));
}

#[test]
fn test_mu_comma_lambda_no_offspring() {
    let mut chromosomes = vec![
        make_chromosome(10.0, 3),
        make_chromosome(5.0, 1),
        make_chromosome(8.0, 2),
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_comma_lambda(&mut chromosomes, 3, config);

    // All are parents -- none survive
    assert!(chromosomes.is_empty());
}
