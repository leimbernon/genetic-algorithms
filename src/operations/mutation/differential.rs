//! DE-style differential mutation operator for range-encoded chromosomes.
//!
//! Implements the classical DE/rand/1 mutation scheme:
//! `mutant[i] = x_r1[i] + F * (x_r2[i] - x_r3[i])`, where `x_r1`, `x_r2`,
//! and `x_r3` are three distinct randomly chosen population members (also
//! distinct from the target individual). The result is clamped to each gene's
//! declared range.
//!
//! This operator is exposed as a free function so that engine integrations
//! (e.g., the standard `Ga<U>` engine in Plan 03) can call it directly with
//! full population context. Calling it via `MutationOperator::mutate` or
//! `factory_with_params` returns `GaError::MutationError` as a safety net.

use crate::error::GaError;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::ChromosomeT;
use crate::chromosomes::Range as RangeChromosome;
use log::debug;
use rand::Rng;
use std::any::Any;
use std::borrow::Cow;

/// DE-style differential mutation: `mutant[i] = x_r1[i] + F * (x_r2[i] - x_r3[i])`,
/// clamped to each gene's range. Requires `Range<T>` chromosome and a population
/// of at least 4 individuals (target + 3 distinct donors).
///
/// # Arguments
///
/// * `individual` - The target chromosome to mutate (mutated in-place).
/// * `chromosomes` - The full population slice (including `individual` at `target_idx`).
/// * `target_idx` - Index of `individual` within `chromosomes`.
/// * `f` - The differential weight (scale factor). Typical range: 0.4–1.0.
///
/// # Errors
///
/// - `GaError::MutationError` if `chromosomes.len() < 4` (D-03)
/// - `GaError::MutationError` if `individual` is not a `Range<T>` chromosome (D-02)
pub fn differential_mutation<U>(
    individual: &mut U,
    chromosomes: &[U],
    target_idx: usize,
    f: f64,
) -> Result<(), GaError>
where
    U: ChromosomeT + 'static,
{
    if chromosomes.len() < 4 {
        return Err(GaError::MutationError(
            "Differential mutation requires at least 4 chromosomes in the population \
             (target + 3 distinct donors)"
                .to_string(),
        ));
    }

    debug!(target: "mutation_events", "Starting differential mutation f={}", f);

    macro_rules! try_type {
        ($t:ty) => {
            if (individual as &dyn Any).is::<RangeChromosome<$t>>() {
                let pop_len = chromosomes.len();
                let mut rng = crate::rng::make_rng();
                let mut r1 = rng.random_range(0..pop_len);
                while r1 == target_idx {
                    r1 = rng.random_range(0..pop_len);
                }
                let mut r2 = rng.random_range(0..pop_len);
                while r2 == target_idx || r2 == r1 {
                    r2 = rng.random_range(0..pop_len);
                }
                let mut r3 = rng.random_range(0..pop_len);
                while r3 == target_idx || r3 == r1 || r3 == r2 {
                    r3 = rng.random_range(0..pop_len);
                }

                // Downcast donors to concrete Range<$t>
                let x1 = (&chromosomes[r1] as &dyn Any)
                    .downcast_ref::<RangeChromosome<$t>>()
                    .ok_or_else(|| {
                        GaError::MutationError(
                            "Differential donor mismatched type".to_string(),
                        )
                    })?;
                let x2 = (&chromosomes[r2] as &dyn Any)
                    .downcast_ref::<RangeChromosome<$t>>()
                    .ok_or_else(|| {
                        GaError::MutationError(
                            "Differential donor mismatched type".to_string(),
                        )
                    })?;
                let x3 = (&chromosomes[r3] as &dyn Any)
                    .downcast_ref::<RangeChromosome<$t>>()
                    .ok_or_else(|| {
                        GaError::MutationError(
                            "Differential donor mismatched type".to_string(),
                        )
                    })?;
                let target = (individual as &mut dyn Any)
                    .downcast_mut::<RangeChromosome<$t>>()
                    .unwrap();

                let n = target.dna().len();
                let mut new_dna = target.dna().to_vec();
                for i in 0..n {
                    if new_dna[i].ranges.is_empty() {
                        continue;
                    }
                    let a = <$t as GaussianConvertible>::to_f64(x1.dna()[i].value);
                    let b = <$t as GaussianConvertible>::to_f64(x2.dna()[i].value);
                    let c = <$t as GaussianConvertible>::to_f64(x3.dna()[i].value);
                    let mutant_f = a + f * (b - c);
                    // Clamp to gene's first declared range
                    let (lo, hi) = new_dna[i].ranges[0];
                    let lo_f = <$t as GaussianConvertible>::to_f64(lo);
                    let hi_f = <$t as GaussianConvertible>::to_f64(hi);
                    let clamped = mutant_f.clamp(lo_f, hi_f);
                    new_dna[i].value = <$t as GaussianConvertible>::from_f64(clamped);
                }
                target.set_dna(Cow::Owned(new_dna));
                debug!(target: "mutation_events", "Finished differential mutation");
                return Ok(());
            }
        };
    }

    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);

    Err(GaError::MutationError(
        "Differential mutation requires a Range<T> chromosome \
         (T = f64, f32, i32, or i64)"
            .to_string(),
    ))
}
