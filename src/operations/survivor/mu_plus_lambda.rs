//! (mu+lambda) survivor selection strategy.
//!
//! All individuals — both the *mu* parents and the *lambda* offspring —
//! compete together for survival. The best `population_size` individuals
//! are kept based on fitness. This is the standard approach in
//! evolution-strategy literature and is functionally identical to
//! fitness-based selection.

pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
use log::{debug, trace};

/// Select survivors using the (mu+lambda) strategy.
///
/// In a (mu+lambda) evolutionary strategy, all individuals -- both the mu parents
/// and the lambda offspring -- compete together for survival. The best
/// `population_size` individuals are kept based on fitness.
///
/// This is functionally identical to fitness-based survivor selection. The
/// semantic distinction is that (mu+lambda) explicitly frames the competition as
/// "parents + offspring together", which is the convention in evolution-strategy
/// literature.
pub fn mu_plus_lambda<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    debug!(target="survivor_events", method="mu_plus_lambda"; "Starting (mu+lambda) survivor selection");
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

    trace!(target="survivor_events", method="mu_plus_lambda"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
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

    debug!(target="survivor_events", method="mu_plus_lambda"; "(mu+lambda) survivor selection finished");
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
    fn mu_plus_lambda_keeps_best() {
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
    fn mu_plus_lambda_keeps_best_minimization() {
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
    fn mu_plus_lambda_at_target_size() {
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
    fn mu_plus_lambda_empty_population() {
        let mut chromosomes: Vec<BinaryChromosome> = vec![];
        let config = LimitConfiguration {
            problem_solving: ProblemSolving::Maximization,
            ..Default::default()
        };

        mu_plus_lambda(&mut chromosomes, 5, config);

        assert!(chromosomes.is_empty());
    }
}
