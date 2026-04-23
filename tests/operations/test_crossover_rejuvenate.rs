#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::CrossoverConfiguration,
    fitness::FitnessFnWrapper,
    operations::crossover::{self, rejuvenate::rejuvenate},
    operations::Crossover,
    traits::CrossoverOperator,
};

#[test]
fn test_rejuvenate_resets_age_to_zero() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 10.0,
        age: 5,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 20.0,
        age: 10,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = rejuvenate(&parent_1, &parent_2).unwrap();

    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].age, 0);
    assert_eq!(offspring[1].age, 0);
}

#[test]
fn test_rejuvenate_preserves_dna() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 10.0,
        age: 5,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 20.0,
        age: 10,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = rejuvenate(&parent_1, &parent_2).unwrap();

    for (i, gene) in offspring[0].dna.iter().enumerate() {
        assert_eq!(gene.id, parent_1.dna[i].id);
    }
    for (i, gene) in offspring[1].dna.iter().enumerate() {
        assert_eq!(gene.id, parent_2.dna[i].id);
    }
}

#[test]
fn test_rejuvenate_different_lengths_returns_error() {
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

    let result = rejuvenate(&parent_1, &parent_2);
    assert!(result.is_err());
}

#[test]
fn test_rejuvenate_returns_two_children() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 3,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 7,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = rejuvenate(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);
}

#[test]
fn test_rejuvenate_parents_with_age_zero_remain_zero() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = rejuvenate(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring[0].age, 0);
    assert_eq!(offspring[1].age, 0);
}

#[test]
fn test_rejuvenate_via_enum_dispatch() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 8,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 12,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = Crossover::Rejuvenate
        .crossover(&parent_1, &parent_2)
        .unwrap();
    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].age, 0);
    assert_eq!(offspring[1].age, 0);
    assert_eq!(offspring[0].dna[0].id, 1);
    assert_eq!(offspring[1].dna[0].id, 3);
}

#[test]
fn test_rejuvenate_via_configuration() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 5,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 9,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let config = CrossoverConfiguration {
        method: Crossover::Rejuvenate,
        ..Default::default()
    };

    let offspring = crossover::factory(&parent_1, &parent_2, config).unwrap();
    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].age, 0);
    assert_eq!(offspring[1].age, 0);
}
