use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Non-uniform mutation for Range<T> chromosomes.
///
/// The mutation magnitude decreases over generations, making it suitable for
/// algorithms that need broad exploration early on and fine-tuning later.
///
/// The time-dependent factor `tau = (1 - generation / max_generations)^b`
/// controls how rapidly the mutation amplitude decays:
/// - `b = 1`: linear decay
/// - `b = 2–5`: increasingly aggressive decay (typical values)
///
/// At early generations the operator can make large jumps; near the end of
/// the run it behaves almost like a local search.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `generation` - The current generation number (0-indexed).
/// * `max_generations` - The total number of generations planned.
/// * `b` - The decay exponent controlling how fast mutation shrinks (typically 2–5).
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError::MutationError)` on invalid parameters.
pub fn non_uniform_mutation<T>(
    individual: &mut RangeChromosome<T>,
    generation: usize,
    max_generations: usize,
    b: f64,
) -> Result<(), GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + NonUniformConvertible,
{
    if max_generations == 0 {
        return Err(GaError::MutationError(
            "max_generations must be greater than 0".to_string(),
        ));
    }

    if b < 0.0 {
        return Err(GaError::MutationError(format!(
            "Decay exponent b must be non-negative, got {}",
            b
        )));
    }

    let len = individual.dna().len();
    if len == 0 {
        debug!(target="mutation_events", method="non_uniform"; "Empty DNA, skipping non-uniform mutation");
        return Ok(());
    }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut dna = individual.dna().to_vec();
    let mut gene = dna[idx].clone();

    if gene.ranges.is_empty() {
        debug!(target="mutation_events", method="non_uniform"; "Gene {} has no ranges, skipping", idx);
        return Ok(());
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current_f64 = T::to_f64(gene.value);
    let lo_f64 = T::to_f64(lo);
    let hi_f64 = T::to_f64(hi);

    // Time-dependent factor: decays from 1.0 (generation 0) toward 0.0
    let t_ratio = generation as f64 / max_generations as f64;
    let tau = (1.0 - t_ratio.min(1.0)).powf(b);

    let r: f64 = rng.random_range(0.0_f64..1.0_f64);
    let coin: bool = rng.random_bool(0.5);

    let new_val_f64 = if coin {
        // Mutate toward upper bound
        current_f64 + (hi_f64 - current_f64) * (1.0 - r.powf(tau))
    } else {
        // Mutate toward lower bound
        current_f64 - (current_f64 - lo_f64) * (1.0 - r.powf(tau))
    };

    let clamped = new_val_f64.clamp(lo_f64, hi_f64);
    gene.value = T::from_f64(clamped);
    dna[idx] = gene;

    individual.set_dna(Cow::Owned(dna));

    debug!(
        target="mutation_events", method="non_uniform";
        "Non-uniform mutation applied at gene {} (gen={}/{}, b={}, tau={:.4})",
        idx, generation, max_generations, b, tau
    );
    Ok(())
}

/// Trait for types that can be converted to/from an f64 value (for non-uniform mutation).
///
/// Implementations should do a reasonable conversion (e.g., rounding for integers).
pub trait NonUniformConvertible {
    /// Converts an f64 value to this type.
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to f64.
    fn to_f64(val: Self) -> f64;
}

impl NonUniformConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl NonUniformConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl NonUniformConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl NonUniformConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genotypes::Range as RangeGenotype;
    use std::borrow::Cow;

    fn build_f64_chromosome(n: usize) -> RangeChromosome<f64> {
        let mut c = RangeChromosome::<f64>::new();
        let dna: Vec<_> = (0..n)
            .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], 50.0))
            .collect();
        c.set_dna(Cow::Owned(dna));
        c
    }

    #[test]
    fn non_uniform_mutation_stays_within_range() {
        let mut c = build_f64_chromosome(5);
        for gen in 0..200 {
            non_uniform_mutation(&mut c, gen, 200, 3.0).unwrap();
            for gene in c.dna() {
                let (lo, hi) = gene.ranges[0];
                assert!(
                    gene.value >= lo && gene.value <= hi,
                    "Gene value {} out of range [{}, {}] at generation {}",
                    gene.value,
                    lo,
                    hi,
                    gen
                );
            }
        }
    }

    #[test]
    fn non_uniform_mutation_empty_dna_does_nothing() {
        let mut c = RangeChromosome::<f64>::new();
        let result = non_uniform_mutation(&mut c, 0, 100, 3.0);
        assert!(result.is_ok());
        assert_eq!(c.dna().len(), 0);
    }

    #[test]
    fn non_uniform_mutation_can_change_value() {
        let mut c = build_f64_chromosome(5);
        let mut changed = false;
        for gen in 0..200 {
            let before = c.dna().to_vec();
            non_uniform_mutation(&mut c, gen, 1000, 2.0).unwrap();
            if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
                changed = true;
                break;
            }
        }
        assert!(
            changed,
            "Non-uniform mutation did not change any value after 200 attempts"
        );
    }

    #[test]
    fn non_uniform_mutation_changes_at_most_one_gene() {
        let mut c = build_f64_chromosome(8);
        let before = c.dna().to_vec();
        non_uniform_mutation(&mut c, 10, 100, 3.0).unwrap();
        let diff_count = before
            .iter()
            .zip(c.dna())
            .filter(|(b, a)| b.value != a.value)
            .count();
        assert!(
            diff_count <= 1,
            "More than one gene changed: {}",
            diff_count
        );
    }

    #[test]
    fn non_uniform_mutation_zero_max_generations_returns_error() {
        let mut c = build_f64_chromosome(3);
        let result = non_uniform_mutation(&mut c, 0, 0, 3.0);
        assert!(result.is_err());
    }

    #[test]
    fn non_uniform_mutation_negative_b_returns_error() {
        let mut c = build_f64_chromosome(3);
        let result = non_uniform_mutation(&mut c, 0, 100, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn non_uniform_mutation_late_generation_small_perturbation() {
        let mut c = RangeChromosome::<f64>::new();
        let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
        c.set_dna(Cow::Owned(dna));

        // Near the end of the run, perturbations should be very small
        for _ in 0..100 {
            let before_val = c.dna()[0].value;
            non_uniform_mutation(&mut c, 999, 1000, 5.0).unwrap();
            let after_val = c.dna()[0].value;
            assert!(
                (after_val - before_val).abs() < 50.0,
                "Perturbation {} too large at generation 999/1000 with b=5",
                (after_val - before_val).abs()
            );
        }
    }

    #[test]
    fn non_uniform_mutation_with_i32() {
        let mut c = RangeChromosome::<i32>::new();
        let dna = vec![
            RangeGenotype::new(0, vec![(0, 100)], 50),
            RangeGenotype::new(1, vec![(0, 100)], 50),
        ];
        c.set_dna(Cow::Owned(dna));

        for gen in 0..100 {
            non_uniform_mutation(&mut c, gen, 100, 3.0).unwrap();
            for gene in c.dna() {
                let (lo, hi) = gene.ranges[0];
                assert!(
                    gene.value >= lo && gene.value <= hi,
                    "Gene value {} out of range [{}, {}]",
                    gene.value,
                    lo,
                    hi
                );
            }
        }
    }
}
