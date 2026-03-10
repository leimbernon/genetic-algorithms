/*!
# Blend Alpha Crossover Operator

Implements blend crossover for floating-point chromosomes.

# Examples

```rust
use genetic_algorithms::operations::crossover::blend_alpha_crossover;
```
*/

use crate::chromosomes::Generic;
use crate::error::GaError;
use rand::Rng;

/// Blend alpha crossover for floating-point chromosomes.
///
/// # Arguments
/// * `parent1` - First parent chromosome.
/// * `parent2` - Second parent chromosome.
/// * `alpha` - Blend parameter (typically 0.5).
///
/// # Returns
/// * `Result<Generic<f64>, GaError>` - Child chromosome.
///
/// # Errors
/// * Returns `GaError` if crossover fails.
///
/// # Examples
/// ```rust
/// use genetic_algorithms::operations::crossover::blend_alpha_crossover;
/// let child = blend_alpha_crossover(&parent1, &parent2, 0.5);
/// ```
pub fn blend_alpha_crossover(
    parent1: &Generic<f64>,
    parent2: &Generic<f64>,
    alpha: f64,
) -> Result<Generic<f64>, GaError> {
    if parent1.dna().len() != parent2.dna().len() {
        return Err(GaError::InvalidChromosomeLength);
    }
    let mut rng = rand::thread_rng();
    let dna: Vec<f64> = parent1
        .dna()
        .iter()
        .zip(parent2.dna().iter())
        .map(|(g1, g2)| {
            let min = g1.min(g2);
            let max = g1.max(g2);
            let range = max - min;
            let lower = min - alpha * range;
            let upper = max + alpha * range;
            rng.gen_range(lower..=upper)
        })
        .collect();
    Ok(Generic::from_dna(dna))
}
