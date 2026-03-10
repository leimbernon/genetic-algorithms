/*!
# Generic Chromosome Initializer

Provides random initialization for generic chromosomes.

# Examples

```rust
use genetic_algorithms::initializers::generic_random_initialization;
```
*/

use crate::chromosomes::Generic;
use crate::error::GaError;
use rand::Rng;

/// Random initialization for generic chromosomes.
///
/// # Arguments
/// * `n_genes` - Number of genes per chromosome.
/// * `gene_fn` - Function to generate a random gene.
///
/// # Returns
/// * `Result<Generic<T>, GaError>` - Initialized generic chromosome.
///
/// # Errors
/// * Returns `GaError` if initialization fails.
///
/// # Examples
/// ```rust
/// use genetic_algorithms::initializers::generic_random_initialization;
/// let chromosome = generic_random_initialization(10, || rand::random::<u8>());
/// ```
pub fn generic_random_initialization<T, F>(n_genes: usize, gene_fn: F) -> Result<Generic<T>, GaError>
where
    F: Fn() -> T,
{
    let dna: Vec<T> = (0..n_genes).map(|_| gene_fn()).collect();
    Ok(Generic::from_dna(dna))
}
