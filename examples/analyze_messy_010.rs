// Analyze test2x_messy_010: 6637 -> 6552
// Expected: [6637, 6636, 6637, 6552]
// Got: [6637, 6636, 6639, 6552]

use rustgame3::Grid;
use rustgame3::pathfinding::find_path_by_id;
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let start = 6637;
    let dest = 6552;

    let (start_x, start_y) = grid.get_coords(start);
    let (dest_x, dest_y) = grid.get_coords(dest);

    println!("===== TEST messy_010: {} -> {} =====", start, dest);
    println!("Start: {} at ({}, {})", start, start_x, start_y);
    println!("Dest:  {} at ({}, {})", dest, dest_x, dest_y);
    println!();

    // Check if same row
    if start_y == dest_y {
        println!("✓ Same row (Y={})!", start_y);
    } else {
        println!("✗ Different rows: start Y={}, dest Y={}", start_y, dest_y);
    }
    println!();

    // Check expected waypoints
    let expected_waypoints = vec![6636, 6637];
    println!("Expected waypoints: {:?}", expected_waypoints);
    for &wp in &expected_waypoints {
        let (wp_x, wp_y) = grid.get_coords(wp);
        println!("  Waypoint {} at ({}, {})", wp, wp_x, wp_y);
    }
    println!();

    println!("===== Analysis =====");
    println!("Expected path: [{}, {}, {}, {}]", start, expected_waypoints[0], expected_waypoints[1], dest);
    println!("Notice: START POSITION {} APPEARS TWICE!", start);
    println!("  First occurrence: messy start position");
    println!("  Second occurrence: waypoint (alignment target)");
    println!();
    println!("This is NOT a same-line case (different Y coordinates).");
    println!("This is normal pathfinding with alignment waypoint.");
    println!();

    // Run Rust pathfinding
    println!("===== Rust Pathfinding =====");
    let result = find_path_by_id(&grid, start, dest, false, true);
    match result {
        Some((path, dist)) => {
            println!("Found path: {:?}", path);
            println!("Distance: {:.2}", dist);
        }
        None => {
            println!("No path found!");
        }
    }
}
