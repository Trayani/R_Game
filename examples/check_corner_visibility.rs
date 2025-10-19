use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== CHECKING 4310 AND 4056 VISIBILITY FROM 1211 =====\n");

    let (start_x, start_y) = grid.get_coords(1211);
    println!("Start: 1211 at ({}, {})\n", start_x, start_y);

    // Get all corners in the grid
    let all_corners = detect_all_corners(&grid);
    println!("Total corners in grid: {}\n", all_corners.len());

    // Check if 4310 and 4056 are corners at all
    let corner_4310 = all_corners.iter().find(|c| grid.get_id(c.x, c.y) == 4310);
    let corner_4056 = all_corners.iter().find(|c| grid.get_id(c.x, c.y) == 4056);

    println!("Is 4310 a corner? {}", if corner_4310.is_some() { "YES" } else { "NO" });
    if let Some(c) = corner_4310 {
        println!("  4310 at ({}, {}) - directions: {:?}", c.x, c.y, c.directions);
    }

    println!("\nIs 4056 a corner? {}", if corner_4056.is_some() { "YES" } else { "NO" });
    if let Some(c) = corner_4056 {
        println!("  4056 at ({}, {}) - directions: {:?}", c.x, c.y, c.directions);
    }

    // Raycast from 1211
    println!("\n===== RAYCASTING FROM 1211 =====\n");
    let visible_from_1211 = raycast(&grid, start_x, start_y, false, false);
    println!("Total visible cells from 1211: {}", visible_from_1211.len());

    // Check if 4310 is in visible cells
    let is_4310_visible = visible_from_1211.contains(&4310);
    println!("\nIs 4310 (cell) visible from 1211? {}", if is_4310_visible { "YES ✓" } else { "NO ✗" });

    // Check if 4056 is in visible cells
    let is_4056_visible = visible_from_1211.contains(&4056);
    println!("Is 4056 (cell) visible from 1211? {}", if is_4056_visible { "YES ✓" } else { "NO ✗" });

    // Get ALL visible corners (not just interesting)
    println!("\n===== ALL VISIBLE CORNERS FROM 1211 =====\n");
    let all_visible_corners: Vec<_> = all_corners.iter()
        .filter(|corner| {
            let corner_id = grid.get_id(corner.x, corner.y);
            visible_from_1211.contains(&corner_id)
        })
        .collect();

    println!("Total visible corners: {}", all_visible_corners.len());

    let is_4310_in_visible_corners = all_visible_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    let is_4056_in_visible_corners = all_visible_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4056);

    println!("\nIs 4310 in ALL visible corners? {}", if is_4310_in_visible_corners { "YES ✓" } else { "NO ✗" });
    println!("Is 4056 in ALL visible corners? {}", if is_4056_in_visible_corners { "YES ✓" } else { "NO ✗" });

    // Get interesting corners
    println!("\n===== INTERESTING CORNERS FROM 1211 =====\n");
    let interesting_corners = filter_interesting_corners(
        &all_corners, &visible_from_1211, &grid, start_x, start_y, false
    );

    println!("Total interesting corners: {}", interesting_corners.len());

    let is_4310_interesting = interesting_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    let is_4056_interesting = interesting_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4056);

    println!("\nIs 4310 in INTERESTING corners? {}", if is_4310_interesting { "YES ✓" } else { "NO ✗" });
    println!("Is 4056 in INTERESTING corners? {}", if is_4056_interesting { "YES ✓" } else { "NO ✗" });

    // List all interesting corners for debugging
    println!("\n===== ALL INTERESTING CORNERS FROM 1211 =====\n");
    for (i, corner) in interesting_corners.iter().enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        let dx = corner.x - start_x;
        let dy = corner.y - start_y;
        let dist = ((dx * dx + dy * dy) as f64).sqrt();
        println!("  [{}] ID {} at ({}, {}) - dist {:.2} - directions {:?}",
                 i, id, corner.x, corner.y, dist, corner.directions);
    }

    // Summary
    println!("\n===== SUMMARY =====\n");
    println!("4310:");
    println!("  - Is a corner? {}", if corner_4310.is_some() { "YES" } else { "NO" });
    println!("  - Visible as cell? {}", if is_4310_visible { "YES" } else { "NO" });
    println!("  - In all visible corners? {}", if is_4310_in_visible_corners { "YES" } else { "NO" });
    println!("  - In interesting corners? {}", if is_4310_interesting { "YES" } else { "NO" });

    println!("\n4056:");
    println!("  - Is a corner? {}", if corner_4056.is_some() { "YES" } else { "NO" });
    println!("  - Visible as cell? {}", if is_4056_visible { "YES" } else { "NO" });
    println!("  - In all visible corners? {}", if is_4056_in_visible_corners { "YES" } else { "NO" });
    println!("  - In interesting corners? {}", if is_4056_interesting { "YES" } else { "NO" });
}
