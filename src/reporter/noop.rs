use crate::traits::ChromosomeT;
use super::Reporter;

/// A no-op reporter. All hook methods use the trait's default empty bodies.
///
/// This type exists primarily for documentation and testing. In practice,
/// `Ga<U>` stores `Option<Box<dyn Reporter<U> + Send>>` and defaults to
/// `None`, so no reporter code runs at all when the user does not configure one.
pub struct NoopReporter;

impl<U: ChromosomeT> Reporter<U> for NoopReporter {}
