// Wave 1 tests: VectorFitness trait baseline
//
// These tests prove that:
// 1. A chromosome implementing VectorFitness can store and retrieve fitness_values.
// 2. The VectorFitness trait is accessible via the `genetic_algorithms` crate re-export.
//
// References:
//   - TRAITS-01: VectorFitness trait definition (D-01, D-02, D-05)
//   - Plan 55-01, Task 2

use genetic_algorithms::traits::{ChromosomeT, VectorFitness};
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Minimal test chromosome
// ---------------------------------------------------------------------------

/// Minimal gene for the test fixture.
#[derive(Debug, Clone, Default, PartialEq)]
struct VfTestGene {
    id: i32,
}

impl genetic_algorithms::traits::GeneT for VfTestGene {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

/// Minimal chromosome implementing both ChromosomeT and VectorFitness.
#[derive(Debug, Clone, Default)]
struct VfTestChromosome {
    dna: Vec<VfTestGene>,
    fitness: f64,
    age: usize,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for VfTestChromosome {
    type Gene = VfTestGene;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn age(&self) -> usize {
        self.age
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }

    fn calculate_fitness(&mut self) {
        // Not exercised in these tests.
    }
}

impl genetic_algorithms::traits::LinearChromosome for VfTestChromosome {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(s) => s.to_vec(),
            Cow::Owned(v) => v,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
    where
        F: Fn(&[VfTestGene]) -> f64 + Send + Sync + 'static,
    {
        self
    }
}

impl VectorFitness for VfTestChromosome {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_vector_fitness_trait_roundtrip() {
    let mut chromosome = VfTestChromosome::default();
    chromosome.set_fitness_values(vec![1.0, 2.0, 3.0]);
    assert_eq!(
        chromosome.fitness_values(),
        &[1.0, 2.0, 3.0],
        "fitness_values should return the slice previously set via set_fitness_values"
    );
}

#[test]
fn test_vector_fitness_reexport() {
    // Prove that genetic_algorithms::VectorFitness re-export is accessible (D-05).
    fn accepts<U: genetic_algorithms::VectorFitness>(_: &U) {}

    let mut chromosome = VfTestChromosome::default();
    chromosome.set_fitness_values(vec![0.5, 1.5]);
    accepts(&chromosome);
}
