use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;

fn build_f64_chromosome(n: usize) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], 50.0))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

fn build_i32_chromosome(n: usize) -> RangeChromosome<i32> {
    let mut c = RangeChromosome::<i32>::new();
    let dna: Vec<_> = (0..n)
        .map(|i| RangeGenotype::new(i as i32, vec![(0, 100)], 50))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

// --- Creep mutation tests ---

#[test]
fn creep_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.get_dna().to_vec();
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(10.0), None).unwrap();
        if before.iter().zip(c.get_dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Creep mutation via factory did not change any value");
}

#[test]
fn creep_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..100 {
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(5.0), None).unwrap();
        for gene in c.get_dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Creep: value {} out of range [{}, {}]",
                gene.value, lo, hi
            );
        }
    }
}

#[test]
fn creep_mutation_i32_via_factory() {
    let mut c = build_i32_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.get_dna().to_vec();
        mutation::factory_with_params(Mutation::Creep, &mut c, Some(5.0), None).unwrap();
        if before.iter().zip(c.get_dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Creep mutation i32 via factory did not change any value");
}

// --- Gaussian mutation tests ---

#[test]
fn gaussian_mutation_via_factory_changes_value() {
    let mut c = build_f64_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.get_dna().to_vec();
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(10.0)).unwrap();
        if before.iter().zip(c.get_dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Gaussian mutation via factory did not change any value");
}

#[test]
fn gaussian_mutation_via_factory_stays_in_range() {
    let mut c = build_f64_chromosome(8);
    for _ in 0..200 {
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(20.0)).unwrap();
        for gene in c.get_dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Gaussian: value {} out of range [{}, {}]",
                gene.value, lo, hi
            );
        }
    }
}

#[test]
fn gaussian_mutation_i32_via_factory() {
    let mut c = build_i32_chromosome(5);
    let mut changed = false;
    for _ in 0..200 {
        let before = c.get_dna().to_vec();
        mutation::factory_with_params(Mutation::Gaussian, &mut c, None, Some(5.0)).unwrap();
        if before.iter().zip(c.get_dna()).any(|(b, a)| b.value != a.value) {
            changed = true;
            break;
        }
    }
    assert!(changed, "Gaussian mutation i32 via factory did not change any value");
}

