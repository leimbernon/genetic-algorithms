//! Behavioral diversity comparison: Lexicase vs Tournament.

/// Tests that lexicase selection produces higher case-score variance across
/// surviving genotypes than tournament on a matched-effort multi-case benchmark (SEL-02 criterion 4).
///
/// Setup: 50 specialists (5 groups × 10 individuals, each group dominant on one case with
/// case_score=1.0 and scalar fitness=0.2) + 10 generalists (all cases 0.3, scalar fitness=0.3).
///
/// Tournament strongly prefers generalists (scalar 0.3 > 0.2) → selected pool
/// is overwhelmingly generalist → low case-score variance.
///
/// Lexicase evaluates case-by-case in shuffled order. Specialists score 1.0 on their case
/// vs generalists' 0.3 → specialists dominate their case's filter → high case-score variance.
#[test]
fn test_lexicase_produces_more_specialists_than_tournament() {
    use crate::structures::{Gene, MultiCaseChromosome};
    use genetic_algorithms::{
        configuration::SelectionConfiguration,
        fitness::FitnessFnWrapper,
        operations::{selection, selection::factory_lexicase, Selection},
        traits::MultiCaseFitness,
    };

    const K: usize = 5;            // number of cases
    const N_SPECIALISTS: usize = 50; // 5 groups × 10 specialists
    const N_GENERALISTS: usize = 10;
    const COUPLES: usize = 50;

    // Specialists: dominant on exactly one case (score=1.0), others=0.0
    // Scalar fitness = mean = 1/K = 0.2
    let mut pop: Vec<MultiCaseChromosome> = (0..N_SPECIALISTS)
        .map(|i| {
            let group = i / 10; // groups 0..4
            let mut scores = vec![0.0f64; K];
            scores[group] = 1.0;
            let mean = 1.0 / K as f64; // 0.2
            let mut c = MultiCaseChromosome {
                dna: vec![Gene { id: i as i32 }],
                fitness: mean,
                age: 0,
                case_scores: vec![],
                fitness_fn: FitnessFnWrapper::default(),
            };
            c.set_case_fitness(scores);
            c
        })
        .collect();

    // Generalists: every case = 0.3, scalar fitness = 0.3 (strictly beats specialist's 0.2)
    for i in N_SPECIALISTS..(N_SPECIALISTS + N_GENERALISTS) {
        let scores = vec![0.3f64; K];
        let mut c = MultiCaseChromosome {
            dna: vec![Gene { id: i as i32 }],
            fitness: 0.3,
            age: 0,
            case_scores: vec![],
            fitness_fn: FitnessFnWrapper::default(),
        };
        c.set_case_fitness(scores);
        pop.push(c);
    }

    // Clone for tournament
    let pop_tour = pop.clone();

    // --- Lexicase selection ---
    let lex_config = SelectionConfiguration {
        method: Selection::Lexicase,
        number_of_couples: COUPLES,
        ..Default::default()
    };
    let lex_pairs = factory_lexicase(&mut pop, lex_config, 1)
        .expect("lexicase selection failed");

    // --- Tournament selection ---
    let tour_config = SelectionConfiguration {
        method: Selection::Tournament,
        number_of_couples: COUPLES,
        ..Default::default()
    };
    let tour_pairs = selection::factory(&pop_tour, tour_config, 1, 2)
        .expect("tournament selection failed");

    // Compute average per-case variance across selected individuals.
    // Higher variance = more diverse case-score profiles = more specialists selected.
    let avg_case_variance = |pairs: &[Vec<usize>], population: &[MultiCaseChromosome]| -> f64 {
        let indices: Vec<usize> = pairs.iter().flat_map(|group| [group[0], group[1]]).collect();
        let total = indices.len() as f64;
        let mut total_var = 0.0;
        for case_i in 0..K {
            let scores: Vec<f64> = indices
                .iter()
                .map(|&idx| population[idx].case_fitness()[case_i])
                .collect();
            let mean = scores.iter().sum::<f64>() / total;
            let var = scores.iter().map(|&s| (s - mean).powi(2)).sum::<f64>() / total;
            total_var += var;
        }
        total_var / K as f64
    };

    let var_lex = avg_case_variance(&lex_pairs, &pop);
    let var_tour = avg_case_variance(&tour_pairs, &pop_tour);

    // Lexicase should produce substantially more diverse selections than tournament.
    // With generalists dominating scalar fitness, tournament converges on generalists
    // (low variance). Lexicase's case-shuffling gives specialists a chance per case.
    assert!(
        var_lex >= 1.2 * var_tour,
        "Lexicase avg case-score variance ({:.4}) should be >= 1.2x tournament's ({:.4}). \
         Specialists dominate per-case filters in lexicase, but tournament prefers generalists \
         with higher scalar fitness.",
        var_lex,
        var_tour
    );
}
