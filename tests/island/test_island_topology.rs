use genetic_algorithms::island::topology::{neighbors, MigrationTopology};

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
    let result = neighbors(0, 6, &MigrationTopology::Grid(2, 3));
    assert_eq!(result, vec![3, 1]);
}

#[test]
fn test_topology_grid_center() {
    let result = neighbors(4, 9, &MigrationTopology::Grid(3, 3));
    assert_eq!(result, vec![1, 7, 3, 5]);
}

#[test]
fn test_topology_grid_edge() {
    let result = neighbors(1, 6, &MigrationTopology::Grid(2, 3));
    assert_eq!(result, vec![4, 0, 2]);
}

#[test]
fn test_topology_grid_bottom_right() {
    let result = neighbors(5, 6, &MigrationTopology::Grid(2, 3));
    assert_eq!(result, vec![2, 4]);
}

// --- Hypercube topology tests ---

#[test]
fn test_topology_hypercube_4_islands() {
    let result = neighbors(0, 4, &MigrationTopology::Hypercube);
    assert_eq!(result, vec![1, 2]);

    let result = neighbors(3, 4, &MigrationTopology::Hypercube);
    assert_eq!(result, vec![2, 1]);
}

#[test]
fn test_topology_hypercube_8_islands() {
    let result = neighbors(5, 8, &MigrationTopology::Hypercube);
    assert_eq!(result, vec![4, 7, 1]);
}

#[test]
fn test_topology_hypercube_2_islands() {
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
    let result = neighbors(5, 6, &MigrationTopology::Custom(adj));
    assert!(result.is_empty());
}
