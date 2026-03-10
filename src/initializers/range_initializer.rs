/*!
# Range Chromosome Initializer

Provides random initialization for chromosomes with genes in a specified range.

# Examples

```rust
use genetic_algorithms::initializers::range_random_initialization;
```
*/

use crate::chromosomes::Generic;
use crate::error::GaError;
use rand::Rng;

/// Random initialization for chromosomes with genes in a specified range.
///
/// # Arguments
/// * `n_genes` - Number of genes per chromosome.
/// * `min` - Minimum value (inclusive).
/// * `max` - Maximum value (inclusive).
///
/// # Returns
/// * `Result<Generic<T>, GaError>` - Initialized chromosome.
///
/// # Errors
/// * Returns `GaError` if initialization fails.
///
/// # Examples
/// ```rust
/// use genetic_algorithms::initializers::range_random_initialization;
/// let chromosome = range_random_initialization(10, 0, 100);
/// ```
pub fn range_random_initialization<T>(n_genes: usize, min: T, max: T) -> Result<Generic<T>, GaError>
where
    T: rand::distributions::uniform::SampleUniform + Copy,
{
    let mut rng = rand::thread_rng();
    let dna: Vec<T> = (0..n_genes)
        .map(|_| rng.gen_range(min..=max))
        .collect();
    Ok(Generic::from_dna(dna))
}
