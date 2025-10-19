// Analyze test2x_messy_022: 2762 -> 2768

use rustgame3::Grid;
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let start = 2762;
    let dest = 2768;

    let (start_x, start_y) = grid.get_coords(start);
    let (dest_x, dest_y) = grid.get_coords(dest);

    println!("===== TEST messy_022: {} -> {} =====", start, dest);
    println!("Start: {} at ({}, {})", start, start_x, start_y);
    println!("Dest:  {} at ({}, {})", dest, dest_x, dest_y);
    println!();

    if start_y == dest_y {
        println!("✓ Same row (Y={})!", start_y);
        println!("This triggers same-line special case.");
    } else {
        println!("✗ Different rows: start Y={}, dest Y={}", start_y, dest_y);
    }
    println!();

    // Check expected waypoints
    let expected_waypoints = vec![2851, 2845];
    println!("Expected waypoints: {:?}", expected_waypoints);
    for &wp in &expected_waypoints {
        let (wp_x, wp_y) = grid.get_coords(wp);
        println!("  Waypoint {} at ({}, {})", wp, wp_x, wp_y);
    }
    println!();

    // Check C# test source
    println!("===== C# Test Source =====");
    println!("Line 269 in ProtoTests.cs:");
    println!("  test2(2762, 2768, 1, false, 2851, 2845);");
    println!();
    println!("test2 => test3(start, dest, size, reverse=false, messyY=true, messyX=false, exPath)");
    println!();
    println!("So this test has:");
    println!("  messyX=false");
    println!("  messyY=true");
    println!();

    println!("===== Analysis =====");
    if start_y == dest_y {
        println!("Start and dest ARE on same row.");
        println!("But C# expects waypoints [2851, 2845] - NOT same-line path!");
        println!();
        println!("This means C#'s same-line special case did NOT trigger.");
        println!("Why not?");
        println!();
        println!("Possible reasons:");
        println!("1. C# checks if start and dest are on same LINE, not just same row");
        println!("2. 'Same line' means same horizontal line SEGMENT (connected walkable cells)");
        println!("3. If there's an obstacle between them, they're on different line segments");
        println!("4. C#'s onLine check: l == l2 (line objects must be equal)");
    }
}
