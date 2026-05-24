use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::pcx::pcx;
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
fn pcx_produces_one_offspring_within_bounds() {
    let parents = build_parents(3);
    let parent_refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();
    for _ in 0..100 {
        let result = pcx(&parent_refs, 3, None, None);
        assert!(result.is_ok(), "pcx returned Err: {:?}", result.err());
        let children = result.unwrap();
        assert_eq!(children.len(), 1, "pcx should produce exactly 1 offspring");
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
fn pcx_offspring_biased_toward_primary_parent() {
    // 3 parents: primary at [10, 80], others at [30, 60] and [50, 40]
    let coords: &[(f64, f64)] = &[(10.0, 80.0), (30.0, 60.0), (50.0, 40.0)];
    let parents: Vec<RangeChromosome<f64>> = coords
        .iter()
        .map(|&(x, y)| {
            let mut p = RangeChromosome::<f64>::new();
            let dna = vec![
                RangeGenotype::new(0, vec![(0.0, 100.0)], x),
                RangeGenotype::new(1, vec![(0.0, 100.0)], y),
            ];
            p.set_dna(Cow::Owned(dna));
            p
        })
        .collect();
    let parent_refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();

    let primary = &parents[0];
    // centroid of parents[1..]
    let centroid_x = coords[1..].iter().map(|&(x, _)| x).sum::<f64>() / (coords.len() - 1) as f64;
    let centroid_y = coords[1..].iter().map(|&(_, y)| y).sum::<f64>() / (coords.len() - 1) as f64;

    let mut dist_to_primary_sum = 0.0f64;
    let mut dist_to_centroid_sum = 0.0f64;

    let n_samples = 200;
    for _ in 0..n_samples {
        let result = pcx(&parent_refs, 3, None, None);
        assert!(result.is_ok());
        let children = result.unwrap();
        let child = &children[0];
        let cx = child.dna()[0].value;
        let cy = child.dna()[1].value;

        let dx_p = cx - primary.dna()[0].value;
        let dy_p = cy - primary.dna()[1].value;
        dist_to_primary_sum += (dx_p * dx_p + dy_p * dy_p).sqrt();

        let dx_c = cx - centroid_x;
        let dy_c = cy - centroid_y;
        dist_to_centroid_sum += (dx_c * dx_c + dy_c * dy_c).sqrt();
    }

    let mean_dist_primary = dist_to_primary_sum / n_samples as f64;
    let mean_dist_centroid = dist_to_centroid_sum / n_samples as f64;

    assert!(
        mean_dist_primary < mean_dist_centroid,
        "PCX offspring should be biased toward primary parent: mean_dist_primary={:.4} vs mean_dist_centroid={:.4}",
        mean_dist_primary,
        mean_dist_centroid
    );
}
