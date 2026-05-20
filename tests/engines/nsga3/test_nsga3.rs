//! NSGA-III engine tests.
//!
//! Wave 0 scope: validate() error paths only. The full engine integration
//! test (run() produces a non-empty Pareto front on a 3-objective problem)
//! is added in Plan 35-03 once the run() loop is implemented.

use genetic_algorithms::traits::ConfigurationT;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
use genetic_algorithms::nsga3::Nsga3Ga;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_nsga3_validate_no_init_fn() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_zero_objectives() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(0)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_population_too_small() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(1)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_mismatched_objective_fns() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0)]); // only 1, expected 3
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(_))));
}

#[test]
fn test_nsga3_validate_missing_reference_points() {
    // Neither with_reference_points_auto nor with_reference_points was called.
    let config = Nsga3Configuration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(msg)) if msg.contains("reference points")));
}

#[test]
fn test_nsga3_validate_custom_reference_point_wrong_dimension() {
    // num_objectives=3 but custom point has length 2 -> should fail validate.
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points(vec![vec![0.5, 0.5]]); // 2-dim, not 3
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = nsga3.validate();
    assert!(matches!(result, Err(GaError::InvalidNsga3Configuration(msg)) if msg.contains("dimension")));
}

#[test]
fn test_nsga3_validate_passes_with_complete_config() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(nsga3.validate().is_ok());
}

#[test]
fn test_nsga3_validate_mismatched_objective_directions() {
    // objective_directions has 1 entry but num_objectives is 3 — should fail.
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_objective_directions(vec![ObjectiveDirection::Minimize]) // 1 != 3
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    assert!(matches!(
        nsga3.validate(),
        Err(GaError::InvalidNsga3Configuration(ref msg)) if msg.contains("objective_directions")
    ));
}

// ===== Run() integration tests — added by Plan 35-03 =====

/// 3-objective DTLZ2 sphere benchmark (M=3, k=2, n=4 variables for fast tests).
///
/// f_1 = cos(x_1 * π/2) * cos(x_2 * π/2) * (1 + g)
/// f_2 = cos(x_1 * π/2) * sin(x_2 * π/2) * (1 + g)
/// f_3 = sin(x_1 * π/2) * (1 + g)
/// g(x) = sum_{i=2}^{n-1} (x_i - 0.5)^2  (zero-indexed: indices 2..n)
fn dtlz2_objectives() -> Vec<Box<genetic_algorithms::multi_objective::ObjectiveFn<RangeGenotype<f64>>>> {
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

fn build_test_nsga3(
    population_size: usize,
    max_generations: usize,
) -> Nsga3Ga<RangeChromosome<f64>> {
    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(population_size)
        .with_max_generations(max_generations)
        .with_reference_points_auto(4); // 15 reference points for M=3, p=4

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_rng_seed(42);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    Nsga3Ga::<RangeChromosome<f64>>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_objective_fns(dtlz2_objectives())
        .build()
        .expect("Nsga3 build should succeed with all required builders called")
}

#[test]
fn test_nsga3_run_produces_pareto_front() {
    let mut nsga3 = build_test_nsga3(40, 10);
    let result = nsga3.run();
    assert!(result.is_ok(), "Nsga3 run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "Pareto front should contain at least one rank-0 individual");
    // Every front individual should have rank == 0.
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All ParetoFront members must be rank 0");
        assert_eq!(ind.objectives.len(), 3);
    }
}

#[test]
fn test_nsga3_run_with_custom_reference_points() {
    let custom = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![0.5, 0.5, 0.0],
        vec![0.5, 0.0, 0.5],
        vec![0.0, 0.5, 0.5],
        vec![0.33, 0.33, 0.34],
    ];
    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(30)
        .with_max_generations(8)
        .with_reference_points(custom);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_rng_seed(7);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_objective_fns(dtlz2_objectives())
        .build()
        .expect("build should succeed");

    let result = nsga3.run();
    assert!(result.is_ok(), "run with custom ref points: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty());
}

/// Counter-based observer to verify that lifecycle hooks fire.
#[derive(Default)]
struct CountingObserver {
    sort_count: AtomicUsize,
    pareto_count: AtomicUsize,
}

impl genetic_algorithms::Nsga3Observer<RangeChromosome<f64>> for CountingObserver {
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
fn test_nsga3_run_invokes_observer_hooks() {
    let observer = Arc::new(CountingObserver::default());
    let observer_handle = observer.clone();

    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(30)
        .with_max_generations(5)
        .with_reference_points_auto(2);

    let ga_config = GaConfiguration::default()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_rng_seed(123);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_objective_fns(dtlz2_objectives())
        .with_observer(observer as Arc<dyn genetic_algorithms::Nsga3Observer<RangeChromosome<f64>> + Send + Sync>)
        .build()
        .expect("build should succeed");

    nsga3.run().expect("run should succeed");

    // on_non_dominated_sort_complete fires only when observer is Some and Instant is available.
    // On non-WASM: fires exactly max_generations times (5).
    // The pareto hook fires unconditionally (not gated by Instant).
    assert_eq!(observer_handle.pareto_count.load(Ordering::Relaxed), 5);
    // sort_count fires from inside the Instant block — always 5 on non-WASM host test.
    assert_eq!(observer_handle.sort_count.load(Ordering::Relaxed), 5);
}
