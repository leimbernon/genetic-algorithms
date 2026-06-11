//! Length-changing mutation operators for variable-length chromosomes.
//!
//! Provides two operators that alter chromosome length:
//!
//! - [`length_insertion_mutation`] — inserts a new gene (sampled from the existing DNA)
//!   at a random position, growing the chromosome by 1 (clamped to `max`).
//! - [`length_deletion_mutation`] — removes a randomly selected gene, shrinking the
//!   chromosome by 1 (clamped to `min`).
//!
//! Both operators require a [`ChromosomeLength::Variable`](crate::chromosomes::ChromosomeLength)
//! configuration. Calling them when `ChromosomeLength::Fixed` is active returns
//! [`GaError::MutationError`].
//!
//! The inserted gene is cloned from a random position in the existing DNA, preserving
//! the allele distribution of the chromosome. This is consistent with the "random from
//! population allele set" design choice documented in the Phase 52 discussion log.

use crate::chromosomes::ChromosomeLength;
use crate::error::GaError;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::borrow::Cow;

/// Inserts a new gene into the chromosome at a random position, growing its length by 1.
///
/// The new gene is sampled by cloning a randomly selected gene from the existing DNA.
/// This preserves the allele distribution of the chromosome without requiring
/// access to an external allele set.
///
/// If the current length is already at `max`, this is a no-op and `Ok(())` is returned.
/// If the chromosome is empty, a no-op is performed and `Ok(())` is returned.
///
/// # Arguments
///
/// * `individual` — The chromosome to mutate.
/// * `chromosome_length` — Must be `ChromosomeLength::Variable { min, max }`.
///   Returns `Err(GaError::MutationError)` for `ChromosomeLength::Fixed`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::length_mutation::length_insertion_mutation;
/// use genetic_algorithms::chromosomes::{Binary, ChromosomeLength};
/// let mut chromosome = Binary::new();
/// let length = ChromosomeLength::Variable { min: 2, max: 10 };
/// let _ = length_insertion_mutation(&mut chromosome, length);
/// ```
///
/// # Errors
///
/// Returns `Err(GaError::MutationError)` when `chromosome_length` is
/// `ChromosomeLength::Fixed(_)`.
pub fn length_insertion_mutation<U: LinearChromosome>(
    individual: &mut U,
    chromosome_length: ChromosomeLength,
) -> Result<(), GaError> {
    let (min, max) = match chromosome_length {
        ChromosomeLength::Fixed(_) => {
            return Err(GaError::MutationError(
                "Mutation::Insertion requires ChromosomeLength::Variable. \
                 ChromosomeLength::Fixed does not allow changing chromosome length."
                    .to_string(),
            ));
        }
        ChromosomeLength::Variable { min, max } => (min, max),
    };

    let current_len = individual.dna().len();

    if current_len == 0 {
        debug!(target = "mutation_events", method = "length_insertion"; "Empty DNA, skipping insertion");
        return Ok(());
    }

    if current_len >= max {
        debug!(target = "mutation_events", method = "length_insertion";
            "DNA length {} already at max {}, skipping insertion", current_len, max);
        return Ok(());
    }

    let _ = min; // min is used for context/documentation; clamping only needed for deletion

    let mut rng = crate::rng::make_rng();

    // Sample a new gene by cloning a random existing gene
    let source_idx = rng.random_range(0..current_len);
    let new_gene = individual.dna()[source_idx].clone();

    // Choose a random insertion position (0..=current_len)
    let insert_pos = rng.random_range(0..=current_len);

    let mut dna = individual.dna().to_vec();
    dna.insert(insert_pos, new_gene);
    individual.set_dna(Cow::Owned(dna));

    debug!(target = "mutation_events", method = "length_insertion";
        "Inserted gene at position {} (new length: {})", insert_pos, current_len + 1);

    Ok(())
}

/// Removes a randomly selected gene from the chromosome, shrinking its length by 1.
///
/// If the current length is already at `min`, this is a no-op and `Ok(())` is returned.
/// If the chromosome is empty, a no-op is performed and `Ok(())` is returned.
///
/// # Arguments
///
/// * `individual` — The chromosome to mutate.
/// * `chromosome_length` — Must be `ChromosomeLength::Variable { min, max }`.
///   Returns `Err(GaError::MutationError)` for `ChromosomeLength::Fixed`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::length_mutation::length_deletion_mutation;
/// use genetic_algorithms::chromosomes::{Binary, ChromosomeLength};
/// let mut chromosome = Binary::new();
/// let length = ChromosomeLength::Variable { min: 2, max: 10 };
/// let _ = length_deletion_mutation(&mut chromosome, length);
/// ```
///
/// # Errors
///
/// Returns `Err(GaError::MutationError)` when `chromosome_length` is
/// `ChromosomeLength::Fixed(_)`.
pub fn length_deletion_mutation<U: LinearChromosome>(
    individual: &mut U,
    chromosome_length: ChromosomeLength,
) -> Result<(), GaError> {
    let (min, max) = match chromosome_length {
        ChromosomeLength::Fixed(_) => {
            return Err(GaError::MutationError(
                "Mutation::Deletion requires ChromosomeLength::Variable. \
                 ChromosomeLength::Fixed does not allow changing chromosome length."
                    .to_string(),
            ));
        }
        ChromosomeLength::Variable { min, max } => (min, max),
    };

    let current_len = individual.dna().len();

    if current_len == 0 {
        debug!(target = "mutation_events", method = "length_deletion"; "Empty DNA, skipping deletion");
        return Ok(());
    }

    if current_len <= min {
        debug!(target = "mutation_events", method = "length_deletion";
            "DNA length {} already at min {}, skipping deletion", current_len, min);
        return Ok(());
    }

    let _ = max; // max is used for context/documentation; clamping only needed for insertion

    let mut rng = crate::rng::make_rng();

    // Choose a random position to delete
    let delete_pos = rng.random_range(0..current_len);

    let mut dna = individual.dna().to_vec();
    dna.remove(delete_pos);
    individual.set_dna(Cow::Owned(dna));

    debug!(target = "mutation_events", method = "length_deletion";
        "Deleted gene at position {} (new length: {})", delete_pos, current_len - 1);

    Ok(())
}
