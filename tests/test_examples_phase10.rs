// Phase 10 — Single-Population Examples: Fitness Function Unit Tests
//
// Gaps covered:
//   EX-01 (rastrigin.rs)        — Rastrigin fitness function math
//   EX-05 (feature_selection.rs) — Feature selection scoring logic
//   EX-06 (niching.rs)          — Gaussian-peak fitness function

use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::genotypes::Range as RangeGenotype;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn range_gene(value: f64) -> RangeGenotype<f64> {
    RangeGenotype::new(0, vec![(-5.12, 5.12)], value)
}

fn range_gene_domain(value: f64, lo: f64, hi: f64) -> RangeGenotype<f64> {
    RangeGenotype::new(0, vec![(lo, hi)], value)
}

fn binary_gene(value: bool) -> Binary {
    Binary { id: 0, value }
}

// ---------------------------------------------------------------------------
// EX-01 — Rastrigin fitness function
//
// f(x) = A*n + sum_i( x_i^2 - A*cos(2*pi*x_i) )   where A = 10
//
// Known values (single dimension, n=1):
//   x=0   → 10*1 + (0 - 10*cos(0))   = 10 + (0 - 10) = 0.0    ← global minimum
//   x=5.12→ 10*1 + (5.12^2 - 10*cos(2*pi*5.12))                ← large positive
// ---------------------------------------------------------------------------

fn rastrigin(dna: &[RangeGenotype<f64>]) -> f64 {
    let a = 10.0;
    let n = dna.len() as f64;
    a * n
        + dna
            .iter()
            .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
            .sum::<f64>()
}

#[test]
fn rastrigin_global_minimum_is_zero_at_origin() {
    let dna = vec![range_gene(0.0)];
    let fitness = rastrigin(&dna);
    assert!(
        fitness.abs() < 1e-10,
        "Expected Rastrigin(0) ≈ 0.0, got {fitness}"
    );
}

#[test]
fn rastrigin_origin_in_five_dimensions_is_zero() {
    let dna: Vec<RangeGenotype<f64>> = (0..5).map(|_| range_gene(0.0)).collect();
    let fitness = rastrigin(&dna);
    assert!(
        fitness.abs() < 1e-10,
        "Expected Rastrigin([0;5]) ≈ 0.0, got {fitness}"
    );
}

#[test]
fn rastrigin_non_zero_input_is_positive() {
    // x=5.12 is at the boundary — not the global minimum, fitness must be > 0
    let dna = vec![range_gene(5.12)];
    let fitness = rastrigin(&dna);
    assert!(
        fitness > 0.0,
        "Expected Rastrigin(5.12) > 0.0, got {fitness}"
    );
}

#[test]
fn rastrigin_is_nonnegative_for_known_local_minimum() {
    // x=1.0 is at a local minimum of the single-variable Rastrigin.
    // f(1) = 10 + (1 - 10*cos(2*pi)) = 10 + 1 - 10 = 1.0
    let dna = vec![range_gene(1.0)];
    let fitness = rastrigin(&dna);
    let expected = 1.0_f64;
    assert!(
        (fitness - expected).abs() < 1e-10,
        "Expected Rastrigin(1.0) = 1.0, got {fitness}"
    );
}

// ---------------------------------------------------------------------------
// EX-05 — Feature selection fitness function
//
// fitness = relevant_selected - 0.5 * irrelevant_selected
// RELEVANT = &[0, 1, 2, 3]  (indices into the dna slice)
// ---------------------------------------------------------------------------

const RELEVANT: &[usize] = &[0, 1, 2, 3];

fn feature_selection_fitness(dna: &[Binary]) -> f64 {
    let relevant_selected = RELEVANT.iter().filter(|&&i| dna[i].value).count() as f64;
    let irrelevant_selected = dna
        .iter()
        .enumerate()
        .filter(|(i, g)| !RELEVANT.contains(i) && g.value)
        .count() as f64;
    relevant_selected - 0.5 * irrelevant_selected
}

fn make_dna_20(selected_indices: &[usize]) -> Vec<Binary> {
    (0..20)
        .map(|i| binary_gene(selected_indices.contains(&i)))
        .collect()
}

#[test]
fn feature_selection_all_relevant_selected_no_irrelevant() {
    // Select only the 4 relevant features → fitness = 4.0
    let dna = make_dna_20(&[0, 1, 2, 3]);
    let fitness = feature_selection_fitness(&dna);
    assert!((fitness - 4.0).abs() < 1e-10, "Expected 4.0, got {fitness}");
}

#[test]
fn feature_selection_no_features_selected() {
    // Nothing selected → fitness = 0.0
    let dna = make_dna_20(&[]);
    let fitness = feature_selection_fitness(&dna);
    assert!(fitness.abs() < 1e-10, "Expected 0.0, got {fitness}");
}

#[test]
fn feature_selection_irrelevant_features_penalised() {
    // Select 2 irrelevant features (indices 4 and 5) → fitness = 0 - 0.5*2 = -1.0
    let dna = make_dna_20(&[4, 5]);
    let fitness = feature_selection_fitness(&dna);
    assert!(
        (fitness - (-1.0)).abs() < 1e-10,
        "Expected -1.0, got {fitness}"
    );
}

#[test]
fn feature_selection_mixed_relevant_and_irrelevant() {
    // Select all 4 relevant + 2 irrelevant → fitness = 4 - 0.5*2 = 3.0
    let dna = make_dna_20(&[0, 1, 2, 3, 4, 5]);
    let fitness = feature_selection_fitness(&dna);
    assert!((fitness - 3.0).abs() < 1e-10, "Expected 3.0, got {fitness}");
}

// ---------------------------------------------------------------------------
// EX-06 — Gaussian-peak fitness function (niching example)
//
// f(x) = peak(x, 2.0, 1.0) + peak(x, 5.0, 0.9) + peak(x, 8.0, 0.8)
// peak(center, height) = height * exp(-(x-center)^2 / (2 * 0.5^2))
//
// At each center the function equals that peak's height.
// Between peaks the value approaches 0.
// ---------------------------------------------------------------------------

fn gaussian_fitness(dna: &[RangeGenotype<f64>]) -> f64 {
    let x = dna[0].value;
    let peak = |center: f64, height: f64| -> f64 {
        height * (-((x - center).powi(2)) / (2.0 * 0.5_f64.powi(2))).exp()
    };
    peak(2.0, 1.0) + peak(5.0, 0.9) + peak(8.0, 0.8)
}

fn niching_gene(value: f64) -> RangeGenotype<f64> {
    range_gene_domain(value, 0.0, 10.0)
}

#[test]
fn gaussian_peak_at_x2_equals_height_1() {
    let dna = vec![niching_gene(2.0)];
    let fitness = gaussian_fitness(&dna);
    // At center the Gaussian equals its height (exp(0) = 1)
    // Contribution from other peaks is negligible (exp(-18) ≈ 0)
    assert!(
        (fitness - 1.0).abs() < 1e-6,
        "Expected ≈1.0 at x=2, got {fitness}"
    );
}

#[test]
fn gaussian_peak_at_x5_equals_height_09() {
    let dna = vec![niching_gene(5.0)];
    let fitness = gaussian_fitness(&dna);
    assert!(
        (fitness - 0.9).abs() < 1e-6,
        "Expected ≈0.9 at x=5, got {fitness}"
    );
}

#[test]
fn gaussian_peak_at_x8_equals_height_08() {
    let dna = vec![niching_gene(8.0)];
    let fitness = gaussian_fitness(&dna);
    assert!(
        (fitness - 0.8).abs() < 1e-6,
        "Expected ≈0.8 at x=8, got {fitness}"
    );
}

#[test]
fn gaussian_between_peaks_is_near_zero() {
    // x=10 is far from all three centers (distances 8, 5, 2)
    // With sigma=0.5, exp(-64) ≈ 0, exp(-25) ≈ 0, exp(-4) ≈ 0.018*0.8 ≈ 0.015
    let dna = vec![niching_gene(10.0)];
    let fitness = gaussian_fitness(&dna);
    assert!(
        fitness < 0.05,
        "Expected near-zero fitness at x=10, got {fitness}"
    );
}

#[test]
fn gaussian_fitness_is_always_nonnegative() {
    // All Gaussian terms are non-negative, so sum must be >= 0
    for x in [0.0, 1.0, 3.5, 6.5, 9.9] {
        let dna = vec![niching_gene(x)];
        let fitness = gaussian_fitness(&dna);
        assert!(
            fitness >= 0.0,
            "Expected non-negative fitness at x={x}, got {fitness}"
        );
    }
}
