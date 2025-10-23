use std::collections::HashMap;

/// Sub-cell coordinate - identifies a specific sub-cell within a grid cell (2x2 or 3x3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubCellCoord {
    /// Grid cell X coordinate
    pub cell_x: i32,
    /// Grid cell Y coordinate
    pub cell_y: i32,
    /// Sub-cell X index within the cell (0-1 for 2x2, 0-2 for 3x3)
    pub sub_x: i32,
    /// Sub-cell Y index within the cell (0-1 for 2x2, 0-2 for 3x3)
    pub sub_y: i32,
    /// Grid size (2 for 2x2, 3 for 3x3)
    pub grid_size: i32,
}

impl SubCellCoord {
    pub fn new(cell_x: i32, cell_y: i32, sub_x: i32, sub_y: i32, grid_size: i32) -> Self {
        SubCellCoord { cell_x, cell_y, sub_x, sub_y, grid_size }
    }

    /// Convert screen position to sub-cell coordinate
    pub fn from_screen_pos(screen_x: f32, screen_y: f32, cell_width: f32, cell_height: f32, grid_size: i32) -> Self {
        // Determine which cell
        let cell_x = (screen_x / cell_width).floor() as i32;
        let cell_y = (screen_y / cell_height).floor() as i32;

        // Position within cell (0.0 to 1.0)
        let cell_local_x = (screen_x / cell_width) - cell_x as f32;
        let cell_local_y = (screen_y / cell_height) - cell_y as f32;

        // Convert to sub-cell index (0-1 for 2x2, 0-2 for 3x3)
        let sub_x = (cell_local_x * grid_size as f32).floor() as i32;
        let sub_y = (cell_local_y * grid_size as f32).floor() as i32;

        let max_index = grid_size - 1;
        SubCellCoord::new(cell_x, cell_y, sub_x.clamp(0, max_index), sub_y.clamp(0, max_index), grid_size)
    }

    /// Get screen position of sub-cell center
    pub fn to_screen_center(&self, cell_width: f32, cell_height: f32) -> (f32, f32) {
        let sub_cell_width = cell_width / self.grid_size as f32;
        let sub_cell_height = cell_height / self.grid_size as f32;

        let screen_x = self.cell_x as f32 * cell_width + (self.sub_x as f32 + 0.5) * sub_cell_width;
        let screen_y = self.cell_y as f32 * cell_height + (self.sub_y as f32 + 0.5) * sub_cell_height;

        (screen_x, screen_y)
    }

    /// Get all 8 neighbors (and self as 9th element for convenience)
    pub fn get_neighbors(&self) -> [SubCellCoord; 8] {
        let mut neighbors = [*self; 8];
        let mut idx = 0;
        let max_index = self.grid_size - 1;

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
                    (self.cell_x - 1, max_index)
                } else if target_sub_x > max_index {
                    (self.cell_x + 1, 0)
                } else {
                    (self.cell_x, target_sub_x)
                };

                let (new_cell_y, new_sub_y) = if target_sub_y < 0 {
                    (self.cell_y - 1, max_index)
                } else if target_sub_y > max_index {
                    (self.cell_y + 1, 0)
                } else {
                    (self.cell_y, target_sub_y)
                };

                neighbors[idx] = SubCellCoord::new(new_cell_x, new_cell_y, new_sub_x, new_sub_y, self.grid_size);
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
    /// Grid size (2 for 2x2, 3 for 3x3)
    grid_size: i32,
}

impl SubCellReservationManager {
    pub fn new(grid_size: i32) -> Self {
        SubCellReservationManager {
            reservations: HashMap::new(),
            grid_size,
        }
    }

    pub fn grid_size(&self) -> i32 {
        self.grid_size
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

    /// Change the grid size and clear all reservations
    pub fn set_grid_size(&mut self, grid_size: i32) {
        self.grid_size = grid_size;
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

/// Spread actors across different CELLS (not sub-cells)
/// This is for final destinations - each actor gets a different cell
/// Uses spiral pattern: center cell, then 8 neighbors, then next ring, etc.
/// Returns a list of (cell_x, cell_y) positions
pub fn spread_cell_destinations(
    target_cell_x: i32,
    target_cell_y: i32,
    num_actors: usize,
) -> Vec<(i32, i32)> {
    let mut destinations = Vec::new();

    // Start with the target cell itself
    destinations.push((target_cell_x, target_cell_y));

    if num_actors <= 1 {
        return destinations;
    }

    // Spiral outward in rings around the target cell
    let mut ring: i32 = 1;
    while destinations.len() < num_actors {
        // Add cells in the current ring
        // Order: N, E, S, W, NE, SE, SW, NW (8 directions for ring 1)
        // For larger rings, we need to add all cells around the perimeter

        for dy in -ring..=ring {
            for dx in -ring..=ring {
                // Skip cells not on the perimeter (only include edge of ring)
                if dy.abs() != ring && dx.abs() != ring {
                    continue;
                }

                let cell_x = target_cell_x + dx;
                let cell_y = target_cell_y + dy;

                destinations.push((cell_x, cell_y));

                if destinations.len() >= num_actors {
                    return destinations;
                }
            }
        }

        ring += 1;
    }

    destinations
}

/// Find unique sub-cell destinations for multiple actors
/// NOTE: This is for INTERMEDIATE movement only, not final destinations!
/// Returns a list of sub-cell coordinates spread around the target position
/// Uses a spiral pattern expanding from center to ensure good distribution
pub fn spread_subcell_destinations(
    target_cell_x: i32,
    target_cell_y: i32,
    num_actors: usize,
    cell_width: f32,
    cell_height: f32,
    grid_size: i32,
) -> Vec<SubCellCoord> {
    let mut destinations = Vec::new();
    let max_index = grid_size - 1;
    let center_index = grid_size / 2;

    // Start with center sub-cell of target cell
    let center_subcell = SubCellCoord::new(target_cell_x, target_cell_y, center_index, center_index, grid_size);
    destinations.push(center_subcell);

    if num_actors <= 1 {
        return destinations;
    }

    // Add remaining sub-cells in target cell
    for dy in 0..grid_size {
        for dx in 0..grid_size {
            if dx == center_index && dy == center_index {
                continue; // Skip center (already added)
            }
            destinations.push(SubCellCoord::new(target_cell_x, target_cell_y, dx, dy, grid_size));
            if destinations.len() >= num_actors {
                return destinations;
            }
        }
    }

    // If we need more, spiral outward to neighboring cells
    // Order: N, E, S, W, NE, SE, SW, NW
    let neighbor_offsets = [
        (0, -1),  // N
        (1, 0),   // E
        (0, 1),   // S
        (-1, 0),  // W
        (1, -1),  // NE
        (1, 1),   // SE
        (-1, 1),  // SW
        (-1, -1), // NW
    ];

    for (cell_dx, cell_dy) in neighbor_offsets.iter() {
        let neighbor_cell_x = target_cell_x + cell_dx;
        let neighbor_cell_y = target_cell_y + cell_dy;

        // Add all sub-cells of this neighbor cell
        for sub_dy in 0..grid_size {
            for sub_dx in 0..grid_size {
                destinations.push(SubCellCoord::new(
                    neighbor_cell_x,
                    neighbor_cell_y,
                    sub_dx,
                    sub_dy,
                    grid_size,
                ));
                if destinations.len() >= num_actors {
                    return destinations;
                }
            }
        }
    }

    destinations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subcell_from_screen_pos() {
        let cell_width = 30.0;
        let cell_height = 30.0;

        // Center of cell (0, 0), sub-cell (1, 1) for 3x3 grid
        let subcell = SubCellCoord::from_screen_pos(15.0, 15.0, cell_width, cell_height, 3);
        assert_eq!(subcell.cell_x, 0);
        assert_eq!(subcell.cell_y, 0);
        assert_eq!(subcell.sub_x, 1);
        assert_eq!(subcell.sub_y, 1);
        assert_eq!(subcell.grid_size, 3);

        // Top-left corner of cell (0, 0), sub-cell (0, 0) for 3x3 grid
        let subcell = SubCellCoord::from_screen_pos(2.0, 2.0, cell_width, cell_height, 3);
        assert_eq!(subcell.cell_x, 0);
        assert_eq!(subcell.cell_y, 0);
        assert_eq!(subcell.sub_x, 0);
        assert_eq!(subcell.sub_y, 0);
    }

    #[test]
    fn test_subcell_to_screen_center() {
        let cell_width = 30.0;
        let cell_height = 30.0;

        let subcell = SubCellCoord::new(0, 0, 1, 1, 3);
        let (x, y) = subcell.to_screen_center(cell_width, cell_height);

        // Center of middle sub-cell should be at (15, 15) for 3x3 grid
        assert!((x - 15.0).abs() < 0.1);
        assert!((y - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_reservation_manager() {
        let mut manager = SubCellReservationManager::new(3);
        let subcell = SubCellCoord::new(0, 0, 1, 1, 3);

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
