use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::nsga2::{binary_tournament, IslandNsga2Ga};
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::pareto::ParetoIndividual;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome};
use std::borrow::Cow;

#[test]
fn test_island_nsga2_validate_zero_islands() {
    let island_config = IslandConfiguration::new().with_num_islands(0);
    let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let ga = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

    assert!(ga.validate().is_err());
}

#[test]
fn test_island_nsga2_validate_zero_objectives() {
    let island_config = IslandConfiguration::new().with_num_islands(2);
    let nsga2_config = Nsga2Configuration::new().with_num_objectives(0);
    let ga_config = GaConfiguration::default();
    let ga = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);

    assert!(ga.validate().is_err());
}

#[test]
fn test_island_nsga2_validate_no_init_fn() {
    let island_config = IslandConfiguration::new().with_num_islands(2);
    let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let ga = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

    assert!(ga.validate().is_err());
}

#[test]
fn test_island_nsga2_validate_mismatched_objectives() {
    let island_config = IslandConfiguration::new().with_num_islands(2);
    let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();
    let ga = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]); // only 1, need 2

    assert!(ga.validate().is_err());
}

#[test]
fn test_island_nsga2_validate_migration_count_exceeds_pop() {
    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(200);
    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(100);
    let ga_config = GaConfiguration::default();
    let ga = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

    assert!(ga.validate().is_err());
}

#[test]
fn test_island_nsga2_validate_ok() {
    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(2);
    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20);
    let ga_config = GaConfiguration::default();
    let ga = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)]);

    assert!(ga.validate().is_ok());
}

#[test]
fn test_island_nsga2_build_ok() {
    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(2);
    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20);
    let ga_config = GaConfiguration::default();
    let result = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0)])
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_island_nsga2_build_fails_invalid() {
    let island_config = IslandConfiguration::new().with_num_islands(0);
    let nsga2_config = Nsga2Configuration::new();
    let ga_config = GaConfiguration::default();
    let result = IslandNsga2Ga::<Binary>::new(island_config, nsga2_config, ga_config).build();

    assert!(result.is_err());
}

#[test]
fn test_binary_tournament_prefers_lower_rank() {
    #[derive(Debug, Clone, Default)]
    struct SimpleChrom {
        dna: Vec<genetic_algorithms::genotypes::Binary>,
    }

    impl ChromosomeT for SimpleChrom {
        type Gene = genetic_algorithms::genotypes::Binary;
        fn calculate_fitness(&mut self) {}
        fn fitness(&self) -> f64 {
            0.0
        }
        fn set_fitness(&mut self, _: f64) -> &mut Self {
            self
        }
        fn set_age(&mut self, _: usize) -> &mut Self {
            self
        }
        fn age(&self) -> usize {
            0
        }
    }

    impl LinearChromosome for SimpleChrom {
        fn dna(&self) -> &[Self::Gene] {
            &self.dna
        }
        fn dna_mut(&mut self) -> &mut [Self::Gene] {
            &mut self.dna
        }
        fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
            self.dna = dna.into_owned();
            self
        }
        fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
        where
            F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
        {
            self
        }
    }

    let pop = vec![
        ParetoIndividual {
            chromosome: <SimpleChrom as Default>::default(),
            objectives: vec![1.0],
            rank: 2,
            crowding_distance: 5.0,
            constraint_violation: 0.0,
        },
        ParetoIndividual {
            chromosome: <SimpleChrom as Default>::default(),
            objectives: vec![1.0],
            rank: 0,
            crowding_distance: 1.0,
            constraint_violation: 0.0,
        },
    ];

    // With only 2 individuals, tournament always picks between index 0 and 1.
    // rank 0 < rank 2, so index 1 should be preferred.
    let mut wins = [0usize; 2];
    let mut rng = genetic_algorithms::rng::make_rng();
    for _ in 0..100 {
        let idx = binary_tournament(&pop, &mut rng);
        wins[idx] += 1;
    }
    // The rank-0 individual (index 1) should win the majority
    assert!(
        wins[1] > wins[0],
        "Rank-0 individual should win more often: wins={:?}",
        wins
    );
}
