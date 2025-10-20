use rustgame3::{Grid, raycast, pathfinding::find_path};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};

fn main() {
    let mut grid = Grid::new(12, 10);

    // Create obstacle
    for y in 3..7 {
        grid.set_cell(6, y, 1);
    }

    let start_x = 2;
    let start_y = 5;
    let dest_x = 10;
    let dest_y = 5;

    println!("Grid: 12x10");
    println!("Obstacle: vertical wall at X=6, Y=3..7");
    println!("Start: ({}, {}) with messyX=true", start_x, start_y);
    println!("Dest: ({}, {})", dest_x, dest_y);
    println!();

    // Visualize grid
    println!("Grid visualization ({}x{}):", 12, 10);
    print!("   ");
    for x in 0..12 {
        print!("{}", x % 10);
    }
    println!();
    for y in 0..10 {
        print!("{:2} ", y);
        for x in 0..12 {
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

    // Check visibility from messy position
    println!("=== From ({}, {}) WITH messyX=true ===", start_x, start_y);
    let visible_messy = raycast(&grid, start_x, start_y, true, false);
    println!("Visible cells: {}", visible_messy.len());
    println!("Can see dest? {}", visible_messy.contains(&grid.get_id(dest_x, dest_y)));

    // Check corners
    let all_corners = detect_all_corners(&grid);
    let interesting_messy = filter_interesting_corners(&all_corners, &visible_messy, &grid, start_x, start_y, true);
    println!("Interesting corners: {}", interesting_messy.len());
    for (i, corner) in interesting_messy.iter().take(10).enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        println!("  [{}] Corner at ({},{}) = ID {}", i, corner.x, corner.y, id);
    }
    println!();

    // Check visibility from clean position
    println!("=== From ({}, {}) WITHOUT messyX (clean) ===", start_x, start_y);
    let visible_clean = raycast(&grid, start_x, start_y, false, false);
    println!("Visible cells: {}", visible_clean.len());
    println!("Can see dest? {}", visible_clean.contains(&grid.get_id(dest_x, dest_y)));

    let interesting_clean = filter_interesting_corners(&all_corners, &visible_clean, &grid, start_x, start_y, false);
    println!("Interesting corners: {}", interesting_clean.len());
    for (i, corner) in interesting_clean.iter().take(10).enumerate() {
        let id = grid.get_id(corner.x, corner.y);
        println!("  [{}] Corner at ({},{}) = ID {}", i, corner.x, corner.y, id);
    }
    println!();

    // Try pathfinding
    println!("=== Pathfinding ===");
    let path = find_path(&grid, start_x, start_y, dest_x, dest_y, true, false);
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
