/// Actor Directives - Simple if-else decision logic for actor state machine
///
/// This file contains all decision-making logic for actors.
/// Keep this simple with clear if-else statements that are easy to understand and modify.
///
/// YOU (the user) will maintain this file directly.

use crate::actor::{Actor, AlignmentState};
use crate::subcell::{SubCellCoord, SubCellReservationManager};
use crate::config::{ReservationEagerness, ReleaseEagerness};
use crate::pathfinding::Position;

/// Calculate Euclidean distance between two points
fn distance(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()
}

/// Check if destination is cardinally aligned with current subcell
///
/// Returns Some(is_horizontal) if destination is cardinally aligned:
/// - Some(false) for vertical alignment (same X, different Y) → NORTH/SOUTH
/// - Some(true) for horizontal alignment (same Y, different X) → EAST/WEST
///
/// Returns None if destination requires diagonal movement (different X AND Y)
/// or if already at destination (same X AND Y)
fn check_cardinal_alignment(
    current: &SubCellCoord,
    dest: &Position,
) -> Option<bool> {
    let same_x = dest.x == current.cell_x;
    let same_y = dest.y == current.cell_y;

    if same_x && !same_y {
        Some(false) // Vertical alignment (NORTH/SOUTH)
    } else if same_y && !same_x {
        Some(true)  // Horizontal alignment (EAST/WEST)
    } else {
        None        // Diagonal movement or at destination
    }
}

// ============================================================================
// STATE TRANSITION DECISIONS
// ============================================================================

/// Should actor reserve its initial subcell?
/// Called when actor has no current_subcell (NoSubcell state)
pub fn should_reserve_initial_subcell(
    _actor_id: usize,
    _actor_pos: (f32, f32),
    _has_destination: bool,
) -> bool {
    // Always try to reserve initial subcell
    // Actor needs a subcell to align to before it can navigate
    true
}

/// Should actor transition from PscAlignment to Idle?
/// Called when actor is aligning to subcell center
pub fn should_transition_to_idle(
    _actor_id: usize,
    distance_to_center: f32,
    alignment_threshold: f32,
) -> bool {
    // Simple rule: transition when close enough to center
    if distance_to_center < alignment_threshold {
        return true;
    }
    false
}

/// Should actor attempt to reserve next subcell?
/// Called when actor is in Idle state at subcell center
pub fn should_attempt_next_reservation(
    _actor_id: usize,
    has_destination: bool,
) -> bool {
    // Only try to reserve next subcell if we have a destination
    if has_destination {
        return true;
    }
    false
}

// ============================================================================
// NAVIGATION DECISIONS
// ============================================================================

/// Should actor try diagonal movement first?
/// Called when actor is ready to reserve next subcell
pub fn should_try_diagonal_first(
    _actor_id: usize,
    dx_to_dest: f32,
    dy_to_dest: f32,
) -> bool {
    // Try diagonal if moving in both X and Y directions
    let moving_x = dx_to_dest.abs() > 5.0;
    let moving_y = dy_to_dest.abs() > 5.0;

    if moving_x && moving_y {
        return true;
    }
    false
}

/// Should actor fallback to horizontal/vertical movement?
/// Called when diagonal reservation fails
pub fn should_fallback_to_hv(
    _actor_id: usize,
) -> bool {
    // Always try H/V fallback if diagonal fails
    true
}

/// Should actor wait when all reservations fail?
/// Called when neither diagonal nor H/V can be reserved
pub fn should_wait_when_blocked(
    _actor_id: usize,
) -> bool {
    // Wait in place if no moves available
    true
}

// ============================================================================
// PSC SWITCHING DECISIONS (CRITICAL - Controls backwards movement)
// ============================================================================

/// Should actor switch from current subcell to reserved subcell?
///
/// This is the CRITICAL decision that controls backwards movement!
///
/// Parameters:
/// - actor_pos: Actor's current (x, y) position
/// - current_center: Center of current subcell
/// - reserved_center: Center of reserved subcell
/// - destination: Destination (x, y) position
/// - has_anchor: Whether this is a diagonal move with anchor subcell
///
/// Returns: true if actor should switch to reserved subcell
pub fn should_switch_to_reserved(
    _actor_id: usize,
    actor_pos: (f32, f32),
    current_center: (f32, f32),
    reserved_center: (f32, f32),
    destination: (f32, f32),
    _has_anchor: bool,
) -> bool {
    // Calculate distances FROM ACTOR'S ACTUAL POSITION (not subcell centers!)
    let actor_to_dest = distance(actor_pos, destination);
    let reserved_center_to_dest = distance(reserved_center, destination);
    let current_center_to_dest = distance(current_center, destination);

    // Calculate how far actor would need to move to reach each subcell center
    let dist_to_current_center = distance(actor_pos, current_center);
    let dist_to_reserved_center = distance(actor_pos, reserved_center);

    // RULE 1: Don't switch if reserved is much farther from destination
    if reserved_center_to_dest > current_center_to_dest + 20.0 {
        return false; // Reserved is significantly farther from destination
    }

    // RULE 2: Don't switch if it requires moving backwards (away from destination)
    // Check if moving to reserved center would increase distance to destination
    let would_move_backwards = dist_to_reserved_center > dist_to_current_center * 1.5;
    if would_move_backwards {
        return false; // Would require moving too far backwards
    }

    // RULE 3: Switch if reserved is closer to destination AND actor is already close to boundary
    let near_boundary = dist_to_current_center > 5.0; // Actor is moving away from current center
    let reserved_is_closer = reserved_center_to_dest < current_center_to_dest - 10.0;

    if near_boundary && reserved_is_closer {
        return true; // Good time to switch - actor is leaving current subcell anyway
    }

    // DEFAULT: Stay at current subcell
    false
}

/// Should actor switch to anchor subcell?
/// Called during diagonal movement when actor has both reserved and anchor subcells
pub fn should_switch_to_anchor(
    _actor_id: usize,
    actor_pos: (f32, f32),
    current_center: (f32, f32),
    anchor_center: (f32, f32),
    destination: (f32, f32),
) -> bool {
    // Similar logic to should_switch_to_reserved but for anchor subcell

    let anchor_center_to_dest = distance(anchor_center, destination);
    let current_center_to_dest = distance(current_center, destination);
    let dist_to_anchor_center = distance(actor_pos, anchor_center);
    let dist_to_current_center = distance(actor_pos, current_center);

    // Don't switch if anchor is farther from destination
    if anchor_center_to_dest > current_center_to_dest + 20.0 {
        return false;
    }

    // Don't switch if it requires significant backwards movement
    if dist_to_anchor_center > dist_to_current_center * 1.5 {
        return false;
    }

    // Switch if anchor is closer and actor is near boundary
    let near_boundary = dist_to_current_center > 5.0;
    let anchor_is_closer = anchor_center_to_dest < current_center_to_dest - 10.0;

    if near_boundary && anchor_is_closer {
        return true;
    }

    false
}

/// Should actor stay at current subcell despite having reservation?
/// Called as final check before switching PSC
pub fn should_stay_at_current(
    _actor_id: usize,
    actor_pos: (f32, f32),
    current_center: (f32, f32),
    _destination: (f32, f32),
) -> bool {
    // Stay if actor is still very close to current subcell center
    let dist_to_center = distance(actor_pos, current_center);

    if dist_to_center < 3.0 {
        return true; // Very close to center, no need to switch yet
    }

    false
}

// ============================================================================
// UTILITY FUNCTIONS (for common calculations)
// ============================================================================

/// Check if actor is moving in the direction of destination
/// Returns true if movement vector aligns with destination direction
pub fn is_moving_toward_destination(
    actor_pos: (f32, f32),
    next_pos: (f32, f32),
    destination: (f32, f32),
) -> bool {
    // Calculate vectors
    let movement_x = next_pos.0 - actor_pos.0;
    let movement_y = next_pos.1 - actor_pos.1;
    let dest_x = destination.0 - actor_pos.0;
    let dest_y = destination.1 - actor_pos.1;

    // Dot product - positive means moving toward destination
    let dot_product = movement_x * dest_x + movement_y * dest_y;

    dot_product > 0.0
}

// ============================================================================
// STATE HANDLERS
// ============================================================================

/// Handle NoSubcell state - actor needs to acquire initial subcell
fn handle_no_subcell_state(
    actor: &mut Actor,
    delta_time: f32,
    reservation_manager: &mut SubCellReservationManager,
    enable_early_reservation: bool,
    enable_anti_cross: bool,
    track_movement: bool,
    reservation_eagerness: ReservationEagerness,
    release_eagerness: ReleaseEagerness,
) -> bool {
    // Delegate to full implementation for now
    // TODO: Extract NoSubcell-specific logic here
    actor.update_subcell_destination_direct_impl(
        delta_time,
        reservation_manager,
        enable_early_reservation,
        false,
        enable_anti_cross,
        track_movement
    )
}

/// Handle PscAlignment state - actor moving to subcell center
fn handle_psc_alignment_state(
    actor: &mut Actor,
    delta_time: f32,
    reservation_manager: &mut SubCellReservationManager,
    enable_early_reservation: bool,
    enable_anti_cross: bool,
    track_movement: bool,
    reservation_eagerness: ReservationEagerness,
    release_eagerness: ReleaseEagerness,
) -> bool {
    // Delegate to full implementation for now
    // TODO: Extract PscAlignment-specific logic here
    actor.update_subcell_destination_direct_impl(
        delta_time,
        reservation_manager,
        enable_early_reservation,
        false,
        enable_anti_cross,
        track_movement
    )
}

/// Try both reservation strategies with specified priority order
/// Returns true if either strategy succeeded
fn try_reservation_with_fallback(
    actor: &mut Actor,
    current: &SubCellCoord,
    dx_to_dest: f32,
    dy_to_dest: f32,
    dest_screen_x: f32,
    dest_screen_y: f32,
    reservation_manager: &mut SubCellReservationManager,
    enable_anti_cross: bool,
    track_movement: bool,
    try_cardinal_first: bool,
) -> bool {
    if try_cardinal_first {
        // Try H/V first, then diagonal fallback
        actor.try_reserve_horizontal_vertical(
            current,
            dx_to_dest,
            dy_to_dest,
            dest_screen_x,
            dest_screen_y,
            reservation_manager,
            track_movement,
        ) || actor.try_reserve_diagonal_with_anchor(
            current,
            None,
            dx_to_dest,
            dy_to_dest,
            dest_screen_x,
            dest_screen_y,
            reservation_manager,
            enable_anti_cross,
            track_movement,
        )
    } else {
        // Try diagonal first, then H/V fallback
        actor.try_reserve_diagonal_with_anchor(
            current,
            None,
            dx_to_dest,
            dy_to_dest,
            dest_screen_x,
            dest_screen_y,
            reservation_manager,
            enable_anti_cross,
            track_movement,
        ) || actor.try_reserve_horizontal_vertical(
            current,
            dx_to_dest,
            dy_to_dest,
            dest_screen_x,
            dest_screen_y,
            reservation_manager,
            track_movement,
        )
    }
}

/// Handle Idle state - actor at subcell center, ready to move
fn handle_idle_state(
    actor: &mut Actor,
    reservation_manager: &mut SubCellReservationManager,
    enable_anti_cross: bool,
    track_movement: bool,
) -> bool {
    // Idle state: Actor is at subcell center, ready to move
    // If destination is NOT defined, do nothing
    // If destination exists, calculate direction and try to reserve next subcell
    // If reservation successful, transition to Move state

    // 1. Check if destination is defined
    let dest = match actor.subcell_destination {
        Some(d) => d,
        None => {
            // No destination - stay in Idle, do nothing
            return false;
        }
    };

    // 2. Get current subcell (must exist in Idle state)
    let current = actor.current_subcell
        .expect("Actor in Idle state must have current_subcell");

    // 3. Calculate direction to destination
    let dest_screen_x = dest.x as f32 * actor.cell_width + actor.cell_width / 2.0;
    let dest_screen_y = dest.y as f32 * actor.cell_height + actor.cell_height / 2.0;
    let dx_to_dest = dest_screen_x - actor.fpos_x;
    let dy_to_dest = dest_screen_y - actor.fpos_y;

    // 4. Try reservation with appropriate priority based on cardinal alignment
    // Per actor_directing_v2.txt: Spec A1 (cardinal) vs A2 (diagonal)
    let is_cardinal = check_cardinal_alignment(&current, &dest).is_some();

    let success = try_reservation_with_fallback(
        actor,
        &current,
        dx_to_dest,
        dy_to_dest,
        dest_screen_x,
        dest_screen_y,
        reservation_manager,
        enable_anti_cross,
        track_movement,
        is_cardinal, // try_cardinal_first
    );

    // 5. Transition to Move state if any reservation succeeded
    if success {
        actor.alignment_state = crate::actor::AlignmentState::Move;
    }
    // else: both failed, stay in Idle (blocked)

    false // Not at destination yet
}

// ============================================================================
// UPDATE ACTOR - DECISION ROOT
// ============================================================================

/// Main update function - serves as the decision root for actor behavior
///
/// This function orchestrates all actor decisions by dispatching to state handlers.
/// All decision logic flows through this function, making it easy to understand and modify.
///
/// Returns: true if actor reached destination, false otherwise
pub fn update_actor(
    actor: &mut Actor,
    delta_time: f32,
    reservation_manager: &mut SubCellReservationManager,
    enable_early_reservation: bool,
    enable_anti_cross: bool,
    track_movement: bool,
    reservation_eagerness: ReservationEagerness,
    release_eagerness: ReleaseEagerness,
) -> bool {
    // Dispatch to appropriate state handler based on actor's current state
    match actor.alignment_state {
        AlignmentState::NoSubcell => handle_no_subcell_state(
            actor,
            delta_time,
            reservation_manager,
            enable_early_reservation,
            enable_anti_cross,
            track_movement,
            reservation_eagerness,
            release_eagerness,
        ),
        AlignmentState::PscAlignment => handle_psc_alignment_state(
            actor,
            delta_time,
            reservation_manager,
            enable_early_reservation,
            enable_anti_cross,
            track_movement,
            reservation_eagerness,
            release_eagerness,
        ),
        AlignmentState::Idle => handle_idle_state(
            actor,
            reservation_manager,
            enable_anti_cross,
            track_movement,
        ),
        AlignmentState::Move => {


            // Delegate to full implementation for now
            // TODO: Extract Move-specific logic here
            actor.update_subcell_destination_direct_impl(
                delta_time,
                reservation_manager,
                enable_early_reservation,
                false,
                enable_anti_cross,
                track_movement
            )
        }
    }
}
