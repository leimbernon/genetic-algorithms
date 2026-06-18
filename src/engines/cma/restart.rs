//! Types for CMA-ES restart strategies (IPOP and BIPOP).
//!
//! This module provides the public API surface for controlling automatic restarts
//! in the CMA-ES engine. Two strategies are supported:
//!
//! - **IPOP** (Increasing POPulation): doubles the population size on each restart
//!   when the search stagnates. Simple and effective on multi-modal landscapes.
//!
//! - **BIPOP** (BImodal POPulation): alternates between a large population restart
//!   (like IPOP) and a small population restart (to exploit basins quickly).
//!
//! The [`RestartStrategy`] enum is attached to [`CmaConfiguration`](super::configuration::CmaConfiguration)
//! via the `restart_strategy` field; the engine checks it after each generation and triggers
//! a restart when stagnation is detected.

/// Strategy controlling how the CMA-ES engine restarts on stagnation.
///
/// Attach to [`CmaConfiguration`](super::configuration::CmaConfiguration) via
/// [`with_restart_strategy`](super::configuration::CmaConfiguration::with_restart_strategy).
///
/// # Example
/// ```rust,no_run
/// // no_run: RestartStrategy example — illustrative API usage
/// use genetic_algorithms::cma::{CmaConfiguration, RestartStrategy};
///
/// let config = CmaConfiguration::default_for_dim(10)
///     .with_restart_strategy(RestartStrategy::Ipop {
///         population_scale: 2.0,
///         stagnation_threshold: 50,
///         max_restarts: 9,
///     });
/// ```
#[derive(Debug, Clone, Copy)]
pub enum RestartStrategy {
    /// IPOP restart: increases population size by `population_scale` on each restart.
    ///
    /// Implements the IPOP-CMA-ES algorithm (Auger & Hansen, 2005). The population
    /// doubles (or scales by `population_scale`) after each stagnation event, promoting
    /// exploration of multi-modal landscapes at the cost of per-generation evaluation
    /// budget.
    Ipop {
        /// Factor to multiply λ (population size) on each large restart.
        ///
        /// Typical value: `2.0` (standard IPOP). Must be > 1.0 for population growth.
        /// A value of 1.0 restarts with the same population size (no IPOP benefit).
        population_scale: f64,

        /// Generations without fitness improvement before triggering a restart.
        ///
        /// Set low (e.g. 10–50) for fast restarts on clearly stagnated runs, or higher
        /// (e.g. 100–500) to allow longer convergence phases before giving up.
        stagnation_threshold: usize,

        /// Maximum number of restarts before the engine exits.
        ///
        /// The engine stops after this many restarts even if the fitness target
        /// has not been reached. A value of 9 is common in IPOP literature.
        max_restarts: usize,
    },

    /// BIPOP restart: alternates between a large-population restart and a
    /// small-population restart.
    ///
    /// Implements the BIPOP-CMA-ES algorithm (Hansen, 2009). Odd-numbered restarts
    /// use a large population (scaled by `population_scale`); even-numbered restarts
    /// use `small_population_size` to quickly exploit promising regions.
    Bipop {
        /// Factor to multiply λ on the large restart (BipopLarge).
        ///
        /// Typical value: `2.0`. Applied on odd-numbered restarts to grow the population
        /// as in IPOP. Must be > 1.0 for meaningful large restarts.
        population_scale: f64,

        /// Fixed λ for the small restart (BipopSmall).
        ///
        /// Used on even-numbered restarts to keep computation cheap while exploring a
        /// different basin. Set to `0` to auto-compute as `max(1, default_lambda / 5)`.
        small_population_size: usize,

        /// Generations without fitness improvement before triggering a restart.
        ///
        /// Applies to both large and small restart phases equally. See
        /// [`Ipop::stagnation_threshold`](RestartStrategy::Ipop::stagnation_threshold)
        /// for guidance on choosing this value.
        stagnation_threshold: usize,

        /// Maximum number of restarts (large + small combined) before the engine exits.
        ///
        /// The engine stops after this many total restarts regardless of whether the
        /// fitness target has been reached.
        max_restarts: usize,
    },
}

/// The kind of restart event that was triggered.
///
/// Carried in [`RestartEvent`] to let observers distinguish between IPOP, BIPOP-large,
/// and BIPOP-small restart phases without inspecting engine internals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestartKind {
    /// An IPOP restart — population size was multiplied by `population_scale`.
    Ipop,

    /// A BIPOP large restart — population size was multiplied by `population_scale`.
    ///
    /// Occurs on odd-numbered BIPOP restarts (1st, 3rd, 5th, …).
    BipopLarge,

    /// A BIPOP small restart — population size was set to `small_population_size`.
    ///
    /// Occurs on even-numbered BIPOP restarts (2nd, 4th, 6th, …).
    BipopSmall,
}

/// Payload delivered to [`GaObserver::on_restart`](crate::observer::GaObserver::on_restart)
/// when the CMA-ES engine triggers an automatic restart.
///
/// Stack-allocated and `Copy`-able — zero heap allocation, matching the
/// [`ExtensionEvent`](crate::observer::ExtensionEvent) design.
#[derive(Debug, Clone, Copy)]
pub struct RestartEvent {
    /// 1-based restart index.
    ///
    /// The first restart fires with `restart_number = 1`, the second with `2`, etc.
    /// Always in the range `[1, max_restarts]`.
    pub restart_number: usize,

    /// The generation at which this restart was triggered.
    ///
    /// Restarts are triggered after `stagnation_threshold` consecutive generations
    /// without improvement. This field records the generation that crossed the threshold.
    pub generation: usize,

    /// Population size immediately before the restart.
    ///
    /// Equal to the λ that was active during the stagnated run.
    pub population_size_before: usize,

    /// Population size immediately after the restart.
    ///
    /// For IPOP and BipopLarge: `population_size_after ≈ population_size_before * population_scale`.
    /// For BipopSmall: `population_size_after == small_population_size`.
    pub population_size_after: usize,

    /// Which kind of restart was performed.
    ///
    /// Allows observers to distinguish IPOP, BIPOP-large, and BIPOP-small events
    /// without tracking restart-number parity themselves.
    pub kind: RestartKind,
}
