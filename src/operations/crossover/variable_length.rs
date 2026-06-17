//! Variable-length crossover implementation.
//!
//! Performs single-point crossover on two chromosomes that may have different
//! DNA lengths. Before recombination the parents are aligned according to the
//! [`AlignmentStrategy`]:
//!
//! - [`AlignmentStrategy::Trim`] — both parents are truncated to `min(len_a, len_b)`.
//! - [`AlignmentStrategy::Pad`] — the shorter parent is padded with genes sampled
//!   from its own DNA until both reach `max(len_a, len_b)`.

use crate::error::GaError;
use crate::operations::AlignmentStrategy;
use crate::traits::LinearChromosome;
use rand::Rng;
use std::borrow::Cow;

/// Variable-length crossover: aligns parents then applies single-point recombination.
///
/// # Arguments
///
/// * `parent_1` — first parent chromosome.
/// * `parent_2` — second parent chromosome.
/// * `strategy` — how to reconcile different-length parents before crossover.
///
/// # Returns
///
/// Two offspring chromosomes. Offspring length equals `min(len_a, len_b)` for
/// `Trim` and `max(len_a, len_b)` for `Pad`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::variable_length_crossover;
/// use genetic_algorithms::operations::AlignmentStrategy;
/// use genetic_algorithms::chromosomes::Binary;
/// let parent1 = Binary::new();
/// let parent2 = Binary::new();
/// let _ = variable_length_crossover(&parent1, &parent2, AlignmentStrategy::Trim);
/// ```
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if both parents are empty.
pub fn variable_length_crossover<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    strategy: AlignmentStrategy,
) -> Result<Vec<U>, GaError> {
    let len_a = parent_1.dna().len();
    let len_b = parent_2.dna().len();

    crate::log_debug!(
        target = "crossover_events",
        method = "variable_length";
        "Starting variable-length crossover (strategy={:?}, len_a={}, len_b={})",
        strategy, len_a, len_b
    );

    let (aligned_a, aligned_b) = match strategy {
        AlignmentStrategy::Trim => {
            let target = len_a.min(len_b);
            let a = parent_1.dna()[..target].to_vec();
            let b = parent_2.dna()[..target].to_vec();
            (a, b)
        }
        AlignmentStrategy::Pad => {
            let target = len_a.max(len_b);
            let mut a = parent_1.dna().to_vec();
            let mut b = parent_2.dna().to_vec();
            let mut rng = crate::rng::make_rng();

            // Pad the shorter chromosome by cloning random genes from its own DNA.
            // This preserves the allele distribution (consistent with Mutation::Insertion).
            while a.len() < target {
                if a.is_empty() {
                    break; // cannot sample from empty DNA — stop padding
                }
                let src = rng.random_range(0..a.len());
                let gene = a[src].clone();
                a.push(gene);
            }
            while b.len() < target {
                if b.is_empty() {
                    break;
                }
                let src = rng.random_range(0..b.len());
                let gene = b[src].clone();
                b.push(gene);
            }
            (a, b)
        }
    };

    let len = aligned_a.len();
    if len == 0 {
        return Err(GaError::CrossoverError(
            "Variable-length crossover: both parents have zero length after alignment".to_string(),
        ));
    }

    let mut rng = crate::rng::make_rng();

    // Crossover point: 1..len (both halves non-empty)
    let point = if len >= 2 {
        rng.random_range(1..len)
    } else {
        0
    };

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    child_dna_1.extend_from_slice(&aligned_a[..point]);
    child_dna_1.extend_from_slice(&aligned_b[point..]);

    child_dna_2.extend_from_slice(&aligned_b[..point]);
    child_dna_2.extend_from_slice(&aligned_a[point..]);

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    crate::log_debug!(
        target = "crossover_events",
        method = "variable_length";
        "Variable-length crossover finished (point={}, offspring_len={})",
        point, len
    );

    Ok(vec![child_1, child_2])
}
