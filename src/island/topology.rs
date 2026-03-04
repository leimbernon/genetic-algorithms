/// Migration topology for the island model.
///
/// Determines how islands are connected and which islands exchange individuals
/// during migration events.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MigrationTopology {
    /// Ring topology: each island sends migrants to the next island in a circular arrangement.
    ///
    /// Island `i` migrates to island `(i + 1) % num_islands`.
    #[default]
    Ring,
    /// Fully connected topology: each island sends migrants to every other island.
    FullyConnected,
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
    }
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
}
