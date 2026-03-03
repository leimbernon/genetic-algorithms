use crate::chromosomes::Range as RangeChromosome;
use crate::traits::ChromosomeT;
use rand::distr::uniform::SampleUniform;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Creep mutation for Range<T> chromosomes.
///
/// Applies a small uniform perturbation to a randomly selected gene.
/// The perturbation is drawn from [-step, +step] and the result is clamped
/// to the gene's declared range.
///
/// This operator is useful for fine-tuning solutions in continuous or
/// integer optimization problems.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `step` - The maximum perturbation magnitude in each direction.
pub fn creep_mutation<T>(individual: &mut RangeChromosome<T>, step: T)
where
    T: Sync
        + Send
        + Clone
        + Default
        + Debug
        + PartialOrd
        + SampleUniform
        + Copy
        + 'static
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>,
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

    // Compute perturbation bounds clamped to the gene's range
    let current = gene.value;
    let perturb_lo = if current - step > lo {
        current - step
    } else {
        lo
    };
    let perturb_hi = if current + step < hi {
        current + step
    } else {
        hi
    };

    let new_val = rng.random_range(perturb_lo..=perturb_hi);
    gene.value = new_val;
    dna[idx] = gene;

    individual.set_dna(Cow::Owned(dna));
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
    fn creep_mutation_stays_within_range() {
        let mut c = build_f64_chromosome(5);
        for _ in 0..100 {
            creep_mutation(&mut c, 5.0);
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
    fn creep_mutation_can_change_value() {
        let mut c = build_f64_chromosome(5);
        let mut changed = false;
        for _ in 0..200 {
            let before = c.get_dna().to_vec();
            creep_mutation(&mut c, 10.0);
            if before
                .iter()
                .zip(c.get_dna())
                .any(|(b, a)| b.value != a.value)
            {
                changed = true;
                break;
            }
        }
        assert!(
            changed,
            "Creep mutation did not change any value after 200 attempts"
        );
    }

    #[test]
    fn creep_mutation_changes_at_most_one_gene() {
        let mut c = build_f64_chromosome(8);
        let before = c.get_dna().to_vec();
        creep_mutation(&mut c, 5.0);
        let diff_count = before
            .iter()
            .zip(c.get_dna())
            .filter(|(b, a)| b.value != a.value)
            .count();
        assert!(
            diff_count <= 1,
            "More than one gene changed: {}",
            diff_count
        );
    }

    #[test]
    fn creep_mutation_respects_step_size() {
        // Set step very small, verify perturbation is small
        let mut c = RangeChromosome::<f64>::new();
        let dna = vec![RangeGenotype::new(0, vec![(0.0, 1000.0)], 500.0)];
        c.set_dna(Cow::Owned(dna));

        for _ in 0..100 {
            let before_val = c.get_dna()[0].value;
            creep_mutation(&mut c, 1.0);
            let after_val = c.get_dna()[0].value;
            assert!(
                (after_val - before_val).abs() <= 1.0 + f64::EPSILON,
                "Perturbation {} exceeded step 1.0",
                (after_val - before_val).abs()
            );
        }
    }

    #[test]
    fn creep_mutation_empty_dna_does_nothing() {
        let mut c = RangeChromosome::<f64>::new();
        creep_mutation(&mut c, 5.0);
        assert_eq!(c.get_dna().len(), 0);
    }
}
