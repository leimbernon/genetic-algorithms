// Wave 0 tests: LinearChromosome supertrait contract
//
// These tests prove that:
// 1. A custom type can implement LinearChromosome and gain default set_gene / reset impls.
// 2. set_gene bounds-checks (out-of-bounds returns self unchanged, logs warn).
// 3. reset() sets fitness to NaN, age to 0, dna to empty slice.
// 4. new_gene() returns TestGene::new() via the default implementation.
//
// References:
//   - ARCH-02: LinearChromosome supertrait with flat-slice contract + default set_gene/reset
//   - D-02: LinearChromosome adds dna/dna_mut/set_dna/set_fitness_fn/new_gene + default set_gene/reset
//   - D-03: reset() -> &mut Self replaces old default(self) -> Self helper
//
// RED state expected until plan 47-01 Task 2 creates LinearChromosome trait.

use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};
use std::borrow::Cow;

/// Minimal test gene — only needs GeneT.
#[derive(Debug, Clone, Default, PartialEq)]
struct TestGene {
    id: i32,
}

impl GeneT for TestGene {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

/// A chromosome that implements both ChromosomeT and LinearChromosome.
/// Uses a Vec<TestGene> as the backing DNA store.
/// The default impls of set_gene() and reset() are inherited from LinearChromosome.
#[derive(Debug, Clone, Default)]
struct LinearChromo {
    dna: Vec<TestGene>,
    fitness: f64,
    age: usize,
}

impl ChromosomeT for LinearChromo {
    type Gene = TestGene;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn calculate_fitness(&mut self) {
        self.fitness = self.dna.len() as f64;
    }

    fn age(&self) -> usize {
        self.age
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }
}

impl LinearChromosome for LinearChromo {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        // Minimal stub for test purposes — not exercised in these tests.
        self
    }
}

/// Test: set_gene in-bounds updates dna[0].
#[test]
fn test_linear_chromosome_default_set_gene_in_bounds() {
    let initial = vec![TestGene { id: 10 }, TestGene { id: 20 }];
    let mut c = LinearChromo {
        dna: initial,
        fitness: 0.0,
        age: 0,
    };

    let new_gene = TestGene { id: 99 };
    c.set_gene(0, new_gene.clone());

    assert_eq!(c.dna()[0], new_gene, "set_gene should update dna[0]");
    // dna[1] is untouched
    assert_eq!(c.dna()[1].id, 20, "set_gene should not touch other indices");
}

/// Test: set_gene out-of-bounds returns self unchanged (no panic, no mutation).
#[test]
fn test_linear_chromosome_default_set_gene_out_of_bounds() {
    let initial = vec![TestGene { id: 5 }];
    let mut c = LinearChromo {
        dna: initial,
        fitness: 0.0,
        age: 0,
    };

    // index 5 is out of bounds for a 1-element dna
    c.set_gene(5, TestGene { id: 99 });

    // dna must be unchanged
    assert_eq!(c.dna().len(), 1, "DNA length must not change");
    assert_eq!(c.dna()[0].id, 5, "DNA content must not change on OOB index");
}

/// Test: reset() sets fitness to NaN, age to 0, dna to empty.
#[test]
fn test_linear_chromosome_default_reset() {
    let initial = vec![TestGene { id: 1 }, TestGene { id: 2 }];
    let mut c = LinearChromo {
        dna: initial,
        fitness: 99.0,
        age: 7,
    };

    let result = c.reset();

    assert!(result.fitness().is_nan(), "reset() must set fitness to NaN");
    assert_eq!(result.age(), 0, "reset() must set age to 0");
    assert!(
        result.dna().is_empty(),
        "reset() must set dna to empty slice"
    );
}

/// Test: new_gene() returns TestGene::new() via the default LinearChromosome impl.
#[test]
fn test_linear_chromosome_new_gene_default() {
    let gene = LinearChromo::new_gene();
    let expected = TestGene::new();
    assert_eq!(
        gene.id(),
        expected.id(),
        "new_gene() must delegate to TestGene::new()"
    );
}
