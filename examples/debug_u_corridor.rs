use rustgame3::{Grid, raycast, pathfinding::find_path};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};

fn main() {
    let mut grid = Grid::new(15, 10);  // Grid::new(rows, cols)

    // Create U-shaped obstacle
    for y in 2..8 {
        grid.set_cell(5, y, 1);
        grid.set_cell(10, y, 1);
    }
    for x in 5..=10 {
        grid.set_cell(x, 2, 1);
    }

    let start_x = 2;
    let start_y = 5;
    let dest_x = 12;
    let dest_y = 5;

    println!("Grid: {} rows × {} cols", grid.rows, grid.cols);
    println!("U-shaped obstacle blocking direct path");
    println!("Start: ({}, {})", start_x, start_y);
    println!("Dest: ({}, {})", dest_x, dest_y);
    println!();

    // Visualize grid
    println!("Grid visualization:");
    print!("   ");
    for x in 0..grid.cols {
        print!("{}", x % 10);
    }
    println!();
    for y in 0..grid.rows {
        print!("{:2} ", y);
        for x in 0..grid.cols {
            if x == start_x && y == start_y {
                print!("S");
            } else if x == dest_x && y == dest_y {
                print!("D");
            } else if grid.get_cell(x, y) == 1 {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
    println!();

    // Check visibility
    println!("=== Visibility from start ===");
    let visible = raycast(&grid, start_x, start_y, false, false);
    println!("Visible cells: {}", visible.len());
    println!("Can see dest? {}", visible.contains(&grid.get_id(dest_x, dest_y)));

    // Check corners
    let all_corners = detect_all_corners(&grid);
    println!("\nTotal corners in grid: {}", all_corners.len());

    let interesting = filter_interesting_corners(&all_corners, &visible, &grid, start_x, start_y, false);
    println!("Interesting corners from start: {}", interesting.len());
    for (i, corner) in interesting.iter().take(10).enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        println!("  [{}] Corner at ({},{}) = ID {}", i, corner.x, corner.y, id);
    }

    // Check destination visibility
    println!("\n=== Visibility from destination ===");
    let dest_visible = raycast(&grid, dest_x, dest_y, false, false);
    println!("Visible cells from dest: {}", dest_visible.len());

    let dest_corners = filter_interesting_corners(&all_corners, &dest_visible, &grid, dest_x, dest_y, false);
    println!("Interesting corners from dest: {}", dest_corners.len());
    for (i, corner) in dest_corners.iter().take(10).enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        println!("  [{}] Corner at ({},{}) = ID {}", i, corner.x, corner.y, id);
    }

    // Try pathfinding
    println!("\n=== Pathfinding ===");
    let path = find_path(&grid, start_x, start_y, dest_x, dest_y, false, false);
    match path {
        Some(positions) => {
            println!("Found path with {} waypoints:", positions.len());
            for (i, pos) in positions.iter().enumerate() {
                println!("  [{}] ({}, {})", i, pos.x, pos.y);
            }
        }
        None => {
            println!("NO PATH FOUND!");
        }
    }
}
