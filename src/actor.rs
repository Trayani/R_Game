use crate::Grid;
use crate::pathfinding::Position;

/// Actor represents a dynamic element in the grid with precise floating-point positioning
#[derive(Clone, Debug)]
pub struct Actor {
    /// Size of the actor's square area (must be ≤ cell size)
    pub size: f32,

    /// Floating-point position (center of actor's square)
    pub fpos_x: f32,
    pub fpos_y: f32,

    /// Speed: roughly the number of pixels the actor can travel per frame
    pub speed: f32,

    /// Pathfinding waypoints (in cell coordinates)
    pub path: Vec<Position>,

    /// Current waypoint index in the path
    pub current_waypoint: usize,

    /// Cell dimensions for converting path to screen coordinates
    pub cell_width: f32,
    pub cell_height: f32,

    /// Grid revision number when the path was calculated
    pub path_grid_revision: u64,

    /// Destination cell coordinates (for path recalculation)
    pub destination: Option<Position>,
}

/// Cell position state describing which cell(s) the actor occupies
#[derive(Clone, Debug, PartialEq)]
pub struct CellPosition {
    /// Top-left cell coordinates
    pub cell_x: i32,
    pub cell_y: i32,

    /// Messy state flags
    pub messy_x: bool,
    pub messy_y: bool,
}

impl Actor {
    /// Create a new actor at the given floating-point position
    pub fn new(fpos_x: f32, fpos_y: f32, size: f32, speed: f32, cell_width: f32, cell_height: f32) -> Self {
        Actor {
            size,
            fpos_x,
            fpos_y,
            speed,
            path: Vec::new(),
            current_waypoint: 0,
            cell_width,
            cell_height,
            path_grid_revision: 0,
            destination: None,
        }
    }

    /// Calculate cell position from floating-point position
    ///
    /// The cell is determined by the TOP-LEFT point of actor's square.
    /// Messiness is determined by comparing top-left and bottom-right cell positions.
    pub fn calculate_cell_position(&self, grid: &Grid, cell_width: f32, cell_height: f32) -> CellPosition {
        let half_size = self.size / 2.0;

        // Calculate top-left corner of actor's square
        let top_left_x = self.fpos_x - half_size;
        let top_left_y = self.fpos_y - half_size;

        // Calculate bottom-right corner of actor's square
        let bottom_right_x = self.fpos_x + half_size;
        let bottom_right_y = self.fpos_y + half_size;

        // Determine cell coordinates for top-left
        let top_left_cell_x = (top_left_x / cell_width).floor() as i32;
        let top_left_cell_y = (top_left_y / cell_height).floor() as i32;

        // Determine cell coordinates for bottom-right
        let bottom_right_cell_x = (bottom_right_x / cell_width).floor() as i32;
        let bottom_right_cell_y = (bottom_right_y / cell_height).floor() as i32;

        // Clamp to grid boundaries
        let cell_x = top_left_cell_x.max(0).min(grid.cols - 1);
        let cell_y = top_left_cell_y.max(0).min(grid.rows - 1);

        // Determine messiness
        let messy_x = top_left_cell_x != bottom_right_cell_x;
        let messy_y = top_left_cell_y != bottom_right_cell_y;

        CellPosition {
            cell_x,
            cell_y,
            messy_x,
            messy_y,
        }
    }

    /// Set a path for the actor to follow
    pub fn set_path(&mut self, path: Vec<Position>, grid_revision: u64) {
        // Store the destination for potential recalculation
        self.destination = path.last().copied();
        self.path = path;
        self.current_waypoint = 0;
        self.path_grid_revision = grid_revision;
    }

    /// Clear the current path
    pub fn clear_path(&mut self) {
        self.path.clear();
        self.current_waypoint = 0;
        self.destination = None;
    }

    /// Check if actor has a path to follow
    pub fn has_path(&self) -> bool {
        !self.path.is_empty() && self.current_waypoint < self.path.len()
    }

    /// Check if the path is outdated (grid has changed since path was calculated)
    pub fn is_path_outdated(&self, current_grid_revision: u64) -> bool {
        self.has_path() && self.path_grid_revision != current_grid_revision
    }

    /// Get the current waypoint in screen coordinates
    fn get_current_waypoint_screen_coords(&self) -> Option<(f32, f32)> {
        if self.current_waypoint < self.path.len() {
            let waypoint = &self.path[self.current_waypoint];
            let screen_x = waypoint.x as f32 * self.cell_width + self.cell_width / 2.0;
            let screen_y = waypoint.y as f32 * self.cell_height + self.cell_height / 2.0;
            Some((screen_x, screen_y))
        } else {
            None
        }
    }

    /// Move the actor along its path (call once per frame)
    /// Returns true if the actor reached the end of its path
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.has_path() {
            return true;
        }

        // Get current waypoint in screen coordinates
        if let Some((waypoint_x, waypoint_y)) = self.get_current_waypoint_screen_coords() {
            // Calculate direction vector to current waypoint
            let dx = waypoint_x - self.fpos_x;
            let dy = waypoint_y - self.fpos_y;

            // Calculate distance
            let distance = (dx * dx + dy * dy).sqrt();

            // Check if we've reached the current waypoint
            let movement_this_frame = self.speed * delta_time;
            if distance <= movement_this_frame {
                // Snap to waypoint
                self.fpos_x = waypoint_x;
                self.fpos_y = waypoint_y;

                // Move to next waypoint
                self.current_waypoint += 1;

                // Check if we've reached the end of the path
                if self.current_waypoint >= self.path.len() {
                    self.clear_path();
                    return true;
                }

                // Continue moving towards next waypoint in the same frame
                return self.update(delta_time);
            }

            // Normalize direction and move
            let dir_x = dx / distance;
            let dir_y = dy / distance;

            self.fpos_x += dir_x * movement_this_frame;
            self.fpos_y += dir_y * movement_this_frame;

            false
        } else {
            // No valid waypoint
            true
        }
    }

    /// Get the final destination of the path (if any)
    pub fn get_path_destination(&self) -> Option<Position> {
        self.path.last().copied()
    }

    /// Get the corners of the actor's square in screen coordinates
    pub fn get_bounds(&self) -> (f32, f32, f32, f32) {
        let half_size = self.size / 2.0;
        let left = self.fpos_x - half_size;
        let top = self.fpos_y - half_size;
        let right = self.fpos_x + half_size;
        let bottom = self.fpos_y + half_size;

        (left, top, right, bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_clean_position() {
        let grid = Grid::new(10, 10);
        let cell_width = 20.0;
        let cell_height = 15.0;

        // Actor centered in cell (5, 5) - should be clean
        let actor = Actor::new(
            5.0 * cell_width + cell_width / 2.0,  // Center of cell (5, 5)
            5.0 * cell_height + cell_height / 2.0,
            10.0,  // Size smaller than cell
            100.0,
            cell_width,
            cell_height,
        );

        let cpos = actor.calculate_cell_position(&grid, cell_width, cell_height);
        assert_eq!(cpos.cell_x, 5);
        assert_eq!(cpos.cell_y, 5);
        assert_eq!(cpos.messy_x, false);
        assert_eq!(cpos.messy_y, false);
    }

    #[test]
    fn test_actor_messy_x() {
        let grid = Grid::new(10, 10);
        let cell_width = 20.0;
        let cell_height = 15.0;

        // Actor straddling cells (5, 5) and (6, 5) horizontally
        let actor = Actor::new(
            5.0 * cell_width + cell_width,  // On the border
            5.0 * cell_height + cell_height / 2.0,
            10.0,
            100.0,
            cell_width,
            cell_height,
        );

        let cpos = actor.calculate_cell_position(&grid, cell_width, cell_height);
        assert_eq!(cpos.messy_x, true);
        assert_eq!(cpos.messy_y, false);
    }

    #[test]
    fn test_actor_messy_y() {
        let grid = Grid::new(10, 10);
        let cell_width = 20.0;
        let cell_height = 15.0;

        // Actor straddling cells (5, 5) and (5, 6) vertically
        let actor = Actor::new(
            5.0 * cell_width + cell_width / 2.0,
            5.0 * cell_height + cell_height,  // On the border
            10.0,
            100.0,
            cell_width,
            cell_height,
        );

        let cpos = actor.calculate_cell_position(&grid, cell_width, cell_height);
        assert_eq!(cpos.messy_x, false);
        assert_eq!(cpos.messy_y, true);
    }

    #[test]
    fn test_actor_messy_xy() {
        let grid = Grid::new(10, 10);
        let cell_width = 20.0;
        let cell_height = 15.0;

        // Actor straddling 4 cells
        let actor = Actor::new(
            5.0 * cell_width + cell_width,
            5.0 * cell_height + cell_height,
            10.0,
            100.0,
            cell_width,
            cell_height,
        );

        let cpos = actor.calculate_cell_position(&grid, cell_width, cell_height);
        assert_eq!(cpos.messy_x, true);
        assert_eq!(cpos.messy_y, true);
    }

    #[test]
    fn test_actor_path_following() {
        let cell_width = 20.0;
        let cell_height = 15.0;
        // Start actor at cell (0,0) center
        let start_x = 0.0 * cell_width + cell_width / 2.0;
        let start_y = 0.0 * cell_height + cell_height / 2.0;
        let mut actor = Actor::new(start_x, start_y, 10.0, 100.0, cell_width, cell_height);

        // Create a simple path: (1,0) -> (2,0) -> (2,1)
        let path = vec![
            Position { x: 1, y: 0 },
            Position { x: 2, y: 0 },
            Position { x: 2, y: 1 },
        ];
        actor.set_path(path, 0);

        assert!(actor.has_path());
        assert_eq!(actor.current_waypoint, 0);

        // Move a little bit - should not finish the entire path
        let reached = actor.update(0.1);
        assert!(!reached); // Should not have finished the entire path yet
        assert!(actor.has_path()); // Should still have a path to follow
    }
}
