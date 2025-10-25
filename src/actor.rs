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
    /// Sub-cell offset (in sub-cell units, e.g., 0.5 for half-cell shift)
    pub subcell_offset_x: f32,
    pub subcell_offset_y: f32,
    /// Current sub-cell position
    pub current_subcell: Option<SubCellCoord>,
    /// Reserved sub-cell that actor is moving toward
    pub reserved_subcell: Option<SubCellCoord>,
    /// Additional reserved sub-cells (for square reservations)
    pub extra_reserved_subcells: Vec<SubCellCoord>,
    /// Final destination for sub-cell movement (cell-level, NOT sub-cell level)
    pub subcell_destination: Option<Position>,
    /// Movement tracking - records positions at key events (reserve, release, reach center)
    pub movement_track: Vec<(f32, f32)>,
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
    pub fn new(id: usize, fpos_x: f32, fpos_y: f32, size: f32, speed: f32, collision_radius: f32, cell_width: f32, cell_height: f32, subcell_grid_size: i32, subcell_offset_x: f32, subcell_offset_y: f32) -> Self {
        // Initialize sub-cell position with offset
        let current_subcell = Some(SubCellCoord::from_screen_pos_with_offset(
            fpos_x,
            fpos_y,
            cell_width,
            cell_height,
            subcell_grid_size,
            subcell_offset_x,
            subcell_offset_y,
        ));

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
            subcell_offset_x,
            subcell_offset_y,
            current_subcell,
            reserved_subcell: None,
            extra_reserved_subcells: Vec::new(),
            subcell_destination: None,
            movement_track: Vec::new(),
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

    /// Check if a move from current to target is diagonal
    fn is_diagonal_move(current: &SubCellCoord, target: &SubCellCoord) -> bool {
        let dx = (target.cell_x - current.cell_x).abs() + (target.sub_x - current.sub_x).abs();
        let dy = (target.cell_y - current.cell_y).abs() + (target.sub_y - current.sub_y).abs();

        // Diagonal if both dx and dy are non-zero
        dx > 0 && dy > 0
    }

    /// Find an anchor cell (horizontal or vertical from current) on the path to diagonal target
    /// Returns the anchor that shares either row or column with current and is adjacent to target
    fn find_anchor_cell(current: &SubCellCoord, target: &SubCellCoord) -> Option<SubCellCoord> {
        // For a diagonal move, we have two possible anchors:
        // 1. Horizontal anchor: same row as current, same column as target
        // 2. Vertical anchor: same column as current, same row as target

        // Try horizontal anchor (move horizontally first, then diagonally)
        let h_anchor = SubCellCoord::new(
            target.cell_x,
            current.cell_y,
            target.sub_x,
            current.sub_y,
            current.grid_size,
        );

        // Try vertical anchor (move vertically first, then diagonally)
        let v_anchor = SubCellCoord::new(
            current.cell_x,
            target.cell_y,
            current.sub_x,
            target.sub_y,
            current.grid_size,
        );

        // Prefer the anchor that's actually adjacent to current (single step away)
        // Check if h_anchor is a neighbor of current
        let current_neighbors = current.get_neighbors();
        if current_neighbors.contains(&h_anchor) {
            return Some(h_anchor);
        }
        if current_neighbors.contains(&v_anchor) {
            return Some(v_anchor);
        }

        // If neither is a direct neighbor, prefer horizontal
        Some(h_anchor)
    }

    /// Set sub-cell destination for movement (cell-level)
    pub fn set_subcell_destination(&mut self, dest: Position) {
        self.subcell_destination = Some(dest);
        // Initialize current sub-cell if not set
        if self.current_subcell.is_none() {
            self.current_subcell = Some(SubCellCoord::from_screen_pos_with_offset(
                self.fpos_x,
                self.fpos_y,
                self.cell_width,
                self.cell_height,
                self.subcell_grid_size,
                self.subcell_offset_x,
                self.subcell_offset_y,
            ));
        }
    }

    /// Check if a diagonal move would create a counter-diagonal crossing
    /// Returns true if crossing detected (should block move)
    fn check_anti_cross(
        from: &SubCellCoord,
        to: &SubCellCoord,
        reservation_manager: &crate::subcell::SubCellReservationManager,
        actor_id: usize,
    ) -> bool {
        // Check if this is a diagonal move
        if !Self::is_diagonal_move(from, to) {
            return false; // Not diagonal, no crossing possible
        }

        // Get counter-diagonal cells
        let counter_diag = crate::subcell::get_counter_diagonal_subcells(from, to);
        let owner1 = reservation_manager.get_owner(&counter_diag[0]);
        let owner2 = reservation_manager.get_owner(&counter_diag[1]);

        // Block if SAME other actor owns BOTH counter-diagonal cells
        if let (Some(id1), Some(id2)) = (owner1, owner2) {
            if id1 == id2 && id1 != actor_id {
                return true; // Crossing detected
            }
        }

        false // No crossing
    }

    /// Try to reserve diagonal sub-cell with H/V anchor (triangle formation)
    /// Returns true if reservation succeeded, false if blocked
    ///
    /// For DestinationDirect mode: diagonal moves reserve 2 sub-cells:
    /// 1. Best diagonal toward destination
    /// 2. One H/V anchor (horizontal or vertical from current)
    /// This creates a triangle: current + diagonal + anchor
    ///
    /// # Parameters
    /// - `previous_current`: Optional previous position for anti-cross check
    fn try_reserve_diagonal_with_anchor(
        &mut self,
        current: &SubCellCoord,
        previous_current: Option<&SubCellCoord>,
        dir_x: f32,
        dir_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_anti_cross: bool,
        track_movement: bool,
    ) -> bool {
        let neighbors = current.get_neighbors();

        // Collect diagonal candidates sorted by alignment
        let mut diagonal_candidates: Vec<(SubCellCoord, f32)> = neighbors
            .iter()
            .filter(|n| Self::is_diagonal_move(current, n))
            .map(|n| {
                let score = current.alignment_score(
                    n,
                    dir_x,
                    dir_y,
                    self.cell_width,
                    self.cell_height,
                );
                (*n, score)
            })
            .collect();

        diagonal_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Try each diagonal with its H/V anchors
        for (diagonal, _score) in &diagonal_candidates {
            // Anti-cross check for diagonal
            if enable_anti_cross {
                if Self::check_anti_cross(current, diagonal, reservation_manager, self.id) {
                    continue;
                }
                if let Some(prev) = previous_current {
                    if Self::check_anti_cross(prev, current, reservation_manager, self.id) {
                        continue;
                    }
                }
            }

            // Find H/V anchor cells for this diagonal
            // For a diagonal move, we need one H or V anchor
            if let Some(anchor) = Self::find_anchor_cell(current, diagonal) {
                // Try to reserve both diagonal and anchor atomically
                if reservation_manager.try_reserve_multiple(&[*diagonal, anchor], self.id) {
                    self.reserved_subcell = Some(*diagonal);
                    self.extra_reserved_subcells = vec![anchor];
                    if track_movement {
                        self.movement_track.push((self.fpos_x, self.fpos_y));
                    }
                    println!("[RESERVE] Actor {} DIAGONAL+ANCHOR: reserved={:?} anchor={:?} (toward dest)",
                        self.id, diagonal, anchor);
                    return true;
                }
            }
        }

        println!("[RESERVE] Actor {} DIAGONAL+ANCHOR: ALL BLOCKED (tried {} candidates)",
            self.id, diagonal_candidates.len());
        false
    }

    /// Try to reserve H/V sub-cell (fallback when diagonal blocked)
    /// Returns true if reservation succeeded
    fn try_reserve_horizontal_vertical(
        &mut self,
        current: &SubCellCoord,
        dir_x: f32,
        dir_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        track_movement: bool,
    ) -> bool {
        let neighbors = current.get_neighbors();

        // Collect H/V candidates sorted by alignment
        let mut hv_candidates: Vec<(SubCellCoord, f32)> = neighbors
            .iter()
            .filter(|n| !Self::is_diagonal_move(current, n))
            .map(|n| {
                let score = current.alignment_score(
                    n,
                    dir_x,
                    dir_y,
                    self.cell_width,
                    self.cell_height,
                );
                (*n, score)
            })
            .collect();

        hv_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Try to reserve best H/V
        for (candidate, _score) in &hv_candidates {
            if reservation_manager.try_reserve(*candidate, self.id) {
                self.reserved_subcell = Some(*candidate);
                self.extra_reserved_subcells.clear();
                if track_movement {
                    self.movement_track.push((self.fpos_x, self.fpos_y));
                }
                println!("[RESERVE] Actor {} H/V FALLBACK: reserved={:?} (diagonal blocked)",
                    self.id, candidate);
                return true;
            }
        }

        println!("[RESERVE] Actor {} H/V FALLBACK: ALL BLOCKED (tried {} candidates)",
            self.id, hv_candidates.len());
        false
    }

    /// Try to reserve next sub-cell toward destination
    /// Returns true if reservation succeeded, false if all candidates blocked
    ///
    /// # Parameters
    /// - `previous_current`: Optional previous position before current (for early reservation anti-cross check)
    ///   When early reservation is enabled, we need to check crossing from the true previous position,
    ///   not just from the newly-updated current position.
    fn try_reserve_next_subcell(
        &mut self,
        current: &SubCellCoord,
        previous_current: Option<&SubCellCoord>,
        dir_x: f32,
        dir_y: f32,
        dest_screen_x: f32,
        dest_screen_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_square_reservation: bool,
        enable_diagonal_constraint: bool,
        enable_no_diagonal: bool,
        enable_anti_cross: bool,
        enable_basic3: bool,
        enable_basic3_anti_cross: bool,
        filter_backward: bool,
        basic3_fallback_enabled: bool,
        track_movement: bool,
    ) -> bool {
        // Calculate the destination sub-cell
        let dest_subcell = SubCellCoord::from_screen_pos_with_offset(
            dest_screen_x,
            dest_screen_y,
            self.cell_width,
            self.cell_height,
            self.subcell_grid_size,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );

        // Check if we're already at the destination sub-cell
        if *current == dest_subcell {
            return false; // Already at destination
        }

        // STEP 1: Try to reserve a 2x2 square in the primary direction (if enabled)
        if enable_square_reservation {
            if let Some((best, additional_cells)) = crate::subcell::find_square_reservation(
                current,
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
                    // Record position when reserving
                    if track_movement {
                        self.movement_track.push((self.fpos_x, self.fpos_y));
                    }
                    return true;
                }
            }
        }

        // STEP 2: Fallback to single cell reservation (with optional diagonal constraint)
        // Get candidate neighbors in priority order
        let candidates = if enable_basic3 || enable_basic3_anti_cross {
            // Use limited 3-candidate search with monotonic distance filtering
            crate::subcell::find_best_3_neighbors(
                current,
                dir_x,
                dir_y,
                self.cell_width,
                self.cell_height,
                filter_backward,
                dest_screen_x,
                dest_screen_y,
                true,  // Always use monotonic filter for Basic3 modes
                basic3_fallback_enabled,  // Allow fallback based on toggle
            )
        } else {
            // Use standard 5-candidate search
            crate::subcell::find_best_neighbors(
                current,
                dir_x,
                dir_y,
                self.cell_width,
                self.cell_height,
                filter_backward,
            )
        };

        // Try to reserve one of the candidates
        for candidate in &candidates {
            // Check if this is a diagonal move
            let is_diagonal = Self::is_diagonal_move(current, candidate);

            // NoDiagonal mode: skip all diagonal candidates
            if enable_no_diagonal && is_diagonal {
                continue;
            }

            // AntiCross mode: check for counter-diagonal crossing
            if (enable_anti_cross || enable_basic3_anti_cross) && is_diagonal {
                // When early reservation is enabled and we have a previous position,
                // we need to check BOTH transitions for crossing:
                // 1. previous_current → current (the just-completed move)
                // 2. current → candidate (the move we're about to make)

                // Check the immediate move: current → candidate
                if Self::check_anti_cross(current, candidate, reservation_manager, self.id) {
                    continue; // Crossing detected in immediate move
                }

                // If we have a previous position (early reservation scenario),
                // also check the just-completed transition: previous → current
                // This ensures we catch crossings that span across the early reservation boundary
                if let Some(prev) = previous_current {
                    if Self::check_anti_cross(prev, current, reservation_manager, self.id) {
                        // The just-completed move created a crossing
                        // We shouldn't allow further moves that could compound this
                        continue;
                    }
                }
            }

            if enable_diagonal_constraint && is_diagonal {
                // Diagonal mode: must also reserve H or V anchor
                // Try to find and reserve an anchor cell (horizontal or vertical from current)
                if let Some(anchor) = Self::find_anchor_cell(current, candidate) {
                    // Try to reserve both anchor and diagonal atomically
                    if reservation_manager.try_reserve_multiple(&[anchor, *candidate], self.id) {
                        self.reserved_subcell = Some(*candidate);
                        // Track the anchor as extra reservation
                        self.extra_reserved_subcells = vec![anchor];
                        // Record position when reserving
                        if track_movement {
                            self.movement_track.push((self.fpos_x, self.fpos_y));
                        }
                        return true;
                    }
                }
                // If we can't reserve with anchor, skip this diagonal candidate
                continue;
            } else {
                // Non-diagonal or diagonal constraint disabled: single reservation
                if reservation_manager.try_reserve(*candidate, self.id) {
                    self.reserved_subcell = Some(*candidate);
                    // Clear extra reserved cells (single-cell only)
                    self.extra_reserved_subcells.clear();
                    // Record position when reserving
                    if track_movement {
                        self.movement_track.push((self.fpos_x, self.fpos_y));
                    }
                    return true;
                }
            }
        }

        // No neighbor could be reserved
        false
    }

    /// Update sub-cell movement
    /// Returns true if destination reached, false otherwise
    ///
    /// This implements the sub-cell movement algorithm:
    /// 1. Move toward reserved sub-cell
    /// 2. When closer to reserved than current, switch current to reserved
    /// 3. When centered on current (or immediately if early_reservation), try to reserve next sub-cell toward destination
    /// 4. Use fallback neighbors if preferred sub-cell is occupied
    ///
    /// # Parameters
    /// - `delta_time`: Time elapsed since last frame
    /// - `reservation_manager`: Manager for sub-cell reservations
    /// - `enable_square_reservation`: If true, try to reserve 2x2 square
    /// - `enable_diagonal_constraint`: If true, diagonal moves require H/V anchor
    /// - `enable_no_diagonal`: If true, skip all diagonal moves entirely
    /// - `enable_anti_cross`: If true, block diagonal if same actor owns both counter-diagonal cells
    /// - `enable_basic3`: If true, limit candidate directions to 3 (best + ±45°)
    /// - `enable_basic3_anti_cross`: If true, combine Basic3 with anti-cross checking
    /// - `enable_early_reservation`: If true, reserve immediately after switching current (skip centering)
    /// - `filter_backward`: If true, filter out candidates that move away from destination
    /// - `basic3_fallback_enabled`: If true, Basic3 modes fall back to best move when all filtered
    /// - `track_movement`: If true, record position at key events (reserve, release, reach center)
    pub fn update_subcell(
        &mut self,
        delta_time: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_square_reservation: bool,
        enable_diagonal_constraint: bool,
        enable_no_diagonal: bool,
        enable_anti_cross: bool,
        enable_basic3: bool,
        enable_basic3_anti_cross: bool,
        enable_early_reservation: bool,
        filter_backward: bool,
        basic3_fallback_enabled: bool,
        track_movement: bool,
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
                let c = SubCellCoord::from_screen_pos_with_offset(
                    self.fpos_x,
                    self.fpos_y,
                    self.cell_width,
                    self.cell_height,
                    self.subcell_grid_size,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                );
                self.current_subcell = Some(c);
                // Register the initial current subcell
                reservation_manager.set_current(c, self.id);
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
            let dest_subcell = SubCellCoord::from_screen_pos_with_offset(
                dest_screen_x,
                dest_screen_y,
                self.cell_width,
                self.cell_height,
                self.subcell_grid_size,
                self.subcell_offset_x,
                self.subcell_offset_y,
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
            let (reserved_x, reserved_y) = reserved.to_screen_center_with_offset(
                self.cell_width,
                self.cell_height,
                self.subcell_offset_x,
                self.subcell_offset_y,
            );
            let (current_x, current_y) = current.to_screen_center_with_offset(
                self.cell_width,
                self.cell_height,
                self.subcell_offset_x,
                self.subcell_offset_y,
            );

            // Calculate distances
            let dx_to_reserved = reserved_x - self.fpos_x;
            let dy_to_reserved = reserved_y - self.fpos_y;
            let dist_to_reserved = (dx_to_reserved * dx_to_reserved + dy_to_reserved * dy_to_reserved).sqrt();

            let dx_to_current = current_x - self.fpos_x;
            let dy_to_current = current_y - self.fpos_y;
            let dist_to_current = (dx_to_current * dx_to_current + dy_to_current * dy_to_current).sqrt();

            // If closer to reserved than current, switch
            if dist_to_reserved <= dist_to_current {
                // Save the previous current for anti-cross checking in early reservation
                let previous_current = current;

                // Only release old current sub-cell if it's different from reserved
                if current != reserved {
                    reservation_manager.release(current, self.id);
                }
                // Release extra reserved cells (from square reservation)
                for extra in &self.extra_reserved_subcells {
                    reservation_manager.release(*extra, self.id);
                }
                self.extra_reserved_subcells.clear();

                // Update current to reserved (reached subcell center)
                self.current_subcell = Some(reserved);
                self.reserved_subcell = None;

                // Register the new current subcell with the manager
                reservation_manager.set_current(reserved, self.id);

                // Record position when reaching subcell center / releasing
                if track_movement {
                    self.movement_track.push((self.fpos_x, self.fpos_y));
                }

                // If early reservation enabled, immediately try to reserve next cell
                // This allows actor to continue moving without centering first
                if enable_early_reservation {
                    // Fall through to reservation logic below (don't return yet)
                    // Update current reference for reservation attempt
                    let current = reserved;

                    // Check if we're already at destination sub-cell
                    let dest_subcell = SubCellCoord::from_screen_pos_with_offset(
                        dest_screen_x,
                        dest_screen_y,
                        self.cell_width,
                        self.cell_height,
                        self.subcell_grid_size,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    if current == dest_subcell {
                        return true;
                    }

                    // Attempt reservation (code duplicated from centering section below)
                    // This is intentional to allow immediate reservation without centering
                    // Pass previous_current for anti-cross check
                    self.try_reserve_next_subcell(
                        &current,
                        Some(&previous_current), // Pass previous position for anti-cross check
                        dx_to_dest,
                        dy_to_dest,
                        dest_screen_x,
                        dest_screen_y,
                        reservation_manager,
                        enable_square_reservation,
                        enable_diagonal_constraint,
                        enable_no_diagonal,
                        enable_anti_cross,
                        enable_basic3,
                        enable_basic3_anti_cross,
                        filter_backward,
                        basic3_fallback_enabled,
                        track_movement,
                    );
                }

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
        let (current_center_x, current_center_y) = current.to_screen_center_with_offset(
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );
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

        // Try to reserve next sub-cell using helper method
        // No previous position needed here (not early reservation)
        self.try_reserve_next_subcell(
            &current,
            None, // No previous position in normal centering flow
            dir_x,
            dir_y,
            dest_screen_x,
            dest_screen_y,
            reservation_manager,
            enable_square_reservation,
            enable_diagonal_constraint,
            enable_no_diagonal,
            enable_anti_cross,
            enable_basic3,
            enable_basic3_anti_cross,
            filter_backward,
            basic3_fallback_enabled,
            track_movement,
        );

        // Wait for a path to open up or continue moving
        false
    }

    /// Update sub-cell movement with destination-direct strategy
    /// Returns true if destination reached, false otherwise
    ///
    /// This implements destination-oriented movement where:
    /// 1. With diagonal reservation: Move freely toward destination within rectangular area
    /// 2. With H/V reservation: Move directly to reserved sub-cell center
    /// 3. No reservation: Move to current center if closer, else wait
    /// 4. Uses same reservation logic as standard sub-cell movement
    ///
    /// # Parameters
    /// - Same as `update_subcell`
    pub fn update_subcell_destination_direct(
        &mut self,
        delta_time: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_square_reservation: bool,
        enable_diagonal_constraint: bool,
        enable_no_diagonal: bool,
        enable_anti_cross: bool,
        enable_basic3: bool,
        enable_basic3_anti_cross: bool,
        enable_early_reservation: bool,
        filter_backward: bool,
        basic3_fallback_enabled: bool,
        track_movement: bool,
    ) -> bool {
        // ALWAYS log first 100 frames for actor 0 to debug GUI freeze
        static mut FRAME_COUNT: u32 = 0;
        static mut LOGGED_NO_DEST: bool = false;
        let always_trace = unsafe {
            if self.id == 0 {
                FRAME_COUNT += 1;
                FRAME_COUNT <= 100
            } else {
                false
            }
        };

        // Check if we have a destination
        let dest = match self.subcell_destination {
            Some(d) => {
                if always_trace {
                    println!("[DestDirect ENTRY] Actor {} frame, dest=({},{})", self.id, d.x, d.y);
                }
                unsafe { LOGGED_NO_DEST = false; } // Reset flag when destination is set
                d
            },
            None => {
                if always_trace && unsafe { !LOGGED_NO_DEST } {
                    println!("[DestDirect] Actor {} has NO DESTINATION", self.id);
                    unsafe { LOGGED_NO_DEST = true; }
                }
                return true; // No destination, we're done
            }
        };

        // Ensure we have current sub-cell
        let current = match self.current_subcell {
            Some(c) => c,
            None => {
                // Initialize from current position
                let c = SubCellCoord::from_screen_pos_with_offset(
                    self.fpos_x,
                    self.fpos_y,
                    self.cell_width,
                    self.cell_height,
                    self.subcell_grid_size,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                );
                self.current_subcell = Some(c);
                // Register the initial current subcell
                reservation_manager.set_current(c, self.id);
                c
            }
        };

        // Get destination screen position (cell level)
        let dest_screen_x = dest.x as f32 * self.cell_width + self.cell_width / 2.0;
        let dest_screen_y = dest.y as f32 * self.cell_height + self.cell_height / 2.0;

        // Check if we've reached the destination
        let dx_to_dest = dest_screen_x - self.fpos_x;
        let dy_to_dest = dest_screen_y - self.fpos_y;
        let dist_to_dest = (dx_to_dest * dx_to_dest + dy_to_dest * dy_to_dest).sqrt();

        // Reached destination if we're very close
        if dist_to_dest < 2.0 {
            // Calculate the destination sub-cell
            let dest_subcell = SubCellCoord::from_screen_pos_with_offset(
                dest_screen_x,
                dest_screen_y,
                self.cell_width,
                self.cell_height,
                self.subcell_grid_size,
                self.subcell_offset_x,
                self.subcell_offset_y,
            );

            // Release all reservations except the destination sub-cell
            if current != dest_subcell {
                reservation_manager.release(current, self.id);
            }
            if let Some(reserved) = self.reserved_subcell {
                if reserved != dest_subcell {
                    reservation_manager.release(reserved, self.id);
                }
            }
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

        // Get anchor sub-cell (first extra reserved cell, if any)
        let anchor_subcell = self.extra_reserved_subcells.first();

        // Calculate optimal target position based on reservation state
        let (target_x, target_y) = crate::subcell::calculate_optimal_boundary(
            &current,
            self.reserved_subcell.as_ref(),
            anchor_subcell,
            dest_screen_x,
            dest_screen_y,
            self.fpos_x,
            self.fpos_y,
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );

        // Calculate distance to target (used for both movement and switching)
        let dx_to_target = target_x - self.fpos_x;
        let dy_to_target = target_y - self.fpos_y;
        let dist_to_target = (dx_to_target * dx_to_target + dy_to_target * dy_to_target).sqrt();

        // TRACE: Log movement state for actor 0
        if always_trace || (self.id == 0 && track_movement) {
            println!("[DestDirect Frame] actor_pos=({:.1},{:.1}) target=({:.1},{:.1}) dist={:.2}",
                self.fpos_x, self.fpos_y, target_x, target_y, dist_to_target);
            println!("  reserved={:?} anchor={:?} dest=({},{})",
                self.reserved_subcell.is_some(),
                self.extra_reserved_subcells.len(),
                dest.x, dest.y);
        }

        // Move toward target
        let movement = self.speed * delta_time;
        if dist_to_target > 0.001 {
            let move_dist = movement.min(dist_to_target);
            self.fpos_x += (dx_to_target / dist_to_target) * move_dist;
            self.fpos_y += (dy_to_target / dist_to_target) * move_dist;

            if always_trace || (self.id == 0 && track_movement) {
                println!("  MOVED: move_dist={:.2} new_pos=({:.1},{:.1})", move_dist, self.fpos_x, self.fpos_y);
            }
        } else if always_trace || (self.id == 0 && track_movement) {
            println!("  NOT MOVING: dist_to_target={:.4} < 0.001", dist_to_target);
        }

        // Check if we should switch from reserved to current (triangle-based switching)
        if let Some(reserved) = self.reserved_subcell {
            // Triangle-based movement: switch based on boundary proximity, not center proximity
            let should_switch = if enable_early_reservation {
                // Early mode: Switch when closer to boundary than to current center
                let (current_x, current_y) = current.to_screen_center_with_offset(
                    self.cell_width,
                    self.cell_height,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                );
                let dx_to_curr = current_x - self.fpos_x;
                let dy_to_curr = current_y - self.fpos_y;
                let dist_to_current_center = (dx_to_curr * dx_to_curr + dy_to_curr * dy_to_curr).sqrt();

                let switch = dist_to_target < dist_to_current_center;
                if self.id == 0 && track_movement {
                    println!("  EARLY CHECK: dist_to_boundary={:.2} dist_to_center={:.2} switch={}",
                        dist_to_target, dist_to_current_center, switch);
                }
                switch
            } else {
                // Standard mode: Switch when at boundary (cannot move further)
                let switch = dist_to_target < 0.5;
                if self.id == 0 && track_movement {
                    println!("  STANDARD CHECK: dist_to_boundary={:.2} < 0.5? switch={}", dist_to_target, switch);
                }
                switch
            };

            if should_switch {
                if self.id == 0 && track_movement {
                    println!("  SWITCHING: current={:?} -> reserved={:?}", current, reserved);
                }
                let previous_current = current;

                // Release old current sub-cell if different
                if current != reserved {
                    reservation_manager.release(current, self.id);
                }
                // Release extra reserved cells
                for extra in &self.extra_reserved_subcells {
                    reservation_manager.release(*extra, self.id);
                }
                self.extra_reserved_subcells.clear();

                // Update current to reserved
                self.current_subcell = Some(reserved);
                self.reserved_subcell = None;

                // Register the new current subcell
                reservation_manager.set_current(reserved, self.id);

                // Record position when reaching subcell
                if track_movement {
                    self.movement_track.push((self.fpos_x, self.fpos_y));
                }

                // If early reservation enabled, immediately try to reserve next cell
                if enable_early_reservation {
                    let current = reserved;

                    // Check if at destination sub-cell
                    let dest_subcell = SubCellCoord::from_screen_pos_with_offset(
                        dest_screen_x,
                        dest_screen_y,
                        self.cell_width,
                        self.cell_height,
                        self.subcell_grid_size,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    if current != dest_subcell {
                        // Attempt reservation with previous position for anti-cross check
                        // DestinationDirect: Try diagonal+anchor first, fallback to H/V
                        if !self.try_reserve_diagonal_with_anchor(
                            &current,
                            Some(&previous_current),
                            dx_to_dest,
                            dy_to_dest,
                            reservation_manager,
                            enable_anti_cross,
                            track_movement,
                        ) {
                            // Diagonal failed, try H/V
                            self.try_reserve_horizontal_vertical(
                                &current,
                                dx_to_dest,
                                dy_to_dest,
                                reservation_manager,
                                track_movement,
                            );
                        }
                    }
                }

                return false;
            }
        } else {
            // No reservation - try to reserve next sub-cell
            // DestinationDirect: Try diagonal+anchor first, fallback to H/V
            if self.id == 0 && track_movement {
                println!("  NO RESERVATION: Attempting diagonal+anchor");
            }

            if !self.try_reserve_diagonal_with_anchor(
                &current,
                None,
                dx_to_dest,
                dy_to_dest,
                reservation_manager,
                enable_anti_cross,
                track_movement,
            ) {
                // Diagonal failed, try H/V
                if self.id == 0 && track_movement {
                    println!("  Diagonal failed, trying H/V");
                }
                let success = self.try_reserve_horizontal_vertical(
                    &current,
                    dx_to_dest,
                    dy_to_dest,
                    reservation_manager,
                    track_movement,
                );
                if self.id == 0 && track_movement {
                    println!("  H/V result: {}", success);
                }
            } else if self.id == 0 && track_movement {
                println!("  Diagonal+anchor SUCCESS");
            }
        }

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
            0.0,  // No offset
            0.0,
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
            0.0,  // No offset
            0.0,
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
            0.0,  // No offset
            0.0,
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
            0.0,  // No offset
            0.0,
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
        let mut actor = Actor::new(0, start_x, start_y, 10.0, 100.0, 6.0, cell_width, cell_height, 3, 0.0, 0.0);

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
