//! OneMax-style fitness function that counts `true` genes.

use crate::genotypes::Binary;

/// Returns the number of genes whose `value` is `true`.
///
/// This is a simple fitness function commonly used for benchmarking binary
/// genetic algorithms (the "OneMax" problem).
///
/// # Example
///
/// ```
/// use genetic_algorithms::genotypes::Binary;
/// use genetic_algorithms::fitness::count_true;
///
/// let dna = vec![
///     Binary { id: 0, value: true },
///     Binary { id: 1, value: false },
///     Binary { id: 2, value: true },
/// ];
/// assert_eq!(count_true(&dna), 2.0);
/// ```
pub fn count_true(dna: &[Binary]) -> f64 {
    dna.iter().filter(|gene| gene.value).count() as f64
}
