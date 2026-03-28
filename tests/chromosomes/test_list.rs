use genetic_algorithms::chromosomes::ListChromosome;
use genetic_algorithms::configuration::{CrossoverConfiguration, ProblemSolving};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::List;
use genetic_algorithms::initializers::list_random_initialization;
use genetic_algorithms::operations::crossover;
use genetic_algorithms::operations::mutation;
use genetic_algorithms::operations::mutation::swap;
use genetic_algorithms::operations::mutation::inversion;
use genetic_algorithms::operations::mutation::scramble;
use genetic_algorithms::operations::mutation::insertion;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, GeneT, MutationConfig, SelectionConfig, StoppingConfig,
};
use std::borrow::Cow;

/// Helper: create a ListChromosome<char> with `n` genes, alleles ['a','b','c','d'],
/// each gene starting at allele index 0.
fn make_list_chromosome(n: usize) -> ListChromosome<char> {
    let mut c = ListChromosome::<char>::new();
    for _ in 0..n {
        c.dna
            .push(List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap());
    }
    c
}

// ── Mutation: Swap ────────────────────────────────────────────────────────────

#[test]
fn test_list_swap_mutation() {
    // Give each gene a distinct id so a position swap is observable
    let mut c = ListChromosome::<char>::new();
    c.dna = vec![
        List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(1, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(2, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(3, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
    ];
    let before_ids: Vec<i32> = c.dna.iter().map(|g| g.id).collect();
    let before_multiset: Vec<i32> = {
        let mut v = before_ids.clone();
        v.sort();
        v
    };

    swap(&mut c);

    assert_eq!(c.dna().len(), 5, "DNA length must be unchanged after swap");

    // Swap preserves the multiset of ids (no values added or removed)
    let after_multiset: Vec<i32> = {
        let mut v: Vec<i32> = c.dna.iter().map(|g| g.id).collect();
        v.sort();
        v
    };
    assert_eq!(
        before_multiset, after_multiset,
        "swap must not change the multiset of gene ids"
    );
}

// ── Mutation: Inversion ───────────────────────────────────────────────────────

#[test]
fn test_list_inversion_mutation() {
    let mut c = make_list_chromosome(5);
    inversion(&mut c);
    assert_eq!(c.dna().len(), 5, "DNA length must be unchanged after inversion");
}

// ── Mutation: Scramble ────────────────────────────────────────────────────────

#[test]
fn test_list_scramble_mutation() {
    let mut c = make_list_chromosome(5);
    scramble(&mut c);
    assert_eq!(c.dna().len(), 5, "DNA length must be unchanged after scramble");
}

// ── Mutation: Insertion ───────────────────────────────────────────────────────

#[test]
fn test_list_insertion_mutation() {
    let mut c = make_list_chromosome(5);
    let result = insertion::insertion_mutation(&mut c);
    assert!(result.is_ok(), "insertion_mutation should return Ok(())");
    assert_eq!(c.dna().len(), 5, "DNA length must be unchanged after insertion");
}

// ── Mutation: ListValue via factory ──────────────────────────────────────────

#[test]
fn test_list_value_mutation_via_factory() {
    for seed in 0..10u64 {
        genetic_algorithms::rng::set_seed(Some(seed));
        let mut c = make_list_chromosome(5);
        // Start with all ids at 0; any ListValue call must change exactly 1
        let original_ids: Vec<i32> = c.dna.iter().map(|g| g.id).collect();

        let result = mutation::factory(Mutation::ListValue, &mut c);

        assert!(result.is_ok(), "factory(ListValue) should return Ok(()) (seed {})", seed);
        let changed = c
            .dna
            .iter()
            .enumerate()
            .filter(|(i, g)| g.id != original_ids[*i])
            .count();
        assert_eq!(changed, 1, "ListValue must change exactly 1 gene (seed {})", seed);
    }
    genetic_algorithms::rng::set_seed(None);
}

// ── Crossover: SinglePoint ────────────────────────────────────────────────────

#[test]
fn test_list_crossover_single_point() {
    let parent1 = make_list_chromosome(5);
    let mut parent2 = make_list_chromosome(5);
    // Give parent2 different allele indices for variety
    parent2.dna = vec![
        List::new(1, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(2, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(3, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(1, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(2, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
    ];

    let config = CrossoverConfiguration {
        method: Crossover::SinglePoint,
        ..Default::default()
    };
    let result = crossover::factory(&parent1, &parent2, config);

    assert!(result.is_ok(), "SinglePoint crossover should succeed");
    let offspring = result.unwrap();
    assert!(!offspring.is_empty(), "Should produce offspring");
    for child in &offspring {
        assert_eq!(child.dna().len(), 5, "Offspring DNA length must match parents");
    }
}

// ── Crossover: Uniform ────────────────────────────────────────────────────────

#[test]
fn test_list_crossover_uniform() {
    let parent1 = make_list_chromosome(5);
    let mut parent2 = make_list_chromosome(5);
    parent2.dna = vec![
        List::new(3, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(2, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(1, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
        List::new(2, vec!['a', 'b', 'c', 'd'], 'a').unwrap(),
    ];

    let config = CrossoverConfiguration {
        method: Crossover::Uniform,
        ..Default::default()
    };
    let result = crossover::factory(&parent1, &parent2, config);

    assert!(result.is_ok(), "Uniform crossover should succeed");
    let offspring = result.unwrap();
    for child in &offspring {
        assert_eq!(child.dna().len(), 5, "Offspring DNA length must match parents");
    }
}

// ── Initializer roundtrip ─────────────────────────────────────────────────────

#[test]
fn test_list_initialization_roundtrip() {
    let templates = vec![List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap()];
    let dna = list_random_initialization(4, Some(&templates), None);

    assert_eq!(dna.len(), 4, "initializer must produce correct number of genes");

    let mut c = ListChromosome::<char>::new();
    c.set_dna(Cow::Owned(dna));
    c.set_fitness_fn(|genes| genes.len() as f64);
    c.calculate_fitness();

    assert_eq!(c.fitness(), 4.0, "fitness function should run on initialized chromosome");
}

// ── Serde roundtrip (feature-gated) ──────────────────────────────────────────

#[cfg(feature = "serde")]
#[test]
fn test_list_serde_roundtrip() {
    let mut c = make_list_chromosome(3);
    c.set_fitness(7.5);
    c.set_age(2);

    let json = serde_json::to_string(&c).expect("serialize ListChromosome");
    let restored: ListChromosome<char> = serde_json::from_str(&json).expect("deserialize ListChromosome");

    assert_eq!(restored.dna.len(), 3, "dna length must be preserved");
    assert_eq!(restored.fitness(), 7.5, "fitness must be preserved");
    assert_eq!(restored.age(), 2, "age must be preserved");
    for (orig, rest) in c.dna.iter().zip(restored.dna.iter()) {
        assert_eq!(orig.id, rest.id, "gene id must be preserved");
        assert_eq!(orig.value, rest.value, "gene value must be preserved");
        assert_eq!(orig.alleles, rest.alleles, "allele set must be preserved");
    }
}

// ── Full GA run ───────────────────────────────────────────────────────────────

#[test]
fn test_list_full_ga_run() {
    // Build a small population: 10 chromosomes, 4 genes each
    let template = List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap();
    let templates = vec![template];

    let mut chromosomes: Vec<ListChromosome<char>> = Vec::new();
    for _ in 0..10 {
        let mut c = ListChromosome::<char>::new();
        let dna = list_random_initialization(4, Some(&templates), None);
        c.set_dna(Cow::Owned(dna));
        chromosomes.push(c);
    }

    let population = Population::new(chromosomes);

    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::SinglePoint)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5)
        .with_population(population)
        .with_fitness_fn(|dna: &[List<char>]| {
            // Fitness = count of genes where value == 'a'
            dna.iter().filter(|g| g.value == 'a').count() as f64
        });
    let result = ga.run();

    assert!(result.is_ok(), "full GA run must succeed: {:?}", result.err());
    let final_pop = result.unwrap();
    assert!(
        final_pop.best_chromosome.fitness() >= 0.0,
        "best fitness must be >= 0.0"
    );
}

// ── Unit tests migrated from src/chromosomes/list.rs ─────────────────────────

fn make_gene(id: i32, alleles: Vec<char>) -> List<char> {
    List::new(id, alleles, 'a').unwrap()
}

#[test]
fn list_chromosome_new_has_empty_dna_nan_fitness_age_zero() {
    let c = ListChromosome::<char>::new();
    assert!(c.dna().is_empty());
    assert!(c.fitness().is_nan());
    assert_eq!(c.age(), 0);
}

#[test]
fn list_chromosome_default_same_as_new() {
    let c: ListChromosome<char> = Default::default();
    assert!(c.dna().is_empty());
    assert!(c.fitness().is_nan());
    assert_eq!(c.age(), 0);
}

#[test]
fn list_chromosome_set_dna_owned() {
    let mut c = ListChromosome::<char>::new();
    let genes = vec![make_gene(0, vec!['a', 'b']), make_gene(1, vec!['a', 'b'])];
    c.set_dna(Cow::Owned(genes.clone()));
    assert_eq!(c.dna().len(), 2);
    assert_eq!(c.dna()[0].id(), 0);
    assert_eq!(c.dna()[1].id(), 1);
}

#[test]
fn list_chromosome_set_dna_borrowed() {
    let mut c = ListChromosome::<char>::new();
    let genes = vec![make_gene(0, vec!['x', 'y', 'z'])];
    c.set_dna(Cow::Borrowed(&genes));
    assert_eq!(c.dna().len(), 1);
    assert_eq!(c.dna()[0].value(), 'x');
}

#[test]
fn list_chromosome_dna_mut_modifications_visible() {
    let mut c = ListChromosome::<char>::new();
    c.dna.push(make_gene(0, vec!['a', 'b', 'c']));
    c.dna_mut()[0].set_id(2);
    assert_eq!(c.dna()[0].value(), 'c');
}

#[test]
fn list_chromosome_set_fitness_and_get() {
    let mut c = ListChromosome::<char>::new();
    c.set_fitness(42.0);
    assert_eq!(c.fitness(), 42.0);
}

#[test]
fn list_chromosome_set_age_and_get() {
    let mut c = ListChromosome::<char>::new();
    c.set_age(5);
    assert_eq!(c.age(), 5);
}

#[test]
fn list_chromosome_calculate_fitness_using_fn() {
    let mut c = ListChromosome::<char>::new();
    c.dna.push(make_gene(0, vec!['a', 'b']));
    c.dna.push(make_gene(1, vec!['a', 'b']));
    c.set_fitness_fn(|dna| dna.len() as f64);
    c.calculate_fitness();
    assert_eq!(c.fitness(), 2.0);
}

#[test]
fn list_chromosome_clone_is_independent() {
    let mut c = ListChromosome::<char>::new();
    c.dna.push(make_gene(0, vec!['a', 'b', 'c']));
    c.set_fitness(1.0);
    let mut cloned = c.clone();
    cloned.set_fitness(99.0);
    cloned.dna.push(make_gene(1, vec!['a', 'b', 'c']));
    assert_eq!(c.fitness(), 1.0);
    assert_eq!(c.dna().len(), 1);
}

#[test]
fn list_chromosome_phenotype_formats_values() {
    let mut c = ListChromosome::<char>::new();
    c.dna.push(make_gene(0, vec!['a', 'b', 'c'])); // value = 'a'
    c.dna.push(make_gene(2, vec!['a', 'b', 'c'])); // value = 'c'
    let p = c.phenotype();
    assert_eq!(p, "'a', 'c'");
}

#[test]
fn list_chromosome_display_format() {
    let mut c = ListChromosome::<char>::new();
    c.dna.push(make_gene(0, vec!['a', 'b'])); // value 'a'
    c.set_fitness(std::f64::consts::PI);
    let s = format!("{}", c);
    assert!(s.contains("fitness="), "display was: {}", s);
}

#[cfg(feature = "serde")]
#[test]
fn list_chromosome_serde_roundtrip() {
    let mut c = ListChromosome::<char>::new();
    c.dna.push(make_gene(1, vec!['a', 'b', 'c']));
    c.set_fitness(7.0);
    c.set_age(3);
    let json = serde_json::to_string(&c).expect("serialize");
    let restored: ListChromosome<char> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.dna.len(), 1);
    assert_eq!(restored.dna[0].id(), 1);
    assert_eq!(restored.fitness(), 7.0);
    assert_eq!(restored.age(), 3);
    // fitness_fn is skipped — that's OK
}
