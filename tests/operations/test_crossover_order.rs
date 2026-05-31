#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::operations::crossover::order::order;

#[test]
fn order_crossover_preserves_length() {
    let parent_1 = Chromosome {
        dna: vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
            Gene { id: 5 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![
            Gene { id: 3 },
            Gene { id: 5 },
            Gene { id: 1 },
            Gene { id: 4 },
            Gene { id: 2 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    let offspring = order(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].dna.len(), 5);
    assert_eq!(offspring[1].dna.len(), 5);
}

#[test]
fn order_crossover_preserves_all_gene_ids() {
    let parent_1 = Chromosome {
        dna: vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
            Gene { id: 5 },
            Gene { id: 6 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![
            Gene { id: 3 },
            Gene { id: 6 },
            Gene { id: 1 },
            Gene { id: 5 },
            Gene { id: 2 },
            Gene { id: 4 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    // Run multiple times because of randomness
    for _ in 0..50 {
        let offspring = order(&parent_1, &parent_2).unwrap();
        let child_1 = &offspring[0];
        let child_2 = &offspring[1];

        // Each child should contain all gene IDs exactly once (permutation property)
        let mut ids_1: Vec<i32> = child_1.dna.iter().map(|g| g.id).collect();
        let mut ids_2: Vec<i32> = child_2.dna.iter().map(|g| g.id).collect();
        ids_1.sort();
        ids_2.sort();

        assert_eq!(
            ids_1,
            vec![1, 2, 3, 4, 5, 6],
            "Child 1 doesn't have all gene IDs"
        );
        assert_eq!(
            ids_2,
            vec![1, 2, 3, 4, 5, 6],
            "Child 2 doesn't have all gene IDs"
        );
    }
}

#[test]
fn order_crossover_error_on_different_lengths() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    assert!(order(&parent_1, &parent_2).is_err());
}

#[test]
fn order_crossover_error_on_too_short() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }, Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    assert!(order(&parent_1, &parent_2).is_err());
}
