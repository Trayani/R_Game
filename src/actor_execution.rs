/// Actor Execution Module
///
/// Contains low-level execution logic for actor operations:
/// - Reservation manager interactions
/// - State modifications
/// - Helper functions for reservations
///
/// This module is called by actor_directives.rs (decision root) to execute decisions.

use crate::subcell::SubCellReservationManager;
use crate::subpoint::SubPoint;

// ============================================================================
// SPIRAL SEARCH HELPERS
// ============================================================================

/// Generate spiral pattern offsets for subcell search
/// Returns (dx, dy) offsets in spiral order from center
pub fn generate_spiral_offsets(radius: i32) -> Vec<(i32, i32)> {
    let mut offsets = vec![(0, 0)]; // Start at center

    for r in 1..=radius {
        // Start from (r, 0) and walk around the square
        // Right edge going up
        for dy in 0..r {
            offsets.push((r, dy));
        }
        // Top edge going left
        for dx in ((-r+1)..=r).rev() {
            offsets.push((dx, r));
        }
        // Left edge going down
        for dy in ((-r+1)..r).rev() {
            offsets.push((-r, dy));
        }
        // Bottom edge going right
        for dx in (-r)..r {
            offsets.push((dx, -r));
        }
    }

    offsets
}

// ============================================================================
// ANCHOR CELL HELPERS
// ============================================================================

/// Get horizontal anchor for diagonal move
pub fn get_horizontal_anchor(psc: &SubPoint, diagonal: &SubPoint) -> SubPoint {
    SubPoint {
        x: diagonal.x,
        y: psc.y,
    }
}

/// Get vertical anchor for diagonal move
pub fn get_vertical_anchor(psc: &SubPoint, diagonal: &SubPoint) -> SubPoint {
    SubPoint {
        x: psc.x,
        y: diagonal.y,
    }
}

/// Get opposite anchor (H→V or V→H)
pub fn get_opposite_anchor(
    psc: &SubPoint,
    diagonal: &SubPoint,
    current_anchor: &SubPoint,
) -> SubPoint {
    let h_anchor = get_horizontal_anchor(psc, diagonal);
    let v_anchor = get_vertical_anchor(psc, diagonal);

    // SubPoint implements PartialEq, so we can compare directly
    if current_anchor == &h_anchor {
        v_anchor
    } else {
        h_anchor
    }
}

// ============================================================================
// DIAGONAL/ANCHOR CHECKS
// ============================================================================

/// Check if move from current to target is diagonal
pub fn is_diagonal_move(current: &SubPoint, target: &SubPoint) -> bool {
    let dx = target.x - current.x;
    let dy = target.y - current.y;
    dx != 0 && dy != 0
}

/// Find anchor cell for diagonal move (H or V)
pub fn find_anchor_cell(current: &SubPoint, target: &SubPoint) -> Option<SubPoint> {
    if !is_diagonal_move(current, target) {
        return None;
    }

    // Return horizontal anchor by default
    Some(SubPoint {
        x: target.x,
        y: current.y,
    })
}

// ============================================================================
// ANTI-CROSS CHECK
// ============================================================================

/// Check if diagonal move violates anti-cross rule
/// Returns true if move should be blocked
pub fn check_anti_cross(
    from: &SubPoint,
    to: &SubPoint,
    reservation_manager: &SubCellReservationManager,
    self_id: usize,
    grid_size: i32,
) -> bool {
    // Only applies to diagonal moves
    if !is_diagonal_move(from, to) {
        return false;
    }

    // Get the two counter-diagonal cells
    let counter1 = SubPoint {
        x: from.x,
        y: to.y,
    };
    let counter2 = SubPoint {
        x: to.x,
        y: from.y,
    };

    // Convert to SubCellCoord for reservation manager lookup
    let counter1_coord = crate::subcell::SubCellCoord::from_subpoint(&counter1, grid_size);
    let counter2_coord = crate::subcell::SubCellCoord::from_subpoint(&counter2, grid_size);

    // Check if SAME actor owns BOTH counter-diagonal cells
    let owner1 = reservation_manager.get_owner(&counter1_coord);
    let owner2 = reservation_manager.get_owner(&counter2_coord);

    match (owner1, owner2) {
        (Some(id1), Some(id2)) if id1 == id2 && id1 != self_id => {
            // Same other actor owns both counter-diagonals: BLOCK
            true
        }
        _ => false,
    }
}

// ============================================================================
// DISTANCE CALCULATIONS
// ============================================================================

/// Calculate distance from subcell center to destination
pub fn subcell_center_distance_to_destination(
    subcell: &SubPoint,
    dest_x: f32,
    dest_y: f32,
    cell_width: f32,
    cell_height: f32,
    grid_size: i32,
    offset_x: f32,
    offset_y: f32,
) -> f32 {
    let center = subcell.to_screen_center_with_offset(cell_width, cell_height, grid_size, offset_x, offset_y);
    let dx = center.0 - dest_x;
    let dy = center.1 - dest_y;
    (dx * dx + dy * dy).sqrt()
}
