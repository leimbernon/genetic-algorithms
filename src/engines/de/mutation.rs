//! DE mutation strategies and adaptive parameter generation.

use super::configuration::DeMutationStrategy;
use super::gene::DeGene;
use crate::traits::ChromosomeT;
use rand::Rng;

// ─── Utility distributions ───────────────────────────────────────────────────

/// Sample from Cauchy(loc, scale) using the inverse-CDF method.
pub(crate) fn cauchy_sample(rng: &mut impl Rng, loc: f64, scale: f64) -> f64 {
    let u: f64 = rng.random::<f64>().clamp(1e-9, 1.0 - 1e-9);
    loc + scale * (std::f64::consts::PI * (u - 0.5)).tan()
}

/// Sample from Normal(mean, std) using the Box-Muller transform.
pub(crate) fn normal_sample(rng: &mut impl Rng, mean: f64, std: f64) -> f64 {
    let u1: f64 = rng.random::<f64>().max(1e-10);
    let u2: f64 = rng.random::<f64>();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + std * z
}

// ─── Random index selection ───────────────────────────────────────────────────

/// Pick `n` distinct indices from `[0, pop_size)`, all different from `exclude`.
pub(crate) fn pick_distinct(
    rng: &mut impl Rng,
    pop_size: usize,
    exclude: usize,
    n: usize,
) -> Vec<usize> {
    assert!(pop_size > n, "population too small for mutation");
    let mut chosen = Vec::with_capacity(n);
    while chosen.len() < n {
        let idx = rng.random_range(0..pop_size);
        if idx != exclude && !chosen.contains(&idx) {
            chosen.push(idx);
        }
    }
    chosen
}

// ─── Core mutation ────────────────────────────────────────────────────────────

/// Produce a mutant vector for individual `i` using the chosen strategy.
///
/// `best_idx` is the index of the globally best individual.
/// `archive` is an optional slice of archived (inferior) individuals used by
/// some adaptive variants.
#[allow(clippy::too_many_arguments)]
pub fn mutate<U>(
    strategy: &DeMutationStrategy,
    pop: &[U],
    i: usize,
    best_idx: usize,
    f: f64,
    rng: &mut impl Rng,
    archive: Option<&[U]>,
) -> Vec<U::Gene>
where
    U: ChromosomeT,
    U::Gene: DeGene,
{
    let dim = pop[i].dna().len();
    match strategy {
        DeMutationStrategy::Rand1 => {
            let rs = pick_distinct(rng, pop.len(), i, 3);
            mutant_from_base(pop[rs[0]].dna(), pop[rs[1]].dna(), pop[rs[2]].dna(), f, dim)
        }
        DeMutationStrategy::Best1 => {
            let rs = pick_distinct(rng, pop.len(), i, 2);
            mutant_from_base(
                pop[best_idx].dna(),
                pop[rs[0]].dna(),
                pop[rs[1]].dna(),
                f,
                dim,
            )
        }
        DeMutationStrategy::CurrentToBest1 => {
            let rs = pick_distinct(rng, pop.len(), i, 2);
            current_to_best(
                pop[i].dna(),
                pop[best_idx].dna(),
                pop[rs[0]].dna(),
                pop[rs[1]].dna(),
                f,
                dim,
                archive,
                rng,
            )
        }
        DeMutationStrategy::Rand2 => {
            let rs = pick_distinct(rng, pop.len(), i, 5);
            two_diff_base(
                pop[rs[0]].dna(),
                pop[rs[1]].dna(),
                pop[rs[2]].dna(),
                pop[rs[3]].dna(),
                pop[rs[4]].dna(),
                f,
                dim,
            )
        }
        DeMutationStrategy::Best2 => {
            let rs = pick_distinct(rng, pop.len(), i, 4);
            two_diff_base(
                pop[best_idx].dna(),
                pop[rs[0]].dna(),
                pop[rs[1]].dna(),
                pop[rs[2]].dna(),
                pop[rs[3]].dna(),
                f,
                dim,
            )
        }
    }
}

/// `base + F * (a - b)`
fn mutant_from_base<G: DeGene>(base: &[G], a: &[G], b: &[G], f: f64, dim: usize) -> Vec<G> {
    (0..dim)
        .map(|j| {
            base[j].with_de_value(base[j].de_value() + f * (a[j].de_value() - b[j].de_value()))
        })
        .collect()
}

/// `current + F*(best-current) + F*(r1-r2)`  (DE/current-to-best/1 or DE/current-to-pbest/1)
#[allow(clippy::too_many_arguments)]
fn current_to_best<G: DeGene>(
    current: &[G],
    best: &[G],
    r1: &[G],
    r2_pop: &[G],
    f: f64,
    dim: usize,
    archive: Option<&[impl ChromosomeT<Gene = G>]>,
    rng: &mut impl Rng,
) -> Vec<G> {
    // r2 is drawn from population ∪ archive when archive is present (JADE)
    let r2_dna: &[G] = if let Some(arc) = archive {
        if !arc.is_empty() && rng.random_bool(0.5) {
            let idx = rng.random_range(0..arc.len());
            arc[idx].dna()
        } else {
            r2_pop
        }
    } else {
        r2_pop
    };

    (0..dim)
        .map(|j| {
            let v = current[j].de_value()
                + f * (best[j].de_value() - current[j].de_value())
                + f * (r1[j].de_value() - r2_dna[j].de_value());
            current[j].with_de_value(v)
        })
        .collect()
}

/// `base + F*(a-b) + F*(c-d)`  (DE/rand/2 or DE/best/2)
fn two_diff_base<G: DeGene>(
    base: &[G],
    a: &[G],
    b: &[G],
    c: &[G],
    d: &[G],
    f: f64,
    dim: usize,
) -> Vec<G> {
    (0..dim)
        .map(|j| {
            let v = base[j].de_value()
                + f * (a[j].de_value() - b[j].de_value())
                + f * (c[j].de_value() - d[j].de_value());
            base[j].with_de_value(v)
        })
        .collect()
}

// ─── JADE adaptive state ──────────────────────────────────────────────────────

/// Mutable state for the JADE adaptive variant.
#[derive(Debug, Clone)]
pub struct JadeState {
    /// Current mean for F (Cauchy location).
    pub mu_f: f64,
    /// Current mean for CR (Normal mean).
    pub mu_cr: f64,
    /// Successful F values accumulated this generation.
    pub s_f: Vec<f64>,
    /// Successful CR values accumulated this generation.
    pub s_cr: Vec<f64>,
}

impl Default for JadeState {
    fn default() -> Self {
        Self::new()
    }
}

impl JadeState {
    pub fn new() -> Self {
        Self {
            mu_f: 0.5,
            mu_cr: 0.5,
            s_f: Vec::new(),
            s_cr: Vec::new(),
        }
    }

    /// Draw F from Cauchy(μ_F, 0.1), clipped to (0, 1].
    pub fn draw_f(&self, rng: &mut impl Rng) -> f64 {
        loop {
            let f = cauchy_sample(rng, self.mu_f, 0.1);
            if f > 0.0 {
                return f.min(1.0);
            }
        }
    }

    /// Draw CR from Normal(μ_CR, 0.1), clipped to [0, 1].
    pub fn draw_cr(&self, rng: &mut impl Rng) -> f64 {
        normal_sample(rng, self.mu_cr, 0.1).clamp(0.0, 1.0)
    }

    /// Record a successful (F, CR) pair.
    pub fn record_success(&mut self, f: f64, cr: f64) {
        self.s_f.push(f);
        self.s_cr.push(cr);
    }

    /// Update μ_F and μ_CR at end of generation; clear accumulators.
    pub fn update(&mut self, c: f64) {
        if !self.s_f.is_empty() {
            self.mu_f = (1.0 - c) * self.mu_f + c * lehmer_mean(&self.s_f);
        }
        if !self.s_cr.is_empty() {
            self.mu_cr = (1.0 - c) * self.mu_cr + c * arithmetic_mean(&self.s_cr);
        }
        self.s_f.clear();
        self.s_cr.clear();
    }
}

// ─── L-SHADE adaptive state ───────────────────────────────────────────────────

/// Mutable state for the L-SHADE adaptive variant.
#[derive(Debug, Clone)]
pub struct LShadeState {
    pub m_f: Vec<f64>,
    pub m_cr: Vec<f64>,
    /// Current write index (cycles through history).
    pub k: usize,
    pub s_f: Vec<f64>,
    pub s_cr: Vec<f64>,
}

impl LShadeState {
    pub fn new(history_size: usize) -> Self {
        let h = history_size.max(1);
        Self {
            m_f: vec![0.5; h],
            m_cr: vec![0.5; h],
            k: 0,
            s_f: Vec::new(),
            s_cr: Vec::new(),
        }
    }

    /// Draw F from Cauchy(M_F\[r\], 0.1), clipped to (0, 1].
    pub fn draw_f(&self, rng: &mut impl Rng) -> f64 {
        let r = rng.random_range(0..self.m_f.len());
        loop {
            let f = cauchy_sample(rng, self.m_f[r], 0.1);
            if f > 0.0 {
                return f.min(1.0);
            }
        }
    }

    /// Draw CR from Normal(M_CR\[r\], 0.1), clipped to \[0, 1\].
    pub fn draw_cr(&self, rng: &mut impl Rng) -> f64 {
        let r = rng.random_range(0..self.m_cr.len());
        normal_sample(rng, self.m_cr[r], 0.1).clamp(0.0, 1.0)
    }

    /// Record a successful (F, CR) pair.
    pub fn record_success(&mut self, f: f64, cr: f64) {
        self.s_f.push(f);
        self.s_cr.push(cr);
    }

    /// Update memory at index k; advance k; clear accumulators.
    pub fn update(&mut self) {
        if !self.s_f.is_empty() {
            let h = self.m_f.len();
            self.m_f[self.k] = lehmer_mean(&self.s_f);
            self.m_cr[self.k] = arithmetic_mean(&self.s_cr);
            self.k = (self.k + 1) % h;
        }
        self.s_f.clear();
        self.s_cr.clear();
    }
}

// ─── Statistical helpers ──────────────────────────────────────────────────────

/// Lehmer mean (power mean p=2): `Σx² / Σx`.
fn lehmer_mean(xs: &[f64]) -> f64 {
    let num: f64 = xs.iter().map(|x| x * x).sum();
    let den: f64 = xs.iter().sum();
    if den == 0.0 {
        0.5
    } else {
        num / den
    }
}

/// Arithmetic mean.
fn arithmetic_mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.5;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}
