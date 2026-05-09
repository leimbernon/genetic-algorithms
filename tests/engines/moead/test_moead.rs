//! MOEA/D engine tests.
//!
//! Wave 0 scope: validate() error paths. Plan 36-02 appends run() integration
//! tests once the run() loop is implemented; Plan 36-03 adds the LogObserver
//! integration test.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ObjectiveDirection, ScalarizationFn};
use genetic_algorithms::moead::MoeaDGa;
use genetic_algorithms::LogObserver;
use genetic_algorithms::MoeaDObserver;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_moead_validate_no_init_fn() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_zero_objectives() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(0)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_population_too_small() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(1)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_mismatched_objective_fns() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}

#[test]
fn test_moead_validate_missing_weight_vectors() {
    // D-06: neither auto nor custom called
    let config = MoeaDConfiguration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("weight vectors")));
}

#[test]
fn test_moead_validate_custom_weight_vector_wrong_dimension() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors(vec![vec![0.5, 0.5]]); // 2-dim, not 3
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("dimension")));
}

#[test]
fn test_moead_validate_das_dennis_p_zero() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(0);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("Das-Dennis")));
}

#[test]
fn test_moead_validate_mismatched_objective_directions() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![ObjectiveDirection::Minimize])
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(matches!(
        moead.validate(),
        Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("objective_directions")
    ));
}

#[test]
fn test_moead_validate_passes_with_complete_config() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(moead.validate().is_ok());
}

// ===== Run() integration tests — added by Plan 36-02 =====

/// 3-objective DTLZ2 sphere benchmark (M=3, k=2, n=4 variables for fast tests).
///
/// f_1 = cos(x_1 * pi/2) * cos(x_2 * pi/2) * (1 + g)
/// f_2 = cos(x_1 * pi/2) * sin(x_2 * pi/2) * (1 + g)
/// f_3 = sin(x_1 * pi/2) * (1 + g)
/// g(x) = sum_{i=2}^{n-1} (x_i - 0.5)^2  (zero-indexed: indices 2..n)
fn dtlz2_objectives(
) -> Vec<Box<genetic_algorithms::multi_objective::ObjectiveFn<RangeGenotype<f64>>>> {
    use std::f64::consts::FRAC_PI_2;

    let g_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
        dna[2..].iter().map(|gene| (gene.value - 0.5).powi(2)).sum::<f64>()
    };
    let f1 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let x1 = dna[0].value;
        let x2 = dna[1].value;
        let g = g_fn(dna);
        (x1 * FRAC_PI_2).cos() * (x2 * FRAC_PI_2).cos() * (1.0 + g)
    };
    let f2 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let x1 = dna[0].value;
        let x2 = dna[1].value;
        let g = g_fn(dna);
        (x1 * FRAC_PI_2).cos() * (x2 * FRAC_PI_2).sin() * (1.0 + g)
    };
    let f3 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let x1 = dna[0].value;
        let g = g_fn(dna);
        (x1 * FRAC_PI_2).sin() * (1.0 + g)
    };
    vec![Box::new(f1), Box::new(f2), Box::new(f3)]
}

fn build_test_moead(
    population_size: usize,
    max_generations: usize,
    scalarization: ScalarizationFn,
) -> MoeaDGa<RangeChromosome<f64>> {
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(population_size)
        .with_max_generations(max_generations)
        .with_weight_vectors_auto(4)            // 15 weight vectors for M=3, p=4
        .with_scalarization(scalarization)
        .with_neighborhood_size(5)              // small T for fast tests
        .with_max_neighbor_replacements(2);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 4;
    ga_config.limit_configuration.alleles_can_be_repeated = true;
    ga_config.rng_seed = Some(42);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    MoeaDGa::<RangeChromosome<f64>>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(dtlz2_objectives())
        .build()
        .expect("MoeaD build should succeed with all required builders called")
}

#[test]
fn test_moead_run_produces_pareto_front() {
    // Tchebycheff scalarization, 15 weight vectors (p=4 for M=3).
    let mut moead = build_test_moead(15, 10, ScalarizationFn::Tchebycheff);
    let result = moead.run();
    assert!(result.is_ok(), "MoeaD run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "Pareto front should contain at least one rank-0 individual");
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All ParetoFront members must be rank 0");
        assert_eq!(ind.objectives.len(), 3);
    }
}

#[test]
fn test_moead_run_with_pbi() {
    // PBI scalarization with theta = 5.0 (Zhang & Li default).
    let mut moead = build_test_moead(15, 10, ScalarizationFn::Pbi { theta: 5.0 });
    let result = moead.run();
    assert!(result.is_ok(), "MoeaD run with PBI should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "PBI run should produce a non-empty front");
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0);
        assert_eq!(ind.objectives.len(), 3);
    }
}

#[test]
fn test_moead_run_with_custom_weight_vectors() {
    let custom = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![0.5, 0.5, 0.0],
        vec![0.5, 0.0, 0.5],
        vec![0.0, 0.5, 0.5],
        vec![0.34, 0.33, 0.33],
    ];
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(7)
        .with_max_generations(8)
        .with_weight_vectors(custom)
        .with_scalarization(ScalarizationFn::Tchebycheff)
        .with_neighborhood_size(3)
        .with_max_neighbor_replacements(2);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 4;
    ga_config.limit_configuration.alleles_can_be_repeated = true;
    ga_config.rng_seed = Some(7);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut moead = MoeaDGa::<RangeChromosome<f64>>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(dtlz2_objectives())
        .build()
        .expect("build should succeed with custom weight vectors");

    let result = moead.run();
    assert!(result.is_ok(), "run with custom weight vectors: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
}

/// Counter-based observer to verify that lifecycle hooks fire.
#[derive(Default)]
struct CountingObserver {
    sort_count: AtomicUsize,
    pareto_count: AtomicUsize,
}

impl MoeaDObserver<RangeChromosome<f64>> for CountingObserver {
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
        self.pareto_count.fetch_add(1, Ordering::Relaxed);
    }

    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {
        self.sort_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn test_moead_run_invokes_observer_hooks() {
    let observer = Arc::new(CountingObserver::default());
    let observer_handle = observer.clone();

    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(15)
        .with_max_generations(5)
        .with_weight_vectors_auto(4)
        .with_neighborhood_size(5)
        .with_max_neighbor_replacements(2);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 4;
    ga_config.limit_configuration.alleles_can_be_repeated = true;
    ga_config.rng_seed = Some(123);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut moead = MoeaDGa::<RangeChromosome<f64>>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(dtlz2_objectives())
        .with_observer(
            observer as Arc<dyn MoeaDObserver<RangeChromosome<f64>> + Send + Sync>,
        )
        .build()
        .expect("build should succeed");

    moead.run().expect("run should succeed");

    // Both hooks fire once per generation (5 total) on non-WASM host tests.
    assert_eq!(observer_handle.pareto_count.load(Ordering::Relaxed), 5);
    assert_eq!(observer_handle.sort_count.load(Ordering::Relaxed), 5);
}

#[test]
fn test_moead_run_rejects_differential_mutation() {
    use genetic_algorithms::operations::Mutation;
    let mut moead = build_test_moead(15, 3, ScalarizationFn::Tchebycheff);
    moead.ga_config.mutation_configuration.method = Mutation::Differential;
    moead.ga_config.mutation_configuration.probability_max = Some(1.0);
    let result = moead.run();
    assert!(matches!(result, Err(GaError::MutationError(ref msg)) if msg.contains("Differential mutation is not supported in MOEA/D")));
}

#[test]
fn test_moead_log_observer() {
    // D-12 smoke test: confirm `impl<U> MoeaDObserver<U> for LogObserver` compiles
    // and runs without panic.
    let mut moead = build_test_moead(15, 3, ScalarizationFn::Tchebycheff)
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn MoeaDObserver<RangeChromosome<f64>> + Send + Sync>,
        );

    let result = moead.run();
    assert!(
        result.is_ok(),
        "MoeaD run with LogObserver should succeed: {:?}",
        result.err()
    );
    let front = result.unwrap();
    assert!(!front.is_empty(), "front should be non-empty under LogObserver");
}
