use crate::configuration::{LimitConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::extension::configuration::ExtensionConfiguration;
use crate::operations::mutation::ValueMutable;
use crate::operations::Mutation;
use crate::traits::{ChromosomeT, LinearChromosome};

/// Trait for parent selection operators.
///
/// Implement this trait to define a custom selection strategy. Built-in
/// implementations are provided for the [`Selection`](crate::operations::Selection)
/// enum variants.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::SelectionOperator;
/// use genetic_algorithms::traits::ChromosomeT;
///
/// struct MySelection;
///
/// impl SelectionOperator for MySelection {
///     fn select<U>(
///         &self,
///         chromosomes: &[U],
///         number_of_couples: usize,
///         number_of_threads: usize,
///         num_parents: usize,
///     ) -> Vec<Vec<usize>>
///     where
///         U: ChromosomeT + Sync + Send + 'static + Clone,
///     {
///         // Custom selection logic: return random parent pairs
///         vec![]
///     }
/// }
/// ```
pub trait SelectionOperator {
    /// Select N-ary parent groups from the population.
    ///
    /// Returns a vector of groups, each containing `num_parents` population
    /// indices representing the selected parents for one crossover operation.
    /// For standard 2-parent crossover pass `num_parents = 2`; for multi-parent
    /// operators (UNDX, SPX, PCX) pass the operator's `num_parents` value.
    fn select<U>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
        num_parents: usize,
    ) -> Vec<Vec<usize>>
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
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::CrossoverOperator;
/// use genetic_algorithms::traits::LinearChromosome;
/// use genetic_algorithms::error::GaError;
///
/// struct MyCrossover;
///
/// impl CrossoverOperator for MyCrossover {
///     fn crossover<U: LinearChromosome>(&self, parent_1: &U, parent_2: &U)
///         -> Result<Vec<U>, GaError>
///     {
///         // Clone-crossover: no recombination, both parents survive unchanged
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
    fn crossover<U: LinearChromosome>(&self, parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>;
}

/// Trait for mutation operators.
///
/// Implement this trait to define a custom mutation strategy. Built-in
/// implementations are provided for the [`Mutation`]
/// enum variants.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::MutationOperator;
/// use genetic_algorithms::traits::LinearChromosome;
/// use genetic_algorithms::operations::mutation::ValueMutable;
/// use genetic_algorithms::operations::Mutation;
/// use genetic_algorithms::error::GaError;
///
/// struct MyMutation;
///
/// impl MutationOperator for MyMutation {
///     fn mutate<U>(
///         &self,
///         individual: &mut U,
///         mutation: &Mutation,
///     ) -> Result<(), GaError>
///     where
///         U: LinearChromosome + ValueMutable + 'static,
///     {
///         // No-op mutation: individual is returned unchanged
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
    /// * `mutation` - The `Mutation` variant to apply, carrying its own inline parameters.
    ///   Each variant extracts its own parameters (e.g. `Gaussian { sigma }` uses
    ///   `sigma.unwrap_or(0.1)`). Context-dependent variants (`Differential`,
    ///   `NonUniform`, `Insertion`, `Deletion`) are handled by the GA engine before
    ///   this trait is called and return `GaError::MutationError` when invoked directly.
    fn mutate<U>(
        &self,
        individual: &mut U,
        mutation: &Mutation,
    ) -> Result<(), GaError>
    where
        U: LinearChromosome + ValueMutable + 'static;
}

/// Trait for survivor selection operators.
///
/// Implement this trait to define a custom survivor selection strategy.
/// Built-in implementations are provided for the
/// [`Survivor`](crate::operations::Survivor) enum variants.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::SurvivorOperator;
/// use genetic_algorithms::traits::LinearChromosome;
/// use genetic_algorithms::configuration::LimitConfiguration;
/// use genetic_algorithms::error::GaError;
///
/// struct MySurvivor;
///
/// impl SurvivorOperator for MySurvivor {
///     fn select_survivors<U: LinearChromosome>(
///         &self,
///         chromosomes: &mut Vec<U>,
///         population_size: usize,
///         limit_configuration: LimitConfiguration,
///     ) -> Result<(), GaError> {
///         // Keep only the first `population_size` individuals (example only)
///         chromosomes.truncate(population_size);
///         Ok(())
///     }
/// }
/// ```
pub trait SurvivorOperator {
    /// Select survivors from the combined parent + offspring population.
    ///
    /// Trims `chromosomes` in-place to at most `population_size` individuals.
    fn select_survivors<U: LinearChromosome>(
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
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::ExtensionOperator;
/// use genetic_algorithms::traits::LinearChromosome;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::extension::configuration::ExtensionConfiguration;
/// use genetic_algorithms::error::GaError;
///
/// struct MyExtension;
///
/// impl ExtensionOperator for MyExtension {
///     fn apply_extension<U: LinearChromosome>(
///         &self,
///         chromosomes: &mut Vec<U>,
///         population_size: usize,
///         problem_solving: ProblemSolving,
///         config: &ExtensionConfiguration,
///     ) -> Result<(), GaError> {
///         // No-op: diversity rescue is skipped in this custom strategy
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
    fn apply_extension<U: LinearChromosome>(
        &self,
        chromosomes: &mut Vec<U>,
        population_size: usize,
        problem_solving: ProblemSolving,
        config: &ExtensionConfiguration,
    ) -> Result<(), GaError>;
}

/// Trait for local search refinement operators used in memetic algorithms.
///
/// Implement this trait to define a custom local search strategy. Built-in
/// implementations are provided for the [`LocalSearch`](crate::operations::LocalSearch)
/// enum variants.
///
/// The fitness function is received as a parameter at each call site (D-02),
/// enabling it to be Arc::cloned across parallel refinement tasks (D-03).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::LocalSearchOperator;
/// use genetic_algorithms::traits::LinearChromosome;
/// use genetic_algorithms::error::GaError;
///
/// struct MyLocalSearch;
///
/// impl LocalSearchOperator for MyLocalSearch {
///     fn improve<U>(
///         &self,
///         individual: &mut U,
///         fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
///     ) -> Result<usize, GaError>
///     where
///         U: LinearChromosome + Send + Sync + 'static + Clone,
///     {
///         // Return 0 improvements (no-op example)
///         Ok(0)
///     }
/// }
/// ```
pub trait LocalSearchOperator {
    /// Apply local search refinement to a single individual.
    ///
    /// # Arguments
    /// * `individual` - The chromosome to refine in-place.
    /// * `fitness_fn` - Fitness function for re-evaluation during refinement.
    ///
    /// # Returns
    /// Number of successful improvements made, or an error.
    fn improve<U>(
        &self,
        individual: &mut U,
        fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
    ) -> Result<usize, GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone;
}
