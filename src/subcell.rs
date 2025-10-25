use std::collections::{HashMap, HashSet};

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
        Self::from_screen_pos_with_offset(screen_x, screen_y, cell_width, cell_height, grid_size, 0.0, 0.0)
    }

    /// Convert screen position to sub-cell coordinate with offset
    /// offset_x, offset_y: offset in sub-cell units (e.g., 0.5 means shift by half a sub-cell)
    pub fn from_screen_pos_with_offset(
        screen_x: f32,
        screen_y: f32,
        cell_width: f32,
        cell_height: f32,
        grid_size: i32,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        // Determine which cell
        let cell_x = (screen_x / cell_width).floor() as i32;
        let cell_y = (screen_y / cell_height).floor() as i32;

        // Position within cell (0.0 to 1.0)
        let cell_local_x = (screen_x / cell_width) - cell_x as f32;
        let cell_local_y = (screen_y / cell_height) - cell_y as f32;

        // Apply offset (offset is in sub-cell units, so divide by grid_size to get cell-local units)
        let offset_cell_x = offset_x / grid_size as f32;
        let offset_cell_y = offset_y / grid_size as f32;

        let adjusted_local_x = cell_local_x + offset_cell_x;
        let adjusted_local_y = cell_local_y + offset_cell_y;

        // Convert to sub-cell index (0-1 for 2x2, 0-2 for 3x3)
        let sub_x = (adjusted_local_x * grid_size as f32).floor() as i32;
        let sub_y = (adjusted_local_y * grid_size as f32).floor() as i32;

        let max_index = grid_size - 1;
        SubCellCoord::new(cell_x, cell_y, sub_x.clamp(0, max_index), sub_y.clamp(0, max_index), grid_size)
    }

    /// Get screen position of sub-cell center
    pub fn to_screen_center(&self, cell_width: f32, cell_height: f32) -> (f32, f32) {
        self.to_screen_center_with_offset(cell_width, cell_height, 0.0, 0.0)
    }

    /// Get screen position of sub-cell center with offset
    /// offset_x, offset_y: offset in sub-cell units (e.g., 0.5 means shift by half a sub-cell)
    pub fn to_screen_center_with_offset(
        &self,
        cell_width: f32,
        cell_height: f32,
        offset_x: f32,
        offset_y: f32,
    ) -> (f32, f32) {
        let sub_cell_width = cell_width / self.grid_size as f32;
        let sub_cell_height = cell_height / self.grid_size as f32;

        // Calculate base position (center of sub-cell without offset)
        let base_x = self.cell_x as f32 * cell_width + (self.sub_x as f32 + 0.5) * sub_cell_width;
        let base_y = self.cell_y as f32 * cell_height + (self.sub_y as f32 + 0.5) * sub_cell_height;

        // Apply offset (subtract because we're going from shifted coords back to screen coords)
        let screen_x = base_x - offset_x * sub_cell_width;
        let screen_y = base_y - offset_y * sub_cell_height;

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
    /// Map from actor ID to their current sub-cell position
    current_subcells: HashMap<usize, SubCellCoord>,
    /// Grid size (2 for 2x2, 3 for 3x3)
    grid_size: i32,
}

impl SubCellReservationManager {
    pub fn new(grid_size: i32) -> Self {
        SubCellReservationManager {
            reservations: HashMap::new(),
            current_subcells: HashMap::new(),
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

    /// Try to reserve multiple sub-cells atomically for an actor
    /// Returns true if ALL cells could be reserved, false otherwise
    /// If reservation fails, no cells are reserved (atomic operation)
    pub fn try_reserve_multiple(&mut self, subcells: &[SubCellCoord], actor_id: usize) -> bool {
        // First check if all cells are available or already reserved by this actor
        for subcell in subcells {
            if let Some(&reserved_by) = self.reservations.get(subcell) {
                if reserved_by != actor_id {
                    // Cell is reserved by another actor - fail
                    return false;
                }
            }
        }

        // All cells are available - reserve them all
        for subcell in subcells {
            self.reservations.insert(*subcell, actor_id);
        }

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

    /// Set the current sub-cell for an actor
    pub fn set_current(&mut self, subcell: SubCellCoord, actor_id: usize) {
        self.current_subcells.insert(actor_id, subcell);
    }

    /// Get the owner of a sub-cell (checks both reservations and current positions)
    /// Returns Some(actor_id) if the sub-cell is either reserved or currently occupied
    pub fn get_owner(&self, subcell: &SubCellCoord) -> Option<usize> {
        // First check reservations
        if let Some(&actor_id) = self.reservations.get(subcell) {
            return Some(actor_id);
        }

        // Then check if any actor is currently at this sub-cell
        for (&actor_id, current_sc) in &self.current_subcells {
            if current_sc == subcell {
                return Some(actor_id);
            }
        }

        None
    }

    /// Clear all reservations (useful for resetting)
    pub fn clear(&mut self) {
        self.reservations.clear();
        self.current_subcells.clear();
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

/// Find four sub-cells in a square configuration toward the primary direction
/// Returns Some((best_neighbor, [3 additional cells])) if a valid square can be formed,
/// or None if the primary direction is unclear or no valid square exists
///
/// The square is formed by:
/// 1. Finding the best aligned neighbor (the primary move)
/// 2. Creating a 2x2 block by extending perpendicular to the movement direction
///
/// Example for horizontal movement (moving right):
/// ```text
/// C = current, B = best
/// C B
/// X Y
/// ```
/// The square includes B (best), and cells X, Y that form a 2x2 block
pub fn find_square_reservation(
    current: &SubCellCoord,
    target_dir_x: f32,
    target_dir_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> Option<(SubCellCoord, [SubCellCoord; 3])> {
    // Determine primary direction (which component is larger)
    let abs_x = target_dir_x.abs();
    let abs_y = target_dir_y.abs();

    // Need significant directional movement
    if abs_x < 0.1 && abs_y < 0.1 {
        return None;
    }

    let neighbors = current.get_neighbors();

    // Calculate alignment scores for all neighbors
    let mut scored_neighbors: Vec<(SubCellCoord, f32)> = neighbors
        .iter()
        .map(|n| (*n, current.alignment_score(n, target_dir_x, target_dir_y, cell_width, cell_height)))
        .collect();

    // Sort by score (highest first)
    scored_neighbors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Get the best aligned neighbor
    let best = scored_neighbors[0].0;

    // Determine perpendicular direction to form a 2x2 square
    // The square extends in the perpendicular direction from both current and best
    let (perp_cell_dx, perp_cell_dy, perp_sub_dx, perp_sub_dy) = if abs_x > abs_y {
        // Primary direction is horizontal
        // Square extends perpendicular (vertically)
        let sign = if target_dir_y >= 0.0 { 1 } else { -1 };
        (0, sign, 0, 0)
    } else {
        // Primary direction is vertical
        // Square extends perpendicular (horizontally)
        let sign = if target_dir_x >= 0.0 { 1 } else { -1 };
        (sign, 0, 0, 0)
    };

    // Build the square:
    // Cell 0 (best): already have it
    // Cell 1: perpendicular from current
    // Cell 2: perpendicular from best
    // This forms: [current, cell1] in one row, [best, cell2] in another row (or columns if vertical)
    let mut square = [best; 3];

    // Actually, let's think about this differently:
    // We want a 2x2 square with best as one corner
    // The square should extend in the direction of movement

    // Let's use a simpler approach: get the best neighbor's neighbors
    // and find those that form a square configuration
    let best_neighbors = best.get_neighbors();

    // Find neighbors of best that are also neighbors of current (these form a square)
    let current_neighbor_set: std::collections::HashSet<_> = neighbors.iter().copied().collect();

    let mut adjacent_to_both: Vec<SubCellCoord> = best_neighbors
        .iter()
        .filter(|n| current_neighbor_set.contains(n))
        .copied()
        .collect();

    if adjacent_to_both.len() < 2 {
        // Can't form a proper square
        return None;
    }

    // Sort by alignment to perpendicular direction
    adjacent_to_both.sort_by(|a, b| {
        let a_score = current.alignment_score(a,
            perp_cell_dx as f32 + perp_sub_dx as f32 * 0.5,
            perp_cell_dy as f32 + perp_sub_dy as f32 * 0.5,
            cell_width, cell_height);
        let b_score = current.alignment_score(b,
            perp_cell_dx as f32 + perp_sub_dx as f32 * 0.5,
            perp_cell_dy as f32 + perp_sub_dy as f32 * 0.5,
            cell_width, cell_height);
        b_score.partial_cmp(&a_score).unwrap()
    });

    // Take the best perpendicular neighbor
    let perp_from_current = adjacent_to_both[0];

    // The fourth corner is the remaining cell
    // It should be adjacent to both best and perp_from_current
    let perp_from_current_neighbors = perp_from_current.get_neighbors();
    let best_neighbor_set: std::collections::HashSet<_> = best_neighbors.iter().copied().collect();

    let fourth_corner = perp_from_current_neighbors
        .iter()
        .find(|n| best_neighbor_set.contains(n) && **n != *current)
        .copied();

    match fourth_corner {
        Some(corner) => {
            square[0] = perp_from_current;
            square[1] = corner;
            square[2] = best;  // Redundant but clear
            Some((best, square))
        },
        None => None,
    }
}

/// Helper function to apply cell and sub-cell offsets
fn apply_offset(base: SubCellCoord, cell_dx: i32, cell_dy: i32, sub_dx: i32, sub_dy: i32, grid_size: i32) -> SubCellCoord {
    let max_index = grid_size - 1;

    let mut new_cell_x = base.cell_x + cell_dx;
    let mut new_cell_y = base.cell_y + cell_dy;
    let mut new_sub_x = base.sub_x + sub_dx;
    let mut new_sub_y = base.sub_y + sub_dy;

    // Handle sub-cell boundary crossing
    if new_sub_x < 0 {
        new_cell_x -= 1;
        new_sub_x = max_index;
    } else if new_sub_x > max_index {
        new_cell_x += 1;
        new_sub_x = 0;
    }

    if new_sub_y < 0 {
        new_cell_y -= 1;
        new_sub_y = max_index;
    } else if new_sub_y > max_index {
        new_cell_y += 1;
        new_sub_y = 0;
    }

    SubCellCoord::new(new_cell_x, new_cell_y, new_sub_x, new_sub_y, grid_size)
}

/// Get the two counter-diagonal subcells for a diagonal move
/// These are the cells that would form a crossing pattern if another actor moves through them
/// Example: moving from (0,0) to (1,1) returns [(0,1), (1,0)]
pub fn get_counter_diagonal_subcells(current: &SubCellCoord, target: &SubCellCoord) -> [SubCellCoord; 2] {
    // Calculate the direction of movement
    let dx = target.cell_x - current.cell_x;
    let dy = target.cell_y - current.cell_y;
    let sub_dx = target.sub_x - current.sub_x;
    let sub_dy = target.sub_y - current.sub_y;

    // Counter-diagonal cells are:
    // 1. Same cell_x as current, same cell_y as target, with adjusted sub indices
    // 2. Same cell_x as target, same cell_y as current, with adjusted sub indices

    // For a move like (0,0,1,1) -> (0,0,2,2), counter-diagonals are (0,0,1,2) and (0,0,2,1)
    // For a move across cells like (0,0,2,2) -> (1,1,0,0), counter-diagonals span cells

    let counter1 = SubCellCoord::new(
        current.cell_x + dx,
        current.cell_y,
        current.sub_x + sub_dx,
        target.sub_y,
        current.grid_size,
    );

    let counter2 = SubCellCoord::new(
        current.cell_x,
        current.cell_y + dy,
        target.sub_x,
        current.sub_y + sub_dy,
        current.grid_size,
    );

    [counter1, counter2]
}

/// Find the best neighbor sub-cell that aligns with the target direction
/// Returns up to 5 candidates in priority order:
/// 1. Best aligned neighbor
/// 2. Clockwise neighbor
/// 3. Counter-clockwise neighbor
/// 4. 2x clockwise
/// 5. 2x counter-clockwise
///
/// If filter_backward is true, candidates with negative scores (moving away from destination)
/// are filtered out, ensuring actors never increase their distance to the destination.
pub fn find_best_neighbors(
    current: &SubCellCoord,
    target_dir_x: f32,
    target_dir_y: f32,
    cell_width: f32,
    cell_height: f32,
    filter_backward: bool,
) -> Vec<SubCellCoord> {
    let neighbors = current.get_neighbors();

    // Calculate alignment scores for all neighbors
    let mut scored_neighbors: Vec<(SubCellCoord, f32)> = neighbors
        .iter()
        .map(|n| (*n, current.alignment_score(n, target_dir_x, target_dir_y, cell_width, cell_height)))
        .collect();

    // Sort by score (highest first)
    scored_neighbors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Filter out backward moves if requested (score < 0.0 means moving away from destination)
    if filter_backward {
        scored_neighbors.retain(|(_, score)| *score >= 0.0);
    }

    // Return top 5 candidates (or fewer if filtered)
    scored_neighbors.iter().take(5).map(|(coord, _)| *coord).collect()
}

/// Filter candidates to only those that decrease Euclidean distance to destination
/// This ensures monotonic approach: overall distance never increases
/// Allows small coordinate increases if total distance still decreases
fn filter_monotonic_approach(
    current: &SubCellCoord,
    candidates: Vec<SubCellCoord>,
    dest_screen_x: f32,
    dest_screen_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> Vec<SubCellCoord> {
    // Calculate current distance to destination
    let (curr_x, curr_y) = current.to_screen_center(cell_width, cell_height);
    let curr_dx = dest_screen_x - curr_x;
    let curr_dy = dest_screen_y - curr_y;
    let current_distance = (curr_dx * curr_dx + curr_dy * curr_dy).sqrt();

    // Filter candidates that would increase distance
    candidates
        .into_iter()
        .filter(|candidate| {
            let (cand_x, cand_y) = candidate.to_screen_center(cell_width, cell_height);
            let cand_dx = dest_screen_x - cand_x;
            let cand_dy = dest_screen_y - cand_y;
            let candidate_distance = (cand_dx * cand_dx + cand_dy * cand_dy).sqrt();

            // Only keep candidates that decrease or maintain distance
            candidate_distance <= current_distance
        })
        .collect()
}

/// Find the best 3 neighbors (strictly limited to ±45° alternatives)
/// Returns exactly 3 candidates (or fewer if filtered):
/// 1. Best aligned neighbor
/// 2. Clockwise neighbor (±45°)
/// 3. Counter-clockwise neighbor (±45°)
///
/// This provides more deterministic pathfinding with fewer alternatives.
/// If filter_backward is true, candidates with negative scores are filtered out.
/// If use_monotonic_filter is true, ensures Euclidean distance to destination never increases.
/// If allow_fallback is true and all candidates are filtered, returns best candidate anyway.
pub fn find_best_3_neighbors(
    current: &SubCellCoord,
    target_dir_x: f32,
    target_dir_y: f32,
    cell_width: f32,
    cell_height: f32,
    filter_backward: bool,
    dest_screen_x: f32,
    dest_screen_y: f32,
    use_monotonic_filter: bool,
    allow_fallback: bool,
) -> Vec<SubCellCoord> {
    let neighbors = current.get_neighbors();

    // Calculate alignment scores for all neighbors
    let mut scored_neighbors: Vec<(SubCellCoord, f32)> = neighbors
        .iter()
        .map(|n| (*n, current.alignment_score(n, target_dir_x, target_dir_y, cell_width, cell_height)))
        .collect();

    // Sort by score (highest first)
    scored_neighbors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Filter out backward moves if requested (score < 0.0 means moving away from destination)
    if filter_backward {
        scored_neighbors.retain(|(_, score)| *score >= 0.0);
    }

    // Take only top 3 candidates
    let candidates: Vec<SubCellCoord> = scored_neighbors.iter().take(3).map(|(coord, _)| *coord).collect();

    // Apply monotonic distance filter if requested (for Basic3 modes)
    if use_monotonic_filter {
        let filtered = filter_monotonic_approach(current, candidates.clone(), dest_screen_x, dest_screen_y, cell_width, cell_height);

        // If all filtered out and fallback allowed, return best candidate
        if filtered.is_empty() && allow_fallback && !candidates.is_empty() {
            vec![candidates[0]]
        } else {
            filtered
        }
    } else {
        candidates
    }
}

/// Calculate rectangular bounds for free-movement area between current and reserved sub-cells
/// Returns (min_x, min_y, max_x, max_y) in screen coordinates
pub fn calculate_rectangle_bounds(
    current: &SubCellCoord,
    reserved: &SubCellCoord,
    cell_width: f32,
    cell_height: f32,
    offset_x: f32,
    offset_y: f32,
) -> (f32, f32, f32, f32) {
    let (curr_x, curr_y) = current.to_screen_center_with_offset(cell_width, cell_height, offset_x, offset_y);
    let (res_x, res_y) = reserved.to_screen_center_with_offset(cell_width, cell_height, offset_x, offset_y);

    let min_x = curr_x.min(res_x);
    let max_x = curr_x.max(res_x);
    let min_y = curr_y.min(res_y);
    let max_y = curr_y.max(res_y);

    (min_x, min_y, max_x, max_y)
}

/// Check if a position is within the rectangular free-movement area
pub fn is_within_rectangle(
    pos_x: f32,
    pos_y: f32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
) -> bool {
    pos_x >= min_x && pos_x <= max_x && pos_y >= min_y && pos_y <= max_y
}

/// Check if a point is inside a triangle using barycentric coordinates
fn point_in_triangle(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> bool {
    let v0x = cx - ax;
    let v0y = cy - ay;
    let v1x = bx - ax;
    let v1y = by - ay;
    let v2x = px - ax;
    let v2y = py - ay;

    let dot00 = v0x * v0x + v0y * v0y;
    let dot01 = v0x * v1x + v0y * v1y;
    let dot02 = v0x * v2x + v0y * v2y;
    let dot11 = v1x * v1x + v1y * v1y;
    let dot12 = v1x * v2x + v1y * v2y;

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    (u >= 0.0) && (v >= 0.0) && (u + v <= 1.0)
}

/// Calculate the target position within triangle for destination-direct movement
///
/// If destination direction intersects triangle interior: move toward destination
/// If destination direction goes outside triangle: move toward triangle edge closest to destination
///
/// Triangle is formed by: current sub-cell, reserved sub-cell, and anchor sub-cell
///
/// # Parameters
/// - `current`, `reserved`, `anchor`: The 3 vertices of the triangle (screen coordinates)
/// - `actor_x`, `actor_y`: Current actor position
/// - `dest_x`, `dest_y`: Destination position
pub fn calculate_triangle_boundary_target(
    current_x: f32,
    current_y: f32,
    reserved_x: f32,
    reserved_y: f32,
    anchor_x: f32,
    anchor_y: f32,
    actor_x: f32,
    actor_y: f32,
    dest_x: f32,
    dest_y: f32,
) -> (f32, f32) {
    // Calculate direction from actor to destination
    let dir_x = dest_x - actor_x;
    let dir_y = dest_y - actor_y;
    let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();

    if dir_len < 0.0001 {
        // Already at destination, stay in place
        return (actor_x, actor_y);
    }

    let norm_dir_x = dir_x / dir_len;
    let norm_dir_y = dir_y / dir_len;

    // Find the farthest point along the destination direction that's still inside the triangle
    // Use binary search to find intersection with triangle boundary
    let mut t_min = 0.0;
    let mut t_max = dir_len * 2.0; // Search beyond destination

    // Binary search for boundary intersection
    for _ in 0..20 {
        let t_mid = (t_min + t_max) / 2.0;
        let test_x = actor_x + norm_dir_x * t_mid;
        let test_y = actor_y + norm_dir_y * t_mid;

        if point_in_triangle(test_x, test_y, current_x, current_y, reserved_x, reserved_y, anchor_x, anchor_y) {
            t_min = t_mid; // Point is inside, search farther
        } else {
            t_max = t_mid; // Point is outside, search closer
        }
    }

    // Use the boundary point (slightly inside to avoid edge cases)
    let boundary_t = t_min * 0.99;
    let target_x = actor_x + norm_dir_x * boundary_t;
    let target_y = actor_y + norm_dir_y * boundary_t;

    (target_x, target_y)
}

/// Calculate the optimal target position for destination-direct movement
///
/// Returns the position the actor should move toward based on:
/// - Diagonal reservation with anchor: Move toward triangle boundary closest to destination
/// - H/V reservation: Return reserved sub-cell center
/// - No reservation: Return current sub-cell center IF closer to destination, else actor's current position
pub fn calculate_optimal_boundary(
    current_subcell: &SubCellCoord,
    reserved_subcell: Option<&SubCellCoord>,
    anchor_subcell: Option<&SubCellCoord>,
    dest_screen_x: f32,
    dest_screen_y: f32,
    actor_pos_x: f32,
    actor_pos_y: f32,
    cell_width: f32,
    cell_height: f32,
    offset_x: f32,
    offset_y: f32,
) -> (f32, f32) {
    match reserved_subcell {
        Some(reserved) => {
            // Check if this is a diagonal reservation
            let dx_cells = (reserved.cell_x - current_subcell.cell_x).abs();
            let dy_cells = (reserved.cell_y - current_subcell.cell_y).abs();
            let dx_subs = (reserved.sub_x - current_subcell.sub_x).abs();
            let dy_subs = (reserved.sub_y - current_subcell.sub_y).abs();

            let is_diagonal = (dx_cells > 0 || dx_subs > 0) && (dy_cells > 0 || dy_subs > 0);

            if is_diagonal {
                // Diagonal reservation with anchor: Use triangle boundary
                if let Some(anchor) = anchor_subcell {
                    let (curr_x, curr_y) = current_subcell.to_screen_center_with_offset(
                        cell_width, cell_height, offset_x, offset_y
                    );
                    let (res_x, res_y) = reserved.to_screen_center_with_offset(
                        cell_width, cell_height, offset_x, offset_y
                    );
                    let (anc_x, anc_y) = anchor.to_screen_center_with_offset(
                        cell_width, cell_height, offset_x, offset_y
                    );

                    calculate_triangle_boundary_target(
                        curr_x, curr_y,
                        res_x, res_y,
                        anc_x, anc_y,
                        actor_pos_x, actor_pos_y,
                        dest_screen_x, dest_screen_y,
                    )
                } else {
                    // Fallback to rectangle if no anchor (shouldn't happen in DestinationDirect)
                    let (min_x, min_y, max_x, max_y) = calculate_rectangle_bounds(
                        current_subcell,
                        reserved,
                        cell_width,
                        cell_height,
                        offset_x,
                        offset_y,
                    );

                    let clamped_x = dest_screen_x.max(min_x).min(max_x);
                    let clamped_y = dest_screen_y.max(min_y).min(max_y);

                    (clamped_x, clamped_y)
                }
            } else {
                // H/V reservation: Move directly to reserved sub-cell center
                reserved.to_screen_center_with_offset(cell_width, cell_height, offset_x, offset_y)
            }
        }
        None => {
            // No reservation: Move toward destination to trigger reservation attempt
            // Return a point in the destination direction (within current sub-cell)
            let (center_x, center_y) = current_subcell.to_screen_center_with_offset(
                cell_width,
                cell_height,
                offset_x,
                offset_y,
            );

            // Calculate direction toward destination
            let dir_x = dest_screen_x - actor_pos_x;
            let dir_y = dest_screen_y - actor_pos_y;
            let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();

            if dir_len < 0.001 {
                // Already at destination
                (actor_pos_x, actor_pos_y)
            } else {
                // Move a small step toward destination (enough to trigger reservation)
                let step_size = (cell_width / 4.0).min(cell_height / 4.0);
                let norm_dir_x = dir_x / dir_len;
                let norm_dir_y = dir_y / dir_len;

                (actor_pos_x + norm_dir_x * step_size, actor_pos_y + norm_dir_y * step_size)
            }
        }
    }
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
