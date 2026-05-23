use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::spx::spx;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;

fn build_parents(n: usize) -> Vec<RangeChromosome<f64>> {
    (0..n)
        .map(|k| {
            let mut p = RangeChromosome::<f64>::new();
            let dna = vec![
                RangeGenotype::new(0, vec![(0.0, 100.0)], 10.0 + 20.0 * k as f64),
                RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0 - 10.0 * k as f64),
            ];
            p.set_dna(Cow::Owned(dna));
            p
        })
        .collect()
}

#[test]
fn spx_produces_one_offspring_within_bounds() {
    let parents = build_parents(3);
    let parent_refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();
    for _ in 0..100 {
        let result = spx(&parent_refs, 3);
        assert!(result.is_ok(), "spx returned Err: {:?}", result.err());
        let children = result.unwrap();
        assert_eq!(children.len(), 1, "spx should produce exactly 1 offspring");
        for child in &children {
            for gene in child.dna() {
                let (lo, hi) = gene.ranges[0];
                assert!(
                    gene.value >= lo && gene.value <= hi,
                    "Gene value {} out of range [{}, {}]",
                    gene.value,
                    lo,
                    hi
                );
            }
        }
    }
}

#[test]
fn spx_offspring_within_expanded_simplex() {
    // Parents at [10,10], [20,20], [30,30] — collinear but within a simplex
    let mut parents: Vec<RangeChromosome<f64>> = Vec::with_capacity(3);
    for &v in &[10.0_f64, 20.0, 30.0] {
        let mut p = RangeChromosome::<f64>::new();
        let dna = vec![
            RangeGenotype::new(0, vec![(0.0, 100.0)], v),
            RangeGenotype::new(1, vec![(0.0, 100.0)], v),
        ];
        p.set_dna(Cow::Owned(dna));
        parents.push(p);
    }
    let parent_refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();

    // epsilon = sqrt(n_parents + 2) = sqrt(5)
    // centroid = [20, 20]
    // expanded vertices: centroid + sqrt(5) * (p[k] - centroid)
    // expanded[0] = 20 + sqrt(5)*(10-20) = 20 - 10*sqrt(5) ≈ -2.36
    // expanded[2] = 20 + sqrt(5)*(30-20) = 20 + 10*sqrt(5) ≈ 42.36
    // bounding box: approx [-2.36, 42.36] but clamped to [0, 100]
    // Offspring must lie within the gene range [0, 100] (the genetic bound)
    let mut not_all_centroid = false;
    for _ in 0..100 {
        let result = spx(&parent_refs, 3);
        assert!(result.is_ok());
        let children = result.unwrap();
        assert_eq!(children.len(), 1);
        let child = &children[0];
        for gene in child.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "SPX simplex test: gene {} out of bounds [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
        // Verify offspring aren't always exactly at centroid (20.0, 20.0)
        let v0 = child.dna()[0].value;
        let v1 = child.dna()[1].value;
        if (v0 - 20.0).abs() > 1e-9 || (v1 - 20.0).abs() > 1e-9 {
            not_all_centroid = true;
        }
    }
    assert!(
        not_all_centroid,
        "SPX offspring collapsed to centroid every iteration — sampling from simplex is broken"
    );
}
