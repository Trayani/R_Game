use rustgame3::{Grid, pathfinding::find_path_by_id, raycast};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let mut grid = Grid::with_blocked(104, 83, &blocked);

    // Add blocks like the test does
    grid.set_cell(4998 % 104, 4998 / 104, 1);  // setBlocked
    grid.set_cell(4999 % 104, 4999 / 104, 1);

    println!("===== TEST 019 ANALYSIS: 4829 → 4750 (with blocks at 4998, 4999) =====\n");

    // C# expected path
    let c_sharp_path = vec![4829, 4917, 5083, 5247, 5245, 4750];
    println!("C# expected path: {:?}", c_sharp_path);

    // Calculate C# path distance
    let mut c_sharp_dist = 0.0;
    for i in 1..c_sharp_path.len() {
        let (x1, y1) = grid.get_coords(c_sharp_path[i-1]);
        let (x2, y2) = grid.get_coords(c_sharp_path[i]);
        let dx = (x2 - x1) as f64;
        let dy = (y2 - y1) as f64;
        let dist = (dx*dx + dy*dy).sqrt();
        c_sharp_dist += dist;
        println!("  {} ({},{}) → {} ({},{}): {:.2}",
                 c_sharp_path[i-1], x1, y1, c_sharp_path[i], x2, y2, dist);
    }
    println!("C# total distance: {:.2}\n", c_sharp_dist);

    // Rust path
    let result = find_path_by_id(&grid, 4829, 4750, false, false);
    match result {
        Some((rust_path, rust_dist)) => {
            println!("Rust found path: {:?}", rust_path);
            println!("Rust distance: {:.2}\n", rust_dist);

            // Calculate segment distances
            for i in 1..rust_path.len() {
                let (x1, y1) = grid.get_coords(rust_path[i-1]);
                let (x2, y2) = grid.get_coords(rust_path[i]);
                let dx = (x2 - x1) as f64;
                let dy = (y2 - y1) as f64;
                let dist = (dx*dx + dy*dy).sqrt();
                println!("  {} ({},{}) → {} ({},{}): {:.2}",
                         rust_path[i-1], x1, y1, rust_path[i], x2, y2, dist);
            }

            println!("\n===== COMPARISON =====");
            println!("C# distance:   {:.2}", c_sharp_dist);
            println!("Rust distance: {:.2}", rust_dist);
            if rust_dist < c_sharp_dist {
                println!("✓ Rust found SHORTER path by {:.2}", c_sharp_dist - rust_dist);
            } else if rust_dist > c_sharp_dist {
                println!("✗ Rust found LONGER path by {:.2}", rust_dist - c_sharp_dist);
            } else {
                println!("= Same distance");
            }

            // Check visibility between waypoints
            println!("\n===== VISIBILITY CHECK =====");
            for i in 1..rust_path.len() {
                let from = rust_path[i-1];
                let (from_x, from_y) = grid.get_coords(from);
                let to = rust_path[i];
                let visible = raycast(&grid, from_x, from_y, false, false);
                let is_visible = visible.contains(&to);
                let status = if is_visible { "✓" } else { "✗" };
                println!("{} {} → {}: {}", status, from, to,
                         if is_visible { "VISIBLE" } else { "NOT VISIBLE" });
            }
        }
        None => {
            println!("Rust found NO PATH");
        }
    }

    // Check if the missing waypoints (4917, 5083) are actually needed
    println!("\n===== CHECKING SKIPPED WAYPOINTS =====");
    println!("C# path has 4917 and 5083, Rust skipped them.");
    println!("Can 4829 see 5247 directly?");
    let (start_x, start_y) = grid.get_coords(4829);
    let visible_from_start = raycast(&grid, start_x, start_y, false, false);
    let can_see_5247 = visible_from_start.contains(&5247);
    println!("  4829 can see 5247: {}", if can_see_5247 { "YES ✓" } else { "NO ✗" });

    if can_see_5247 {
        let (x1, y1) = grid.get_coords(4829);
        let (x2, y2) = grid.get_coords(5247);
        let dx = (x2 - x1) as f64;
        let dy = (y2 - y1) as f64;
        let direct_dist = (dx*dx + dy*dy).sqrt();
        println!("  Direct distance 4829 → 5247: {:.2}", direct_dist);

        let indirect_dist = {
            let (x1, y1) = grid.get_coords(4829);
            let (x2, y2) = grid.get_coords(4917);
            let dx = (x2 - x1) as f64;
            let dy = (y2 - y1) as f64;
            let d1 = (dx*dx + dy*dy).sqrt();

            let (x1, y1) = grid.get_coords(4917);
            let (x2, y2) = grid.get_coords(5083);
            let dx = (x2 - x1) as f64;
            let dy = (y2 - y1) as f64;
            let d2 = (dx*dx + dy*dy).sqrt();

            let (x1, y1) = grid.get_coords(5083);
            let (x2, y2) = grid.get_coords(5247);
            let dx = (x2 - x1) as f64;
            let dy = (y2 - y1) as f64;
            let d3 = (dx*dx + dy*dy).sqrt();

            d1 + d2 + d3
        };
        println!("  Indirect distance 4829 → 4917 → 5083 → 5247: {:.2}", indirect_dist);
        println!("  Savings from direct path: {:.2}", indirect_dist - direct_dist);
    }
}
