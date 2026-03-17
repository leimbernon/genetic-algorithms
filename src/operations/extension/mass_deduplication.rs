//! MassDeduplication extension strategy.
//!
//! Removes duplicate chromosomes (by gene ID comparison), keeping the one
//! with the best fitness in each group. The GA loop handles regrowing the
//! population back to its target size.

use crate::configuration::ProblemSolving;
use crate::traits::{ChromosomeT, GeneT};
use log::info;
use std::collections::HashSet;

/// Applies mass deduplication: removes chromosomes with duplicate gene-ID sequences.
///
/// For each group of duplicates, the one with the best fitness is kept.
/// The population may be smaller after this operation; the GA loop handles regrowth.
pub fn mass_deduplication<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    problem_solving: ProblemSolving,
) {
    if chromosomes.is_empty() {
        return;
    }

    let original_len = chromosomes.len();

    // Sort by fitness (best first) so when we encounter duplicates,
    // the first one seen is the best.
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

    // Use gene IDs as the deduplication key
    let mut seen: HashSet<Vec<i32>> = HashSet::new();
    chromosomes.retain(|c| {
        let key: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
        seen.insert(key)
    });

    let removed = original_len - chromosomes.len();
    info!(
        target = "extension_events",
        method = "mass_deduplication";
        "MassDeduplication applied: removed {} duplicates, {} unique remain",
        removed,
        chromosomes.len()
    );
}
