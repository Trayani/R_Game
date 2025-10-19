use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let mut grid = Grid::with_blocked(104, 83, &blocked);

    // Add blocks like the test does
    grid.set_cell(4998 % 104, 4998 / 104, 1);
    grid.set_cell(4999 % 104, 4999 / 104, 1);

    println!("===== DEBUGGING TEST 019: Checking corner discovery from 4829 =====\n");

    let (start_x, start_y) = grid.get_coords(4829);
    println!("Start: 4829 at ({}, {})", start_x, start_y);

    // Get interesting corners from start
    let visible_cells = raycast(&grid, start_x, start_y, false, false);
    println!("\nVisible cells from 4829: {}", visible_cells.len());

    let all_corners = detect_all_corners(&grid);
    let interesting_corners = filter_interesting_corners(
        &all_corners, &visible_cells, &grid, start_x, start_y, false
    );

    println!("Interesting corners from 4829: {}", interesting_corners.len());

    // Check if expected waypoints are in interesting corners
    let waypoints = [4917, 5083, 5247, 5245];
    println!("\nChecking expected waypoints:");
    for &wp in &waypoints {
        let is_interesting = interesting_corners.iter().any(|c| grid.get_id(c.x, c.y) == wp);
        let is_visible = visible_cells.contains(&wp);
        let (x, y) = grid.get_coords(wp);
        println!("  {} at ({},{}): visible={}, interesting={}",
                 wp, x, y,
                 if is_visible { "✓" } else { "✗" },
                 if is_interesting { "✓" } else { "✗" });
    }

    // List all interesting corners
    println!("\n===== ALL INTERESTING CORNERS FROM 4829 =====");
    for (i, corner) in interesting_corners.iter().enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        let is_expected = waypoints.contains(&id);
        let marker = if is_expected { "👉" } else { "  " };
        println!("{} [{}] ID {} at ({},{})", marker, i, id, corner.x, corner.y);
    }

    // Now check from each waypoint what corners they can see
    println!("\n===== CORNER DISCOVERY CHAIN =====");
    for &wp in &waypoints {
        let (x, y) = grid.get_coords(wp);
        let visible = raycast(&grid, x, y, false, false);
        let corners_from_wp = filter_interesting_corners(
            &all_corners, &visible, &grid, x, y, false
        );

        println!("\nFrom {} at ({},{}):", wp, x, y);
        println!("  Visible cells: {}", visible.len());
        println!("  Interesting corners: {}", corners_from_wp.len());

        // Check which of the other waypoints are interesting corners
        for &other in &waypoints {
            if other == wp { continue; }
            let is_interesting = corners_from_wp.iter().any(|c| grid.get_id(c.x, c.y) == other);
            let is_visible = visible.contains(&other);
            print!("    {} → {}: ", wp, other);
            if is_interesting {
                println!("✓ INTERESTING");
            } else if is_visible {
                println!("○ visible but not interesting");
            } else {
                println!("✗ not visible");
            }
        }

        // Check dest (4750)
        let is_dest_interesting = corners_from_wp.iter().any(|c| grid.get_id(c.x, c.y) == 4750);
        let is_dest_visible = visible.contains(&4750);
        print!("    {} → 4750 (dest): ", wp);
        if is_dest_interesting {
            println!("✓ INTERESTING");
        } else if is_dest_visible {
            println!("○ visible but not interesting");
        } else {
            println!("✗ not visible");
        }
    }
}
