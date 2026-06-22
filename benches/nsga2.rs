use genetic_algorithms::nsga2::crowding_distance::assign_crowding_distance;
use genetic_algorithms::nsga2::non_dominated_sort::non_dominated_sort;
use rand::Rng;

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

/// Generate `n` individuals, each with `m` random objective values in [0, 1).
#[cfg(not(tarpaulin_include))]
fn random_objectives(n: usize, m: usize) -> Vec<Vec<f64>> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| (0..m).map(|_| rng.random_range(0.0..1.0)).collect())
        .collect()
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

mod non_dominated_sort {
    use super::*;

    /// args = (pop_size, num_objectives)
    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 2usize), (10, 3), (10, 5),
        (50, 2), (50, 3), (50, 5),
        (100, 2), (100, 3), (100, 5),
        (500, 2), (500, 3), (500, 5),
        (1000, 2), (1000, 5),
    ])]
    fn bench(bencher: divan::Bencher, (pop_size, n_obj): (usize, usize)) {
        let objectives = random_objectives(pop_size, n_obj);
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        bencher.bench(|| {
            let _ = non_dominated_sort(&refs);
        });
    }
}

mod crowding_distance {
    use super::*;

    /// args = (pop_size, num_objectives)
    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 2usize), (10, 3), (10, 5),
        (50, 2), (50, 3), (50, 5),
        (100, 2), (100, 3), (100, 5),
        (500, 2), (500, 3), (500, 5),
    ])]
    fn bench(bencher: divan::Bencher, (pop_size, n_obj): (usize, usize)) {
        let objectives = random_objectives(pop_size, n_obj);
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        bencher.bench(|| {
            let mut crowding = vec![0.0_f64; pop_size];
            assign_crowding_distance(&refs, &mut crowding);
        });
    }
}

fn main() {
    divan::main();
}
