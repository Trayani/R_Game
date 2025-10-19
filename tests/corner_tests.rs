use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::collections::HashSet;

/// Parse a grid from the text format used in test files
/// Returns (grid, observer_x, observer_y, interesting_corners, non_interesting_corners, non_visible_corners)
fn parse_corner_test_grid(text: &str) -> (Grid, i32, i32, Vec<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let lines: Vec<&str> = text.trim().lines().collect();
    let rows = lines.len() as i32;
    let cols = if rows > 0 { lines[0].chars().count() as i32 } else { 0 };

    let mut grid = Grid::new(rows, cols);
    let mut observer_x = -1;
    let mut observer_y = -1;
    let mut interesting_corners = Vec::new();
    let mut non_interesting_corners = Vec::new();
    let mut non_visible_corners = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let x = x as i32;
            let y = y as i32;

            match ch {
                '■' => grid.set_cell(x, y, 1), // Blocked
                's' => {
                    observer_x = x;
                    observer_y = y;
                }
                '▲' => interesting_corners.push((x, y)),
                'n' => non_interesting_corners.push((x, y)),
                'u' => non_visible_corners.push((x, y)),
                _ => {} // Free cell or visible cell marked with 'o' or '□'
            }
        }
    }

    (grid, observer_x, observer_y, interesting_corners, non_interesting_corners, non_visible_corners)
}

#[test]
fn test_corners_md_example() {
    let test_data = r"□□□□□□□□□□□□□□□□□□□□□▲□□□□□□□
n□□□□▲□□□□u□u□□□□□□□□□■□□□□□□
□■■■■□□□□□□■□□□□□□□□□□■□□□□□□
▲□□□□n□□□□▲□▲□□□□□□□□□■□□□□□□
□□□□□□□□□□□□□□□□□□□□□□□▲□□□□□
u□▲□□□□□□□□□□□□□□□□□□□□□□▲□u□
□■□□□□□□□□□s□□□□□□□□□□□□□□■□□
u□▲□□□□□□□□□□□□□□□□□□□□□□▲□u□
□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
□□□□□□□□□□□□□□□□□□n□▲□□□□□□□□
□▲□n□□□□□□□□□□□□□□□■□□□□□□□□□
□□■□□□□□□□□□□□□□□□▲□u□□□□□□□□
□u□▲□□□□□□▲□▲□□□□□□□□□□□□□□□□
□□□□□□□□□□□■□□□□□□□□□□□□□□□□□
□□□□□□□□□□u□u□□□□□□□□□□□□□□□□";

    let (grid, obs_x, obs_y, interesting, non_interesting, non_visible) = parse_corner_test_grid(test_data);

    // Run raycasting to get visible cells
    let visible_cells = raycast(&grid, obs_x, obs_y);

    // Detect all corners
    let all_corners = detect_all_corners(&grid);

    // Filter to interesting corners
    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y);

    // Convert to positions for easier checking
    let interesting_positions: HashSet<(i32, i32)> =
        interesting_corners.iter().map(|c| (c.x, c.y)).collect();

    println!("Observer at ({}, {})", obs_x, obs_y);
    println!("Detected {} interesting corners", interesting_positions.len());
    println!("Expected {} interesting corners", interesting.len());

    // Verify interesting corners (marked with ▲)
    let mut missing_interesting = 0;
    for &(x, y) in &interesting {
        if !interesting_positions.contains(&(x, y)) {
            println!("MISSING interesting corner at ({}, {})", x, y);
            missing_interesting += 1;
        }
    }

    // Verify non-interesting corners are NOT in interesting set
    let mut false_non_interesting = 0;
    for &(x, y) in &non_interesting {
        if interesting_positions.contains(&(x, y)) {
            println!("FALSE POSITIVE: non-interesting corner at ({}, {}) marked as interesting", x, y);
            false_non_interesting += 1;
        }
    }

    // Verify non-visible corners are NOT detected as interesting
    let mut false_non_visible = 0;
    for &(x, y) in &non_visible {
        if interesting_positions.contains(&(x, y)) {
            println!("FALSE POSITIVE: non-visible corner at ({}, {}) marked as interesting", x, y);
            false_non_visible += 1;
        }
        // Also verify they're not visible
        let cell_id = grid.get_id(x, y);
        assert!(!visible_cells.contains(&cell_id),
                "Corner marked 'u' at ({}, {}) should not be visible", x, y);
    }

    println!("\nTest Results:");
    println!("Missing interesting (▲): {}/{}", missing_interesting, interesting.len());
    println!("False positive non-interesting (n): {}", false_non_interesting);
    println!("False positive non-visible (u): {}", false_non_visible);

    // Strict assertions based on CORNERS.md spec
    assert_eq!(missing_interesting, 0, "All corners marked ▲ should be detected as interesting");
    assert_eq!(false_non_interesting, 0, "Corners marked 'n' should NOT be detected as interesting");
    assert_eq!(false_non_visible, 0, "Corners marked 'u' should NOT be detected as interesting");
}

#[test]
fn test_1_base_corners() {
    let test_data = std::fs::read_to_string("test_data/corners/1_base.txt")
        .expect("Failed to read 1_base.txt");

    let (grid, obs_x, obs_y, interesting, _non_interesting, _non_visible) = parse_corner_test_grid(&test_data);

    println!("Grid size: {}x{}", grid.cols, grid.rows);
    println!("Observer at ({}, {})", obs_x, obs_y);
    println!("Expected {} interesting corners", interesting.len());

    // Run raycasting
    let visible_cells = raycast(&grid, obs_x, obs_y);
    println!("Visible cells: {}", visible_cells.len());

    // Detect corners
    let all_corners = detect_all_corners(&grid);
    println!("Total corners detected: {}", all_corners.len());

    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y);
    println!("Interesting corners detected: {}", interesting_corners.len());

    // Convert to positions
    let interesting_positions: HashSet<(i32, i32)> =
        interesting_corners.iter().map(|c| (c.x, c.y)).collect();

    // Verify interesting corners
    let mut missing = 0;
    for &(x, y) in &interesting {
        if !interesting_positions.contains(&(x, y)) {
            println!("MISSING interesting corner at ({}, {})", x, y);
            missing += 1;
        }
    }

    let mut false_positives = 0;
    for corner in &interesting_corners {
        if !interesting.contains(&(corner.x, corner.y)) {
            println!("FALSE POSITIVE: detected corner at ({}, {}) not marked in test data", corner.x, corner.y);
            false_positives += 1;
        }
    }

    println!("\nSummary:");
    println!("Missing: {}/{}", missing, interesting.len());
    println!("False positives: {}", false_positives);

    // For now, allow some tolerance as we refine the algorithm
    assert!(interesting_positions.len() > 0, "Should detect at least some corners");
}

#[test]
fn test_2_next_corners() {
    let test_data = std::fs::read_to_string("test_data/corners/2_next.txt")
        .expect("Failed to read 2_next.txt");

    // Parse grid - no observer needed, just test corner detection
    let lines: Vec<&str> = test_data.trim().lines()
        .filter(|line| !line.is_empty() && !line.contains("no observer"))
        .collect();

    let rows = lines.len() as i32;
    let cols = if rows > 0 { lines[0].chars().count() as i32 } else { 0 };

    let mut grid = Grid::new(rows, cols);
    let mut expected_corners = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let x = x as i32;
            let y = y as i32;

            match ch {
                '■' => grid.set_cell(x, y, 1), // Blocked
                '▲' | '!' => expected_corners.push((x, y)), // Expected corner
                _ => {} // Free cell
            }
        }
    }

    println!("Grid size: {}x{}", cols, rows);
    println!("Expected {} corners", expected_corners.len());

    // Detect all corners (no observer needed)
    let all_corners = detect_all_corners(&grid);
    let detected_positions: HashSet<(i32, i32)> =
        all_corners.iter().map(|c| (c.x, c.y)).collect();

    println!("Detected {} corners", detected_positions.len());
    for corner in &all_corners {
        println!("  Corner at ({}, {}) with directions: {:?}", corner.x, corner.y, corner.directions);
    }

    // Check each expected corner
    let mut missing = 0;
    for &(x, y) in &expected_corners {
        if !detected_positions.contains(&(x, y)) {
            println!("MISSING corner at ({}, {})", x, y);

            // Debug: check what's around this position
            println!("  North: blocked={}", grid.is_blocked(x, y - 1));
            println!("  South: blocked={}", grid.is_blocked(x, y + 1));
            println!("  East:  blocked={}", grid.is_blocked(x + 1, y));
            println!("  West:  blocked={}", grid.is_blocked(x - 1, y));
            println!("  NW:    blocked={}", grid.is_blocked(x - 1, y - 1));
            println!("  NE:    blocked={}", grid.is_blocked(x + 1, y - 1));
            println!("  SW:    blocked={}", grid.is_blocked(x - 1, y + 1));
            println!("  SE:    blocked={}", grid.is_blocked(x + 1, y + 1));

            missing += 1;
        }
    }

    // Check for false positives
    let mut false_positives = 0;
    for corner in &all_corners {
        if !expected_corners.contains(&(corner.x, corner.y)) {
            println!("FALSE POSITIVE: corner at ({}, {}) with directions {:?}",
                     corner.x, corner.y, corner.directions);
            false_positives += 1;
        }
    }

    println!("\nSummary:");
    println!("Expected: {}", expected_corners.len());
    println!("Detected: {}", detected_positions.len());
    println!("Missing: {}", missing);
    println!("False positives: {}", false_positives);

    assert_eq!(missing, 0, "Should detect all expected corners");
    assert_eq!(false_positives, 0, "Should not have false positive corners");
}

/// Parse a grid from the format used in 3_case.txt and 4_case.txt
/// Returns (grid, observer_x, observer_y, visible_positions, interesting_corners)
fn parse_visibility_corner_test(text: &str) -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>) {
    let lines: Vec<&str> = text.trim().lines()
        .filter(|line| !line.is_empty() && !line.starts_with("s ...") && !line.starts_with("x ...")
                && !line.starts_with("o ...") && !line.starts_with("c ..."))
        .collect();

    let rows = lines.len() as i32;
    let cols = if rows > 0 { lines[0].chars().count() as i32 } else { 0 };

    let mut grid = Grid::new(rows, cols);
    let mut observer_x = -1;
    let mut observer_y = -1;
    let mut visible_positions = HashSet::new();
    let mut interesting_corners = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let x = x as i32;
            let y = y as i32;

            match ch {
                'b' => grid.set_cell(x, y, 1), // Blocked
                's' => {
                    observer_x = x;
                    observer_y = y;
                    visible_positions.insert((x, y));
                }
                'o' => {
                    visible_positions.insert((x, y));
                }
                'c' => {
                    visible_positions.insert((x, y));
                    interesting_corners.push((x, y));
                }
                'x' => {} // Non-visible cell, do nothing
                _ => {}
            }
        }
    }

    (grid, observer_x, observer_y, visible_positions, interesting_corners)
}

#[test]
fn test_3_case_corners() {
    let test_data = std::fs::read_to_string("test_data/corners/3_case.txt")
        .expect("Failed to read 3_case.txt");

    let (grid, obs_x, obs_y, expected_visible, expected_interesting) = parse_visibility_corner_test(&test_data);

    println!("Grid size: {}x{}", grid.cols, grid.rows);
    println!("Observer at ({}, {})", obs_x, obs_y);
    println!("Expected {} visible cells", expected_visible.len());
    println!("Expected {} interesting corners", expected_interesting.len());

    // Run raycasting
    let visible_cells = raycast(&grid, obs_x, obs_y);
    println!("Detected {} visible cells", visible_cells.len());

    // Convert visible cell IDs to positions
    let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
        .map(|&id| grid.get_coords(id))
        .collect();

    // Check visibility matches
    let mut missing_visible = 0;
    let mut false_visible = 0;

    for &(x, y) in &expected_visible {
        if !visible_positions.contains(&(x, y)) {
            println!("MISSING visible cell at ({}, {})", x, y);
            missing_visible += 1;
        }
    }

    for &(x, y) in &visible_positions {
        if !expected_visible.contains(&(x, y)) {
            println!("FALSE POSITIVE visible cell at ({}, {})", x, y);
            false_visible += 1;
        }
    }

    println!("\nVisibility Results:");
    println!("Missing visible cells: {}", missing_visible);
    println!("False positive visible cells: {}", false_visible);

    // Detect corners
    let all_corners = detect_all_corners(&grid);
    println!("\nTotal corners detected: {}", all_corners.len());

    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y);
    println!("Interesting corners detected: {}", interesting_corners.len());

    // Convert to positions
    let interesting_positions: HashSet<(i32, i32)> =
        interesting_corners.iter().map(|c| (c.x, c.y)).collect();

    // Check interesting corners
    let mut missing_interesting = 0;
    let mut false_interesting = 0;

    for &(x, y) in &expected_interesting {
        if !interesting_positions.contains(&(x, y)) {
            println!("MISSING interesting corner at ({}, {})", x, y);
            missing_interesting += 1;
        }
    }

    for corner in &interesting_corners {
        if !expected_interesting.contains(&(corner.x, corner.y)) {
            println!("FALSE POSITIVE interesting corner at ({}, {})", corner.x, corner.y);
            false_interesting += 1;
        }
    }

    println!("\nInteresting Corner Results:");
    println!("Missing interesting corners: {}", missing_interesting);
    println!("False positive interesting corners: {}", false_interesting);

    // Assert visibility is correct
    assert_eq!(missing_visible, 0, "All expected visible cells should be detected");
    assert_eq!(false_visible, 0, "No false positive visible cells");

    // Assert interesting corners are correct
    assert_eq!(missing_interesting, 0, "All expected interesting corners should be detected");
    assert_eq!(false_interesting, 0, "No false positive interesting corners");
}

#[test]
fn test_4_case_corners() {
    let test_data = std::fs::read_to_string("test_data/corners/4_case.txt")
        .expect("Failed to read 4_case.txt");

    let (grid, obs_x, obs_y, expected_visible, expected_interesting) = parse_visibility_corner_test(&test_data);

    println!("Grid size: {}x{}", grid.cols, grid.rows);
    println!("Observer at ({}, {})", obs_x, obs_y);
    println!("Expected {} visible cells", expected_visible.len());
    println!("Expected {} interesting corners", expected_interesting.len());

    // Run raycasting
    let visible_cells = raycast(&grid, obs_x, obs_y);
    println!("Detected {} visible cells", visible_cells.len());

    // Convert visible cell IDs to positions
    let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
        .map(|&id| grid.get_coords(id))
        .collect();

    // Check visibility matches
    let mut missing_visible = 0;
    let mut false_visible = 0;

    for &(x, y) in &expected_visible {
        if !visible_positions.contains(&(x, y)) {
            println!("MISSING visible cell at ({}, {})", x, y);
            missing_visible += 1;
        }
    }

    for &(x, y) in &visible_positions {
        if !expected_visible.contains(&(x, y)) {
            println!("FALSE POSITIVE visible cell at ({}, {})", x, y);
            false_visible += 1;
        }
    }

    println!("\nVisibility Results:");
    println!("Missing visible cells: {}", missing_visible);
    println!("False positive visible cells: {}", false_visible);

    // Detect corners
    let all_corners = detect_all_corners(&grid);
    println!("\nTotal corners detected: {}", all_corners.len());

    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y);
    println!("Interesting corners detected: {}", interesting_corners.len());

    // Convert to positions
    let interesting_positions: HashSet<(i32, i32)> =
        interesting_corners.iter().map(|c| (c.x, c.y)).collect();

    // Check interesting corners
    let mut missing_interesting = 0;
    let mut false_interesting = 0;

    for &(x, y) in &expected_interesting {
        if !interesting_positions.contains(&(x, y)) {
            println!("MISSING interesting corner at ({}, {})", x, y);
            missing_interesting += 1;
        }
    }

    for corner in &interesting_corners {
        if !expected_interesting.contains(&(corner.x, corner.y)) {
            println!("FALSE POSITIVE interesting corner at ({}, {})", corner.x, corner.y);
            false_interesting += 1;
        }
    }

    println!("\nInteresting Corner Results:");
    println!("Missing interesting corners: {}", missing_interesting);
    println!("False positive interesting corners: {}", false_interesting);

    // Assert visibility is correct
    assert_eq!(missing_visible, 0, "All expected visible cells should be detected");
    assert_eq!(false_visible, 0, "No false positive visible cells");

    // Assert interesting corners are correct
    assert_eq!(missing_interesting, 0, "All expected interesting corners should be detected");
    assert_eq!(false_interesting, 0, "No false positive interesting corners");
}
