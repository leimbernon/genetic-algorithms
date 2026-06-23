//! Extracted from src/engines/ga.rs in phase 69-04 — init_population, initialize_random, initialize_with_seeds.

use super::*;

impl<U> Ga<U>
where
    U: LinearChromosome
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize
        + MaybeDeserialize
        + OperatorCompat,
    U::Gene: 'static + Debug,
{
    /// Randomly initializes the population using the provided initialization function.
    ///
    /// Behavior:
    /// - Validates configuration and alleles before starting.
    /// - Creates and evaluates chromosomes in parallel using rayon.
    /// - Sets the internal `population` with the collected chromosomes.
    pub fn initialization(&mut self) -> Result<&mut Self, GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        // Before starting initialization, verify that initializer is set
        if self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        // Validate configuration
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        // Delegate to seed-aware or standard init
        if self.seeds.is_some() {
            self.initialize_with_seeds()?;
        } else {
            self.initialize_random()?;
        }

        // Apply repair operator to initial population if configured
        if let Some(ref repair_op) = self.repair_operator {
            for c in self.population.chromosomes.iter_mut() {
                repair_op(c)?;
                c.calculate_fitness();
            }
        }

        Ok(self)
    }

    /// Creates a random initial population (no seeds).
    fn initialize_random(&mut self) -> Result<(), GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        let population_size = self.configuration.limit_configuration.population_size;
        let chromosome_length = self.configuration.limit_configuration.chromosome_length;
        let init_fn = self.initialization_fn.as_ref().unwrap();
        // In batch mode `fitness_fn` is None; pass None to initialize_chromosomes_par
        // so chromosomes start with default 0.0 fitness — batch_evaluate runs afterward
        // to assign correct values.
        let fitness_fn = self.fitness_fn.as_ref();

        let chromosomes = match chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(length) => {
                crate::traits::initialize_chromosomes_par::<U>(
                    population_size,
                    length,
                    if self.alleles.is_empty() {
                        None
                    } else {
                        Some(&self.alleles)
                    },
                    init_fn,
                    fitness_fn,
                    0,
                )
            }
            crate::chromosomes::ChromosomeLength::Variable { min, max } => {
                // For variable-length chromosomes, each individual gets a random
                // length sampled uniformly from [min, max].
                // Decision: pass sampled length as genes_per_chromosome to init_fn
                // (per Phase 52 discussion log — zero changes to init_fn signature).
                let alleles_ref: Option<&[U::Gene]> = if self.alleles.is_empty() {
                    None
                } else {
                    Some(self.alleles.as_slice())
                };
                // In batch mode fitness_fn is None; ff_opt carries the optional reference
                let ff = fitness_fn.cloned();

                #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
                let result: Vec<U> = (0..population_size)
                    .into_par_iter()
                    .map(|_| {
                        let len = {
                            let mut rng = crate::rng::make_rng();
                            rng.random_range(min..=max)
                        };
                        let genes = init_fn(len, alleles_ref);
                        let mut c = U::new();
                        c.set_dna(std::borrow::Cow::Owned(genes));
                        if let Some(ref ff_arc) = ff {
                            let ff_clone = std::sync::Arc::clone(ff_arc);
                            c.set_fitness_fn(move |dna| ff_clone(dna));
                        }
                        c.calculate_fitness();
                        c.set_age(0);
                        c
                    })
                    .collect();
                #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
                let result: Vec<U> = (0..population_size)
                    .map(|_| {
                        let len = {
                            let mut rng = crate::rng::make_rng();
                            rng.random_range(min..=max)
                        };
                        let genes = init_fn(len, alleles_ref);
                        let mut c = U::new();
                        c.set_dna(std::borrow::Cow::Owned(genes));
                        if let Some(ref ff_arc) = ff {
                            let ff_clone = std::sync::Arc::clone(ff_arc);
                            c.set_fitness_fn(move |dna| ff_clone(dna));
                        }
                        c.calculate_fitness();
                        c.set_age(0);
                        c
                    })
                    .collect();
                result
            }
        };

        // Set population directly (with_population is consuming, so we assign inline)
        let new_population = Population::new(chromosomes);
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples =
                new_population.size() / 2;
        }
        self.population = new_population;

        Ok(())
    }

    /// Initializes the population with pre-evaluated seeds placed at the front.
    ///
    /// Seeds are moved into the population in order, then remaining slots are
    /// filled with randomly generated chromosomes. Fill chromosomes are
    /// genotypically deduplicated against all existing seeds (and prior fills),
    /// using the same DNA comparison pattern as HallOfFame.
    ///
    /// When a HallOfFame is configured, seeds and fill chromosomes are both
    /// evaluated for archive entry during initialization (generation 0).
    ///
    /// WASM compatible: seed placement and dedup are pure data operations.
    fn initialize_with_seeds(&mut self) -> Result<(), GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        if self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        let seeds = self.seeds.take().unwrap();
        let population_size = self.configuration.limit_configuration.population_size;
        let fill_count = population_size - seeds.len();
        let length = match self.configuration.limit_configuration.chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(n) => n,
            crate::chromosomes::ChromosomeLength::Variable { .. } => {
                return Err(GaError::ConfigurationError(
                    "ChromosomeLength::Variable is not yet supported (Phase 52). Use ChromosomeLength::Fixed.".into(),
                ));
            }
        };
        let init_fn = self.initialization_fn.as_ref().unwrap();
        // In batch mode `fitness_fn` is None — chromosomes start with default fitness;
        // batch_evaluate runs afterward to assign correct values.
        let fitness_fn = self.fitness_fn.as_ref();

        // Step 1: Collect seed DNA for dedup comparison
        let seed_dnas: Vec<&[U::Gene]> = seeds.iter().map(|s| s.dna()).collect();

        // Step 2: Generate random fill with genotypic dedup against seeds
        let mut fill_chromosomes: Vec<U> = Vec::with_capacity(fill_count);
        // Use sequential generation for dedup (parallel would make retry logic complex)
        // Use a max retry bound to prevent infinite loop in degenerate cases
        let max_attempts = fill_count * 10;
        let mut attempts = 0;

        while fill_chromosomes.len() < fill_count && attempts < max_attempts {
            attempts += 1;

            // Generate one random chromosome using the initialization function
            let genes = init_fn(
                length,
                if self.alleles.is_empty() {
                    None
                } else {
                    Some(&self.alleles)
                },
            );
            let mut new_chromosome = U::new();
            new_chromosome.set_dna(std::borrow::Cow::Owned(genes));

            // Check genotypic uniqueness against seed DNAs.
            // id-based dedup is only meaningful when all allele IDs are distinct.
            // For Range<T> chromosomes all alleles share id=0, so id-based dedup
            // incorrectly treats every generated chromosome as a duplicate of every
            // seed, exhausting max_attempts and returning an InitializationError.
            // When allele IDs are non-unique, skip dedup entirely — the random
            // initializer produces statistically unique chromosomes in practice.
            let new_dna = new_chromosome.dna();

            let ids_are_unique = {
                let mut seen = std::collections::HashSet::new();
                self.alleles.iter().all(|g| seen.insert(g.id()))
            };

            if ids_are_unique {
                let is_duplicate = seed_dnas.iter().any(|seed_dna| {
                    let max_len = new_dna.len().max(seed_dna.len());
                    if max_len == 0 {
                        return true;
                    }
                    (0..max_len).all(|i| {
                        let id_a = new_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                        let id_b = seed_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                        id_a == id_b
                    })
                });

                if is_duplicate {
                    continue; // Discard and retry
                }

                // Also dedup against already-generated fill chromosomes
                let is_fill_duplicate = fill_chromosomes.iter().any(|existing| {
                    let existing_dna = existing.dna();
                    let max_len = new_dna.len().max(existing_dna.len());
                    if max_len == 0 {
                        return true;
                    }
                    (0..max_len).all(|i| {
                        let id_a = new_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                        let id_b = existing_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                        id_a == id_b
                    })
                });

                if is_fill_duplicate {
                    continue; // Discard and retry
                }
            }

            // Set fitness function and evaluate (skipped in batch mode — fitness_fn is None)
            if let Some(ff) = fitness_fn {
                let ff_clone = Arc::clone(ff);
                new_chromosome.set_fitness_fn(move |genes| ff_clone(genes));
            }
            new_chromosome.calculate_fitness();
            new_chromosome.set_age(0);

            fill_chromosomes.push(new_chromosome);
        }

        if fill_chromosomes.len() < fill_count {
            return Err(GaError::InitializationError(format!(
                "Failed to generate {} unique random chromosomes (max attempts {} reached). \
                 Try reducing the number of seeds or increasing population_size.",
                fill_count, max_attempts,
            )));
        }

        // Step 3: Build population: seeds placed first, then fill
        let mut all_chromosomes: Vec<U> = Vec::with_capacity(population_size);
        all_chromosomes.extend(seeds); // Seeds first (trusted fitness)
        all_chromosomes.extend(fill_chromosomes); // Fill (evaluated)

        let new_population = Population::new(all_chromosomes);
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples =
                new_population.size() / 2;
        }
        self.population = new_population;

        // Step 4: Admit seeds to Hall of Fame if configured (per D-08)
        if let Some(ref mut hof) = self.hall_of_fame {
            for c in self.population.chromosomes.iter() {
                hof.try_insert(c, 0); // Generation 0: initialization
            }
        }

        Ok(())
    }
}
