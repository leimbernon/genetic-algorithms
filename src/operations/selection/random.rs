use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/**
 * Function to make the random parent selection between the list of chromosomes
 */
pub fn random<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)> {
    let mut mating = Vec::new();
    let mut indexes = Vec::new();
    let mut rng = rand::rng();
    debug!(target="selection_events", method="random"; "Starting random selection");

    //Setting the indexes of the chromosomes
    let mut i = 0;
    while i < chromosomes.len() {
        indexes.push(i);
        i += 1;
    }

    //In this loop we create the mating vector
    while !indexes.is_empty() {
        //Getting the chromosome 1
        //We must have at least 2 remaining elements
        if indexes.len() < 2 {
            break;
        }
        let mut random_index_1 = 0;
        if indexes.len() > 1 {
            random_index_1 = rng.random_range(0..indexes.len() - 1);
        }
        let index_value_1 = indexes[random_index_1];
        indexes.remove(random_index_1);

        //Getting the chromosome 2
        let mut random_index_2 = 0;
        if indexes.len() > 1 {
            random_index_2 = rng.random_range(0..indexes.len() - 1);
        }

        //Adding the two chromosomes as a pair
        mating.push((index_value_1, indexes[random_index_2]));
        indexes.remove(random_index_2);

        trace!(target="selection_events", method="random"; "Mating index 1 {} with index 2 {}", index_value_1, random_index_2);
    }

    mating
}
