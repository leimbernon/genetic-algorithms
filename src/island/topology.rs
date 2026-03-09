/// Migration topology for the island model.
///
/// Determines how islands are connected and which islands exchange individuals
/// during migration events.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_ring_neighbors() {
        let result = neighbors(0, 4, &MigrationTopology::Ring);
        assert_eq!(result, vec![1]);

        let result = neighbors(3, 4, &MigrationTopology::Ring);
        assert_eq!(result, vec![0]);

        let result = neighbors(2, 4, &MigrationTopology::Ring);
        assert_eq!(result, vec![3]);
    }

    #[test]
    fn test_topology_fully_connected_neighbors() {
        let result = neighbors(0, 4, &MigrationTopology::FullyConnected);
        assert_eq!(result, vec![1, 2, 3]);

        let result = neighbors(2, 4, &MigrationTopology::FullyConnected);
        assert_eq!(result, vec![0, 1, 3]);
    }

    #[test]
    fn test_topology_single_island() {
        let result = neighbors(0, 1, &MigrationTopology::Ring);
        assert!(result.is_empty());

        let result = neighbors(0, 1, &MigrationTopology::FullyConnected);
        assert!(result.is_empty());
    }

    #[test]
    fn test_topology_two_islands_ring() {
        let result = neighbors(0, 2, &MigrationTopology::Ring);
        assert_eq!(result, vec![1]);

        let result = neighbors(1, 2, &MigrationTopology::Ring);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_topology_default_is_ring() {
        let t = MigrationTopology::default();
        assert_eq!(t, MigrationTopology::Ring);
    }

    // --- Grid topology tests ---

    #[test]
    fn test_topology_grid_corner() {
        // 2x3 grid: indices 0..5
        // Layout:
        //  0  1  2
        //  3  4  5
        // Corner (0,0) = index 0: neighbors are down(3) and right(1)
        let result = neighbors(0, 6, &MigrationTopology::Grid(2, 3));
        assert_eq!(result, vec![3, 1]);
    }

    #[test]
    fn test_topology_grid_center() {
        // 3x3 grid: indices 0..8
        // Layout:
        //  0  1  2
        //  3  4  5
        //  6  7  8
        // Center (1,1) = index 4: neighbors are up(1), down(7), left(3), right(5)
        let result = neighbors(4, 9, &MigrationTopology::Grid(3, 3));
        assert_eq!(result, vec![1, 7, 3, 5]);
    }

    #[test]
    fn test_topology_grid_edge() {
        // 2x3 grid
        // Index 1 = (0,1): neighbors are down(4), left(0), right(2)
        let result = neighbors(1, 6, &MigrationTopology::Grid(2, 3));
        assert_eq!(result, vec![4, 0, 2]);
    }

    #[test]
    fn test_topology_grid_bottom_right() {
        // 2x3 grid
        // Index 5 = (1,2): neighbors are up(2), left(4)
        let result = neighbors(5, 6, &MigrationTopology::Grid(2, 3));
        assert_eq!(result, vec![2, 4]);
    }

    // --- Hypercube topology tests ---

    #[test]
    fn test_topology_hypercube_4_islands() {
        // 4 islands = 2-bit hypercube
        // Index 0 (00): neighbors 1 (01), 2 (10)
        let result = neighbors(0, 4, &MigrationTopology::Hypercube);
        assert_eq!(result, vec![1, 2]);

        // Index 3 (11): neighbors 2 (10), 1 (01)
        let result = neighbors(3, 4, &MigrationTopology::Hypercube);
        assert_eq!(result, vec![2, 1]);
    }

    #[test]
    fn test_topology_hypercube_8_islands() {
        // 8 islands = 3-bit hypercube
        // Index 5 (101): neighbors 4 (100), 7 (111), 1 (001)
        let result = neighbors(5, 8, &MigrationTopology::Hypercube);
        assert_eq!(result, vec![4, 7, 1]);
    }

    #[test]
    fn test_topology_hypercube_2_islands() {
        // 2 islands = 1-bit hypercube
        let result = neighbors(0, 2, &MigrationTopology::Hypercube);
        assert_eq!(result, vec![1]);

        let result = neighbors(1, 2, &MigrationTopology::Hypercube);
        assert_eq!(result, vec![0]);
    }

    // --- Custom topology tests ---

    #[test]
    fn test_topology_custom_adjacency() {
        let adj = vec![
            vec![1, 2], // island 0 -> 1, 2
            vec![0],    // island 1 -> 0
            vec![0, 1], // island 2 -> 0, 1
        ];
        let result = neighbors(0, 3, &MigrationTopology::Custom(adj.clone()));
        assert_eq!(result, vec![1, 2]);

        let result = neighbors(1, 3, &MigrationTopology::Custom(adj.clone()));
        assert_eq!(result, vec![0]);

        let result = neighbors(2, 3, &MigrationTopology::Custom(adj));
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_topology_custom_out_of_bounds_returns_empty() {
        let adj = vec![vec![1]];
        // Asking for island 5 which doesn't exist in adjacency list
        let result = neighbors(5, 6, &MigrationTopology::Custom(adj));
        assert!(result.is_empty());
    }
}
