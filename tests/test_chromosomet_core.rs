// Wave 0 tests: minimal ChromosomeT contract
//
// These tests prove that a custom type can implement `ChromosomeT` using
// ONLY the evaluation contract (fitness, age, calculate_fitness,
// fitness_distance) without implementing any flat-slice / DNA methods.
//
// References:
//   - ARCH-01: User can implement ChromosomeT with only 5 methods (no dna)
//   - D-01: ChromosomeT retains only fitness/set_fitness/calculate_fitness/age/set_age/fitness_distance
//
// RED state expected until plan 47-01 Task 2 shrinks ChromosomeT.

use genetic_algorithms::traits::{ChromosomeT, GeneT};

/// Minimal test gene — only needs GeneT.
#[derive(Debug, Clone, Default)]
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

/// Minimal chromosome that implements ONLY the ChromosomeT evaluation contract.
/// It has NO dna field, NO set_dna, NO dna_mut, NO set_fitness_fn.
/// Compilation of this struct proves ARCH-01: ChromosomeT is a pure evaluation contract.
#[derive(Debug, Clone, Default)]
struct MinimalChromo {
    fitness: f64,
    age: usize,
}

impl ChromosomeT for MinimalChromo {
    type Gene = TestGene;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn calculate_fitness(&mut self) {
        // Minimal implementation: just set a constant fitness value.
        self.fitness = 42.0;
    }

    fn age(&self) -> usize {
        self.age
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }
}

/// Test: MinimalChromo implements ChromosomeT with no dna methods required.
/// Proves that fitness_distance default impl works and setters return &mut Self (chainable).
#[test]
fn test_chromosomet_core_minimal_impl() {
    let mut c = MinimalChromo::default();

    // Setters are chainable (return &mut Self)
    c.set_fitness(5.0).set_age(3);

    assert_eq!(c.fitness(), 5.0);
    assert_eq!(c.age(), 3);

    // Default fitness_distance implementation: |target - fitness|
    let distance = c.fitness_distance(&0.0);
    assert!(
        (distance - 5.0).abs() < 1e-10,
        "expected 5.0, got {distance}"
    );

    let distance_neg = c.fitness_distance(&10.0);
    assert!(
        (distance_neg - 5.0).abs() < 1e-10,
        "expected 5.0, got {distance_neg}"
    );
}

/// Test: MinimalChromo has no dna(), dna_mut(), set_dna(), or set_fitness_fn() methods.
/// Compilation of this test alone is the proof (the struct definition above has no such fields).
/// The test body just exercises calculate_fitness() and reads fitness() to ensure they work.
#[test]
fn test_chromosomet_core_no_dna_required() {
    let mut c = MinimalChromo::default();
    c.calculate_fitness();
    assert_eq!(c.fitness(), 42.0);

    // MinimalChromo compiles WITHOUT any dna-related methods —
    // this test cannot call c.dna() because there is no such method.
    // The absence of a compiler error IS the proof.
}
