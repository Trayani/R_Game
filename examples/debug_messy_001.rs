// Debug test2x_messy_001: 6388 -> 6139 with messyY=true
// C# Test Line: ProtoTests.cs:235
// test2(6388, 6139, 1, false, 6141, 6307, 6306, 6389)
// test2 => test3(start, dest, size, reverse=false, messyY=true, messyX=false, exPath)

use rustgame3::{Grid, pathfinding::find_path_by_id, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let start = 6388;
    let dest = 6139;
    let messy_x = false;
    let messy_y = true;

    println!("===== DEBUG TEST messy_001: {} -> {} =====", start, dest);
    println!("messyX={}, messyY={}", messy_x, messy_y);
    println!("C# Test Line: ProtoTests.cs:235");
    println!("Expected waypoints: [6141, 6307, 6306, 6389]");
    println!();

    let (start_x, start_y) = grid.get_coords(start);
    let (dest_x, dest_y) = grid.get_coords(dest);
    println!("Start: {} at ({}, {})", start, start_x, start_y);
    println!("Dest:  {} at ({}, {})", dest, dest_x, dest_y);
    println!();

    // Check if start and dest are free
    println!("===== GRID CELL VALIDATION =====");
    println!("Start cell value: {}", grid.get_cell(start_x, start_y));
    println!("Dest cell value: {}", grid.get_cell(dest_x, dest_y));
    println!();

    // Run pathfinding
    println!("===== RUNNING PATHFINDING =====");
    let result = find_path_by_id(&grid, start, dest, messy_x, messy_y);

    match result {
        Some((path, dist)) => {
            println!("✓ Path found!");
            println!("Distance: {:.2}", dist);
            println!("Full path: {:?}", path);

            if path.len() > 2 {
                let waypoints = &path[1..path.len()-1];
                println!("Waypoints only: {:?}", waypoints);
            }

            let expected = vec![6141, 6307, 6306, 6389];
            let actual_waypoints: Vec<i32> = path[1..path.len()-1].to_vec();

            if actual_waypoints == expected {
                println!("✓ Matches C# expected waypoints!");
            } else {
                println!("✗ MISMATCH!");
                println!("  Expected: {:?}", expected);
                println!("  Got:      {:?}", actual_waypoints);
            }
        }
        None => {
            println!("✗ NO PATH FOUND (Rust returns None)");
            println!();
            println!("===== INVESTIGATING WHY NO PATH =====");

            // Check corner discovery from start
            println!("\n--- Corners from start {} ---", start);
            let start_visible = raycast(&grid, start_x, start_y, messy_x, messy_y);
            println!("Start visible cells: {}", start_visible.len());

            let all_corners = detect_all_corners(&grid);
            println!("Total corners in grid: {}", all_corners.len());

            let start_corners = filter_interesting_corners(
                &all_corners, &start_visible, &grid, start_x, start_y, messy_y
            );
            println!("Interesting corners from start: {}", start_corners.len());

            if start_corners.is_empty() {
                println!("⚠️  START HAS NO INTERESTING CORNERS!");
                println!("This means start cannot navigate anywhere.");
            } else {
                println!("Start corners (first 10):");
                for (i, corner) in start_corners.iter().take(10).enumerate() {
                    let corner_id = grid.get_id(corner.x, corner.y);
                    let dx = (corner.x - start_x) as f64;
                    let dy = (corner.y - start_y) as f64;
                    let dist = (dx*dx + dy*dy).sqrt();
                    println!("  [{}] ID {} at ({},{}) - dist {:.2}", i, corner_id, corner.x, corner.y, dist);
                }
            }

            // Check corner discovery from dest
            println!("\n--- Corners from dest {} ---", dest);
            let dest_visible = raycast(&grid, dest_x, dest_y, messy_x, messy_y);
            println!("Dest visible cells: {}", dest_visible.len());

            let dest_corners = filter_interesting_corners(
                &all_corners, &dest_visible, &grid, dest_x, dest_y, messy_y
            );
            println!("Interesting corners from dest: {}", dest_corners.len());

            if dest_corners.is_empty() {
                println!("⚠️  DEST HAS NO INTERESTING CORNERS!");
                println!("This means dest cannot be reached from any corner.");
            } else {
                println!("Dest corners (first 10):");
                for (i, corner) in dest_corners.iter().take(10).enumerate() {
                    let corner_id = grid.get_id(corner.x, corner.y);
                    let dx = (corner.x - dest_x) as f64;
                    let dy = (corner.y - dest_y) as f64;
                    let dist = (dx*dx + dy*dy).sqrt();
                    println!("  [{}] ID {} at ({},{}) - dist {:.2}", i, corner_id, corner.x, corner.y, dist);
                }
            }

            // Check expected waypoints
            println!("\n--- Expected Waypoints Validation ---");
            let expected = vec![6141, 6307, 6306, 6389];
            for &wp in &expected {
                let (wp_x, wp_y) = grid.get_coords(wp);
                let cell_val = grid.get_cell(wp_x, wp_y);
                let is_corner = all_corners.iter().any(|c| grid.get_id(c.x, c.y) == wp);

                println!("WP {} at ({},{}): cell_val={}, is_corner={}",
                         wp, wp_x, wp_y, cell_val, is_corner);
            }
        }
    }
}
