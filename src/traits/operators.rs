use crate::configuration::{LimitConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::extension::configuration::ExtensionConfiguration;
use crate::operations::mutation::ValueMutable;
use crate::traits::ChromosomeT;

/// Trait for parent selection operators.
///
/// Implement this trait to define a custom selection strategy. Built-in
/// implementations are provided for the [`Selection`](crate::operations::Selection)
/// enum variants.
///
/// # Example
///
/// ```rust,ignore
/// use genetic_algorithms::traits::SelectionOperator;
///
/// struct MySelection;
///
/// impl SelectionOperator for MySelection {
///     fn select<U>(
///         &self,
///         chromosomes: &[U],
///         number_of_couples: usize,
///         number_of_threads: usize,
///     ) -> Vec<(usize, usize)>
///     where
///         U: ChromosomeT + Sync + Send + 'static + Clone,
///     {
///         // Custom selection logic here
///         vec![]
///     }
/// }
/// ```
pub trait SelectionOperator {
    /// Select parent pairs from the population.
    ///
    /// Returns a vector of `(index_a, index_b)` pairs representing
    /// the indices of selected parents in the `chromosomes` slice.
    fn select<U>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
    ) -> Vec<(usize, usize)>
    where
        U: ChromosomeT + Sync + Send + 'static + Clone;
}

/// Trait for crossover operators.
///
/// Implement this trait to define a custom crossover strategy. Built-in
/// implementations are provided for all [`Crossover`](crate::operations::Crossover)
/// enum variants. `Sbx` and `BlendAlpha` use runtime downcasting and work
/// automatically when `U` is [`Range<T>`](crate::chromosomes::Range) with
/// `T` being `f64`, `f32`, `i32`, or `i64`.
///
/// # Example
///
/// ```rust,ignore
/// use genetic_algorithms::traits::CrossoverOperator;
///
/// struct MyCrossover;
///
/// impl CrossoverOperator for MyCrossover {
///     fn crossover<U: ChromosomeT>(&self, parent_1: &U, parent_2: &U)
///         -> Result<Vec<U>, GaError>
///     {
///         // Custom crossover logic here
///         Ok(vec![parent_1.clone(), parent_2.clone()])
///     }
/// }
/// ```
pub trait CrossoverOperator {
    /// Perform crossover between two parents.
    ///
    /// Returns a vector of children (typically 2), or an error if the
    /// crossover cannot be performed.
    ///
    /// **Note:** Additional parameters (e.g., `number_of_points` for
    /// multi-point crossover) are expected to be stored in the operator
    /// struct or captured from configuration.
    fn crossover<U: ChromosomeT>(&self, parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>;
}

/// Trait for mutation operators.
///
/// Implement this trait to define a custom mutation strategy. Built-in
/// implementations are provided for the [`Mutation`](crate::operations::Mutation)
/// enum variants.
///
/// # Example
///
/// ```rust,ignore
/// use genetic_algorithms::traits::MutationOperator;
///
/// struct MyMutation;
///
/// impl MutationOperator for MyMutation {
///     fn mutate<U>(
///         &self,
///         individual: &mut U,
///         step: Option<f64>,
///         sigma: Option<f64>,
///     ) -> Result<(), GaError>
///     where
///         U: ChromosomeT + ValueMutable + 'static,
///     {
///         // Custom mutation logic here
///         Ok(())
///     }
/// }
/// ```
pub trait MutationOperator {
    /// Mutate an individual chromosome in-place.
    ///
    /// # Arguments
    ///
    /// * `individual` - The chromosome to mutate.
    /// * `step` - Optional step size (used by Creep mutation); **also used as the
    ///   `scale` (γ) parameter for `Mutation::Cauchy`** (default 1.0 when `None`).
    /// * `sigma` - Optional standard deviation (used by Gaussian mutation); **also
    ///   used as the stability index `α` for `Mutation::LevyFlight`** (default 1.5 when `None`).
    fn mutate<U>(
        &self,
        individual: &mut U,
        step: Option<f64>,
        sigma: Option<f64>,
    ) -> Result<(), GaError>
    where
        U: ChromosomeT + ValueMutable + 'static;
}

/// Trait for survivor selection operators.
///
/// Implement this trait to define a custom survivor selection strategy.
/// Built-in implementations are provided for the
/// [`Survivor`](crate::operations::Survivor) enum variants.
///
/// # Example
///
/// ```rust,ignore
/// use genetic_algorithms::traits::SurvivorOperator;
///
/// struct MySurvivor;
///
/// impl SurvivorOperator for MySurvivor {
///     fn select_survivors<U: ChromosomeT>(
///         &self,
///         chromosomes: &mut Vec<U>,
///         population_size: usize,
///         limit_configuration: LimitConfiguration,
///     ) -> Result<(), GaError> {
///         // Custom survivor selection logic here
///         Ok(())
///     }
/// }
/// ```
pub trait SurvivorOperator {
    /// Select survivors from the combined parent + offspring population.
    ///
    /// Trims `chromosomes` in-place to at most `population_size` individuals.
    fn select_survivors<U: ChromosomeT>(
        &self,
        chromosomes: &mut Vec<U>,
        population_size: usize,
        limit_configuration: LimitConfiguration,
    ) -> Result<(), GaError>;
}

/// Trait for extension operators (population diversity control).
///
/// Implement this trait to define a custom extension strategy. Built-in
/// implementations are provided for the
/// [`Extension`](crate::operations::Extension) enum variants.
///
/// # Example
///
/// ```rust,ignore
/// use genetic_algorithms::traits::ExtensionOperator;
///
/// struct MyExtension;
///
/// impl ExtensionOperator for MyExtension {
///     fn apply_extension<U: ChromosomeT>(
///         &self,
///         chromosomes: &mut Vec<U>,
///         population_size: usize,
///         problem_solving: ProblemSolving,
///         config: &ExtensionConfiguration,
///     ) -> Result<(), GaError> {
///         // Custom extension logic here
///         Ok(())
///     }
/// }
/// ```
pub trait ExtensionOperator {
    /// Apply the extension strategy to the population.
    ///
    /// This is called when population diversity drops below the configured
    /// threshold. Implementations may reduce the population (requiring regrowth)
    /// or modify chromosomes in-place.
    fn apply_extension<U: ChromosomeT>(
        &self,
        chromosomes: &mut Vec<U>,
        population_size: usize,
        problem_solving: ProblemSolving,
        config: &ExtensionConfiguration,
    ) -> Result<(), GaError>;
}
