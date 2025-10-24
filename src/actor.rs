use crate::Grid;
use crate::pathfinding::Position;
use crate::subcell::SubCellCoord;

/// Movement event for logging
#[derive(Clone, Debug)]
pub enum MovementEvent {
    /// Actor started moving to a new waypoint
    StartedMovingTo { actor_id: usize, cell_x: i32, cell_y: i32, cell_id: i32 },
    /// Actor reached a waypoint and continues to next
    ReachedWaypoint { actor_id: usize, cell_x: i32, cell_y: i32, cell_id: i32, next_cell_x: i32, next_cell_y: i32, next_cell_id: i32 },
    /// Actor reached final destination
    ReachedDestination { actor_id: usize, cell_x: i32, cell_y: i32, cell_id: i32 },
}

/// Actor represents a dynamic element in the grid with precise floating-point positioning
#[derive(Clone, Debug)]
pub struct Actor {
    /// Unique actor ID
    pub id: usize,

    /// Size of the actor's square area (must be ≤ cell size)
    pub size: f32,

    /// Floating-point position (center of actor's square)
    pub fpos_x: f32,
    pub fpos_y: f32,

    /// Speed: roughly the number of pixels the actor can travel per frame
    pub speed: f32,

    /// Collision radius: circular space that should not be shared with other actors (in pixels)
    pub collision_radius: f32,

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

    /// Last frame's collision state - ID of actor that blocked us (for deduplication)
    pub last_blocking_actor: Option<usize>,

    // Sub-cell movement state
    /// Sub-cell grid size (2 for 2x2, 3 for 3x3)
    pub subcell_grid_size: i32,
    /// Current sub-cell position
    pub current_subcell: Option<SubCellCoord>,
    /// Reserved sub-cell that actor is moving toward
    pub reserved_subcell: Option<SubCellCoord>,
    /// Additional reserved sub-cells (for square reservations)
    pub extra_reserved_subcells: Vec<SubCellCoord>,
    /// Final destination for sub-cell movement (cell-level, NOT sub-cell level)
    pub subcell_destination: Option<Position>,
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
    pub fn new(id: usize, fpos_x: f32, fpos_y: f32, size: f32, speed: f32, collision_radius: f32, cell_width: f32, cell_height: f32, subcell_grid_size: i32) -> Self {
        // Initialize sub-cell position
        let current_subcell = Some(SubCellCoord::from_screen_pos(fpos_x, fpos_y, cell_width, cell_height, subcell_grid_size));

        Actor {
            id,
            size,
            fpos_x,
            fpos_y,
            speed,
            collision_radius,
            path: Vec::new(),
            current_waypoint: 0,
            cell_width,
            cell_height,
            path_grid_revision: 0,
            destination: None,
            last_blocking_actor: None,
            subcell_grid_size,
            current_subcell,
            reserved_subcell: None,
            extra_reserved_subcells: Vec::new(),
            subcell_destination: None,
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

    /// Clear the current path (keeps destination for retry)
    pub fn clear_path(&mut self) {
        self.path.clear();
        self.current_waypoint = 0;
        // Keep destination - actor will retry pathfinding if grid changes
    }

    /// Clear the current path and destination (completely stop)
    pub fn clear_destination(&mut self) {
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
    pub fn get_current_waypoint_screen_coords(&self) -> Option<(f32, f32)> {
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

    /// Move the actor along its path with Next Position Validation (NPV)
    /// NPV prevents actors from moving into blocked cells due to imprecise movement
    /// Also performs collision radius checking with nearby actors
    /// Returns (reached_end, optional_movement_event)
    pub fn update_with_npv(&mut self, delta_time: f32, grid: &Grid, nearby_actors: &[&Actor]) -> (bool, Option<MovementEvent>) {
        if !self.has_path() {
            return (true, None);
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

                // Get current waypoint info for logging
                let current_wp = &self.path[self.current_waypoint];
                let current_cell_id = grid.get_id(current_wp.x, current_wp.y);

                // Move to next waypoint
                self.current_waypoint += 1;

                // Check if we've reached the end of the path
                if self.current_waypoint >= self.path.len() {
                    // Reached final destination
                    let event = MovementEvent::ReachedDestination {
                        actor_id: self.id,
                        cell_x: current_wp.x,
                        cell_y: current_wp.y,
                        cell_id: current_cell_id,
                    };
                    self.clear_path();
                    return (true, Some(event));
                }

                // Get next waypoint info for logging
                let next_wp = &self.path[self.current_waypoint];
                let next_cell_id = grid.get_id(next_wp.x, next_wp.y);

                let event = MovementEvent::ReachedWaypoint {
                    actor_id: self.id,
                    cell_x: current_wp.x,
                    cell_y: current_wp.y,
                    cell_id: current_cell_id,
                    next_cell_x: next_wp.x,
                    next_cell_y: next_wp.y,
                    next_cell_id: next_cell_id,
                };

                // Continue moving towards next waypoint in the same frame
                let (reached, _) = self.update_with_npv(delta_time, grid, nearby_actors);
                return (reached, Some(event));
            }

            // Normalize direction and calculate next position
            let dir_x = dx / distance;
            let dir_y = dy / distance;

            let next_fpos_x = self.fpos_x + dir_x * movement_this_frame;
            let next_fpos_y = self.fpos_y + dir_y * movement_this_frame;

            // NPV: Validate that next position doesn't occupy any blocked cells
            // and maintains collision radius with nearby actors
            if self.is_position_valid_with_collision(next_fpos_x, next_fpos_y, grid, nearby_actors) {
                // Position is valid - move to it
                self.fpos_x = next_fpos_x;
                self.fpos_y = next_fpos_y;
                // Clear collision state when moving successfully
                self.last_blocking_actor = None;
                (false, None)
            } else {
                // Position is blocked - stay in place and clear path
                self.clear_path();
                (true, None)
            }
        } else {
            // No valid waypoint
            (true, None)
        }
    }

    /// Get all cells occupied by the actor at a given position
    /// Returns a vector of (cell_x, cell_y) tuples
    /// Handles messy positions (can occupy 1x1 up to 4x4 cells)
    pub fn get_occupied_cells(&self, fpos_x: f32, fpos_y: f32, grid: &Grid) -> Vec<(i32, i32)> {
        let half_size = self.size / 2.0;

        // Calculate corners of actor's square at the proposed position
        let top_left_x = fpos_x - half_size;
        let top_left_y = fpos_y - half_size;
        let bottom_right_x = fpos_x + half_size;
        let bottom_right_y = fpos_y + half_size;

        // Get cell coordinates for all corners
        let top_left_cell_x = (top_left_x / self.cell_width).floor() as i32;
        let top_left_cell_y = (top_left_y / self.cell_height).floor() as i32;
        let bottom_right_cell_x = (bottom_right_x / self.cell_width).floor() as i32;
        let bottom_right_cell_y = (bottom_right_y / self.cell_height).floor() as i32;

        let mut cells = Vec::new();

        // Collect all cells in the range
        for cy in top_left_cell_y..=bottom_right_cell_y {
            for cx in top_left_cell_x..=bottom_right_cell_x {
                // Only include cells within grid boundaries
                if cx >= 0 && cx < grid.cols && cy >= 0 && cy < grid.rows {
                    cells.push((cx, cy));
                }
            }
        }

        cells
    }

    /// Check if a position is valid (doesn't occupy any blocked cells)
    /// Used for Next Position Validation (NPV)
    fn is_position_valid(&self, fpos_x: f32, fpos_y: f32, grid: &Grid) -> bool {
        let half_size = self.size / 2.0;

        // Calculate corners of actor's square at the proposed position
        let top_left_x = fpos_x - half_size;
        let top_left_y = fpos_y - half_size;
        let bottom_right_x = fpos_x + half_size;
        let bottom_right_y = fpos_y + half_size;

        // Get cell coordinates for all corners
        let top_left_cell_x = (top_left_x / self.cell_width).floor() as i32;
        let top_left_cell_y = (top_left_y / self.cell_height).floor() as i32;
        let bottom_right_cell_x = (bottom_right_x / self.cell_width).floor() as i32;
        let bottom_right_cell_y = (bottom_right_y / self.cell_height).floor() as i32;

        // Check all cells that the actor would occupy
        for cy in top_left_cell_y..=bottom_right_cell_y {
            for cx in top_left_cell_x..=bottom_right_cell_x {
                // Check bounds
                if cx < 0 || cx >= grid.cols || cy < 0 || cy >= grid.rows {
                    return false; // Out of bounds
                }
                // Check if cell is blocked
                if grid.is_blocked(cx, cy) {
                    return false; // Would occupy a blocked cell
                }
            }
        }

        true // All cells are free
    }

    /// Check if a position is valid with collision radius checking
    /// Used for Next Position Validation (NPV) with multi-actor collision avoidance
    ///
    /// Returns true if:
    /// 1. Position doesn't occupy any blocked cells, AND
    /// 2. Either no cells are shared with nearby actors, OR
    /// 3. For each nearby actor: distance check passes (radius check OR better distance)
    fn is_position_valid_with_collision(&self, fpos_x: f32, fpos_y: f32, grid: &Grid, nearby_actors: &[&Actor]) -> bool {
        // Step 1: Check that next position doesn't occupy any blocked cells
        let occupied_cells = self.get_occupied_cells(fpos_x, fpos_y, grid);

        for (cx, cy) in &occupied_cells {
            if grid.is_blocked(*cx, *cy) {
                return false; // Would occupy a blocked cell
            }
        }

        // Step 2: If no nearby actors, position is valid
        if nearby_actors.is_empty() {
            return true;
        }

        // Step 3: Check collision radius with each nearby actor
        for nearby in nearby_actors {
            // Skip self (shouldn't happen but safety check)
            if nearby.id == self.id {
                continue;
            }

            // Calculate distances
            let next_dx = fpos_x - nearby.fpos_x;
            let next_dy = fpos_y - nearby.fpos_y;
            let next_distance = (next_dx * next_dx + next_dy * next_dy).sqrt();

            let current_dx = self.fpos_x - nearby.fpos_x;
            let current_dy = self.fpos_y - nearby.fpos_y;
            let current_distance = (current_dx * current_dx + current_dy * current_dy).sqrt();

            // Radius check: ideal case where actors maintain proper distance
            let radius_check = next_distance > self.collision_radius + nearby.collision_radius;

            // Better distance: movement is allowed as long as distance increases
            // (even if radius is currently violated!)
            let better_distance = next_distance > current_distance;

            // If NEITHER condition is satisfied, movement is blocked
            if !radius_check && !better_distance {
                return false;
            }
        }

        // Step 4: All checks passed
        true
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

    /// Set sub-cell destination for movement (cell-level)
    pub fn set_subcell_destination(&mut self, dest: Position) {
        self.subcell_destination = Some(dest);
        // Initialize current sub-cell if not set
        if self.current_subcell.is_none() {
            self.current_subcell = Some(SubCellCoord::from_screen_pos(
                self.fpos_x,
                self.fpos_y,
                self.cell_width,
                self.cell_height,
                self.subcell_grid_size,
            ));
        }
    }


    /// Update sub-cell movement
    /// Returns true if destination reached, false otherwise
    ///
    /// This implements the sub-cell movement algorithm:
    /// 1. Move toward reserved sub-cell
    /// 2. When closer to reserved than current, switch current to reserved
    /// 3. When centered on current, try to reserve next sub-cell toward destination
    /// 4. Use fallback neighbors if preferred sub-cell is occupied
    pub fn update_subcell(
        &mut self,
        delta_time: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
    ) -> bool {
        // Check if we have a destination
        let dest = match self.subcell_destination {
            Some(d) => d,
            None => return true, // No destination, we're done
        };

        // Ensure we have current sub-cell
        let current = match self.current_subcell {
            Some(c) => c,
            None => {
                // Initialize from current position
                let c = SubCellCoord::from_screen_pos(
                    self.fpos_x,
                    self.fpos_y,
                    self.cell_width,
                    self.cell_height,
                    self.subcell_grid_size,
                );
                self.current_subcell = Some(c);
                c
            }
        };

        // Get destination screen position
        // Destination is always at cell level (center of cell)
        // Sub-cells are ONLY used for intermediate movement, not final destinations
        let dest_screen_x = dest.x as f32 * self.cell_width + self.cell_width / 2.0;
        let dest_screen_y = dest.y as f32 * self.cell_height + self.cell_height / 2.0;

        // Check if we've reached the destination
        let dx_to_dest = dest_screen_x - self.fpos_x;
        let dy_to_dest = dest_screen_y - self.fpos_y;
        let dist_to_dest = (dx_to_dest * dx_to_dest + dy_to_dest * dy_to_dest).sqrt();

        // Reached destination if we're very close
        if dist_to_dest < 2.0 {
            // Calculate the destination sub-cell
            let dest_subcell = SubCellCoord::from_screen_pos(
                dest_screen_x,
                dest_screen_y,
                self.cell_width,
                self.cell_height,
                self.subcell_grid_size,
            );

            // Release all reservations except the destination sub-cell
            // Release current if it's not the destination
            if current != dest_subcell {
                reservation_manager.release(current, self.id);
            }
            // Release reserved if it exists and is not the destination
            if let Some(reserved) = self.reserved_subcell {
                if reserved != dest_subcell {
                    reservation_manager.release(reserved, self.id);
                }
            }
            // Release all extra reserved cells
            for extra in &self.extra_reserved_subcells {
                if *extra != dest_subcell {
                    reservation_manager.release(*extra, self.id);
                }
            }
            self.extra_reserved_subcells.clear();

            // Keep only the destination sub-cell reserved
            self.current_subcell = Some(dest_subcell);
            self.subcell_destination = None;
            self.reserved_subcell = None;
            return true;
        }

        // If we have a reserved sub-cell, move toward it
        if let Some(reserved) = self.reserved_subcell {
            let (reserved_x, reserved_y) = reserved.to_screen_center(self.cell_width, self.cell_height);
            let (current_x, current_y) = current.to_screen_center(self.cell_width, self.cell_height);

            // Calculate distances
            let dx_to_reserved = reserved_x - self.fpos_x;
            let dy_to_reserved = reserved_y - self.fpos_y;
            let dist_to_reserved = (dx_to_reserved * dx_to_reserved + dy_to_reserved * dy_to_reserved).sqrt();

            let dx_to_current = current_x - self.fpos_x;
            let dy_to_current = current_y - self.fpos_y;
            let dist_to_current = (dx_to_current * dx_to_current + dy_to_current * dy_to_current).sqrt();

            // If closer to reserved than current, switch
            if dist_to_reserved <= dist_to_current {
                // Only release old current sub-cell if it's different from reserved
                if current != reserved {
                    reservation_manager.release(current, self.id);
                }
                // Release extra reserved cells (from square reservation)
                for extra in &self.extra_reserved_subcells {
                    reservation_manager.release(*extra, self.id);
                }
                self.extra_reserved_subcells.clear();

                // Update current to reserved
                self.current_subcell = Some(reserved);
                self.reserved_subcell = None;

                // No snapping - let actor position flow smoothly
                // Don't recurse - process next sub-cell reservation on next frame
                return false;
            }

            // Move toward reserved sub-cell
            let movement = self.speed * delta_time;
            if dist_to_reserved > 0.001 {
                self.fpos_x += (dx_to_reserved / dist_to_reserved) * movement;
                self.fpos_y += (dy_to_reserved / dist_to_reserved) * movement;
            }

            return false;
        }

        // No reserved sub-cell - we're centered on current
        // Try to reserve next sub-cell toward destination
        let (current_center_x, current_center_y) = current.to_screen_center(self.cell_width, self.cell_height);
        let center_dx = current_center_x - self.fpos_x;
        let center_dy = current_center_y - self.fpos_y;
        let dist_to_center = (center_dx * center_dx + center_dy * center_dy).sqrt();

        // If not centered yet, move toward center
        if dist_to_center > 1.0 {
            let movement = self.speed * delta_time;
            if dist_to_center > 0.001 {
                self.fpos_x += (center_dx / dist_to_center) * movement;
                self.fpos_y += (center_dy / dist_to_center) * movement;
            }
            return false;
        }

        // We're centered - try to reserve next sub-cell
        // Calculate direction to destination
        let dir_x = dx_to_dest;
        let dir_y = dy_to_dest;

        // Calculate the destination sub-cell
        let dest_subcell = SubCellCoord::from_screen_pos(dest_screen_x, dest_screen_y, self.cell_width, self.cell_height, self.subcell_grid_size);

        // Check if we're already at the destination sub-cell
        if current == dest_subcell {
            // We're at destination sub-cell - stop here
            return true;
        }

        // STEP 1: Try to reserve a 2x2 square in the primary direction
        if let Some((best, additional_cells)) = crate::subcell::find_square_reservation(
            &current,
            dir_x,
            dir_y,
            self.cell_width,
            self.cell_height,
        ) {
            // Try to reserve all four cells atomically
            let mut all_cells = vec![best];
            all_cells.extend_from_slice(&additional_cells);

            if reservation_manager.try_reserve_multiple(&all_cells, self.id) {
                // Successfully reserved square - move to best cell
                self.reserved_subcell = Some(best);
                // Track the additional 3 cells
                self.extra_reserved_subcells = additional_cells.to_vec();
                return false;
            }
        }

        // STEP 2: Fallback to current behavior (single cell reservation)
        // Get candidate neighbors in priority order
        let candidates = crate::subcell::find_best_neighbors(
            &current,
            dir_x,
            dir_y,
            self.cell_width,
            self.cell_height,
        );

        // Try to reserve one of the candidates
        for candidate in &candidates {
            if reservation_manager.try_reserve(*candidate, self.id) {
                self.reserved_subcell = Some(*candidate);
                // Clear extra reserved cells (we're doing single-cell now)
                self.extra_reserved_subcells.clear();
                return false;
            }
        }

        // No neighbor could be reserved
        // Check if the destination sub-cell is one of our blocked candidates
        if candidates.contains(&dest_subcell) {
            // Destination sub-cell is blocked - stop here
            return true;
        }

        // Wait for a path to open up
        false
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
            0,  // actor ID
            5.0 * cell_width + cell_width / 2.0,  // Center of cell (5, 5)
            5.0 * cell_height + cell_height / 2.0,
            10.0,  // Size smaller than cell
            100.0,
            6.0,  // collision_radius
            cell_width,
            cell_height,
            3,  // subcell_grid_size
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
            0,  // actor ID
            5.0 * cell_width + cell_width,  // On the border
            5.0 * cell_height + cell_height / 2.0,
            10.0,
            100.0,
            6.0,  // collision_radius
            cell_width,
            cell_height,
            3,  // subcell_grid_size
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
            0,  // actor ID
            5.0 * cell_width + cell_width / 2.0,
            5.0 * cell_height + cell_height,  // On the border
            10.0,
            100.0,
            6.0,  // collision_radius
            cell_width,
            cell_height,
            3,  // subcell_grid_size
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
            0,  // actor ID
            5.0 * cell_width + cell_width,
            5.0 * cell_height + cell_height,
            10.0,
            100.0,
            6.0,  // collision_radius
            cell_width,
            cell_height,
            3,  // subcell_grid_size
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
        let mut actor = Actor::new(0, start_x, start_y, 10.0, 100.0, 6.0, cell_width, cell_height, 3);

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
