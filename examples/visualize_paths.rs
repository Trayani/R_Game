use rustgame3::{Grid, pathfinding::find_path_by_id};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== RUST PATHFINDING RESULTS =====\n");

    visualize_path(&grid, 1211, 4396, "Test 002");
    visualize_path(&grid, 875, 4396, "Test 004");
    visualize_path(&grid, 4396, 875, "Test 005");

    // Test 019 with blocks
    let mut grid_blocked = grid.clone();
    grid_blocked.set_cell(4998 % 83, 4998 / 83, 1);
    grid_blocked.set_cell(4999 % 83, 4999 / 83, 1);
    visualize_path(&grid_blocked, 4829, 4750, "Test 019 (with blocks)");
}

fn visualize_path(grid: &Grid, start: i32, dest: i32, test_name: &str) {
    let (start_x, start_y) = grid.get_coords(start);
    let (dest_x, dest_y) = grid.get_coords(dest);

    println!("\n===== {} =====", test_name);
    println!("Start: {} at ({}, {})", start, start_x, start_y);
    println!("Dest:  {} at ({}, {})", dest, dest_x, dest_y);

    match find_path_by_id(grid, start, dest, false, false) {
        Some((path, dist)) => {
            println!("Distance: {:.2}", dist);
            println!("\nRust path: {:?}", path);
            println!("\nFull path with coordinates:");
            for (i, &id) in path.iter().enumerate() {
                let (x, y) = grid.get_coords(id);
                let label = if i == 0 {
                    "START"
                } else if i == path.len() - 1 {
                    " DEST"
                } else {
                    "   WP"
                };
                println!("  {}: {:4} at ({:3}, {:2})", label, id, x, y);
            }

            println!("\nSegment distances:");
            let mut total_geom = 0.0;
            for i in 1..path.len() {
                let (x1, y1) = grid.get_coords(path[i-1]);
                let (x2, y2) = grid.get_coords(path[i]);
                let dist = (((x2-x1).pow(2) + (y2-y1).pow(2)) as f64).sqrt();
                total_geom += dist;
                println!("  {:4}({:3},{:2}) -> {:4}({:3},{:2}): {:6.2}",
                    path[i-1], x1, y1, path[i], x2, y2, dist);
            }
            println!("Total geometric distance: {:.2}", total_geom);
        }
        None => {
            println!("No path found!");
        }
    }
}
