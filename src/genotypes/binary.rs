//! A binary gene with an identifier and a boolean value.
//!
//! This struct implements the `GeneT` trait, allowing it to be used in genetic
//! algorithms. The `id` field uniquely identifies the gene, while the `value` field
//! represents its binary state.
//!
//! # Examples
//!
//! ```
//! use genetic_algorithms::genotypes::Binary;
//! use genetic_algorithms::traits::GeneT;
//!
//! let mut gene = <Binary as Default>::default();
//! gene.set_id(1);
//! gene.set_value(true);
//! assert_eq!(gene.get_id(), 1);
//! assert_eq!(gene.get_value(), true);
//! ```
//!
//! The binary gene can be used in mutation and crossover operations to
//! evolve populations in a genetic algorithm.

use crate::traits::GeneT;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Binary{
    pub id: i32,
    pub value: bool,
}
impl GeneT for Binary {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

impl Binary{
    pub fn new(&mut self, id: i32, value: bool) -> &mut Self {
        self.id = id;
        self.value = value;
        self
    }
    pub fn get_value(&self) -> bool {
        self.value
    }
    pub fn set_value(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }
}