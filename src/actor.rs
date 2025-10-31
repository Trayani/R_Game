use crate::Grid;
use crate::pathfinding::Position;
use crate::subcell::SubCellCoord;
use crate::subpoint::SubPoint;
use crate::actor_directives;

/// Affinity for diagonal movement (actor_directing_v2.txt Section B)
/// Determines which anchor subcell to reserve alongside diagonal subcell
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Affinity {
    /// Horizontal-favoring: Ray hits vertical edge first, use horizontal anchor
    Horizontal,
    /// Vertical-favoring: Ray hits horizontal edge first, use vertical anchor
    Vertical,
    /// Both edges hit simultaneously (corner): Choose anchor based on actor position
    Both,
}

/// Type of movement target the actor is using
#[derive(Clone, Copy, Debug, PartialEq)]
enum TargetType {
    /// Direct target: Using locked_target from actor_directing_v2
    Direct,
    /// Subcell point target: Fallback to subcell point (H/V or no reservation)
    SubcellPoint,
}

/// Actor alignment state machine (actor_states.txt Section 3)
/// Tracks progression: NoSubcell → PscAlignment → Idle → Move
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlignmentState {
    /// Actor has no current_subcell yet, attempting to acquire one
    NoSubcell,
    /// Actor moving to center of current_subcell (immune to collision)
    PscAlignment,
    /// Actor at subcell center, ready to reserve next cell
    Idle,
    /// Actor has reservation, moving toward reserved subcell
    Move,
}

/// Cardinal direction in 8-way movement (45° sectors)
#[derive(Clone, Copy, Debug, PartialEq)]
enum CardinalDirection {
    N,   // North (up)
    NE,  // Northeast
    E,   // East (right)
    SE,  // Southeast
    S,   // South (down)
    SW,  // Southwest
    W,   // West (left)
    NW,  // Northwest
}

/// Result of affinity calculation using ray-rectangle intersection
/// (actor_directing_v2.txt Section B: Rectangle Intersection Formula)
#[derive(Clone, Debug)]
pub struct AffinityResult {
    /// Calculated affinity (H/V/Both)
    pub affinity: Affinity,
    /// Target position on rectangle boundary (locked for movement)
    pub target_x: f32,
    pub target_y: f32,
    /// Anchor subcell to reserve (determined by affinity)
    pub anchor: SubCellCoord,
    /// Parametric t value for vertical edge (debug/visualization)
    pub t_vertical: f32,
    /// Parametric t value for horizontal edge (debug/visualization)
    pub t_horizontal: f32,
}

/// Directing information for logging - captures the complete state of a directing decision
#[derive(Clone, Debug)]
pub struct DirectingInfo {
    pub affinity: Affinity,
    pub target_x: f32,
    pub target_y: f32,
    pub reserved: SubCellCoord,
    pub anchor: SubCellCoord,
}

/// PSC selection information for logging - captures PSC switching decision
#[derive(Clone, Debug)]
pub struct PSCSelectionInfo {
    pub old_psc: SubCellCoord,
    pub reserved: SubCellCoord,
    pub reserved_dist: f32,
    pub anchor: Option<SubCellCoord>,  // None for H/V moves
    pub anchor_dist: Option<f32>,
    pub chosen: SubCellCoord,
    pub chosen_name: String,  // "Reserved" or "Anchor"
}

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
    /// Current sub-cell position (migrated to SubPoint flat coordinates)
    pub current_subcell: Option<SubPoint>,
    /// Reserved sub-cell that actor is moving toward
    pub reserved_subcell: Option<SubCellCoord>,
    /// Additional reserved sub-cells (for square reservations)
    pub extra_reserved_subcells: Vec<SubCellCoord>,
    /// Final destination for sub-cell movement (cell-level, NOT sub-cell level)
    pub subcell_destination: Option<Position>,
    /// Movement tracking - records positions at key events (reserve, release, reach center)
    pub movement_track: Vec<(f32, f32)>,

    // Actor Directing V2 (ray-rectangle intersection)
    /// Locked target position from affinity calculation (actor_directing_v2.txt Section B)
    /// Set when diagonal reservation succeeds, cleared when reservation released
    pub locked_target: Option<(f32, f32)>,
    /// Locked affinity for current reservation (debug/visualization)
    pub locked_affinity: Option<Affinity>,
    /// Use actor_directing_v2 ray-rectangle intersection algorithm (default: true)
    /// Set to false to revert to old alignment-based diagonal selection
    pub use_directing_v2: bool,
    /// Last directing decision for logging - cleared after main.rs logs it
    pub last_directing_info: Option<DirectingInfo>,
    /// Last PSC selection for logging - cleared after main.rs logs it
    pub last_psc_selection: Option<PSCSelectionInfo>,
    /// Diagnostic messages to log to action log (cleared after main.rs logs them)
    pub diagnostic_messages: Vec<String>,

    /// Tolerance multiplier for distance rule check (default: 0.6)
    /// Applied as: tolerance = subcell_width * distance_tolerance_multiplier
    /// Set to 0.0 for strict monotonic distance decrease enforcement
    pub distance_tolerance_multiplier: f32,

    /// Enable look-ahead (2-step) evaluation for deadlock breaking (default: true)
    /// When hysteresis detects equidistant candidates, evaluate 2-step paths to break deadlocks
    pub enable_lookahead: bool,

    /// PSC switching threshold - remaining distance ratio (default: 0.5)
    /// Controls when actor switches from current PSC to reserved PSC
    /// 0.0 = switch only when reaching reserved exactly, 0.5 = switch at midpoint
    pub psc_switch_threshold: f32,

    // PSC Alignment State Machine (actor_states.txt)
    /// Current alignment state: NoSubcell → PscAlignment → Idle → Move
    pub alignment_state: AlignmentState,
    /// Target position during PscAlignment (center of current_subcell)
    pub alignment_target: Option<(f32, f32)>,
    /// Distance threshold for considering actor "aligned" to PSC center (default: 2.0 pixels)
    pub alignment_threshold: f32,

    // Direction change tracking (for detecting indirect pathfinding)
    /// Target position from two frames ago
    previous_old_target: Option<(f32, f32)>,
    /// Target position from last frame
    old_target: Option<(f32, f32)>,
    /// Type of target being used
    target_type: TargetType,
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

/// Get the cardinal direction from one point to another using 8-way directional sectors
/// Each sector spans 45 degrees (PI/4 radians)
fn get_cardinal_direction(from_x: f32, from_y: f32, to_x: f32, to_y: f32) -> CardinalDirection {
    use std::f32::consts::PI;

    let dx = to_x - from_x;
    let dy = to_y - from_y;

    // Handle zero vector (no movement)
    if dx.abs() < 0.0001 && dy.abs() < 0.0001 {
        return CardinalDirection::N; // Default to North for zero vector
    }

    // Calculate angle: atan2(dy, dx) returns angle in radians
    // East = 0°, North = +90° (PI/2), West = ±180° (PI), South = -90° (-PI/2)
    let angle = dy.atan2(dx);

    // Map angle to 8 directions using 45° sectors
    // Each direction spans from -22.5° to +22.5° around its center angle
    if angle >= -PI / 8.0 && angle < PI / 8.0 {
        CardinalDirection::E  // 0° (east/right)
    } else if angle >= PI / 8.0 && angle < 3.0 * PI / 8.0 {
        CardinalDirection::SE  // 45° (southeast)
    } else if angle >= 3.0 * PI / 8.0 && angle < 5.0 * PI / 8.0 {
        CardinalDirection::S  // 90° (south/down)
    } else if angle >= 5.0 * PI / 8.0 && angle < 7.0 * PI / 8.0 {
        CardinalDirection::SW  // 135° (southwest)
    } else if angle >= 7.0 * PI / 8.0 || angle < -7.0 * PI / 8.0 {
        CardinalDirection::W  // 180° (west/left)
    } else if angle >= -7.0 * PI / 8.0 && angle < -5.0 * PI / 8.0 {
        CardinalDirection::NW  // -135° (northwest)
    } else if angle >= -5.0 * PI / 8.0 && angle < -3.0 * PI / 8.0 {
        CardinalDirection::N  // -90° (north/up)
    } else {
        CardinalDirection::NE  // -45° (northeast)
    }
}

impl Actor {
    /// Create a new actor at the given floating-point position
    pub fn new(id: usize, fpos_x: f32, fpos_y: f32, size: f32, speed: f32, collision_radius: f32, cell_width: f32, cell_height: f32, subcell_grid_size: i32, subcell_offset_x: f32, subcell_offset_y: f32, enable_lookahead: bool, psc_switch_threshold: f32) -> Self {
        // Start with no current_subcell - actor will reserve one on first update
        // This ensures proper reservation through the reservation manager
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
            current_subcell: None,  // Will be reserved on first update
            reserved_subcell: None,
            extra_reserved_subcells: Vec::new(),
            subcell_destination: None,
            movement_track: Vec::new(),
            // Actor Directing V2 - default enabled
            locked_target: None,
            locked_affinity: None,
            use_directing_v2: true,
            last_directing_info: None,
            last_psc_selection: None,
            diagnostic_messages: Vec::new(),
            distance_tolerance_multiplier: 0.6,  // Default: 60% of subcell width
            enable_lookahead,
            psc_switch_threshold,
            // PSC Alignment State Machine - start in NoSubcell state
            alignment_state: AlignmentState::NoSubcell,
            alignment_target: None,  // Will be set when subcell is reserved
            alignment_threshold: 2.0,  // 2.0 pixels - per actor_states.txt PSC_ALIGNMENT_THRESHOLD
            previous_old_target: None,
            old_target: None,
            target_type: TargetType::SubcellPoint,  // Default to SubcellPoint
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

    /// Generate spiral offsets for searching nearby cells
    /// Returns offsets in order: (0,0), then ring 1, then ring 2, etc.
    /// Ring N contains all cells at Chebyshev distance N from origin
    ///
    /// Example for radius=2:
    /// Ring 0: (0,0)
    /// Ring 1: (1,0), (1,1), (0,1), (-1,1), (-1,0), (-1,-1), (0,-1), (1,-1)
    /// Ring 2: (2,-1), (2,0), ..., (-2,-2), ...
    fn generate_spiral_offsets(radius: i32) -> Vec<(i32, i32)> {
        let mut offsets = Vec::new();

        // Start with center
        offsets.push((0, 0));

        // Add rings outward
        for ring in 1..=radius {
            // Top edge: from (ring, -ring+1) to (ring, ring)
            for dy in (-ring + 1)..=ring {
                offsets.push((ring, dy));
            }

            // Right edge: from (ring-1, ring) to (-ring, ring)
            for dx in ((-ring)..=(ring - 1)).rev() {
                offsets.push((dx, ring));
            }

            // Bottom edge: from (-ring, ring-1) to (-ring, -ring)
            for dy in ((-ring)..=(ring - 1)).rev() {
                offsets.push((-ring, dy));
            }

            // Left edge: from (-ring+1, -ring) to (ring-1, -ring)
            for dx in (-ring + 1)..ring {
                offsets.push((dx, -ring));
            }
        }

        offsets
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
            self.current_subcell = Some(SubPoint::from_screen_pos_with_offset(
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

    /// Calculate affinity and target position using ray-rectangle intersection
    /// Implements actor_directing_v2.txt Section B: Rectangle Intersection Formula
    ///
    /// # Parameters
    /// - `actor_x`, `actor_y`: Actor's precise float position
    /// - `psc`: Primary subcell (current logical position)
    /// - `diagonal`: Diagonal subcell candidate
    /// - `dest_x`, `dest_y`: Destination position (float coordinates)
    ///
    /// # Returns
    /// `AffinityResult` containing affinity, target position, and anchor subcell
    ///
    /// # Algorithm
    /// 1. Define rectangle bounds between PSC and diagonal (both at grid intersections)
    /// 2. Cast ray from actor position toward destination
    /// 3. Calculate where ray intersects rectangle boundary
    /// 4. Affinity = which edge hits first (vertical → H, horizontal → V)
    /// 5. Target = intersection point on boundary
    /// 6. Anchor = horizontal or vertical neighbor based on affinity
    pub fn calculate_affinity_and_target(
        &self,
        actor_x: f32,
        actor_y: f32,
        psc: &SubCellCoord,
        diagonal: &SubCellCoord,
        dest_x: f32,
        dest_y: f32,
    ) -> AffinityResult {
        const EPSILON: f32 = 1e-6;

        // Step 1: Define rectangle bounds
        // Use offset coordinate system for fluid movement (matches to_screen_center_with_offset)
        // When offset=0.5, subcells are positioned at their actual screen locations in offset system
        let sub_cell_width = self.cell_width / self.subcell_grid_size as f32;
        let sub_cell_height = self.cell_height / self.subcell_grid_size as f32;

        // Calculate subcell positions in offset coordinate system
        let psc_x = psc.cell_x as f32 * self.cell_width + psc.sub_x as f32 * sub_cell_width
            - self.subcell_offset_x * sub_cell_width;
        let psc_y = psc.cell_y as f32 * self.cell_height + psc.sub_y as f32 * sub_cell_height
            - self.subcell_offset_y * sub_cell_height;
        let diag_x = diagonal.cell_x as f32 * self.cell_width + diagonal.sub_x as f32 * sub_cell_width
            - self.subcell_offset_x * sub_cell_width;
        let diag_y = diagonal.cell_y as f32 * self.cell_height + diagonal.sub_y as f32 * sub_cell_height
            - self.subcell_offset_y * sub_cell_height;

        let rect_min_x = psc_x.min(diag_x);
        let rect_max_x = psc_x.max(diag_x);
        let rect_min_y = psc_y.min(diag_y);
        let rect_max_y = psc_y.max(diag_y);

        // Step 2: Calculate destination direction
        let dest_dx = dest_x - actor_x;
        let dest_dy = dest_y - actor_y;
        let dest_len = (dest_dx * dest_dx + dest_dy * dest_dy).sqrt();

        // Handle edge case: actor already at destination
        if dest_len < EPSILON {
            // Choose anchor based on actor position offset from PSC
            let offset_x = (actor_x - psc_x).abs();
            let offset_y = (actor_y - psc_y).abs();
            let anchor = if offset_x > offset_y {
                Self::get_horizontal_anchor(psc, diagonal)
            } else {
                Self::get_vertical_anchor(psc, diagonal)
            };
            return AffinityResult {
                affinity: Affinity::Both,
                target_x: actor_x,
                target_y: actor_y,
                anchor,
                t_vertical: 0.0,
                t_horizontal: 0.0,
            };
        }

        let dest_ray_x = dest_dx / dest_len;
        let dest_ray_y = dest_dy / dest_len;

        // Step 3: Ray-rectangle intersection
        // Calculate t for ALL 4 edges, pick smallest positive t with valid intersection

        let mut t_vertical = f32::INFINITY;
        let mut t_horizontal = f32::INFINITY;

        // Vertical edges - calculate BOTH left and right
        if dest_ray_x.abs() > EPSILON {
            let t_left = (rect_min_x - actor_x) / dest_ray_x;
            let t_right = (rect_max_x - actor_x) / dest_ray_x;

            // Check which intersections are valid (positive t, y within bounds)
            let y_at_left = actor_y + t_left * dest_ray_y;
            let y_at_right = actor_y + t_right * dest_ray_y;

            let left_valid = t_left > 0.0 && y_at_left >= rect_min_y && y_at_left <= rect_max_y;
            let right_valid = t_right > 0.0 && y_at_right >= rect_min_y && y_at_right <= rect_max_y;

            // Pick smallest valid t
            if left_valid && right_valid {
                t_vertical = t_left.min(t_right);
            } else if left_valid {
                t_vertical = t_left;
            } else if right_valid {
                t_vertical = t_right;
            }
        }

        // Horizontal edges - calculate BOTH top and bottom
        if dest_ray_y.abs() > EPSILON {
            let t_top = (rect_min_y - actor_y) / dest_ray_y;
            let t_bottom = (rect_max_y - actor_y) / dest_ray_y;

            // Check which intersections are valid (positive t, x within bounds)
            let x_at_top = actor_x + t_top * dest_ray_x;
            let x_at_bottom = actor_x + t_bottom * dest_ray_x;

            let top_valid = t_top > 0.0 && x_at_top >= rect_min_x && x_at_top <= rect_max_x;
            let bottom_valid = t_bottom > 0.0 && x_at_bottom >= rect_min_x && x_at_bottom <= rect_max_x;

            // Pick smallest valid t
            if top_valid && bottom_valid {
                t_horizontal = t_top.min(t_bottom);
            } else if top_valid {
                t_horizontal = t_top;
            } else if bottom_valid {
                t_horizontal = t_bottom;
            }
        }

        // Handle edge case: no valid intersection found
        if t_vertical.is_infinite() && t_horizontal.is_infinite() {
            // No valid ray-rectangle intersection (shouldn't happen in normal cases)
            // Clamp actor position to rectangle as fallback
            let offset_x = (actor_x - psc_x).abs();
            let offset_y = (actor_y - psc_y).abs();
            let anchor = if offset_x > offset_y {
                Self::get_horizontal_anchor(psc, diagonal)
            } else {
                Self::get_vertical_anchor(psc, diagonal)
            };
            return AffinityResult {
                affinity: Affinity::Both,
                target_x: actor_x.max(rect_min_x).min(rect_max_x),
                target_y: actor_y.max(rect_min_y).min(rect_max_y),
                anchor,
                t_vertical: 0.0,
                t_horizontal: 0.0,
            };
        }

        // Step 4: Determine affinity and target based on which edge hits first
        let affinity: Affinity;
        let target_x: f32;
        let target_y: f32;
        let anchor: SubCellCoord;

        if (t_vertical - t_horizontal).abs() < EPSILON {
            // BOTH: Hits corner (both edges at same t)
            affinity = Affinity::Both;
            target_x = actor_x + dest_ray_x * t_vertical;
            target_y = actor_y + dest_ray_y * t_vertical;

            // Choose anchor based on actor's position offset from PSC
            // If actor is more horizontally offset, use horizontal anchor
            let offset_x = (actor_x - psc_x).abs();
            let offset_y = (actor_y - psc_y).abs();
            anchor = if offset_x > offset_y {
                Self::get_horizontal_anchor(psc, diagonal)
            } else {
                Self::get_vertical_anchor(psc, diagonal)
            };
        } else if t_vertical < t_horizontal {
            // H-affinity: Hits vertical edge first
            affinity = Affinity::Horizontal;
            target_x = actor_x + dest_ray_x * t_vertical;
            target_y = actor_y + dest_ray_y * t_vertical;
            anchor = Self::get_horizontal_anchor(psc, diagonal);
        } else {
            // V-affinity: Hits horizontal edge first
            affinity = Affinity::Vertical;
            target_x = actor_x + dest_ray_x * t_horizontal;
            target_y = actor_y + dest_ray_y * t_horizontal;
            anchor = Self::get_vertical_anchor(psc, diagonal);
        }

        // LOGICAL CLAMPING: Apply affinity-specific clamping
        // - Horizontal affinity: clamp X only (target on vertical edge, Y determined by ray)
        // - Vertical affinity: clamp Y only (target on horizontal edge, X determined by ray)
        // - Both affinity: clamp both (target at corner)
        let target_x = match affinity {
            Affinity::Horizontal | Affinity::Both => target_x.max(rect_min_x).min(rect_max_x),
            Affinity::Vertical => target_x, // No X clamping for vertical affinity
        };
        let target_y = match affinity {
            Affinity::Vertical | Affinity::Both => target_y.max(rect_min_y).min(rect_max_y),
            Affinity::Horizontal => target_y, // No Y clamping for horizontal affinity
        };

        AffinityResult {
            affinity,
            target_x,
            target_y,
            anchor,
            t_vertical,
            t_horizontal,
        }
    }

    /// Calculate affinity and target using simple distance-based algorithm (no ray-rectangle intersection)
    /// This is the expected simple algorithm for DestinationDirect mode
    pub fn calculate_simple_affinity_and_target(
        &self,
        actor_x: f32,
        actor_y: f32,
        psc: &SubCellCoord,
        diagonal: &SubCellCoord,
        dest_x: f32,
        dest_y: f32,
    ) -> AffinityResult {
        const EPSILON: f32 = 1e-6;

        // Step 1: Calculate distances to destination
        let dx = dest_x - actor_x;
        let dy = dest_y - actor_y;
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        // Step 2: Determine affinity based on simple distance comparison
        // If |dx| > |dy|, horizontal affinity (X is the dominant axis)
        // Otherwise, vertical affinity (Y is the dominant axis)
        let affinity = if abs_dx > abs_dy {
            Affinity::Horizontal
        } else {
            Affinity::Vertical
        };

        // Step 3: Get diagonal subcell center position
        let sub_cell_width = self.cell_width / self.subcell_grid_size as f32;
        let sub_cell_height = self.cell_height / self.subcell_grid_size as f32;

        let diag_center_x = diagonal.cell_x as f32 * self.cell_width
            + diagonal.sub_x as f32 * sub_cell_width
            + sub_cell_width / 2.0
            - self.subcell_offset_x * sub_cell_width;
        let diag_center_y = diagonal.cell_y as f32 * self.cell_height
            + diagonal.sub_y as f32 * sub_cell_height
            + sub_cell_height / 2.0
            - self.subcell_offset_y * sub_cell_height;

        // Step 4: Calculate target using straight-line intersection
        let (target_x, target_y) = match affinity {
            Affinity::Horizontal => {
                // Horizontal affinity: target.x = diagonal_subcell.x
                // target.y calculated from straight line through actor to destination
                let target_x = diag_center_x;
                let target_y = if abs_dx < EPSILON {
                    // Avoid division by zero: if dx=0, move vertically
                    actor_y + (target_x - actor_x) * dy.signum() * 1e6
                } else {
                    actor_y + (target_x - actor_x) * dy / dx
                };
                (target_x, target_y)
            },
            Affinity::Vertical => {
                // Vertical affinity: target.y = diagonal_subcell.y
                // target.x calculated from straight line through actor to destination
                let target_y = diag_center_y;
                let target_x = if abs_dy < EPSILON {
                    // Avoid division by zero: if dy=0, move horizontally
                    actor_x + (target_y - actor_y) * dx.signum() * 1e6
                } else {
                    actor_x + (target_y - actor_y) * dx / dy
                };
                (target_x, target_y)
            },
            Affinity::Both => {
                // Should not happen in this simple algorithm
                (diag_center_x, diag_center_y)
            }
        };

        // Step 5: Get anchor based on affinity
        let anchor = match affinity {
            Affinity::Horizontal => Self::get_horizontal_anchor(psc, diagonal),
            Affinity::Vertical => Self::get_vertical_anchor(psc, diagonal),
            Affinity::Both => Self::get_horizontal_anchor(psc, diagonal), // Fallback
        };

        AffinityResult {
            affinity,
            target_x,
            target_y,
            anchor,
            t_vertical: 0.0,    // Not used in simple algorithm
            t_horizontal: 0.0,  // Not used in simple algorithm
        }
    }

    /// Get horizontal anchor for diagonal move (anchor is horizontal neighbor of PSC)
    fn get_horizontal_anchor(psc: &SubCellCoord, diagonal: &SubCellCoord) -> SubCellCoord {
        // Horizontal anchor: shares Y coordinate with PSC, X coordinate with diagonal
        // Example: PSC=(5,5), Diag=(6,4) → Anchor=(6,5)
        SubCellCoord {
            cell_x: diagonal.cell_x,  // X from diagonal
            cell_y: psc.cell_y,        // Y from PSC
            sub_x: diagonal.sub_x,
            sub_y: psc.sub_y,
            grid_size: psc.grid_size,
        }
    }

    /// Get vertical anchor for diagonal move (anchor is vertical neighbor of PSC)
    fn get_vertical_anchor(psc: &SubCellCoord, diagonal: &SubCellCoord) -> SubCellCoord {
        // Vertical anchor: shares X coordinate with PSC, Y coordinate with diagonal
        // Example: PSC=(5,5), Diag=(6,4) → Anchor=(5,4)
        SubCellCoord {
            cell_x: psc.cell_x,         // X from PSC
            cell_y: diagonal.cell_y,    // Y from diagonal
            sub_x: psc.sub_x,
            sub_y: diagonal.sub_y,
            grid_size: psc.grid_size,
        }
    }

    /// Evaluate look-ahead (2-step) path quality from a candidate subcell
    /// Returns the distance from the best 2nd-step neighbor to destination
    /// Returns None if no valid 2nd-step exists (all neighbors blocked/reserved by others)
    ///
    /// This is used to break deadlocks when hysteresis detects equidistant candidates.
    /// By looking one step ahead, we can choose the path that leads to better future positions.
    fn evaluate_lookahead_candidate(
        candidate: &SubCellCoord,
        dest_x: f32,
        dest_y: f32,
        cell_width: f32,
        cell_height: f32,
        subcell_offset_x: f32,
        subcell_offset_y: f32,
        reservation_manager: &crate::subcell::SubCellReservationManager,
        actor_id: usize,
    ) -> Option<f32> {
        // Get all 8 neighbors FROM the candidate position
        let neighbors = candidate.get_neighbors();

        // Calculate direction from candidate to destination for alignment scoring
        let (candidate_center_x, candidate_center_y) = candidate.to_screen_center_with_offset(
            cell_width,
            cell_height,
            subcell_offset_x,
            subcell_offset_y,
        );
        let dir_x = dest_x - candidate_center_x;
        let dir_y = dest_y - candidate_center_y;

        // Filter and score valid neighbors
        let mut best_distance: Option<f32> = None;

        for neighbor in &neighbors {
            // Skip if reserved by ANOTHER actor (we're allowed to move through our own reservations)
            if let Some(reserver_id) = reservation_manager.is_reserved(neighbor) {
                if reserver_id != actor_id {
                    continue; // Reserved by someone else, skip
                }
            }

            // Calculate alignment score (dot product of direction vectors)
            let alignment = neighbor.alignment_score(candidate, dir_x, dir_y, cell_width, cell_height);

            // Skip backward moves (negative alignment)
            if alignment < 0.0 {
                continue;
            }

            // Calculate distance from this 2nd-step position to destination
            let distance = Self::subcell_center_distance_to_destination(
                neighbor,
                dest_x,
                dest_y,
                cell_width,
                cell_height,
                subcell_offset_x,
                subcell_offset_y,
            );

            // Track best (smallest) distance
            best_distance = Some(match best_distance {
                None => distance,
                Some(current_best) => current_best.min(distance),
            });
        }

        best_distance
    }

    /// Calculate Euclidean distance from subcell center to destination
    /// Used for PSC switching logic to determine which subcell is closer to destination
    fn subcell_center_distance_to_destination(
        subcell: &SubCellCoord,
        dest_x: f32,
        dest_y: f32,
        cell_width: f32,
        cell_height: f32,
        subcell_offset_x: f32,
        subcell_offset_y: f32,
    ) -> f32 {
        // Convert subcell to its center position in screen coordinates
        let (center_x, center_y) = subcell.to_screen_center_with_offset(
            cell_width,
            cell_height,
            subcell_offset_x,
            subcell_offset_y,
        );

        // Calculate Euclidean distance to destination
        let dx = dest_x - center_x;
        let dy = dest_y - center_y;
        (dx * dx + dy * dy).sqrt()
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
    pub(crate) fn try_reserve_diagonal_with_anchor(
        &mut self,
        current: &SubCellCoord,
        previous_current: Option<&SubCellCoord>,
        dir_x: f32,
        dir_y: f32,
        dest_screen_x: f32,
        dest_screen_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_anti_cross: bool,
        track_movement: bool,
    ) -> bool {
        // Branch based on actor_directing_v2 feature flag
        if self.use_directing_v2 {
            // NEW: Use ray-rectangle intersection for affinity calculation
            self.try_reserve_diagonal_with_affinity(
                current,
                previous_current,
                dest_screen_x,
                dest_screen_y,
                reservation_manager,
                enable_anti_cross,
                track_movement,
            )
        } else {
            // OLD: Use alignment score approach
            self.try_reserve_diagonal_with_anchor_legacy(
                current,
                previous_current,
                dir_x,
                dir_y,
                dest_screen_x,
                dest_screen_y,
                reservation_manager,
                enable_anti_cross,
                track_movement,
            )
        }
    }

    /// LEGACY: Try to reserve diagonal using alignment score (old algorithm)
    fn try_reserve_diagonal_with_anchor_legacy(
        &mut self,
        current: &SubCellCoord,
        previous_current: Option<&SubCellCoord>,
        dir_x: f32,
        dir_y: f32,
        dest_screen_x: f32,
        dest_screen_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_anti_cross: bool,
        track_movement: bool,
    ) -> bool {
        let neighbors = current.get_neighbors();

        // Collect diagonal candidates sorted by alignment
        let mut diagonal_candidates: Vec<(SubCellCoord, f32)> = neighbors
            .iter()
            .filter(|n| Self::is_diagonal_move(current, n))
            .filter(|n| {
                // DESIGN DOC RULE (line 20): Filter candidates that would increase distance
                // "individual Manhattan-like distances of X and Y float coordinates must never increase"
                !current.violates_distance_rule(
                    n,
                    self.fpos_x,
                    self.fpos_y,
                    dest_screen_x,
                    dest_screen_y,
                    self.cell_width,
                    self.cell_height,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                    self.distance_tolerance_multiplier,
                )
            })
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
            // Anti-cross check for diagonal (optional - disabled by default to test if 3-cell reservation prevents crossing)
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

                    // Check if this is an optimal triangle (for logging/debugging)
                    let is_optimal = self.is_triangle_optimal(
                        current,
                        diagonal,
                        &anchor,
                        dir_x + self.fpos_x,  // Convert direction to destination position
                        dir_y + self.fpos_y,
                    );

                    if is_optimal {
                        println!("[RESERVE] Actor {} OPTIMAL TRIANGLE: reserved={:?} anchor={:?}",
                            self.id, diagonal, anchor);
                    } else {
                        println!("[RESERVE] Actor {} NON-OPTIMAL TRIANGLE: reserved={:?} anchor={:?}",
                            self.id, diagonal, anchor);
                    }
                    return true;
                }
            }
        }

        println!("[RESERVE] Actor {} DIAGONAL+ANCHOR: ALL BLOCKED (tried {} candidates)",
            self.id, diagonal_candidates.len());
        false
    }

    /// V2: Try to reserve diagonal using ray-rectangle intersection (actor_directing_v2.txt)
    fn try_reserve_diagonal_with_affinity(
        &mut self,
        current: &SubCellCoord,
        previous_current: Option<&SubCellCoord>,
        dest_screen_x: f32,
        dest_screen_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_anti_cross: bool,
        track_movement: bool,
    ) -> bool {
        // Log function entry for diagnostics
        let dx_to_dest = dest_screen_x - self.fpos_x;
        let dy_to_dest = dest_screen_y - self.fpos_y;
        self.diagnostic_messages.push(format!(
            "[DIAG RESERVE] Actor {} trying diagonal+affinity from ({},{},{},{}) toward dest dir=({:.1},{:.1})",
            self.id, current.cell_x, current.cell_y, current.sub_x, current.sub_y, dx_to_dest, dy_to_dest
        ));

        let neighbors = current.get_neighbors();

        // Collect diagonal candidates with distance rule filter
        let all_diagonals: Vec<SubCellCoord> = neighbors
            .iter()
            .filter(|n| Self::is_diagonal_move(current, n))
            .copied()
            .collect();

        // Calculate direction to destination for alignment scoring
        let dx_to_dest = dest_screen_x - self.fpos_x;
        let dy_to_dest = dest_screen_y - self.fpos_y;

        // Collect diagonal candidates with distance rule filter and alignment scores
        // WITH DETAILED LOGGING for debugging
        let mut diagonal_candidates: Vec<(SubCellCoord, f32)> = Vec::new();

        for n in &all_diagonals {
            let violates = current.violates_distance_rule(
                n,
                self.fpos_x,
                self.fpos_y,
                dest_screen_x,
                dest_screen_y,
                self.cell_width,
                self.cell_height,
                self.subcell_offset_x,
                self.subcell_offset_y,
                self.distance_tolerance_multiplier,
            );

            // Calculate distance details for logging
            let (curr_x, curr_y) = current.to_screen_center_with_offset(
                self.cell_width, self.cell_height, self.subcell_offset_x, self.subcell_offset_y
            );
            let (other_x, other_y) = n.to_screen_center_with_offset(
                self.cell_width, self.cell_height, self.subcell_offset_x, self.subcell_offset_y
            );
            let curr_dist_x = (dest_screen_x - curr_x).abs();
            let curr_dist_y = (dest_screen_y - curr_y).abs();
            let new_dist_x = (dest_screen_x - other_x).abs();
            let new_dist_y = (dest_screen_y - other_y).abs();
            let subcell_width = self.cell_width / 2.0;
            let tolerance = subcell_width * self.distance_tolerance_multiplier;
            let x_change = new_dist_x - curr_dist_x;
            let y_change = new_dist_y - curr_dist_y;

            // Log detailed distance check to diagnostic messages (written to action_log.db)
            self.diagnostic_messages.push(format!(
                "[DIST CHECK] Actor {} candidate ({},{},{},{}): curr_center=({:.1},{:.1}) other_center=({:.1},{:.1}) dest=({:.1},{:.1})",
                self.id, n.cell_x, n.cell_y, n.sub_x, n.sub_y, curr_x, curr_y, other_x, other_y, dest_screen_x, dest_screen_y
            ));
            self.diagnostic_messages.push(format!(
                "[DIST CHECK] Actor {} candidate ({},{},{},{}): curr_dist=({:.1},{:.1}) new_dist=({:.1},{:.1}) change=({:+.1},{:+.1}) tolerance={:.1} violates={}",
                self.id, n.cell_x, n.cell_y, n.sub_x, n.sub_y, curr_dist_x, curr_dist_y, new_dist_x, new_dist_y, x_change, y_change, tolerance, violates
            ));

            if !violates {
                let alignment = current.alignment_score(n, dx_to_dest, dy_to_dest, self.cell_width, self.cell_height);
                diagonal_candidates.push((*n, alignment));
                self.diagnostic_messages.push(format!(
                    "[DIST CHECK] Actor {} candidate ({},{},{},{}) PASSED - alignment={:.3}",
                    self.id, n.cell_x, n.cell_y, n.sub_x, n.sub_y, alignment
                ));
            } else {
                self.diagnostic_messages.push(format!(
                    "[DIST CHECK] Actor {} candidate ({},{},{},{}) FILTERED (violates distance rule)",
                    self.id, n.cell_x, n.cell_y, n.sub_x, n.sub_y
                ));
            }
        }

        // Sort by alignment score (descending - highest score first)
        diagonal_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Log distance rule filtering results
        self.diagnostic_messages.push(format!(
            "[DIAG RESERVE] Actor {} distance filter: {} diagonal neighbors, {} passed distance rule, {} filtered out",
            self.id, all_diagonals.len(), diagonal_candidates.len(), all_diagonals.len() - diagonal_candidates.len()
        ));

        if diagonal_candidates.is_empty() {
            self.diagnostic_messages.push(format!(
                "[DIAG RESERVE] Actor {} NO diagonal candidates passed distance rule - all would increase distance",
                self.id
            ));
            return false;
        }

        // Take ONLY the best diagonal (highest alignment score)
        // Per actor_directing_v2.txt Section C: try best diagonal with primary + opposite affinity,
        // then fall back to cardinals, then wait (don't try other diagonals)
        let (best_diagonal, best_score) = diagonal_candidates[0];

        if track_movement {
            println!("[RESERVE V2 DEBUG] Actor {} trying BEST diagonal only: ({},{},{},{}) with alignment score {:.3}",
                self.id, best_diagonal.cell_x, best_diagonal.cell_y, best_diagonal.sub_x, best_diagonal.sub_y, best_score);
        }

        self.diagnostic_messages.push(format!(
            "[DIAG RESERVE] Actor {} trying BEST diagonal: ({},{},{},{}) alignment={:.3}",
            self.id, best_diagonal.cell_x, best_diagonal.cell_y, best_diagonal.sub_x, best_diagonal.sub_y, best_score
        ));

        let diagonal = &best_diagonal;

        // Anti-cross check for diagonal (optional - disabled by default to test if 3-cell reservation prevents crossing)
        // If anti-cross blocks the best diagonal, skip directly to cardinal fallback
        let mut diagonal_blocked_by_anticross = false;
        if enable_anti_cross {
            if Self::check_anti_cross(current, diagonal, reservation_manager, self.id) {
                // Get counter-diagonal cells for logging
                let counter_diag = crate::subcell::get_counter_diagonal_subcells(current, diagonal);
                let owner1 = reservation_manager.get_owner(&counter_diag[0]);
                let owner2 = reservation_manager.get_owner(&counter_diag[1]);
                self.diagnostic_messages.push(format!(
                    "[DIAG RESERVE] Actor {} BEST diagonal BLOCKED by anti-cross: counter-diag cells ({},{},{},{}) owner={:?} and ({},{},{},{}) owner={:?}",
                    self.id,
                    counter_diag[0].cell_x, counter_diag[0].cell_y, counter_diag[0].sub_x, counter_diag[0].sub_y, owner1,
                    counter_diag[1].cell_x, counter_diag[1].cell_y, counter_diag[1].sub_x, counter_diag[1].sub_y, owner2
                ));
                if track_movement {
                    println!("[RESERVE V2 DEBUG]   Best diagonal blocked by anti-cross check");
                }
                diagonal_blocked_by_anticross = true;
            }
            if !diagonal_blocked_by_anticross {
                if let Some(prev) = previous_current {
                    if Self::check_anti_cross(prev, current, reservation_manager, self.id) {
                        let counter_diag = crate::subcell::get_counter_diagonal_subcells(prev, current);
                        let owner1 = reservation_manager.get_owner(&counter_diag[0]);
                        let owner2 = reservation_manager.get_owner(&counter_diag[1]);
                        self.diagnostic_messages.push(format!(
                            "[DIAG RESERVE] Actor {} BEST diagonal BLOCKED by anti-cross (prev check): counter-diag owner={:?} and {:?}",
                            self.id, owner1, owner2
                        ));
                        if track_movement {
                            println!("[RESERVE V2 DEBUG]   Best diagonal blocked by anti-cross check (prev)");
                        }
                        diagonal_blocked_by_anticross = true;
                    }
                }
            }
        }

        // If not blocked by anti-cross, try to reserve the best diagonal
        if !diagonal_blocked_by_anticross {

            // Calculate affinity and target using ray-rectangle intersection
            if self.id == 0 {
                self.diagnostic_messages.push(format!(
                    "[AFFINITY INPUT] actor_pos=({:.1},{:.1}) current=({},{},{},{}) diagonal=({},{},{},{}) dest_screen=({:.1},{:.1})",
                    self.fpos_x, self.fpos_y,
                    current.cell_x, current.cell_y, current.sub_x, current.sub_y,
                    diagonal.cell_x, diagonal.cell_y, diagonal.sub_x, diagonal.sub_y,
                    dest_screen_x, dest_screen_y
                ));
            }
            let affinity_result = self.calculate_simple_affinity_and_target(
                self.fpos_x,
                self.fpos_y,
                current,
                diagonal,
                dest_screen_x,
                dest_screen_y,
            );

            if track_movement {
                println!("[RESERVE V2 DEBUG]   Affinity={:?}, anchor=({},{},{},{})",
                    affinity_result.affinity,
                    affinity_result.anchor.cell_x, affinity_result.anchor.cell_y,
                    affinity_result.anchor.sub_x, affinity_result.anchor.sub_y);
            }

            // Check ownership before attempting reservation
            let diag_owner = reservation_manager.get_owner(diagonal);
            let anchor_owner = reservation_manager.get_owner(&affinity_result.anchor);
            self.diagnostic_messages.push(format!(
                "[DIAG RESERVE] Actor {} BEST diagonal: affinity={:?} anchor=({},{},{},{}) | Ownership: diagonal={:?} anchor={:?}",
                self.id, affinity_result.affinity,
                affinity_result.anchor.cell_x, affinity_result.anchor.cell_y,
                affinity_result.anchor.sub_x, affinity_result.anchor.sub_y,
                diag_owner, anchor_owner
            ));

            // Try to reserve diagonal + calculated anchor
            if track_movement {
                println!("[RESERVE V2 DEBUG]   Attempting to reserve diagonal + anchor...");
            }
            if reservation_manager.try_reserve_multiple(&[*diagonal, affinity_result.anchor], self.id) {
                self.reserved_subcell = Some(*diagonal);
                self.extra_reserved_subcells = vec![affinity_result.anchor];
                // LOCK target position and affinity (actor_directing_v2.txt)
                self.locked_target = Some((affinity_result.target_x, affinity_result.target_y));
                self.locked_affinity = Some(affinity_result.affinity);

                // Store directing info for logging
                self.last_directing_info = Some(DirectingInfo {
                    affinity: affinity_result.affinity,
                    target_x: affinity_result.target_x,
                    target_y: affinity_result.target_y,
                    reserved: *diagonal,
                    anchor: affinity_result.anchor,
                });

                if track_movement {
                    self.movement_track.push((self.fpos_x, self.fpos_y));
                }

                println!("[RESERVE V2] Actor {} PRIMARY affinity={:?} target=({:.2},{:.2}) reserved={:?} anchor={:?}",
                    self.id, affinity_result.affinity, affinity_result.target_x, affinity_result.target_y,
                    diagonal, affinity_result.anchor);
                self.diagnostic_messages.push(format!(
                    "[DIAG RESERVE] Actor {} PRIMARY SUCCESS: Reserved diagonal+anchor with affinity={:?}",
                    self.id, affinity_result.affinity
                ));
                return true;
            } else {
                if track_movement {
                    println!("[RESERVE V2 DEBUG]   Primary reservation FAILED");
                }
                let blocking_cell = if diag_owner.is_some() && diag_owner != Some(self.id) {
                    "diagonal"
                } else if anchor_owner.is_some() && anchor_owner != Some(self.id) {
                    "anchor"
                } else {
                    "unknown"
                };
                self.diagnostic_messages.push(format!(
                    "[DIAG RESERVE] Actor {} PRIMARY RESERVATION FAILED: {} blocked (diag_owner={:?}, anchor_owner={:?})",
                    self.id, blocking_cell, diag_owner, anchor_owner
                ));
            }

            // Try opposite affinity fallback (actor_directing_v2.txt Section C1)
            if track_movement {
                println!("[RESERVE V2 DEBUG]   Trying opposite affinity fallback...");
            }
            if let Some(opposite_anchor) = self.get_opposite_anchor(&affinity_result.affinity, current, diagonal, Some(&affinity_result.anchor)) {
                if track_movement {
                    println!("[RESERVE V2 DEBUG]   Opposite anchor=({},{},{},{})",
                        opposite_anchor.cell_x, opposite_anchor.cell_y,
                        opposite_anchor.sub_x, opposite_anchor.sub_y);
                }

                // Check ownership of opposite anchor
                let opp_anchor_owner = reservation_manager.get_owner(&opposite_anchor);
                self.diagnostic_messages.push(format!(
                    "[DIAG RESERVE] Actor {} trying OPPOSITE affinity: opposite_anchor=({},{},{},{}) owner={:?}",
                    self.id,
                    opposite_anchor.cell_x, opposite_anchor.cell_y,
                    opposite_anchor.sub_x, opposite_anchor.sub_y,
                    opp_anchor_owner
                ));

                if track_movement {
                    println!("[RESERVE V2 DEBUG]   Attempting to reserve diagonal + opposite anchor...");
                }
                if reservation_manager.try_reserve_multiple(&[*diagonal, opposite_anchor], self.id) {
                    self.reserved_subcell = Some(*diagonal);
                    self.extra_reserved_subcells = vec![opposite_anchor];

                    // Recalculate target for opposite affinity
                    // (Simplified: use diagonal's position as target for now)
                    let diag_screen = diagonal.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let flipped_affinity = self.flip_affinity(affinity_result.affinity);
                    self.locked_target = Some((diag_screen.0, diag_screen.1));
                    self.locked_affinity = Some(flipped_affinity);

                    // Store directing info for logging (opposite affinity case)
                    self.last_directing_info = Some(DirectingInfo {
                        affinity: flipped_affinity,
                        target_x: diag_screen.0,
                        target_y: diag_screen.1,
                        reserved: *diagonal,
                        anchor: opposite_anchor,
                    });

                    if track_movement {
                        self.movement_track.push((self.fpos_x, self.fpos_y));
                    }

                    println!("[RESERVE V2 OPPOSITE] Actor {} opposite affinity succeeded, reserved={:?} anchor={:?}",
                        self.id, diagonal, opposite_anchor);
                    self.diagnostic_messages.push(format!(
                        "[DIAG RESERVE] Actor {} OPPOSITE SUCCESS: Reserved with flipped affinity",
                        self.id
                    ));
                    return true;
                } else {
                    if track_movement {
                        println!("[RESERVE V2 DEBUG]   Opposite reservation also FAILED");
                    }
                    self.diagnostic_messages.push(format!(
                        "[DIAG RESERVE] Actor {} OPPOSITE FAILED: opposite anchor blocked by actor {:?}",
                        self.id, opp_anchor_owner
                    ));
                }
            } else {
                if track_movement {
                    println!("[RESERVE V2 DEBUG]   No opposite anchor available (affinity was BOTH)");
                }
                self.diagnostic_messages.push(format!(
                    "[DIAG RESERVE] Actor {} no opposite anchor (affinity was BOTH)",
                    self.id
                ));
            }
        } // End of !diagonal_blocked_by_anticross

        // Per actor_directing_v2.txt Section C2: Try cardinal fallback
        // This happens when:
        // 1. Anti-cross blocked the best diagonal, OR
        // 2. Primary affinity failed AND opposite affinity failed
        if track_movement {
            println!("[RESERVE V2 DEBUG] Best diagonal exhausted, trying cardinal fallback (Section C2)...");
        }
        self.diagnostic_messages.push(format!(
            "[DIAG RESERVE] Actor {} BEST DIAGONAL EXHAUSTED - trying cardinal fallback",
            self.id
        ));

        let cardinal_success = self.try_reserve_horizontal_vertical(
            current,
            dx_to_dest,
            dy_to_dest,
            dest_screen_x,
            dest_screen_y,
            reservation_manager,
            track_movement,
        );

        if cardinal_success {
            if track_movement {
                println!("[RESERVE V2 DEBUG] Cardinal fallback SUCCEEDED");
            }
            self.diagnostic_messages.push(format!(
                "[DIAG RESERVE] Actor {} CARDINAL FALLBACK SUCCESS",
                self.id
            ));
            return true;
        } else {
            if track_movement {
                println!("[RESERVE V2 DEBUG] Cardinal fallback FAILED - actor will wait");
            }
            self.diagnostic_messages.push(format!(
                "[DIAG RESERVE] Actor {} CARDINAL FALLBACK FAILED - all directions blocked, actor will wait",
                self.id
            ));
            println!("[RESERVE V2] Actor {} ALL DIRECTIONS BLOCKED (best diagonal + cardinals) - WAITING",
                self.id);
            return false;
        }
    }

    /// Get opposite anchor for fallback (Section C1)
    fn get_opposite_anchor(
        &self,
        original_affinity: &Affinity,
        psc: &SubCellCoord,
        diagonal: &SubCellCoord,
        tried_anchor: Option<&SubCellCoord>,
    ) -> Option<SubCellCoord> {
        match original_affinity {
            Affinity::Horizontal => {
                // Was H, try V anchor
                Some(Self::get_vertical_anchor(psc, diagonal))
            }
            Affinity::Vertical => {
                // Was V, try H anchor
                Some(Self::get_horizontal_anchor(psc, diagonal))
            }
            Affinity::Both => {
                // For BOTH affinity, try the anchor we didn't try yet
                // Compare tried_anchor with H and V anchors to determine which to try
                if let Some(tried) = tried_anchor {
                    let h_anchor = Self::get_horizontal_anchor(psc, diagonal);
                    let v_anchor = Self::get_vertical_anchor(psc, diagonal);

                    // If tried anchor was H, return V; if tried was V, return H
                    if tried.cell_x == h_anchor.cell_x && tried.cell_y == h_anchor.cell_y &&
                       tried.sub_x == h_anchor.sub_x && tried.sub_y == h_anchor.sub_y {
                        Some(v_anchor)  // Tried H, return V
                    } else {
                        Some(h_anchor)  // Tried V (or unknown), return H
                    }
                } else {
                    // No tried anchor info, default to H anchor
                    Some(Self::get_horizontal_anchor(psc, diagonal))
                }
            }
        }
    }

    /// Flip affinity for opposite anchor (debug/logging)
    fn flip_affinity(&self, affinity: Affinity) -> Affinity {
        match affinity {
            Affinity::Horizontal => Affinity::Vertical,
            Affinity::Vertical => Affinity::Horizontal,
            Affinity::Both => Affinity::Both,
        }
    }

    /// Test if a ray intersects a triangle using barycentric coordinates
    ///
    /// Returns true if the ray from origin in direction (dir_x, dir_y) intersects
    /// the triangle formed by the three vertices (in screen coordinates)
    fn ray_intersects_triangle(
        origin_x: f32,
        origin_y: f32,
        dir_x: f32,
        dir_y: f32,
        v0_x: f32,
        v0_y: f32,
        v1_x: f32,
        v1_y: f32,
        v2_x: f32,
        v2_y: f32,
    ) -> bool {
        // 2D ray-triangle intersection using barycentric coordinates
        // The ray is: P(t) = origin + t * dir (where t >= 0)
        // The triangle is: P(u, v) = v0 + u*(v1-v0) + v*(v2-v0) (where u,v >= 0 and u+v <= 1)

        // Edge vectors
        let edge1_x = v1_x - v0_x;
        let edge1_y = v1_y - v0_y;
        let edge2_x = v2_x - v0_x;
        let edge2_y = v2_y - v0_y;

        // Vector from v0 to ray origin
        let tvec_x = origin_x - v0_x;
        let tvec_y = origin_y - v0_y;

        // Calculate determinant (cross product in 2D)
        let det = edge1_x * edge2_y - edge1_y * edge2_x;

        // If determinant is near zero, ray and triangle are parallel
        if det.abs() < 1e-6 {
            return false;
        }

        let inv_det = 1.0 / det;

        // Calculate u parameter (barycentric coordinate)
        let u = (tvec_x * edge2_y - tvec_y * edge2_x) * inv_det;
        if u < 0.0 || u > 1.0 {
            return false;
        }

        // Calculate v parameter (barycentric coordinate)
        let v = (dir_x * tvec_y - dir_y * tvec_x) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return false;
        }

        // Calculate t parameter (distance along ray)
        let t = (edge2_x * tvec_y - edge2_y * tvec_x) * inv_det;

        // Ray intersects triangle if t >= 0 (forward direction)
        t >= 0.0
    }

    /// Check if a triangle is optimal (contains the destination vector from current position)
    ///
    /// Per spec Q2.2: A triangle is optimal if the ray from the actor toward the destination
    /// intersects the triangle formed by (current, diagonal, anchor)
    fn is_triangle_optimal(
        &self,
        current: &SubCellCoord,
        diagonal: &SubCellCoord,
        anchor: &SubCellCoord,
        dest_x: f32,
        dest_y: f32,
    ) -> bool {
        // Get triangle vertices in screen coordinates
        let (v0_x, v0_y) = current.to_screen_center_with_offset(
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );
        let (v1_x, v1_y) = diagonal.to_screen_center_with_offset(
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );
        let (v2_x, v2_y) = anchor.to_screen_center_with_offset(
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );

        // Direction from current position toward destination
        let dir_x = dest_x - self.fpos_x;
        let dir_y = dest_y - self.fpos_y;

        // Test if ray intersects triangle
        Self::ray_intersects_triangle(
            self.fpos_x, self.fpos_y,
            dir_x, dir_y,
            v0_x, v0_y,
            v1_x, v1_y,
            v2_x, v2_y,
        )
    }

    /// Try to reserve ANY available subcell in the current supercell (cell)
    /// Returns true if any reservation succeeded
    ///
    /// This is used when preferred directions are blocked - actor tries to reserve
    /// any available subcell in their current cell, sorted by distance to actor's position
    fn try_reserve_any_available_in_current_cell(
        &mut self,
        current: &SubCellCoord,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        track_movement: bool,
    ) -> bool {
        // Get all subcells in the current supercell (cell)
        let mut subcells_in_cell: Vec<SubCellCoord> = Vec::new();

        for sub_y in 0..self.subcell_grid_size {
            for sub_x in 0..self.subcell_grid_size {
                let sc = SubCellCoord::new(
                    current.cell_x,
                    current.cell_y,
                    sub_x,
                    sub_y,
                    self.subcell_grid_size,
                );
                subcells_in_cell.push(sc);
            }
        }

        // Sort by distance to actor's current float position
        subcells_in_cell.sort_by(|a, b| {
            let (a_x, a_y) = a.to_screen_center_with_offset(
                self.cell_width,
                self.cell_height,
                self.subcell_offset_x,
                self.subcell_offset_y,
            );
            let (b_x, b_y) = b.to_screen_center_with_offset(
                self.cell_width,
                self.cell_height,
                self.subcell_offset_x,
                self.subcell_offset_y,
            );

            let dist_a = ((a_x - self.fpos_x).powi(2) + (a_y - self.fpos_y).powi(2)).sqrt();
            let dist_b = ((b_x - self.fpos_x).powi(2) + (b_y - self.fpos_y).powi(2)).sqrt();

            dist_a.partial_cmp(&dist_b).unwrap()
        });

        // Try to reserve each subcell in order of distance
        for sc in &subcells_in_cell {
            // Skip if it's already our current subcell
            if sc == current {
                continue;
            }

            if reservation_manager.try_reserve(*sc, self.id) {
                self.reserved_subcell = Some(*sc);
                self.extra_reserved_subcells.clear();
                // Clear locked values when changing reservation
                self.locked_target = None;
                self.locked_affinity = None;
                if track_movement {
                    self.movement_track.push((self.fpos_x, self.fpos_y));
                }
                println!("[RESERVE] Actor {} ANY-AVAILABLE: reserved={:?} (all preferred blocked)",
                    self.id, sc);
                return true;
            }
        }

        println!("[RESERVE] Actor {} ANY-AVAILABLE: ALL BLOCKED (cell too crowded)",
            self.id);
        false
    }

    /// Check if we should attempt reservation based on eagerness mode
    ///
    /// CENTER mode: Reserve when destination subcell center is within threshold distance
    /// ROUND mode: Reserve when destination subcell center is closer than all currently reserved subcells
    fn should_attempt_reservation(
        &self,
        reservation_eagerness: crate::config::ReservationEagerness,
        reservation_threshold_distance: f32,
        dest_screen_x: f32,
        dest_screen_y: f32,
    ) -> bool {
        use crate::config::ReservationEagerness;

        // Calculate destination subcell
        let dest_subcell = SubCellCoord::from_screen_pos_with_offset(
            dest_screen_x,
            dest_screen_y,
            self.cell_width,
            self.cell_height,
            self.subcell_grid_size,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );

        let (dest_sc_x, dest_sc_y) = dest_subcell.to_screen_center_with_offset(
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );

        let dx = dest_sc_x - self.fpos_x;
        let dy = dest_sc_y - self.fpos_y;
        let dist_to_dest_sc = (dx * dx + dy * dy).sqrt();

        match reservation_eagerness {
            ReservationEagerness::Center => {
                // Reserve when within threshold distance of destination subcell center
                dist_to_dest_sc <= reservation_threshold_distance
            },
            ReservationEagerness::Round => {
                // Reserve when destination subcell is closer than all currently reserved subcells
                // Check current subcell
                if let Some(current_sc) = self.current_subcell {
                    let (curr_x, curr_y) = current_sc.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_grid_size,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let dx_curr = curr_x - self.fpos_x;
                    let dy_curr = curr_y - self.fpos_y;
                    let dist_to_curr = (dx_curr * dx_curr + dy_curr * dy_curr).sqrt();

                    if dist_to_dest_sc >= dist_to_curr {
                        return false; // Current is closer or equal, don't reserve yet
                    }
                }

                // Check reserved subcell
                if let Some(reserved_sc) = self.reserved_subcell {
                    let (res_x, res_y) = reserved_sc.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let dx_res = res_x - self.fpos_x;
                    let dy_res = res_y - self.fpos_y;
                    let dist_to_res = (dx_res * dx_res + dy_res * dy_res).sqrt();

                    if dist_to_dest_sc >= dist_to_res {
                        return false; // Reserved is closer or equal, don't reserve yet
                    }
                }

                // Check extra reserved subcells
                for extra_sc in &self.extra_reserved_subcells {
                    let (extra_x, extra_y) = extra_sc.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let dx_extra = extra_x - self.fpos_x;
                    let dy_extra = extra_y - self.fpos_y;
                    let dist_to_extra = (dx_extra * dx_extra + dy_extra * dy_extra).sqrt();

                    if dist_to_dest_sc >= dist_to_extra {
                        return false; // Extra is closer or equal, don't reserve yet
                    }
                }

                // Destination subcell is closer than all currently reserved subcells
                true
            }
        }
    }


    /// Try to reserve H/V sub-cell (fallback when diagonal blocked)
    /// Returns true if reservation succeeded
    pub(crate) fn try_reserve_horizontal_vertical(
        &mut self,
        current: &SubCellCoord,
        dir_x: f32,
        dir_y: f32,
        dest_screen_x: f32,
        dest_screen_y: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        track_movement: bool,
    ) -> bool {
        let neighbors = current.get_neighbors();

        // Priority list per spec Q2.3:
        // 1. Dominant cardinal direction (H or V based on which component is larger)
        // 2. Perpendicular cardinal direction

        // Determine dominant direction
        let abs_dx = dir_x.abs();
        let abs_dy = dir_y.abs();
        let horizontal_dominant = abs_dx > abs_dy;

        // Separate H and V candidates
        let mut horizontal_candidates: Vec<SubCellCoord> = Vec::new();
        let mut vertical_candidates: Vec<SubCellCoord> = Vec::new();

        for n in neighbors.iter() {
            if Self::is_diagonal_move(current, n) {
                continue; // Skip diagonals
            }

            // DESIGN DOC RULE (line 20): Filter candidates that would increase distance
            // "individual Manhattan-like distances of X and Y float coordinates must never increase"
            if current.violates_distance_rule(
                n,
                self.fpos_x,
                self.fpos_y,
                dest_screen_x,
                dest_screen_y,
                self.cell_width,
                self.cell_height,
                self.subcell_offset_x,
                self.subcell_offset_y,
                self.distance_tolerance_multiplier,
            ) {
                continue; // Skip - would increase distanceX or distanceY
            }

            // Check if it's horizontal or vertical
            let dx_cells = (n.cell_x - current.cell_x).abs();
            let dy_cells = (n.cell_y - current.cell_y).abs();
            let dx_subs = (n.sub_x - current.sub_x).abs();
            let dy_subs = (n.sub_y - current.sub_y).abs();

            if (dx_cells > 0 || dx_subs > 0) && dy_cells == 0 && dy_subs == 0 {
                horizontal_candidates.push(*n);
            } else if (dy_cells > 0 || dy_subs > 0) && dx_cells == 0 && dx_subs == 0 {
                vertical_candidates.push(*n);
            }
        }

        // Sort each group by alignment score
        horizontal_candidates.sort_by(|a, b| {
            let score_a = current.alignment_score(a, dir_x, dir_y, self.cell_width, self.cell_height);
            let score_b = current.alignment_score(b, dir_x, dir_y, self.cell_width, self.cell_height);
            score_b.partial_cmp(&score_a).unwrap()
        });

        vertical_candidates.sort_by(|a, b| {
            let score_a = current.alignment_score(a, dir_x, dir_y, self.cell_width, self.cell_height);
            let score_b = current.alignment_score(b, dir_x, dir_y, self.cell_width, self.cell_height);
            score_b.partial_cmp(&score_a).unwrap()
        });

        // Try in priority order: dominant direction first, then perpendicular
        let priority_order: Vec<&Vec<SubCellCoord>> = if horizontal_dominant {
            vec![&horizontal_candidates, &vertical_candidates]
        } else {
            vec![&vertical_candidates, &horizontal_candidates]
        };

        for candidate_group in priority_order {
            for candidate in candidate_group.iter() {
                if reservation_manager.try_reserve(*candidate, self.id) {
                    self.reserved_subcell = Some(*candidate);
                    self.extra_reserved_subcells.clear();

                    // Calculate locked target for H/V movement using simple distance-based affinity
                    // Affinity based on |dx| vs |dy|, target via straight-line intersection
                    let affinity_result = self.calculate_simple_affinity_and_target(
                        self.fpos_x,
                        self.fpos_y,
                        current,
                        candidate,
                        dest_screen_x,
                        dest_screen_y,
                    );
                    self.locked_target = Some((affinity_result.target_x, affinity_result.target_y));
                    self.locked_affinity = Some(affinity_result.affinity);

                    if track_movement {
                        self.movement_track.push((self.fpos_x, self.fpos_y));
                    }
                    let direction = if horizontal_candidates.contains(candidate) { "H" } else { "V" };
                    println!("[RESERVE] Actor {} {}-FALLBACK: reserved={:?} target=({:.1},{:.1}) (diagonal blocked)",
                        self.id, direction, candidate, affinity_result.target_x, affinity_result.target_y);
                    return true;
                }
            }
        }

        println!("[RESERVE] Actor {} H/V FALLBACK: ALL BLOCKED (H={}, V={})",
            self.id, horizontal_candidates.len(), vertical_candidates.len());
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
                    // Clear locked values when changing reservation
                    self.locked_target = None;
                    self.locked_affinity = None;
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

    // ========================================================================
    // LOGGING HELPERS
    // ========================================================================

    /// Log actor's alignment to subcell center
    fn log_subcell_alignment(&self, subcell: &SubCellCoord, center_x: f32, center_y: f32, always_trace: bool) {
        if always_trace {
            println!("[ALIGN] Actor {} reserved subcell ({},{},{},{}) at distance {:.1}px from ({:.1},{:.1})",
                self.id, subcell.cell_x, subcell.cell_y, subcell.sub_x, subcell.sub_y,
                ((self.fpos_x - center_x).powi(2) + (self.fpos_y - center_y).powi(2)).sqrt(),
                self.fpos_x, self.fpos_y);
        }
    }

    /// Log failure to reserve any subcell
    fn log_no_subcell_available(&self, radius: i32, always_trace: bool) {
        if always_trace {
            println!("[ALIGN] Actor {} could NOT reserve ANY subcell within radius {} - all occupied, will retry",
                self.id, radius);
        }
    }

    /// Log entering PscAlignment state
    fn log_entering_psc_alignment(&self, center_x: f32, center_y: f32, always_trace: bool) {
        if always_trace {
            println!("[ALIGN] Actor {} entering PscAlignment to subcell center ({:.1},{:.1})",
                self.id, center_x, center_y);
        }
    }

    /// Log alignment target being set
    fn log_alignment_target_set(&self, target_x: f32, target_y: f32, always_trace: bool) {
        if always_trace {
            println!("[ALIGN] Actor {} set initial alignment_target to ({:.1},{:.1})",
                self.id, target_x, target_y);
        }
    }

    /// Log PscAlignment movement progress
    fn log_psc_alignment_movement(&self, old_x: f32, old_y: f32, dist_to_center: f32, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[ALIGN] Actor {} moving to PSC center: ({:.1},{:.1}) → ({:.1},{:.1}), dist={:.2}px",
                self.id, old_x, old_y, self.fpos_x, self.fpos_y, dist_to_center);
        }
    }

    /// Log transition to Idle state
    fn log_transition_to_idle(&self, subcell: &SubCellCoord, dist: f32, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[ALIGN] Actor {} reached PSC center (dist={:.2}px < threshold=2.00px), entering Idle state",
                self.id, dist);
        }
    }

    /// Log no destination available
    fn log_no_destination(&self, always_trace: bool) {
        if always_trace {
            println!("[DestDirect] Actor {} has NO DESTINATION (Idle at subcell center)", self.id);
        }
    }

    /// Log destination information
    fn log_destination(&self, dest_x: f32, dest_y: f32, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[DestDirect] Destination: ({:.0},{:.0})", dest_x, dest_y);
        }
    }

    /// Log destination reached
    fn log_destination_reached(&self, dist: f32, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[DEST REACHED] Actor {} within 2.0px (dist={:.2}px)", self.id, dist);
        }
    }

    /// Log movement execution
    fn log_movement(&self, move_dist: f32, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[MOVE] Actor {} moved {:.2}px to ({:.1},{:.1})",
                self.id, move_dist, self.fpos_x, self.fpos_y);
        }
    }

    /// Log PSC switching decision
    fn log_psc_switch(&self, from: &SubCellCoord, to: &SubCellCoord, reason: &str, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[PSC SWITCH] Actor {} switching from ({},{},{},{}) to ({},{},{},{}) - {}",
                self.id,
                from.cell_x, from.cell_y, from.sub_x, from.sub_y,
                to.cell_x, to.cell_y, to.sub_x, to.sub_y,
                reason);
        }
    }

    /// Log staying at current PSC
    fn log_staying_at_psc(&self, psc: &SubCellCoord, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[PSC] Actor {} staying at ({},{},{},{})",
                self.id, psc.cell_x, psc.cell_y, psc.sub_x, psc.sub_y);
        }
    }

    /// Log reservation attempt
    fn log_reservation_attempt(&self, reservation_type: &str, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[RESERVE] Actor {} attempting {} reservation", self.id, reservation_type);
        }
    }

    /// Log reservation success
    fn log_reservation_success(&self, reserved: &SubCellCoord, reservation_type: &str, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[RESERVE] Actor {} {} SUCCESS: reserved ({},{},{},{})",
                self.id, reservation_type,
                reserved.cell_x, reserved.cell_y, reserved.sub_x, reserved.sub_y);
        }
    }

    /// Log reservation failure
    fn log_reservation_failure(&self, reservation_type: &str, reason: &str, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            println!("[RESERVE] Actor {} {} FAILED: {}", self.id, reservation_type, reason);
        }
    }

    /// Log function return
    fn log_return(&self, reached: bool, always_trace: bool, track_movement: bool) {
        if always_trace || (self.id == 0 && track_movement) {
            if reached {
                println!("  [RETURN] Returning true (reached destination)");
            } else {
                println!("  [RETURN] Returning false (not reached destination)");
            }
        }
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
    /// Internal implementation of update_subcell_destination_direct
    /// Called by actor_directives::update_actor() (the decision root)
    pub(crate) fn update_subcell_destination_direct_impl(
        &mut self,
        delta_time: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_early_reservation: bool,
        filter_backward: bool,
        enable_anti_cross: bool,
        track_movement: bool
    ) -> bool {
        // ALWAYS log first 100 frames for actor 0 to debug GUI freeze
        // ALSO log when actor is at critical moments (target reached, switching, etc.)
        static mut FRAME_COUNT: u32 = 0;
        static mut LOGGED_NO_DEST: bool = false;
        static mut LAST_POS_X: f32 = 0.0;
        static mut LAST_POS_Y: f32 = 0.0;
        static mut STUCK_FRAMES: u32 = 0;

        let always_trace = unsafe {
            if self.id == 0 {
                FRAME_COUNT += 1;

                // Check if actor is stuck (not moving)
                let moved = (self.fpos_x - LAST_POS_X).abs() > 0.001 || (self.fpos_y - LAST_POS_Y).abs() > 0.001;
                LAST_POS_X = self.fpos_x;
                LAST_POS_Y = self.fpos_y;

                if !moved {
                    STUCK_FRAMES += 1;
                } else {
                    STUCK_FRAMES = 0;
                }

                // Log if: first 100 frames, or stuck for 3+ consecutive frames
                FRAME_COUNT <= 100 || STUCK_FRAMES >= 3
            } else {
                false
            }
        };

        // Log stuck status if tracing
        // if always_trace {
        //     unsafe {
        //         if STUCK_FRAMES > 0 {
        //             println!("[DestDirect ENTRY] Actor {} frame {} STUCK for {} frames", self.id, FRAME_COUNT, STUCK_FRAMES);
        //         } else {
        //             println!("[DestDirect ENTRY] Actor {} frame {} moving", self.id, FRAME_COUNT);
        //         }
        //     }
        // }

        // Ensure we have current sub-cell FIRST (before checking destination)
        // This allows PscAlignment to proceed even without a destination
        let current = match self.current_subcell {
            Some(c) => c,
            None => {
                // Actor is in NoSubcell state - try to reserve a nearby subcell
                // Get the cell actor is in based on position
                let cell_coord = SubCellCoord::from_screen_pos_with_offset(
                    self.fpos_x,
                    self.fpos_y,
                    self.cell_width,
                    self.cell_height,
                    self.subcell_grid_size,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                );

                // Try to reserve subcells in nearby cells using spiral search pattern
                // Spiral: (0,0) → (1,0) → (1,1) → (0,1) → (-1,1) → (-1,0) → (-1,-1) → (0,-1) → (1,-1) → ...
                let search_radius = 5; // Search up to 5 cells away
                let spiral_offsets = Self::generate_spiral_offsets(search_radius);

                let mut all_candidate_subcells: Vec<SubCellCoord> = Vec::new();

                // For each cell in spiral pattern
                for (cell_dx, cell_dy) in spiral_offsets {
                    let target_cell_x = cell_coord.cell_x + cell_dx;
                    let target_cell_y = cell_coord.cell_y + cell_dy;

                    // Generate all 4 subcells in this cell
                    for sub_y in 0..self.subcell_grid_size {
                        for sub_x in 0..self.subcell_grid_size {
                            let sc = SubCellCoord::new(
                                target_cell_x,
                                target_cell_y,
                                sub_x,
                                sub_y,
                                self.subcell_grid_size,
                            );
                            all_candidate_subcells.push(sc);
                        }
                    }
                }

                // Sort ALL candidate subcells by:
                // 1. PRIORITY: Prefer subcells in same cell (cell_dx=0, cell_dy=0)
                // 2. SECONDARY: Sort by distance to actor's position
                // This prevents actors from aligning to adjacent cells when current cell has space
                all_candidate_subcells.sort_by(|a, b| {
                    let a_in_current_cell = (a.cell_x == cell_coord.cell_x) && (a.cell_y == cell_coord.cell_y);
                    let b_in_current_cell = (b.cell_x == cell_coord.cell_x) && (b.cell_y == cell_coord.cell_y);

                    // Primary sort: same cell first
                    match (a_in_current_cell, b_in_current_cell) {
                        (true, false) => return std::cmp::Ordering::Less,    // a in current cell, prefer a
                        (false, true) => return std::cmp::Ordering::Greater, // b in current cell, prefer b
                        _ => {} // Both in same category, continue to distance sort
                    }

                    // Secondary sort: by distance
                    let (a_x, a_y) = a.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let (b_x, b_y) = b.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let dist_a = ((a_x - self.fpos_x).powi(2) + (a_y - self.fpos_y).powi(2)).sqrt();
                    let dist_b = ((b_x - self.fpos_x).powi(2) + (b_y - self.fpos_y).powi(2)).sqrt();
                    dist_a.partial_cmp(&dist_b).unwrap()
                });

                // Try to reserve the nearest available subcell (across all cells in spiral)
                let mut reserved_subcell = None;
                for sc in &all_candidate_subcells {
                    if reservation_manager.try_reserve(*sc, self.id) {
                        reserved_subcell = Some(*sc);
                        if always_trace {
                            println!("[ALIGN] Actor {} reserved subcell ({},{},{},{}) at distance {:.1}px from ({:.1},{:.1})",
                                self.id, sc.cell_x, sc.cell_y, sc.sub_x, sc.sub_y,
                                {
                                    let (cx, cy) = sc.to_screen_center_with_offset(
                                        self.cell_width, self.cell_height,
                                        self.subcell_offset_x, self.subcell_offset_y
                                    );
                                    ((cx - self.fpos_x).powi(2) + (cy - self.fpos_y).powi(2)).sqrt()
                                },
                                self.fpos_x, self.fpos_y);
                        }
                        break;
                    }
                }

                // If no subcell could be reserved even with spiral search, stay in NoSubcell state
                if reserved_subcell.is_none() {
                    if always_trace {
                        println!("[ALIGN] Actor {} could NOT reserve ANY subcell within radius {} - all occupied, will retry",
                            self.id, search_radius);
                    }
                    return false; // Stay in NoSubcell state, retry next frame
                }

                let c = reserved_subcell.unwrap();
                self.current_subcell = Some(c.to_subpoint());
                reservation_manager.set_current(c, self.id);

                // Enter PscAlignment state (NoSubcell → PscAlignment transition)
                self.alignment_state = AlignmentState::PscAlignment;
                let (center_x, center_y) = c.to_screen_center_with_offset(
                    self.cell_width,
                    self.cell_height,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                );
                self.alignment_target = Some((center_x, center_y));
                if always_trace {
                    println!("[ALIGN] Actor {} entering PscAlignment to subcell center ({:.1},{:.1})",
                        self.id, center_x, center_y);
                }

                c.to_subpoint()
            }
        };

        // Initialize alignment_target if in PscAlignment state but target not set
        // (Happens on first update after Actor::new())
        if self.alignment_state == AlignmentState::PscAlignment && self.alignment_target.is_none() {
            let (center_x, center_y) = current.to_screen_center_with_offset(
                self.cell_width,
                self.cell_height,
                self.subcell_grid_size,
                self.subcell_offset_x,
                self.subcell_offset_y,
            );
            self.alignment_target = Some((center_x, center_y));
            if always_trace {
                println!("[ALIGN] Actor {} set initial alignment_target to ({:.1},{:.1})",
                    self.id, center_x, center_y);
            }
        }

        // Handle PscAlignment state - actor moves to PSC center before navigation
        if self.alignment_state == AlignmentState::PscAlignment {
            if let Some((target_x, target_y)) = self.alignment_target {
                let dx = target_x - self.fpos_x;
                let dy = target_y - self.fpos_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < self.alignment_threshold {
                    // Reached center - transition to Idle
                    self.alignment_state = AlignmentState::Idle;
                    self.alignment_target = None;
                    if always_trace || (self.id == 0 && track_movement) {
                        println!("[ALIGN] Actor {} reached PSC center (dist={:.2}px < threshold={:.2}px), entering Idle state",
                            self.id, dist, self.alignment_threshold);
                    }
                    // Return early - wait one frame at center before attempting reservation
                    // This ensures actors are actually "at rest" at subcell center
                    return false;
                } else {
                    // Move toward center (using same logic as moving toward destination)
                    let movement = self.speed * delta_time;
                    let move_dist = movement.min(dist);
                    let dir_x = dx / dist;
                    let dir_y = dy / dist;
                    self.fpos_x += dir_x * move_dist;
                    self.fpos_y += dir_y * move_dist;

                    if always_trace || (self.id == 0 && track_movement) {
                        println!("[ALIGN] Actor {} moving to PSC center: ({:.1},{:.1}) → ({:.1},{:.1}), dist={:.2}px",
                            self.id, self.fpos_x - dir_x * move_dist, self.fpos_y - dir_y * move_dist,
                            self.fpos_x, self.fpos_y, dist);
                    }
                    return false; // Still aligning, not at destination
                }
            }
        }

        // After PscAlignment, check if we have a destination for navigation
        // If no destination, actor stays in Idle state at subcell center
        let dest = match self.subcell_destination {
            Some(d) => {
                if always_trace {
                    println!("[DestDirect] Destination: ({},{})", d.x, d.y);
                }
                unsafe { LOGGED_NO_DEST = false; } // Reset flag when destination is set
                d
            },
            None => {
                // No destination - actor stays at current position (Idle state)
                if always_trace && unsafe { !LOGGED_NO_DEST } {
                    println!("[DestDirect] Actor {} has NO DESTINATION (Idle at subcell center)", self.id);
                    unsafe { LOGGED_NO_DEST = true; }
                }
                return true; // No destination, stay at current position
            }
        };

        // Get destination screen position and quantize to subcell grid point
        // Convert cell coordinates to screen coordinates (use cell CENTER, not corner)
        let dest_screen_x_raw = dest.x as f32 * self.cell_width + self.cell_width / 2.0;
        let dest_screen_y_raw = dest.y as f32 * self.cell_height + self.cell_height / 2.0;

        // Quantize destination to nearest subcell grid point
        // This ensures destinations are properly aligned with the subcell pathfinding grid
        let dest_subcell_quantized = SubCellCoord::from_screen_pos_with_offset(
            dest_screen_x_raw,
            dest_screen_y_raw,
            self.cell_width,
            self.cell_height,
            self.subcell_grid_size,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );
        let (dest_screen_x, dest_screen_y) = dest_subcell_quantized.to_screen_center_with_offset(
            self.cell_width,
            self.cell_height,
            self.subcell_offset_x,
            self.subcell_offset_y,
        );

        if always_trace || (self.id == 0 && track_movement) {
            if self.id == 0 {
                let dx = dest_screen_x - self.fpos_x;
                let dy = dest_screen_y - self.fpos_y;
                self.diagnostic_messages.push(format!(
                    "[DEST CALC] dest_cell=({},{}) dest_screen=({:.1},{:.1}) actor_pos=({:.1},{:.1}) expected_dir: dx={:.1} dy={:.1} ({} and {})",
                    dest.x, dest.y, dest_screen_x, dest_screen_y, self.fpos_x, self.fpos_y,
                    dx, dy,
                    if dx > 0.0 { "RIGHT" } else { "LEFT" },
                    if dy > 0.0 { "DOWN" } else { "UP" }
                ));
            }
        }

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

            println!("[REACHED] Actor {} reached destination at ({:.1},{:.1}) subcell=({},{},{},{})",
                self.id, self.fpos_x, self.fpos_y,
                dest_subcell.cell_x, dest_subcell.cell_y, dest_subcell.sub_x, dest_subcell.sub_y);

            // Release all reservations except the destination sub-cell
            let current_coord = SubCellCoord::from_subpoint(&current, self.subcell_grid_size);
            if current_coord != dest_subcell {
                reservation_manager.release(current_coord, self.id);
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
            self.current_subcell = Some(dest_subcell.to_subpoint());
            self.subcell_destination = None;
            self.reserved_subcell = None;
            // Clear locked values from affinity calculation
            self.locked_target = None;
            self.locked_affinity = None;
            return true;
        }

        // Get anchor sub-cell (first extra reserved cell, if any)
        let anchor_subcell = self.extra_reserved_subcells.first();

        // Calculate target position: use locked target from affinity calc if available
        let (target_x, target_y) = if let Some((locked_x, locked_y)) = self.locked_target {
            // Use locked target from actor_directing_v2 ray-rectangle intersection
            (locked_x, locked_y)
        } else {
            // Fallback: calculate optimal boundary (legacy or non-diagonal movement)
            let current_coord = SubCellCoord::from_subpoint(&current, self.subcell_grid_size);
            crate::subcell::calculate_optimal_boundary(
                &current_coord,
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
            )
        };

        // DIRECTION CHANGE DETECTION
        // Track when actors change direction mid-journey (indicates indirect pathfinding)
        let new_target = (target_x, target_y);
        let new_target_type = if self.locked_target.is_some() {
            TargetType::Direct
        } else {
            TargetType::SubcellPoint
        };

        // Check for direction change using the 3-rule logic
        if let Some(old_tgt) = self.old_target {
            let direction_changed = if new_target_type == TargetType::Direct &&
                                       self.target_type == TargetType::Direct {
                // Rule 1: Both targets are Direct → no direction change
                false
            } else if let Some(prev_old_tgt) = self.previous_old_target {
                // Rule 2: Check if same cardinal direction
                let old_direction = get_cardinal_direction(prev_old_tgt.0, prev_old_tgt.1, old_tgt.0, old_tgt.1);
                let new_direction = get_cardinal_direction(old_tgt.0, old_tgt.1, new_target.0, new_target.1);
                old_direction != new_direction
            } else {
                // First frame with history, can't check yet
                false
            };

            if direction_changed {
                self.diagnostic_messages.push(format!(
                    "[DIRECTION CHANGE] Actor {} changed direction! old_target=({:.1},{:.1}) new_target=({:.1},{:.1}) old_type={:?} new_type={:?}",
                    self.id, old_tgt.0, old_tgt.1, new_target.0, new_target.1, self.target_type, new_target_type
                ));
            }
        }

        // Update history
        self.previous_old_target = self.old_target;
        self.old_target = Some(new_target);
        self.target_type = new_target_type;

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

        // Recalculate distance to target AFTER movement for consistent switching check
        // (Previously used pre-movement distance, causing overshoot bugs)
        let dist_to_target_after = {
            let dx = target_x - self.fpos_x;
            let dy = target_y - self.fpos_y;
            (dx * dx + dy * dy).sqrt()
        };

        // Check if we should switch from reserved to current (triangle-based switching)
        if let Some(reserved) = self.reserved_subcell {
            if always_trace || (self.id == 0 && track_movement) {
                println!("  [STATE] Has reservation: {:?}", reserved);
            }
            // Triangle-based movement: switch based on boundary proximity using threshold
            let should_switch = if enable_early_reservation {
                // Early mode: Use threshold-based switching
                // Calculate total distance from current center to target boundary
                let (current_x, current_y) = current.to_screen_center_with_offset(
                    self.cell_width,
                    self.cell_height,
                    self.subcell_grid_size,
                    self.subcell_offset_x,
                    self.subcell_offset_y,
                );
                let dx_curr_to_target = target_x - current_x;
                let dy_curr_to_target = target_y - current_y;
                let total_distance = (dx_curr_to_target * dx_curr_to_target + dy_curr_to_target * dy_curr_to_target).sqrt();

                // Calculate remaining distance ratio (0.0 = at target, 1.0 = at current center)
                let remaining_ratio = if total_distance > 0.001 {
                    dist_to_target_after / total_distance
                } else {
                    0.0  // Already at target, switch immediately
                };

                // Switch when remaining distance ratio <= threshold
                let switch = remaining_ratio <= self.psc_switch_threshold;
                if always_trace || (self.id == 0 && track_movement) {
                    println!("  [SWITCH CHECK] EARLY MODE: remaining_ratio={:.4} <= threshold={:.4} = {}",
                        remaining_ratio, self.psc_switch_threshold, switch);
                }
                switch
            } else {
                // Standard mode: Switch when at boundary (cannot move further)
                let switch = dist_to_target_after < 0.5;
                if always_trace || (self.id == 0 && track_movement) {
                    println!("  [SWITCH CHECK] STANDARD MODE: dist={:.4} < 0.5 = {}",
                        dist_to_target_after, switch);
                }
                switch
            };

            if should_switch {
                if always_trace || (self.id == 0 && track_movement) {
                    println!("  [SWITCH] Switching subcells: current={:?} -> reserved={:?}", current, reserved);
                }
                let previous_current = current;

                // MIRROR TRIANGLE LOGIC: DISABLED
                // This feature was causing incorrect reservation releases after boundary crossing
                // Actors now use standard switching logic which correctly maintains reservations
                let mirror_reserved = false;

                // CRITICAL: Choose the CLOSER subcell as new PSC (before switching)
                // For diagonal moves, we have both 'reserved' (diagonal) and 'anchor' (H or V)
                // We must choose whichever is closer to the destination
                let (new_psc, psc_selection_info) = if let Some(anchor) = self.extra_reserved_subcells.get(0).copied() {
                    // Diagonal move - use directives to choose between current, reserved, and anchor

                    // Get subcell centers
                    let current_center = current.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_grid_size,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let reserved_center = reserved.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let anchor_center = anchor.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    // Use directive functions to decide - these use actor's ACTUAL POSITION
                    let should_use_anchor = actor_directives::should_switch_to_anchor(
                        self.id,
                        (self.fpos_x, self.fpos_y),
                        current_center,
                        anchor_center,
                        (dest_screen_x, dest_screen_y),
                    );

                    let should_use_reserved = actor_directives::should_switch_to_reserved(
                        self.id,
                        (self.fpos_x, self.fpos_y),
                        current_center,
                        reserved_center,
                        (dest_screen_x, dest_screen_y),
                        true, // has_anchor
                    );

                    // Choose best option based on directive results
                    let (chosen, chosen_name) = if should_use_anchor && should_use_reserved {
                        // Both are better than current, choose closer one to actor position
                        let dist_to_anchor = {
                            let dx = self.fpos_x - anchor_center.0;
                            let dy = self.fpos_y - anchor_center.1;
                            (dx * dx + dy * dy).sqrt()
                        };
                        let dist_to_reserved = {
                            let dx = self.fpos_x - reserved_center.0;
                            let dy = self.fpos_y - reserved_center.1;
                            (dx * dx + dy * dy).sqrt()
                        };

                        if always_trace || track_movement {
                            println!("  [PSC_DIAG] Both valid: dist_to_anchor={:.3} dist_to_reserved={:.3}",
                                dist_to_anchor, dist_to_reserved);
                        }

                        if dist_to_reserved < dist_to_anchor {
                            if always_trace || (self.id == 0 && track_movement) {
                                println!("  [PSC SELECTION] Chose RESERVED (diagonal, closer to actor)");
                            }
                            (reserved, "Reserved (directive, both valid)".to_string())
                        } else {
                            if always_trace || (self.id == 0 && track_movement) {
                                println!("  [PSC SELECTION] Chose ANCHOR (H/V, closer to actor)");
                            }
                            (anchor, "Anchor (directive, both valid)".to_string())
                        }
                    } else if should_use_reserved {
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [PSC SELECTION] Chose RESERVED (diagonal, directive)");
                        }
                        (reserved, "Reserved (directive)".to_string())
                    } else if should_use_anchor {
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [PSC SELECTION] Chose ANCHOR (H/V, directive)");
                        }
                        (anchor, "Anchor (directive)".to_string())
                    } else {
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [PSC SELECTION] Staying at CURRENT (directive)");
                        }
                        (SubCellCoord::from_subpoint(&current, self.subcell_grid_size), "Current (directive, stay)".to_string())
                    };

                    // Calculate distances for logging (using subcell centers for consistency with old logs)
                    let dist_reserved = Self::subcell_center_distance_to_destination(
                        &reserved,
                        dest_screen_x,
                        dest_screen_y,
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let dist_anchor = Self::subcell_center_distance_to_destination(
                        &anchor,
                        dest_screen_x,
                        dest_screen_y,
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    // DEBUG: Log diagonal PSC selection
                    if always_trace || track_movement {
                        let (curr_cx, curr_cy) = current.to_cell(self.subcell_grid_size);
                        let (curr_sx, curr_sy) = current.subcell_offset(self.subcell_grid_size);
                        println!("  [PSC_DIAG] old_psc=({},{},{},{}) reserved=({},{},{},{}) anchor=({},{},{},{})",
                            curr_cx, curr_cy, curr_sx, curr_sy,
                            reserved.cell_x, reserved.cell_y, reserved.sub_x, reserved.sub_y,
                            anchor.cell_x, anchor.cell_y, anchor.sub_x, anchor.sub_y);
                        println!("  [PSC_DIAG] dist_reserved={:.6} dist_anchor={:.6}",
                            dist_reserved, dist_anchor);
                    }

                    // Create PSC selection info for logging
                    let info = PSCSelectionInfo {
                        old_psc: SubCellCoord::from_subpoint(&current, self.subcell_grid_size),
                        reserved,
                        reserved_dist: dist_reserved,
                        anchor: Some(anchor),
                        anchor_dist: Some(dist_anchor),
                        chosen,
                        chosen_name,
                    };

                    (chosen, Some(info))
                } else {
                    // H/V move - no anchor, compare current vs reserved with hysteresis
                    // Hysteresis epsilon defined in diagonal section above
                    const HYSTERESIS_EPSILON: f32 = 0.001;

                    // Get subcell centers
                    let current_center = current.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_grid_size,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    let reserved_center = reserved.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    // Calculate distance from current PSC to destination (for logging)
                    let current_coord = SubCellCoord::from_subpoint(&current, self.subcell_grid_size);
                    let dist_current = Self::subcell_center_distance_to_destination(
                        &current_coord,
                        dest_screen_x,
                        dest_screen_y,
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    // Calculate distance from reserved PSC to destination (for logging)
                    let dist_reserved = Self::subcell_center_distance_to_destination(
                        &reserved,
                        dest_screen_x,
                        dest_screen_y,
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );

                    // DEBUG: Log H/V PSC selection with high precision
                    if always_trace || track_movement {
                        let (curr_cx, curr_cy) = current.to_cell(self.subcell_grid_size);
                        let (curr_sx, curr_sy) = current.subcell_offset(self.subcell_grid_size);
                        println!("  [PSC_HV] old_psc=({},{},{},{}) reserved=({},{},{},{}) no anchor",
                            curr_cx, curr_cy, curr_sx, curr_sy,
                            reserved.cell_x, reserved.cell_y, reserved.sub_x, reserved.sub_y);
                        println!("  [PSC_HV] dist_current={:.6} dist_reserved={:.6} diff={:.6}",
                            dist_current, dist_reserved, (dist_reserved - dist_current).abs());
                    }

                    // Use directive function to decide whether to switch
                    // This uses actor's ACTUAL POSITION instead of subcell centers
                    let should_switch = actor_directives::should_switch_to_reserved(
                        self.id,
                        (self.fpos_x, self.fpos_y),
                        current_center,
                        reserved_center,
                        (dest_screen_x, dest_screen_y),
                        false, // H/V move, no anchor
                    );

                    let (chosen, chosen_name) = if should_switch {
                        if always_trace || track_movement {
                            println!("  [PSC_HV_DIRECTIVE] SWITCH to RESERVED (directive approved)");
                        }
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [PSC SELECTION] H/V move, using reserved as new PSC (directive)");
                        }
                        (reserved, "Reserved (directive)".to_string())
                    } else {
                        if always_trace || track_movement {
                            println!("  [PSC_HV_DIRECTIVE] STAY at CURRENT (directive rejected switch)");
                        }
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [PSC SELECTION] H/V move, staying at current PSC (directive)");
                        }
                        (SubCellCoord::from_subpoint(&current, self.subcell_grid_size), "Current (directive)".to_string())
                    };

                    let info = PSCSelectionInfo {
                        old_psc: SubCellCoord::from_subpoint(&current, self.subcell_grid_size),
                        reserved,
                        reserved_dist: dist_reserved,
                        anchor: None,
                        anchor_dist: None,
                        chosen,
                        chosen_name,
                    };

                    (chosen, Some(info))
                };

                // Store PSC selection info for main.rs to log
                self.last_psc_selection = psc_selection_info;

                // If mirror failed or not applicable, fall back to standard switching
                if !mirror_reserved {
                    if always_trace || (self.id == 0 && track_movement) {
                        println!("  [SWITCH] Performing standard switch (mirror disabled)");
                    }

                    // Release old current sub-cell if different
                    let current_coord = SubCellCoord::from_subpoint(&current, self.subcell_grid_size);
                    if current_coord != reserved {
                        reservation_manager.release(current_coord, self.id);
                    }

                    // Release extra reserved cells (not chosen as PSC)
                    for extra in &self.extra_reserved_subcells {
                        if *extra != new_psc {
                            reservation_manager.release(*extra, self.id);
                        }
                    }
                    self.extra_reserved_subcells.clear();

                    // Update current to the CLOSER subcell
                    self.current_subcell = Some(new_psc.to_subpoint());
                    self.reserved_subcell = None;
                    // Clear locked values when changing reservation
                    self.locked_target = None;
                    self.locked_affinity = None;

                    // Register the new current subcell
                    reservation_manager.set_current(new_psc, self.id);

                    // Enter PscAlignment state for new PSC (Move → PscAlignment transition)
                    self.alignment_state = AlignmentState::PscAlignment;
                    let (center_x, center_y) = new_psc.to_screen_center_with_offset(
                        self.cell_width,
                        self.cell_height,
                        self.subcell_offset_x,
                        self.subcell_offset_y,
                    );
                    self.alignment_target = Some((center_x, center_y));
                    if always_trace || (self.id == 0 && track_movement) {
                        println!("  [SWITCH] Switched to new PSC ({},{},{},{}), entering PscAlignment to center ({:.1},{:.1})",
                            new_psc.cell_x, new_psc.cell_y, new_psc.sub_x, new_psc.sub_y, center_x, center_y);
                    }

                    // Record position when reaching subcell
                    if track_movement {
                        self.movement_track.push((self.fpos_x, self.fpos_y));
                    }
                }

                // Always try to reserve next cell after switching
                // (Only if mirror didn't already set up next reservation)
                // This prevents stuttering at boundary crossings
                // HOWEVER: If we just entered PscAlignment, skip reservation - must align first
                if !mirror_reserved && self.alignment_state != AlignmentState::PscAlignment {
                    let current = new_psc;  // Use the NEW PSC (closer subcell)

                    if always_trace || (self.id == 0 && track_movement) {
                        println!("  [SWITCH] After switching, current is now: {:?}", current);
                    }

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

                    if always_trace || (self.id == 0 && track_movement) {
                        println!("  [SWITCH] Checking if at destination subcell: current={:?} dest={:?} match={}",
                            current, dest_subcell, current == dest_subcell);
                    }

                    if current != dest_subcell {
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [SWITCH] Not at destination, attempting to reserve next subcell");
                        }
                        // Always attempt reservation after switching (no eagerness check)
                        // Eagerness only applies to early reservations (next-next cell)
                        // DestinationDirect: Try diagonal+anchor first, fallback to H/V
                        // Note: current is already SubCellCoord (new_psc) in this code path
                        let previous_current_coord = SubCellCoord::from_subpoint(&previous_current, self.subcell_grid_size);
                        let diag_success = self.try_reserve_diagonal_with_anchor(
                            &current,
                            Some(&previous_current_coord),
                            dx_to_dest,
                            dy_to_dest,
                            dest_screen_x,
                            dest_screen_y,
                            reservation_manager,
                            enable_anti_cross,
                            track_movement,
                        );

                        if !diag_success {
                            if always_trace || (self.id == 0 && track_movement) {
                                println!("  [SWITCH] Post-switch diagonal FAILED, trying H/V");
                            }
                            // Diagonal failed, try H/V
                            let hv_success = self.try_reserve_horizontal_vertical(
                                &current,
                                dx_to_dest,
                                dy_to_dest,
                                dest_screen_x,
                                dest_screen_y,
                                reservation_manager,
                                track_movement,
                            );
                            if always_trace || (self.id == 0 && track_movement) {
                                if hv_success {
                                    println!("  [SWITCH] Post-switch H/V SUCCEEDED");
                                } else {
                                    println!("  [SWITCH] Post-switch H/V FAILED - no reservation after switch!");
                                }
                            }
                        } else if always_trace || (self.id == 0 && track_movement) {
                            println!("  [SWITCH] Post-switch diagonal SUCCEEDED");
                        }
                    } else if always_trace || (self.id == 0 && track_movement) {
                        println!("  [SWITCH] At destination subcell, not reserving next");
                    }
                }

                return false;
            } else if always_trace || (self.id == 0 && track_movement) {
                println!("  [STATE] Has reservation but NOT switching yet (dist={:.2} threshold=0.5)", dist_to_target);
            }
        } else {
            // No reservation - attempt to reserve only when in Idle state (actor at PSC center)
            // The eagerness check applies to EARLY reservations (reserving next-next cell),
            // not to initial reservations (reserving the first next cell)

            if always_trace || (self.id == 0 && track_movement) {
                println!("  [STATE] NO RESERVATION - alignment_state={:?}", self.alignment_state);
                println!("  [STATE] Current subcell: {:?}", current);
                println!("  [STATE] Direction to dest: dx={:.1} dy={:.1}", dx_to_dest, dy_to_dest);
            }

            // Only attempt reservation when Idle (aligned to PSC center)
            if self.alignment_state == AlignmentState::Idle {
                // Try to reserve next sub-cell
                // DestinationDirect: Try diagonal+anchor first, fallback to H/V
                if self.id == 0 && track_movement {
                    println!("  NO RESERVATION (Idle): Attempting diagonal+anchor");
                }

                // Try diagonal+anchor first
                let current_coord = SubCellCoord::from_subpoint(&current, self.subcell_grid_size);
                let diagonal_success = self.try_reserve_diagonal_with_anchor(
                    &current_coord,
                    None,
                    dx_to_dest,
                    dy_to_dest,
                    dest_screen_x,
                    dest_screen_y,
                    reservation_manager,
                    enable_anti_cross,
                    track_movement,
                );

                if diagonal_success {
                    // Reservation succeeded - transition Idle → Move
                    self.alignment_state = AlignmentState::Move;
                    if always_trace || (self.id == 0 && track_movement) {
                        println!("  [RESERVE] Diagonal+anchor SUCCEEDED - transitioning Idle → Move");
                    }
                } else {
                    // Diagonal failed, try H/V
                    if always_trace || (self.id == 0 && track_movement) {
                        println!("  [RESERVE] Diagonal+anchor FAILED, trying H/V fallback");
                    }
                    let hv_success = self.try_reserve_horizontal_vertical(
                        &current_coord,
                        dx_to_dest,
                        dy_to_dest,
                        dest_screen_x,
                        dest_screen_y,
                        reservation_manager,
                        track_movement,
                    );

                    if hv_success {
                        // Reservation succeeded - transition Idle → Move
                        self.alignment_state = AlignmentState::Move;
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [RESERVE] H/V fallback SUCCEEDED - transitioning Idle → Move");
                        }
                    } else {
                        if always_trace || (self.id == 0 && track_movement) {
                            println!("  [RESERVE] H/V fallback FAILED - actor will stay in Idle (all directions blocked)");
                            println!("  [RESERVE] Actor is STUCK with no available moves");
                        }
                        // Note: If H/V also fails, actor stays in Idle state (all directions blocked)
                    }
                }
            } else if self.alignment_state == AlignmentState::PscAlignment {
                // Still aligning - don't attempt reservation yet
                if always_trace || (self.id == 0 && track_movement) {
                    println!("  [STATE] Still in PscAlignment, skipping reservation attempt");
                }
            }
        }

        if always_trace || (self.id == 0 && track_movement) {
            println!("  [RETURN] Returning false (not reached destination)");
        }
        false
    }

    /// Public API for updating actor with destination-direct movement
    /// Delegates to actor_directives::update_actor() (the decision root)
    pub fn update_subcell_destination_direct(
        &mut self,
        delta_time: f32,
        reservation_manager: &mut crate::subcell::SubCellReservationManager,
        enable_early_reservation: bool,
        _filter_backward: bool, // Unused, kept for API compatibility
        enable_anti_cross: bool,
        track_movement: bool,
        _reservation_threshold_distance: f32, // Unused, kept for API compatibility
        reservation_eagerness: crate::config::ReservationEagerness,
        release_eagerness: crate::config::ReleaseEagerness,
    ) -> bool {
        // Delegate to actor_directives::update_actor() (the decision root)
        actor_directives::update_actor(
            self,
            delta_time,
            reservation_manager,
            enable_early_reservation,
            enable_anti_cross,
            track_movement,
            reservation_eagerness,
            release_eagerness,
        )
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
            true,  // enable_lookahead
            0.5,  // psc_switch_threshold
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
            true,  // enable_lookahead
            0.5,  // psc_switch_threshold
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
            true,  // enable_lookahead
            0.5,  // psc_switch_threshold
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
            true,  // enable_lookahead
            0.5,  // psc_switch_threshold
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
        let mut actor = Actor::new(0, start_x, start_y, 10.0, 100.0, 6.0, cell_width, cell_height, 3, 0.0, 0.0, true, 0.5);

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
