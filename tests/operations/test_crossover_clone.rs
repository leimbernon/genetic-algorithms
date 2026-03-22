#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::CrossoverConfiguration,
    fitness::FitnessFnWrapper,
    operations::crossover::{self, clone::clone_crossover},
    operations::Crossover,
    traits::CrossoverOperator,
};

#[test]
fn test_clone_crossover_produces_exact_copies() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 10.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let mut offspring = clone_crossover(&parent_1, &parent_2).unwrap();
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    // Children should be exact copies of parents
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), parent_2.dna.len());

    for (i, gene) in child_1.dna.iter().enumerate() {
        assert_eq!(gene.id, parent_1.dna[i].id);
    }
    for (i, gene) in child_2.dna.iter().enumerate() {
        assert_eq!(gene.id, parent_2.dna[i].id);
    }
}

#[test]
fn test_clone_crossover_returns_two_children() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = clone_crossover(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);
}

#[test]
fn test_clone_crossover_different_lengths_returns_error() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let result = clone_crossover(&parent_1, &parent_2);
    assert!(result.is_err());
}

#[test]
fn test_clone_crossover_identical_parents() {
    let dna = vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }];
    let parent = Chromosome {
        dna: dna.clone(),
        fitness: 5.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = clone_crossover(&parent, &parent).unwrap();
    for child in &offspring {
        for (i, gene) in child.dna.iter().enumerate() {
            assert_eq!(gene.id, dna[i].id);
        }
    }
}

#[test]
fn test_clone_crossover_single_gene() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 42 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 99 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = clone_crossover(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring[0].dna[0].id, 42);
    assert_eq!(offspring[1].dna[0].id, 99);
}

#[test]
fn test_clone_crossover_empty_dna() {
    let parent_1 = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = clone_crossover(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);
    assert!(offspring[0].dna.is_empty());
    assert!(offspring[1].dna.is_empty());
}

#[test]
fn test_clone_crossover_via_enum_dispatch() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = Crossover::Clone.crossover(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].dna[0].id, 1);
    assert_eq!(offspring[0].dna[1].id, 2);
    assert_eq!(offspring[1].dna[0].id, 3);
    assert_eq!(offspring[1].dna[1].id, 4);
}

#[test]
fn test_clone_crossover_via_configuration() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let config = CrossoverConfiguration {
        method: Crossover::Clone,
        ..Default::default()
    };

    let offspring = crossover::factory(&parent_1, &parent_2, config).unwrap();
    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].dna[0].id, 1);
    assert_eq!(offspring[1].dna[0].id, 3);
}
