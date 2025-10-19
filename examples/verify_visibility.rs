use rustgame3::{Grid, raycast, corners::{detect_all_corners, filter_interesting_corners}, pathfinding::find_path_by_id};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== VERIFYING RUST PATH FOR TEST 002: 1211 -> 4396 =====\n");

    // Run actual pathfinding to get Rust path
    let (rust_path, dist) = find_path_by_id(&grid, 1211, 4396, false, false)
        .expect("Failed to find path");

    println!("Distance: {:.2}", dist);

    println!("Rust path: {:?}", rust_path);
    println!("\nPath coordinates:");
    for &id in &rust_path {
        let (x, y) = grid.get_coords(id);
        println!("  {} at ({}, {})", id, x, y);
    }

    println!("\n===== CHECKING VISIBILITY BETWEEN CONSECUTIVE WAYPOINTS =====\n");

    for i in 0..rust_path.len()-1 {
        let from_id = rust_path[i];
        let to_id = rust_path[i+1];
        let (from_x, from_y) = grid.get_coords(from_id);
        let (to_x, to_y) = grid.get_coords(to_id);

        println!("Segment {}: {} ({},{}) -> {} ({},{})",
            i+1, from_id, from_x, from_y, to_id, to_x, to_y);

        // Check if 'to' is visible from 'from'
        let visible_from_start = raycast(&grid, from_x, from_y, false, false);
        let is_visible = visible_from_start.contains(&to_id);

        println!("  Is {} visible from {}? {}", to_id, from_id, is_visible);

        if !is_visible {
            println!("  ❌ PROBLEM: Next waypoint is NOT visible!");
            println!("  Visible cells from {} ({}): {} cells",
                from_id, from_id, visible_from_start.len());

            // Check if it's in interesting corners
            let all_corners = detect_all_corners(&grid);
            let interesting = filter_interesting_corners(&all_corners, &visible_from_start, &grid, from_x, from_y, false);

            let to_is_corner = all_corners.iter().any(|c| grid.get_id(c.x, c.y) == to_id);
            let to_is_interesting = interesting.iter().any(|c| grid.get_id(c.x, c.y) == to_id);

            println!("  Is {} a corner? {}", to_id, to_is_corner);
            println!("  Is {} in interesting corners from {}? {}", to_id, from_id, to_is_interesting);
        } else {
            println!("  ✓ Visibility OK");
        }
        println!();
    }

    println!("\n===== CHECKING C# PATH: [1211, 4310, 4056, 4396] =====\n");

    let csharp_path = vec![1211, 4310, 4056, 4396];
    println!("C# path: {:?}", csharp_path);
    println!("\nPath coordinates:");
    for &id in &csharp_path {
        let (x, y) = grid.get_coords(id);
        println!("  {} at ({}, {})", id, x, y);
    }

    println!("\n===== CHECKING VISIBILITY FOR C# PATH =====\n");

    for i in 0..csharp_path.len()-1 {
        let from_id = csharp_path[i];
        let to_id = csharp_path[i+1];
        let (from_x, from_y) = grid.get_coords(from_id);
        let (to_x, to_y) = grid.get_coords(to_id);

        println!("Segment {}: {} ({},{}) -> {} ({},{})",
            i+1, from_id, from_x, from_y, to_id, to_x, to_y);

        // Check if 'to' is visible from 'from'
        let visible_from_start = raycast(&grid, from_x, from_y, false, false);
        let is_visible = visible_from_start.contains(&to_id);

        println!("  Is {} visible from {}? {}", to_id, from_id, is_visible);

        if is_visible {
            println!("  ✓ Visibility OK");
        } else {
            println!("  ❌ Next waypoint is NOT visible!");
        }
        println!();
    }

    println!("\n===== DISTANCE COMPARISON =====\n");

    fn calc_path_distance(grid: &Grid, path: &[i32]) -> f64 {
        let mut total = 0.0;
        for i in 1..path.len() {
            let (x1, y1) = grid.get_coords(path[i-1]);
            let (x2, y2) = grid.get_coords(path[i]);
            let dist = (((x2-x1).pow(2) + (y2-y1).pow(2)) as f64).sqrt();
            total += dist;
        }
        total
    }

    let rust_geom_dist = calc_path_distance(&grid, &rust_path);
    let csharp_geom_dist = calc_path_distance(&grid, &csharp_path);

    println!("Rust path geometric distance: {:.2}", rust_geom_dist);
    println!("C# path geometric distance: {:.2}", csharp_geom_dist);
    println!("Difference: {:.2}", rust_geom_dist - csharp_geom_dist);
}
