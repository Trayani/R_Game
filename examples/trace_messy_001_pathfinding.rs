// Trace full pathfinding for messy_001 with detailed logging

use rustgame3::{Grid, pathfinding::find_path_by_id};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let start = 6388;
    let dest = 6139;
    let messy_x = false;
    let messy_y = true;

    println!("===== TRACING PATHFINDING FOR MESSY_001 =====");
    println!("Start: {} at ({}, {})", start, 80, 76);
    println!("Dest:  {} at ({}, {})", dest, 80, 73);
    println!("messyX={}, messyY={}", messy_x, messy_y);
    println!();

    // Enable tracing in the pathfinding module would require changing const TRACE_PATHFINDING
    // Instead, let's manually trace the key steps

    println!("Expected from C#:");
    println!("  - Distance: 7.41");
    println!("  - Waypoints: [6141, 6307, 6306, 6389]");
    println!("  - Full path: [6388, 6141, 6307, 6306, 6389, 6139]");
    println!();

    println!("Running Rust pathfinding...");
    let result = find_path_by_id(&grid, start, dest, messy_x, messy_y);

    match result {
        Some((path, dist)) => {
            println!("✓ PATH FOUND!");
            println!("Distance: {:.2}", dist);
            println!("Full path: {:?}", path);

            if path.len() > 2 {
                let waypoints: Vec<i32> = path[1..path.len()-1].to_vec();
                println!("Waypoints: {:?}", waypoints);

                let expected = vec![6141, 6307, 6306, 6389];
                if waypoints == expected {
                    println!("✓✓✓ MATCHES C# EXPECTED WAYPOINTS!");
                } else {
                    println!("✗ MISMATCH");
                    println!("  Expected: {:?}", expected);
                    println!("  Got:      {:?}", waypoints);
                }
            }
        }
        None => {
            println!("✗ NO PATH FOUND");
            println!();
            println!("This means one of the following failed:");
            println!("  1. Start raycast returned 0 cells");
            println!("  2. Start has no interesting corners");
            println!("  3. Dest raycast returned 0 cells");
            println!("  4. Dest has no interesting corners (finished corners)");
            println!("  5. A* search failed to connect start corners to finished corners");
            println!();
            println!("From test_raycast_6139.rs we know:");
            println!("  - Start (80,76) with messyY=true: 4 visible cells ✓");
            println!("  - Dest (80,73) with clean: 3 visible cells ✓");
            println!("  - So raycasting itself works!");
            println!();
            println!("The bug must be in:");
            println!("  - Corner filtering logic");
            println!("  - A* search connectivity");
            println!("  - Or some other pathfinding step");
        }
    }
}
