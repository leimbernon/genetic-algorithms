use rand::Rng;
use crate::genotypes::Binary as BinaryGenotype;

pub fn binary_random_initialization(genes_per_chromosome: i32, _alleles: Option<&[BinaryGenotype]>, _needs_unique_ids: Option<bool>) -> Vec<BinaryGenotype> {
    let mut genes = Vec::new();
    let mut rng = rand::thread_rng();
    for i in 0..genes_per_chromosome {
        let gene = BinaryGenotype{id:i, value: rng.gen_bool(0.5)};
        genes.push(gene);
    }
    genes
}