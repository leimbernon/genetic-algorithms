use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Polynomial mutation for Range<T> chromosomes.
///
/// This operator is commonly used in NSGA-II and other multi-objective
/// evolutionary algorithms. It perturbs a randomly selected gene using a
/// polynomial probability distribution controlled by the distribution index
/// `eta_m`.
///
/// - Low `eta_m` (e.g., 1–5): larger perturbations (exploration).
/// - High `eta_m` (e.g., 20–100): smaller perturbations (exploitation).
///
/// The result is always clamped to the gene's declared range.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `eta_m` - The distribution index controlling mutation spread.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError::MutationError)` if `eta_m` is negative.
pub fn polynomial_mutation<T>(
    individual: &mut RangeChromosome<T>,
    eta_m: f64,
) -> Result<(), GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + PolynomialConvertible,
{
    if eta_m < 0.0 {
        return Err(GaError::MutationError(format!(
            "Distribution index eta_m must be non-negative, got {}",
            eta_m
        )));
    }

    let len = individual.dna().len();
    if len == 0 {
        debug!(target="mutation_events", method="polynomial"; "Empty DNA, skipping polynomial mutation");
        return Ok(());
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..len);

    let mut dna = individual.dna().to_vec();
    let mut gene = dna[idx].clone();

    if gene.ranges.is_empty() {
        debug!(target="mutation_events", method="polynomial"; "Gene {} has no ranges, skipping", idx);
        return Ok(());
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current_f64 = T::to_f64(gene.value);
    let lo_f64 = T::to_f64(lo);
    let hi_f64 = T::to_f64(hi);
    let range_span = hi_f64 - lo_f64;

    if range_span <= 0.0 {
        return Ok(());
    }

    // Sample u from (0, 1) — exclude endpoints to avoid domain errors in powf
    let u: f64 = rng.random_range(f64::EPSILON..(1.0 - f64::EPSILON));

    // Compute delta using polynomial distribution
    let delta = if u < 0.5 {
        (2.0 * u).powf(1.0 / (eta_m + 1.0)) - 1.0
    } else {
        1.0 - (2.0 * (1.0 - u)).powf(1.0 / (eta_m + 1.0))
    };

    let new_val_f64 = (current_f64 + delta * range_span).clamp(lo_f64, hi_f64);

    gene.value = T::from_f64(new_val_f64);
    dna[idx] = gene;

    individual.set_dna(Cow::Owned(dna));

    debug!(target="mutation_events", method="polynomial"; "Polynomial mutation applied at gene {} with eta_m={}", idx, eta_m);
    Ok(())
}

/// Trait for types that can be converted to/from an f64 value (for polynomial mutation).
///
/// Implementations should do a reasonable conversion (e.g., rounding for integers).
pub trait PolynomialConvertible {
    /// Converts an f64 value to this type.
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to f64.
    fn to_f64(val: Self) -> f64;
}

impl PolynomialConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl PolynomialConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl PolynomialConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl PolynomialConvertible for i64 {
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
    fn polynomial_mutation_stays_within_range() {
        let mut c = build_f64_chromosome(5);
        for _ in 0..200 {
            polynomial_mutation(&mut c, 20.0).unwrap();
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

    #[test]
    fn polynomial_mutation_empty_dna_does_nothing() {
        let mut c = RangeChromosome::<f64>::new();
        let result = polynomial_mutation(&mut c, 20.0);
        assert!(result.is_ok());
        assert_eq!(c.dna().len(), 0);
    }

    #[test]
    fn polynomial_mutation_can_change_value() {
        let mut c = build_f64_chromosome(5);
        let mut changed = false;
        for _ in 0..200 {
            let before = c.dna().to_vec();
            polynomial_mutation(&mut c, 2.0).unwrap();
            if before.iter().zip(c.dna()).any(|(b, a)| b.value != a.value) {
                changed = true;
                break;
            }
        }
        assert!(
            changed,
            "Polynomial mutation did not change any value after 200 attempts"
        );
    }

    #[test]
    fn polynomial_mutation_changes_at_most_one_gene() {
        let mut c = build_f64_chromosome(8);
        let before = c.dna().to_vec();
        polynomial_mutation(&mut c, 10.0).unwrap();
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
    fn polynomial_mutation_negative_eta_returns_error() {
        let mut c = build_f64_chromosome(3);
        let result = polynomial_mutation(&mut c, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn polynomial_mutation_with_i32() {
        let mut c = RangeChromosome::<i32>::new();
        let dna = vec![
            RangeGenotype::new(0, vec![(0, 100)], 50),
            RangeGenotype::new(1, vec![(0, 100)], 50),
        ];
        c.set_dna(Cow::Owned(dna));

        for _ in 0..100 {
            polynomial_mutation(&mut c, 5.0).unwrap();
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

    #[test]
    fn polynomial_mutation_high_eta_small_perturbation() {
        let mut c = RangeChromosome::<f64>::new();
        let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
        c.set_dna(Cow::Owned(dna));

        // With very high eta, perturbations should be small
        for _ in 0..100 {
            let before_val = c.dna()[0].value;
            polynomial_mutation(&mut c, 200.0).unwrap();
            let after_val = c.dna()[0].value;
            assert!(
                (after_val - before_val).abs() < 100.0,
                "Perturbation {} too large for eta_m=200",
                (after_val - before_val).abs()
            );
        }
    }
}
