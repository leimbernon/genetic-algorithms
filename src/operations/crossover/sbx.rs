use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Simulated Binary Crossover (SBX) for Range<T> chromosomes.
///
/// SBX simulates single-point crossover in continuous space. The distribution
/// index `eta` controls how close offspring are to their parents:
/// - Low eta (e.g., 2): offspring spread far from parents (exploration).
/// - High eta (e.g., 20): offspring stay close to parents (exploitation).
///
/// This is the standard crossover for continuous numerical optimization
/// (e.g., real-valued function optimization).
///
/// # Arguments
///
/// * `parent_1` - First parent chromosome.
/// * `parent_2` - Second parent chromosome.
/// * `eta` - Distribution index (typically 2–20).
///
/// # Returns
///
/// Two children chromosomes, or an error if parents have mismatched lengths.
pub fn sbx<T>(
    parent_1: &RangeChromosome<T>,
    parent_2: &RangeChromosome<T>,
    eta: f64,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + SbxConvertible,
{
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }

    debug!(target="crossover_events", method="sbx"; "Starting SBX crossover with eta={}", eta);

    let mut rng = crate::rng::make_rng();
    let dna1 = parent_1.dna();
    let dna2 = parent_2.dna();

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    for i in 0..len {
        let p1_val: f64 = T::to_f64(dna1[i].value);
        let p2_val: f64 = T::to_f64(dna2[i].value);

        if (p1_val - p2_val).abs() < 1e-14 {
            // Parents are identical at this gene; children inherit the same value
            child_dna_1.push(dna1[i].clone());
            child_dna_2.push(dna2[i].clone());
            continue;
        }

        let u: f64 = rng.random_range(0.0..1.0);

        // Compute the spread factor beta
        let beta = if u <= 0.5 {
            (2.0 * u).powf(1.0 / (eta + 1.0))
        } else {
            (1.0 / (2.0 * (1.0 - u))).powf(1.0 / (eta + 1.0))
        };

        let c1_val = 0.5 * ((1.0 + beta) * p1_val + (1.0 - beta) * p2_val);
        let c2_val = 0.5 * ((1.0 - beta) * p1_val + (1.0 + beta) * p2_val);

        // Clamp to range if available
        let (c1_clamped, c2_clamped) = if !dna1[i].ranges.is_empty() {
            let lo: f64 = T::to_f64(dna1[i].ranges[0].0);
            let hi: f64 = T::to_f64(dna1[i].ranges[0].1);
            (c1_val.clamp(lo, hi), c2_val.clamp(lo, hi))
        } else {
            (c1_val, c2_val)
        };

        let mut gene1 = dna1[i].clone();
        let mut gene2 = dna2[i].clone();
        gene1.value = T::from_f64(c1_clamped);
        gene2.value = T::from_f64(c2_clamped);

        child_dna_1.push(gene1);
        child_dna_2.push(gene2);
    }

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="sbx"; "SBX crossover finished");
    Ok(vec![child_1, child_2])
}

/// Trait for types that can be converted to/from an f64 value (for SBX).
pub trait SbxConvertible {
    fn from_f64(val: f64) -> Self;
    fn to_f64(val: Self) -> f64;
}

impl SbxConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl SbxConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl SbxConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl SbxConvertible for i64 {
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
            RangeGenotype::new(0, vec![(0.0, 100.0)], 20.0),
            RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0),
            RangeGenotype::new(2, vec![(0.0, 100.0)], 50.0),
        ];
        let dna2 = vec![
            RangeGenotype::new(0, vec![(0.0, 100.0)], 60.0),
            RangeGenotype::new(1, vec![(0.0, 100.0)], 30.0),
            RangeGenotype::new(2, vec![(0.0, 100.0)], 50.0),
        ];
        p1.set_dna(Cow::Owned(dna1));
        p2.set_dna(Cow::Owned(dna2));
        (p1, p2)
    }

    #[test]
    fn sbx_produces_two_children_same_length() {
        let (p1, p2) = build_parents();
        let children = sbx(&p1, &p2, 2.0).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].dna().len(), 3);
        assert_eq!(children[1].dna().len(), 3);
    }

    #[test]
    fn sbx_children_stay_within_range() {
        let (p1, p2) = build_parents();
        for _ in 0..100 {
            let children = sbx(&p1, &p2, 2.0).unwrap();
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
    fn sbx_error_on_different_lengths() {
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
        let result = sbx(&p1, &p2, 2.0);
        assert!(result.is_err());
    }

    #[test]
    fn sbx_identical_parents_produce_same_children() {
        let mut p1 = RangeChromosome::<f64>::new();
        let dna = vec![
            RangeGenotype::new(0, vec![(0.0, 100.0)], 50.0),
            RangeGenotype::new(1, vec![(0.0, 100.0)], 50.0),
        ];
        p1.set_dna(Cow::Owned(dna.clone()));
        let p2 = p1.clone();
        let children = sbx(&p1, &p2, 10.0).unwrap();
        for child in &children {
            for (i, gene) in child.dna().iter().enumerate() {
                assert!(
                    (gene.value - 50.0).abs() < 1e-10,
                    "Gene {} should be 50.0, got {}",
                    i,
                    gene.value
                );
            }
        }
    }

    #[test]
    fn sbx_with_i32() {
        let mut p1 = RangeChromosome::<i32>::new();
        let mut p2 = RangeChromosome::<i32>::new();
        p1.set_dna(Cow::Owned(vec![
            RangeGenotype::new(0, vec![(0, 100)], 20),
            RangeGenotype::new(1, vec![(0, 100)], 80),
        ]));
        p2.set_dna(Cow::Owned(vec![
            RangeGenotype::new(0, vec![(0, 100)], 60),
            RangeGenotype::new(1, vec![(0, 100)], 30),
        ]));
        let children = sbx(&p1, &p2, 2.0).unwrap();
        assert_eq!(children.len(), 2);
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

    #[test]
    fn sbx_high_eta_produces_children_close_to_parents() {
        let (p1, p2) = build_parents();
        // With very high eta, children should be very close to parents
        let mut close_count = 0;
        for _ in 0..100 {
            let children = sbx(&p1, &p2, 100.0).unwrap();
            let c1_val = children[0].dna()[0].value;
            let p1_val = p1.dna()[0].value;
            let p2_val = p2.dna()[0].value;
            let midpoint = (p1_val + p2_val) / 2.0;
            let range = (p1_val - p2_val).abs();
            // Children should be within parent range
            if (c1_val - midpoint).abs() <= range {
                close_count += 1;
            }
        }
        assert!(
            close_count > 90,
            "High eta should keep children close to parents"
        );
    }
}
