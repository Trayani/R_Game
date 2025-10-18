use rustgame3::{Grid, raycast};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Test data structure matching C# JSON export format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RaycastTestData {
    #[serde(rename = "testName")]
    pub test_name: String,
    #[serde(rename = "gridRows")]
    pub grid_rows: i32,
    #[serde(rename = "gridCols")]
    pub grid_cols: i32,
    #[serde(rename = "blockedCells")]
    pub blocked_cells: Vec<i32>,
    #[serde(rename = "startX")]
    pub start_x: i32,
    #[serde(rename = "startY")]
    pub start_y: i32,
    #[serde(rename = "expectedVisible")]
    pub expected_visible: Vec<i32>,
    pub log: Vec<String>,
}

/// Load a test from JSON file
pub fn load_test(path: &Path) -> Result<RaycastTestData, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let test_data: RaycastTestData = serde_json::from_str(&contents)?;
    Ok(test_data)
}

/// Helper functions for coordinate transformation
pub fn cell_id_to_coords(cell_id: i32, cols: i32) -> (i32, i32) {
    let x = cell_id % cols;
    let y = cell_id / cols;
    (x, y)
}

pub fn coords_to_cell_id(x: i32, y: i32, cols: i32) -> i32 {
    x + y * cols
}

/// Flip test horizontally (mirror left-right)
pub fn flip_test_horizontal(test: &RaycastTestData) -> RaycastTestData {
    let mut flipped = RaycastTestData {
        test_name: format!("{}_h_flip", test.test_name),
        grid_rows: test.grid_rows,
        grid_cols: test.grid_cols,
        blocked_cells: Vec::new(),
        start_x: test.grid_cols - 1 - test.start_x,
        start_y: test.start_y,
        expected_visible: Vec::new(),
        log: Vec::new(),
    };

    for &cell_id in &test.blocked_cells {
        let (x, y) = cell_id_to_coords(cell_id, test.grid_cols);
        let new_x = test.grid_cols - 1 - x;
        flipped.blocked_cells.push(coords_to_cell_id(new_x, y, test.grid_cols));
    }

    for &cell_id in &test.expected_visible {
        let (x, y) = cell_id_to_coords(cell_id, test.grid_cols);
        let new_x = test.grid_cols - 1 - x;
        flipped.expected_visible.push(coords_to_cell_id(new_x, y, test.grid_cols));
    }

    flipped
}

/// Flip test vertically (mirror top-bottom)
pub fn flip_test_vertical(test: &RaycastTestData) -> RaycastTestData {
    let mut flipped = RaycastTestData {
        test_name: format!("{}_v_flip", test.test_name),
        grid_rows: test.grid_rows,
        grid_cols: test.grid_cols,
        blocked_cells: Vec::new(),
        start_x: test.start_x,
        start_y: test.grid_rows - 1 - test.start_y,
        expected_visible: Vec::new(),
        log: Vec::new(),
    };

    for &cell_id in &test.blocked_cells {
        let (x, y) = cell_id_to_coords(cell_id, test.grid_cols);
        let new_y = test.grid_rows - 1 - y;
        flipped.blocked_cells.push(coords_to_cell_id(x, new_y, test.grid_cols));
    }

    for &cell_id in &test.expected_visible {
        let (x, y) = cell_id_to_coords(cell_id, test.grid_cols);
        let new_y = test.grid_rows - 1 - y;
        flipped.expected_visible.push(coords_to_cell_id(x, new_y, test.grid_cols));
    }

    flipped
}

/// Flip test both horizontally and vertically
pub fn flip_test_both(test: &RaycastTestData) -> RaycastTestData {
    let mut flipped = RaycastTestData {
        test_name: format!("{}_hv_flip", test.test_name),
        grid_rows: test.grid_rows,
        grid_cols: test.grid_cols,
        blocked_cells: Vec::new(),
        start_x: test.grid_cols - 1 - test.start_x,
        start_y: test.grid_rows - 1 - test.start_y,
        expected_visible: Vec::new(),
        log: Vec::new(),
    };

    for &cell_id in &test.blocked_cells {
        let (x, y) = cell_id_to_coords(cell_id, test.grid_cols);
        let new_x = test.grid_cols - 1 - x;
        let new_y = test.grid_rows - 1 - y;
        flipped.blocked_cells.push(coords_to_cell_id(new_x, new_y, test.grid_cols));
    }

    for &cell_id in &test.expected_visible {
        let (x, y) = cell_id_to_coords(cell_id, test.grid_cols);
        let new_x = test.grid_cols - 1 - x;
        let new_y = test.grid_rows - 1 - y;
        flipped.expected_visible.push(coords_to_cell_id(new_x, new_y, test.grid_cols));
    }

    flipped
}

/// Run a single test variant
pub fn run_single_test(test_data: &RaycastTestData) -> (bool, usize, usize, HashSet<i32>, HashSet<i32>) {
    let grid = Grid::with_blocked(
        test_data.grid_rows,
        test_data.grid_cols,
        &test_data.blocked_cells,
    );

    let actual_visible = raycast(&grid, test_data.start_x, test_data.start_y);
    let expected_set: HashSet<i32> = test_data.expected_visible.iter().copied().collect();

    let missing: Vec<_> = expected_set.difference(&actual_visible).copied().collect();
    let extra: Vec<_> = actual_visible.difference(&expected_set).copied().collect();

    let passed = missing.is_empty() && extra.is_empty();

    (passed, missing.len(), extra.len(), expected_set.clone(), actual_visible.clone())
}

/// Run a test with all 4 variants (original, h_flip, v_flip, hv_flip)
/// Returns (all_passed, failed_variant_name_if_any, missing_count, extra_count)
pub fn run_test(test_data: &RaycastTestData) -> (bool, Option<String>, usize, usize) {
    // Test all 4 variants
    let variants = vec![
        ("original", test_data.clone()),
        ("h_flip", flip_test_horizontal(test_data)),
        ("v_flip", flip_test_vertical(test_data)),
        ("hv_flip", flip_test_both(test_data)),
    ];

    for (variant_name, variant_test) in variants {
        let (passed, missing_count, extra_count, _, _) = run_single_test(&variant_test);
        if !passed {
            return (false, Some(variant_name.to_string()), missing_count, extra_count);
        }
    }

    (true, None, 0, 0)
}

/// Parse a standard format test file
/// Format:
/// - s: start position (observer)
/// - ■: blocked cell
/// - □: free cell
/// - o: visible free cell
pub fn parse_standard_test(path: &Path) -> Result<(Grid, i32, i32, HashSet<i32>), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();

    // Find non-empty lines that are valid grid rows
    let all_non_empty: Vec<&str> = lines.iter()
        .copied()
        .filter(|line| !line.trim().is_empty())
        .collect();

    if all_non_empty.is_empty() {
        return Err("No non-empty lines found in test file".into());
    }

    // Determine grid width from first line
    let grid_cols = all_non_empty[0].chars().count() as i32;

    // Valid grid characters
    let valid_chars = ['s', '■', '□', 'o', ' '];

    // Collect only valid grid rows (same width and valid characters)
    let mut grid_lines = Vec::new();
    for line in all_non_empty {
        let line_width = line.chars().count() as i32;

        // Stop if line width doesn't match expected grid width
        if line_width != grid_cols {
            break;
        }

        // Stop if line contains invalid characters
        let has_invalid = line.chars().any(|c| !valid_chars.contains(&c));
        if has_invalid {
            break;
        }

        grid_lines.push(line);
    }

    if grid_lines.is_empty() {
        return Err("No valid grid lines found in test file".into());
    }

    let grid_rows = grid_lines.len() as i32;

    let mut blocked_cells = Vec::new();
    let mut start_x = -1;
    let mut start_y = -1;
    let mut expected_visible = HashSet::new();

    for (y, line) in grid_lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let cell_id = x as i32 + (y as i32) * grid_cols;

            match ch {
                's' => {
                    start_x = x as i32;
                    start_y = y as i32;
                    expected_visible.insert(cell_id);
                }
                '■' => {
                    blocked_cells.push(cell_id);
                }
                'o' => {
                    expected_visible.insert(cell_id);
                }
                '□' | ' ' => {
                    // Free cell, do nothing
                }
                _ => {
                    // Unknown character, skip
                }
            }
        }
    }

    if start_x == -1 || start_y == -1 {
        return Err("No start position 's' found in test file".into());
    }

    let grid = Grid::with_blocked(grid_rows, grid_cols, &blocked_cells);
    Ok((grid, start_x, start_y, expected_visible))
}

/// Flip standard test data horizontally
pub fn flip_standard_horizontal(grid: &Grid, start_x: i32, start_y: i32, expected: &HashSet<i32>) -> (Grid, i32, i32, HashSet<i32>) {
    let mut blocked_cells = Vec::new();
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_x = grid.cols - 1 - x;
                blocked_cells.push(coords_to_cell_id(new_x, y, grid.cols));
            }
        }
    }

    let new_grid = Grid::with_blocked(grid.rows, grid.cols, &blocked_cells);
    let new_start_x = grid.cols - 1 - start_x;
    let new_expected: HashSet<i32> = expected.iter()
        .map(|&cell_id| {
            let (x, y) = cell_id_to_coords(cell_id, grid.cols);
            let new_x = grid.cols - 1 - x;
            coords_to_cell_id(new_x, y, grid.cols)
        })
        .collect();

    (new_grid, new_start_x, start_y, new_expected)
}

/// Flip standard test data vertically
pub fn flip_standard_vertical(grid: &Grid, start_x: i32, start_y: i32, expected: &HashSet<i32>) -> (Grid, i32, i32, HashSet<i32>) {
    let mut blocked_cells = Vec::new();
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_y = grid.rows - 1 - y;
                blocked_cells.push(coords_to_cell_id(x, new_y, grid.cols));
            }
        }
    }

    let new_grid = Grid::with_blocked(grid.rows, grid.cols, &blocked_cells);
    let new_start_y = grid.rows - 1 - start_y;
    let new_expected: HashSet<i32> = expected.iter()
        .map(|&cell_id| {
            let (x, y) = cell_id_to_coords(cell_id, grid.cols);
            let new_y = grid.rows - 1 - y;
            coords_to_cell_id(x, new_y, grid.cols)
        })
        .collect();

    (new_grid, start_x, new_start_y, new_expected)
}

/// Flip standard test data both horizontally and vertically
pub fn flip_standard_both(grid: &Grid, start_x: i32, start_y: i32, expected: &HashSet<i32>) -> (Grid, i32, i32, HashSet<i32>) {
    let mut blocked_cells = Vec::new();
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.is_blocked(x, y) {
                let new_x = grid.cols - 1 - x;
                let new_y = grid.rows - 1 - y;
                blocked_cells.push(coords_to_cell_id(new_x, new_y, grid.cols));
            }
        }
    }

    let new_grid = Grid::with_blocked(grid.rows, grid.cols, &blocked_cells);
    let new_start_x = grid.cols - 1 - start_x;
    let new_start_y = grid.rows - 1 - start_y;
    let new_expected: HashSet<i32> = expected.iter()
        .map(|&cell_id| {
            let (x, y) = cell_id_to_coords(cell_id, grid.cols);
            let new_x = grid.cols - 1 - x;
            let new_y = grid.rows - 1 - y;
            coords_to_cell_id(new_x, new_y, grid.cols)
        })
        .collect();

    (new_grid, new_start_x, new_start_y, new_expected)
}
