//! MassExtinction extension strategy.
//!
//! Randomly culls the population to a survival rate, protecting elite individuals.

use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;
use log::info;
use rand::seq::SliceRandom;

/// Applies mass extinction: keeps elite individuals and randomly selects
/// survivors from the rest at the configured survival rate.
///
/// The population may be smaller after this operation; the GA loop handles regrowth.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::extension::mass_extinction::mass_extinction;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::ProblemSolving;
/// let mut population: Vec<Binary> = vec![Binary::new(); 20];
/// mass_extinction(&mut population, 20, ProblemSolving::Maximization, 0.5, 2);
/// ```
pub fn mass_extinction<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    problem_solving: ProblemSolving,
    survival_rate: f64,
    elite_count: usize,
) {
    if chromosomes.is_empty() {
        return;
    }

    let elite_count = elite_count.min(chromosomes.len());
    let target_survivors =
        ((population_size as f64 * survival_rate).ceil() as usize).max(elite_count);

    // Sort by fitness to identify elite
    sort_by_fitness(chromosomes, problem_solving);

    // Extract elite (the first `elite_count` after sorting are the best)
    let elite: Vec<U> = chromosomes.iter().take(elite_count).cloned().collect();
    let mut rest: Vec<U> = chromosomes.drain(elite_count..).collect();
    chromosomes.clear();

    // Randomly select survivors from the rest
    let rest_survivors = target_survivors.saturating_sub(elite_count).min(rest.len());
    let mut rng = crate::rng::make_rng();
    rest.shuffle(&mut rng);
    rest.truncate(rest_survivors);

    // Rebuild population: elite + random survivors
    chromosomes.extend(elite);
    chromosomes.extend(rest);

    info!(
        target = "extension_events",
        method = "mass_extinction";
        "MassExtinction applied: population reduced to {}",
        chromosomes.len()
    );
}

fn sort_by_fitness<U: ChromosomeT>(chromosomes: &mut [U], problem_solving: ProblemSolving) {
    match problem_solving {
        ProblemSolving::Maximization => {
            chromosomes.sort_by(|a, b| {
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
            chromosomes.sort_by(|a, b| {
                a.fitness()
                    .partial_cmp(&b.fitness())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}
