use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::island::configuration::{IslandConfiguration, MigrationPolicy};
use genetic_algorithms::island::migration::{migrate, migrate_pareto};
use genetic_algorithms::island::topology::MigrationTopology;
use genetic_algorithms::nsga2::pareto::ParetoIndividual;
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};
use std::borrow::Cow;

/// Simple test gene satisfying `GeneT`.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
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

/// Simple test chromosome for migration tests.
#[derive(Debug, Clone, Default, PartialEq)]
struct MigrationTestChromosome {
    dna: Vec<TestGene>,
    fitness: f64,
    age: usize,
}

impl ChromosomeT for MigrationTestChromosome {
    type Gene = TestGene;

    fn calculate_fitness(&mut self) {}

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }

    fn age(&self) -> usize {
        self.age
    }
}

impl LinearChromosome for MigrationTestChromosome {
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

    fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self
    }
}

fn make_population(fitnesses: &[f64]) -> Population<MigrationTestChromosome> {
    let chromosomes: Vec<MigrationTestChromosome> = fitnesses
        .iter()
        .map(|&f| MigrationTestChromosome {
            dna: vec![],
            fitness: f,
            age: 0,
        })
        .collect();
    Population::new(chromosomes)
}

fn make_pareto_individual(rank: usize, crowding: f64) -> ParetoIndividual<MigrationTestChromosome> {
    let mut ind = ParetoIndividual::new(
        MigrationTestChromosome {
            dna: vec![],
            fitness: 0.0,
            age: 0,
        },
        vec![],
    );
    ind.rank = rank;
    ind.crowding_distance = crowding;
    ind
}

#[test]
fn test_migrate_single_island_is_noop() {
    let mut islands = vec![make_population(&[1.0, 2.0, 3.0])];
    let config = IslandConfiguration::new()
        .with_num_islands(1)
        .with_migration_count(1);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_ok());
}

#[test]
fn test_migrate_empty_island_returns_error() {
    let mut islands = vec![make_population(&[1.0, 2.0]), Population::new_empty()];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_err());
}

#[test]
fn test_migrate_count_exceeds_size_returns_error() {
    let mut islands = vec![make_population(&[1.0]), make_population(&[2.0])];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(5);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_err());
}

#[test]
fn test_migrate_ring_topology_transfers_best() {
    let mut islands = vec![
        make_population(&[10.0, 20.0, 30.0]),
        make_population(&[100.0, 200.0, 300.0]),
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_ok());

    let island1_fitnesses: Vec<f64> = islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
    assert!(
        island1_fitnesses.contains(&10.0),
        "Island 1 should contain migrated individual with fitness 10.0"
    );
}

// ---- Pareto migration tests ----

#[test]
fn test_migrate_pareto_single_island_noop() {
    let mut islands = vec![vec![make_pareto_individual(0, 1.0)]];
    let config = IslandConfiguration::new()
        .with_num_islands(1)
        .with_migration_count(1);

    let result = migrate_pareto(&mut islands, &config);
    assert!(result.is_ok());
}

#[test]
fn test_migrate_pareto_empty_island_error() {
    let mut islands: Vec<Vec<ParetoIndividual<MigrationTestChromosome>>> =
        vec![vec![make_pareto_individual(0, 1.0)], vec![]];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1);

    let result = migrate_pareto(&mut islands, &config);
    assert!(result.is_err());
}

#[test]
fn test_migrate_pareto_count_exceeds_size_error() {
    let mut islands = vec![
        vec![make_pareto_individual(0, 1.0)],
        vec![make_pareto_individual(1, 0.5)],
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(5);

    let result = migrate_pareto(&mut islands, &config);
    assert!(result.is_err());
}

#[test]
fn test_migrate_pareto_ring_transfers_best() {
    let mut islands = vec![
        vec![
            make_pareto_individual(0, 2.0),
            make_pareto_individual(1, 1.0),
            make_pareto_individual(2, 0.5),
        ],
        vec![
            make_pareto_individual(2, 0.1),
            make_pareto_individual(2, 0.2),
            make_pareto_individual(2, 0.3),
        ],
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring);

    let result = migrate_pareto(&mut islands, &config);
    assert!(result.is_ok());

    let has_rank_0 = islands[1].iter().any(|ind| ind.rank == 0);
    assert!(
        has_rank_0,
        "Island 1 should contain a migrated rank-0 individual"
    );
}

#[test]
fn test_migrate_pareto_replaces_worst() {
    let mut islands = vec![
        vec![
            make_pareto_individual(0, 10.0),
            make_pareto_individual(0, 5.0),
            make_pareto_individual(1, 1.0),
        ],
        vec![
            make_pareto_individual(0, 3.0),
            make_pareto_individual(1, 2.0),
            make_pareto_individual(3, 0.01),
        ],
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring);

    let result = migrate_pareto(&mut islands, &config);
    assert!(result.is_ok());

    let has_rank_3 = islands[1].iter().any(|ind| ind.rank == 3);
    assert!(
        !has_rank_3,
        "Island 1 should no longer contain the rank-3 individual"
    );
}

// ---- Migration policy tests ----

#[test]
fn test_migrate_random_replace_worst_policy() {
    let mut islands = vec![
        make_population(&[10.0, 20.0, 30.0]),
        make_population(&[100.0, 200.0, 300.0]),
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring)
        .with_migration_policy(MigrationPolicy::RandomReplaceWorst);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_ok());

    let island1_fitnesses: Vec<f64> = islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
    let has_original_worst = island1_fitnesses.contains(&300.0);
    let has_migrant = island1_fitnesses
        .iter()
        .any(|f| [10.0, 20.0, 30.0].contains(f));
    assert!(
        !has_original_worst || has_migrant,
        "Worst individual should be replaced or a migrant should be present"
    );
}

#[test]
fn test_migrate_random_replace_random_policy() {
    let mut islands = vec![
        make_population(&[10.0, 20.0, 30.0]),
        make_population(&[100.0, 200.0, 300.0]),
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring)
        .with_migration_policy(MigrationPolicy::RandomReplaceRandom);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_ok());

    let island1_fitnesses: Vec<f64> = islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
    let has_migrant = island1_fitnesses
        .iter()
        .any(|f| [10.0, 20.0, 30.0].contains(f));
    assert!(
        has_migrant,
        "Island 1 should contain a migrant from island 0"
    );
}

#[test]
fn test_migrate_tournament_policy() {
    let mut islands = vec![
        make_population(&[10.0, 20.0, 30.0]),
        make_population(&[100.0, 200.0, 300.0]),
    ];
    let config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring)
        .with_migration_policy(MigrationPolicy::TournamentMigrant);

    let result = migrate(&mut islands, &config, ProblemSolving::Minimization);
    assert!(result.is_ok());

    let island1_fitnesses: Vec<f64> = islands[1].chromosomes.iter().map(|c| c.fitness()).collect();
    let has_migrant = island1_fitnesses
        .iter()
        .any(|f| [10.0, 20.0, 30.0].contains(f));
    assert!(
        has_migrant,
        "Island 1 should contain a tournament-selected migrant"
    );
}
