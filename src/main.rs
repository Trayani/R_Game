use arboard::Clipboard;
use macroquad::prelude::*;
use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners, Corner, CornerDirection};
use rustgame3::pathfinding::find_path;
use std::collections::HashSet;

/// Visualization state
struct VisState {
    grid: Grid,
    observer_x: i32,
    observer_y: i32,
    messy_x: bool,
    messy_y: bool,
    visible_cells: HashSet<i32>,
    all_corners: Vec<Corner>,
    interesting_corners: Vec<Corner>,
    cell_width: f32,
    cell_height: f32,
}

impl VisState {
    fn new() -> Self {
        let grid = Grid::new(40, 40);
        let observer_x = 20;
        let observer_y = 20;
        let messy_x = false;
        let messy_y = false;
        let visible_cells = raycast(&grid, observer_x, observer_y, messy_x, messy_y);

        let all_corners = detect_all_corners(&grid);
        let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, observer_x, observer_y, false);

        VisState {
            grid,
            observer_x,
            observer_y,
            messy_x,
            messy_y,
            visible_cells,
            all_corners,
            interesting_corners,
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
                    // Validate messy boundaries - snap to valid position if needed
                    let mut target_x = grid_x;
                    let mut target_y = grid_y;

                    // Check messy X boundary
                    if self.messy_x && target_x >= self.grid.cols - 1 {
                        target_x = self.grid.cols - 2; // Snap to rightmost valid position
                    }

                    // Check messy Y boundary
                    if self.messy_y && target_y >= self.grid.rows - 1 {
                        target_y = self.grid.rows - 2; // Snap to bottommost valid position
                    }

                    // Only move if position changed and target is not blocked
                    if (self.observer_x != target_x || self.observer_y != target_y) && !self.grid.is_blocked(target_x, target_y) {
                        self.observer_x = target_x;
                        self.observer_y = target_y;
                        self.update_visible();
                    }
                }
            }
        }
    }

    fn update_visible(&mut self) {
        self.visible_cells = raycast(&self.grid, self.observer_x, self.observer_y, self.messy_x, self.messy_y);

        // Update corners
        self.all_corners = detect_all_corners(&self.grid);
        self.interesting_corners = filter_interesting_corners(&self.all_corners, &self.visible_cells, &self.grid, self.observer_x, self.observer_y, false);
    }

    fn toggle_messy_x(&mut self) {
        // Check if toggling messy X would put observer out of bounds
        if !self.messy_x && self.observer_x >= self.grid.cols - 1 {
            // Can't enable messy X at rightmost column
            return;
        }
        self.messy_x = !self.messy_x;
        self.update_visible();
    }

    fn toggle_messy_y(&mut self) {
        // Check if toggling messy Y would put observer out of bounds
        if !self.messy_y && self.observer_y >= self.grid.rows - 1 {
            // Can't enable messy Y at bottom row
            return;
        }
        self.messy_y = !self.messy_y;
        self.update_visible();
    }

    /// Check if a given cell is part of the observer
    fn is_observer_cell(&self, x: i32, y: i32) -> bool {
        // Check primary observer cell
        if x == self.observer_x && y == self.observer_y {
            return true;
        }
        // Check messy X cell
        if self.messy_x && x == self.observer_x + 1 && y == self.observer_y {
            return true;
        }
        // Check messy Y cell
        if self.messy_y && x == self.observer_x && y == self.observer_y + 1 {
            return true;
        }
        // Check messy X+Y cell
        if self.messy_x && self.messy_y && x == self.observer_x + 1 && y == self.observer_y + 1 {
            return true;
        }
        false
    }

    fn grid_to_string(&self) -> String {
        let mut result = String::new();

        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let cell_id = self.grid.get_id(x, y);
                let symbol = if self.is_observer_cell(x, y) {
                    's' // Observer position (any cell in messy observer)
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

    fn paste_from_clipboard(&mut self) {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                match clipboard.get_text() {
                    Ok(text) => {
                        match self.parse_grid_from_string(&text) {
                            Ok(_) => {
                                println!("Grid layout pasted from clipboard!");
                                self.update_visible();
                            }
                            Err(e) => {
                                println!("Failed to parse grid: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to read from clipboard: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to access clipboard: {}", e);
            }
        }
    }

    fn parse_grid_from_string(&mut self, text: &str) -> Result<(), String> {
        // Parse lines and collect grid data
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();

        if lines.is_empty() {
            return Err("Empty grid data".to_string());
        }

        let rows = lines.len();
        let cols = lines[0].chars().count();

        // Validate all lines have same width
        for (i, line) in lines.iter().enumerate() {
            let line_width = line.chars().count();
            if line_width != cols {
                return Err(format!("Line {} has width {} but expected {}", i, line_width, cols));
            }
        }

        // Create new grid with parsed dimensions
        let new_grid = Grid::new(cols as i32, rows as i32);
        self.grid = new_grid;

        // Track observer positions (for messy detection)
        let mut observer_positions: Vec<(i32, i32)> = Vec::new();

        // Parse each cell
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let cell_id = self.grid.get_id(x as i32, y as i32);

                match ch {
                    '■' | 'b' => {
                        // Blocked cell
                        self.grid.cells[cell_id as usize] = 1;
                    }
                    's' | 'z' => {
                        // Observer position
                        observer_positions.push((x as i32, y as i32));
                        self.grid.cells[cell_id as usize] = 0; // Observer cell is free
                    }
                    'o' | '□' | 'c' | 'n' | 'u' | 'x' => {
                        // Free cells (various types from test formats)
                        self.grid.cells[cell_id as usize] = 0;
                    }
                    '▲' => {
                        // Interesting corner marker (from test data) - treat as free
                        self.grid.cells[cell_id as usize] = 0;
                    }
                    _ => {
                        // Unknown character - treat as free cell
                        self.grid.cells[cell_id as usize] = 0;
                    }
                }
            }
        }

        // Determine observer position and messy state
        if observer_positions.is_empty() {
            return Err("No observer position (s) found in grid".to_string());
        }

        // Sort positions to identify patterns
        observer_positions.sort();

        // Detect messy configuration
        let (obs_x, obs_y, messy_x, messy_y) = if observer_positions.len() == 1 {
            // Single cell observer
            (observer_positions[0].0, observer_positions[0].1, false, false)
        } else if observer_positions.len() == 2 {
            let (x1, y1) = observer_positions[0];
            let (x2, y2) = observer_positions[1];

            if y1 == y2 && x2 == x1 + 1 {
                // Horizontal adjacency: messy X
                (x1, y1, true, false)
            } else if x1 == x2 && y2 == y1 + 1 {
                // Vertical adjacency: messy Y
                (x1, y1, false, true)
            } else {
                return Err(format!("Observer positions ({}, {}) and ({}, {}) are not adjacent", x1, y1, x2, y2));
            }
        } else if observer_positions.len() == 4 {
            // Should be a 2x2 block for messy X+Y
            let (x1, y1) = observer_positions[0];
            let expected = vec![
                (x1, y1),
                (x1 + 1, y1),
                (x1, y1 + 1),
                (x1 + 1, y1 + 1),
            ];
            let mut sorted_expected = expected.clone();
            sorted_expected.sort();

            if observer_positions == sorted_expected {
                (x1, y1, true, true)
            } else {
                return Err(format!("Observer positions don't form a 2x2 block: {:?}", observer_positions));
            }
        } else {
            return Err(format!("Invalid number of observer positions: {}", observer_positions.len()));
        };

        // Update observer state
        self.observer_x = obs_x;
        self.observer_y = obs_y;
        self.messy_x = messy_x;
        self.messy_y = messy_y;

        Ok(())
    }

    fn draw_corners(&self) {
        let corner_size = 6.0; // Size of corner indicator squares

        // Create a set of interesting corner positions for quick lookup
        let interesting_positions: HashSet<(i32, i32)> =
            self.interesting_corners.iter().map(|c| (c.x, c.y)).collect();

        // Draw all corners
        for corner in &self.all_corners {
            let cell_x = corner.x as f32 * self.cell_width;
            let cell_y = corner.y as f32 * self.cell_height;

            // Determine color: white for interesting, yellow for non-interesting
            let corner_color = if interesting_positions.contains(&(corner.x, corner.y)) {
                WHITE
            } else {
                YELLOW
            };

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

    fn draw_path_list(&self, path: &[rustgame3::pathfinding::Position]) {
        // Build the horizontal path string
        let mut path_parts: Vec<String> = Vec::new();

        for pos in path.iter() {
            let cell_id = self.grid.get_id(pos.x, pos.y);
            path_parts.push(format!("({},{})#{}", pos.x, pos.y, cell_id));
        }

        let path_text = format!("PATH: {}", path_parts.join(" -> "));

        // Calculate position at bottom of grid
        let grid_height = self.grid.rows as f32 * self.cell_height;
        let y_pos = grid_height + 20.0;
        let font_size = 18.0;

        // Draw on a dark background for readability
        let text_width = measure_text(&path_text, None, font_size as u16, 1.0).width;
        draw_rectangle(
            5.0,
            y_pos - 18.0,
            text_width + 10.0,
            25.0,
            Color::from_rgba(20, 20, 20, 220)
        );

        // Draw the path text
        draw_text(&path_text, 10.0, y_pos, font_size, WHITE);
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(30, 30, 30, 255));

        // Draw grid cells
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let cell_id = self.grid.get_id(x, y);
                let px = x as f32 * self.cell_width;
                let py = y as f32 * self.cell_height;

                // Check if this cell is part of the observer
                let is_observer = self.is_observer_cell(x, y);

                let color = if is_observer {
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
            // Calculate observer block bounds (entire messy block, not just one cell)
            let obs_block_left = self.observer_x;
            let obs_block_right = if self.messy_x { self.observer_x + 1 } else { self.observer_x };
            let obs_block_top = self.observer_y;
            let obs_block_bottom = if self.messy_y { self.observer_y + 1 } else { self.observer_y };

            // Observer block center
            let observer_center_x = (obs_block_left as f32 + obs_block_right as f32) / 2.0 * self.cell_width + self.cell_width / 2.0;
            let observer_center_y = (obs_block_top as f32 + obs_block_bottom as f32) / 2.0 * self.cell_height + self.cell_height / 2.0;

            let mouse_center_x = mouse_grid_x as f32 * self.cell_width + self.cell_width / 2.0;
            let mouse_center_y = mouse_grid_y as f32 * self.cell_height + self.cell_height / 2.0;

            // Draw center line
            draw_line(observer_center_x, observer_center_y, mouse_center_x, mouse_center_y, 2.0, YELLOW);

            // Calculate which corners to use for edge lines
            let dx = mouse_center_x - observer_center_x;
            let dy = mouse_center_y - observer_center_y;

            if dx != 0.0 || dy != 0.0 {
                // Get all corners of the ENTIRE observer block (not just one cell)
                let obs_left_px = obs_block_left as f32 * self.cell_width;
                let obs_right_px = (obs_block_right + 1) as f32 * self.cell_width;
                let obs_top_px = obs_block_top as f32 * self.cell_height;
                let obs_bottom_px = (obs_block_bottom + 1) as f32 * self.cell_height;

                // Get all four corners of mouse cell
                let mouse_left = mouse_grid_x as f32 * self.cell_width;
                let mouse_right = (mouse_grid_x + 1) as f32 * self.cell_width;
                let mouse_top = mouse_grid_y as f32 * self.cell_height;
                let mouse_bottom = (mouse_grid_y + 1) as f32 * self.cell_height;

                // Find which corners are on opposite sides of the center line
                // Using cross product to determine which side each corner is on
                let corners = [
                    ((obs_left_px, obs_top_px), (mouse_left, mouse_top)),       // top-left
                    ((obs_right_px, obs_top_px), (mouse_right, mouse_top)),     // top-right
                    ((obs_left_px, obs_bottom_px), (mouse_left, mouse_bottom)), // bottom-left
                    ((obs_right_px, obs_bottom_px), (mouse_right, mouse_bottom)), // bottom-right
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

            // Draw pathfinding path
            if let Some(path) = find_path(&self.grid, self.observer_x, self.observer_y, mouse_grid_x, mouse_grid_y, self.messy_x, self.messy_y) {
                // Draw path lines with white outline and black core
                for i in 1..path.len() {
                    let from = &path[i - 1];
                    let to = &path[i];
                    let from_x = from.x as f32 * self.cell_width + self.cell_width / 2.0;
                    let from_y = from.y as f32 * self.cell_height + self.cell_height / 2.0;
                    let to_x = to.x as f32 * self.cell_width + self.cell_width / 2.0;
                    let to_y = to.y as f32 * self.cell_height + self.cell_height / 2.0;

                    // White outline
                    draw_line(from_x, from_y, to_x, to_y, 6.0, WHITE);
                    // Black core
                    draw_line(from_x, from_y, to_x, to_y, 4.0, BLACK);
                }

                // Draw waypoint circles with white outline and black core
                for (i, pos) in path.iter().enumerate() {
                    let px = pos.x as f32 * self.cell_width + self.cell_width / 2.0;
                    let py = pos.y as f32 * self.cell_height + self.cell_height / 2.0;
                    let radius = if i == 0 || i == path.len() - 1 { 5.0 } else { 4.0 };

                    // White outline
                    draw_circle(px, py, radius + 1.0, WHITE);
                    // Black core
                    draw_circle(px, py, radius, BLACK);
                }

                // Draw path list on the right side
                self.draw_path_list(&path);
            }
        }

        // Draw info
        let messy_status = match (self.messy_x, self.messy_y) {
            (false, false) => String::new(),
            (true, false) => " [Messy X]".to_string(),
            (false, true) => " [Messy Y]".to_string(),
            (true, true) => " [Messy X+Y]".to_string(),
        };

        let info = format!(
            "Observer: ({}, {}){}\nVisible: {} cells\nCorners: {} total, {} interesting\nWhite=interesting, Yellow=non-interesting\nLeft click: toggle obstacle | Right hold: move observer\nM: toggle messy X | N: toggle messy Y\nC: copy grid | V: paste grid | Esc: close",
            self.observer_x,
            self.observer_y,
            messy_status,
            self.visible_cells.len(),
            self.all_corners.len(),
            self.interesting_corners.len()
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

        // Paste grid from clipboard on V key
        if is_key_pressed(KeyCode::V) {
            state.paste_from_clipboard();
        }

        // Toggle messy X on M key
        if is_key_pressed(KeyCode::M) {
            state.toggle_messy_x();
        }

        // Toggle messy Y on N key
        if is_key_pressed(KeyCode::N) {
            state.toggle_messy_y();
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
