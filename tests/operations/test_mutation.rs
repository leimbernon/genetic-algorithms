#[cfg(test)]
use crate::structures::{Gene, Chromosome};
use genetic_algorithms::{fitness::FitnessFnWrapper, operations::mutation::{aga_probability, inversion, scramble, swap}};

#[test]
fn test_swap_mutation(){
    
    //We create 1 dna for 1 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}, Gene{id:9}, Gene{id:10}, Gene{id:11}, Gene{id:12}, Gene{id:13},
    Gene{id:14}, Gene{id:15}, Gene{id:16}, Gene{id:17}, Gene{id:18}, Gene{id:19}, Gene{id:20}, Gene{id:21}, Gene{id:22}, Gene{id:23}, Gene{id:24}, Gene{id:25}, Gene{id:26}, Gene{id:27}, Gene{id:28}, Gene{id:29},
    Gene{id:30}, Gene{id:31}, Gene{id:32}, Gene{id:33}, Gene{id:34}, Gene{id:35}, Gene{id:36}, Gene{id:37}, Gene{id:38}, Gene{id:39}, Gene{id:40}, Gene{id:41}, Gene{id:42}, Gene{id:43}, Gene{id:44}, Gene{id:45},
    Gene{id:46}, Gene{id:47}, Gene{id:48}, Gene{id:49}, Gene{id:50}]; 

    //We create the individuals
    let mut individual_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_1_copy = individual_1.clone();

    //We mutate the dna
    swap::swap(&mut individual_1);
    assert_ne!(individual_1, individual_1_copy);
}

#[test]
fn test_inversion_mutation(){
    
    //We create 1 dna for 1 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}, Gene{id:9}, Gene{id:10}, Gene{id:11}, Gene{id:12}, Gene{id:13},
    Gene{id:14}, Gene{id:15}, Gene{id:16}, Gene{id:17}, Gene{id:18}, Gene{id:19}, Gene{id:20}, Gene{id:21}, Gene{id:22}, Gene{id:23}, Gene{id:24}, Gene{id:25}, Gene{id:26}, Gene{id:27}, Gene{id:28}, Gene{id:29},
    Gene{id:30}, Gene{id:31}, Gene{id:32}, Gene{id:33}, Gene{id:34}, Gene{id:35}, Gene{id:36}, Gene{id:37}, Gene{id:38}, Gene{id:39}, Gene{id:40}, Gene{id:41}, Gene{id:42}, Gene{id:43}, Gene{id:44}, Gene{id:45},
    Gene{id:46}, Gene{id:47}, Gene{id:48}, Gene{id:49}, Gene{id:50}]; 

    //We create the individuals
    let mut individual_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_1_copy = individual_1.clone();

    //We mutate the dna
    inversion::inversion(&mut individual_1);
    assert_ne!(individual_1, individual_1_copy);
}

#[test]
fn test_scramble_mutation(){
    
    //We create 1 dna for 1 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}, Gene{id:9}, Gene{id:10}, Gene{id:11}, Gene{id:12}, Gene{id:13},
    Gene{id:14}, Gene{id:15}, Gene{id:16}, Gene{id:17}, Gene{id:18}, Gene{id:19}, Gene{id:20}, Gene{id:21}, Gene{id:22}, Gene{id:23}, Gene{id:24}, Gene{id:25}, Gene{id:26}, Gene{id:27}, Gene{id:28}, Gene{id:29},
    Gene{id:30}, Gene{id:31}, Gene{id:32}, Gene{id:33}, Gene{id:34}, Gene{id:35}, Gene{id:36}, Gene{id:37}, Gene{id:38}, Gene{id:39}, Gene{id:40}, Gene{id:41}, Gene{id:42}, Gene{id:43}, Gene{id:44}, Gene{id:45},
    Gene{id:46}, Gene{id:47}, Gene{id:48}, Gene{id:49}, Gene{id:50}]; 

    //We create the individuals
    let mut individual_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_1_copy = individual_1.clone();

    //We mutate the dna
    scramble::scramble(&mut individual_1);
    assert_ne!(individual_1, individual_1_copy);
}

#[test]
fn test_mutation_aga_probability_over_avg(){

    let parent_1 = Chromosome{dna: Vec::<Gene>::new(), fitness: 25.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let parent_2 = Chromosome{dna: Vec::<Gene>::new(), fitness: 100.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for mutation
    let aga_mutation_probability = aga_probability(&parent_1, &parent_2, f_avg, probability_max, probability_min);

    //We verify the result of the aga mutation probability
    assert_eq!(aga_mutation_probability, probability_min);
}


#[test]
fn test_mutation_aga_probability_under_avg(){

    let parent_1 = Chromosome{dna: Vec::<Gene>::new(), fitness: 25.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let parent_2 = Chromosome{dna: Vec::<Gene>::new(), fitness: 49.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for mutation
    let aga_mutation_probability = aga_probability(&parent_1, &parent_2, f_avg, probability_max, probability_min);

    //We verify the result of the aga mutation probability
    assert_eq!(aga_mutation_probability, probability_max);
}