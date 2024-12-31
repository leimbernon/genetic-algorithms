use crate::genotypes::Binary;

pub fn count_true(dna: &[Binary]) -> f64 {
    dna.iter().filter(|gene| gene.value).count() as f64
}