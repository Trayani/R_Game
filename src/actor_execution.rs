/// Actor Execution Module
///
/// Contains low-level execution logic for actor operations:
/// - Reservation manager interactions
/// - State modifications
/// - Helper functions for reservations
///
/// This module is called by actor_directives.rs (decision root) to execute decisions.

use crate::actor::{Actor, Affinity, AlignmentState};
use crate::subcell::{SubCellCoord, SubCellReservationManager};
use crate::config::ReservationEagerness;

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
pub fn get_horizontal_anchor(psc: &SubCellCoord, diagonal: &SubCellCoord) -> SubCellCoord {
    SubCellCoord {
        cell_x: diagonal.cell_x,
        cell_y: psc.cell_y,
        sub_x: diagonal.sub_x,
        sub_y: psc.sub_y,
        grid_size: psc.grid_size,
    }
}

/// Get vertical anchor for diagonal move
pub fn get_vertical_anchor(psc: &SubCellCoord, diagonal: &SubCellCoord) -> SubCellCoord {
    SubCellCoord {
        cell_x: psc.cell_x,
        cell_y: diagonal.cell_y,
        sub_x: psc.sub_x,
        sub_y: diagonal.sub_y,
        grid_size: psc.grid_size,
    }
}

/// Get opposite anchor (H→V or V→H)
pub fn get_opposite_anchor(
    psc: &SubCellCoord,
    diagonal: &SubCellCoord,
    current_anchor: &SubCellCoord,
) -> SubCellCoord {
    let h_anchor = get_horizontal_anchor(psc, diagonal);
    let v_anchor = get_vertical_anchor(psc, diagonal);

    if current_anchor.cell_x == h_anchor.cell_x
        && current_anchor.cell_y == h_anchor.cell_y
        && current_anchor.sub_x == h_anchor.sub_x
        && current_anchor.sub_y == h_anchor.sub_y
    {
        v_anchor
    } else {
        h_anchor
    }
}

// ============================================================================
// DIAGONAL/ANCHOR CHECKS
// ============================================================================

/// Check if move from current to target is diagonal
pub fn is_diagonal_move(current: &SubCellCoord, target: &SubCellCoord) -> bool {
    let dx = (target.cell_x + target.sub_x) - (current.cell_x + current.sub_x);
    let dy = (target.cell_y + target.sub_y) - (current.cell_y + current.sub_y);
    dx != 0 && dy != 0
}

/// Find anchor cell for diagonal move (H or V)
pub fn find_anchor_cell(current: &SubCellCoord, target: &SubCellCoord) -> Option<SubCellCoord> {
    if !is_diagonal_move(current, target) {
        return None;
    }

    // Return horizontal anchor by default
    Some(SubCellCoord {
        cell_x: target.cell_x,
        cell_y: current.cell_y,
        sub_x: target.sub_x,
        sub_y: current.sub_y,
        grid_size: current.grid_size,
    })
}

// ============================================================================
// ANTI-CROSS CHECK
// ============================================================================

/// Check if diagonal move violates anti-cross rule
/// Returns true if move should be blocked
pub fn check_anti_cross(
    from: &SubCellCoord,
    to: &SubCellCoord,
    reservation_manager: &SubCellReservationManager,
    self_id: usize,
) -> bool {
    // Only applies to diagonal moves
    if !is_diagonal_move(from, to) {
        return false;
    }

    // Get the two counter-diagonal cells
    let counter1 = SubCellCoord {
        cell_x: from.cell_x,
        cell_y: to.cell_y,
        sub_x: from.sub_x,
        sub_y: to.sub_y,
        grid_size: from.grid_size,
    };
    let counter2 = SubCellCoord {
        cell_x: to.cell_x,
        cell_y: from.cell_y,
        sub_x: to.sub_x,
        sub_y: from.sub_y,
        grid_size: from.grid_size,
    };

    // Check if SAME actor owns BOTH counter-diagonal cells
    let owner1 = reservation_manager.get_owner(&counter1);
    let owner2 = reservation_manager.get_owner(&counter2);

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
    subcell: &SubCellCoord,
    dest_x: f32,
    dest_y: f32,
    cell_width: f32,
    cell_height: f32,
    offset_x: f32,
    offset_y: f32,
) -> f32 {
    let center = subcell.to_screen_center_with_offset(cell_width, cell_height, offset_x, offset_y);
    let dx = center.0 - dest_x;
    let dy = center.1 - dest_y;
    (dx * dx + dy * dy).sqrt()
}
