/// Flat coordinate system for subcells
///
/// SubPoint represents a position in a flattened 2D subcell grid.
/// Unlike the hierarchical SubCellCoord (cell_x, cell_y, sub_x, sub_y),
/// SubPoint uses direct (x, y) coordinates in subcell space.
///
/// Conversion formulas:
/// - x = cell_x * grid_size + sub_x
/// - y = cell_y * grid_size + sub_y
///
/// Benefits:
/// - Simpler identity: 2-tuple instead of 4-tuple
/// - Direct arithmetic for neighbors
/// - No cell-boundary wrapping complexity
/// - 60% memory reduction (8 bytes vs 20 bytes)

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubPoint {
    pub x: i32,
    pub y: i32,
}

impl SubPoint {
    // ========== Construction ==========

    /// Create a new SubPoint from flat coordinates
    pub fn new(x: i32, y: i32) -> Self {
        SubPoint { x, y }
    }

    /// Create SubPoint from hierarchical cell/subcell coordinates
    ///
    /// # Arguments
    /// * `cell_x` - Cell X coordinate
    /// * `cell_y` - Cell Y coordinate
    /// * `sub_x` - Subcell X index (0 to grid_size-1)
    /// * `sub_y` - Subcell Y index (0 to grid_size-1)
    /// * `grid_size` - Subcell grid size (typically 2 for 2x2)
    pub fn from_cell_subcell(cell_x: i32, cell_y: i32, sub_x: i32, sub_y: i32, grid_size: i32) -> Self {
        SubPoint {
            x: cell_x * grid_size + sub_x,
            y: cell_y * grid_size + sub_y,
        }
    }

    // ========== Screen Conversion ==========

    /// Convert screen position to SubPoint
    pub fn from_screen_pos(
        screen_x: f32,
        screen_y: f32,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
    ) -> Self {
        let subcell_width = cell_width / grid_size as f32;
        let subcell_height = cell_height / grid_size as f32;

        SubPoint {
            x: (screen_x / subcell_width).floor() as i32,
            y: (screen_y / subcell_height).floor() as i32,
        }
    }

    /// Convert screen position to SubPoint with offset
    ///
    /// Offsets are in subcell units (e.g., 0.5 shifts by half a subcell)
    pub fn from_screen_pos_with_offset(
        screen_x: f32,
        screen_y: f32,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        let subcell_width = cell_width / grid_size as f32;
        let subcell_height = cell_height / grid_size as f32;

        let adjusted_x = screen_x + offset_x * subcell_width;
        let adjusted_y = screen_y + offset_y * subcell_height;

        SubPoint {
            x: (adjusted_x / subcell_width).floor() as i32,
            y: (adjusted_y / subcell_height).floor() as i32,
        }
    }

    /// Convert SubPoint to screen position (center of subcell)
    pub fn to_screen_center(
        &self,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
    ) -> (f32, f32) {
        let subcell_width = cell_width / grid_size as f32;
        let subcell_height = cell_height / grid_size as f32;

        (
            self.x as f32 * subcell_width + subcell_width / 2.0,
            self.y as f32 * subcell_height + subcell_height / 2.0,
        )
    }

    /// Convert SubPoint to screen position with offset
    pub fn to_screen_center_with_offset(
        &self,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
        offset_x: f32,
        offset_y: f32,
    ) -> (f32, f32) {
        let subcell_width = cell_width / grid_size as f32;
        let subcell_height = cell_height / grid_size as f32;

        (
            self.x as f32 * subcell_width + subcell_width / 2.0 - offset_x * subcell_width,
            self.y as f32 * subcell_height + subcell_height / 2.0 - offset_y * subcell_height,
        )
    }

    // ========== Cell Extraction (for compatibility) ==========

    /// Get the cell coordinates containing this SubPoint
    pub fn to_cell(&self, grid_size: i32) -> (i32, i32) {
        (
            self.x.div_euclid(grid_size),
            self.y.div_euclid(grid_size),
        )
    }

    /// Get the subcell offset within the containing cell
    pub fn subcell_offset(&self, grid_size: i32) -> (i32, i32) {
        (
            self.x.rem_euclid(grid_size),
            self.y.rem_euclid(grid_size),
        )
    }

    // ========== Neighbors ==========

    /// Get all 8 neighbors (N, NE, E, SE, S, SW, W, NW)
    pub fn get_neighbors(&self) -> [SubPoint; 8] {
        [
            SubPoint { x: self.x - 1, y: self.y - 1 }, // NW
            SubPoint { x: self.x,     y: self.y - 1 }, // N
            SubPoint { x: self.x + 1, y: self.y - 1 }, // NE
            SubPoint { x: self.x - 1, y: self.y     }, // W
            SubPoint { x: self.x + 1, y: self.y     }, // E
            SubPoint { x: self.x - 1, y: self.y + 1 }, // SW
            SubPoint { x: self.x,     y: self.y + 1 }, // S
            SubPoint { x: self.x + 1, y: self.y + 1 }, // SE
        ]
    }

    // ========== Direction & Alignment ==========

    /// Calculate normalized direction vector from this SubPoint to another
    pub fn direction_to(
        &self,
        other: &SubPoint,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
    ) -> (f32, f32) {
        let (self_x, self_y) = self.to_screen_center(cell_width, cell_height, grid_size);
        let (other_x, other_y) = other.to_screen_center(cell_width, cell_height, grid_size);

        let dx = other_x - self_x;
        let dy = other_y - self_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 0.001 {
            (0.0, 0.0)
        } else {
            (dx / distance, dy / distance)
        }
    }

    /// Calculate alignment score (dot product) between direction to other and target direction
    pub fn alignment_score(
        &self,
        other: &SubPoint,
        target_dir_x: f32,
        target_dir_y: f32,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
    ) -> f32 {
        let (dir_x, dir_y) = self.direction_to(other, cell_width, cell_height, grid_size);

        let target_len = (target_dir_x * target_dir_x + target_dir_y * target_dir_y).sqrt();
        if target_len < 0.001 {
            return 0.0;
        }

        let norm_target_x = target_dir_x / target_len;
        let norm_target_y = target_dir_y / target_len;

        dir_x * norm_target_x + dir_y * norm_target_y
    }

    /// Check if moving from this SubPoint to another would violate distance rule
    ///
    /// The distance rule: individual Manhattan-like distances in X and Y
    /// must never increase when moving toward a destination
    pub fn violates_distance_rule(
        &self,
        candidate: &SubPoint,
        actor_x: f32,
        actor_y: f32,
        dest_x: f32,
        dest_y: f32,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
        offset_x: f32,
        offset_y: f32,
        tolerance_multiplier: f32,
    ) -> bool {
        // Calculate candidate center position
        let (cand_center_x, cand_center_y) = candidate.to_screen_center_with_offset(
            cell_width,
            cell_height,
            grid_size,
            offset_x,
            offset_y,
        );

        // Calculate current distances
        let curr_dist_x = (dest_x - actor_x).abs();
        let curr_dist_y = (dest_y - actor_y).abs();

        // Calculate candidate distances
        let new_dist_x = (dest_x - cand_center_x).abs();
        let new_dist_y = (dest_y - cand_center_y).abs();

        // Allow small tolerance
        let subcell_width = cell_width / grid_size as f32;
        let subcell_height = cell_height / grid_size as f32;
        let tolerance_x = subcell_width * tolerance_multiplier;
        let tolerance_y = subcell_height * tolerance_multiplier;

        // Check if either distance increased beyond tolerance
        let x_increased = new_dist_x > curr_dist_x + tolerance_x;
        let y_increased = new_dist_y > curr_dist_y + tolerance_y;

        x_increased || y_increased
    }
}

// ========== Utility Functions ==========

/// Get the two counter-diagonal SubPoints between from and to
///
/// For a diagonal move from (x1, y1) to (x2, y2), returns:
/// - (x1, y2) and (x2, y1)
pub fn get_counter_diagonal(from: &SubPoint, to: &SubPoint) -> [SubPoint; 2] {
    [
        SubPoint { x: from.x, y: to.y },
        SubPoint { x: to.x, y: from.y },
    ]
}

/// Check if a move from one SubPoint to another is diagonal
pub fn is_diagonal_move(from: &SubPoint, to: &SubPoint) -> bool {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    dx > 0 && dy > 0
}

// ========== Reservation Manager ==========

pub struct SubPointReservationManager {
    reservations: HashMap<SubPoint, usize>,
    current_points: HashMap<usize, SubPoint>,
    grid_size: i32,
    world_cols: i32,
    world_rows: i32,
}

impl SubPointReservationManager {
    pub fn new(grid_size: i32, world_cols: i32, world_rows: i32) -> Self {
        SubPointReservationManager {
            reservations: HashMap::new(),
            current_points: HashMap::new(),
            grid_size,
            world_cols,
            world_rows,
        }
    }

    /// Try to reserve a SubPoint for an actor
    /// Returns true if successful, false if already reserved or out of bounds
    pub fn try_reserve(&mut self, point: SubPoint, actor_id: usize) -> bool {
        // Bounds check
        let (cell_x, cell_y) = point.to_cell(self.grid_size);
        if cell_x < 0 || cell_x >= self.world_cols || cell_y < 0 || cell_y >= self.world_rows {
            return false;
        }

        // Check reservation
        if let Some(&reserved_by) = self.reservations.get(&point) {
            if reserved_by == actor_id {
                return true; // Already reserved by this actor
            }
            return false; // Reserved by another actor
        }

        // Check current occupation
        for (&other_id, other_point) in &self.current_points {
            if other_point == &point && other_id != actor_id {
                return false; // Occupied by another actor
            }
        }

        self.reservations.insert(point, actor_id);
        true
    }

    /// Try to reserve multiple SubPoints atomically
    /// Either all succeed or all fail
    pub fn try_reserve_multiple(&mut self, points: &[SubPoint], actor_id: usize) -> bool {
        // Check all first (atomic)
        for point in points {
            let (cell_x, cell_y) = point.to_cell(self.grid_size);
            if cell_x < 0 || cell_x >= self.world_cols || cell_y < 0 || cell_y >= self.world_rows {
                return false;
            }

            if let Some(&reserved_by) = self.reservations.get(point) {
                if reserved_by != actor_id {
                    return false;
                }
            }

            for (&other_id, other_point) in &self.current_points {
                if other_point == point && other_id != actor_id {
                    return false;
                }
            }
        }

        // All clear - reserve all
        for point in points {
            self.reservations.insert(*point, actor_id);
        }
        true
    }

    /// Release a reservation
    pub fn release(&mut self, point: SubPoint, actor_id: usize) {
        if let Some(&reserved_by) = self.reservations.get(&point) {
            if reserved_by == actor_id {
                self.reservations.remove(&point);
            }
        }
    }

    /// Release all reservations for an actor
    pub fn release_all(&mut self, actor_id: usize) {
        self.reservations.retain(|_, &mut id| id != actor_id);
    }

    /// Get the owner (actor ID) of a SubPoint
    /// Checks both reservations and current positions
    pub fn get_owner(&self, point: &SubPoint) -> Option<usize> {
        // Check reservations first
        if let Some(&actor_id) = self.reservations.get(point) {
            return Some(actor_id);
        }

        // Check current positions
        for (&actor_id, current_point) in &self.current_points {
            if current_point == point {
                return Some(actor_id);
            }
        }

        None
    }

    /// Set the current SubPoint for an actor
    pub fn set_current(&mut self, point: SubPoint, actor_id: usize) {
        self.current_points.insert(actor_id, point);
    }

    /// Get the current SubPoint for an actor
    pub fn get_current(&self, actor_id: usize) -> Option<SubPoint> {
        self.current_points.get(&actor_id).copied()
    }

    /// Clear all reservations and current positions
    pub fn clear(&mut self) {
        self.reservations.clear();
        self.current_points.clear();
    }

    /// Get the grid size
    pub fn grid_size(&self) -> i32 {
        self.grid_size
    }

    /// Get the number of active reservations
    pub fn reservation_count(&self) -> usize {
        self.reservations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subpoint_construction() {
        let point = SubPoint::new(10, 20);
        assert_eq!(point.x, 10);
        assert_eq!(point.y, 20);
    }

    #[test]
    fn test_from_cell_subcell() {
        // 2x2 grid: cell (5, 3), subcell (1, 0)
        // Expected: x = 5*2 + 1 = 11, y = 3*2 + 0 = 6
        let point = SubPoint::from_cell_subcell(5, 3, 1, 0, 2);
        assert_eq!(point.x, 11);
        assert_eq!(point.y, 6);
    }

    #[test]
    fn test_to_cell() {
        let point = SubPoint::new(11, 6);
        let (cell_x, cell_y) = point.to_cell(2);
        assert_eq!(cell_x, 5);
        assert_eq!(cell_y, 3);
    }

    #[test]
    fn test_subcell_offset() {
        let point = SubPoint::new(11, 6);
        let (sub_x, sub_y) = point.subcell_offset(2);
        assert_eq!(sub_x, 1);
        assert_eq!(sub_y, 0);
    }

    #[test]
    fn test_screen_conversion_2x2() {
        let cell_width = 100.0;
        let cell_height = 100.0;
        let grid_size = 2;

        // Subcell (0, 0) in cell (5, 3)
        // x = 10, y = 6
        // Expected screen center: x = 10 * 50 + 25 = 525, y = 6 * 50 + 25 = 325
        let point = SubPoint::new(10, 6);
        let (screen_x, screen_y) = point.to_screen_center(cell_width, cell_height, grid_size);
        assert_eq!(screen_x, 525.0);
        assert_eq!(screen_y, 325.0);

        // Reverse conversion
        let point2 = SubPoint::from_screen_pos(525.0, 325.0, cell_width, cell_height, grid_size);
        assert_eq!(point2.x, 10);
        assert_eq!(point2.y, 6);
    }

    #[test]
    fn test_get_neighbors() {
        let point = SubPoint::new(10, 10);
        let neighbors = point.get_neighbors();

        // Check all 8 neighbors
        assert!(neighbors.contains(&SubPoint::new(9, 9)));   // NW
        assert!(neighbors.contains(&SubPoint::new(10, 9)));  // N
        assert!(neighbors.contains(&SubPoint::new(11, 9)));  // NE
        assert!(neighbors.contains(&SubPoint::new(9, 10)));  // W
        assert!(neighbors.contains(&SubPoint::new(11, 10))); // E
        assert!(neighbors.contains(&SubPoint::new(9, 11)));  // SW
        assert!(neighbors.contains(&SubPoint::new(10, 11))); // S
        assert!(neighbors.contains(&SubPoint::new(11, 11))); // SE

        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn test_is_diagonal_move() {
        let from = SubPoint::new(5, 5);
        let to_diag = SubPoint::new(6, 6);
        let to_horiz = SubPoint::new(6, 5);
        let to_vert = SubPoint::new(5, 6);

        assert!(is_diagonal_move(&from, &to_diag));
        assert!(!is_diagonal_move(&from, &to_horiz));
        assert!(!is_diagonal_move(&from, &to_vert));
    }

    #[test]
    fn test_counter_diagonal() {
        let from = SubPoint::new(5, 5);
        let to = SubPoint::new(7, 7);

        let [cd1, cd2] = get_counter_diagonal(&from, &to);

        assert_eq!(cd1, SubPoint::new(5, 7)); // (from.x, to.y)
        assert_eq!(cd2, SubPoint::new(7, 5)); // (to.x, from.y)
    }

    #[test]
    fn test_reservation_manager() {
        let mut mgr = SubPointReservationManager::new(2, 10, 10);

        let point1 = SubPoint::new(5, 5);
        let point2 = SubPoint::new(6, 6);

        // Actor 1 reserves point1
        assert!(mgr.try_reserve(point1, 1));
        assert_eq!(mgr.get_owner(&point1), Some(1));

        // Actor 2 cannot reserve point1
        assert!(!mgr.try_reserve(point1, 2));

        // Actor 2 can reserve point2
        assert!(mgr.try_reserve(point2, 2));
        assert_eq!(mgr.get_owner(&point2), Some(2));

        // Release point1
        mgr.release(point1, 1);
        assert_eq!(mgr.get_owner(&point1), None);

        // Now actor 2 can reserve point1
        assert!(mgr.try_reserve(point1, 2));
    }

    #[test]
    fn test_bounds_checking() {
        let mut mgr = SubPointReservationManager::new(2, 10, 10);

        // Valid point (cell 9, subcell 1 → x=19, within bounds since cell < 10)
        let valid = SubPoint::new(19, 19);
        assert!(mgr.try_reserve(valid, 1));

        // Invalid point (cell 10, out of bounds)
        let invalid = SubPoint::new(20, 5);
        assert!(!mgr.try_reserve(invalid, 1));
    }
}
