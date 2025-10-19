// Test2X test cases with 3FLIP (horizontal, vertical, and both flips)
// Tests that pathfinding is symmetric under grid transformations

use rustgame3::{Grid, pathfinding::find_path_by_id};
use std::fs;
use serde_json;

fn load_test_grid() -> Grid {
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    Grid::with_blocked(104, 83, &blocked)
}

/// Flip grid horizontally (mirror left-right)
fn flip_grid_horizontal(grid: &Grid) -> Grid {
    let rows = grid.rows;
    let cols = grid.cols;
    let mut blocked = Vec::new();

    for y in 0..rows {
        for x in 0..cols {
            let cell_value = grid.get_cell(x, y);
            if cell_value != 0 {
                let new_x = cols - 1 - x;
                let new_id = new_x + y * cols;
                blocked.push(new_id);
            }
        }
    }

    Grid::with_blocked(rows, cols, &blocked)
}

/// Flip grid vertically (mirror top-bottom)
fn flip_grid_vertical(grid: &Grid) -> Grid {
    let rows = grid.rows;
    let cols = grid.cols;
    let mut blocked = Vec::new();

    for y in 0..rows {
        for x in 0..cols {
            let cell_value = grid.get_cell(x, y);
            if cell_value != 0 {
                let new_y = rows - 1 - y;
                let new_id = x + new_y * cols;
                blocked.push(new_id);
            }
        }
    }

    Grid::with_blocked(rows, cols, &blocked)
}

/// Flip grid both horizontally and vertically
fn flip_grid_both(grid: &Grid) -> Grid {
    let rows = grid.rows;
    let cols = grid.cols;
    let mut blocked = Vec::new();

    for y in 0..rows {
        for x in 0..cols {
            let cell_value = grid.get_cell(x, y);
            if cell_value != 0 {
                let new_x = cols - 1 - x;
                let new_y = rows - 1 - y;
                let new_id = new_x + new_y * cols;
                blocked.push(new_id);
            }
        }
    }

    Grid::with_blocked(rows, cols, &blocked)
}

/// Transform cell ID for horizontal flip
fn flip_id_horizontal(id: i32, cols: i32) -> i32 {
    let (x, y) = (id % cols, id / cols);
    let new_x = cols - 1 - x;
    new_x + y * cols
}

/// Transform cell ID for vertical flip
fn flip_id_vertical(id: i32, cols: i32, rows: i32) -> i32 {
    let (x, y) = (id % cols, id / cols);
    let new_y = rows - 1 - y;
    x + new_y * cols
}

/// Transform cell ID for both flips
fn flip_id_both(id: i32, cols: i32, rows: i32) -> i32 {
    let (x, y) = (id % cols, id / cols);
    let new_x = cols - 1 - x;
    let new_y = rows - 1 - y;
    new_x + new_y * cols
}

/// Helper to run pathfinding and validate with 3FLIP
fn assert_path_3flip(
    grid: &Grid,
    start: i32,
    dest: i32,
    expected_waypoints: Option<&[i32]>,
    test_name: &str,
) {
    let cols = grid.cols;
    let rows = grid.rows;

    // Test variants: original, h_flip, v_flip, hv_flip
    let variants = vec![
        ("original", grid.clone(), start, dest, expected_waypoints.map(|w| w.to_vec())),
        (
            "h_flip",
            flip_grid_horizontal(grid),
            flip_id_horizontal(start, cols),
            flip_id_horizontal(dest, cols),
            expected_waypoints.map(|w| w.iter().map(|&id| flip_id_horizontal(id, cols)).collect()),
        ),
        (
            "v_flip",
            flip_grid_vertical(grid),
            flip_id_vertical(start, cols, rows),
            flip_id_vertical(dest, cols, rows),
            expected_waypoints.map(|w| w.iter().map(|&id| flip_id_vertical(id, cols, rows)).collect()),
        ),
        (
            "hv_flip",
            flip_grid_both(grid),
            flip_id_both(start, cols, rows),
            flip_id_both(dest, cols, rows),
            expected_waypoints.map(|w| w.iter().map(|&id| flip_id_both(id, cols, rows)).collect()),
        ),
    ];

    for (variant_name, variant_grid, variant_start, variant_dest, variant_waypoints) in variants {
        let result = find_path_by_id(&variant_grid, variant_start, variant_dest, false, false);

        match (result, variant_waypoints.as_ref()) {
            (Some((path, _dist)), Some(waypoints)) => {
                // Build expected full path: start + waypoints + dest
                let mut expected_full = vec![variant_start];
                expected_full.extend_from_slice(waypoints);
                expected_full.push(variant_dest);

                assert_eq!(
                    path, expected_full,
                    "{} ({}): Path mismatch. Expected {:?}, got {:?}",
                    test_name, variant_name, expected_full, path
                );
            }
            (Some((path, _)), None) => {
                // Expected direct path (no waypoints) = [start, dest]
                let expected_direct = vec![variant_start, variant_dest];
                assert_eq!(
                    path, expected_direct,
                    "{} ({}): Expected direct path {:?}, got {:?}",
                    test_name, variant_name, expected_direct, path
                );
            }
            (None, Some(waypoints)) => {
                panic!("{} ({}): Expected path with waypoints {:?}, but got None", test_name, variant_name, waypoints);
            }
            (None, None) => {
                panic!("{} ({}): Both result and expected are None - this case shouldn't happen in valid tests", test_name, variant_name);
            }
        }
    }
}

// Test cases - using a subset of the full test suite for 3FLIP validation
#[test]
fn test2x_001_4396_to_1211_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 4396, 1211, Some(&[4056, 4310]), "001_3flip");
}

#[test]
fn test2x_002_1211_to_4396_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 1211, 4396, Some(&[4310, 4056]), "002_3flip");
}

#[test]
fn test2x_003_1211_to_4310_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 1211, 4310, Some(&[4056]), "003_3flip");
}

#[test]
fn test2x_004_875_to_4396_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 875, 4396, Some(&[4310, 4056, 1211]), "004_3flip");
}

#[test]
fn test2x_005_4396_to_875_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 4396, 875, Some(&[1211, 4056, 4310]), "005_3flip");
}

#[test]
fn test2x_006_2679_to_2768_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 2679, 2768, Some(&[2851, 2845]), "006_3flip");
}

#[test]
fn test2x_007_2597_to_2768_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 2597, 2768, Some(&[2851, 2845, 2679]), "007_3flip");
}

#[test]
fn test2x_010_2254_to_2255_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 2254, 2255, None, "010_3flip");
}

#[test]
fn test2x_011_4240_to_4980_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 4240, 4980, Some(&[4732]), "011_3flip");
}

#[test]
fn test2x_013_947_to_2110_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 947, 2110, Some(&[1114]), "013_3flip");
}

#[test]
fn test2x_016_946_to_2437_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 946, 2437, Some(&[2523, 3769, 3771, 2194, 1114]), "016_3flip");
}

#[test]
fn test2x_017_946_to_2440_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 946, 2440, Some(&[3769, 3771, 2194, 1114]), "017_3flip");
}

#[test]
fn test2x_018_4829_to_4750_3flip() {
    let grid = load_test_grid();
    assert_path_3flip(&grid, 4829, 4750, Some(&[5247, 5245]), "018_3flip");
}

#[test]
fn test2x_019_4829_to_4750_with_blocks_3flip() {
    let mut grid = load_test_grid();
    let (x1, y1) = grid.get_coords(4998);
    let (x2, y2) = grid.get_coords(4999);
    grid.set_cell(x1, y1, 1);
    grid.set_cell(x2, y2, 1);
    assert_path_3flip(&grid, 4829, 4750, Some(&[4917, 5083, 5247, 5245]), "019_3flip");
}
