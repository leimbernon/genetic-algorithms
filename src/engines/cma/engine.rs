//! `CmaEngine` — Covariance Matrix Adaptation Evolution Strategy (CMA-ES) execution loop.
//!
//! Implements Hansen's reference algorithm (arXiv:1604.00772) for real-valued
//! black-box continuous optimization. The engine is generic over the chromosome
//! type `U`; `U::Gene` must implement [`RealGene`] so that covariance updates
//! and sampling can be performed on gene values.
//!
//! # WASM compatibility
//!
//! The core loop contains no `Instant::now()` calls and no parallel iteration.
//! The engine compiles safely for `wasm32-unknown-unknown`.

use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::configuration::ProblemSolving;
use crate::error::GaError;
use crate::fitness::BatchFitnessEvaluator;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, LinearChromosome, RealGene};

use super::configuration::CmaConfiguration;
use super::restart::{RestartEvent, RestartKind, RestartStrategy};

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Classical Jacobi eigendecomposition for a symmetric n×n matrix (row-major).
///
/// Returns `(b_mat, d_vec)` where:
/// - `b_mat[i*n + j]` is the i-th element of the j-th eigenvector (column-major
///   eigenvectors stored as columns of the matrix, row-major storage).
/// - `d_vec[i]` is `sqrt(eigenvalue_i)` (clamped to ≥ 1e-10 × max(d_vec) for
///   positive-definiteness).
///
/// Converges in ≤ 50 Jacobi sweeps for n ≤ 100 (sufficient for typical CMA-ES use).
fn jacobi_eigendecompose(c: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    // Work on a mutable copy of C
    let mut a: Vec<f64> = c.to_vec();
    // Initialize eigenvector matrix B as identity
    let mut b: Vec<f64> = (0..n * n)
        .map(|k| if k / n == k % n { 1.0 } else { 0.0 })
        .collect();

    for _ in 0..50 {
        // Find the largest off-diagonal element
        let mut max_off = 0.0_f64;
        let mut p = 0;
        let mut q = 1;
        for i in 0..n {
            for j in (i + 1)..n {
                let abs_aij = a[i * n + j].abs();
                if abs_aij > max_off {
                    max_off = abs_aij;
                    p = i;
                    q = j;
                }
            }
        }

        if max_off < 1e-12 {
            break;
        }

        // Compute Jacobi rotation angle
        let theta = if (a[p * n + p] - a[q * n + q]).abs() < 1e-300 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * ((2.0 * a[p * n + q]) / (a[p * n + p] - a[q * n + q])).atan()
        };
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // Apply Jacobi rotation: A' = G^T A G
        let mut a_new = a.clone();

        // Update rows p and q
        for k in 0..n {
            if k != p && k != q {
                let a_kp = a[k * n + p];
                let a_kq = a[k * n + q];
                a_new[k * n + p] = cos_t * a_kp + sin_t * a_kq;
                a_new[p * n + k] = a_new[k * n + p];
                a_new[k * n + q] = -sin_t * a_kp + cos_t * a_kq;
                a_new[q * n + k] = a_new[k * n + q];
            }
        }
        // Update pivot elements
        let app = a[p * n + p];
        let aqq = a[q * n + q];
        let apq = a[p * n + q];
        a_new[p * n + p] = cos_t * cos_t * app + 2.0 * sin_t * cos_t * apq + sin_t * sin_t * aqq;
        a_new[q * n + q] = sin_t * sin_t * app - 2.0 * sin_t * cos_t * apq + cos_t * cos_t * aqq;
        a_new[p * n + q] = 0.0;
        a_new[q * n + p] = 0.0;
        a = a_new;

        // Update eigenvector matrix B: B' = B G
        let mut b_new = b.clone();
        for k in 0..n {
            let b_kp = b[k * n + p];
            let b_kq = b[k * n + q];
            b_new[k * n + p] = cos_t * b_kp + sin_t * b_kq;
            b_new[k * n + q] = -sin_t * b_kp + cos_t * b_kq;
        }
        b = b_new;
    }

    // Extract eigenvalues (diagonal of A) and take sqrt
    let mut d: Vec<f64> = (0..n).map(|i| a[i * n + i]).collect();

    // Clamp to enforce positive semi-definiteness
    let max_d = d.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let floor = 1e-10 * max_d.max(1e-300); // avoid zero floor when max_d = 0
    for di in &mut d {
        if *di < floor {
            *di = floor;
        }
        *di = di.sqrt();
    }

    (b, d)
}

/// Compute `C^{-1/2} = B · diag(1/d) · B^T` (row-major, n×n).
fn compute_invsqrtc(b: &[f64], d: &[f64], n: usize) -> Vec<f64> {
    let mut result = vec![0.0_f64; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                // B[i,k] * (1/d[k]) * B[j,k]   (B^T gives B[j,k])
                sum += b[i * n + k] * b[j * n + k] / d[k];
            }
            result[i * n + j] = sum;
        }
    }
    result
}

/// Box-Muller transform — returns two independent N(0,1) samples.
fn standard_normal_pair<R: Rng>(rng: &mut R) -> (f64, f64) {
    let u1: f64 = rng.random::<f64>().max(f64::MIN_POSITIVE);
    let u2: f64 = rng.random::<f64>();
    let mag = (-2.0 * u1.ln()).sqrt();
    let angle = 2.0 * std::f64::consts::PI * u2;
    (mag * angle.cos(), mag * angle.sin())
}

/// Returns one standard-normal sample (discards the second Box-Muller sample).
fn standard_normal<R: Rng>(rng: &mut R) -> f64 {
    standard_normal_pair(rng).0
}

/// Matrix-vector product `y = A * x` for row-major A (n×n), x length n.
fn matvec(a: &[f64], x: &[f64], n: usize) -> Vec<f64> {
    let mut y = vec![0.0_f64; n];
    for i in 0..n {
        let mut s = 0.0;
        for j in 0..n {
            s += a[i * n + j] * x[j];
        }
        y[i] = s;
    }
    y
}

// ─── CmaState ────────────────────────────────────────────────────────────────

/// Internal bookkeeping for one CMA-ES run.
///
/// All fields follow Hansen's variable naming from arXiv:1604.00772.
#[allow(dead_code)]
struct CmaState {
    /// Problem dimension.
    n: usize,
    /// Population size λ.
    lambda: usize,
    /// Number of parents / selection size μ.
    mu: usize,
    /// Recombination weights (length μ, sum = 1).
    weights: Vec<f64>,
    /// Effective selection mass μ_eff = 1 / Σ(w_i²).
    mu_eff: f64,
    /// Step-size control cumulation rate c_s.
    cs: f64,
    /// Step-size damping factor d_s.
    ds: f64,
    /// Expected norm of N(0,I): χ_n ≈ √n (1 − 1/(4n) + 1/(21n²)).
    chi_n: f64,
    /// Covariance matrix cumulation rate c_c.
    cc: f64,
    /// Rank-one update learning rate c_1.
    c1: f64,
    /// Rank-μ update learning rate c_μ.
    cmu: f64,
    /// Eigendecomposition update interval (generations).
    t_eigen: usize,
    /// Current distribution mean (length n).
    mean: Vec<f64>,
    /// Current global step size σ.
    sigma: f64,
    /// Conjugate evolution path for C (length n).
    pc: Vec<f64>,
    /// Conjugate evolution path for σ (length n).
    ps: Vec<f64>,
    /// Covariance matrix C (n×n row-major).
    c_mat: Vec<f64>,
    /// Eigenvector matrix B of C (n×n row-major; columns = eigenvectors).
    b_mat: Vec<f64>,
    /// sqrt(eigenvalues) of C (length n).
    d_vec: Vec<f64>,
    /// C^{-1/2} = B · diag(1/d) · B^T (n×n row-major).
    invsqrtc: Vec<f64>,
    /// Generation counter at which eigendecomposition was last computed.
    eigeneval: usize,
}

impl CmaState {
    fn new(n: usize, lambda: usize, config: &CmaConfiguration, initial_mean: Vec<f64>) -> Self {
        let mu = lambda / 2;

        // --- Recombination weights (un-normalized log weights) -----------------
        let w_raw: Vec<f64> = (0..mu)
            .map(|i| ((lambda as f64 + 1.0) / 2.0).ln() - ((i + 1) as f64).ln())
            .collect();
        let w_sum: f64 = w_raw.iter().sum();
        let weights: Vec<f64> = w_raw.iter().map(|w| w / w_sum).collect();

        let mu_eff = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();

        // --- Strategy parameters (Hansen arXiv:1604.00772 defaults) -----------
        let nf = n as f64;
        let cs = config
            .cs
            .unwrap_or((mu_eff + 2.0) / (nf + mu_eff + 5.0));
        let ds = 1.0
            + 2.0 * (((mu_eff - 1.0) / (nf + 1.0)).max(0.0)).sqrt()
            + cs;
        let chi_n = nf.sqrt()
            * (1.0 - 1.0 / (4.0 * nf) + 1.0 / (21.0 * nf * nf));
        let cc = config
            .cc
            .unwrap_or((4.0 + mu_eff / nf) / (nf + 4.0 + 2.0 * mu_eff / nf));
        let c1 = config
            .c1
            .unwrap_or(2.0 / ((nf + 1.3).powi(2) + mu_eff));
        let cmu = config.cmu.unwrap_or(
            ((2.0 * (mu_eff - 2.0 + 1.0 / mu_eff)) / ((nf + 2.0).powi(2) + mu_eff))
                .min(1.0 - c1),
        );

        // Hansen arXiv:1604.00772
        let t_eigen = (((n as f64).powf(1.5) * 10.0 / lambda as f64).floor() as usize).max(1);

        // --- Matrix / vector initializations ---------------------------------
        let identity: Vec<f64> = (0..n * n)
            .map(|k| if k / n == k % n { 1.0 } else { 0.0 })
            .collect();
        let c_mat = identity.clone();
        let b_mat = identity.clone();
        let invsqrtc = identity;
        let d_vec = vec![1.0_f64; n];
        let pc = vec![0.0_f64; n];
        let ps = vec![0.0_f64; n];

        CmaState {
            n,
            lambda,
            mu,
            weights,
            mu_eff,
            cs,
            ds,
            chi_n,
            cc,
            c1,
            cmu,
            t_eigen,
            mean: initial_mean,
            sigma: config.sigma0,
            pc,
            ps,
            c_mat,
            b_mat,
            d_vec,
            invsqrtc,
            eigeneval: 0,
        }
    }
}

// ─── CmaResult ───────────────────────────────────────────────────────────────

/// Result returned by [`CmaEngine::run`].
pub struct CmaResult<U: LinearChromosome> {
    /// Final population (all individuals evaluated).
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
    /// Total number of restarts that fired during the run.
    ///
    /// Always `0` when no `restart_strategy` is configured on
    /// [`CmaConfiguration`].
    /// Each restart corresponds to one call to `on_restart` on the observer.
    pub total_restarts: usize,
}

// ─── CmaEngine ───────────────────────────────────────────────────────────────

/// CMA-ES engine.
///
/// Generic over the chromosome type `U`; `U::Gene` must implement [`RealGene`]
/// so that covariance updates and sampling can be performed on gene values.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::genotypes::Range as RangeGene;
///
/// let config = CmaConfiguration::default_for_dim(5)
///     .with_max_generations(500);
///
/// let mut engine = CmaEngine::new(
///     config,
///     |n| { /* return Vec<RangeChromosome<f64>> of length n */ todo!() },
///     |dna| dna.iter().map(|g| g.real_value().powi(2)).sum(),
/// );
/// let result = engine.run();
/// ```
pub struct CmaEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: CmaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
    batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>>,
    fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
}

impl<U: LinearChromosome + Clone> CmaEngine<U>
where
    U::Gene: RealGene,
{
    /// Construct a new engine.
    ///
    /// * `config` — algorithm parameters.
    /// * `init_fn` — called once with `population_size`; must return that many
    ///   initialised chromosomes.
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    pub fn new(
        config: CmaConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,
            batch_evaluator: None,
            fitness_cache: None,
        }
    }

    /// Attach a lifecycle observer (see [`GaObserver`] for available hooks).
    pub fn with_observer(mut self, obs: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Configure a batch fitness evaluator (D-03).
    ///
    /// When set, `run()` skips the scalar `fitness_fn` for all chromosomes and
    /// calls `evaluate_batch` instead. If a fitness cache is also configured,
    /// only cache misses are forwarded to `evaluate_batch` (D-06 partition).
    pub fn with_batch_evaluator(
        mut self,
        evaluator: Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>,
    ) -> Self {
        self.batch_evaluator = Some(evaluator);
        self
    }

    /// Evaluate all chromosomes in `pop` via the batch path (D-04).
    ///
    /// No-op when no batch evaluator is configured (Case A). Uses the D-06
    /// hit/miss partition when both evaluator and cache are configured (Case C).
    fn batch_evaluate_pop(&self, pop: &mut [U]) -> Result<(), GaError>
    where
        U::Gene: Debug,
    {
        let evaluator = match self.batch_evaluator.as_ref() {
            Some(e) => Arc::clone(e),
            None => return Ok(()),
        };

        if pop.is_empty() {
            return Ok(());
        }

        match self.fitness_cache.as_ref() {
            None => {
                // Case B: batch-evaluate everything
                let values = evaluator.evaluate_batch(pop);
                debug_assert_eq!(
                    values.len(),
                    pop.len(),
                    "evaluate_batch returned {} values for {} chromosomes (T-60-01)",
                    values.len(),
                    pop.len()
                );
                for (i, chromosome) in pop.iter_mut().enumerate() {
                    chromosome.set_fitness(values[i]);
                }
            }
            Some(cache_handle) => {
                // Case C: D-06 partition — only misses go to evaluate_batch
                let mut fitness_values: Vec<f64> = vec![0.0; pop.len()];
                let mut miss_indices: Vec<usize> = Vec::new();

                {
                    let mut cache = cache_handle.lock().expect("fitness cache lock poisoned");
                    for (i, chromosome) in pop.iter().enumerate() {
                        let key = crate::fitness::cache::hash_dna(chromosome.dna());
                        match cache.get(key) {
                            Some(f) => fitness_values[i] = f,
                            None => miss_indices.push(i),
                        }
                    }
                } // Lock released (Pitfall 2 — never hold lock across evaluate_batch)

                if !miss_indices.is_empty() {
                    let miss_chromosomes: Vec<U> =
                        miss_indices.iter().map(|&i| pop[i].clone()).collect();
                    let miss_values = evaluator.evaluate_batch(&miss_chromosomes);
                    debug_assert_eq!(
                        miss_values.len(),
                        miss_indices.len(),
                        "evaluate_batch returned {} values for {} miss chromosomes (T-60-01)",
                        miss_values.len(),
                        miss_indices.len()
                    );

                    let mut cache = cache_handle.lock().expect("fitness cache lock poisoned");
                    for (pos, &orig_i) in miss_indices.iter().enumerate() {
                        let f = miss_values[pos];
                        fitness_values[orig_i] = f;
                        let key = crate::fitness::cache::hash_dna(pop[orig_i].dna());
                        cache.put(key, f);
                    }
                }

                for (i, chromosome) in pop.iter_mut().enumerate() {
                    chromosome.set_fitness(fitness_values[i]);
                }
            }
        }

        Ok(())
    }

    /// Dispatches an observer hook if an observer is attached. No-op otherwise.
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Returns `true` if `candidate` is better than `current` under the
    /// configured optimization direction.
    #[inline]
    fn is_better(&self, candidate: f64, current: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => candidate < current,
            ProblemSolving::Maximization => candidate > current,
            ProblemSolving::FixedFitness => {
                if let Some(t) = self.config.fitness_target {
                    (candidate - t).abs() < (current - t).abs()
                } else {
                    candidate < current
                }
            }
        }
    }

    /// Returns `true` if the `fitness` value satisfies the stopping `target`.
    fn reached_target(&self, fitness: f64, target: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => fitness <= target,
            ProblemSolving::Maximization => fitness >= target,
            ProblemSolving::FixedFitness => (fitness - target).abs() < 1e-6,
        }
    }

    /// Returns `(index, fitness)` of the best individual in `pop`.
    fn find_best(&self, pop: &[U]) -> (usize, f64) {
        let mut best_idx = 0;
        let mut best_fit = pop[0].fitness();
        for (i, ind) in pop.iter().enumerate().skip(1) {
            if self.is_better(ind.fitness(), best_fit) {
                best_fit = ind.fitness();
                best_idx = i;
            }
        }
        (best_idx, best_fit)
    }

    /// Compute the population size (lambda) for the next restart.
    ///
    /// `restart_count` is the 0-based count of restarts fired **before** this one.
    /// The result is clamped to at least 2 (so that mu = lambda/2 >= 1).
    fn compute_next_lambda(
        strategy: &RestartStrategy,
        current_lambda: usize,
        default_lambda: usize,
        restart_count: usize,
    ) -> usize {
        let raw = match strategy {
            RestartStrategy::Ipop { population_scale, .. } => {
                ((current_lambda as f64) * population_scale).floor() as usize
            }
            RestartStrategy::Bipop {
                population_scale,
                small_population_size,
                ..
            } => {
                let next_restart_number = restart_count + 1;
                if next_restart_number % 2 == 1 {
                    // Odd restart number → large restart (IPOP-style)
                    ((current_lambda as f64) * population_scale).floor() as usize
                } else if *small_population_size == 0 {
                    // Even restart number, auto-compute small size
                    (default_lambda / 5).max(1)
                } else {
                    *small_population_size
                }
            }
        };
        // Clamp: mu = lambda/2 must be >= 1, so lambda >= 2
        raw.max(2)
    }

    /// Derive the [`RestartKind`] for the next restart.
    ///
    /// `restart_count` is the 0-based count of restarts fired **before** this one.
    fn restart_kind(strategy: &RestartStrategy, restart_count: usize) -> RestartKind {
        let next_restart_number = restart_count + 1;
        match strategy {
            RestartStrategy::Ipop { .. } => RestartKind::Ipop,
            RestartStrategy::Bipop { .. } => {
                if next_restart_number % 2 == 1 {
                    RestartKind::BipopLarge
                } else {
                    RestartKind::BipopSmall
                }
            }
        }
    }

    /// Run the CMA-ES algorithm and return the result.
    ///
    /// If a `restart_strategy` is configured, this method wraps the core CMA-ES
    /// generation loop in an outer restart loop. Each restart re-initialises the
    /// population and `CmaState` with an updated `current_lambda` (per IPOP or BIPOP
    /// rules). The `max_generations` budget applies **per restart**, not in total;
    /// `result.generations` is the sum of all generations completed across all restarts.
    pub fn run(&mut self) -> CmaResult<U>
    where
        U::Gene: Debug,
    {
        let mut rng = make_rng();
        let is_maximization =
            matches!(self.config.problem_solving, ProblemSolving::Maximization);

        // D-05/D-06: bootstrap cache handle at run() start.
        if let Some(size) = self.config.fitness_cache_size {
            if self.fitness_cache.is_none() {
                if self.batch_evaluator.is_none() {
                    // Scalar path: wrap fitness_fn with LRU cache (mirrors Ga::build())
                    let (wrapped_fn, cache_handle) =
                        crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
                    self.fitness_fn = wrapped_fn;
                    self.fitness_cache = Some(cache_handle);
                } else {
                    // Batch path: create bare cache for D-06 partition
                    self.fitness_cache = Some(Arc::new(Mutex::new(
                        crate::fitness::cache::FitnessCache::new(size),
                    )));
                }
            }
        }

        self.notify(|obs| obs.on_run_start());

        // ── Determine problem dimension via a peek init ───────────────────────
        // Call init_fn with at least 1 to peek at problem dimension.
        let peek_size = self.config.population_size.max(1);
        let peek_pop: Vec<U> = (self.init_fn)(peek_size);

        // Guard: empty population from user's init_fn
        if peek_pop.is_empty() {
            log::warn!(
                target: "cma_events",
                "CmaEngine: init_fn returned an empty population; returning empty result"
            );
            self.notify(|obs| {
                obs.on_run_end(TerminationCause::GenerationLimitReached, &[])
            });
            panic!("CmaEngine: init_fn returned an empty population");
        }

        let n = peek_pop[0].dna().len();
        assert!(
            n > 0,
            "CmaEngine: chromosomes must have non-zero DNA length"
        );

        // Compute default lambda (needed for BIPOP small-restart formula).
        // Captured once and never mutated.
        let default_lambda = if self.config.population_size == 0 {
            4 + (3.0 * (n as f64).ln()).floor() as usize
        } else {
            self.config.population_size
        };

        // ── Outer restart loop variables ──────────────────────────────────────
        let mut total_restarts: usize = 0;
        let mut current_lambda = default_lambda;

        // Global best tracking — initialised to the worst possible value.
        let mut global_best_fitness = if is_maximization {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        let mut global_best: Option<U> = None;

        let mut termination_cause = TerminationCause::GenerationLimitReached;
        let mut all_stats: Vec<GenerationStats> =
            Vec::with_capacity(self.config.max_generations);

        // ── Outer restart loop ────────────────────────────────────────────────
        // `pop` is initialised at the top of each outer iteration. We declare it
        // here so it remains accessible after `'restart_loop` exits.
        // Initialise with the first population so the compiler can prove it is
        // always initialised before use (avoids unused-assignment lint).
        let mut pop: Vec<U> = (self.init_fn)(current_lambda);
        if pop.is_empty() {
            panic!("CmaEngine: init_fn returned an empty population (first init)");
        }

        'restart_loop: loop {
            // ── Initialise population for this (re)start ─────────────────────
            // On the first iteration this re-uses the already-initialised `pop`.
            // On subsequent iterations, a fresh population is sampled.
            if total_restarts > 0 {
                pop = (self.init_fn)(current_lambda);
                if pop.is_empty() {
                    panic!("CmaEngine: init_fn returned an empty population (restart)");
                }
            }

            // Clone template chromosome for offspring construction
            let template = pop[0].clone();

            // Evaluate fitness (D-04: batch path replaces scalar loop)
            if self.batch_evaluator.is_some() {
                self.batch_evaluate_pop(&mut pop)
                    .expect("batch_evaluate_pop failed on initial population");
            } else {
                for ind in &mut pop {
                    let f = (self.fitness_fn)(ind.dna());
                    ind.set_fitness(f);
                }
            }

            // Compute mean from this restart's initial population
            let mut restart_mean = vec![0.0_f64; n];
            for chr in &pop {
                for (j, g) in chr.dna().iter().enumerate() {
                    restart_mean[j] += g.real_value();
                }
            }
            let pop_len = pop.len() as f64;
            for v in &mut restart_mean {
                *v /= pop_len;
            }

            // Initialise CMA state (full reset: sigma0, identity C, zero paths)
            let mut state = CmaState::new(n, current_lambda, &self.config, restart_mean);

            // Initial eigendecomposition
            let (b_init, d_init) = jacobi_eigendecompose(&state.c_mat, n);
            state.b_mat = b_init;
            state.d_vec = d_init;
            state.invsqrtc = compute_invsqrtc(&state.b_mat, &state.d_vec, n);

            // ── Per-restart best tracking ─────────────────────────────────────
            // Stagnation compares against restart_best_fitness (NOT global_best_fitness)
            // so that a restart which makes local progress is not counted as stagnant.
            let (restart_init_idx, restart_init_fitness) = self.find_best(&pop);
            let mut restart_best_fitness = restart_init_fitness;
            let mut stagnation_count: usize = 0;

            // Update global best from this restart's initial population
            if self.is_better(restart_init_fitness, global_best_fitness) {
                global_best_fitness = restart_init_fitness;
                global_best = Some(pop[restart_init_idx].clone());
                let gb_clone = global_best.as_ref().unwrap().clone();
                self.notify(|obs| obs.on_new_best(0, gb_clone));
            }

            // ── Inner generation loop ─────────────────────────────────────────
            for gen in 0..self.config.max_generations {
                // D-07: snapshot cache counters before this generation to compute deltas.
                let (prev_cache_hits, prev_cache_misses) = match &self.fitness_cache {
                    Some(ch) => {
                        let c = ch.lock().expect("fitness cache lock poisoned");
                        (c.hits(), c.misses())
                    }
                    None => (0, 0),
                };

                self.notify(|obs| obs.on_generation_start(gen));

                // ── Sample λ offspring ────────────────────────────────────────
                let mut offspring: Vec<U> = Vec::with_capacity(current_lambda);
                for _ in 0..current_lambda {
                    // Sample z ~ N(0, I)
                    let z_k: Vec<f64> = (0..n).map(|_| standard_normal(&mut rng)).collect();
                    // y_k = B * (D * z_k)
                    let dz: Vec<f64> = (0..n).map(|i| state.d_vec[i] * z_k[i]).collect();
                    let y_k = matvec(&state.b_mat, &dz, n);
                    // x_k = mean + sigma * y_k
                    let x_k: Vec<f64> = (0..n)
                        .map(|j| state.mean[j] + state.sigma * y_k[j])
                        .collect();

                    // Build offspring chromosome from x_k values
                    let new_dna: Vec<U::Gene> = template
                        .dna()
                        .iter()
                        .enumerate()
                        .map(|(j, g)| g.with_real_value(x_k[j]))
                        .collect();
                    let mut child = template.clone();
                    child.set_dna(Cow::Owned(new_dna));
                    // D-04: skip inline eval in batch mode; batch_evaluate_pop runs below.
                    if self.batch_evaluator.is_none() {
                        let f = (self.fitness_fn)(child.dna());
                        child.set_fitness(f);
                    }
                    offspring.push(child);
                }

                // D-04: batch-evaluate offspring after the build loop (collect-then-batch).
                if self.batch_evaluator.is_some() {
                    self.batch_evaluate_pop(&mut offspring)
                        .expect("batch_evaluate_pop failed on offspring");
                }

                pop = offspring;

                // ── Sort by fitness and select μ best ─────────────────────────
                let mut indices: Vec<usize> = (0..pop.len()).collect();
                if is_maximization {
                    indices.sort_unstable_by(|&a, &b| {
                        pop[b]
                            .fitness()
                            .partial_cmp(&pop[a].fitness())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    indices.sort_unstable_by(|&a, &b| {
                        pop[a]
                            .fitness()
                            .partial_cmp(&pop[b].fitness())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                let mu = state.mu;
                let selected_indices = &indices[..mu];

                // ── Update mean ───────────────────────────────────────────────
                let old_mean = state.mean.clone();
                let mut new_mean = vec![0.0_f64; n];
                for (k, &idx) in selected_indices.iter().enumerate() {
                    for (j, nm) in new_mean.iter_mut().enumerate() {
                        *nm += state.weights[k] * pop[idx].dna()[j].real_value();
                    }
                }

                // ── Validate mean (T-56-03-02: guard against NaN/Inf) ────────
                if !new_mean.iter().all(|v| v.is_finite()) {
                    log::warn!(
                        target: "cma_events",
                        "CmaEngine generation {}: new_mean contains NaN/Inf — stopping early",
                        gen
                    );
                    break;
                }

                // ── Compute step ──────────────────────────────────────────────
                let step: Vec<f64> = (0..n)
                    .map(|i| (new_mean[i] - old_mean[i]) / state.sigma)
                    .collect();

                // ── Update ps (cumulation for σ) ──────────────────────────────
                let invsqrtc_step = matvec(&state.invsqrtc, &step, n);
                let sqrt_cs_factor = (state.cs * (2.0 - state.cs) * state.mu_eff).sqrt();
                let ps_new: Vec<f64> = (0..n)
                    .map(|i| {
                        (1.0 - state.cs) * state.ps[i] + sqrt_cs_factor * invsqrtc_step[i]
                    })
                    .collect();
                state.ps = ps_new;

                let ps_norm = state
                    .ps
                    .iter()
                    .map(|x| x * x)
                    .sum::<f64>()
                    .sqrt();

                // ── h_sigma (stall indicator) ─────────────────────────────────
                let denom = (1.0 - (1.0 - state.cs).powi(2 * (gen + 1) as i32)).sqrt();
                let h_sigma = if ps_norm / denom / state.chi_n
                    < 1.4 + 2.0 / (n as f64 + 1.0)
                {
                    1.0_f64
                } else {
                    0.0_f64
                };

                // ── Update pc (evolution path for C) ─────────────────────────
                let sqrt_cc_factor =
                    (state.cc * (2.0 - state.cc) * state.mu_eff).sqrt();
                let pc_new: Vec<f64> = (0..n)
                    .map(|i| {
                        (1.0 - state.cc) * state.pc[i] + h_sigma * sqrt_cc_factor * step[i]
                    })
                    .collect();
                state.pc = pc_new;

                // ── Update covariance matrix C ────────────────────────────────
                // Decay term: (1 - c1 - cmu) * C
                let mut c_new: Vec<f64> = state
                    .c_mat
                    .iter()
                    .map(|&v| (1.0 - state.c1 - state.cmu) * v)
                    .collect();

                // Rank-one update: c1 * (pc * pc^T + (1-h_sigma)*cc*(2-cc)*C)
                for i in 0..n {
                    for j in 0..n {
                        c_new[i * n + j] += state.c1
                            * (state.pc[i] * state.pc[j]
                                + (1.0 - h_sigma)
                                    * state.cc
                                    * (2.0 - state.cc)
                                    * state.c_mat[i * n + j]);
                    }
                }

                // Rank-μ update: cmu * Σ_k w_k * y_k * y_k^T
                for (k, &idx) in selected_indices.iter().enumerate() {
                    let y_k: Vec<f64> = (0..n)
                        .map(|j| {
                            (pop[idx].dna()[j].real_value() - old_mean[j]) / state.sigma
                        })
                        .collect();
                    for i in 0..n {
                        for j in 0..n {
                            c_new[i * n + j] += state.cmu * state.weights[k] * y_k[i] * y_k[j];
                        }
                    }
                }

                // Enforce symmetry
                for i in 0..n {
                    for j in (i + 1)..n {
                        let avg = (c_new[i * n + j] + c_new[j * n + i]) / 2.0;
                        c_new[i * n + j] = avg;
                        c_new[j * n + i] = avg;
                    }
                }
                state.c_mat = c_new;

                // ── Update sigma ──────────────────────────────────────────────
                state.sigma *=
                    ((state.cs / state.ds) * (ps_norm / state.chi_n - 1.0)).exp();
                // Clamp to prevent NaN/Inf (T-56-03-04)
                state.sigma = state.sigma.clamp(1e-20, 1e20);

                // ── Update mean ───────────────────────────────────────────────
                state.mean = new_mean;

                // ── Eigendecomposition (scheduled) ────────────────────────────
                if gen >= state.eigeneval + state.t_eigen {
                    let (b_new, d_new) = jacobi_eigendecompose(&state.c_mat, n);
                    state.b_mat = b_new;
                    state.d_vec = d_new;
                    state.invsqrtc = compute_invsqrtc(&state.b_mat, &state.d_vec, n);
                    state.eigeneval = gen;
                }

                // ── Best tracking ─────────────────────────────────────────────
                let (bi, bf) = self.find_best(&pop);

                // Stagnation tracking (per-restart, not global)
                if self.is_better(bf, restart_best_fitness) {
                    restart_best_fitness = bf;
                    stagnation_count = 0;
                } else {
                    stagnation_count += 1;
                }

                // Update global best (gated — only fires on_new_best when global improves)
                if self.is_better(bf, global_best_fitness) {
                    global_best_fitness = bf;
                    global_best = Some(pop[bi].clone());
                    let gb_clone = global_best.as_ref().unwrap().clone();
                    self.notify(|obs| obs.on_new_best(gen, gb_clone));
                }

                // ── Statistics ────────────────────────────────────────────────
                let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
                let mut stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
                // D-07: populate per-generation cache delta stats when a cache is active.
                if let Some(ref ch) = self.fitness_cache {
                    let c = ch.lock().expect("fitness cache lock poisoned");
                    stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
                    stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
                }
                self.notify(|obs| obs.on_generation_end(&stats));
                all_stats.push(stats);

                // ── Restart trigger ───────────────────────────────────────────
                if let Some(ref strategy) = self.config.restart_strategy {
                    let (threshold, max_r) = match strategy {
                        RestartStrategy::Ipop { stagnation_threshold, max_restarts, .. } => {
                            (*stagnation_threshold, *max_restarts)
                        }
                        RestartStrategy::Bipop { stagnation_threshold, max_restarts, .. } => {
                            (*stagnation_threshold, *max_restarts)
                        }
                    };
                    if stagnation_count >= threshold {
                        // Pitfall 1: check limit BEFORE incrementing
                        if total_restarts >= max_r {
                            break 'restart_loop;
                        }
                        let pop_before = current_lambda;
                        current_lambda = Self::compute_next_lambda(
                            strategy,
                            current_lambda,
                            default_lambda,
                            total_restarts,
                        );
                        let kind = Self::restart_kind(strategy, total_restarts);
                        total_restarts += 1;
                        let event = RestartEvent {
                            restart_number: total_restarts,
                            generation: gen,
                            population_size_before: pop_before,
                            population_size_after: current_lambda,
                            kind,
                        };
                        self.notify(|obs| obs.on_restart(&event));
                        break; // break inner loop → outer loop re-inits
                    }
                }

                // ── Early stopping (fitness target) ───────────────────────────
                if let Some(target) = self.config.fitness_target {
                    if self.reached_target(global_best_fitness, target) {
                        termination_cause = TerminationCause::FitnessTargetReached;
                        break 'restart_loop;
                    }
                }
            }

            // Inner loop exhausted max_generations without triggering a restart trigger.
            // If there is no restart strategy, exit. Otherwise count this as a forced
            // restart and continue with an updated lambda if budget remains.
            if self.config.restart_strategy.is_none() {
                break 'restart_loop;
            }
            let max_r = match &self.config.restart_strategy {
                Some(RestartStrategy::Ipop { max_restarts, .. }) => *max_restarts,
                Some(RestartStrategy::Bipop { max_restarts, .. }) => *max_restarts,
                None => 0,
            };
            // Budget exhausted: exit
            if total_restarts >= max_r {
                break 'restart_loop;
            }
            // Treat exhausted max_generations as a forced restart: update lambda and count
            if let Some(ref strategy) = self.config.restart_strategy {
                let pop_before = current_lambda;
                current_lambda = Self::compute_next_lambda(
                    strategy,
                    current_lambda,
                    default_lambda,
                    total_restarts,
                );
                let kind = Self::restart_kind(strategy, total_restarts);
                total_restarts += 1;
                let event = RestartEvent {
                    restart_number: total_restarts,
                    generation: self.config.max_generations.saturating_sub(1),
                    population_size_before: pop_before,
                    population_size_after: current_lambda,
                    kind,
                };
                self.notify(|obs| obs.on_restart(&event));
            }
        }

        // Unwrap global best (always set after at least one iteration)
        let final_best = global_best.unwrap_or_else(|| {
            // Defensive fallback; should never be reached in practice since
            // we always init at least one pop above.
            panic!("CmaEngine: no best chromosome found (empty run)")
        });

        let generations = all_stats.len();
        let all_stats_ref = all_stats.as_slice();
        self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));

        CmaResult {
            population: pop,
            best: final_best,
            best_fitness: global_best_fitness,
            generations,
            total_restarts,
        }
    }
}
