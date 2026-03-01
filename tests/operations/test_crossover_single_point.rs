#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::operations::crossover::single_point::single_point;

#[test]
fn single_point_crossover_preserves_length() {
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
    };
    let parent_2 = Chromosome {
        dna: vec![
            Gene { id: 7 },
            Gene { id: 8 },
            Gene { id: 9 },
            Gene { id: 10 },
            Gene { id: 11 },
            Gene { id: 12 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let offspring = single_point(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);
    assert_eq!(offspring[0].dna.len(), parent_1.dna.len());
    assert_eq!(offspring[1].dna.len(), parent_2.dna.len());
}

#[test]
fn single_point_crossover_genes_come_from_parents() {
    let parent_1 = Chromosome {
        dna: vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![
            Gene { id: 5 },
            Gene { id: 6 },
            Gene { id: 7 },
            Gene { id: 8 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // Run multiple times to account for randomness
    for _ in 0..20 {
        let offspring = single_point(&parent_1, &parent_2).unwrap();
        let child_1 = &offspring[0];
        let child_2 = &offspring[1];

        // Each gene in child must come from one of the parents at the same position
        for i in 0..4 {
            let from_p1 = child_1.dna[i].id == parent_1.dna[i].id;
            let from_p2 = child_1.dna[i].id == parent_2.dna[i].id;
            assert!(
                from_p1 || from_p2,
                "Child 1 gene at {} not from either parent",
                i
            );

            let from_p1 = child_2.dna[i].id == parent_1.dna[i].id;
            let from_p2 = child_2.dna[i].id == parent_2.dna[i].id;
            assert!(
                from_p1 || from_p2,
                "Child 2 gene at {} not from either parent",
                i
            );
        }
    }
}

#[test]
fn single_point_crossover_error_on_different_lengths() {
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

    assert!(single_point(&parent_1, &parent_2).is_err());
}

#[test]
fn single_point_crossover_error_on_too_short() {
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

    assert!(single_point(&parent_1, &parent_2).is_err());
}
