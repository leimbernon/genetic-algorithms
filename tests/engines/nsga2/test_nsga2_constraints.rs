use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::operations::{Crossover, Mutation, Selection};
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig};

#[test]
fn test_nsga2_with_constraints() {
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 10_i32)], 0)];
    let alleles_clone = alleles.clone();

    let constraint = |dna: &[RangeGene<i32>]| {
        let val = dna[0].value();
        (5.0 - val as f64).max(0.0)
    };

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(50)
        .with_max_generations(30);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(3))
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap);

    let mut nsga2 = Nsga2Ga::<RangeChromosome<i32>>::new(nsga2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_objective_fns(vec![
            Box::new(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum()),
            Box::new(|dna: &[RangeGene<i32>]| dna.iter().map(|g| (10 - g.value()) as f64).sum()),
        ])
        .with_constraint_fns(vec![
            Box::new(constraint) as Box<dyn Fn(&[RangeGene<i32>]) -> f64 + Send + Sync>,
        ]);

    let result = nsga2.run();
    assert!(
        result.is_ok(),
        "NSGA-II with constraints should succeed, got: {:?}",
        result.err()
    );

    let front = result.unwrap();
    assert!(
        !front.individuals.is_empty(),
        "Pareto front should have individuals"
    );
}
