use rustgame3::{Grid, raycast, corners::{detect_all_corners, filter_interesting_corners}};
use std::fs;

#[test]
fn debug_corner_comparison_with_csharp() {
    // Load grid
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    // C# uses Grid.init(rows, cols) = Grid.init(104, 83)
    // Rust Grid::with_blocked(rows, cols)
    let grid = Grid::with_blocked(104, 83, &blocked);

    let start_id = 4396;
    let dest_id = 1211;
    let (start_x, start_y) = grid.get_coords(start_id);
    let (dest_x, dest_y) = grid.get_coords(dest_id);

    println!("\n===== RUST CORNER DISCOVERY FROM START {} =====", start_id);
    println!("Start: {} = ({}, {})", start_id, start_x, start_y);

    // Get interesting corners from start
    let visible_from_start = raycast(&grid, start_x, start_y, false, false);
    let all_corners = detect_all_corners(&grid);
    let start_corners = filter_interesting_corners(&all_corners, &visible_from_start, &grid, start_x, start_y, false);

    println!("\nTotal interesting corners from start: {}", start_corners.len());
    println!("\nAll corners from start:");
    for (i, c) in start_corners.iter().enumerate() {
        let id = grid.get_id(c.x, c.y);
        let dist = ((c.x - start_x).pow(2) + (c.y - start_y).pow(2)) as f64;
        let dist = dist.sqrt();
        println!("  [{:2}] ID={:4} ({:3},{:3})  dist={:6.2}  dirs={:?}", i, id, c.x, c.y, dist, c.directions);
    }

    println!("\n\n===== RUST CORNER DISCOVERY FROM DEST {} =====", dest_id);
    println!("Dest: {} = ({}, {})", dest_id, dest_x, dest_y);

    // Get interesting corners from dest
    let visible_from_dest = raycast(&grid, dest_x, dest_y, false, false);
    let dest_corners = filter_interesting_corners(&all_corners, &visible_from_dest, &grid, dest_x, dest_y, false);

    println!("\nTotal interesting corners from dest: {}", dest_corners.len());
    println!("\nAll corners from dest:");
    for (i, c) in dest_corners.iter().enumerate() {
        let id = grid.get_id(c.x, c.y);
        let dist = ((c.x - dest_x).pow(2) + (c.y - dest_y).pow(2)) as f64;
        let dist = dist.sqrt();
        println!("  [{:2}] ID={:4} ({:3},{:3})  dist={:6.2}  dirs={:?}", i, id, c.x, c.y, dist, c.directions);
    }

    println!("\n\n===== CHECK EXPECTED WAYPOINTS =====");
    println!("Expected waypoints: [4056, 4310]");

    let wp1_id = 4056;
    let wp2_id = 4310;
    let (wp1_x, wp1_y) = grid.get_coords(wp1_id);
    let (wp2_x, wp2_y) = grid.get_coords(wp2_id);
    println!("WP1: {} = ({}, {})", wp1_id, wp1_x, wp1_y);
    println!("WP2: {} = ({}, {})", wp2_id, wp2_x, wp2_y);

    let wp1_in_start = start_corners.iter().any(|c| c.x == wp1_x && c.y == wp1_y);
    let wp2_in_start = start_corners.iter().any(|c| c.x == wp2_x && c.y == wp2_y);
    let wp1_in_dest = dest_corners.iter().any(|c| c.x == wp1_x && c.y == wp1_y);
    let wp2_in_dest = dest_corners.iter().any(|c| c.x == wp2_x && c.y == wp2_y);

    println!("\nWP1 ({}) in start corners? {}", wp1_id, wp1_in_start);
    println!("WP1 ({}) in dest corners? {}", wp1_id, wp1_in_dest);
    println!("WP2 ({}) in start corners? {}", wp2_id, wp2_in_start);
    println!("WP2 ({}) in dest corners? {}", wp2_id, wp2_in_dest);

    println!("\n===== C# EXPECTED RESULTS =====");
    println!("C# found 19 corners from start, 16 from dest");
    println!("WP1 (4056) in C# start corners? False");
    println!("WP1 (4056) in C# dest corners? True");
    println!("WP2 (4310) in C# start corners? True");
    println!("WP2 (4310) in C# dest corners? False");
}
