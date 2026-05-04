//! Deterministic Crowding survivor selection.
//!
//! Implements a niching strategy where each offspring competes only against the
//! parent it most closely resembles (by genotype), rather than against the entire
//! population. This preserves multiple fitness peaks naturally.
//!
//! **Algorithm:**
//! 1. Partition the population into offspring (`age() == 0`) and parents (`age() > 0`).
//! 2. For each offspring, find the most similar available parent using Hamming
//!    distance on gene IDs (positions where `gene_a.id() != gene_b.id()`).
//!    Comparisons use `min(len_a, len_b)` positions — extra positions are ignored.
//! 3. Keep the fitter of the (offspring, most-similar-parent) pair; discard the other.
//! 4. Offspring that have no available parent to pair with survive unconditionally.

use crate::traits::{ChromosomeT, GeneT};
use log::{debug, trace};

/// Hamming distance between two DNA slices on gene IDs.
///
/// Counts positions where `gene_a.id() != gene_b.id()`, comparing up to
/// `min(len_a, len_b)` positions. Extra positions beyond the shorter slice
/// are ignored (D-08).
fn hamming_distance<U: ChromosomeT>(a: &U, b: &U) -> usize {
    a.dna()
        .iter()
        .zip(b.dna().iter())
        .filter(|(ga, gb)| ga.id() != gb.id())
        .count()
}

/// Deterministic Crowding survivor selection.
///
/// Each offspring competes against its most similar parent (by Hamming distance).
/// The fitter individual of the pair survives; the other is removed. Offspring
/// with no available parent survive unconditionally.
///
/// The population is modified in place. The `population_size` and
/// `limit_configuration` parameters are accepted for API compatibility but the
/// final population size reflects the outcome of the crowding competition.
///
/// # Arguments
///
/// * `chromosomes` - Combined parents + offspring (modified in place).
pub fn deterministic_crowding<U: ChromosomeT>(chromosomes: &mut Vec<U>) {
    debug!(target="survivor_events", method="deterministic_crowding"; "Starting deterministic crowding survivor");

    // Partition indices into offspring (age==0) and parents (age>0).
    let mut offspring_indices: Vec<usize> = Vec::new();
    let mut parent_indices: Vec<usize> = Vec::new();

    for (i, c) in chromosomes.iter().enumerate() {
        if c.age() == 0 {
            offspring_indices.push(i);
        } else {
            parent_indices.push(i);
        }
    }

    trace!(target="survivor_events", method="deterministic_crowding";
        "Offspring: {} | Parents: {}", offspring_indices.len(), parent_indices.len());

    // Track which indices survive.
    // Start with all parents surviving; we'll update as crowding pairs are resolved.
    let total = chromosomes.len();
    let mut survive = vec![true; total];

    // Track which parent indices are still available for pairing.
    let mut available_parents: Vec<usize> = parent_indices.clone();

    for &off_idx in &offspring_indices {
        if available_parents.is_empty() {
            // No parent available — offspring survives unconditionally (D-06).
            trace!(target="survivor_events", method="deterministic_crowding";
                "Offspring {} has no available parent; survives unconditionally", off_idx);
            continue;
        }

        // Find the most similar available parent (minimum Hamming distance).
        let (best_pos, best_parent_idx, _) = available_parents
            .iter()
            .enumerate()
            .map(|(pos, &par_idx)| {
                let dist = hamming_distance(&chromosomes[off_idx], &chromosomes[par_idx]);
                (pos, par_idx, dist)
            })
            .min_by_key(|&(_, _, dist)| dist)
            .expect("available_parents is non-empty");

        // Keep the fitter individual; remove the other.
        let off_fitness = chromosomes[off_idx].fitness();
        let par_fitness = chromosomes[best_parent_idx].fitness();

        if off_fitness >= par_fitness {
            // Offspring wins — remove parent.
            survive[best_parent_idx] = false;
            trace!(target="survivor_events", method="deterministic_crowding";
                "Offspring {} (fit={}) beats parent {} (fit={})", off_idx, off_fitness, best_parent_idx, par_fitness);
        } else {
            // Parent wins — remove offspring.
            survive[off_idx] = false;
            trace!(target="survivor_events", method="deterministic_crowding";
                "Parent {} (fit={}) beats offspring {} (fit={})", best_parent_idx, par_fitness, off_idx, off_fitness);
        }

        // Remove the matched parent from the available pool regardless of winner.
        available_parents.swap_remove(best_pos);
    }

    // Retain only survivors, preserving original order.
    let mut i = 0;
    chromosomes.retain(|_| {
        let keep = survive[i];
        i += 1;
        keep
    });

    debug!(target="survivor_events", method="deterministic_crowding";
        "Deterministic crowding finished: {} survivors", chromosomes.len());
}
