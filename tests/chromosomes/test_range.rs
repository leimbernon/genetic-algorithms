use genetic_algorithms::chromosomes::Range;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::traits::ChromosomeT;

#[test]
fn test_new_range() {
    let chromosome = Range::<i32>::new();
    assert_eq!(chromosome.dna.len(), 0);
    assert_eq!(chromosome.fitness, 0.0);
    assert_eq!(chromosome.age, 0);
}

#[test]
fn test_phenotype() {
    let mut chromosome = Range::<i32>::new();
    chromosome.dna.push(RangeGenotype{id: 10, ranges: vec![(0, 10)], value: 5});
    let phenotype = chromosome.phenotype();
    assert_eq!(phenotype, "5");
}

#[test]
fn test_set_dna() {
    let mut chromosome = Range::<i32>::new();
    let dna = vec![RangeGenotype{id: 10, ranges: vec![(0, 10)], value: 5},
                    RangeGenotype{id: 20, ranges: vec![(10, 20)], value: 15}];
    use std::borrow::Cow;
    chromosome.set_dna(Cow::Borrowed(&dna));
    assert_eq!(chromosome.get_dna(), &dna[..]);
}

#[test]
fn test_set_fitness_fn() {
    let mut chromosome = Range::<i32>::new();
    chromosome.set_fitness_fn(|dna| dna.len() as f64);
    chromosome.calculate_fitness();
    assert_eq!(chromosome.get_fitness(), 0.0);

    let dna = vec![RangeGenotype::new(0, vec![(0, 10)], 0)];
    use std::borrow::Cow;
    chromosome.set_dna(Cow::Borrowed(dna.as_slice()));
    chromosome.calculate_fitness();
    assert_eq!(chromosome.get_fitness(), 1.0);
}

#[test]
fn test_set_fitness() {
    let mut chromosome = Range::<i32>::new();
    chromosome.set_fitness(42.0);
    assert_eq!(chromosome.get_fitness(), 42.0);
}

#[test]
fn test_set_age() {
    let mut chromosome = Range::<i32>::new();
    chromosome.set_age(5);
    assert_eq!(chromosome.get_age(), 5);
}