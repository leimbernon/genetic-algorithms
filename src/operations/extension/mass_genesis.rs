//! MassGenesis extension strategy.
//!
//! Trims the population to the 2 best chromosomes. The GA loop handles
//! regrowing the population back to its target size.

use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;
use log::info;

/// Applies mass genesis: keeps only the 2 best chromosomes.
///
/// The population will be smaller after this operation; the GA loop handles regrowth.
pub fn mass_genesis<U: ChromosomeT>(chromosomes: &mut Vec<U>, problem_solving: ProblemSolving) {
    if chromosomes.len() <= 2 {
        return;
    }

    // Single-pass O(n): find the indices of the best and second-best chromosomes.
    let is_better = |a: f64, b: f64| -> bool {
        match problem_solving {
            ProblemSolving::Maximization => a > b,
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => a < b,
        }
    };

    let mut best_idx = 0;
    let mut second_idx = 1;
    if is_better(chromosomes[1].fitness(), chromosomes[0].fitness()) {
        best_idx = 1;
        second_idx = 0;
    }

    for i in 2..chromosomes.len() {
        let f = chromosomes[i].fitness();
        if is_better(f, chromosomes[best_idx].fitness()) {
            second_idx = best_idx;
            best_idx = i;
        } else if is_better(f, chromosomes[second_idx].fitness()) {
            second_idx = i;
        }
    }

    // Move best to index 0 and second-best to index 1, then truncate.
    chromosomes.swap(0, best_idx);
    // If second_idx was 0, it was displaced by the swap above — it now lives at best_idx.
    if second_idx == 0 {
        second_idx = best_idx;
    }
    chromosomes.swap(1, second_idx);
    chromosomes.truncate(2);

    info!(
        target = "extension_events",
        method = "mass_genesis";
        "MassGenesis applied: population trimmed to 2 best chromosomes"
    );
}
