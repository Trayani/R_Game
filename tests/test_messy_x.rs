use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners_with_observer_corners};
use std::collections::HashSet;

/// Parse messy X test format
/// Observer is marked with two adjacent 's' cells (observer occupies both)
/// Returns (grid, observer_x, observer_y, visible_positions, interesting_corners, observer_corners)
/// observer_x is the leftmost of the two observer cells
fn parse_messy_x_test(text: &str) -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let lines: Vec<&str> = text.trim().lines()
        .filter(|line| !line.is_empty() && !line.starts_with("s ...") && !line.starts_with("b ...")
                && !line.starts_with("x ...") && !line.starts_with("o ...") && !line.starts_with("c ...") && !line.starts_with("z ..."))
        .collect();

    let rows = lines.len() as i32;
    let cols = if rows > 0 { lines[0].chars().count() as i32 } else { 0 };

    let mut grid = Grid::new(rows, cols);
    let mut observer_x = -1;
    let mut observer_y = -1;
    let mut visible_positions = HashSet::new();
    let mut interesting_corners = Vec::new();
    let mut observer_corners = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let x = x as i32;
            let y = y as i32;

            match ch {
                'b' => grid.set_cell(x, y, 1), // Blocked
                's' => {
                    // First 's' marks the left observer cell
                    if observer_x == -1 {
                        observer_x = x;
                        observer_y = y;
                    }
                    visible_positions.insert((x, y));
                }
                'o' => {
                    visible_positions.insert((x, y));
                }
                'c' => {
                    visible_positions.insert((x, y));
                    interesting_corners.push((x, y));
                }
                'z' => {
                    // Observer corner - visible and auto-interesting
                    visible_positions.insert((x, y));
                    observer_corners.push((x, y));
                }
                'x' => {} // Non-visible cell
                _ => {}
            }
        }
    }

    (grid, observer_x, observer_y, visible_positions, interesting_corners, observer_corners)
}

/// Flip messy X test horizontally
/// For messy X: new_obs_x = cols - obs_x - 2 (to maintain left/right cell ordering)
fn flip_messy_x_horizontal(grid: &Grid, obs_x: i32, obs_y: i32,
                           visible: &HashSet<(i32, i32)>,
                           corners: &Vec<(i32, i32)>,
                           observer_corners: &Vec<(i32, i32)>)
    -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let mut new_grid = Grid::new(grid.rows, grid.cols);
    let new_obs_x = grid.cols - obs_x - 2;  // Messy X offset formula
    let new_obs_y = obs_y;

    // Flip blocked cells
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_x = grid.cols - 1 - x;
                new_grid.set_cell(new_x, y, 1);
            }
        }
    }

    // Flip visible positions
    let new_visible: HashSet<(i32, i32)> = visible.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, y))
        .collect();

    // Flip corners
    let new_corners: Vec<(i32, i32)> = corners.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, y))
        .collect();

    // Flip observer corners
    let new_observer_corners: Vec<(i32, i32)> = observer_corners.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, y))
        .collect();

    (new_grid, new_obs_x, new_obs_y, new_visible, new_corners, new_observer_corners)
}

/// Flip messy X test vertically
fn flip_messy_x_vertical(grid: &Grid, obs_x: i32, obs_y: i32,
                         visible: &HashSet<(i32, i32)>,
                         corners: &Vec<(i32, i32)>,
                         observer_corners: &Vec<(i32, i32)>)
    -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let mut new_grid = Grid::new(grid.rows, grid.cols);
    let new_obs_x = obs_x;  // X stays same for vertical flip
    let new_obs_y = grid.rows - 1 - obs_y;

    // Flip blocked cells
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_y = grid.rows - 1 - y;
                new_grid.set_cell(x, new_y, 1);
            }
        }
    }

    // Flip visible positions
    let new_visible: HashSet<(i32, i32)> = visible.iter()
        .map(|&(x, y)| (x, grid.rows - 1 - y))
        .collect();

    // Flip corners
    let new_corners: Vec<(i32, i32)> = corners.iter()
        .map(|&(x, y)| (x, grid.rows - 1 - y))
        .collect();

    // Flip observer corners
    let new_observer_corners: Vec<(i32, i32)> = observer_corners.iter()
        .map(|&(x, y)| (x, grid.rows - 1 - y))
        .collect();

    (new_grid, new_obs_x, new_obs_y, new_visible, new_corners, new_observer_corners)
}

/// Flip messy X test both horizontally and vertically
fn flip_messy_x_both(grid: &Grid, obs_x: i32, obs_y: i32,
                     visible: &HashSet<(i32, i32)>,
                     corners: &Vec<(i32, i32)>,
                     observer_corners: &Vec<(i32, i32)>)
    -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let mut new_grid = Grid::new(grid.rows, grid.cols);
    let new_obs_x = grid.cols - obs_x - 2;  // Messy X offset formula
    let new_obs_y = grid.rows - 1 - obs_y;

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

    // Flip visible positions
    let new_visible: HashSet<(i32, i32)> = visible.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, grid.rows - 1 - y))
        .collect();

    // Flip corners
    let new_corners: Vec<(i32, i32)> = corners.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, grid.rows - 1 - y))
        .collect();

    // Flip observer corners
    let new_observer_corners: Vec<(i32, i32)> = observer_corners.iter()
        .map(|&(x, y)| (grid.cols - 1 - x, grid.rows - 1 - y))
        .collect();

    (new_grid, new_obs_x, new_obs_y, new_visible, new_corners, new_observer_corners)
}

#[test]
fn test_5_messy_x() {
    let test_data = std::fs::read_to_string("test_data/corners/5_messy_x.txt")
        .expect("Failed to read 5_messy_x.txt");

    // Extract the messy X test cases (lines after "with messy X")
    let messy_x_section: Vec<&str> = test_data.lines()
        .skip_while(|line| !line.contains("with messy X"))
        .skip(2) // Skip header and empty line
        .take(4) // Take the 4 test case lines
        .collect();

    // Each line contains multiple test cases separated by '|'
    let test_cases: Vec<Vec<&str>> = messy_x_section.iter()
        .map(|line| line.split('|').map(|s| s.trim()).collect())
        .collect();

    // Transpose to get individual test cases
    let num_tests = test_cases[0].len();

    for test_idx in 0..num_tests {
        // Build test grid from this column
        let mut test_lines = Vec::new();
        for row in &test_cases {
            if test_idx < row.len() && !row[test_idx].is_empty() {
                test_lines.push(row[test_idx]);
            }
        }

        if test_lines.is_empty() {
            continue;
        }

        let test_text = test_lines.join("\n");

        let (grid, obs_x, obs_y, expected_visible, expected_interesting, observer_corners) =
            parse_messy_x_test(&test_text);

        if obs_x == -1 {
            continue; // No observer found, skip
        }

        println!("\n=== Messy X Test Case {} ===", test_idx + 1);
        println!("Grid: {}x{}", grid.cols, grid.rows);
        println!("Observer at ({}, {}) + ({}, {}) [messy X]", obs_x, obs_y, obs_x + 1, obs_y);
        println!("Expected {} visible cells", expected_visible.len());
        println!("Expected {} interesting corners", expected_interesting.len());
        println!("Expected {} observer corners", observer_corners.len());

        // Test all 4 variants with 3FLIP
        let variants = vec![
            ("original", (grid.clone(), obs_x, obs_y, expected_visible.clone(), expected_interesting.clone(), observer_corners.clone())),
            ("h_flip", flip_messy_x_horizontal(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
            ("v_flip", flip_messy_x_vertical(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
            ("hv_flip", flip_messy_x_both(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
        ];

        for (variant_name, (variant_grid, variant_obs_x, variant_obs_y, variant_expected_visible, variant_expected_interesting, variant_observer_corners)) in variants {
            println!("\n  Testing variant: {}", variant_name);
            println!("  Observer at ({}, {}) + ({}, {})", variant_obs_x, variant_obs_y, variant_obs_x + 1, variant_obs_y);

            // Run raycasting with messy_x=true
            let visible_cells = raycast(&variant_grid, variant_obs_x, variant_obs_y, true);
            let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
                .map(|&id| variant_grid.get_coords(id))
                .collect();

            // Check visibility
            let mut missing_visible = 0;
            let mut false_visible = 0;

            for &(x, y) in &variant_expected_visible {
                if !visible_positions.contains(&(x, y)) {
                    println!("    MISSING visible at ({}, {})", x, y);
                    missing_visible += 1;
                }
            }

            for &(x, y) in &visible_positions {
                if !variant_expected_visible.contains(&(x, y)) {
                    println!("    FALSE POSITIVE visible at ({}, {})", x, y);
                    false_visible += 1;
                }
            }

            // Detect corners
            let all_corners = detect_all_corners(&variant_grid);
            let interesting_corners = filter_interesting_corners_with_observer_corners(
                &all_corners, &visible_cells, &variant_grid, variant_obs_x, variant_obs_y, true, &variant_observer_corners);

            let interesting_positions: HashSet<(i32, i32)> =
                interesting_corners.iter().map(|c| (c.x, c.y)).collect();

            // Check corners (including observer corners)
            let mut all_expected_interesting = variant_expected_interesting.clone();
            all_expected_interesting.extend(variant_observer_corners.clone());

            let mut missing_corners = 0;
            let mut false_corners = 0;

            for &(x, y) in &all_expected_interesting {
                if !interesting_positions.contains(&(x, y)) {
                    println!("    MISSING corner at ({}, {})", x, y);
                    missing_corners += 1;
                }
            }

            for corner in &interesting_corners {
                if !all_expected_interesting.contains(&(corner.x, corner.y)) {
                    println!("    FALSE POSITIVE corner at ({}, {})", corner.x, corner.y);
                    false_corners += 1;
                }
            }

            // Assert test passes
            assert_eq!(missing_visible, 0, "Test case {} {}: Missing visible cells", test_idx + 1, variant_name);
            assert_eq!(false_visible, 0, "Test case {} {}: False positive visible cells", test_idx + 1, variant_name);
            assert_eq!(missing_corners, 0, "Test case {} {}: Missing interesting corners", test_idx + 1, variant_name);
            assert_eq!(false_corners, 0, "Test case {} {}: False positive interesting corners", test_idx + 1, variant_name);

            println!("  ✓ Variant {} PASSED", variant_name);
        }

        println!("Test case {} (all 4 variants) PASSED", test_idx + 1);
    }

    println!("\nAll messy X test cases passed!");
}

#[test]
fn test_6_messy_x() {
    let test_data = std::fs::read_to_string("test_data/corners/6_messy_x.txt")
        .expect("Failed to read 6_messy_x.txt");

    // Skip comment lines and parse the grid
    let grid_lines: Vec<&str> = test_data.lines()
        .skip_while(|line| line.starts_with("s ...") || line.starts_with("x ...")
                    || line.starts_with("o ...") || line.starts_with("c ...") || line.trim().is_empty())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let test_text = grid_lines.join("\n");

    let (grid, obs_x, obs_y, expected_visible, expected_interesting, observer_corners) =
        parse_messy_x_test(&test_text);

    println!("\n=== Test 6_messy_x.txt ===");
    println!("Grid: {}x{}", grid.cols, grid.rows);
    println!("Observer at ({}, {}) + ({}, {}) [messy X]", obs_x, obs_y, obs_x + 1, obs_y);
    println!("Expected {} visible cells", expected_visible.len());
    println!("Expected {} interesting corners", expected_interesting.len());
    println!("Expected {} observer corners", observer_corners.len());

    // Test all 4 variants with 3FLIP
    let variants = vec![
        ("original", (grid.clone(), obs_x, obs_y, expected_visible.clone(), expected_interesting.clone(), observer_corners.clone())),
        ("h_flip", flip_messy_x_horizontal(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
        ("v_flip", flip_messy_x_vertical(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
        ("hv_flip", flip_messy_x_both(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
    ];

    for (variant_name, (variant_grid, variant_obs_x, variant_obs_y, variant_expected_visible, variant_expected_interesting, variant_observer_corners)) in variants {
        println!("\n  Testing variant: {}", variant_name);
        println!("  Observer at ({}, {}) + ({}, {})", variant_obs_x, variant_obs_y, variant_obs_x + 1, variant_obs_y);

        // Run raycasting with messy_x=true
        let visible_cells = raycast(&variant_grid, variant_obs_x, variant_obs_y, true);
        let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
            .map(|&id| variant_grid.get_coords(id))
            .collect();

        // Check visibility
        let mut missing_visible = 0;
        let mut false_visible = 0;

        for &(x, y) in &variant_expected_visible {
            if !visible_positions.contains(&(x, y)) {
                println!("    MISSING visible at ({}, {})", x, y);
                missing_visible += 1;
            }
        }

        for &(x, y) in &visible_positions {
            if !variant_expected_visible.contains(&(x, y)) {
                println!("    FALSE POSITIVE visible at ({}, {})", x, y);
                false_visible += 1;
            }
        }

        // Detect corners
        let all_corners = detect_all_corners(&variant_grid);
        let interesting_corners = filter_interesting_corners_with_observer_corners(
            &all_corners, &visible_cells, &variant_grid, variant_obs_x, variant_obs_y, true, &variant_observer_corners);

        let interesting_positions: HashSet<(i32, i32)> =
            interesting_corners.iter().map(|c| (c.x, c.y)).collect();

        // Check corners
        let mut all_expected_interesting = variant_expected_interesting.clone();
        all_expected_interesting.extend(variant_observer_corners.clone());

        let mut missing_corners = 0;
        let mut false_corners = 0;

        for &(x, y) in &all_expected_interesting {
            if !interesting_positions.contains(&(x, y)) {
                println!("    MISSING corner at ({}, {})", x, y);
                missing_corners += 1;
            }
        }

        for corner in &interesting_corners {
            if !all_expected_interesting.contains(&(corner.x, corner.y)) {
                println!("    FALSE POSITIVE corner at ({}, {})", corner.x, corner.y);
                false_corners += 1;
            }
        }

        // Assert
        assert_eq!(missing_visible, 0, "Variant {}: Missing visible cells", variant_name);
        assert_eq!(false_visible, 0, "Variant {}: False positive visible cells", variant_name);
        assert_eq!(missing_corners, 0, "Variant {}: Missing interesting corners", variant_name);
        assert_eq!(false_corners, 0, "Variant {}: False positive interesting corners", variant_name);

        println!("  ✓ Variant {} PASSED", variant_name);
    }

    println!("\nTest 6_messy_x.txt (all 4 variants) PASSED");
}

#[test]
fn test_7_messy_x2() {
    let test_data = std::fs::read_to_string("test_data/corners/7_messy_x2.txt")
        .expect("Failed to read 7_messy_x2.txt");

    // Skip comment lines and parse the grid
    let grid_lines: Vec<&str> = test_data.lines()
        .skip_while(|line| line.starts_with("s ...") || line.starts_with("x ...")
                    || line.starts_with("o ...") || line.starts_with("c ...") || line.trim().is_empty())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let test_text = grid_lines.join("\n");

    let (grid, obs_x, obs_y, expected_visible, expected_interesting, observer_corners) =
        parse_messy_x_test(&test_text);

    println!("\n=== Test 7_messy_x2.txt ===");
    println!("Grid: {}x{}", grid.cols, grid.rows);
    println!("Observer at ({}, {}) + ({}, {}) [messy X]", obs_x, obs_y, obs_x + 1, obs_y);
    println!("Expected {} visible cells", expected_visible.len());
    println!("Expected {} interesting corners", expected_interesting.len());
    println!("Expected {} observer corners", observer_corners.len());

    // Test all 4 variants with 3FLIP
    let variants = vec![
        ("original", (grid.clone(), obs_x, obs_y, expected_visible.clone(), expected_interesting.clone(), observer_corners.clone())),
        ("h_flip", flip_messy_x_horizontal(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
        ("v_flip", flip_messy_x_vertical(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
        ("hv_flip", flip_messy_x_both(&grid, obs_x, obs_y, &expected_visible, &expected_interesting, &observer_corners)),
    ];

    for (variant_name, (variant_grid, variant_obs_x, variant_obs_y, variant_expected_visible, variant_expected_interesting, variant_observer_corners)) in variants {
        println!("\n  Testing variant: {}", variant_name);
        println!("  Observer at ({}, {}) + ({}, {})", variant_obs_x, variant_obs_y, variant_obs_x + 1, variant_obs_y);

        // Run raycasting with messy_x=true
        let visible_cells = raycast(&variant_grid, variant_obs_x, variant_obs_y, true);
        let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
            .map(|&id| variant_grid.get_coords(id))
            .collect();

        // Check visibility
        let mut missing_visible = 0;
        let mut false_visible = 0;

        for &(x, y) in &variant_expected_visible {
            if !visible_positions.contains(&(x, y)) {
                println!("    MISSING visible at ({}, {})", x, y);
                missing_visible += 1;
            }
        }

        for &(x, y) in &visible_positions {
            if !variant_expected_visible.contains(&(x, y)) {
                println!("    FALSE POSITIVE visible at ({}, {})", x, y);
                false_visible += 1;
            }
        }

        // Detect corners
        let all_corners = detect_all_corners(&variant_grid);
        let interesting_corners = filter_interesting_corners_with_observer_corners(
            &all_corners, &visible_cells, &variant_grid, variant_obs_x, variant_obs_y, true, &variant_observer_corners);

        let interesting_positions: HashSet<(i32, i32)> =
            interesting_corners.iter().map(|c| (c.x, c.y)).collect();

        // Check corners
        let mut all_expected_interesting = variant_expected_interesting.clone();
        all_expected_interesting.extend(variant_observer_corners.clone());

        let mut missing_corners = 0;
        let mut false_corners = 0;

        for &(x, y) in &all_expected_interesting {
            if !interesting_positions.contains(&(x, y)) {
                println!("    MISSING corner at ({}, {})", x, y);
                missing_corners += 1;
            }
        }

        for corner in &interesting_corners {
            if !all_expected_interesting.contains(&(corner.x, corner.y)) {
                println!("    FALSE POSITIVE corner at ({}, {})", corner.x, corner.y);
                false_corners += 1;
            }
        }

        // Assert
        assert_eq!(missing_visible, 0, "Variant {}: Missing visible cells", variant_name);
        assert_eq!(false_visible, 0, "Variant {}: False positive visible cells", variant_name);
        assert_eq!(missing_corners, 0, "Variant {}: Missing interesting corners", variant_name);
        assert_eq!(false_corners, 0, "Variant {}: False positive interesting corners", variant_name);

        println!("  ✓ Variant {} PASSED", variant_name);
    }

    println!("\nTest 7_messy_x2.txt (all 4 variants) PASSED");
}
