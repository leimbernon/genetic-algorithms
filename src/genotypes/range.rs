use crate::traits::GeneT;

/// A range gene with an identifier, a list of ranges, and a value.
///
/// This struct implements the `GeneT` trait, allowing it to be used in genetic
/// algorithms. The `id` field uniquely identifies the gene, while the `ranges` field
/// represents a list of value ranges, and the `value` field represents a value within
/// those ranges.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::Range;
/// use genetic_algorithms::traits::GeneT;
///
/// let mut gene = <Range<i32> as Default>::default();
/// gene.set_id(1);
/// gene.set_value(5);
/// assert_eq!(gene.get_id(), 1);
/// assert_eq!(gene.get_value(), 5);
/// ```
///
/// The range gene can be used in mutation and crossover operations to
/// evolve populations in a genetic algorithm.
#[derive(Debug, PartialEq, Clone)]
pub struct Range<T> {
    pub id: i32,
    pub ranges: Vec<(T, T)>,
    pub value: T,
}

impl<T: Default> Default for Range<T> {
    fn default() -> Self {
        Self {
            id: 0,
            ranges: Vec::new(),
            value: Default::default(),
        }
    }
}

impl<T: Sync + Send + Clone + Default> GeneT for Range<T> {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

impl<T: Clone + Default> Range<T> {
    /// Creates a new `Range` gene with the given identifier, ranges, and value.
    ///
    /// # Arguments
    ///
    /// * `id` - An integer representing the unique identifier.
    /// * `ranges` - A vector of tuples representing the range of values.
    /// * `value` - A value within the range.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub fn new(&mut self, id: i32, ranges: Vec<(T, T)>, value: T) -> &mut Self {
        self.id = id;
        self.ranges = ranges;
        self.value = value;
        self
    }

    /// Returns the value of the gene.
    pub fn get_value(&self) -> T {
        self.value.clone()
    }

    /// Sets the value of the gene.
    ///
    /// # Arguments
    ///
    /// * `value` - A value within the range.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub fn set_value(&mut self, value: T) -> &mut Self {
        self.value = value;
        self
    }
}