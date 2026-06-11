//! Age-based survivor selection.
//!
//! Retains the youngest individuals (lowest age) after parents and offspring
//! have been merged. This strategy favours fresh genetic material and
//! prevents long-lived individuals from dominating the population.

pub(crate) use crate::traits::ChromosomeT;
use log::{debug, trace};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

/// Age-based survivor selection: sorts individuals by age (youngest first)
/// and keeps the top `population_size`.
///
/// # Arguments
///
/// * `chromosomes` - Combined parents + offspring (modified in place).
/// * `population_size` - Desired population size after selection.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::survivor::age_based;
/// use genetic_algorithms::chromosomes::Binary;
/// let mut population: Vec<Binary> = vec![Binary::new(); 20];
/// age_based(&mut population, 10);
/// ```
pub fn age_based<U: ChromosomeT>(chromosomes: &mut Vec<U>, population_size: usize) {
    //We first sort the chromosomes by their fitness
    debug!(target="survivor_events", method="age_based"; "Starting age based survivor method");
    #[cfg(not(target_arch = "wasm32"))]
    chromosomes.par_sort_unstable_by(|a, b| b.age().cmp(&a.age()));
    #[cfg(target_arch = "wasm32")]
    chromosomes.sort_unstable_by(|a, b| b.age().cmp(&a.age()));

    // Drop surplus individuals from the tail in O(1) instead of
    // loop of Vec::remove (O(N) per removal).
    trace!(target="survivor_events", method="age_based"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
    if chromosomes.len() > population_size {
        chromosomes.truncate(population_size);
    }
    debug!(target="survivor_events", method="age_based"; "Age based survivor method finished");
}
