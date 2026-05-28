#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper, operations::selection::boltzmann::boltzmann_selection,
};

fn make_chromosome(fitness: f64) -> Chromosome {
    Chromosome {
        dna: vec![Gene { id: 0 }],
        fitness,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    }
}

#[test]
fn test_boltzmann_selection_produces_correct_number_of_pairs() {
    let pop: Vec<Chromosome> = (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();
    let pairs = boltzmann_selection(&pop, 5, 1.0, 2);
    assert_eq!(pairs.len(), 5);
}

#[test]
fn test_boltzmann_selection_empty_on_small_population() {
    // Single chromosome
    let pop = vec![make_chromosome(10.0)];
    let pairs = boltzmann_selection(&pop, 3, 1.0, 2);
    assert!(pairs.is_empty());

    // Empty population
    let empty: Vec<Chromosome> = Vec::new();
    let pairs = boltzmann_selection(&empty, 3, 1.0, 2);
    assert!(pairs.is_empty());
}

#[test]
fn test_boltzmann_selection_high_temperature_approaches_uniform() {
    // With a very high temperature the Boltzmann distribution should be
    // nearly uniform, so every individual should be selected roughly the
    // same number of times.
    let n = 5;
    let pop: Vec<Chromosome> = (0..n).map(|i| make_chromosome(i as f64 * 100.0)).collect();

    let trials = 2000;
    let couples_per_trial = 50;
    let mut counts = vec![0usize; n];

    for _ in 0..trials {
        let pairs = boltzmann_selection(&pop, couples_per_trial, 1e12, 2);
        for group in &pairs {
            counts[group[0]] += 1;
            counts[group[1]] += 1;
        }
    }

    let total_selections: usize = counts.iter().sum();
    let expected_per_individual = total_selections as f64 / n as f64;

    for (i, &count) in counts.iter().enumerate() {
        let ratio = count as f64 / expected_per_individual;
        assert!(
            (0.8..=1.2).contains(&ratio),
            "Individual {} selected {} times (ratio {:.3}), expected roughly {:.0} — \
             distribution is not uniform enough at high temperature",
            i,
            count,
            ratio,
            expected_per_individual,
        );
    }
}

#[test]
fn test_boltzmann_selection_low_temperature_favors_fittest() {
    let pop: Vec<Chromosome> = (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();

    let mut fittest_count = 0;
    let trials = 500;
    for _ in 0..trials {
        let pairs = boltzmann_selection(&pop, 5, 0.01, 2);
        for group in &pairs {
            if group[0] == 9 {
                fittest_count += 1;
            }
            if group[1] == 9 {
                fittest_count += 1;
            }
        }
    }

    // With very low temperature the fittest individual (index 9) should
    // dominate selections. Total selections = 500 * 10 = 5000.
    assert!(
        fittest_count > 3000,
        "Fittest individual appeared {} times out of 5000, expected > 3000 at low temperature",
        fittest_count,
    );
}

#[test]
fn test_boltzmann_selection_handles_equal_fitness() {
    // When all fitnesses are equal, selection should be effectively uniform
    let pop: Vec<Chromosome> = (0..4).map(|_| make_chromosome(42.0)).collect();
    let pairs = boltzmann_selection(&pop, 3, 1.0, 2);
    assert_eq!(pairs.len(), 3);
    for group in &pairs {
        assert!(group[0] < pop.len());
        assert!(group[1] < pop.len());
    }
}

#[test]
fn test_boltzmann_selection_invalid_temperature_uses_fallback() {
    let pop: Vec<Chromosome> = (0..5).map(|i| make_chromosome(i as f64)).collect();

    // Zero temperature
    let pairs = boltzmann_selection(&pop, 2, 0.0, 2);
    assert_eq!(pairs.len(), 2);

    // Negative temperature
    let pairs = boltzmann_selection(&pop, 2, -5.0, 2);
    assert_eq!(pairs.len(), 2);
}
