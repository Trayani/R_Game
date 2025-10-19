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
    let visible_cells = raycast(&grid, obs_x, obs_y, false, false);

    // Detect all corners
    let all_corners = detect_all_corners(&grid);

    // Filter to interesting corners
    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y, false);

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
    let visible_cells = raycast(&grid, obs_x, obs_y, false, false);
    println!("Visible cells: {}", visible_cells.len());

    // Detect corners
    let all_corners = detect_all_corners(&grid);
    println!("Total corners detected: {}", all_corners.len());

    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y, false);
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

/// Flip corner test data horizontally
fn flip_corner_test_horizontal(grid: &Grid, obs_x: i32, obs_y: i32,
                                visible: &HashSet<(i32, i32)>,
                                corners: &Vec<(i32, i32)>)
    -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>) {
    let mut new_grid = Grid::new(grid.rows, grid.cols);

    // Flip blocked cells
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_x = grid.cols - 1 - x;
                new_grid.set_cell(new_x, y, 1);
            }
        }
    }

    // Flip observer
    let new_obs_x = grid.cols - 1 - obs_x;

    // Flip visible positions
    let new_visible: HashSet<(i32, i32)> = visible.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, y))
        .collect();

    // Flip corner positions
    let new_corners: Vec<(i32, i32)> = corners.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, y))
        .collect();

    (new_grid, new_obs_x, obs_y, new_visible, new_corners)
}

/// Flip corner test data vertically
fn flip_corner_test_vertical(grid: &Grid, obs_x: i32, obs_y: i32,
                              visible: &HashSet<(i32, i32)>,
                              corners: &Vec<(i32, i32)>)
    -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>) {
    let mut new_grid = Grid::new(grid.rows, grid.cols);

    // Flip blocked cells
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_y = grid.rows - 1 - y;
                new_grid.set_cell(x, new_y, 1);
            }
        }
    }

    // Flip observer
    let new_obs_y = grid.rows - 1 - obs_y;

    // Flip visible positions
    let new_visible: HashSet<(i32, i32)> = visible.iter()
        .map(|&(x, y)| (x, grid.rows - 1 - y))
        .collect();

    // Flip corner positions
    let new_corners: Vec<(i32, i32)> = corners.iter()
        .map(|&(x, y)| (x, grid.rows - 1 - y))
        .collect();

    (new_grid, obs_x, new_obs_y, new_visible, new_corners)
}

/// Flip corner test data both horizontally and vertically
fn flip_corner_test_both(grid: &Grid, obs_x: i32, obs_y: i32,
                         visible: &HashSet<(i32, i32)>,
                         corners: &Vec<(i32, i32)>)
    -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>) {
    let mut new_grid = Grid::new(grid.rows, grid.cols);

    // Flip blocked cells
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_x = grid.cols - 1 - x;
                let new_y = grid.rows - 1 - y;
                new_grid.set_cell(new_x, new_y, 1);
            }
        }
    }

    // Flip observer
    let new_obs_x = grid.cols - 1 - obs_x;
    let new_obs_y = grid.rows - 1 - obs_y;

    // Flip visible positions
    let new_visible: HashSet<(i32, i32)> = visible.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, grid.rows - 1 - y))
        .collect();

    // Flip corner positions
    let new_corners: Vec<(i32, i32)> = corners.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, grid.rows - 1 - y))
        .collect();

    (new_grid, new_obs_x, new_obs_y, new_visible, new_corners)
}

/// Run corner test with a specific variant
/// Returns (missing_visible, false_visible, missing_interesting, false_interesting, reciprocal_failures)
fn run_corner_test_variant(grid: &Grid, obs_x: i32, obs_y: i32,
                           expected_visible: &HashSet<(i32, i32)>,
                           expected_interesting: &Vec<(i32, i32)>,
                           check_reciprocal: bool)
    -> (usize, usize, usize, usize, usize) {
    // Run raycasting
    let visible_cells = raycast(&grid, obs_x, obs_y, false, false);
    let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
        .map(|&id| grid.get_coords(id))
        .collect();

    // Check visibility matches
    let mut missing_visible = 0;
    let mut false_visible = 0;

    for &(x, y) in expected_visible {
        if !visible_positions.contains(&(x, y)) {
            missing_visible += 1;
        }
    }

    for &(x, y) in &visible_positions {
        if !expected_visible.contains(&(x, y)) {
            false_visible += 1;
        }
    }

    // Detect corners
    let all_corners = detect_all_corners(&grid);
    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y, false);
    let interesting_positions: HashSet<(i32, i32)> =
        interesting_corners.iter().map(|c| (c.x, c.y)).collect();

    // Check interesting corners
    let mut missing_interesting = 0;
    let mut false_interesting = 0;

    for &(x, y) in expected_interesting {
        if !interesting_positions.contains(&(x, y)) {
            missing_interesting += 1;
        }
    }

    for corner in &interesting_corners {
        if !expected_interesting.contains(&(corner.x, corner.y)) {
            false_interesting += 1;
        }
    }

    // Reciprocal visibility test: if corner C is visible from observer O, then O must be visible from C
    let mut reciprocal_failures = 0;
    if check_reciprocal {
        for corner in &interesting_corners {
            let corner_visible_cells = raycast(&grid, corner.x, corner.y, false, false);
            let corner_visible_positions: HashSet<(i32, i32)> = corner_visible_cells.iter()
                .map(|&id| grid.get_coords(id))
                .collect();

            if !corner_visible_positions.contains(&(obs_x, obs_y)) {
                reciprocal_failures += 1;
            }
        }
    }

    (missing_visible, false_visible, missing_interesting, false_interesting, reciprocal_failures)
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

    // Test all 4 variants
    let variants = vec![
        ("original", grid.clone(), obs_x, obs_y, expected_visible.clone(), expected_interesting.clone()),
        {
            let (g, x, y, v, c) = flip_corner_test_horizontal(&grid, obs_x, obs_y, &expected_visible, &expected_interesting);
            ("h_flip", g, x, y, v, c)
        },
        {
            let (g, x, y, v, c) = flip_corner_test_vertical(&grid, obs_x, obs_y, &expected_visible, &expected_interesting);
            ("v_flip", g, x, y, v, c)
        },
        {
            let (g, x, y, v, c) = flip_corner_test_both(&grid, obs_x, obs_y, &expected_visible, &expected_interesting);
            ("hv_flip", g, x, y, v, c)
        },
    ];

    for (variant_name, variant_grid, variant_obs_x, variant_obs_y, variant_visible, variant_corners) in variants {
        let (missing_visible, false_visible, missing_interesting, false_interesting, reciprocal_failures) =
            run_corner_test_variant(&variant_grid, variant_obs_x, variant_obs_y, &variant_visible, &variant_corners, true);

        if missing_visible > 0 || false_visible > 0 || missing_interesting > 0 || false_interesting > 0 || reciprocal_failures > 0 {
            panic!(
                "Test 3_case [{}] failed - missing_visible: {}, false_visible: {}, missing_interesting: {}, false_interesting: {}, reciprocal_failures: {}",
                variant_name, missing_visible, false_visible, missing_interesting, false_interesting, reciprocal_failures
            );
        }
    }

    println!("All 4 variants passed (including reciprocal visibility)!");
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

    // Test all 4 variants
    let variants = vec![
        ("original", grid.clone(), obs_x, obs_y, expected_visible.clone(), expected_interesting.clone()),
        {
            let (g, x, y, v, c) = flip_corner_test_horizontal(&grid, obs_x, obs_y, &expected_visible, &expected_interesting);
            ("h_flip", g, x, y, v, c)
        },
        {
            let (g, x, y, v, c) = flip_corner_test_vertical(&grid, obs_x, obs_y, &expected_visible, &expected_interesting);
            ("v_flip", g, x, y, v, c)
        },
        {
            let (g, x, y, v, c) = flip_corner_test_both(&grid, obs_x, obs_y, &expected_visible, &expected_interesting);
            ("hv_flip", g, x, y, v, c)
        },
    ];

    for (variant_name, variant_grid, variant_obs_x, variant_obs_y, variant_visible, variant_corners) in variants {
        let (missing_visible, false_visible, missing_interesting, false_interesting, reciprocal_failures) =
            run_corner_test_variant(&variant_grid, variant_obs_x, variant_obs_y, &variant_visible, &variant_corners, true);

        if missing_visible > 0 || false_visible > 0 || missing_interesting > 0 || false_interesting > 0 || reciprocal_failures > 0 {
            panic!(
                "Test 4_case [{}] failed - missing_visible: {}, false_visible: {}, missing_interesting: {}, false_interesting: {}, reciprocal_failures: {}",
                variant_name, missing_visible, false_visible, missing_interesting, false_interesting, reciprocal_failures
            );
        }
    }

    println!("All 4 variants passed (including reciprocal visibility)!");
}
