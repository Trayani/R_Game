use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== FINDING BRIDGE FROM 1211 TO 4310 =====\n");

    let (start_x, start_y) = grid.get_coords(1211);
    let visible_from_start = raycast(&grid, start_x, start_y, false, false);

    let all_corners = detect_all_corners(&grid);
    let interesting_from_start = filter_interesting_corners(
        &all_corners, &visible_from_start, &grid, start_x, start_y, false
    );

    println!("Corners visible from 1211: {}", interesting_from_start.len());

    // Check which corners from 1211 can see 4310
    let (target_x, target_y) = grid.get_coords(4310);

    println!("\nChecking which corners from 1211 can see 4310...\n");

    for corner in &interesting_from_start {
        let corner_id = grid.get_id(corner.x, corner.y);
        let visible_from_corner = raycast(&grid, corner.x, corner.y, false, false);

        if visible_from_corner.contains(&4310) {
            let (cx, cy) = (corner.x, corner.y);
            let dx1 = ((cx - start_x) as f64).powi(2) + ((cy - start_y) as f64).powi(2);
            let dx2 = ((target_x - cx) as f64).powi(2) + ((target_y - cy) as f64).powi(2);
            let dist1 = dx1.sqrt();
            let dist2 = dx2.sqrt();

            println!("  Bridge corner: {} at ({}, {})", corner_id, cx, cy);
            println!("    1211 -> {} -> 4310", corner_id);
            println!("    Distance: {:.2} + {:.2} = {:.2}", dist1, dist2, dist1 + dist2);
        }
    }

    // Also check 4056 (which is in the C# path)
    println!("\n\nChecking if 4056 can see 4310...");
    let visible_from_4056 = raycast(&grid, 72, 48, false, false);
    if visible_from_4056.contains(&4310) {
        println!("  ✓ YES: 4056 can see 4310");
        let (x1, y1) = grid.get_coords(4056);
        let (x2, y2) = grid.get_coords(4310);
        let dist = (((x2 - x1) as f64).powi(2) + ((y2 - y1) as f64).powi(2)).sqrt();
        println!("    Distance 4056 -> 4310: {:.2}", dist);
    } else {
        println!("  ❌ NO: 4056 cannot see 4310");
    }
}
