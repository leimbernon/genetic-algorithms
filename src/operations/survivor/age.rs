pub(crate) use crate::traits::ChromosomeT;
use log::{debug, trace};

pub fn age_based<U: ChromosomeT>(chromosomes: &mut Vec<U>, population_size: usize) {
    //We first sort the chromosomes by their fitness
    debug!(target="survivor_events", method="age_based"; "Starting age based survivor method");
    chromosomes.sort_by_key(|a| std::cmp::Reverse(a.age()));

    // Drop surplus individuals from the tail in O(1) instead of
    // loop of Vec::remove (O(N) per removal).
    trace!(target="survivor_events", method="age_based"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
    if chromosomes.len() > population_size {
        chromosomes.truncate(population_size);
    }
    debug!(target="survivor_events", method="age_based"; "Age based survivor method finished");
}
