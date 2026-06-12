use crate::genotypes::Binary as BinaryGenotype;

/// Computes the Hamming distance between two binary DNA sequences.
///
/// The Hamming distance counts the number of positions where the corresponding
/// genes differ.
///
/// # Arguments
///
/// * `dna_a` - First binary DNA sequence.
/// * `dna_b` - Second binary DNA sequence.
///
/// # Returns
///
/// The Hamming distance as an `f64`. If the sequences have different lengths,
/// extra genes in the longer sequence count as differences.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::niching::distance::hamming_distance;
/// use genetic_algorithms::genotypes::Binary;
///
/// let a = vec![Binary { id: 0, value: true }, Binary { id: 1, value: false }, Binary { id: 2, value: true }];
/// let b = vec![Binary { id: 0, value: true }, Binary { id: 1, value: true }, Binary { id: 2, value: false }];
///
/// assert!((hamming_distance(&a, &b) - 2.0).abs() < f64::EPSILON);
/// ```
pub fn hamming_distance(dna_a: &[BinaryGenotype], dna_b: &[BinaryGenotype]) -> f64 {
    let max_len = dna_a.len().max(dna_b.len());
    let mut distance = 0usize;

    for i in 0..max_len {
        let a_val = dna_a.get(i).map(|g| g.value);
        let b_val = dna_b.get(i).map(|g| g.value);
        if a_val != b_val {
            distance += 1;
        }
    }

    distance as f64
}

/// Computes the Euclidean distance between two numeric DNA sequences.
///
/// Each gene's value is converted to `f64` and the standard Euclidean norm
/// is computed: `sqrt(sum((a_i - b_i)^2))`.
///
/// # Arguments
///
/// * `dna_a` - First DNA sequence.
/// * `dna_b` - Second DNA sequence.
///
/// # Returns
///
/// The Euclidean distance as an `f64`.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::niching::distance::euclidean_distance;
/// use genetic_algorithms::genotypes::Range;
///
/// let a = vec![
///     Range::new(0, vec![(0, 10)], 1),
///     Range::new(1, vec![(0, 10)], 2),
/// ];
/// let b = vec![
///     Range::new(0, vec![(0, 10)], 4),
///     Range::new(1, vec![(0, 10)], 6),
/// ];
///
/// let dist = euclidean_distance(&a, &b);
/// assert!((dist - 5.0).abs() < 1e-10);
/// ```
pub fn euclidean_distance<T>(
    dna_a: &[crate::genotypes::Range<T>],
    dna_b: &[crate::genotypes::Range<T>],
) -> f64
where
    T: Into<f64> + Copy + Sync + Send + Clone + Default + std::fmt::Debug,
{
    let min_len = dna_a.len().min(dna_b.len());
    let mut sum_sq = 0.0f64;

    for i in 0..min_len {
        let a_val: f64 = dna_a[i].value().into();
        let b_val: f64 = dna_b[i].value().into();
        sum_sq += (a_val - b_val).powi(2);
    }

    // Extra genes in the longer sequence: treat missing as 0.0
    if dna_a.len() > min_len {
        for gene in &dna_a[min_len..] {
            let v: f64 = gene.value().into();
            sum_sq += v.powi(2);
        }
    }
    if dna_b.len() > min_len {
        for gene in &dna_b[min_len..] {
            let v: f64 = gene.value().into();
            sum_sq += v.powi(2);
        }
    }

    sum_sq.sqrt()
}

/// Trait for computing distance between two chromosomes' DNA.
///
/// Implement this trait for custom distance metrics.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::niching::distance::{DistanceMetric, HammingDistance};
/// use genetic_algorithms::genotypes::Binary as BinaryGene;
///
/// let a = vec![BinaryGene::new(), BinaryGene::new()];
/// let b = vec![BinaryGene::new(), BinaryGene::new()];
/// let d = HammingDistance::distance(&a, &b);
/// assert!(d >= 0.0);
/// ```
pub trait DistanceMetric<G> {
    /// Computes the distance between two DNA sequences.
    fn distance(dna_a: &[G], dna_b: &[G]) -> f64;
}

/// Hamming distance metric for binary chromosomes.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::niching::distance::{DistanceMetric, HammingDistance};
/// use genetic_algorithms::genotypes::Binary as BinaryGene;
///
/// let a = vec![BinaryGene::new(), BinaryGene::new()];
/// let b = vec![BinaryGene::new(), BinaryGene::new()];
/// let d = HammingDistance::distance(&a, &b);
/// assert!(d >= 0.0);
/// ```
pub struct HammingDistance;

impl DistanceMetric<BinaryGenotype> for HammingDistance {
    fn distance(dna_a: &[BinaryGenotype], dna_b: &[BinaryGenotype]) -> f64 {
        hamming_distance(dna_a, dna_b)
    }
}

/// Euclidean distance metric for range chromosomes.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::niching::distance::{DistanceMetric, EuclideanDistance};
/// use genetic_algorithms::genotypes::Range as RangeGene;
///
/// let a = vec![RangeGene::new(0, vec![], 1.0_f64), RangeGene::new(1, vec![], 0.0_f64)];
/// let b = vec![RangeGene::new(0, vec![], 0.0_f64), RangeGene::new(1, vec![], 1.0_f64)];
/// let d = EuclideanDistance::distance(&a, &b);
/// assert!((d - std::f64::consts::SQRT_2).abs() < 1e-9);
/// ```
pub struct EuclideanDistance;

impl<T> DistanceMetric<crate::genotypes::Range<T>> for EuclideanDistance
where
    T: Into<f64> + Copy + Sync + Send + Clone + Default + std::fmt::Debug,
{
    fn distance(dna_a: &[crate::genotypes::Range<T>], dna_b: &[crate::genotypes::Range<T>]) -> f64 {
        euclidean_distance(dna_a, dna_b)
    }
}
