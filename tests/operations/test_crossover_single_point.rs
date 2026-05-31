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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    assert!(single_point(&parent_1, &parent_2).is_err());
}

#[test]
fn single_point_crossover_children_start_with_fresh_metadata() {
    // CLONE-02: children must be built via U::new(), not parent.clone().
    // The observable consequence is that children have age==0 and fitness==0.0
    // even when parents carry non-zero age and non-zero fitness.
    let mut parent_1 = Chromosome {
        dna: vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
        ],
        fitness: 99.0,
        age: 5,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let mut parent_2 = Chromosome {
        dna: vec![
            Gene { id: 5 },
            Gene { id: 6 },
            Gene { id: 7 },
            Gene { id: 8 },
        ],
        fitness: 77.0,
        age: 5,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    // Confirm parents have the non-zero metadata we expect
    assert_eq!(parent_1.age, 5);
    assert_eq!(parent_2.age, 5);

    let offspring = single_point(&parent_1, &parent_2).unwrap();
    assert_eq!(offspring.len(), 2);

    // Children must start with fresh metadata regardless of parent age/fitness.
    // age == 0 proves U::new() was used instead of parent.clone().
    assert_eq!(
        offspring[0].age, 0,
        "child_1 must have age=0 (U::new()), not inherit parent age {}",
        parent_1.age
    );
    assert_eq!(
        offspring[1].age, 0,
        "child_2 must have age=0 (U::new()), not inherit parent age {}",
        parent_2.age
    );

    // Suppress unused-mut warnings from the compiler; parents are set up this way
    // to mirror real GA usage where parents accumulate age over generations.
    let _ = &mut parent_1;
    let _ = &mut parent_2;
}

#[test]
fn single_point_crossover_error_on_too_short() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };

    assert!(single_point(&parent_1, &parent_2).is_err());
}
