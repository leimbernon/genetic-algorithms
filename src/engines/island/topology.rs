/// Migration topology for the island model.
///
/// Determines how islands are connected and which islands exchange individuals
/// during migration events.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::island::topology::MigrationTopology;
///
/// let topology = MigrationTopology::Ring;
/// assert_eq!(topology, MigrationTopology::Ring);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MigrationTopology {
    /// Ring topology: each island sends migrants to the next island in a circular arrangement.
    ///
    /// Island `i` migrates to island `(i + 1) % num_islands`.
    #[default]
    Ring,
    /// Fully connected topology: each island sends migrants to every other island.
    FullyConnected,
    /// 2D grid (lattice) topology with the given number of rows and columns.
    ///
    /// Each island is connected to its 4-neighbors (up, down, left, right).
    /// The total number of islands must equal `rows * cols`.
    Grid(usize, usize),
    /// Hypercube topology for power-of-2 island counts.
    ///
    /// Two islands are neighbors if their indices differ in exactly one bit.
    Hypercube,
    /// User-defined adjacency list.
    ///
    /// `adjacency[i]` is the list of neighbor indices for island `i`.
    Custom(Vec<Vec<usize>>),
}

/// Returns the list of neighbor island indices for a given island under the specified topology.
///
/// # Arguments
///
/// * `island_index` - The index of the source island.
/// * `num_islands` - Total number of islands.
/// * `topology` - The migration topology.
///
/// # Returns
///
/// A vector of island indices that are neighbors of `island_index`.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::island::topology::{MigrationTopology, neighbors};
///
/// let ns = neighbors(0, 4, &MigrationTopology::Ring);
/// assert_eq!(ns, vec![1]);
/// ```
pub fn neighbors(
    island_index: usize,
    num_islands: usize,
    topology: &MigrationTopology,
) -> Vec<usize> {
    if num_islands <= 1 {
        return vec![];
    }
    match topology {
        MigrationTopology::Ring => {
            vec![(island_index + 1) % num_islands]
        }
        MigrationTopology::FullyConnected => {
            (0..num_islands).filter(|&i| i != island_index).collect()
        }
        MigrationTopology::Grid(rows, cols) => grid_neighbors(island_index, *rows, *cols),
        MigrationTopology::Hypercube => hypercube_neighbors(island_index, num_islands),
        MigrationTopology::Custom(adjacency) => {
            adjacency.get(island_index).cloned().unwrap_or_default()
        }
    }
}

/// Returns the 4-connected neighbors on a 2D grid.
fn grid_neighbors(index: usize, rows: usize, cols: usize) -> Vec<usize> {
    let row = index / cols;
    let col = index % cols;
    let mut result = Vec::with_capacity(4);

    // Up
    if row > 0 {
        result.push((row - 1) * cols + col);
    }
    // Down
    if row + 1 < rows {
        result.push((row + 1) * cols + col);
    }
    // Left
    if col > 0 {
        result.push(row * cols + (col - 1));
    }
    // Right
    if col + 1 < cols {
        result.push(row * cols + (col + 1));
    }

    result
}

/// Returns neighbors in a hypercube topology (indices differing by one bit).
fn hypercube_neighbors(index: usize, num_islands: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let bits = (num_islands as f64).log2().ceil() as u32;
    for bit in 0..bits {
        let neighbor = index ^ (1 << bit);
        if neighbor < num_islands {
            result.push(neighbor);
        }
    }
    result
}
