mod cone;
mod grid;
mod ray;
mod raycast;

use arboard::Clipboard;
use grid::Grid;
use macroquad::prelude::*;
use raycast::raycast;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Test data structure matching C# JSON export format
#[derive(Debug, Clone, Deserialize, Serialize)]
struct RaycastTestData {
    #[serde(rename = "testName")]
    test_name: String,
    #[serde(rename = "gridRows")]
    grid_rows: i32,
    #[serde(rename = "gridCols")]
    grid_cols: i32,
    #[serde(rename = "blockedCells")]
    blocked_cells: Vec<i32>,
    #[serde(rename = "startX")]
    start_x: i32,
    #[serde(rename = "startY")]
    start_y: i32,
    #[serde(rename = "expectedVisible")]
    expected_visible: Vec<i32>,
    log: Vec<String>,
}

/// Load a test from JSON file
fn load_test(path: &Path) -> Result<RaycastTestData, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let test_data: RaycastTestData = serde_json::from_str(&contents)?;
    Ok(test_data)
}

/// Helper functions for coordinate transformation
fn cell_id_to_coords(cell_id: i32, cols: i32) -> (i32, i32) {
    let x = cell_id % cols;
    let y = cell_id / cols;
    (x, y)
}

fn coords_to_cell_id(x: i32, y: i32, cols: i32) -> i32 {
    x + y * cols
}

/// Flip test horizontally (mirror left-right)
fn flip_test_horizontal(test: &RaycastTestData) -> RaycastTestData {
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
fn flip_test_vertical(test: &RaycastTestData) -> RaycastTestData {
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
fn flip_test_both(test: &RaycastTestData) -> RaycastTestData {
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
fn run_single_test(test_data: &RaycastTestData) -> (bool, usize, usize, HashSet<i32>, HashSet<i32>) {
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
fn run_test(test_data: &RaycastTestData) -> (bool, Option<String>, usize, usize) {
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
fn parse_standard_test(path: &Path) -> Result<(Grid, i32, i32, HashSet<i32>), Box<dyn std::error::Error>> {
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
fn flip_standard_horizontal(grid: &Grid, start_x: i32, start_y: i32, expected: &HashSet<i32>) -> (Grid, i32, i32, HashSet<i32>) {
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
fn flip_standard_vertical(grid: &Grid, start_x: i32, start_y: i32, expected: &HashSet<i32>) -> (Grid, i32, i32, HashSet<i32>) {
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
fn flip_standard_both(grid: &Grid, start_x: i32, start_y: i32, expected: &HashSet<i32>) -> (Grid, i32, i32, HashSet<i32>) {
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

/// Visualization state
struct VisState {
    grid: Grid,
    observer_x: i32,
    observer_y: i32,
    visible_cells: HashSet<i32>,
    cell_width: f32,
    cell_height: f32,
}

impl VisState {
    fn new() -> Self {
        let grid = Grid::new(40, 40);
        let observer_x = 20;
        let observer_y = 20;
        let visible_cells = raycast(&grid, observer_x, observer_y);

        VisState {
            grid,
            observer_x,
            observer_y,
            visible_cells,
            cell_width: 10.0,
            cell_height: 7.0,
        }
    }

    fn handle_click(&mut self, mouse_x: f32, mouse_y: f32) {
        let grid_x = (mouse_x / self.cell_width) as i32;
        let grid_y = (mouse_y / self.cell_height) as i32;

        if grid_x >= 0 && grid_x < self.grid.cols && grid_y >= 0 && grid_y < self.grid.rows {
            let cell_id = self.grid.get_id(grid_x, grid_y);

            // Left click: toggle obstacle
            if is_mouse_button_pressed(MouseButton::Left) {
                let current = self.grid.cells[cell_id as usize];
                self.grid.cells[cell_id as usize] = if current == 1 { 0 } else { 1 };
                self.update_visible();
            }
            // Right click: move observer
            else if is_mouse_button_pressed(MouseButton::Right) {
                if !self.grid.is_blocked(grid_x, grid_y) {
                    self.observer_x = grid_x;
                    self.observer_y = grid_y;
                    self.update_visible();
                }
            }
        }
    }

    fn update_visible(&mut self) {
        self.visible_cells = raycast(&self.grid, self.observer_x, self.observer_y);
    }

    fn grid_to_string(&self) -> String {
        let mut result = String::new();

        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let cell_id = self.grid.get_id(x, y);
                let symbol = if x == self.observer_x && y == self.observer_y {
                    's' // Start position
                } else if self.grid.is_blocked(x, y) {
                    '■' // Blocked cell
                } else if self.visible_cells.contains(&cell_id) {
                    'o' // Visible free cell
                } else {
                    '□' // Non-visible free cell
                };
                result.push(symbol);
            }
            result.push('\n');
        }

        result
    }

    fn copy_to_clipboard(&self) {
        let grid_string = self.grid_to_string();
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&grid_string) {
                    println!("Failed to copy to clipboard: {}", e);
                } else {
                    println!("Grid layout copied to clipboard!");
                    // Keep clipboard alive for a moment to ensure clipboard managers can capture it
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            Err(e) => {
                println!("Failed to access clipboard: {}", e);
            }
        }
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(30, 30, 30, 255));

        // Draw grid cells
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let cell_id = self.grid.get_id(x, y);
                let px = x as f32 * self.cell_width;
                let py = y as f32 * self.cell_height;

                let color = if x == self.observer_x && y == self.observer_y {
                    BLUE // Observer
                } else if self.grid.is_blocked(x, y) {
                    RED // Blocked
                } else if self.visible_cells.contains(&cell_id) {
                    Color::from_rgba(100, 200, 100, 255) // Visible
                } else {
                    Color::from_rgba(60, 60, 60, 255) // Not visible
                };

                draw_rectangle(px, py, self.cell_width - 1.0, self.cell_height - 1.0, color);
            }
        }

        // Draw line from observer to mouse cell center
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_grid_x = (mouse_x / self.cell_width) as i32;
        let mouse_grid_y = (mouse_y / self.cell_height) as i32;

        // Only draw line if mouse is within grid bounds
        if mouse_grid_x >= 0 && mouse_grid_x < self.grid.cols && mouse_grid_y >= 0 && mouse_grid_y < self.grid.rows {
            let observer_center_x = self.observer_x as f32 * self.cell_width + self.cell_width / 2.0;
            let observer_center_y = self.observer_y as f32 * self.cell_height + self.cell_height / 2.0;
            let mouse_center_x = mouse_grid_x as f32 * self.cell_width + self.cell_width / 2.0;
            let mouse_center_y = mouse_grid_y as f32 * self.cell_height + self.cell_height / 2.0;

            // Draw center line
            draw_line(observer_center_x, observer_center_y, mouse_center_x, mouse_center_y, 2.0, YELLOW);

            // Calculate which corners to use for edge lines
            let dx = mouse_center_x - observer_center_x;
            let dy = mouse_center_y - observer_center_y;

            if dx != 0.0 || dy != 0.0 {
                // Get all four corners of observer cell
                let obs_left = self.observer_x as f32 * self.cell_width;
                let obs_right = (self.observer_x + 1) as f32 * self.cell_width;
                let obs_top = self.observer_y as f32 * self.cell_height;
                let obs_bottom = (self.observer_y + 1) as f32 * self.cell_height;

                // Get all four corners of mouse cell
                let mouse_left = mouse_grid_x as f32 * self.cell_width;
                let mouse_right = (mouse_grid_x + 1) as f32 * self.cell_width;
                let mouse_top = mouse_grid_y as f32 * self.cell_height;
                let mouse_bottom = (mouse_grid_y + 1) as f32 * self.cell_height;

                // Find which corners are on opposite sides of the center line
                // Using cross product to determine which side each corner is on
                let corners = [
                    ((obs_left, obs_top), (mouse_left, mouse_top)),       // top-left
                    ((obs_right, obs_top), (mouse_right, mouse_top)),     // top-right
                    ((obs_left, obs_bottom), (mouse_left, mouse_bottom)), // bottom-left
                    ((obs_right, obs_bottom), (mouse_right, mouse_bottom)), // bottom-right
                ];

                let mut side_values: Vec<(f32, usize)> = Vec::new();
                for (i, ((ox, oy), _)) in corners.iter().enumerate() {
                    // Cross product with direction vector to determine side
                    let corner_dx = ox - observer_center_x;
                    let corner_dy = oy - observer_center_y;
                    let cross = dx * corner_dy - dy * corner_dx;
                    side_values.push((cross, i));
                }

                // Sort by cross product value
                side_values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                // Draw lines to the two extreme corners (most negative and most positive cross product)
                let corner1 = corners[side_values[0].1];
                let corner2 = corners[side_values[3].1];

                draw_line(corner1.0.0, corner1.0.1, corner1.1.0, corner1.1.1, 1.0, YELLOW);
                draw_line(corner2.0.0, corner2.0.1, corner2.1.0, corner2.1.1, 1.0, YELLOW);
            }
        }

        // Draw info
        let info = format!(
            "Observer: ({}, {})\nVisible cells: {}\nLeft click: toggle obstacle\nRight click: move observer\nC: copy grid to clipboard\nEsc: close window",
            self.observer_x,
            self.observer_y,
            self.visible_cells.len()
        );
        draw_text(&info, 10.0, 20.0, 20.0, WHITE);
    }
}

#[macroquad::main("RustGame3 - Raycasting")]
async fn main() {
    let mut state = VisState::new();

    loop {
        // Handle input
        if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right)
        {
            let (mouse_x, mouse_y) = mouse_position();
            state.handle_click(mouse_x, mouse_y);
        }

        // Copy grid to clipboard on C key
        if is_key_pressed(KeyCode::C) {
            state.copy_to_clipboard();
        }

        // Close window on Escape
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Draw
        state.draw();

        next_frame().await
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn json_validation_tests() {
        let test_dir = "./test_data";
        let mut passed = 0;

        if let Ok(entries) = fs::read_dir(test_dir) {
            let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(test_data) = load_test(&path) {
                        let (all_passed, failed_variant, missing_count, extra_count) = run_test(&test_data);

                        if all_passed {
                            passed += 1;
                        } else {
                            let variant = failed_variant.unwrap_or_else(|| "unknown".to_string());
                            panic!(
                                "Test '{}' failed [{}] (missing: {}, extra: {})",
                                test_data.test_name, variant, missing_count, extra_count
                            );
                        }
                    }
                }
            }
        }

        println!("All {} JSON validation tests passed", passed);
    }

    #[test]
    fn standard_format_tests() {
        let test_dir = "./test_data/standard";
        let mut passed = 0;

        if let Ok(entries) = fs::read_dir(test_dir) {
            let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();

                // Skip markdown files and directories
                if path.extension().and_then(|s| s.to_str()) == Some("md") || path.is_dir() {
                    continue;
                }

                let test_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                match parse_standard_test(&path) {
                    Ok((grid, start_x, start_y, expected_visible)) => {
                        // Test all 4 variants
                        let variants = vec![
                            ("original", (grid.clone(), start_x, start_y, expected_visible.clone())),
                            ("h_flip", flip_standard_horizontal(&grid, start_x, start_y, &expected_visible)),
                            ("v_flip", flip_standard_vertical(&grid, start_x, start_y, &expected_visible)),
                            ("hv_flip", flip_standard_both(&grid, start_x, start_y, &expected_visible)),
                        ];

                        for (variant_name, (variant_grid, variant_x, variant_y, variant_expected)) in variants {
                            let actual_visible = raycast(&variant_grid, variant_x, variant_y);
                            let missing: Vec<_> = variant_expected.difference(&actual_visible).copied().collect();
                            let extra: Vec<_> = actual_visible.difference(&variant_expected).copied().collect();

                            if !missing.is_empty() || !extra.is_empty() {
                                panic!(
                                    "Test '{}' [{}] failed (missing: {}, extra: {})",
                                    test_name, variant_name, missing.len(), extra.len()
                                );
                            }
                        }

                        passed += 1;
                    }
                    Err(e) => {
                        panic!("Test '{}' failed to parse: {}", test_name, e);
                    }
                }
            }
        }

        println!("All {} standard format tests passed", passed);
    }
}
