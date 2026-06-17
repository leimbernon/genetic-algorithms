//! Multi-group PMX (Partially Mapped Crossover) for `MultiUniqueChromosome<T>`.
//!
//! Applies PMX independently within each permutation group defined by
//! `parent.group_ranges()` (a default method on `LinearChromosome` that
//! `MultiUniqueChromosome<T>` overrides). No gene ever crosses a group boundary
//! in the produced children — each group's permutation is recombined in isolation.
//!
//! # Semantics
//!
//! Both parents must share the same group structure (equal `group_ranges()` results).
//! If the groups are empty, an error is returned (no silent no-op — RESEARCH Pitfall 7).

use crate::error::GaError;
use crate::operations::crossover::pmx::pmx_build_child;
use crate::traits::LinearChromosome;
use std::borrow::Cow;

/// Multi-group Partially Mapped Crossover (PMX).
///
/// Applies `pmx_build_child` independently to each group slice defined by
/// `parent_1.group_ranges()`. Returns two children whose DNA is the concatenation
/// of group-local PMX results — no gene migrates across group boundaries.
///
/// # Errors
///
/// - `GaError::ConfigurationError` — if `parent_1.group_ranges()` is empty (chromosome
///   has no groups) or if the two parents have different group structures.
///
/// # Panics
///
/// `pmx_build_child` panics if a gene ID in a parent slice does not appear in the
/// other parent's slice (parents must be permutations within each group).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::multi_group_pmx::multi_group_pmx;
/// use genetic_algorithms::chromosomes::Binary;
/// let parent1 = Binary::new();
/// let parent2 = Binary::new();
/// let _ = multi_group_pmx(&parent1, &parent2);
/// ```
pub fn multi_group_pmx<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
) -> Result<Vec<U>, GaError> {
    let groups = parent_1.group_ranges();

    if groups.is_empty() {
        return Err(GaError::ConfigurationError(
            "MultiUniqueChromosome must have at least one group for multi-group PMX crossover. \
             Check that group alphabets were supplied when constructing the chromosome."
                .to_string(),
        ));
    }

    // Defensive: both parents must have the same group structure (T-48-11).
    let p2_groups = parent_2.group_ranges();
    if groups != p2_groups {
        return Err(GaError::ConfigurationError(format!(
            "MultiGroupPmx: parents have different group structures. \
             Parent 1 groups: {:?}, Parent 2 groups: {:?}",
            groups, p2_groups
        )));
    }

    let p1_dna = parent_1.dna();
    let p2_dna = parent_2.dna();

    let mut child_dna_1 = p1_dna.to_vec();
    let mut child_dna_2 = p2_dna.to_vec();

    for (start, end) in &groups {
        let group_len = end - start;
        // Apply PMX within each group slice; reuse pmx_build_child (pub(crate) since 48-01)
        let slice_1 =
            pmx_build_child(&p1_dna[*start..=*end], &p2_dna[*start..=*end], 0, group_len);
        let slice_2 =
            pmx_build_child(&p2_dna[*start..=*end], &p1_dna[*start..=*end], 0, group_len);
        child_dna_1[*start..=*end].clone_from_slice(&slice_1);
        child_dna_2[*start..=*end].clone_from_slice(&slice_2);
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    Ok(vec![child_1, child_2])
}
