use rustgame3::{Grid, raycast};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== ANALYZING TEST 002: 1211 -> 4396 =====\n");

    // C# path: 1211 -> 4310 -> 4056 -> 4396
    let csharp_path = vec![1211, 4310, 4056, 4396];

    println!("C# path: {:?}", csharp_path);
    println!("\nCoordinates:");
    for &id in &csharp_path {
        let (x, y) = grid.get_coords(id);
        println!("  {} at ({}, {})", id, x, y);
    }

    // Calculate distances
    println!("\n===== DISTANCE CALCULATION =====");
    let mut total_dist = 0.0;
    for i in 1..csharp_path.len() {
        let (x1, y1) = grid.get_coords(csharp_path[i-1]);
        let (x2, y2) = grid.get_coords(csharp_path[i]);
        let dx = (x2 - x1) as f64;
        let dy = (y2 - y1) as f64;
        let dist = (dx * dx + dy * dy).sqrt();
        total_dist += dist;
        println!("  {} ({},{}) -> {} ({},{}): {:.2}",
                 csharp_path[i-1], x1, y1, csharp_path[i], x2, y2, dist);
    }
    println!("  TOTAL: {:.2}", total_dist);

    // Check visibility between waypoints
    println!("\n===== VISIBILITY CHECK IN RUST =====");
    for i in 1..csharp_path.len() {
        let from_id = csharp_path[i-1];
        let to_id = csharp_path[i];
        let (from_x, from_y) = grid.get_coords(from_id);
        let (to_x, to_y) = grid.get_coords(to_id);

        let visible_from = raycast(&grid, from_x, from_y, false, false);
        let is_visible = visible_from.contains(&to_id);

        let status = if is_visible { "✓" } else { "❌" };
        println!("  {} Segment {}: {} ({},{}) -> {} ({},{}): {}",
                 status, i, from_id, from_x, from_y, to_id, to_x, to_y,
                 if is_visible { "VISIBLE" } else { "NOT VISIBLE" });
    }

    // Check if 4310 and 4056 are discovered as corners
    println!("\n===== CORNER DISCOVERY CHECK =====");
    use rustgame3::corners::{detect_all_corners, filter_interesting_corners};

    let all_corners = detect_all_corners(&grid);
    println!("Total corners in grid: {}", all_corners.len());

    let (start_x, start_y) = grid.get_coords(1211);
    let visible_from_start = raycast(&grid, start_x, start_y, false, false);

    // Check ALL visible corners from start
    let all_visible_from_start: Vec<_> = all_corners.iter()
        .filter(|c| {
            let id = grid.get_id(c.x, c.y);
            visible_from_start.contains(&id)
        })
        .collect();

    println!("\nALL visible corners from 1211: {}", all_visible_from_start.len());
    let has_4310_all = all_visible_from_start.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    let has_4056_all = all_visible_from_start.iter().any(|c| grid.get_id(c.x, c.y) == 4056);
    println!("  4310 in ALL visible corners: {}", has_4310_all);
    println!("  4056 in ALL visible corners: {}", has_4056_all);

    // Check interesting corners from start
    let interesting_from_start = filter_interesting_corners(
        &all_corners, &visible_from_start, &grid, start_x, start_y, false
    );

    println!("\nINTERESTING corners from 1211: {}", interesting_from_start.len());
    let has_4310_int = interesting_from_start.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    let has_4056_int = interesting_from_start.iter().any(|c| grid.get_id(c.x, c.y) == 4056);
    println!("  4310 in INTERESTING corners: {}", has_4310_int);
    println!("  4056 in INTERESTING corners: {}", has_4056_int);

    if has_4310_all || has_4056_all {
        println!("\n4310 coordinates: {:?}", grid.get_coords(4310));
        println!("4056 coordinates: {:?}", grid.get_coords(4056));
    }
}
