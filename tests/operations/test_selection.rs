use std::sync::Arc;

#[cfg(test)]
use crate::structures::{Gene, Chromosome};
use genetic_algorithms::operations::{selection::random, selection::fitness_proportionate, selection::tournament};

#[test]
fn test_random_even_selection(){

    //We create 6 dna's for 6 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];
    let dna_6 = vec![Gene{id:11}, Gene{id:12}];

    //We create the individuals
    let individual_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_3 = Chromosome{dna: dna_3, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_4 = Chromosome{dna: dna_4, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_5 = Chromosome{dna: dna_5, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_6 = Chromosome{dna: dna_6, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //We create the population and create the random mating
    let population = vec![individual_1, individual_2, individual_3, individual_4, individual_5, individual_6];
    let mating_population = random::random(&population);
    assert_eq!(mating_population.len(), 3);

}

#[test]
fn test_random_odd_selection(){
    
    //We create 6 dna's for 6 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the individuals
    let individual_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_3 = Chromosome{dna: dna_3, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_4 = Chromosome{dna: dna_4, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_5 = Chromosome{dna: dna_5, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //We create the population and create the random mating
    let population = vec![individual_1, individual_2, individual_3, individual_4, individual_5];
    let mating_population = random::random(&population);
    assert_eq!(mating_population.len(), 2);
}


#[test]
fn test_roulette_wheel_selection(){
    //We create 6 dna's for 6 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the individuals
    let individual_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //We create the population and create the random mating
    let population = vec![individual_1, individual_2, individual_3, individual_4, individual_5];
    let mating_population = fitness_proportionate::roulette_wheel_selection(&population);
    assert_ne!(mating_population.len(), 0);
}

#[test]
fn test_stochastic_universal_sampling(){
    //We create 6 dna's for 6 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];
    let dna_6 = vec![Gene{id:11}, Gene{id:12}];
    let dna_7 = vec![Gene{id:13}, Gene{id:14}];

    //We create the individuals
    let individual_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_6 = Chromosome{dna: dna_6, fitness: 60.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_7 = Chromosome{dna: dna_7, fitness: 70.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //We create the population and create the random mating
    let population = vec![individual_1, individual_2, individual_3, individual_4, individual_5, individual_6, individual_7];
    let mating_population = fitness_proportionate::stochastic_universal_sampling(&population, 3);
    assert_ne!(mating_population.len(), 0);
}


#[test]
fn test_tournament_singlethread(){
    //We create 6 dna's for 6 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the individuals
    let individual_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //We create the population and create the random mating
    let population = vec![individual_1, individual_2, individual_3, individual_4, individual_5];
    let mating_population = tournament::tournament(&population, 2, 1);
    assert_eq!(mating_population.len(), 2);
    assert_ne!(mating_population.len(), 0);
}

#[test]
fn test_tournament_multithread(){
    //We create 6 dna's for 6 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the individuals
    let individual_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let individual_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //We create the population and create the random mating
    let population = vec![individual_1, individual_2, individual_3, individual_4, individual_5];
    let mating_population = tournament::tournament(&population, 2, 2);
    assert_eq!(mating_population.len(), 2);
    assert_ne!(mating_population.len(), 0);
}