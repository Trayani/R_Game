use rustgame3::{Grid, raycast};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    let corner_6636 = 6636;  // (79, 79)
    let corner_6637 = 6637;  // (80, 79)
    let dest = 6552;  // (78, 78)

    let (c636_x, c636_y) = grid.get_coords(corner_6636);
    let (c637_x, c637_y) = grid.get_coords(corner_6637);
    let (d_x, d_y) = grid.get_coords(dest);

    println!("Corner 6636: ({}, {})", c636_x, c636_y);
    println!("Corner 6637: ({}, {})", c637_x, c637_y);
    println!("Dest 6552: ({}, {})", d_x, d_y);

    let visible_from_6636 = raycast(&grid, c636_x, c636_y, false, false);
    println!("\nFrom 6636, can see dest? {}", visible_from_6636.contains(&dest));
    println!("From 6636, can see 6637? {}", visible_from_6636.contains(&corner_6637));

    let visible_from_6637 = raycast(&grid, c637_x, c637_y, false, false);
    println!("\nFrom 6637(clean), can see dest? {}", visible_from_6637.contains(&dest));
    println!("From 6637(clean), can see 6636? {}", visible_from_6637.contains(&corner_6636));
}
