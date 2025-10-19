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

        // Run raycasting with messy_x=true
        let visible_cells = raycast(&grid, obs_x, obs_y, true);
        let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
            .map(|&id| grid.get_coords(id))
            .collect();

        println!("Detected {} visible cells", visible_positions.len());

        // Check visibility
        let mut missing_visible = 0;
        let mut false_visible = 0;

        for &(x, y) in &expected_visible {
            if !visible_positions.contains(&(x, y)) {
                println!("MISSING visible at ({}, {})", x, y);
                missing_visible += 1;
            }
        }

        for &(x, y) in &visible_positions {
            if !expected_visible.contains(&(x, y)) {
                println!("FALSE POSITIVE visible at ({}, {})", x, y);
                false_visible += 1;
            }
        }

        // Detect corners
        let all_corners = detect_all_corners(&grid);
        println!("All corners: {}", all_corners.len());
        for corner in &all_corners {
            println!("  Corner at ({}, {}) with dirs: {:?}", corner.x, corner.y, corner.directions);
        }

        let interesting_corners = filter_interesting_corners_with_observer_corners(&all_corners, &visible_cells, &grid, obs_x, obs_y, true, &observer_corners);

        println!("Detected {} interesting corners", interesting_corners.len());
        for corner in &interesting_corners {
            println!("  Interesting at ({}, {}) with dirs: {:?}", corner.x, corner.y, corner.directions);
        }

        let interesting_positions: HashSet<(i32, i32)> =
            interesting_corners.iter().map(|c| (c.x, c.y)).collect();

        // Check corners (including observer corners)
        let mut all_expected_interesting = expected_interesting.clone();
        all_expected_interesting.extend(observer_corners.clone());

        let mut missing_corners = 0;
        let mut false_corners = 0;

        for &(x, y) in &all_expected_interesting {
            if !interesting_positions.contains(&(x, y)) {
                println!("MISSING corner at ({}, {})", x, y);
                missing_corners += 1;
            }
        }

        for corner in &interesting_corners {
            if !all_expected_interesting.contains(&(corner.x, corner.y)) {
                println!("FALSE POSITIVE corner at ({}, {})", corner.x, corner.y);
                false_corners += 1;
            }
        }

        // Assert test passes
        assert_eq!(missing_visible, 0, "Test case {}: Missing visible cells", test_idx + 1);
        assert_eq!(false_visible, 0, "Test case {}: False positive visible cells", test_idx + 1);
        assert_eq!(missing_corners, 0, "Test case {}: Missing interesting corners", test_idx + 1);
        assert_eq!(false_corners, 0, "Test case {}: False positive interesting corners", test_idx + 1);

        println!("Test case {} PASSED", test_idx + 1);
    }

    println!("\nAll messy X test cases passed!");
}
