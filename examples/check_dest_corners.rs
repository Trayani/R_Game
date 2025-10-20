use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};

fn main() {
    let mut grid = Grid::new(12, 10);

    // Create obstacle
    for y in 3..7 {
        grid.set_cell(6, y, 1);
    }

    let dest_x = 10;
    let dest_y = 5;

    println!("Checking corners visible from destination ({}, {})", dest_x, dest_y);
    println!();

    let dest_visible = raycast(&grid, dest_x, dest_y, false, false);
    println!("Visible cells from dest: {}", dest_visible.len());

    let all_corners = detect_all_corners(&grid);
    println!("Total corners in grid: {}", all_corners.len());

    let dest_corners = filter_interesting_corners(
        &all_corners,
        &dest_visible,
        &grid,
        dest_x,
        dest_y,
        false,
    );

    println!("Interesting corners from dest: {}", dest_corners.len());
    for (i, corner) in dest_corners.iter().enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        println!("  [{}] Corner at ({},{}) = ID {}", i, corner.x, corner.y, id);
    }

    // Check if dest itself is a corner
    let dest_is_corner = all_corners.iter().any(|c| c.x == dest_x && c.y == dest_y);
    println!("\nIs dest itself a corner? {}", dest_is_corner);
}
