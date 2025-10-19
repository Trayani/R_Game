use rustgame3::Grid;
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let mut grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== VERIFYING GRID BLOCKING FOR TEST 019 =====\n");

    // Check cells 4998 and 4999 BEFORE blocking
    let (x1, y1) = grid.get_coords(4998);
    let (x2, y2) = grid.get_coords(4999);

    println!("BEFORE blocking:");
    println!("  Cell 4998 at ({},{}): {}", x1, y1, grid.get_cell(x1, y1));
    println!("  Cell 4999 at ({},{}): {}", x2, y2, grid.get_cell(x2, y2));

    // Block them (WRONG way - using %)
    println!("\nAttempting to block using modulo operator...");
    println!("  4998 % 104 = {}, 4998 / 104 = {}", 4998 % 104, 4998 / 104);
    println!("  4999 % 104 = {}, 4999 / 104 = {}", 4999 % 104, 4999 / 104);

    grid.set_cell(4998 % 104, 4998 / 104, 1);
    grid.set_cell(4999 % 104, 4999 / 104, 1);

    println!("\nAFTER blocking with modulo:");
    println!("  Cell 4998 at ({},{}): {}", x1, y1, grid.get_cell(x1, y1));
    println!("  Cell 4999 at ({},{}): {}", x2, y2, grid.get_cell(x2, y2));

    // Check what we actually blocked
    let actual_blocked_1 = grid.get_id(4998 % 104, 4998 / 104);
    let actual_blocked_2 = grid.get_id(4999 % 104, 4999 / 104);
    println!("\nWhat we ACTUALLY blocked:");
    println!("  Blocked cell at ({},{}): ID = {}", 4998 % 104, 4998 / 104, actual_blocked_1);
    println!("  Blocked cell at ({},{}): ID = {}", 4999 % 104, 4999 / 104, actual_blocked_2);

    // The CORRECT way using get_coords
    println!("\n\n===== CORRECT WAY: Using get_coords =====");
    let mut grid2 = Grid::with_blocked(104, 83, &blocked);

    println!("\nBEFORE blocking:");
    println!("  Cell 4998 at ({},{}): {}", x1, y1, grid2.get_cell(x1, y1));
    println!("  Cell 4999 at ({},{}): {}", x2, y2, grid2.get_cell(x2, y2));

    // Use get_coords to get the correct x,y
    let (x1, y1) = grid2.get_coords(4998);
    let (x2, y2) = grid2.get_coords(4999);

    grid2.set_cell(x1, y1, 1);
    grid2.set_cell(x2, y2, 1);

    println!("\nAFTER blocking with get_coords:");
    println!("  Cell 4998 at ({},{}): {}", x1, y1, grid2.get_cell(x1, y1));
    println!("  Cell 4999 at ({},{}): {}", x2, y2, grid2.get_cell(x2, y2));

    // Verify
    println!("\n===== VERIFICATION =====");
    println!("Grid dimensions: {} cols × {} rows", grid2.cols, grid2.rows);
    println!("Total cells: {}", grid2.cols * grid2.rows);
    println!("\nID formula should be: ID = x + y * cols");
    println!("For 4998: x={}, y={} → ID = {} + {} * {} = {}",
             x1, y1, x1, y1, grid2.cols, x1 + y1 * grid2.cols);
    println!("For 4999: x={}, y={} → ID = {} + {} * {} = {}",
             x2, y2, x2, y2, grid2.cols, x2 + y2 * grid2.cols);
}
