use arboard::Clipboard;
use macroquad::prelude::*;
use rustgame3::{Action, ActionLog, Actor, Config, Grid, MovementEvent, raycast, SaveState, SubCellCoord, SubCellReservationManager, spread_cell_destinations};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners, Corner, CornerDirection};
use rustgame3::pathfinding::{find_path, find_path_with_cache, Position};
use std::collections::HashSet;

/// Sub-cell display mode
#[derive(Debug, Clone, Copy, PartialEq)]
enum SubCellMode {
    None,
    Grid1x1,
    Grid2x2,
    Grid3x3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SubCellOffset {
    None,    // No offset - standard grid alignment
    X,       // Offset by 0.5 sub-cells horizontally
    Y,       // Offset by 0.5 sub-cells vertically
    XY,      // Offset by 0.5 sub-cells in both directions
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ReservationMode {
    Square,           // Reserve 2×2 square in movement direction
    Diagonal,         // Diagonal moves require H/V anchor reservation
    NoDiagonal,       // Diagonal moves not allowed (skip diagonal candidates)
    AntiCross,        // Diagonal moves blocked only if same actor owns both counter-diagonal cells
    Basic3,           // Limit to 3 candidates (best + ±45° alternatives)
    Basic3AntiCross,  // Basic3 + anti-cross checking
}

impl ReservationMode {
    fn to_string(&self) -> &'static str {
        match self {
            ReservationMode::Square => "Square",
            ReservationMode::Diagonal => "Diagonal",
            ReservationMode::NoDiagonal => "NoDiagonal",
            ReservationMode::AntiCross => "AntiCross",
            ReservationMode::Basic3 => "Basic3",
            ReservationMode::Basic3AntiCross => "Basic3AntiCross",
        }
    }

    fn next(&self) -> ReservationMode {
        match self {
            ReservationMode::Square => ReservationMode::Diagonal,
            ReservationMode::Diagonal => ReservationMode::NoDiagonal,
            ReservationMode::NoDiagonal => ReservationMode::AntiCross,
            ReservationMode::AntiCross => ReservationMode::Basic3,
            ReservationMode::Basic3 => ReservationMode::Basic3AntiCross,
            ReservationMode::Basic3AntiCross => ReservationMode::Square,
        }
    }
}

impl SubCellOffset {
    fn next(&self) -> Self {
        match self {
            SubCellOffset::None => SubCellOffset::X,
            SubCellOffset::X => SubCellOffset::Y,
            SubCellOffset::Y => SubCellOffset::XY,
            SubCellOffset::XY => SubCellOffset::None,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            SubCellOffset::None => "None",
            SubCellOffset::X => "X",
            SubCellOffset::Y => "Y",
            SubCellOffset::XY => "XY",
        }
    }

    fn get_offsets(&self) -> (f32, f32) {
        match self {
            SubCellOffset::None => (0.0, 0.0),
            SubCellOffset::X => (0.5, 0.0),
            SubCellOffset::Y => (0.0, 0.5),
            SubCellOffset::XY => (0.5, 0.5),
        }
    }
}

impl SubCellMode {
    fn next(&self) -> Self {
        match self {
            SubCellMode::None => SubCellMode::Grid1x1,
            SubCellMode::Grid1x1 => SubCellMode::Grid2x2,
            SubCellMode::Grid2x2 => SubCellMode::Grid3x3,
            SubCellMode::Grid3x3 => SubCellMode::None,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            SubCellMode::None => "None",
            SubCellMode::Grid1x1 => "1x1",
            SubCellMode::Grid2x2 => "2x2",
            SubCellMode::Grid3x3 => "3x3",
        }
    }

    fn grid_size(&self) -> i32 {
        match self {
            SubCellMode::None => 3, // Default to 3x3 when display is off
            SubCellMode::Grid1x1 => 1,
            SubCellMode::Grid2x2 => 2,
            SubCellMode::Grid3x3 => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TrackingMode {
    Disabled,
    Tracking,
}

impl TrackingMode {
    fn toggle(&self) -> TrackingMode {
        match self {
            TrackingMode::Disabled => TrackingMode::Tracking,
            TrackingMode::Tracking => TrackingMode::Disabled,
        }
    }

    fn to_string(&self) -> &'static str {
        match self {
            TrackingMode::Disabled => "Disabled",
            TrackingMode::Tracking => "Tracking",
        }
    }
}

/// Visualization state
struct VisState {
    grid: Grid,
    observer_x: i32,
    observer_y: i32,
    messy_x: bool,
    messy_y: bool,
    destination_x: Option<i32>,
    destination_y: Option<i32>,
    visible_cells: HashSet<i32>,
    all_corners: Vec<Corner>,
    interesting_corners: Vec<Corner>,
    corners_grid_revision: u64,  // Track when corners were last calculated
    cell_width: f32,
    cell_height: f32,
    actors: Vec<Actor>,
    action_log: ActionLog,
    next_actor_id: usize,
    subcell_mode: SubCellMode,
    subcell_offset: SubCellOffset,  // Offset for sub-cell grid alignment
    subcell_movement_enabled: bool,
    subcell_reservation_manager: SubCellReservationManager,
    show_subcell_markers: bool,  // Toggle for green/yellow sub-cell debug markers
    reservation_mode: ReservationMode,  // Square or Diagonal reservation strategy
    early_reservation_enabled: bool,  // If true, reserve immediately after switching current
    filter_backward_moves: bool,  // If true, filter out candidates that move away from destination
    // Random subset destination feature
    highlighted_actors: HashSet<usize>,
    highlight_timer: f32,
    // Movement tracking
    tracking_mode: TrackingMode,  // Toggle for recording actor movement paths
    // Historical tracking data - preserved even when actors are cleared
    historical_tracks: Vec<Vec<(f32, f32)>>,  // Stores tracks from cleared actors
    // Actor configuration
    actor_speed: f32,  // Default actor speed in pixels/second
    actor_size_ratio: f32,  // Actor size as ratio of cell size
    actor_collision_radius_ratio: f32,  // Collision radius as ratio of cell size
}

impl VisState {
    fn new(config: &Config) -> Self {
        // Create initial grid with configured dimensions
        let mut grid = Grid::new(config.grid.cols, config.grid.rows);
        let mut observer_x = config.observer.x;
        let mut observer_y = config.observer.y;
        let mut messy_x = config.observer.messy_x;
        let mut messy_y = config.observer.messy_y;

        // Try to load default grid layout from configured file path
        if !config.default_grid_file.path.is_empty() {
            if let Ok(default_layout) = std::fs::read_to_string(&config.default_grid_file.path) {
            // Parse the default layout
            if let Ok((parsed_grid, obs_x, obs_y, m_x, m_y)) = Self::parse_grid_layout(&default_layout) {
                grid = parsed_grid;
                observer_x = obs_x;
                observer_y = obs_y;
                messy_x = m_x;
                messy_y = m_y;
            } else {
                    eprintln!("Warning: Failed to parse {}, using empty grid", config.default_grid_file.path);
                }
            }
        }

        let visible_cells = raycast(&grid, observer_x, observer_y, messy_x, messy_y);

        let all_corners = detect_all_corners(&grid);
        let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, &grid, observer_x, observer_y, false);
        let corners_grid_revision = grid.get_revision();

        // Parse sub-cell display mode from config
        let (subcell_mode, subcell_grid_size) = match config.subcell.display_mode.to_lowercase().as_str() {
            "1x1" | "1" => (SubCellMode::Grid1x1, 1),
            "2x2" | "2" => (SubCellMode::Grid2x2, 2),
            "3x3" | "3" => (SubCellMode::Grid3x3, 3),
            _ => (SubCellMode::None, 3), // Default to 3x3 for grid size even if display is off
        };

        VisState {
            grid,
            observer_x,
            observer_y,
            messy_x,
            messy_y,
            destination_x: None,
            destination_y: None,
            visible_cells,
            all_corners,
            interesting_corners,
            corners_grid_revision,
            cell_width: config.grid.cell_width,
            cell_height: config.grid.cell_height,
            actors: Vec::new(),
            action_log: ActionLog::new(),
            next_actor_id: 0,
            subcell_mode,
            subcell_offset: SubCellOffset::None,  // No offset by default
            subcell_movement_enabled: config.subcell.movement_enabled,
            subcell_reservation_manager: SubCellReservationManager::new(subcell_grid_size),
            show_subcell_markers: config.subcell.show_markers,
            reservation_mode: ReservationMode::Square,  // Square by default
            early_reservation_enabled: false,  // Disabled by default
            filter_backward_moves: true,  // Enabled by default
            highlighted_actors: HashSet::new(),
            highlight_timer: 0.0,
            tracking_mode: TrackingMode::Disabled,  // Disabled by default
            historical_tracks: Vec::new(),
            actor_speed: config.actors.default_speed,
            actor_size_ratio: config.actors.size_ratio,
            actor_collision_radius_ratio: config.actors.collision_radius_ratio,
        }
    }

    /// Sort actors by distance to target position, return indices
    /// Closest actors appear first in the returned vector
    fn actors_sorted_by_distance(&self, target_x: f32, target_y: f32) -> Vec<usize> {
        let mut actor_distances: Vec<(usize, f32)> = self.actors
            .iter()
            .enumerate()
            .map(|(idx, actor)| {
                let dx = actor.fpos_x - target_x;
                let dy = actor.fpos_y - target_y;
                let distance = (dx * dx + dy * dy).sqrt();
                (idx, distance)
            })
            .collect();

        // Sort by distance (ascending - closest first)
        actor_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Return just the indices
        actor_distances.iter().map(|(idx, _)| *idx).collect()
    }

    fn handle_mouse(&mut self, mouse_x: f32, mouse_y: f32) {
        let grid_x = (mouse_x / self.cell_width) as i32;
        let grid_y = (mouse_y / self.cell_height) as i32;

        if grid_x >= 0 && grid_x < self.grid.cols && grid_y >= 0 && grid_y < self.grid.rows {
            let cell_id = self.grid.get_id(grid_x, grid_y);

            // Shift + Left hold: set cell as blocked (drawing mode)
            if is_key_down(KeyCode::LeftShift) && is_mouse_button_down(MouseButton::Left) {
                if self.grid.cells[cell_id as usize] != 1 {
                    self.action_log.log_start(Action::SetBlocked { x: grid_x, y: grid_y });
                    self.grid.set_cell(grid_x, grid_y, 1);
                    self.update_visible();
                    self.action_log.log_finish(Action::SetBlocked { x: grid_x, y: grid_y });
                }
            }
            // Shift + Right hold: set cell as free (erasing mode)
            else if is_key_down(KeyCode::LeftShift) && is_mouse_button_down(MouseButton::Right) {
                if self.grid.cells[cell_id as usize] != 0 {
                    self.action_log.log_start(Action::SetFree { x: grid_x, y: grid_y });
                    self.grid.set_cell(grid_x, grid_y, 0);
                    self.update_visible();
                    self.action_log.log_finish(Action::SetFree { x: grid_x, y: grid_y });
                }
            }
            // Left click (without shift): toggle obstacle
            else if is_mouse_button_pressed(MouseButton::Left) {
                self.action_log.log_start(Action::ToggleCell { x: grid_x, y: grid_y });
                let current = self.grid.cells[cell_id as usize];
                self.grid.set_cell(grid_x, grid_y, if current == 1 { 0 } else { 1 });
                self.update_visible();
                self.action_log.log_finish(Action::ToggleCell { x: grid_x, y: grid_y });
            }
            // Right button DOWN (without shift, continuous): move observer
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
                        self.action_log.log_start(Action::MoveObserver {
                            x: target_x,
                            y: target_y,
                            messy_x: self.messy_x,
                            messy_y: self.messy_y,
                        });
                        self.observer_x = target_x;
                        self.observer_y = target_y;
                        self.update_visible();
                        self.action_log.log_finish(Action::MoveObserver {
                            x: target_x,
                            y: target_y,
                            messy_x: self.messy_x,
                            messy_y: self.messy_y,
                        });
                    }
                }
            }
        }
    }

    fn update_visible(&mut self) {
        self.visible_cells = raycast(&self.grid, self.observer_x, self.observer_y, self.messy_x, self.messy_y);

        // Only recalculate corners if grid has changed
        let current_grid_revision = self.grid.get_revision();
        if self.corners_grid_revision != current_grid_revision {
            self.all_corners = detect_all_corners(&self.grid);
            self.corners_grid_revision = current_grid_revision;
        }

        // Always recalculate interesting corners (depends on visibility, which changes with observer position)
        self.interesting_corners = filter_interesting_corners(&self.all_corners, &self.visible_cells, &self.grid, self.observer_x, self.observer_y, false);
    }

    fn toggle_messy_x(&mut self) {
        // Check if toggling messy X would put observer out of bounds
        if !self.messy_x && self.observer_x >= self.grid.cols - 1 {
            // Can't enable messy X at rightmost column
            return;
        }
        self.action_log.log_start(Action::ToggleMessyX);
        self.messy_x = !self.messy_x;
        self.update_visible();
        self.action_log.log_finish(Action::ToggleMessyX);
    }

    fn toggle_messy_y(&mut self) {
        // Check if toggling messy Y would put observer out of bounds
        if !self.messy_y && self.observer_y >= self.grid.rows - 1 {
            // Can't enable messy Y at bottom row
            return;
        }
        self.action_log.log_start(Action::ToggleMessyY);
        self.messy_y = !self.messy_y;
        self.update_visible();
        self.action_log.log_finish(Action::ToggleMessyY);
    }

    fn toggle_subcell_mode(&mut self) {
        self.subcell_mode = self.subcell_mode.next();
        let new_grid_size = self.subcell_mode.grid_size();

        // Update reservation manager grid size (clears all reservations)
        self.subcell_reservation_manager.set_grid_size(new_grid_size);

        // Update all actors to use new grid size and re-orient their sub-cell positions
        for actor in &mut self.actors {
            actor.subcell_grid_size = new_grid_size;
            // Re-calculate current sub-cell position with new grid size
            actor.current_subcell = Some(SubCellCoord::from_screen_pos(
                actor.fpos_x,
                actor.fpos_y,
                self.cell_width,
                self.cell_height,
                new_grid_size,
            ));
            // Clear reserved sub-cell (will be recalculated on next update)
            actor.reserved_subcell = None;
        }

        println!("Sub-cell display mode: {} (grid size: {}x{})",
                 self.subcell_mode.to_string(), new_grid_size, new_grid_size);
    }

    fn toggle_subcell_movement(&mut self) {
        self.subcell_movement_enabled = !self.subcell_movement_enabled;

        if self.subcell_movement_enabled {
            // Enable sub-cell movement mode
            println!("Sub-cell movement: ENABLED");
            // Clear any existing paths and switch to sub-cell destinations
            for actor in &mut self.actors {
                if let Some(dest) = actor.destination {
                    actor.set_subcell_destination(dest);
                    actor.clear_path();
                }
            }
        } else {
            // Disable sub-cell movement mode
            println!("Sub-cell movement: DISABLED");
            // Clear sub-cell reservations
            self.subcell_reservation_manager.clear();
            // Clear sub-cell destinations from actors
            for actor in &mut self.actors {
                actor.subcell_destination = None;
                actor.reserved_subcell = None;
            }
        }
    }

    fn set_destination(&mut self, x: i32, y: i32) {
        if x >= 0 && x < self.grid.cols && y >= 0 && y < self.grid.rows {
            self.action_log.log_start(Action::SetObserverDestination { x, y });
            self.destination_x = Some(x);
            self.destination_y = Some(y);

            // Print path to terminal for text selection
            if let Some(path) = find_path(&self.grid, self.observer_x, self.observer_y, x, y, self.messy_x, self.messy_y) {
                let path_parts: Vec<String> = path.iter()
                    .map(|pos| {
                        let cell_id = self.grid.get_id(pos.x, pos.y);
                        format!("({},{})#{}", pos.x, pos.y, cell_id)
                    })
                    .collect();
                println!("PATH: {}", path_parts.join(" -> "));
            } else {
                println!("No path found to ({}, {})", x, y);
            }
            self.action_log.log_finish(Action::SetObserverDestination { x, y });
        }
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
                        // Log start before parsing
                        self.action_log.log_start(Action::PasteGrid {
                            rows: 0,  // Will be updated after parse
                            cols: 0,
                        });
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

    /// Parse grid layout from string (static version for initialization)
    /// Returns (Grid, observer_x, observer_y, messy_x, messy_y)
    fn parse_grid_layout(text: &str) -> Result<(Grid, i32, i32, bool, bool), String> {
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
        let mut grid = Grid::new(cols as i32, rows as i32);

        // Track observer positions (for messy detection)
        let mut observer_positions: Vec<(i32, i32)> = Vec::new();

        // Parse each cell
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let cell_id = grid.get_id(x as i32, y as i32);

                match ch {
                    '■' | 'b' => {
                        // Blocked cell
                        grid.cells[cell_id as usize] = 1;
                    }
                    's' | 'z' => {
                        // Observer position
                        observer_positions.push((x as i32, y as i32));
                        grid.cells[cell_id as usize] = 0; // Observer cell is free
                    }
                    'o' | '□' | 'c' | 'n' | 'u' | 'x' => {
                        // Free cells (various types from test formats)
                        grid.cells[cell_id as usize] = 0;
                    }
                    '▲' => {
                        // Interesting corner marker (from test data) - treat as free
                        grid.cells[cell_id as usize] = 0;
                    }
                    _ => {
                        // Unknown character - treat as free cell
                        grid.cells[cell_id as usize] = 0;
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

        Ok((grid, obs_x, obs_y, messy_x, messy_y))
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

        // Increment grid revision since we modified cells
        self.grid.revision += 1;

        // Log the paste action finish
        self.action_log.log_finish(Action::PasteGrid {
            rows: self.grid.rows,
            cols: self.grid.cols,
        });

        Ok(())
    }

    /// Find an available destination cell near the target position
    /// Tries the target first, then expands in a spiral pattern to find a free cell
    fn find_available_destination(
        &self,
        target_x: i32,
        target_y: i32,
        occupied: &HashSet<(i32, i32)>,
    ) -> (i32, i32) {
        // Check if target is available
        if !occupied.contains(&(target_x, target_y)) && !self.grid.is_blocked(target_x, target_y) {
            return (target_x, target_y);
        }

        // Spiral search pattern: expand outward in concentric squares
        for radius in 1..=10 {
            // Check cells in a square pattern at this radius
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    // Only check the perimeter (not interior cells we already checked)
                    if (dx as i32).abs() != radius && (dy as i32).abs() != radius {
                        continue;
                    }

                    let check_x = target_x + dx;
                    let check_y = target_y + dy;

                    // Bounds check
                    if check_x < 0 || check_x >= self.grid.cols || check_y < 0 || check_y >= self.grid.rows {
                        continue;
                    }

                    // Check if available
                    if !occupied.contains(&(check_x, check_y)) && !self.grid.is_blocked(check_x, check_y) {
                        return (check_x, check_y);
                    }
                }
            }
        }

        // Fallback: return target even if occupied (pathfinding will fail naturally)
        (target_x, target_y)
    }

    fn draw_subcells(&self) {
        // Don't draw sub-cells if mode is None
        if self.subcell_mode == SubCellMode::None {
            return;
        }

        let subdivisions = match self.subcell_mode {
            SubCellMode::None => return,
            SubCellMode::Grid1x1 => 1,
            SubCellMode::Grid2x2 => 2,
            SubCellMode::Grid3x3 => 3,
        };

        // Semi-transparent white color for sub-cell lines
        let line_color = Color::from_rgba(255, 255, 255, 80);
        let line_thickness = 1.0;

        // Draw sub-cell lines for each cell
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                let px = x as f32 * self.cell_width;
                let py = y as f32 * self.cell_height;

                // Draw vertical lines within the cell
                for i in 1..subdivisions {
                    let offset_x = px + (i as f32 * self.cell_width / subdivisions as f32);
                    draw_line(
                        offset_x,
                        py,
                        offset_x,
                        py + self.cell_height - 1.0,
                        line_thickness,
                        line_color,
                    );
                }

                // Draw horizontal lines within the cell
                for i in 1..subdivisions {
                    let offset_y = py + (i as f32 * self.cell_height / subdivisions as f32);
                    draw_line(
                        px,
                        offset_y,
                        px + self.cell_width - 1.0,
                        offset_y,
                        line_thickness,
                        line_color,
                    );
                }
            }
        }
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

    fn draw_actor(&self) {
        // Draw historical tracks first (from cleared actors)
        if self.tracking_mode == TrackingMode::Tracking {
            for track in &self.historical_tracks {
                if !track.is_empty() {
                    // Draw thin black lines between consecutive tracked positions
                    for i in 1..track.len() {
                        let (x1, y1) = track[i - 1];
                        let (x2, y2) = track[i];
                        draw_line(x1, y1, x2, y2, 1.0, BLACK); // Thin black trail
                    }

                    // Draw small black dots at tracked positions
                    for &(x, y) in track {
                        draw_circle(x, y, 1.5, BLACK); // Small black dots
                    }
                }
            }
        }

        for actor in &self.actors {
            // Draw movement track (walked path) if tracking is enabled
            if self.tracking_mode == TrackingMode::Tracking && !actor.movement_track.is_empty() {
                // Draw thin black lines between consecutive tracked positions
                for i in 1..actor.movement_track.len() {
                    let (x1, y1) = actor.movement_track[i - 1];
                    let (x2, y2) = actor.movement_track[i];
                    draw_line(x1, y1, x2, y2, 1.0, BLACK); // Thin black trail
                }

                // Draw small black dots at tracked positions
                for &(x, y) in &actor.movement_track {
                    draw_circle(x, y, 1.5, BLACK); // Small black dots
                }
            }

            let (left, top, right, bottom) = actor.get_bounds();

            // Draw actor square with semi-transparent purple fill
            let actor_width = right - left;
            let actor_height = bottom - top;
            draw_rectangle(left, top, actor_width, actor_height, Color::from_rgba(200, 100, 255, 150));

            // Draw border
            draw_rectangle_lines(left, top, actor_width, actor_height, 2.0, BLACK);

            // Draw center point
            draw_circle(actor.fpos_x, actor.fpos_y, 3.0, MAGENTA);

            // Draw sub-cell positions if in sub-cell movement mode AND markers enabled
            if self.subcell_movement_enabled && self.show_subcell_markers {
                let (offset_x, offset_y) = self.subcell_offset.get_offsets();

                // Draw current sub-cell (green)
                if let Some(current_sc) = actor.current_subcell {
                    let (cx, cy) = current_sc.to_screen_center_with_offset(self.cell_width, self.cell_height, offset_x, offset_y);
                    draw_circle(cx, cy, 4.0, GREEN);
                    draw_circle_lines(cx, cy, 6.0, 1.5, GREEN);
                }

                // Draw reserved sub-cell (yellow)
                if let Some(reserved_sc) = actor.reserved_subcell {
                    let (rx, ry) = reserved_sc.to_screen_center_with_offset(self.cell_width, self.cell_height, offset_x, offset_y);
                    draw_circle(rx, ry, 4.0, YELLOW);
                    draw_circle_lines(rx, ry, 6.0, 1.5, YELLOW);

                    // Draw line from actor to reserved sub-cell
                    draw_line(actor.fpos_x, actor.fpos_y, rx, ry, 1.5, YELLOW);
                }

                // Draw extra reserved sub-cells (from square reservation) with black lines
                for extra_sc in &actor.extra_reserved_subcells {
                    let (ex, ey) = extra_sc.to_screen_center_with_offset(self.cell_width, self.cell_height, offset_x, offset_y);
                    draw_circle(ex, ey, 3.0, BLACK);
                    draw_circle_lines(ex, ey, 5.0, 1.5, BLACK);

                    // Draw black line from actor to extra reserved sub-cell
                    draw_line(actor.fpos_x, actor.fpos_y, ex, ey, 1.5, BLACK);
                }

                // Draw destination
                if let Some(dest) = actor.subcell_destination {
                    let dest_x = dest.x as f32 * self.cell_width + self.cell_width / 2.0;
                    let dest_y = dest.y as f32 * self.cell_height + self.cell_height / 2.0;
                    draw_circle(dest_x, dest_y, 6.0, ORANGE);
                    draw_circle_lines(dest_x, dest_y, 8.0, 2.0, ORANGE);
                }
            }

            // Draw highlight ring if this actor is selected (random subset feature)
            if self.highlight_timer > 0.0 && self.highlighted_actors.contains(&actor.id) {
                let (left, top, right, bottom) = actor.get_bounds();
                let center_x = (left + right) / 2.0;
                let center_y = (top + bottom) / 2.0;
                let radius = ((right - left) / 2.0).max((bottom - top) / 2.0) + 8.0;

                // Draw pulsing orange/yellow highlight ring
                let pulse = (self.highlight_timer * 3.0).sin() * 0.3 + 0.7; // Pulse between 0.4 and 1.0
                let highlight_color = Color::from_rgba(255, 200, 0, (pulse * 255.0) as u8);
                draw_circle_lines(center_x, center_y, radius, 3.0, highlight_color);
            }

            // If actor has a path, draw the path and destination
            if actor.has_path() {
                // Draw path waypoints
                for i in 0..actor.path.len() {
                    let pos = &actor.path[i];
                    let px = pos.x as f32 * self.cell_width + self.cell_width / 2.0;
                    let py = pos.y as f32 * self.cell_height + self.cell_height / 2.0;

                    // Highlight current waypoint differently
                    if i == actor.current_waypoint {
                        draw_circle(px, py, 6.0, MAGENTA);
                    } else if i < actor.current_waypoint {
                        // Already visited waypoints
                        draw_circle(px, py, 4.0, Color::from_rgba(150, 50, 150, 100));
                    } else {
                        // Future waypoints
                        draw_circle(px, py, 4.0, Color::from_rgba(200, 100, 255, 150));
                    }

                    // Draw lines between waypoints
                    if i > 0 {
                        let prev_pos = &actor.path[i - 1];
                        let prev_px = prev_pos.x as f32 * self.cell_width + self.cell_width / 2.0;
                        let prev_py = prev_pos.y as f32 * self.cell_height + self.cell_height / 2.0;

                        let line_color = if i <= actor.current_waypoint {
                            Color::from_rgba(150, 50, 150, 100) // Visited path
                        } else {
                            Color::from_rgba(200, 100, 255, 150) // Future path
                        };
                        draw_line(prev_px, prev_py, px, py, 2.0, line_color);
                    }
                }

                // Draw line from actor to current waypoint
                if actor.get_path_destination().is_some() {
                    if actor.current_waypoint < actor.path.len() {
                        let waypoint = &actor.path[actor.current_waypoint];
                        let wx = waypoint.x as f32 * self.cell_width + self.cell_width / 2.0;
                        let wy = waypoint.y as f32 * self.cell_height + self.cell_height / 2.0;
                        draw_line(actor.fpos_x, actor.fpos_y, wx, wy, 2.0, MAGENTA);
                    }
                }
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

        // Draw sub-cell grids
        self.draw_subcells();

        // Draw corner indicators
        self.draw_corners();

        // Get mouse position
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_grid_x = (mouse_x / self.cell_width) as i32;
        let mouse_grid_y = (mouse_y / self.cell_height) as i32;

        // Draw mouse cell highlight (light overlay)
        if mouse_grid_x >= 0 && mouse_grid_x < self.grid.cols && mouse_grid_y >= 0 && mouse_grid_y < self.grid.rows {
            let px = mouse_grid_x as f32 * self.cell_width;
            let py = mouse_grid_y as f32 * self.cell_height;
            draw_rectangle(px, py, self.cell_width - 1.0, self.cell_height - 1.0, Color::from_rgba(255, 255, 255, 50));
        }

        // Draw destination cell highlight (if set)
        if let (Some(dest_x), Some(dest_y)) = (self.destination_x, self.destination_y) {
            let px = dest_x as f32 * self.cell_width;
            let py = dest_y as f32 * self.cell_height;
            // Draw orange/yellow border for destination
            draw_rectangle_lines(px, py, self.cell_width - 1.0, self.cell_height - 1.0, 3.0, ORANGE);
        }

        // Draw pathfinding path to destination (if set)
        if let (Some(dest_x), Some(dest_y)) = (self.destination_x, self.destination_y) {
            // Calculate observer block bounds
            let obs_block_left = self.observer_x;
            let obs_block_right = if self.messy_x { self.observer_x + 1 } else { self.observer_x };
            let obs_block_top = self.observer_y;
            let obs_block_bottom = if self.messy_y { self.observer_y + 1 } else { self.observer_y };

            // Observer block center
            let observer_center_x = (obs_block_left as f32 + obs_block_right as f32) / 2.0 * self.cell_width + self.cell_width / 2.0;
            let observer_center_y = (obs_block_top as f32 + obs_block_bottom as f32) / 2.0 * self.cell_height + self.cell_height / 2.0;

            let dest_center_x = dest_x as f32 * self.cell_width + self.cell_width / 2.0;
            let dest_center_y = dest_y as f32 * self.cell_height + self.cell_height / 2.0;

            // Draw center line to destination
            draw_line(observer_center_x, observer_center_y, dest_center_x, dest_center_y, 2.0, YELLOW);

            // Calculate which corners to use for edge lines
            let dx = dest_center_x - observer_center_x;
            let dy = dest_center_y - observer_center_y;

            if dx != 0.0 || dy != 0.0 {
                // Get all corners of the ENTIRE observer block
                let obs_left_px = obs_block_left as f32 * self.cell_width;
                let obs_right_px = (obs_block_right + 1) as f32 * self.cell_width;
                let obs_top_px = obs_block_top as f32 * self.cell_height;
                let obs_bottom_px = (obs_block_bottom + 1) as f32 * self.cell_height;

                // Get all four corners of destination cell
                let dest_left = dest_x as f32 * self.cell_width;
                let dest_right = (dest_x + 1) as f32 * self.cell_width;
                let dest_top = dest_y as f32 * self.cell_height;
                let dest_bottom = (dest_y + 1) as f32 * self.cell_height;

                let corners = [
                    ((obs_left_px, obs_top_px), (dest_left, dest_top)),
                    ((obs_right_px, obs_top_px), (dest_right, dest_top)),
                    ((obs_left_px, obs_bottom_px), (dest_left, dest_bottom)),
                    ((obs_right_px, obs_bottom_px), (dest_right, dest_bottom)),
                ];

                let mut side_values: Vec<(f32, usize)> = Vec::new();
                for (i, ((ox, oy), _)) in corners.iter().enumerate() {
                    let corner_dx = ox - observer_center_x;
                    let corner_dy = oy - observer_center_y;
                    let cross = dx * corner_dy - dy * corner_dx;
                    side_values.push((cross, i));
                }

                side_values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                let corner1 = corners[side_values[0].1];
                let corner2 = corners[side_values[3].1];

                draw_line(corner1.0.0, corner1.0.1, corner1.1.0, corner1.1.1, 1.0, YELLOW);
                draw_line(corner2.0.0, corner2.0.1, corner2.1.0, corner2.1.1, 1.0, YELLOW);
            }

            // Draw pathfinding path
            if let Some(path) = find_path(&self.grid, self.observer_x, self.observer_y, dest_x, dest_y, self.messy_x, self.messy_y) {
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

                // Draw path list at bottom
                self.draw_path_list(&path);
            }
        }

        // Draw actor on top of everything
        self.draw_actor();

        // Draw info
        let messy_status = match (self.messy_x, self.messy_y) {
            (false, false) => String::new(),
            (true, false) => " [Messy X]".to_string(),
            (false, true) => " [Messy Y]".to_string(),
            (true, true) => " [Messy X+Y]".to_string(),
        };

        let dest_status = if let (Some(dx), Some(dy)) = (self.destination_x, self.destination_y) {
            format!(" | Dest: ({}, {})", dx, dy)
        } else {
            String::new()
        };

        let actor_status = if !self.actors.is_empty() {
            format!(" | Actors: {}", self.actors.len())
        } else {
            String::new()
        };

        let subcell_status = format!(" | SubCell: {}", self.subcell_mode.to_string());
        let subcell_movement_status = if self.subcell_movement_enabled {
            format!(" | SubCell Movement: ON (reservations: {})", self.subcell_reservation_manager.reservation_count())
        } else {
            " | SubCell Movement: OFF".to_string()
        };

        let info = format!(
            "Observer: ({}, {}){}{}{}{}{}\nVisible: {} cells\nCorners: {} total, {} interesting\nWhite=interesting, Yellow=non-interesting\nLeft click: toggle | Shift+Left hold: draw walls | Shift+Right hold: erase walls\nRight hold: move observer | D: set destination | G: toggle sub-cell grid (None/2x2/3x3) | T: toggle sub-cell offset (None/X/Y/XY)\nM: toggle messy X | N: toggle messy Y | S: toggle sub-cell movement | B: toggle markers | Q: cycle reservation (Square/Diagonal/NoDiagonal/AntiCross) | E: toggle early reservation | F: toggle backward filter\nO: spawn actor | 0: clear all actors | L: toggle tracking ({}) | P: set destination (all) | R: random subset (30%, closest) | C: copy | V: paste | F5: save state | F9: load state | Esc: close",
            self.observer_x,
            self.observer_y,
            messy_status,
            dest_status,
            actor_status,
            subcell_status,
            subcell_movement_status,
            self.visible_cells.len(),
            self.all_corners.len(),
            self.interesting_corners.len(),
            self.tracking_mode.to_string()
        );
        draw_text(&info, 10.0, 20.0, 20.0, WHITE);
    }
}

#[macroquad::main("RustGame3 - Raycasting")]
async fn main() {
    // Load configuration from config.toml (or use defaults)
    let config = Config::load();

    let mut state = VisState::new(&config);

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

        // Save state on F5 key
        if is_key_pressed(KeyCode::F5) {
            let save_state = SaveState::from_grid_and_actors(&state.grid, &state.actors, state.cell_width, state.cell_height);
            match save_state.save_to_file("save_state.json") {
                Ok(_) => println!("State saved to save_state.json ({} actors, {} blocked cells)",
                    state.actors.len(), save_state.blocked_cells.len()),
                Err(e) => eprintln!("Failed to save state: {}", e),
            }
        }

        // Load state on F9 key
        if is_key_pressed(KeyCode::F9) {
            match SaveState::load_from_file("save_state.json") {
                Ok(save_state) => {
                    // Clear current movement state
                    state.subcell_reservation_manager.clear();

                    // Restore grid
                    state.grid = save_state.restore_grid();

                    // Restore actors with configured speed (not saved speed)
                    let subcell_grid_size = state.subcell_reservation_manager.grid_size();
                    let (offset_x, offset_y) = state.subcell_offset.get_offsets();
                    state.actors = save_state.restore_actors(state.cell_width, state.cell_height, subcell_grid_size, offset_x, offset_y, state.actor_speed);

                    // Update next_actor_id to avoid ID conflicts
                    state.next_actor_id = state.actors.iter().map(|a| a.id).max().unwrap_or(0) + 1;

                    // Recalculate corners
                    state.all_corners = detect_all_corners(&state.grid);
                    state.corners_grid_revision = state.grid.get_revision();
                    let visible_cells = raycast(&state.grid, state.observer_x, state.observer_y, state.messy_x, state.messy_y);
                    state.interesting_corners = filter_interesting_corners(&state.all_corners, &visible_cells, &state.grid, state.observer_x, state.observer_y, false);
                    state.visible_cells = visible_cells;

                    println!("State loaded from save_state.json ({} actors, grid {}x{})",
                        state.actors.len(), save_state.grid_cols, save_state.grid_rows);
                }
                Err(e) => eprintln!("Failed to load state: {}", e),
            }
        }

        // Toggle messy X on M key
        if is_key_pressed(KeyCode::M) {
            state.toggle_messy_x();
        }

        // Toggle messy Y on N key
        if is_key_pressed(KeyCode::N) {
            state.toggle_messy_y();
        }

        // Toggle sub-cell display mode on G key
        if is_key_pressed(KeyCode::G) {
            state.toggle_subcell_mode();
        }

        // Toggle sub-cell movement on S key
        if is_key_pressed(KeyCode::S) {
            state.toggle_subcell_movement();
        }

        // Toggle sub-cell markers (green/yellow circles) on B key
        if is_key_pressed(KeyCode::B) {
            state.show_subcell_markers = !state.show_subcell_markers;
            println!("Sub-cell markers: {}", if state.show_subcell_markers { "ON" } else { "OFF" });
        }

        // Cycle reservation mode (Square → Diagonal → NoDiagonal → AntiCross → Basic3 → Basic3AntiCross → Square) on Q key
        if is_key_pressed(KeyCode::Q) {
            state.reservation_mode = state.reservation_mode.next();
            println!("Reservation Mode: {}", state.reservation_mode.to_string());
        }

        // Toggle early reservation on E key
        if is_key_pressed(KeyCode::E) {
            state.early_reservation_enabled = !state.early_reservation_enabled;
            println!("Early Reservation: {}", if state.early_reservation_enabled { "ON" } else { "OFF" });
        }

        // Toggle backward move filter on F key
        if is_key_pressed(KeyCode::F) {
            state.filter_backward_moves = !state.filter_backward_moves;
            println!("Filter Backward Moves: {}", if state.filter_backward_moves { "ON" } else { "OFF" });
        }

        // Toggle sub-cell offset on T key (cycle through None, X, Y, XY)
        if is_key_pressed(KeyCode::T) {
            state.subcell_offset = state.subcell_offset.next();
            println!("SubCell Offset: {}", state.subcell_offset.to_string());
            // Update all existing actors with the new offset
            let (offset_x, offset_y) = state.subcell_offset.get_offsets();
            for actor in &mut state.actors {
                actor.subcell_offset_x = offset_x;
                actor.subcell_offset_y = offset_y;
                // Recalculate current sub-cell with new offset
                if actor.current_subcell.is_some() {
                    actor.current_subcell = Some(SubCellCoord::from_screen_pos_with_offset(
                        actor.fpos_x,
                        actor.fpos_y,
                        actor.cell_width,
                        actor.cell_height,
                        actor.subcell_grid_size,
                        offset_x,
                        offset_y,
                    ));
                }
            }
        }

        // Set destination on D key (to current mouse position)
        if is_key_pressed(KeyCode::D) {
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_grid_x = (mouse_x / state.cell_width) as i32;
            let mouse_grid_y = (mouse_y / state.cell_height) as i32;
            state.set_destination(mouse_grid_x, mouse_grid_y);
        }

        // Place actor at mouse position on O key (Shift+O spawns multiple)
        if is_key_pressed(KeyCode::O) {
            let (mouse_x, mouse_y) = mouse_position();
            state.action_log.log_start(Action::SpawnActor { x: mouse_x, y: mouse_y });
            // Actor size and collision radius from configuration
            let actor_size = state.cell_width.min(state.cell_height) * state.actor_size_ratio;
            let actor_id = state.next_actor_id;
            state.next_actor_id += 1;
            let collision_radius = state.cell_width.min(state.cell_height) * state.actor_collision_radius_ratio;
            let subcell_grid_size = state.subcell_reservation_manager.grid_size();
            let (offset_x, offset_y) = state.subcell_offset.get_offsets();
            let actor = Actor::new(actor_id, mouse_x, mouse_y, actor_size, state.actor_speed, collision_radius, state.cell_width, state.cell_height, subcell_grid_size, offset_x, offset_y);
            state.actors.push(actor);
            state.action_log.log_finish(Action::SpawnActor { x: mouse_x, y: mouse_y });
            println!("Actor {} spawned at ({:.1}, {:.1}). Total actors: {}", actor_id, mouse_x, mouse_y, state.actors.len());
        }

        // Clear all actors on Key0 (but preserve their tracks if tracking is enabled)
        if is_key_pressed(KeyCode::Key0) {
            let actor_count = state.actors.len();
            if actor_count > 0 {
                // Save tracks to historical storage before clearing actors
                if state.tracking_mode == TrackingMode::Tracking {
                    for actor in &state.actors {
                        if !actor.movement_track.is_empty() {
                            state.historical_tracks.push(actor.movement_track.clone());
                        }
                    }
                }

                state.actors.clear();
                state.subcell_reservation_manager.clear();
                state.highlighted_actors.clear();
                println!("Cleared {} actors (tracks preserved)", actor_count);
            }
        }

        // Toggle movement tracking on L key
        if is_key_pressed(KeyCode::L) {
            state.tracking_mode = state.tracking_mode.toggle();
            println!("Movement Tracking: {}", state.tracking_mode.to_string());

            // When disabling, clear all tracking vectors (current and historical)
            if state.tracking_mode == TrackingMode::Disabled {
                for actor in &mut state.actors {
                    actor.movement_track.clear();
                }
                state.historical_tracks.clear();
                println!("Cleared all movement tracks");
            }
        }

        // Set actor destination on P key (uses pathfinding or sub-cell movement) - applies to ALL actors
        if is_key_pressed(KeyCode::P) {
            if !state.actors.is_empty() {
                let (mouse_x, mouse_y) = mouse_position();
                let target_grid_x = (mouse_x / state.cell_width) as i32;
                let target_grid_y = (mouse_y / state.cell_height) as i32;

                state.action_log.log_start(Action::SetActorDestination {
                    x: target_grid_x,
                    y: target_grid_y,
                    actor_count: state.actors.len(),
                });

                if state.subcell_movement_enabled {
                    // Sub-cell movement mode - spread actors across different CELLS
                    // Sub-cells are only for intermediate movement, not final destinations!
                    let cell_destinations = spread_cell_destinations(
                        target_grid_x,
                        target_grid_y,
                        state.actors.len(),
                    );
                    for (actor, (dest_x, dest_y)) in state.actors.iter_mut().zip(cell_destinations.iter()) {
                        let dest_pos = Position { x: *dest_x, y: *dest_y };
                        actor.set_subcell_destination(dest_pos);
                    }
                    println!("Sub-cell destinations set: ({}, {}) for {} actors (spread across {} cells)",
                        target_grid_x, target_grid_y, state.actors.len(), cell_destinations.len());
                } else {
                    // Normal pathfinding mode
                    // First pass: calculate unique destinations for each actor
                    let mut occupied_destinations = HashSet::new();
                    let mut actor_destinations = Vec::new();

                    for _actor in &state.actors {
                        // Find unique destination for this actor using spiral search
                        let dest = state.find_available_destination(
                            target_grid_x,
                            target_grid_y,
                            &occupied_destinations,
                        );

                        // Mark this destination as occupied for next actor
                        occupied_destinations.insert(dest);
                        actor_destinations.push(dest);
                    }

                    // Second pass: assign destinations and calculate paths
                    let mut paths_set = 0;
                    let mut no_paths = 0;

                    for (actor, dest) in state.actors.iter_mut().zip(actor_destinations.iter()) {
                        // Calculate actor's current cell position
                        let actor_cpos = actor.calculate_cell_position(&state.grid, state.cell_width, state.cell_height);

                        // Find path using pathfinding WITH CACHED CORNERS
                        if let Some(mut path) = find_path_with_cache(
                            &state.grid,
                            actor_cpos.cell_x,
                            actor_cpos.cell_y,
                            dest.0,
                            dest.1,
                            actor_cpos.messy_x,
                            actor_cpos.messy_y,
                            Some(&state.all_corners),
                        ) {
                            // Skip the first waypoint if it's the actor's current cell
                            if path.len() >= 2 {
                                let first_waypoint = &path[0];
                                if first_waypoint.x == actor_cpos.cell_x && first_waypoint.y == actor_cpos.cell_y {
                                    path.remove(0);
                                }
                            }

                            actor.set_path(path, state.grid.get_revision());
                            paths_set += 1;
                        } else {
                            no_paths += 1;
                        }
                    }

                    println!("Destination target ({}, {}): {} actors have unique paths, {} blocked",
                        target_grid_x, target_grid_y, paths_set, no_paths);
                }

                state.action_log.log_finish(Action::SetActorDestination {
                    x: target_grid_x,
                    y: target_grid_y,
                    actor_count: state.actors.len(),
                });
            }
        }

        // Set destination for RANDOM SUBSET (30%) on R key - closest actors get priority
        if is_key_pressed(KeyCode::R) {
            if !state.actors.is_empty() {
                let (mouse_x, mouse_y) = mouse_position();
                let target_grid_x = (mouse_x / state.cell_width) as i32;
                let target_grid_y = (mouse_y / state.cell_height) as i32;

                // Calculate 30% of actors (minimum 1)
                let num_to_select = ((state.actors.len() * 30) / 100).max(1);

                // Sort actors by distance to clicked position
                let sorted_indices = state.actors_sorted_by_distance(mouse_x, mouse_y);

                // Select closest 30% of actors
                let selected_indices: Vec<usize> = sorted_indices.into_iter().take(num_to_select).collect();

                // Log action
                state.action_log.log_start(Action::SetActorDestination {
                    x: target_grid_x,
                    y: target_grid_y,
                    actor_count: selected_indices.len(),
                });

                if state.subcell_movement_enabled {
                    // Sub-cell movement mode - spread selected actors across different CELLS
                    let cell_destinations = spread_cell_destinations(
                        target_grid_x,
                        target_grid_y,
                        selected_indices.len(),
                    );

                    for (i, &actor_idx) in selected_indices.iter().enumerate() {
                        if let Some((dest_x, dest_y)) = cell_destinations.get(i) {
                            let dest_pos = Position { x: *dest_x, y: *dest_y };
                            state.actors[actor_idx].set_subcell_destination(dest_pos);
                        }
                    }
                } else {
                    // Normal pathfinding mode
                    let mut occupied_destinations = HashSet::new();
                    let mut actor_destinations = Vec::new();

                    for _ in &selected_indices {
                        let dest = state.find_available_destination(
                            target_grid_x,
                            target_grid_y,
                            &occupied_destinations,
                        );
                        occupied_destinations.insert(dest);
                        actor_destinations.push(dest);
                    }

                    for (i, &actor_idx) in selected_indices.iter().enumerate() {
                        if let Some(dest) = actor_destinations.get(i) {
                            let actor = &mut state.actors[actor_idx];
                            let actor_cpos = actor.calculate_cell_position(&state.grid, state.cell_width, state.cell_height);

                            if let Some(mut path) = find_path_with_cache(
                                &state.grid,
                                actor_cpos.cell_x,
                                actor_cpos.cell_y,
                                dest.0,
                                dest.1,
                                actor_cpos.messy_x,
                                actor_cpos.messy_y,
                                Some(&state.all_corners),
                            ) {
                                if !path.is_empty() {
                                    path.remove(0);
                                }
                                actor.set_path(path, state.grid.get_revision());
                            }
                        }
                    }
                }

                // Set up visual highlight for selected actors
                state.highlighted_actors.clear();
                for &actor_idx in &selected_indices {
                    state.highlighted_actors.insert(state.actors[actor_idx].id);
                }
                state.highlight_timer = 2.0; // Highlight for 2 seconds

                // Collect actor IDs for console output
                let actor_ids: Vec<usize> = selected_indices.iter()
                    .map(|&idx| state.actors[idx].id)
                    .collect();

                println!("Random destination: {}/{} actors selected (IDs: {:?}) → cell ({}, {})",
                    selected_indices.len(), state.actors.len(), actor_ids, target_grid_x, target_grid_y);

                state.action_log.log_finish(Action::SetActorDestination {
                    x: target_grid_x,
                    y: target_grid_y,
                    actor_count: selected_indices.len(),
                });
            }
        }

        // Check if actors' paths need recalculation due to grid changes
        // OR if actor has a destination but no path (blocked, waiting for opening)
        for actor in &mut state.actors {
            let should_recalculate = actor.is_path_outdated(state.grid.get_revision())
                || (actor.destination.is_some() && !actor.has_path());

            if should_recalculate {
                // Grid has changed or actor is waiting - (re)calculate path to destination
                if let Some(dest) = actor.destination {
                    let actor_cpos = actor.calculate_cell_position(&state.grid, state.cell_width, state.cell_height);

                    if let Some(mut path) = find_path_with_cache(
                        &state.grid,
                        actor_cpos.cell_x,
                        actor_cpos.cell_y,
                        dest.x,
                        dest.y,
                        actor_cpos.messy_x,
                        actor_cpos.messy_y,
                        Some(&state.all_corners),
                    ) {
                        // Skip first waypoint if it's the current cell
                        if path.len() >= 2 {
                            let first_waypoint = &path[0];
                            if first_waypoint.x == actor_cpos.cell_x && first_waypoint.y == actor_cpos.cell_y {
                                path.remove(0);
                            }
                        }

                        let was_stopped = !actor.has_path();
                        actor.set_path(path, state.grid.get_revision());
                        if was_stopped {
                            // Only print once when first actor resumes to avoid spam
                            static mut RESUME_LOGGED: bool = false;
                            unsafe {
                                if !RESUME_LOGGED {
                                    println!("Actor(s) resumed - path found after obstacle removed");
                                    RESUME_LOGGED = true;
                                }
                            }
                        }
                    } else {
                        // No path found - clear the path but keep destination
                        if actor.has_path() {
                            actor.clear_path();
                        }
                    }
                }
            }
        }

        // Update actor movement with NPV (Next Position Validation) and collision checking
        let delta_time = get_frame_time();

        // Update highlight timer for random subset feature
        if state.highlight_timer > 0.0 {
            state.highlight_timer -= delta_time;
            if state.highlight_timer <= 0.0 {
                state.highlighted_actors.clear();
            }
        }

        if state.subcell_movement_enabled {
            // Sub-cell movement mode - update all actors with sub-cell logic
            let enable_square = state.reservation_mode == ReservationMode::Square;
            let enable_diagonal = state.reservation_mode == ReservationMode::Diagonal;
            let enable_no_diagonal = state.reservation_mode == ReservationMode::NoDiagonal;
            let enable_anti_cross = state.reservation_mode == ReservationMode::AntiCross;
            let enable_basic3 = state.reservation_mode == ReservationMode::Basic3;
            let enable_basic3_anti_cross = state.reservation_mode == ReservationMode::Basic3AntiCross;
            let track_movement = state.tracking_mode == TrackingMode::Tracking;
            for i in 0..state.actors.len() {
                let _reached = state.actors[i].update_subcell(
                    delta_time,
                    &mut state.subcell_reservation_manager,
                    enable_square,
                    enable_diagonal,
                    enable_no_diagonal,
                    enable_anti_cross,
                    enable_basic3,
                    enable_basic3_anti_cross,
                    state.early_reservation_enabled,
                    state.filter_backward_moves,
                    track_movement,
                );
                // Note: ignoring reached status for now - no event logging in sub-cell mode
            }
        } else {
            // Normal pathfinding mode with NPV and collision checking
            // Collect nearby actors data for each actor (positions and radii only)
            // This avoids borrowing issues by cloning only the data we need
            #[derive(Clone)]
            struct ActorCollisionData {
                id: usize,
                fpos_x: f32,
                fpos_y: f32,
                collision_radius: f32,
                size: f32,
                cell_width: f32,
                cell_height: f32,
            }

            let actor_data: Vec<ActorCollisionData> = state.actors.iter().map(|a| ActorCollisionData {
                id: a.id,
                fpos_x: a.fpos_x,
                fpos_y: a.fpos_y,
                collision_radius: a.collision_radius,
                size: a.size,
                cell_width: a.cell_width,
                cell_height: a.cell_height,
            }).collect();

            // Helper to get occupied cells from collision data
            let get_cells = |data: &ActorCollisionData| -> Vec<(i32, i32)> {
                let half_size = data.size / 2.0;
                let top_left_x = data.fpos_x - half_size;
                let top_left_y = data.fpos_y - half_size;
                let bottom_right_x = data.fpos_x + half_size;
                let bottom_right_y = data.fpos_y + half_size;

                let top_left_cell_x = (top_left_x / data.cell_width).floor() as i32;
                let top_left_cell_y = (top_left_y / data.cell_height).floor() as i32;
                let bottom_right_cell_x = (bottom_right_x / data.cell_width).floor() as i32;
                let bottom_right_cell_y = (bottom_right_y / data.cell_height).floor() as i32;

                let mut cells = Vec::new();
                for cy in top_left_cell_y..=bottom_right_cell_y {
                    for cx in top_left_cell_x..=bottom_right_cell_x {
                        if cx >= 0 && cx < state.grid.cols && cy >= 0 && cy < state.grid.rows {
                            cells.push((cx, cy));
                        }
                    }
                }
                cells
            };

            // Build nearby actor indices for each actor
            let mut actor_nearby_lists: Vec<Vec<usize>> = Vec::new();
            for i in 0..state.actors.len() {
                let actor = &state.actors[i];

                // Calculate next position for this actor
                if let Some((waypoint_x, waypoint_y)) = actor.get_current_waypoint_screen_coords() {
                    let dx = waypoint_x - actor.fpos_x;
                    let dy = waypoint_y - actor.fpos_y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let movement_this_frame = actor.speed * delta_time;

                    let (next_x, next_y) = if distance <= movement_this_frame {
                        (waypoint_x, waypoint_y)
                    } else {
                        let dir_x = dx / distance;
                        let dir_y = dy / distance;
                        (actor.fpos_x + dir_x * movement_this_frame,
                         actor.fpos_y + dir_y * movement_this_frame)
                    };

                    let next_cells = actor.get_occupied_cells(next_x, next_y, &state.grid);

                    let mut nearby_indices = Vec::new();
                    for j in 0..actor_data.len() {
                        if i == j { continue; }
                        let other_cells = get_cells(&actor_data[j]);
                        if next_cells.iter().any(|nc| other_cells.contains(nc)) {
                            nearby_indices.push(j);
                        }
                    }
                    actor_nearby_lists.push(nearby_indices);
                } else {
                    actor_nearby_lists.push(Vec::new());
                }
            }

            // Now update each actor with collision data of nearby actors
            for i in 0..state.actors.len() {
                // Create temporary Actor instances from collision data for nearby actors
                let subcell_grid_size = state.subcell_reservation_manager.grid_size();
                let (offset_x, offset_y) = state.subcell_offset.get_offsets();
                let nearby_actors: Vec<Actor> = actor_nearby_lists[i]
                    .iter()
                    .map(|&idx| {
                        let data = &actor_data[idx];
                        Actor::new(data.id, data.fpos_x, data.fpos_y, data.size, 0.0, data.collision_radius, data.cell_width, data.cell_height, subcell_grid_size, offset_x, offset_y)
                    })
                    .collect();

                let nearby_refs: Vec<&Actor> = nearby_actors.iter().collect();
                let (_reached, event) = state.actors[i].update_with_npv(delta_time, &state.grid, &nearby_refs);

                // Log movement events
                if let Some(movement_event) = event {
                    match movement_event {
                        MovementEvent::StartedMovingTo { actor_id, cell_x, cell_y, cell_id } => {
                            state.action_log.log_event(Action::ActorStartMovingToCell {
                                actor_id,
                                cell_x,
                                cell_y,
                                cell_id,
                            });
                        }
                        MovementEvent::ReachedWaypoint { actor_id, cell_x, cell_y, cell_id, next_cell_x, next_cell_y, next_cell_id } => {
                            state.action_log.log_event(Action::ActorReachedWaypoint {
                                actor_id,
                                cell_x,
                                cell_y,
                                cell_id,
                                next_cell_x,
                                next_cell_y,
                                next_cell_id,
                            });
                        }
                        MovementEvent::ReachedDestination { actor_id, cell_x, cell_y, cell_id } => {
                            state.action_log.log_event(Action::ActorReachedDestination {
                                actor_id,
                                cell_x,
                                cell_y,
                                cell_id,
                            });
                        }
                    }
                }
            }
        }

        // Close window on Escape
        if is_key_pressed(KeyCode::Escape) {
            // Print summary and full log with durations
            println!("\n{}", state.action_log.summary());
            state.action_log.print_with_durations();

            // Close the streaming JSON log
            state.action_log.close_json_stream();
            println!("Action log saved to action_log.json (streaming mode)");

            // Save to files (compact binary only, JSON already streamed)
            if let Err(e) = state.action_log.save_to_file("action_log.json") {
                eprintln!("Failed to finalize JSON action log: {}", e);
            }

            if let Err(e) = state.action_log.save_compact_to_file("action_log.bin") {
                eprintln!("Failed to save compact action log: {}", e);
            } else {
                let (json_size, compact_size, compression_ratio) = state.action_log.get_compact_stats();
                println!("Compact action log saved to action_log.bin ({} bytes vs {} bytes JSON, {:.1}% smaller)",
                    compact_size, json_size, compression_ratio);
            }

            break;
        }

        // Draw
        state.draw();

        next_frame().await
    }
}
