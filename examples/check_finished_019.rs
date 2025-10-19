use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let mut grid = Grid::with_blocked(104, 83, &blocked);

    // Add blocks
    grid.set_cell(4998 % 104, 4998 / 104, 1);
    grid.set_cell(4999 % 104, 4999 / 104, 1);

    println!("===== CHECKING FINISHED CORNERS FOR TEST 019 =====\n");

    let dest_id = 4750;
    let (dest_x, dest_y) = grid.get_coords(dest_id);
    println!("Destination: {} at ({}, {})", dest_id, dest_x, dest_y);

    // Get finished corners (corners visible from dest)
    let dest_visible = raycast(&grid, dest_x, dest_y, false, false);
    let all_corners = detect_all_corners(&grid);
    let dest_corners = filter_interesting_corners(
        &all_corners, &dest_visible, &grid, dest_x, dest_y, false
    );

    println!("\nFinished corners (interesting corners from dest): {}", dest_corners.len());

    let waypoints = [4829, 4917, 5083, 5247, 5245];
    println!("\nChecking which waypoints are finished corners:");
    for &wp in &waypoints {
        let (x, y) = grid.get_coords(wp);
        let is_finished = dest_corners.iter().any(|c| grid.get_id(c.x, c.y) == wp);
        let is_visible = dest_visible.contains(&wp);
        let dist = if is_visible {
            let dx = (x - dest_x) as f64;
            let dy = (y - dest_y) as f64;
            (dx*dx + dy*dy).sqrt()
        } else {
            0.0
        };

        print!("  {} at ({},{}): ", wp, x, y);
        if is_finished {
            println!("✓ FINISHED (dist to dest: {:.2})", dist);
        } else if is_visible {
            println!("○ visible but not finished (dist: {:.2})", dist);
        } else {
            println!("✗ not visible");
        }
    }

    println!("\n===== ALL FINISHED CORNERS =====");
    for (i, corner) in dest_corners.iter().enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        let is_waypoint = waypoints.contains(&id);
        let marker = if is_waypoint { "👉" } else { "  " };
        let dx = (corner.x - dest_x) as f64;
        let dy = (corner.y - dest_y) as f64;
        let dist = (dx*dx + dy*dy).sqrt();
        println!("{} [{}] ID {} at ({},{}) - dist {:.2}", marker, i, id, corner.x, corner.y, dist);
    }
}
