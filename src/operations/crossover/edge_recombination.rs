//! Edge Recombination Crossover (ERX) implementation.

use crate::error::GaError;
use crate::traits::{LinearChromosome, GeneT};
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Edge Recombination Crossover (Whitley 1989). Permutation operator that
/// preserves adjacency relationships from both parents. Produces 2 children.
///
/// Algorithm:
/// 1. Build a union adjacency map where each gene ID maps to the set of its
///    neighbours in either parent (circular: first and last genes are adjacent).
/// 2. Starting from a chosen gene, iteratively select the next gene as the
///    unvisited neighbour with the fewest remaining unvisited neighbours.
/// 3. On tie or exhausted neighbour list, fall back to a random unvisited gene
///    (D-06).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::erx;
/// use genetic_algorithms::chromosomes::Binary;
/// let parent1 = Binary::new();
/// let parent2 = Binary::new();
/// let _ = erx(&parent1, &parent2);
/// ```
///
/// # Errors
/// - [`GaError::CrossoverError`] if parent lengths differ.
/// - [`GaError::CrossoverError`] if length < 2 (D-07).
/// - [`GaError::CrossoverError`] if either parent has duplicate gene IDs, or if
///   both parents do not contain the same gene set (D-08).
pub fn erx<U: LinearChromosome>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }
    if len < 2 {
        return Err(GaError::CrossoverError(
            "EdgeRecombination crossover requires DNA of length >= 2".to_string(),
        ));
    }

    // D-08: gene uniqueness on both parents
    let ids_p1: HashSet<i32> = parent_1.dna().iter().map(|g| g.id()).collect();
    if ids_p1.len() != len {
        return Err(GaError::CrossoverError(
            "EdgeRecombination crossover requires unique gene IDs in parent_1 (permutation chromosomes only)".to_string(),
        ));
    }
    let ids_p2: HashSet<i32> = parent_2.dna().iter().map(|g| g.id()).collect();
    if ids_p2.len() != len {
        return Err(GaError::CrossoverError(
            "EdgeRecombination crossover requires unique gene IDs in parent_2 (permutation chromosomes only)".to_string(),
        ));
    }
    if ids_p1 != ids_p2 {
        return Err(GaError::CrossoverError(
            "EdgeRecombination crossover requires both parents to be permutations of the same gene set".to_string(),
        ));
    }

    debug!(target="crossover_events", method="edge_recombination"; "Starting ERX crossover");
    let mut rng = crate::rng::make_rng();

    // Build adjacency map (circular: index wraps at boundaries)
    let mut adj: HashMap<i32, HashSet<i32>> = HashMap::with_capacity(len);
    for g in parent_1.dna() {
        adj.entry(g.id()).or_default();
    }
    for parent_dna in [parent_1.dna(), parent_2.dna()] {
        for i in 0..len {
            let curr = parent_dna[i].id();
            let left = parent_dna[(i + len - 1) % len].id();
            let right = parent_dna[(i + 1) % len].id();
            adj.entry(curr).or_default().insert(left);
            adj.entry(curr).or_default().insert(right);
        }
    }

    // Build a lookup: gene_id -> Gene so we can reconstruct DNA from chosen IDs
    let gene_by_id: HashMap<i32, U::Gene> =
        parent_1.dna().iter().map(|g| (g.id(), g.clone())).collect();

    let all_ids: Vec<i32> = parent_1.dna().iter().map(|g| g.id()).collect();
    let start_1 = parent_1.dna()[0].id();
    let start_2 = parent_2.dna()[0].id();

    let child_ids_1 = erx_build_child(start_1, &mut adj.clone(), &all_ids, &mut rng);
    let child_ids_2 = erx_build_child(start_2, &mut adj, &all_ids, &mut rng);

    let dna_1: Vec<U::Gene> = child_ids_1
        .iter()
        .map(|id| {
            gene_by_id.get(id).cloned().ok_or_else(|| {
                GaError::CrossoverError(format!("ERX: gene id {} not found in parent_1", id))
            })
        })
        .collect::<Result<_, _>>()?;
    let dna_2: Vec<U::Gene> = child_ids_2
        .iter()
        .map(|id| {
            gene_by_id.get(id).cloned().ok_or_else(|| {
                GaError::CrossoverError(format!("ERX: gene id {} not found in parent_1", id))
            })
        })
        .collect::<Result<_, _>>()?;

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(dna_1));
    child_2.set_dna(Cow::Owned(dna_2));

    debug!(target="crossover_events", method="edge_recombination"; "ERX crossover finished");
    Ok(vec![child_1, child_2])
}

/// Builds one ERX child starting from `start`.
///
/// Mutates `adj` in place (removing consumed entries) so the adjacency data
/// is not shared between the two child builds; callers must clone before the
/// first call when the map is needed for a second child.
fn erx_build_child(
    start: i32,
    adj: &mut HashMap<i32, HashSet<i32>>,
    all_ids: &[i32],
    rng: &mut impl Rng,
) -> Vec<i32> {
    let n = all_ids.len();
    let mut child = Vec::with_capacity(n);
    let mut visited: HashSet<i32> = HashSet::with_capacity(n);
    let mut current = start;

    for _ in 0..n {
        child.push(current);
        visited.insert(current);

        // Remove `current` from every neighbour set to keep tie-breaking counts
        // accurate (D-06 prep).
        let neighbors_of_current = adj.remove(&current).unwrap_or_default();
        for nbr_id in &neighbors_of_current {
            if let Some(set) = adj.get_mut(nbr_id) {
                set.remove(&current);
            }
        }

        if visited.len() == n {
            break;
        }

        let unvisited_neighbors: Vec<i32> = neighbors_of_current
            .into_iter()
            .filter(|id| !visited.contains(id))
            .collect();

        current = if unvisited_neighbors.is_empty() {
            // D-06 fallback: pick a random unvisited gene
            let remaining: Vec<i32> = all_ids
                .iter()
                .copied()
                .filter(|id| !visited.contains(id))
                .collect();
            if remaining.is_empty() {
                break;
            }
            remaining[rng.random_range(0..remaining.len())]
        } else {
            // Pick the unvisited neighbour with the fewest remaining unvisited
            // neighbours (smallest adjacency list after filtering); ties favour
            // the first element yielded by the iterator.
            *unvisited_neighbors
                .iter()
                .min_by_key(|id| {
                    adj.get(id)
                        .map(|s| s.iter().filter(|n| !visited.contains(n)).count())
                        .unwrap_or(0)
                })
                .unwrap()
        };
    }
    child
}
