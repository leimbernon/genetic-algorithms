/*!
# Binary Chromosome Initializer

Provides random initialization for binary chromosomes.

# Examples

```rust
use genetic_algorithms::initializers::binary_random_initialization;
```
*/

use crate::chromosomes::Binary;
use crate::genotypes::Binary as BinaryGene;
use crate::error::GaError;
use rand::Rng;

/// Random initialization for binary chromosomes.
///
/// # Arguments
/// * `n_genes` - Number of genes per chromosome.
///
/// # Returns
/// * `Result<Binary, GaError>` - Initialized binary chromosome.
///
/// # Errors
/// * Returns `GaError` if initialization fails.
///
/// # Examples
/// ```rust
/// use genetic_algorithms::initializers::binary_random_initialization;
/// let chromosome = binary_random_initialization(10);
/// ```
pub fn binary_random_initialization(n_genes: usize) -> Result<Binary, GaError> {
    let mut rng = rand::thread_rng();
    let dna: Vec<BinaryGene> = (0..n_genes)
        .map(|_| rng.gen_bool(0.5))
        .collect();
    Ok(Binary::from_dna(dna))
}
