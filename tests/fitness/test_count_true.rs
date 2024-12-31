use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::fitness::count_true;

#[cfg(test)]

#[test]
fn test_count_true() {
    let dna = vec![Binary{id:1, value:false}, Binary{id:2, value:true},
    Binary{id:3, value:false}, Binary{id:4, value:true}, Binary{id:5, value:false}];

    assert_eq!(count_true(&dna),2.0);
}