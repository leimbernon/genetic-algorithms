use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::CrossoverConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover;
use genetic_algorithms::operations::Crossover;
use genetic_algorithms::traits::ChromosomeT;

fn build_range_chromosome_from(vals: &[i32], n: i32) -> RangeChromosome<i32> {
    let mut c = RangeChromosome::<i32>::new();
    let dna: Vec<_> = vals
        .iter()
        .enumerate()
        .map(|(i, v)| RangeGenotype::new(i as i32, vec![(0, n - 1)], *v))
        .collect();
    use std::borrow::Cow;
    c.set_dna(Cow::Borrowed(&dna));
    c
}

#[test]
fn uniform_crossover_preserves_length_and_values_from_parents() {
    let n = 8;
    let p1 = build_range_chromosome_from(&(0..n).collect::<Vec<_>>(), n);
    let p2 = build_range_chromosome_from(&(0..n).rev().collect::<Vec<_>>(), n);

    let cfg = CrossoverConfiguration {
        number_of_points: None,
        probability_max: None,
        probability_min: None,
        method: Crossover::Uniform,
    };

    let children = crossover::factory(&p1, &p2, cfg).expect("Crossover should succeed");
    assert_eq!(children.len(), 2);
    for child in children.iter() {
        assert_eq!(child.get_dna().len(), p1.get_dna().len());
        for (i, gene) in child.get_dna().iter().enumerate() {
            let v = gene.value;
            let v1 = p1.get_dna()[i].value;
            let v2 = p2.get_dna()[i].value;
            assert!(
                v == v1 || v == v2,
                "Child gene {} value {} not taken from parents {} or {}",
                i,
                v,
                v1,
                v2
            );
        }
    }
}
