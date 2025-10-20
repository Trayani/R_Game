use rustgame3::{Grid, pathfinding::{find_path, format_path, Position}};

/// Visualize a path on a grid
fn visualize_path(grid: &Grid, path: &[Position], start: Position, dest: Position) -> String {
    let mut result = String::new();

    result.push_str(&format!("\nPath: {}\n", format_path(path)));
    result.push_str(&format!("Length: {} steps, Distance: {:.2}\n\n",
        path.len() - 1,
        calculate_total_distance(path)));

    // Create visualization
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            let pos = Position::new(x, y);
            let symbol = if pos == start {
                'S'  // Start
            } else if pos == dest {
                'D'  // Destination
            } else if path.contains(&pos) {
                '*'  // Path
            } else if grid.is_blocked(x, y) {
                '█'  // Blocked
            } else {
                '.'  // Free
            };
            result.push(symbol);
        }
        result.push('\n');
    }

    result
}

fn calculate_total_distance(path: &[Position]) -> f64 {
    let mut total = 0.0;
    for i in 1..path.len() {
        total += path[i - 1].distance(&path[i]);
    }
    total
}

#[test]
fn test_direct_line_of_sight() {
    println!("\n=== Test 1: Direct Line of Sight ===");

    let mut grid = Grid::new(10, 10);
    let start = Position::new(2, 2);
    let dest = Position::new(7, 2);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist");
    let path = path.unwrap();

    // Should be direct path (just start and dest)
    assert_eq!(path.len(), 2, "Direct line of sight should have 2 points");
    assert_eq!(path[0], start);
    assert_eq!(path[1], dest);

    println!("{}", visualize_path(&grid, &path, start, dest));
}

#[test]
fn test_simple_obstacle() {
    println!("\n=== Test 2: Simple Obstacle ===");

    let mut grid = Grid::new(10, 10);

    // Create vertical wall
    for y in 2..8 {
        grid.set_cell(5, y, 1);
    }

    let start = Position::new(2, 4);
    let dest = Position::new(8, 4);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist around obstacle");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));

    // Path should go around the wall
    assert!(path.len() > 2, "Path should have intermediate points");
}

#[test]
fn test_u_shaped_corridor() {
    println!("\n=== Test 3: U-Shaped Corridor ===");

    let mut grid = Grid::new(15, 10);

    // Create U-shaped obstacle
    for y in 2..8 {
        grid.set_cell(5, y, 1);
        grid.set_cell(10, y, 1);
    }
    for x in 5..=10 {
        grid.set_cell(x, 2, 1);
    }

    let start = Position::new(2, 5);
    let dest = Position::new(12, 5);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist through U-corridor");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));

    // Path should navigate around the U (at least 2 intermediate points)
    assert!(path.len() >= 3, "Path should have waypoints");
}

#[test]
fn test_no_path_blocked() {
    println!("\n=== Test 4: No Path (Fully Blocked) ===");

    let mut grid = Grid::new(10, 10);

    // Create complete vertical wall
    for y in 0..grid.rows {
        grid.set_cell(5, y, 1);
    }

    let start = Position::new(2, 5);
    let dest = Position::new(8, 5);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_none(), "No path should exist");

    println!("\nNo path found (as expected - completely blocked)");
}

#[test]
fn test_maze_navigation() {
    println!("\n=== Test 5: Maze Navigation ===");

    let mut grid = Grid::new(15, 15);

    // Create maze-like structure
    for x in 3..12 {
        grid.set_cell(x, 3, 1);
        grid.set_cell(x, 11, 1);
    }
    for y in 3..12 {
        grid.set_cell(3, y, 1);
        grid.set_cell(11, y, 1);
    }

    // Internal walls
    for x in 5..10 {
        grid.set_cell(x, 6, 1);
    }
    for y in 7..11 {
        grid.set_cell(7, y, 1);
    }

    // Openings
    grid.set_cell(3, 5, 0);   // Left entrance
    grid.set_cell(11, 9, 0);  // Right exit
    grid.set_cell(6, 6, 0);   // Internal passage

    let start = Position::new(1, 5);
    let dest = Position::new(13, 9);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist through maze");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));
}

#[test]
fn test_corner_to_corner() {
    println!("\n=== Test 6: Corner to Corner ===");

    let mut grid = Grid::new(12, 12);

    // Create room with obstacles
    for x in 4..8 {
        for y in 4..8 {
            grid.set_cell(x, y, 1);
        }
    }

    let start = Position::new(2, 2);
    let dest = Position::new(10, 10);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));
}

#[test]
fn test_messy_x_pathfinding() {
    println!("\n=== Test 7: Messy X Observer ===");

    let mut grid = Grid::new(10, 12);  // Grid::new(rows, cols)

    // Create obstacle
    for y in 3..7 {
        grid.set_cell(6, y, 1);
    }

    let start = Position::new(2, 5);
    let dest = Position::new(10, 5);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, true, false);

    assert!(path.is_some(), "Path should exist with messy X");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));
    println!("Note: Path computed with Messy X (2-cell horizontal observer)");
}

#[test]
fn test_deterministic_equal_paths() {
    println!("\n=== Test 8: Deterministic Equal Distance Paths ===");

    let mut grid = Grid::new(10, 10);

    // Create symmetrical obstacle
    grid.set_cell(5, 4, 1);
    grid.set_cell(5, 5, 1);
    grid.set_cell(5, 6, 1);

    let start = Position::new(3, 5);
    let dest = Position::new(7, 5);

    // Run pathfinding multiple times - should get same result
    let path1 = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);
    let path2 = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);
    let path3 = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert_eq!(path1, path2, "Paths should be identical (deterministic)");
    assert_eq!(path2, path3, "Paths should be identical (deterministic)");

    if let Some(path) = path1 {
        println!("{}", visualize_path(&grid, &path, start, dest));
        println!("✓ Deterministic: Same path generated 3 times");
    }
}

#[test]
fn test_complex_visibility() {
    println!("\n=== Test 9: Complex Visibility Scenario ===");

    let mut grid = Grid::new(20, 20);

    // Create complex obstacle pattern
    for x in 5..15 {
        grid.set_cell(x, 8, 1);
    }
    for y in 5..15 {
        grid.set_cell(8, y, 1);
    }

    // Create openings
    grid.set_cell(8, 10, 0);
    grid.set_cell(10, 8, 0);

    let start = Position::new(3, 3);
    let dest = Position::new(16, 16);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist in complex scenario");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));
}

#[test]
fn test_diagonal_path() {
    println!("\n=== Test 10: Diagonal Path ===");

    let mut grid = Grid::new(15, 15);

    // No obstacles - should allow diagonal movement via corners
    let start = Position::new(2, 2);
    let dest = Position::new(12, 12);

    let path = find_path(&grid, start.x, start.y, dest.x, dest.y, false, false);

    assert!(path.is_some(), "Path should exist");
    let path = path.unwrap();

    println!("{}", visualize_path(&grid, &path, start, dest));

    // In open space with direct visibility, should be direct
    assert_eq!(path.len(), 2, "Open diagonal should be direct");
}
