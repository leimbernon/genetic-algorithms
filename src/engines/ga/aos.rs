//! Extracted from src/engines/ga.rs in phase 69-04 — Adaptive Operator Selection (AOS) integration.

use super::*;

/// Initialises the AOS crossover and mutation state machines from the configured portfolios.
///
/// Called once at the start of `run_with_callback`, before the generation loop.
/// When a crossover portfolio (`crossover_portfolio`) or mutation portfolio
/// (`mutation_portfolio`) is configured, creates a `Mutex<AosState>` for each
/// and assigns it to the corresponding field.  No-op when no portfolio is set
/// (default single-operator mode, zero overhead).
pub(crate) fn init_aos_state(
    configuration: &GaConfiguration,
    aos_crossover: &mut Option<Mutex<AosState>>,
    aos_mutation: &mut Option<Mutex<AosState>>,
) {
    if let Some(ref xover_pf) = configuration.crossover_portfolio {
        *aos_crossover = Some(Mutex::new(AosState::new(
            xover_pf.len(),
            configuration.aos_strategy.clone(),
            configuration.aos_reward_window,
        )));
    }
    if let Some(ref mut_pf) = configuration.mutation_portfolio {
        *aos_mutation = Some(Mutex::new(AosState::new(
            mut_pf.len(),
            configuration.aos_strategy.clone(),
            configuration.aos_reward_window,
        )));
    }
}
