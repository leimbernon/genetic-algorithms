//! MassDeduplication extension strategy.
//!
//! Removes duplicate chromosomes (by gene ID comparison), keeping the one
//! with the best fitness in each group. The GA loop handles regrowing the
//! population back to its target size.

use crate::configuration::ProblemSolving;
use crate::traits::{LinearChromosome, GeneT};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Applies mass deduplication: removes chromosomes with duplicate gene-ID sequences.
///
/// For each group of duplicates, the one with the best fitness is kept.
/// The population may be smaller after this operation; the GA loop handles regrowth.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::extension::mass_deduplication::mass_deduplication;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::ProblemSolving;
/// let mut population: Vec<Binary> = vec![Binary::new(); 20];
/// mass_deduplication(&mut population, ProblemSolving::Maximization);
/// ```
pub fn mass_deduplication<U: LinearChromosome>(
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

    // Use gene IDs as the deduplication key via incremental DefaultHasher.
    // On the common path (no collision), no Vec<i32> is allocated.
    let mut seen: HashMap<u64, Vec<i32>> = HashMap::new();
    chromosomes.retain(|c| {
        // Incremental hash of gene IDs — no Vec<i32> allocation on common path
        let mut hasher = DefaultHasher::new();
        for g in c.dna() {
            g.id().hash(&mut hasher);
        }
        let h = hasher.finish();

        match seen.entry(h) {
            std::collections::hash_map::Entry::Vacant(e) => {
                // First chromosome with this hash — store gene IDs for collision check
                let ids: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
                e.insert(ids);
                true // keep
            }
            std::collections::hash_map::Entry::Occupied(e) => {
                // Hash collision — exact compare gene IDs
                let ids: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
                if ids != *e.get() {
                    true // hash collision but distinct chromosome — keep
                } else {
                    false // true duplicate — remove (best was kept via prior sort)
                }
            }
        }
    });

    let removed = original_len - chromosomes.len();
    crate::log_info!(
        target = "extension_events",
        method = "mass_deduplication";
        "MassDeduplication applied: removed {} duplicates, {} unique remain",
        removed,
        chromosomes.len()
    );
}
