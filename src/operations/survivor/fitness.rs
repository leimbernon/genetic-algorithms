pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
use log::{debug, trace};

pub fn fitness_based<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    debug!(target="survivor_events", method="fitness_based"; "Starting fitness based survivor method");
    if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
        //We sort the chromosomes by their fitness if there is not a fixed fitness problem
        chromosomes.sort_by(|a, b| {
            b.get_fitness()
                .partial_cmp(&a.get_fitness())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        //We sort the chromosomes by their distance with the fitness target in a fixed fitness problem
        let target = limit_configuration.fitness_target.unwrap_or(0.0);
        chromosomes.sort_by(|a, b| {
            b.get_fitness_distance(&target)
                .partial_cmp(&a.get_fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    //If there is more chromosomes than the defined population number
    trace!(target="survivor_events", method="fitness_based"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
    if chromosomes.len() > population_size {
        let chromosomes_to_remove = chromosomes.len() - population_size;

        match limit_configuration.problem_solving {
            ProblemSolving::Maximization => {
                for _i in 0..chromosomes_to_remove {
                    chromosomes.remove(chromosomes.len() - 1);
                }
            }
            ProblemSolving::Minimization => {
                for _i in 0..chromosomes_to_remove {
                    chromosomes.remove(0);
                }
            }
            ProblemSolving::FixedFitness => {
                for _i in 0..chromosomes_to_remove {
                    chromosomes.remove(0);
                }
            }
        }
    }

    debug!(target="survivor_events", method="fitness_based"; "Fitness based survivor method finished");
}
