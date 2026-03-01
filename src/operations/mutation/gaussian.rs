use crate::chromosomes::Range as RangeChromosome;
use crate::traits::ChromosomeT;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Gaussian mutation for Range<T> chromosomes where T can be converted to/from f64.
///
/// Applies a perturbation drawn from a normal distribution N(0, sigma) to a
/// randomly selected gene. The result is clamped to the gene's declared range.
///
/// This is the standard mutation operator for continuous numerical optimization.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `sigma` - The standard deviation of the Gaussian perturbation.
pub fn gaussian_mutation<T>(individual: &mut RangeChromosome<T>, sigma: f64)
where
    T: Sync
        + Send
        + Clone
        + Default
        + Debug
        + PartialOrd
        + Copy
        + 'static
        + GaussianConvertible,
{
    let len = individual.get_dna().len();
    if len == 0 {
        return;
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..len);

    let mut dna = individual.get_dna().to_vec();
    let mut gene = dna[idx].clone();

    if gene.ranges.is_empty() {
        return;
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current: f64 = T::to_f64(gene.value);
    let lo_f64: f64 = T::to_f64(lo);
    let hi_f64: f64 = T::to_f64(hi);

    // Box-Muller transform for N(0,1) sample
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;
    let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);

    gene.value = T::from_f64(new_val_f64);
    dna[idx] = gene;

    individual.set_dna(Cow::Owned(dna));
}

/// Trait for types that can be converted to/from an f64 value.
///
/// This is needed because Rust's standard library doesn't provide a universal
/// `From<f64>` for numeric types. Implementations should do a reasonable
/// conversion (e.g., rounding for integers).
pub trait GaussianConvertible {
    fn from_f64(val: f64) -> Self;
    fn to_f64(val: Self) -> f64;
}

impl GaussianConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl GaussianConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl GaussianConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl GaussianConvertible for i64 {
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
    fn gaussian_mutation_stays_within_range() {
        let mut c = build_f64_chromosome(5);
        for _ in 0..200 {
            gaussian_mutation(&mut c, 10.0);
            for gene in c.get_dna() {
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
    fn gaussian_mutation_can_change_value() {
        let mut c = build_f64_chromosome(5);
        let mut changed = false;
        for _ in 0..200 {
            let before = c.get_dna().to_vec();
            gaussian_mutation(&mut c, 10.0);
            if before.iter().zip(c.get_dna()).any(|(b, a)| b.value != a.value) {
                changed = true;
                break;
            }
        }
        assert!(changed, "Gaussian mutation did not change any value after 200 attempts");
    }

    #[test]
    fn gaussian_mutation_changes_at_most_one_gene() {
        let mut c = build_f64_chromosome(8);
        let before = c.get_dna().to_vec();
        gaussian_mutation(&mut c, 5.0);
        let diff_count = before
            .iter()
            .zip(c.get_dna())
            .filter(|(b, a)| b.value != a.value)
            .count();
        assert!(diff_count <= 1, "More than one gene changed: {}", diff_count);
    }

    #[test]
    fn gaussian_mutation_empty_dna_does_nothing() {
        let mut c = RangeChromosome::<f64>::new();
        gaussian_mutation(&mut c, 5.0);
        assert_eq!(c.get_dna().len(), 0);
    }

    #[test]
    fn gaussian_mutation_with_i32() {
        let mut c = RangeChromosome::<i32>::new();
        let dna = vec![
            RangeGenotype::new(0, vec![(0, 100)], 50),
            RangeGenotype::new(1, vec![(0, 100)], 50),
        ];
        c.set_dna(Cow::Owned(dna));

        for _ in 0..100 {
            gaussian_mutation(&mut c, 5.0);
            for gene in c.get_dna() {
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
    fn gaussian_mutation_small_sigma_small_perturbation() {
        let mut c = RangeChromosome::<f64>::new();
        let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
        c.set_dna(Cow::Owned(dna));

        // With sigma=0.001, perturbations should be very small
        for _ in 0..100 {
            let before_val = c.get_dna()[0].value;
            gaussian_mutation(&mut c, 0.001);
            let after_val = c.get_dna()[0].value;
            // 6-sigma bound: very unlikely to exceed 0.006
            assert!(
                (after_val - before_val).abs() < 1.0,
                "Perturbation {} too large for sigma=0.001",
                (after_val - before_val).abs()
            );
        }
    }
}




