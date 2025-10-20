use rustgame3::{Grid, raycast};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let start = 6637;  // (80, 79)
    let corner_6636 = 6636;  // (79, 79)
    let corner_6639 = 6639;  // (82, 79)

    let (start_x, start_y) = grid.get_coords(start);

    println!("=== From 6637 WITH messyY=true ===");
    let visible_messy = raycast(&grid, start_x, start_y, false, true);
    println!("Visible cells: {}", visible_messy.len());
    println!("Can see 6636? {}", visible_messy.contains(&corner_6636));
    println!("Can see 6639? {}", visible_messy.contains(&corner_6639));

    println!("\n=== From 6637 WITHOUT messyY (clean) ===");
    let visible_clean = raycast(&grid, start_x, start_y, false, false);
    println!("Visible cells: {}", visible_clean.len());
    println!("Can see 6636? {}", visible_clean.contains(&corner_6636));
    println!("Can see 6639? {}", visible_clean.contains(&corner_6639));
}
