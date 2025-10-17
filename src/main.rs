mod cone;
mod grid;
mod ray;
mod raycast;

use grid::Grid;
use macroquad::prelude::*;
use raycast::raycast;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Test data structure matching C# JSON export format
#[derive(Debug, Deserialize, Serialize)]
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

/// Run a single test
fn run_test(test_data: &RaycastTestData) -> (bool, usize, usize, HashSet<i32>, HashSet<i32>) {
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

/// Run all tests from the test_data directory
fn run_all_tests() {
    let test_dir = "/home/jan/hobby/RustGame2/test_data";
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    println!("Running raycasting tests from {}\n", test_dir);

    if let Ok(entries) = fs::read_dir(test_dir) {
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(test_data) = load_test(&path) {
                    let (test_passed, missing_count, extra_count, expected, actual) = run_test(&test_data);

                    if test_passed {
                        passed += 1;
                        println!("✓ {}", test_data.test_name);
                    } else {
                        failed += 1;
                        println!(
                            "✗ {} (missing: {}, extra: {})",
                            test_data.test_name, missing_count, extra_count
                        );

                        // Debug first failed test
                        if failures.is_empty() {
                            println!("\n[DEBUG] First failed test: {}", test_data.test_name);
                            println!("Expected {} cells, got {}", expected.len(), actual.len());
                            println!("C# log lines:");
                            for line in &test_data.log {
                                if line.contains("PATH_FINDER") {
                                    println!("  {}", line);
                                }
                            }
                        }

                        failures.push(test_data.test_name.clone());
                    }
                }
            }
        }
    }

    println!("\n========================================");
    println!("Test Results: {} passed, {} failed", passed, failed);
    println!("========================================");

    if !failures.is_empty() {
        println!("\nFailed tests:");
        for name in failures {
            println!("  - {}", name);
        }
    }
}

/// Visualization state
struct VisState {
    grid: Grid,
    observer_x: i32,
    observer_y: i32,
    visible_cells: HashSet<i32>,
    cell_size: f32,
}

impl VisState {
    fn new() -> Self {
        let grid = Grid::new(20, 20);
        let observer_x = 10;
        let observer_y = 10;
        let visible_cells = raycast(&grid, observer_x, observer_y);

        VisState {
            grid,
            observer_x,
            observer_y,
            visible_cells,
            cell_size: 30.0,
        }
    }

    fn handle_click(&mut self, mouse_x: f32, mouse_y: f32) {
        let grid_x = (mouse_x / self.cell_size) as i32;
        let grid_y = (mouse_y / self.cell_size) as i32;

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

    fn draw(&self) {
        clear_background(Color::from_rgba(30, 30, 30, 255));

        // Draw grid cells
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let cell_id = self.grid.get_id(x, y);
                let px = x as f32 * self.cell_size;
                let py = y as f32 * self.cell_size;

                let color = if x == self.observer_x && y == self.observer_y {
                    BLUE // Observer
                } else if self.grid.is_blocked(x, y) {
                    RED // Blocked
                } else if self.visible_cells.contains(&cell_id) {
                    Color::from_rgba(100, 200, 100, 255) // Visible
                } else {
                    Color::from_rgba(60, 60, 60, 255) // Not visible
                };

                draw_rectangle(px, py, self.cell_size - 1.0, self.cell_size - 1.0, color);
            }
        }

        // Draw info
        let info = format!(
            "Observer: ({}, {})\nVisible cells: {}\nLeft click: toggle obstacle\nRight click: move observer\nT: run tests",
            self.observer_x,
            self.observer_y,
            self.visible_cells.len()
        );
        draw_text(&info, 10.0, 20.0, 20.0, WHITE);
    }
}

#[macroquad::main("RustGame3 - Raycasting")]
async fn main() {
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        run_all_tests();
        return;
    }

    let mut state = VisState::new();

    loop {
        // Handle input
        if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right)
        {
            let (mouse_x, mouse_y) = mouse_position();
            state.handle_click(mouse_x, mouse_y);
        }

        // Run tests on T key
        if is_key_pressed(KeyCode::T) {
            println!("\n===== Running Tests =====");
            run_all_tests();
        }

        // Draw
        state.draw();

        next_frame().await
    }
}
