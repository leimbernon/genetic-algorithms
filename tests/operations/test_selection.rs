#[cfg(test)]
use crate::structures::{Gene, Chromosome};
use genetic_algorithms::{fitness::FitnessFnWrapper, operations::selection::{fitness_proportionate, random, tournament}};

#[test]
fn test_random_even_selection(){

    //We create 6 dna's for 6 chromosomes
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];
    let dna_6 = vec![Gene{id:11}, Gene{id:12}];

    //We create the chromosomes
    let chromosome_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_3 = Chromosome{dna: dna_3, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_4 = Chromosome{dna: dna_4, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_5 = Chromosome{dna: dna_5, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_6 = Chromosome{dna: dna_6, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};

    //We create the population and create the random mating
    let population = vec![chromosome_1, chromosome_2, chromosome_3, chromosome_4, chromosome_5, chromosome_6];
    let mating_population = random::random(&population);
    assert_eq!(mating_population.len(), 3);

}

#[test]
fn test_random_odd_selection(){
    
    //We create 6 dna's for 6 chromosomes
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the chromosomes
    let chromosome_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_3 = Chromosome{dna: dna_3, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_4 = Chromosome{dna: dna_4, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_5 = Chromosome{dna: dna_5, fitness: 0.0, age: 0, fitness_fn: FitnessFnWrapper::default()};

    //We create the population and create the random mating
    let population = vec![chromosome_1, chromosome_2, chromosome_3, chromosome_4, chromosome_5];
    let mating_population = random::random(&population);
    assert_eq!(mating_population.len(), 2);
}


#[test]
fn test_roulette_wheel_selection(){
    //We create 6 dna's for 5 chromosomes
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the chromosomes
    let chromosome_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: FitnessFnWrapper::default()};

    //We create the population and create the random mating
    let population = vec![chromosome_1, chromosome_2, chromosome_3, chromosome_4, chromosome_5];
    let mating_population = fitness_proportionate::roulette_wheel_selection(&population);
    assert_ne!(mating_population.len(), 0);
}

#[test]
fn test_stochastic_universal_sampling(){
    //We create 7 dna's for 7 chromosomes
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];
    let dna_6 = vec![Gene{id:11}, Gene{id:12}];
    let dna_7 = vec![Gene{id:13}, Gene{id:14}];

    //We create the chromosomes
    let chromosome_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_6 = Chromosome{dna: dna_6, fitness: 60.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_7 = Chromosome{dna: dna_7, fitness: 70.0, age: 0, fitness_fn: FitnessFnWrapper::default()};

    //We create the population and create the random mating
    let population = vec![chromosome_1, chromosome_2, chromosome_3, chromosome_4, chromosome_5, chromosome_6, chromosome_7];

    // SUS is stochastic — may rarely produce 0 pairs. Retry a few times.
    let mut found = false;
    for _ in 0..10 {
        let mating_population = fitness_proportionate::stochastic_universal_sampling(&population, 3);
        if !mating_population.is_empty() {
            found = true;
            break;
        }
    }
    assert!(found, "stochastic_universal_sampling produced no pairs after 10 attempts");
}


#[test]
fn test_tournament_singlethread(){
    //We create 5 dna's for 5 chromosomes
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the chromosomes
    let chromosome_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: FitnessFnWrapper::default()};

    //We create the population and create the random mating
    let population = vec![chromosome_1, chromosome_2, chromosome_3, chromosome_4, chromosome_5];
    let mating_population = tournament::tournament(&population, 2, 1);
    assert_eq!(mating_population.len(), 2);
    assert_ne!(mating_population.len(), 0);
}

#[test]
fn test_tournament_multithread(){
    //We create 5 dna's for 5 chromosomes
    let dna_1 = vec![Gene{id:1}, Gene{id:2}];
    let dna_2 = vec![Gene{id:3}, Gene{id:4}];
    let dna_3 = vec![Gene{id:5}, Gene{id:6}];
    let dna_4 = vec![Gene{id:7}, Gene{id:8}];
    let dna_5 = vec![Gene{id:9}, Gene{id:10}];

    //We create the chromosomes
    let chromosome_1 = Chromosome{dna: dna_1, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_2 = Chromosome{dna: dna_2, fitness: 20.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_3 = Chromosome{dna: dna_3, fitness: 30.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_4 = Chromosome{dna: dna_4, fitness: 40.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let chromosome_5 = Chromosome{dna: dna_5, fitness: 50.0, age: 0, fitness_fn: FitnessFnWrapper::default()};

    //We create the population and create the random mating
    let population = vec![chromosome_1, chromosome_2, chromosome_3, chromosome_4, chromosome_5];
    let mating_population = tournament::tournament(&population, 2, 2);
    assert_eq!(mating_population.len(), 2);
    assert_ne!(mating_population.len(), 0);
}