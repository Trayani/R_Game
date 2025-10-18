use arboard::Clipboard;
use macroquad::prelude::*;
use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners, Corner, CornerDirection};
use std::collections::HashSet;

/// Visualization state
struct VisState {
    grid: Grid,
    observer_x: i32,
    observer_y: i32,
    visible_cells: HashSet<i32>,
    corners: Vec<Corner>,
    cell_width: f32,
    cell_height: f32,
}

impl VisState {
    fn new() -> Self {
        let grid = Grid::new(40, 40);
        let observer_x = 20;
        let observer_y = 20;
        let visible_cells = raycast(&grid, observer_x, observer_y);

        let all_corners = detect_all_corners(&grid);
        let corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, observer_x, observer_y);

        VisState {
            grid,
            observer_x,
            observer_y,
            visible_cells,
            corners,
            cell_width: 20.0,
            cell_height: 15.0,
        }
    }

    fn handle_mouse(&mut self, mouse_x: f32, mouse_y: f32) {
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
            // Right button DOWN (continuous): move observer
            else if is_mouse_button_down(MouseButton::Right) {
                if !self.grid.is_blocked(grid_x, grid_y) {
                    if self.observer_x != grid_x || self.observer_y != grid_y {
                        self.observer_x = grid_x;
                        self.observer_y = grid_y;
                        self.update_visible();
                    }
                }
            }
        }
    }

    fn update_visible(&mut self) {
        self.visible_cells = raycast(&self.grid, self.observer_x, self.observer_y);

        // Update corners
        let all_corners = detect_all_corners(&self.grid);
        self.corners = filter_interesting_corners(&all_corners, &self.visible_cells, &self.grid, self.observer_x, self.observer_y);
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

    fn draw_corners(&self) {
        let corner_size = 4.0; // Size of corner indicator squares
        let corner_color = ORANGE;

        for corner in &self.corners {
            let cell_x = corner.x as f32 * self.cell_width;
            let cell_y = corner.y as f32 * self.cell_height;

            // Draw a small square at each corner direction
            for &dir in &corner.directions {
                let (offset_x, offset_y) = match dir {
                    CornerDirection::NW => (0.0, 0.0), // Top-left
                    CornerDirection::NE => (self.cell_width - corner_size, 0.0), // Top-right
                    CornerDirection::SW => (0.0, self.cell_height - corner_size), // Bottom-left
                    CornerDirection::SE => (self.cell_width - corner_size, self.cell_height - corner_size), // Bottom-right
                };

                draw_rectangle(
                    cell_x + offset_x,
                    cell_y + offset_y,
                    corner_size,
                    corner_size,
                    corner_color,
                );
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

        // Draw corner indicators
        self.draw_corners();

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
            "Observer: ({}, {})\nVisible: {} cells, {} corners\nLeft click: toggle obstacle\nRight hold: move observer\nC: copy grid\nEsc: close",
            self.observer_x,
            self.observer_y,
            self.visible_cells.len(),
            self.corners.len()
        );
        draw_text(&info, 10.0, 20.0, 20.0, WHITE);
    }
}

#[macroquad::main("RustGame3 - Raycasting")]
async fn main() {
    let mut state = VisState::new();

    loop {
        // Handle input continuously
        let (mouse_x, mouse_y) = mouse_position();
        state.handle_mouse(mouse_x, mouse_y);

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
