use std::sync::Arc;

#[cfg(test)]
use crate::structures::{Gene, Chromosome};
use genetic_algorithms::operations::crossover::{cycle, multipoint, uniform_crossover, aga_probability};


#[test]
fn test_cycle_crossover(){

    //we create 2 dnas of 4 genes for 2 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}];
    let dna_2 = vec![Gene{id:4}, Gene{id:3}, Gene{id:2}, Gene{id:1}];

    let parent_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let parent_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //Getting the offspring
    let mut offspring = cycle::cycle(&parent_1, &parent_2).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), child_2.dna.len());
    assert_eq!(parent_1.dna.len(), parent_2.dna.len());

    //Checking that the crossover has been well executed for the child 1
    assert_eq!(child_2.dna.get(0).unwrap().id, 4);
    assert_eq!(child_2.dna.get(1).unwrap().id, 2);
    assert_eq!(child_2.dna.get(2).unwrap().id, 3);
    assert_eq!(child_2.dna.get(3).unwrap().id, 1);

    //Checking that the crossover has been well executed for the child 2
    assert_eq!(child_1.dna.get(0).unwrap().id, 1);
    assert_eq!(child_1.dna.get(1).unwrap().id, 3);
    assert_eq!(child_1.dna.get(2).unwrap().id, 2);
    assert_eq!(child_1.dna.get(3).unwrap().id, 4);
}



#[test]
fn test_multipoint_crossover_2_points(){

    //we create 2 dnas of 4 genes for 2 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:5}, Gene{id:6}];
    let dna_2 = vec![Gene{id:6}, Gene{id:5}, Gene{id:4}, Gene{id:3}, Gene{id:2}, Gene{id:1}];

    let parent_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let parent_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //Getting the offspring
    let mut offspring = multipoint(&parent_1, &parent_2, &2).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), child_2.dna.len());
    assert_eq!(parent_1.dna.len(), parent_2.dna.len());

    //Checking that the crossover has been well executed for the child 1
    assert_eq!(child_1.dna.get(0).unwrap().id, 1);
    assert_eq!(child_1.dna.get(1).unwrap().id, 2);
    assert_eq!(child_1.dna.get(2).unwrap().id, 4);
    assert_eq!(child_1.dna.get(3).unwrap().id, 3);
    assert_eq!(child_1.dna.get(4).unwrap().id, 5);
    assert_eq!(child_1.dna.get(5).unwrap().id, 6);

    //Checking that the crossover has been well executed for the child 2
    assert_eq!(child_2.dna.get(0).unwrap().id, 6);
    assert_eq!(child_2.dna.get(1).unwrap().id, 5);
    assert_eq!(child_2.dna.get(2).unwrap().id, 3);
    assert_eq!(child_2.dna.get(3).unwrap().id, 4);
    assert_eq!(child_2.dna.get(4).unwrap().id, 2);
    assert_eq!(child_2.dna.get(5).unwrap().id, 1);
}

#[test]
fn test_multipoint_crossover_4_points(){

    //we create 2 dnas of 4 genes for 2 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:5}, Gene{id:6}];
    let dna_2 = vec![Gene{id:6}, Gene{id:5}, Gene{id:4}, Gene{id:3}, Gene{id:2}, Gene{id:1}];

    let parent_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let parent_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //Getting the offspring
    let mut offspring = multipoint(&parent_1, &parent_2, &4).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), child_2.dna.len());
    assert_eq!(parent_1.dna.len(), parent_2.dna.len());

    //Checking that the crossover has been well executed for the child 1
    assert_eq!(child_1.dna.get(0).unwrap().id, 1);
    assert_eq!(child_1.dna.get(1).unwrap().id, 5);
    assert_eq!(child_1.dna.get(2).unwrap().id, 3);
    assert_eq!(child_1.dna.get(3).unwrap().id, 3);
    assert_eq!(child_1.dna.get(4).unwrap().id, 5);
    assert_eq!(child_1.dna.get(5).unwrap().id, 1);

    //Checking that the crossover has been well executed for the child 2
    assert_eq!(child_2.dna.get(0).unwrap().id, 6);
    assert_eq!(child_2.dna.get(1).unwrap().id, 2);
    assert_eq!(child_2.dna.get(2).unwrap().id, 4);
    assert_eq!(child_2.dna.get(3).unwrap().id, 4);
    assert_eq!(child_2.dna.get(4).unwrap().id, 2);
    assert_eq!(child_2.dna.get(5).unwrap().id, 6);
}

#[test]
fn test_uniform_crossover(){

    //we create 2 dnas of 4 genes for 2 individuals
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}];
    let dna_2 = vec![Gene{id:4}, Gene{id:3}, Gene{id:2}, Gene{id:1}];

    let parent_1 = Chromosome{dna: dna_1, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let parent_2 = Chromosome{dna: dna_2, fitness: 0.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};

    //Getting the offspring
    let mut offspring = uniform_crossover::uniform(&parent_1, &parent_2).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), child_2.dna.len());
    assert_eq!(parent_1.dna.len(), parent_2.dna.len());
}

#[test]
fn test_xover_aga_probability_over_avg(){

    let parent_1 = Chromosome{dna: Vec::<Gene>::new(), fitness: 25.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let parent_2 = Chromosome{dna: Vec::<Gene>::new(), fitness: 100.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let f_max = 150.0;
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for crossover
    let aga_xover_probability = aga_probability(&parent_1, &parent_2, f_max, f_avg, probability_max, probability_min);

    //We verify the result of the aga crossover probability
    assert_eq!(aga_xover_probability, 0.375);
}


#[test]
fn test_xover_aga_probability_under_avg(){

    let parent_1 = Chromosome{dna: Vec::<Gene>::new(), fitness: 25.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let parent_2 = Chromosome{dna: Vec::<Gene>::new(), fitness: 49.0, age: 0, fitness_fn: Arc::new(|_| 0.0)};
    let f_max = 150.0;
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for crossover
    let aga_xover_probability = aga_probability(&parent_1, &parent_2, f_max, f_avg, probability_max, probability_min);

    //We verify the result of the aga crossover probability
    assert_eq!(aga_xover_probability, 0.25);
}