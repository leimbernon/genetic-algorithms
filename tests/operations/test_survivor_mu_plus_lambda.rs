#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::{LimitConfiguration, ProblemSolving},
    fitness::FitnessFnWrapper,
    operations::survivor::mu_plus_lambda::mu_plus_lambda,
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
fn test_mu_plus_lambda_keeps_best() {
    let mut chromosomes = vec![
        make_chromosome(1.0, 2),
        make_chromosome(5.0, 0),
        make_chromosome(3.0, 1),
        make_chromosome(4.0, 0),
        make_chromosome(2.0, 3),
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_plus_lambda(&mut chromosomes, 3, config);

    assert_eq!(chromosomes.len(), 3);
    // Best three by maximization: 5.0, 4.0, 3.0
    assert_eq!(chromosomes[0].fitness(), 5.0);
    assert_eq!(chromosomes[1].fitness(), 4.0);
    assert_eq!(chromosomes[2].fitness(), 3.0);
}

#[test]
fn test_mu_plus_lambda_keeps_best_minimization() {
    let mut chromosomes = vec![
        make_chromosome(10.0, 2),
        make_chromosome(1.0, 0),
        make_chromosome(5.0, 1),
        make_chromosome(3.0, 0),
        make_chromosome(7.0, 3),
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Minimization,
        ..Default::default()
    };

    mu_plus_lambda(&mut chromosomes, 2, config);

    assert_eq!(chromosomes.len(), 2);
    // Best two by minimization: 3.0, 1.0 (descending sort, front drained)
    let mut fitnesses: Vec<f64> = chromosomes.iter().map(|c| c.fitness()).collect();
    fitnesses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(fitnesses, vec![1.0, 3.0]);
}

#[test]
fn test_mu_plus_lambda_at_target_size() {
    let mut chromosomes = vec![
        make_chromosome(1.0, 0),
        make_chromosome(2.0, 1),
        make_chromosome(3.0, 2),
    ];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_plus_lambda(&mut chromosomes, 3, config);

    // No truncation needed -- all three survive
    assert_eq!(chromosomes.len(), 3);
}

#[test]
fn test_mu_plus_lambda_empty_population() {
    let mut chromosomes: Vec<Chromosome> = vec![];
    let config = LimitConfiguration {
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    };

    mu_plus_lambda(&mut chromosomes, 5, config);

    assert!(chromosomes.is_empty());
}
