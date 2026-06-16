//! Extracted from src/engines/ga.rs in phase 69-04 — Extension trigger and diversity check.

use super::*;

/// Returns `true` when the extension strategy should be triggered for this generation.
///
/// Extension is triggered when:
/// - An `ExtensionConfiguration` is set, **and**
/// - The extension method is not `Extension::Noop`, **and**
/// - The current population diversity is below the configured threshold.
///
/// This check is performed after each generation in `run_with_callback`.  When it
/// returns `true`, the caller invokes `extension_ops::factory` and optionally regrows
/// the population to the initial size.
pub(crate) fn should_trigger_extension(
    ext_config: &crate::extension::configuration::ExtensionConfiguration,
    diversity: f64,
) -> bool {
    ext_config.method != Extension::Noop && diversity < ext_config.diversity_threshold
}
