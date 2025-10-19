// Test why raycast returns 0 cells for position 6139 (80, 73)

use rustgame3::{Grid, raycast};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let test_positions = vec![
        (6139, "Destination from messy_001"),
        (6388, "Start from messy_001"),
        (6140, "Expected corner near dest"),
        (6141, "Expected corner near dest"),
        (6389, "Expected corner near start"),
    ];

    println!("===== RAYCASTING TEST FOR MESSY_001 POSITIONS =====\n");
    println!("Grid: 83 cols × 104 rows\n");

    for (id, description) in test_positions {
        let (x, y) = grid.get_coords(id);
        let cell_value = grid.get_cell(x, y);

        println!("--- Position {} at ({}, {}) - {} ---", id, x, y, description);
        println!("Cell value: {}", cell_value);

        // Test with all messy flag combinations
        let test_cases = vec![
            (false, false, "clean"),
            (true, false, "messyX"),
            (false, true, "messyY"),
            (true, true, "both messy"),
        ];

        for (messy_x, messy_y, label) in test_cases {
            let visible = raycast(&grid, x, y, messy_x, messy_y);
            print!("  {:<15}: {} visible cells", label, visible.len());

            if visible.len() > 0 {
                println!(" (first 5: {:?})", visible.iter().take(5).collect::<Vec<_>>());
            } else {
                println!(" ⚠️ EMPTY!");
            }
        }

        // Check surrounding cells
        println!("  Surrounding cells:");
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < grid.cols && ny >= 0 && ny < grid.rows {
                    let nid = grid.get_id(nx, ny);
                    let nval = grid.get_cell(nx, ny);
                    print!("    ({:2},{:2})={:4} val={} ", nx, ny, nid, nval);
                    if nval != 0 {
                        print!("BLOCKED");
                    }
                    println!();
                }
            }
        }

        println!();
    }

    // Now test C#'s expected behavior
    println!("\n===== C# EXPECTED BEHAVIOR =====");
    println!("From C# trace:");
    println!("  Start 6388 at (80,76) with messyY=true:");
    println!("    - Should see 1+ corners");
    println!("    - C# discovers corner 6389");
    println!();
    println!("  Dest 6139 at (80,73) with NO messy flags:");
    println!("    - Should see 2+ corners");
    println!("    - C# discovers corners 6140 and 6141");
    println!();

    let start_visible = raycast(&grid, 80, 76, false, true);
    let dest_visible = raycast(&grid, 80, 73, false, false);

    println!("Rust Results:");
    println!("  Start 6388 (80,76) with messyY=true: {} visible cells", start_visible.len());
    println!("  Dest  6139 (80,73) clean: {} visible cells", dest_visible.len());

    if dest_visible.len() == 0 {
        println!("\n⚠️ CONFIRMED BUG: Dest raycast returns 0 cells!");
        println!("This prevents discovery of finished corners.");
    }

    if start_visible.len() == 0 {
        println!("\n⚠️ CONFIRMED BUG: Start raycast with messyY returns 0 cells!");
        println!("This prevents discovery of interesting corners from start.");
    }
}
