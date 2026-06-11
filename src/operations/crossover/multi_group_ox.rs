//! Multi-group OX (Order Crossover) for `MultiUniqueChromosome<T>`.
//!
//! Applies OX independently within each permutation group defined by
//! `parent.group_ranges()` (a default method on `LinearChromosome` that
//! `MultiUniqueChromosome<T>` overrides). No gene ever crosses a group boundary
//! in the produced children — each group's permutation is recombined in isolation.
//!
//! # Semantics
//!
//! Both parents must share the same group structure (equal `group_ranges()` results).
//! If the groups are empty, an error is returned (no silent no-op — RESEARCH Pitfall 7).
//!
//! Random crossover point pairs are sampled independently per group — ensuring
//! each group-level OX gets a locally valid pair of positions.

use crate::error::GaError;
use crate::operations::crossover::order::ox_build_child;
use crate::traits::LinearChromosome;
use rand::Rng;
use std::borrow::Cow;

/// Multi-group Order Crossover (OX).
///
/// Applies `ox_build_child` independently to each group slice defined by
/// `parent_1.group_ranges()`. For each group, two random crossover positions
/// within the group's length are selected. Returns two children whose DNA is
/// the concatenation of group-local OX results — no gene migrates across group
/// boundaries.
///
/// # Errors
///
/// - `GaError::ConfigurationError` — if `parent_1.group_ranges()` is empty (chromosome
///   has no groups) or if the two parents have different group structures.
///
/// # Group size handling
///
/// - Groups of size 1: no crossover performed (single element copied from parent).
/// - Groups of size 2: positions (0, 1) are used (full segment crossover).
/// - Groups of size >= 3: two distinct positions are sampled randomly.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::multi_group_ox::multi_group_ox;
/// use genetic_algorithms::chromosomes::Binary;
/// let parent1 = Binary::new();
/// let parent2 = Binary::new();
/// let _ = multi_group_ox(&parent1, &parent2);
/// ```
///
/// # Panics
///
/// `ox_build_child` panics if gene IDs are not unique within a group slice.
pub fn multi_group_ox<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
) -> Result<Vec<U>, GaError> {
    let groups = parent_1.group_ranges();

    if groups.is_empty() {
        return Err(GaError::ConfigurationError(
            "MultiUniqueChromosome must have at least one group for multi-group OX crossover. \
             Check that group alphabets were supplied when constructing the chromosome."
                .to_string(),
        ));
    }

    // Defensive: both parents must have the same group structure (T-48-11).
    let p2_groups = parent_2.group_ranges();
    if groups != p2_groups {
        return Err(GaError::ConfigurationError(format!(
            "MultiGroupOx: parents have different group structures. \
             Parent 1 groups: {:?}, Parent 2 groups: {:?}",
            groups, p2_groups
        )));
    }

    let p1_dna = parent_1.dna();
    let p2_dna = parent_2.dna();

    let mut child_dna_1 = p1_dna.to_vec();
    let mut child_dna_2 = p2_dna.to_vec();

    let mut rng = crate::rng::make_rng();

    for (start, end) in &groups {
        let group_len = end - start + 1;

        // For groups of size 1, just copy from parent 1 (nothing to crossover).
        if group_len == 1 {
            // No crossover possible for a single element; parent 1 segment is used as-is.
            // child_dna_1 and child_dna_2 already contain the parent DNA, so no action needed.
            continue;
        }

        // For groups of size 2, the only valid position pair is (0, 1): full segment.
        // For groups of size >= 3, sample two distinct positions randomly.
        let (p1_pos, p2_pos) = if group_len == 2 {
            (0, 1)
        } else {
            let mut pos1 = rng.random_range(0..group_len);
            let mut pos2 = rng.random_range(0..group_len);
            while pos1 == pos2 {
                pos2 = rng.random_range(0..group_len);
            }
            if pos1 > pos2 {
                std::mem::swap(&mut pos1, &mut pos2);
            }
            (pos1, pos2)
        };

        // Apply OX within each group slice; reuse ox_build_child (pub(crate) since 48-01).
        let slice_1 =
            ox_build_child(&p1_dna[*start..=*end], &p2_dna[*start..=*end], p1_pos, p2_pos);
        let slice_2 =
            ox_build_child(&p2_dna[*start..=*end], &p1_dna[*start..=*end], p1_pos, p2_pos);
        child_dna_1[*start..=*end].clone_from_slice(&slice_1);
        child_dna_2[*start..=*end].clone_from_slice(&slice_2);
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    Ok(vec![child_1, child_2])
}
