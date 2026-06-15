//! Structured lifecycle observer for the GA execution loop.
//!
//! This module provides the [`GaObserver`] trait, which exposes 12 hooks that
//! fire at precise points in the GA run loop. Observers:
//!
//! - Are stored as `Arc<dyn GaObserver + Send + Sync>` (sharable across island threads)
//! - Receive `&self` (not `&mut self`), enabling interior mutability patterns
//! - Cover 13 lifecycle, operator-timing, and special-event hooks
//! - Require `Send + Sync` for safe use in rayon parallel regions
//!
//! # Hooks
//!
//! | Hook | When it fires |
//! |------|--------------|
//! | `on_run_start` | Once before the first generation |
//! | `on_generation_start` | Start of each generation, before any operators |
//! | `on_selection_complete` | After parent selection |
//! | `on_crossover_complete` | After crossover produces offspring |
//! | `on_mutation_complete` | After mutation is applied |
//! | `on_fitness_evaluation_complete` | After fitness evaluation of new population |
//! | `on_survivor_selection_complete` | After survivor selection prunes population |
//! | `on_new_best` | When the population's best fitness improves |
//! | `on_stagnation` | Each time the stagnation counter increments |
//! | `on_extension_triggered` | When an extension strategy fires |
//! | `on_generation_end` | End of each generation, after statistics collected |
//! | `on_run_end` | Once after the GA loop exits |
//! | `on_restart` | When the CMA-ES engine triggers an automatic restart |

use crate::cma::restart::RestartEvent;
use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use std::time::Duration;

/// Payload for the [`GaObserver::on_extension_triggered`] hook.
///
/// Stack-allocated and `Copy`-able — zero heap allocation.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::observer::ExtensionEvent;
///
/// let event = ExtensionEvent {
///     generation: 42,
///     diversity: 0.15,
///     extension_type: "MassExtinction",
///     threshold: 0.2,
/// };
/// assert_eq!(event.generation, 42);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ExtensionEvent {
    /// The generation at which the extension fired.
    pub generation: usize,
    /// Population diversity at the time of extension.
    pub diversity: f64,
    /// Name of the extension strategy (e.g. `"MassExtinction"`).
    pub extension_type: &'static str,
    /// Diversity threshold that triggered the extension.
    pub threshold: f64,
}

/// Structured lifecycle observer for [`Ga<U>`](crate::ga::Ga).
///
/// All methods have default no-op implementations — implement only the hooks
/// you need. The `Send + Sync` supertraits are required for safe sharing
/// across rayon threads (island model) via `Arc`.
///
/// # GaObserver vs the removed Reporter trait
///
/// | Aspect | Reporter (removed v3.0) | GaObserver |
/// |--------|------------------------|------------|
/// | Storage | `Box<dyn Reporter + Send>` | `Arc<dyn GaObserver + Send + Sync>` |
/// | Mutability | `&mut self` | `&self` |
/// | Hooks | 4 lifecycle | 13 (lifecycle + operator + special) |
/// | Thread safety | `Send` only | `Send + Sync` |
///
/// See [`MIGRATION.md`](https://docs.rs/genetic_algorithms) for migration recipes.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::GaObserver;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::stats::GenerationStats;
/// use genetic_algorithms::ga::TerminationCause;
///
/// struct MyObserver;
///
/// impl GaObserver<Binary> for MyObserver {
///     fn on_run_start(&self) { println!("GA started"); }
///     fn on_run_end(&self, cause: TerminationCause, _stats: &[GenerationStats]) {
///         println!("GA ended: {:?}", cause);
///     }
/// }
/// ```
pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    /// Called once before the first generation.
    fn on_run_start(&self) {}
    /// Called at the start of each generation, before any operators run.
    fn on_generation_start(&self, _generation: usize) {}
    /// Called after parent selection completes.
    fn on_selection_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
    }
    /// Called after crossover produces offspring.
    fn on_crossover_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _offspring_count: usize,
    ) {
    }
    /// Called after mutation is applied to offspring.
    fn on_mutation_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
    }
    /// Called after fitness evaluation of the new population.
    fn on_fitness_evaluation_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
    }
    /// Called after survivor selection prunes the population.
    fn on_survivor_selection_complete(
        &self,
        _generation: usize,
        _duration: Duration,
        _population_size: usize,
    ) {
    }
    /// Called when the population's best fitness improves.
    fn on_new_best(&self, _generation: usize, _best: &U) {}
    /// Called each time the stagnation counter increments.
    fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {}
    /// Called when an extension strategy fires due to low diversity.
    fn on_extension_triggered(&self, _event: ExtensionEvent) {}
    /// Called when the CMA-ES engine triggers an automatic restart.
    ///
    /// Fires once per restart event, after state has been reset and before the next
    /// restart's generation loop begins. Only relevant when a [`RestartStrategy`](crate::cma::RestartStrategy)
    /// is configured on [`CmaConfiguration`](crate::cma::CmaConfiguration).
    fn on_restart(&self, _event: &RestartEvent) {}
    /// Called at the end of each generation, after statistics are collected.
    fn on_generation_end(&self, _stats: &GenerationStats) {}
    /// Called once after the GA loop exits.
    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {}
}

/// Zero-sized no-op observer. All hooks use their default empty bodies.
///
/// Useful as a compile-check type or as a placeholder.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::observer::NoopObserver;
///
/// let _obs = NoopObserver;
/// ```
pub struct NoopObserver;

impl<U: ChromosomeT> GaObserver<U> for NoopObserver {}

/// Observer for [`IslandGa<U>`](crate::island::IslandGa) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon island threads via `Arc`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::IslandGaObserver;
/// use genetic_algorithms::chromosomes::Binary;
///
/// struct MyIslandObserver;
///
/// impl IslandGaObserver<Binary> for MyIslandObserver {
///     fn on_island_run_start(&self, island_id: usize) {
///         println!("Island {} started", island_id);
///     }
/// }
/// ```
pub trait IslandGaObserver<U: ChromosomeT>: Send + Sync {
    /// Called when an island run starts.
    fn on_island_run_start(&self, _island_id: usize) {}
    /// Called when an island run ends.
    fn on_island_run_end(&self, _island_id: usize) {}
    /// Called at the end of each generation for each island.
    fn on_island_generation_end(
        &self,
        _island_id: usize,
        _generation: usize,
        _stats: &GenerationStats,
    ) {
    }
    /// Called when migration is triggered between islands.
    fn on_migration_triggered(&self, _generation: usize, _migration_count: usize) {}
}

/// Observer for [`Nsga2Ga<U>`](crate::nsga2::Nsga2Ga) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::Nsga2Observer;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MyNsga2Observer;
///
/// impl Nsga2Observer<Range<f64>> for MyNsga2Observer {
///     fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, _pop: usize) {
///         println!("Gen {}: {} Pareto fronts", generation, front_count);
///     }
/// }
/// ```
pub trait Nsga2Observer<U: ChromosomeT>: Send + Sync {
    /// Called after Pareto fronts are assigned.
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
    }
    /// Called after non-dominated sorting completes.
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
    /// Called after crowding distance calculation completes.
    fn on_crowding_distance_calculated(&self, _generation: usize, _duration_ms: f64) {}
}

/// Observer for [`Nsga3Ga<U>`](crate::nsga3::Nsga3Ga) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `Nsga3Observer<U>` in Phase 35 — adding it
/// would be a breaking change for existing `AllObserver` implementors. Use
/// [`Nsga3Ga::with_observer`](crate::nsga3::Nsga3Ga::with_observer) to attach
/// an `Nsga3Observer` independently.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::Nsga3Observer;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MyNsga3Observer;
/// impl Nsga3Observer<Range<f64>> for MyNsga3Observer {}
/// ```
pub trait Nsga3Observer<U: ChromosomeT>: Send + Sync {
    /// Called after Pareto fronts are assigned for the current generation.
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
    }
    /// Called after non-dominated sorting completes for the current generation.
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}

/// Observer for [`MoeaDGa<U>`](crate::moead::MoeaDGa) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `MoeaDObserver<U>` in Phase 36 — adding it
/// would be a breaking change for existing `AllObserver` implementors. Use
/// [`MoeaDGa::with_observer`](crate::moead::MoeaDGa::with_observer) to attach
/// a `MoeaDObserver` independently.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::MoeaDObserver;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MyMoeaDObserver;
/// impl MoeaDObserver<Range<f64>> for MyMoeaDObserver {}
/// ```
pub trait MoeaDObserver<U: ChromosomeT>: Send + Sync {
    /// Called after Pareto fronts are assigned for the current generation.
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
    }
    /// Called after non-dominated sorting completes for the current generation.
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}

/// Observer for [`Spea2Ga<U>`](crate::spea2::Spea2Ga) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `Spea2Observer<U>` in Phase 37 — adding it
/// would be a breaking change for existing `AllObserver` implementors (D-07). Use
/// [`Spea2Ga::with_observer`](crate::spea2::Spea2Ga::with_observer) to attach
/// a `Spea2Observer` independently.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::Spea2Observer;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MySpea2Observer;
/// impl Spea2Observer<Range<f64>> for MySpea2Observer {}
/// ```
pub trait Spea2Observer<U: ChromosomeT>: Send + Sync {
    /// Called after strength + density fitness is assigned to all individuals
    /// in the combined population + archive set.
    fn on_fitness_assigned(
        &self,
        _generation: usize,
        _duration_ms: f64,
        _pop_size: usize,
        _archive_size: usize,
    ) {
    }
    /// Called after environmental selection completes and the archive is updated
    /// (including truncation if necessary).
    fn on_archive_updated(
        &self,
        _generation: usize,
        _archive_size: usize,
        _non_dominated_count: usize,
    ) {
    }
}

/// Observer for [`SmsEmoaGa<U>`](crate::sms_emoa::SmsEmoaGa) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `SmsEmoaObserver<U>` in Phase 38 — adding it
/// would be a breaking change for existing `AllObserver` implementors. Use
/// [`SmsEmoaGa::with_observer`](crate::sms_emoa::SmsEmoaGa::with_observer) to attach
/// a `SmsEmoaObserver` independently.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::SmsEmoaObserver;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MySmsEmoaObserver;
/// impl SmsEmoaObserver<Range<f64>> for MySmsEmoaObserver {}
/// ```
pub trait SmsEmoaObserver<U: ChromosomeT>: Send + Sync {
    /// Called after hypervolume contribution calculation completes for the current generation.
    fn on_hypervolume_contribution_assigned(
        &self,
        _generation: usize,
        _duration_ms: f64,
        _population_size: usize,
    ) {
    }
    /// Called after steady-state (mu+1) removal in the current generation.
    fn on_steady_state_removal(&self, _generation: usize, _population_size: usize) {}
}

/// Observer for [`IbeaGa<U>`](crate::ibea::IbeaGa) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `IbeaObserver<U>` in Phase 38 — adding it
/// would be a breaking change for existing `AllObserver` implementors. Use
/// [`IbeaGa::with_observer`](crate::ibea::IbeaGa::with_observer) to attach
/// an `IbeaObserver` independently.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::IbeaObserver;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MyIbeaObserver;
/// impl IbeaObserver<Range<f64>> for MyIbeaObserver {}
/// ```
pub trait IbeaObserver<U: ChromosomeT>: Send + Sync {
    /// Called after indicator-based fitness assignment completes for the current generation.
    fn on_indicator_fitness_assigned(
        &self,
        _generation: usize,
        _duration_ms: f64,
        _population_size: usize,
    ) {
    }
    /// Called after environmental selection removes an individual.
    fn on_environmental_selection(
        &self,
        _generation: usize,
        _population_size: usize,
        _removed_index: usize,
    ) {
    }
}

/// Observer for [`CmaEngine<U>`](crate::cma::CmaEngine) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `CmaObserver<U>` — adding it would be a
/// breaking change for existing `AllObserver` implementors. Attach a
/// `CmaObserver` independently to a `CmaEngine` if your build wires it through.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::CmaObserver;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MyCmaObserver;
/// impl CmaObserver<Range<f64>> for MyCmaObserver {}
/// ```
pub trait CmaObserver<U: ChromosomeT>: Send + Sync {
    /// Called after the global step size `sigma` is updated for the current generation.
    fn on_sigma_updated(&self, _generation: usize, _sigma: f64) {}
    /// Called after the covariance matrix is updated for the current generation.
    fn on_covariance_updated(&self, _generation: usize, _condition_number: f64) {}
    /// Called when eigendecomposition completes for the current generation,
    /// carrying the wall-clock duration in milliseconds.
    fn on_eigendecomposition_complete(&self, _generation: usize, _duration_ms: f64) {}
}

/// Observer for [`PsoEngine<U>`](crate::pso::PsoEngine) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `PsoObserver<U>` — adding it would be a
/// breaking change for existing `AllObserver` implementors. Attach a
/// `PsoObserver` independently to a `PsoEngine` if your build wires it through.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::PsoObserver;
/// use genetic_algorithms::chromosomes::Range;
///
/// struct MyPsoObserver;
/// impl PsoObserver<Range<f64>> for MyPsoObserver {}
/// ```
pub trait PsoObserver<U: ChromosomeT>: Send + Sync {
    /// Called after velocities are updated for the current generation.
    fn on_velocity_updated(&self, _generation: usize, _population_size: usize) {}
    /// Called after a particle's personal best is improved.
    fn on_personal_best_updated(&self, _generation: usize, _particle_index: usize, _fitness: f64) {}
    /// Called after the swarm's global (or neighborhood) best is updated.
    fn on_global_best_updated(&self, _generation: usize, _fitness: f64) {}
}

/// Observer for [`EdaEngine<U>`](crate::eda::EdaEngine) / [`EdaRealEngine<U>`](crate::eda::EdaRealEngine) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `EdaObserver<U>` — adding it would be a
/// breaking change for existing `AllObserver` implementors. Attach an
/// `EdaObserver` independently if your build wires it through.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::EdaObserver;
/// use genetic_algorithms::chromosomes::Binary;
///
/// struct MyEdaObserver;
/// impl EdaObserver<Binary> for MyEdaObserver {}
/// ```
pub trait EdaObserver<U: ChromosomeT>: Send + Sync {
    /// Called after the probabilistic model is re-estimated from the selected
    /// parents for the current generation.
    fn on_model_updated(&self, _generation: usize, _selected_count: usize) {}
    /// Called after offspring are sampled from the current model.
    fn on_offspring_sampled(&self, _generation: usize, _offspring_count: usize) {}
}

/// Combined observer bound for use with [`CompositeObserver`].
///
/// Any type that implements [`GaObserver<U>`], [`IslandGaObserver<U>`],
/// [`Nsga2Observer<U>`], and [`Send + Sync`] automatically satisfies this
/// supertrait via the blanket impl below.
///
/// `AllObserver<U>` has zero methods of its own — it is a pure supertrait
/// marker and is object-safe: `dyn AllObserver<U>` is valid.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::observer::{GaObserver, IslandGaObserver, Nsga2Observer, AllObserver};
/// use genetic_algorithms::chromosomes::Binary;
///
/// struct MyAllObserver;
/// impl GaObserver<Binary> for MyAllObserver {}
/// impl IslandGaObserver<Binary> for MyAllObserver {}
/// impl Nsga2Observer<Binary> for MyAllObserver {}
/// // MyAllObserver now satisfies AllObserver<Binary> via blanket impl
/// ```
pub trait AllObserver<U: ChromosomeT>:
    GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync
{
}

impl<U, T> AllObserver<U> for T
where
    U: ChromosomeT,
    T: GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync,
{
}

mod log;
pub use log::LogObserver;

#[cfg(feature = "observer-tracing")]
mod tracing_observer;
#[cfg(feature = "observer-tracing")]
pub use tracing_observer::TracingObserver;

#[cfg(feature = "observer-metrics")]
mod metrics_observer;
#[cfg(feature = "observer-metrics")]
pub use metrics_observer::MetricsObserver;

mod composite;
pub use composite::CompositeObserver;
