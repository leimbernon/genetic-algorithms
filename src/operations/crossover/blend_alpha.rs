use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Blend Crossover (BLX-α) for Range<T> chromosomes.
///
/// Generates offspring values uniformly within an extended interval around the
/// parents' values. For each gene, if parents have values `p1` and `p2`:
///
/// ```text
/// d = |p1 - p2|
/// child ∈ [min(p1,p2) - α*d, max(p1,p2) + α*d]
/// ```
///
/// The parameter `alpha` controls the amount of exploration:
/// - α = 0: children are strictly between parents.
/// - α = 0.5: standard BLX-α, extends 50% beyond parents.
/// - α > 0.5: wider exploration.
///
/// # Arguments
///
/// * `parent_1` - First parent.
/// * `parent_2` - Second parent.
/// * `alpha` - Exploration parameter (typically 0.5).
///
/// # Returns
///
/// Two children chromosomes, or an error if parents have mismatched lengths.
pub fn blend_alpha<T>(
    parent_1: &RangeChromosome<T>,
    parent_2: &RangeChromosome<T>,
    alpha: f64,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + BlendConvertible,
{
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }

    debug!(target="crossover_events", method="blend_alpha"; "Starting BLX-α crossover with alpha={}", alpha);

    let mut rng = rand::rng();
    let dna1 = parent_1.dna();
    let dna2 = parent_2.dna();

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    for i in 0..len {
        let p1_val: f64 = T::to_f64(dna1[i].value);
        let p2_val: f64 = T::to_f64(dna2[i].value);

        let min_val = p1_val.min(p2_val);
        let max_val = p1_val.max(p2_val);
        let d = max_val - min_val;

        let lo = min_val - alpha * d;
        let hi = max_val + alpha * d;

        // Clamp to gene range if available
        let (clamped_lo, clamped_hi) = if !dna1[i].ranges.is_empty() {
            let range_lo: f64 = T::to_f64(dna1[i].ranges[0].0);
            let range_hi: f64 = T::to_f64(dna1[i].ranges[0].1);
            (lo.max(range_lo), hi.min(range_hi))
        } else {
            (lo, hi)
        };

        let c1_val = rng.random_range(clamped_lo..=clamped_hi);
        let c2_val = rng.random_range(clamped_lo..=clamped_hi);

        let mut gene1 = dna1[i].clone();
        let mut gene2 = dna2[i].clone();
        gene1.value = T::from_f64(c1_val);
        gene2.value = T::from_f64(c2_val);

        child_dna_1.push(gene1);
        child_dna_2.push(gene2);
    }

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="blend_alpha"; "BLX-α crossover finished");
    Ok(vec![child_1, child_2])
}

/// Trait for types that can be converted to/from an f64 value (for BLX-α).
pub trait BlendConvertible {
    fn from_f64(val: f64) -> Self;
    fn to_f64(val: Self) -> f64;
}

impl BlendConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl BlendConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl BlendConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl BlendConvertible for i64 {
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

    fn build_parents() -> (RangeChromosome<f64>, RangeChromosome<f64>) {
        let mut p1 = RangeChromosome::<f64>::new();
        let mut p2 = RangeChromosome::<f64>::new();
        let dna1 = vec![
            RangeGenotype::new(0, vec![(0.0, 100.0)], 30.0),
            RangeGenotype::new(1, vec![(0.0, 100.0)], 70.0),
        ];
        let dna2 = vec![
            RangeGenotype::new(0, vec![(0.0, 100.0)], 60.0),
            RangeGenotype::new(1, vec![(0.0, 100.0)], 40.0),
        ];
        p1.set_dna(Cow::Owned(dna1));
        p2.set_dna(Cow::Owned(dna2));
        (p1, p2)
    }

    #[test]
    fn blend_alpha_produces_two_children_same_length() {
        let (p1, p2) = build_parents();
        let children = blend_alpha(&p1, &p2, 0.5).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].dna().len(), 2);
        assert_eq!(children[1].dna().len(), 2);
    }

    #[test]
    fn blend_alpha_children_stay_within_range() {
        let (p1, p2) = build_parents();
        for _ in 0..200 {
            let children = blend_alpha(&p1, &p2, 0.5).unwrap();
            for child in &children {
                for gene in child.dna() {
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

    #[test]
    fn blend_alpha_error_on_different_lengths() {
        let mut p1 = RangeChromosome::<f64>::new();
        let mut p2 = RangeChromosome::<f64>::new();
        p1.set_dna(Cow::Owned(vec![RangeGenotype::new(
            0,
            vec![(0.0, 10.0)],
            5.0,
        )]));
        p2.set_dna(Cow::Owned(vec![
            RangeGenotype::new(0, vec![(0.0, 10.0)], 5.0),
            RangeGenotype::new(1, vec![(0.0, 10.0)], 5.0),
        ]));
        let result = blend_alpha(&p1, &p2, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn blend_alpha_zero_keeps_children_between_parents() {
        let (p1, p2) = build_parents();
        // alpha=0 means children strictly between parent values
        for _ in 0..100 {
            let children = blend_alpha(&p1, &p2, 0.0).unwrap();
            for child in &children {
                let val = child.dna()[0].value;
                assert!(
                    (30.0..=60.0).contains(&val),
                    "With alpha=0, value {} should be between 30 and 60",
                    val
                );
            }
        }
    }

    #[test]
    fn blend_alpha_with_i32() {
        let mut p1 = RangeChromosome::<i32>::new();
        let mut p2 = RangeChromosome::<i32>::new();
        p1.set_dna(Cow::Owned(vec![RangeGenotype::new(0, vec![(0, 100)], 30)]));
        p2.set_dna(Cow::Owned(vec![RangeGenotype::new(0, vec![(0, 100)], 70)]));
        let children = blend_alpha(&p1, &p2, 0.5).unwrap();
        assert_eq!(children.len(), 2);
        for child in &children {
            let (lo, hi) = child.dna()[0].ranges[0];
            assert!(child.dna()[0].value >= lo && child.dna()[0].value <= hi);
        }
    }
}
