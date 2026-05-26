use genetic_algorithms::ChromosomeLength;

#[test]
fn test_chromosome_length_variants() {
    let fixed = ChromosomeLength::Fixed(8);
    let variable = ChromosomeLength::Variable { min: 2, max: 16 };

    assert_eq!(fixed, ChromosomeLength::Fixed(8));
    assert_eq!(variable, ChromosomeLength::Variable { min: 2, max: 16 });
    assert_ne!(fixed, ChromosomeLength::Fixed(9));
    assert_ne!(fixed, ChromosomeLength::Variable { min: 2, max: 16 });

    // Copy semantics
    let fixed_copy = fixed;
    assert_eq!(fixed_copy, fixed);

    // Clone semantics — ChromosomeLength implements Copy; .clone() is equivalent to a copy
    #[allow(clippy::clone_on_copy)]
    let variable_clone = variable.clone();
    assert_eq!(variable_clone, variable);

    // Debug formatting does not panic
    let _ = format!("{:?}", fixed);
    let _ = format!("{:?}", variable);
}

#[test]
fn test_chromosome_length_default_is_fixed_zero() {
    let default = ChromosomeLength::default();
    assert_eq!(default, ChromosomeLength::Fixed(0));
}

#[cfg(feature = "serde")]
#[test]
fn test_chromosome_length_serde_roundtrip() {
    let fixed = ChromosomeLength::Fixed(42);
    let serialized = serde_json::to_string(&fixed).expect("serialize Fixed");
    let deserialized: ChromosomeLength =
        serde_json::from_str(&serialized).expect("deserialize Fixed");
    assert_eq!(fixed, deserialized);

    let variable = ChromosomeLength::Variable { min: 3, max: 10 };
    let serialized = serde_json::to_string(&variable).expect("serialize Variable");
    let deserialized: ChromosomeLength =
        serde_json::from_str(&serialized).expect("deserialize Variable");
    assert_eq!(variable, deserialized);
}
