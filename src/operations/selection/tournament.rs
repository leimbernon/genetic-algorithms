use crate::traits::ChromosomeT;
use std::collections::HashMap;
use std::sync::Arc;
use std::{sync::Mutex, thread};
use rand::Rng;
use log::{trace, debug};

/**
 * Main function for tournament selection
 */
pub fn tournament<U>(chromosomes: &Vec<U>, couples: i32, number_of_threads: i32) -> HashMap<usize, usize>
where
U:ChromosomeT + Send + Sync + 'static + Clone
{
    
    if number_of_threads == 1{
        tournament_single_thread(chromosomes, couples)
    }else{
        let number_of_threads_t = if number_of_threads > couples {couples}else{number_of_threads};
        let number_of_threads_t = if number_of_threads_t & 1 == 1 {number_of_threads_t-1}else{number_of_threads_t};
        tournament_multithread(chromosomes, couples, number_of_threads_t)
    }
}


/**
 * Function for tournament selection in a single thread 
 */
fn tournament_single_thread<U>(chromosomes: &Vec<U>, couples: i32) -> HashMap<usize, usize>
where
U:ChromosomeT
{

    debug!(target="selection_events", method="tournament"; "Starting tournament selection in single thread");
    let mut rng = rand::thread_rng();
    let mut mating = HashMap::new();
    let chromosome_couples = couples*2;

    let mut indexes = Vec::new();

    //Getting the list of indexes
    for i in 0..chromosome_couples {
        indexes.push(i);
    }

    //Getting the parents from the population
    let mut parent_1 = None;

    for _i in 0..indexes.len(){
        let index_1 = rng.gen_range(0..indexes.len());
        let final_index_1 = indexes[index_1];
        trace!(target="selection_events", method="tournament"; "Index 1: {}, final index 1: {}", index_1, final_index_1);

        let index_2 = rng.gen_range(0..indexes.len());
        let final_index_2 = indexes[index_2];
        trace!(target="selection_events", method="tournament"; "Index 2: {}, final index 2: {}", index_2, final_index_2);

        let final_index;
        let index_to_delete;

        //Fights between both parents
        if chromosomes[final_index_1 as usize].get_fitness() >= chromosomes[final_index_2 as usize].get_fitness() {
            final_index = final_index_1;
            index_to_delete = index_1;
        }else{
            final_index = final_index_2;
            index_to_delete = index_2;
        }

        //Sets the mating
        if parent_1.is_none() {
            parent_1 = Some(final_index);
        }else{
            trace!(target="selection_events", method="tournament"; "Mating index 1: {} - index 2: {}", parent_1.unwrap(), final_index);
            mating.insert(parent_1.unwrap() as usize, final_index as usize);
            parent_1 = None;
        }

        indexes.remove(index_to_delete);
    }

    debug!(target="selection_events", method="tournament"; "Tournament selection in single thread finished");
    mating
}

/**
 * Function for tournament selection in multithread 
 */
fn tournament_multithread<U>(chromosomes: &Vec<U>, couples: i32, number_of_threads: i32) -> HashMap<usize, usize>
where
U:ChromosomeT+ Send + Sync + 'static + Clone
{

    debug!(target="selection_events", method="tournament"; "Starting tournament selection in multiple threads ({})", number_of_threads);
    let mut mating = HashMap::new();
    let couples = if couples*2 > chromosomes.len() as i32 {(chromosomes.len() / 2) as i32}else{couples};

    //Sets the indexes
    let mut indexes = Vec::new();
    for i in 0..couples*2{
        indexes.push(i);
    }

    //Thread control
    let mut handles = vec![];

    //Variables that will be sent
    let left = Arc::new(Mutex::new(Vec::new()));
    let right = Arc::new(Mutex::new(Vec::new()));
    //let indexes = Arc::new(Mutex::new(indexes));
    let mut start_index = 0;
    let jump = indexes.len() as i32 / number_of_threads;
    let chromosomes = Arc::new(Mutex::new(Vec::from_iter(chromosomes[..].iter().cloned())));
    trace!(target="selection_events", method="tournament"; "start index: {}, jump: {}", start_index, jump);

    //Running the different threads
    for thread in 0..number_of_threads{
        
        //Copies of the variables
        let winners = if thread & 1 == 1 {Arc::clone(&left)}else{Arc::clone(&right)};
        let chromosomes = Arc::clone(&chromosomes);

        let indexes_len = indexes.len();
        let end_index = if start_index + jump > indexes_len as i32 {indexes_len as i32}else{start_index + jump};
        let indexes = Vec::from_iter(indexes[start_index as usize..end_index as usize].iter().cloned());
        let indexes = Arc::new(Mutex::new(indexes));

        trace!(target="selection_events", method="tournament"; "Thread {} - indexes length {} - end index {}", thread, indexes_len, end_index);

        //Run the thread
        let handle = thread::spawn(move || {
            
            let mut rng = rand::thread_rng();
            let chromosomes_t = chromosomes.lock().unwrap().clone();
            let mut indexes_t = indexes.lock().unwrap().clone();

            for _ in 0..indexes_t.len(){

                //Gets the indexes for the tournament
                let index_1 = rng.gen_range(0..indexes_t.len());
                let final_index_1 = indexes_t[index_1];

                let index_2 = rng.gen_range(0..indexes_t.len());
                let final_index_2 = indexes_t[index_2];
                trace!(target="selection_events", method="tournament"; "Thread {} - indexes 1 {} - final index 1 {} - index 2 {} - final index 2 {} ", thread, index_1, final_index_1, index_2, final_index_2);

                //Compare both chromosomes
                if chromosomes_t[final_index_1 as usize].get_fitness() >= chromosomes_t[final_index_2 as usize].get_fitness(){
                    winners.lock().unwrap().push(final_index_1);
                    indexes_t.remove(index_1);
                }else{
                    winners.lock().unwrap().push(final_index_2);
                    indexes_t.remove(index_2);
                }
            }

        });
        handles.push(handle);

        start_index = if start_index + jump > indexes_len as i32 {indexes_len as i32}else{start_index + jump};
        trace!(target="selection_events", method="tournament"; "Thread {} - start index {}", thread, start_index);
    }

    for handle in handles{
        handle.join().unwrap();
    }

    //Gets the final vectors
    let left = left.lock().unwrap();
    let right = right.lock().unwrap();

    //Inserts the keys and values into the hashmap
    for item in 0..left.len() {
        mating.insert(left[item] as usize, right[item] as usize);
        trace!(target="selection_events", method="tournament"; "Mating index 1: {} - index 2: {} ", left[item], right[item]);
    }

    debug!(target="selection_events", method="tournament"; "Tournament selection in multiple threads finished");
    mating
}