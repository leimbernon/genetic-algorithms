pub(crate) use crate::traits::ChromosomeT;
use log::{debug, trace};

pub fn age_based<U: ChromosomeT>(chromosomes: &mut Vec<U>, population_size: usize) {
    //We first sort the chromosomes by their fitness
    debug!(target="survivor_events", method="age_based"; "Starting age based survivor method");
    chromosomes.sort_by_key(|a| std::cmp::Reverse(a.get_age()));

    //If there is more chromosomes than the defined population number
    trace!(target="survivor_events", method="age_based"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
    if chromosomes.len() > population_size {
        let chromosomes_to_remove = chromosomes.len() - population_size;
        for _i in 0..chromosomes_to_remove {
            chromosomes.remove(chromosomes.len() - 1);
        }
    }
    debug!(target="survivor_events", method="age_based"; "Age based survivor method finished");
}
