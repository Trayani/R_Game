use std::collections::HashMap;

/// Sub-cell coordinate - identifies a specific 3x3 sub-cell within a grid cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubCellCoord {
    /// Grid cell X coordinate
    pub cell_x: i32,
    /// Grid cell Y coordinate
    pub cell_y: i32,
    /// Sub-cell X index within the cell (0, 1, or 2 for 3x3)
    pub sub_x: i32,
    /// Sub-cell Y index within the cell (0, 1, or 2 for 3x3)
    pub sub_y: i32,
}

impl SubCellCoord {
    pub fn new(cell_x: i32, cell_y: i32, sub_x: i32, sub_y: i32) -> Self {
        SubCellCoord { cell_x, cell_y, sub_x, sub_y }
    }

    /// Convert screen position to sub-cell coordinate
    pub fn from_screen_pos(screen_x: f32, screen_y: f32, cell_width: f32, cell_height: f32) -> Self {
        // Determine which cell
        let cell_x = (screen_x / cell_width).floor() as i32;
        let cell_y = (screen_y / cell_height).floor() as i32;

        // Position within cell (0.0 to 1.0)
        let cell_local_x = (screen_x / cell_width) - cell_x as f32;
        let cell_local_y = (screen_y / cell_height) - cell_y as f32;

        // Convert to sub-cell index (0, 1, or 2 for 3x3)
        let sub_x = (cell_local_x * 3.0).floor() as i32;
        let sub_y = (cell_local_y * 3.0).floor() as i32;

        SubCellCoord::new(cell_x, cell_y, sub_x.clamp(0, 2), sub_y.clamp(0, 2))
    }

    /// Get screen position of sub-cell center
    pub fn to_screen_center(&self, cell_width: f32, cell_height: f32) -> (f32, f32) {
        let sub_cell_width = cell_width / 3.0;
        let sub_cell_height = cell_height / 3.0;

        let screen_x = self.cell_x as f32 * cell_width + (self.sub_x as f32 + 0.5) * sub_cell_width;
        let screen_y = self.cell_y as f32 * cell_height + (self.sub_y as f32 + 0.5) * sub_cell_height;

        (screen_x, screen_y)
    }

    /// Get all 8 neighbors (and self as 9th element for convenience)
    pub fn get_neighbors(&self) -> [SubCellCoord; 8] {
        let mut neighbors = [*self; 8];
        let mut idx = 0;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Skip self
                }

                // Calculate target sub-cell position
                let target_sub_x = self.sub_x + dx;
                let target_sub_y = self.sub_y + dy;

                // Handle cell boundary crossing
                let (new_cell_x, new_sub_x) = if target_sub_x < 0 {
                    (self.cell_x - 1, 2)
                } else if target_sub_x > 2 {
                    (self.cell_x + 1, 0)
                } else {
                    (self.cell_x, target_sub_x)
                };

                let (new_cell_y, new_sub_y) = if target_sub_y < 0 {
                    (self.cell_y - 1, 2)
                } else if target_sub_y > 2 {
                    (self.cell_y + 1, 0)
                } else {
                    (self.cell_y, target_sub_y)
                };

                neighbors[idx] = SubCellCoord::new(new_cell_x, new_cell_y, new_sub_x, new_sub_y);
                idx += 1;
            }
        }

        neighbors
    }

    /// Get direction vector to another sub-cell (normalized)
    pub fn direction_to(&self, other: &SubCellCoord, cell_width: f32, cell_height: f32) -> (f32, f32) {
        let (self_x, self_y) = self.to_screen_center(cell_width, cell_height);
        let (other_x, other_y) = other.to_screen_center(cell_width, cell_height);

        let dx = other_x - self_x;
        let dy = other_y - self_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 0.001 {
            (0.0, 0.0)
        } else {
            (dx / distance, dy / distance)
        }
    }

    /// Calculate alignment score with a target direction
    /// Returns value from -1.0 (opposite) to 1.0 (same direction)
    pub fn alignment_score(&self, other: &SubCellCoord, target_dir_x: f32, target_dir_y: f32, cell_width: f32, cell_height: f32) -> f32 {
        let (dir_x, dir_y) = self.direction_to(other, cell_width, cell_height);

        // Normalize target direction
        let target_len = (target_dir_x * target_dir_x + target_dir_y * target_dir_y).sqrt();
        if target_len < 0.001 {
            return 0.0;
        }
        let norm_target_x = target_dir_x / target_len;
        let norm_target_y = target_dir_y / target_len;

        // Dot product
        dir_x * norm_target_x + dir_y * norm_target_y
    }
}

/// Sub-cell reservation manager
pub struct SubCellReservationManager {
    /// Map from sub-cell coordinate to actor ID that reserved it
    reservations: HashMap<SubCellCoord, usize>,
}

impl SubCellReservationManager {
    pub fn new() -> Self {
        SubCellReservationManager {
            reservations: HashMap::new(),
        }
    }

    /// Try to reserve a sub-cell for an actor
    /// Returns true if reservation succeeded, false if already reserved
    pub fn try_reserve(&mut self, subcell: SubCellCoord, actor_id: usize) -> bool {
        if let Some(&reserved_by) = self.reservations.get(&subcell) {
            // Already reserved
            if reserved_by == actor_id {
                // Already reserved by this actor - that's ok
                return true;
            }
            return false;
        }

        // Not reserved - reserve it
        self.reservations.insert(subcell, actor_id);
        true
    }

    /// Release a sub-cell reservation
    pub fn release(&mut self, subcell: SubCellCoord, actor_id: usize) {
        if let Some(&reserved_by) = self.reservations.get(&subcell) {
            if reserved_by == actor_id {
                self.reservations.remove(&subcell);
            }
        }
    }

    /// Check if a sub-cell is reserved (and by whom)
    pub fn is_reserved(&self, subcell: &SubCellCoord) -> Option<usize> {
        self.reservations.get(subcell).copied()
    }

    /// Clear all reservations (useful for resetting)
    pub fn clear(&mut self) {
        self.reservations.clear();
    }

    /// Get total number of reservations (for debugging)
    pub fn reservation_count(&self) -> usize {
        self.reservations.len()
    }
}

/// Find the best neighbor sub-cell that aligns with the target direction
/// Returns up to 5 candidates in priority order:
/// 1. Best aligned neighbor
/// 2. Clockwise neighbor
/// 3. Counter-clockwise neighbor
/// 4. 2x clockwise
/// 5. 2x counter-clockwise
pub fn find_best_neighbors(
    current: &SubCellCoord,
    target_dir_x: f32,
    target_dir_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> Vec<SubCellCoord> {
    let neighbors = current.get_neighbors();

    // Calculate alignment scores for all neighbors
    let mut scored_neighbors: Vec<(SubCellCoord, f32)> = neighbors
        .iter()
        .map(|n| (*n, current.alignment_score(n, target_dir_x, target_dir_y, cell_width, cell_height)))
        .collect();

    // Sort by score (highest first)
    scored_neighbors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Return top 5 candidates
    scored_neighbors.iter().take(5).map(|(coord, _)| *coord).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subcell_from_screen_pos() {
        let cell_width = 30.0;
        let cell_height = 30.0;

        // Center of cell (0, 0), sub-cell (1, 1)
        let subcell = SubCellCoord::from_screen_pos(15.0, 15.0, cell_width, cell_height);
        assert_eq!(subcell.cell_x, 0);
        assert_eq!(subcell.cell_y, 0);
        assert_eq!(subcell.sub_x, 1);
        assert_eq!(subcell.sub_y, 1);

        // Top-left corner of cell (0, 0), sub-cell (0, 0)
        let subcell = SubCellCoord::from_screen_pos(2.0, 2.0, cell_width, cell_height);
        assert_eq!(subcell.cell_x, 0);
        assert_eq!(subcell.cell_y, 0);
        assert_eq!(subcell.sub_x, 0);
        assert_eq!(subcell.sub_y, 0);
    }

    #[test]
    fn test_subcell_to_screen_center() {
        let cell_width = 30.0;
        let cell_height = 30.0;

        let subcell = SubCellCoord::new(0, 0, 1, 1);
        let (x, y) = subcell.to_screen_center(cell_width, cell_height);

        // Center of middle sub-cell should be at (15, 15)
        assert!((x - 15.0).abs() < 0.1);
        assert!((y - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_reservation_manager() {
        let mut manager = SubCellReservationManager::new();
        let subcell = SubCellCoord::new(0, 0, 1, 1);

        // Reserve for actor 0
        assert!(manager.try_reserve(subcell, 0));
        assert_eq!(manager.is_reserved(&subcell), Some(0));

        // Try to reserve for actor 1 - should fail
        assert!(!manager.try_reserve(subcell, 1));
        assert_eq!(manager.is_reserved(&subcell), Some(0));

        // Release by actor 0
        manager.release(subcell, 0);
        assert_eq!(manager.is_reserved(&subcell), None);

        // Now actor 1 can reserve
        assert!(manager.try_reserve(subcell, 1));
        assert_eq!(manager.is_reserved(&subcell), Some(1));
    }
}
