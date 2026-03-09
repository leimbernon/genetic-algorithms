pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
use log::{debug, trace};

/// Select survivors using the (mu,lambda) strategy.
///
/// In a (mu,lambda) evolutionary strategy, only the offspring (individuals with
/// `age == 0`) are eligible to survive. All parents are unconditionally discarded.
/// This enforces that every generation is composed entirely of newly created
/// individuals, preventing stagnation from long-lived parents.
///
/// If fewer offspring exist than `population_size`, all offspring are kept
/// (resulting in a temporarily smaller population). The remaining offspring are
/// then ranked by fitness and truncated to `population_size`.
pub fn mu_comma_lambda<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    debug!(target="survivor_events", method="mu_comma_lambda"; "Starting (mu,lambda) survivor selection");

    // Discard all parents -- only offspring (age == 0) survive.
    chromosomes.retain(|c| c.age() == 0);
    trace!(target="survivor_events", method="mu_comma_lambda"; "Offspring count after parent removal: {}", chromosomes.len());

    if chromosomes.len() <= population_size {
        debug!(target="survivor_events", method="mu_comma_lambda"; "(mu,lambda) survivor selection finished (all offspring kept)");
        return;
    }

    // Rank offspring by fitness and truncate.
    if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
        chromosomes.sort_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        let target = limit_configuration.fitness_target.unwrap_or(0.0);
        chromosomes.sort_by(|a, b| {
            b.fitness_distance(&target)
                .partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    if chromosomes.len() > population_size {
        match limit_configuration.problem_solving {
            ProblemSolving::Maximization => {
                chromosomes.truncate(population_size);
            }
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                let excess = chromosomes.len() - population_size;
                chromosomes.drain(0..excess);
            }
        }
    }

    debug!(target="survivor_events", method="mu_comma_lambda"; "(mu,lambda) survivor selection finished");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::genotypes::Binary as BinaryGenotype;
    use std::borrow::Cow;

    fn make_chromosome(fitness: f64, age: usize) -> BinaryChromosome {
        let mut c = BinaryChromosome::new();
        c.set_dna(Cow::Owned(vec![BinaryGenotype { id: 0, value: true }]));
        c.set_fitness(fitness);
        c.set_age(age);
        c
    }

    #[test]
    fn mu_comma_lambda_only_keeps_offspring() {
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
    fn mu_comma_lambda_minimization() {
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
    fn mu_comma_lambda_empty_population() {
        let mut chromosomes: Vec<BinaryChromosome> = vec![];
        let config = LimitConfiguration {
            problem_solving: ProblemSolving::Maximization,
            ..Default::default()
        };

        mu_comma_lambda(&mut chromosomes, 5, config);

        assert!(chromosomes.is_empty());
    }

    #[test]
    fn mu_comma_lambda_fewer_offspring_than_target() {
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
    fn mu_comma_lambda_no_offspring() {
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
}
