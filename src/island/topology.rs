/// Migration topology for the island model.
///
/// Determines how islands are connected and which islands exchange individuals
/// during migration events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MigrationTopology {
    /// Ring topology: each island sends migrants to the next island in a circular arrangement.
    ///
    /// Island `i` migrates to island `(i + 1) % num_islands`.
    Ring,
    /// Fully connected topology: each island sends migrants to every other island.
    FullyConnected,
}

impl Default for MigrationTopology {
    fn default() -> Self {
        MigrationTopology::Ring
    }
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
pub fn get_neighbors(
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_ring_neighbors() {
        let neighbors = get_neighbors(0, 4, &MigrationTopology::Ring);
        assert_eq!(neighbors, vec![1]);

        let neighbors = get_neighbors(3, 4, &MigrationTopology::Ring);
        assert_eq!(neighbors, vec![0]);

        let neighbors = get_neighbors(2, 4, &MigrationTopology::Ring);
        assert_eq!(neighbors, vec![3]);
    }

    #[test]
    fn test_topology_fully_connected_neighbors() {
        let neighbors = get_neighbors(0, 4, &MigrationTopology::FullyConnected);
        assert_eq!(neighbors, vec![1, 2, 3]);

        let neighbors = get_neighbors(2, 4, &MigrationTopology::FullyConnected);
        assert_eq!(neighbors, vec![0, 1, 3]);
    }

    #[test]
    fn test_topology_single_island() {
        let neighbors = get_neighbors(0, 1, &MigrationTopology::Ring);
        assert!(neighbors.is_empty());

        let neighbors = get_neighbors(0, 1, &MigrationTopology::FullyConnected);
        assert!(neighbors.is_empty());
    }

    #[test]
    fn test_topology_two_islands_ring() {
        let neighbors = get_neighbors(0, 2, &MigrationTopology::Ring);
        assert_eq!(neighbors, vec![1]);

        let neighbors = get_neighbors(1, 2, &MigrationTopology::Ring);
        assert_eq!(neighbors, vec![0]);
    }

    #[test]
    fn test_topology_default_is_ring() {
        let t = MigrationTopology::default();
        assert_eq!(t, MigrationTopology::Ring);
    }
}
