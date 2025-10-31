// Test file to expose PSC (Present Sub-Cell) state inconsistency bug
//
// Bug Description:
// The actor's current_subcell (PSC) can become stale during movement, not reflecting
// the actor's actual float position. This causes incorrect boundary rectangle calculations
// in DestinationDirect mode, leading to targets being placed at diagonal subcells
// instead of within the actor's current 4-subcell boundary.

use rustgame3::Actor;
use rustgame3::SubPointReservationManager;
use rustgame3::subcell::{SubCellCoord, };
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Calculate the boundary rectangle for a 4-subcell area containing actor
///
/// For a 2x2 subcell grid, an actor in subcell (cell_x, cell_y, sub_x, sub_y)
/// occupies a rectangle spanning that subcell and adjacent subcells.
///
/// Returns: (min_x, max_x, min_y, max_y)
fn calculate_boundary_from_psc(
    psc: &SubCellCoord,
    cell_width: f32,
    cell_height: f32,
) -> (f32, f32, f32, f32) {
    let subcell_width = cell_width / psc.grid_size as f32;
    let subcell_height = cell_height / psc.grid_size as f32;

    // Calculate subcell center position
    let subcell_x = psc.cell_x as f32 * cell_width + psc.sub_x as f32 * subcell_width;
    let subcell_y = psc.cell_y as f32 * cell_height + psc.sub_y as f32 * subcell_height;

    // The 4-subcell boundary extends one subcell width/height in all directions
    let min_x = subcell_x - subcell_width;
    let max_x = subcell_x + subcell_width;
    let min_y = subcell_y - subcell_height;
    let max_y = subcell_y + subcell_height;

    (min_x, max_x, min_y, max_y)
}

/// Check if a point is within a rectangle (inclusive)
fn is_point_in_rect(
    point: (f32, f32),
    rect: (f32, f32, f32, f32), // (min_x, max_x, min_y, max_y)
) -> bool {
    let (px, py) = point;
    let (min_x, max_x, min_y, max_y) = rect;
    px >= min_x && px <= max_x && py >= min_y && py <= max_y
}

/// Check if a point is at a diagonal subcell center
/// (i.e., at a subcell grid intersection, not within the current 4-subcell area)
fn is_at_diagonal_subcell(
    point: (f32, f32),
    actor_pos: (f32, f32),
    cell_width: f32,
    cell_height: f32,
    grid_size: i32,
) -> bool {
    let subcell_width = cell_width / grid_size as f32;
    let subcell_height = cell_height / grid_size as f32;
    let epsilon = 0.01;

    // Check if point is at a subcell grid intersection
    let x_mod = point.0 % subcell_width;
    let y_mod = point.1 % subcell_height;
    let at_grid_intersection =
        (x_mod < epsilon || x_mod > subcell_width - epsilon) &&
        (y_mod < epsilon || y_mod > subcell_height - epsilon);

    if !at_grid_intersection {
        return false;
    }

    // Check if it's a diagonal move from actor position
    let dx = (point.0 - actor_pos.0).abs();
    let dy = (point.1 - actor_pos.1).abs();

    // Diagonal if both X and Y change significantly
    dx > subcell_width * 0.5 && dy > subcell_height * 0.5
}

/// Print comprehensive diagnostic report for PSC state analysis
#[allow(clippy::too_many_arguments)]
fn print_diagnostic_report(
    test_name: &str,
    actor_pos: (f32, f32),
    stored_psc: &Option<SubCellCoord>,
    actual_psc: &SubCellCoord,
    destination: (i32, i32),
    locked_target: Option<(f32, f32)>,
    cell_width: f32,
    cell_height: f32,
) {
    println!("\n{:=<70}", "");
    println!("Test: {}", test_name);
    println!("{:=<70}\n", "");

    println!("Actor Position: ({:.1}, {:.1})", actor_pos.0, actor_pos.1);

    // Compare stored vs actual PSC
    if let Some(stored) = stored_psc {
        println!("Stored PSC: cell({}, {}), sub({}, {})",
            stored.cell_x, stored.cell_y, stored.sub_x, stored.sub_y);
        println!("Actual PSC: cell({}, {}), sub({}, {})",
            actual_psc.cell_x, actual_psc.cell_y, actual_psc.sub_x, actual_psc.sub_y);

        let psc_mismatch = stored.cell_x != actual_psc.cell_x
            || stored.cell_y != actual_psc.cell_y
            || stored.sub_x != actual_psc.sub_x
            || stored.sub_y != actual_psc.sub_y;

        if psc_mismatch {
            println!("❗ PSC MISMATCH DETECTED!");
        } else {
            println!("✓ PSC matches actual position");
        }

        // Calculate boundary rectangles
        let stored_boundary = calculate_boundary_from_psc(stored, cell_width, cell_height);
        let actual_boundary = calculate_boundary_from_psc(actual_psc, cell_width, cell_height);

        println!("\nStored PSC Boundary: [{:.1}, {:.1}] x [{:.1}, {:.1}]",
            stored_boundary.0, stored_boundary.1, stored_boundary.2, stored_boundary.3);
        println!("Actual PSC Boundary: [{:.1}, {:.1}] x [{:.1}, {:.1}]",
            actual_boundary.0, actual_boundary.1, actual_boundary.2, actual_boundary.3);

        if stored_boundary != actual_boundary {
            println!("❗ BOUNDARY RECTANGLES DIFFER!");
        }

        // Check actor position within boundaries
        let actor_in_stored = is_point_in_rect(actor_pos, stored_boundary);
        let actor_in_actual = is_point_in_rect(actor_pos, actual_boundary);

        println!("\nActor in Stored Boundary: {}", if actor_in_stored { "✓" } else { "✗" });
        println!("Actor in Actual Boundary: {}", if actor_in_actual { "✓" } else { "✗" });

        if !actor_in_stored {
            println!("❌ BUG: Actor position OUTSIDE stored PSC boundary!");
            let dx_stored = if actor_pos.0 < stored_boundary.0 {
                stored_boundary.0 - actor_pos.0
            } else if actor_pos.0 > stored_boundary.1 {
                actor_pos.0 - stored_boundary.1
            } else {
                0.0
            };
            let dy_stored = if actor_pos.1 < stored_boundary.2 {
                stored_boundary.2 - actor_pos.1
            } else if actor_pos.1 > stored_boundary.3 {
                actor_pos.1 - stored_boundary.3
            } else {
                0.0
            };
            println!("   Distance outside: X={:.2}, Y={:.2}", dx_stored, dy_stored);
        }

        // Analyze locked target
        if let Some(target) = locked_target {
            println!("\nLocked Target: ({:.1}, {:.1})", target.0, target.1);

            let target_in_stored = is_point_in_rect(target, stored_boundary);
            let target_in_actual = is_point_in_rect(target, actual_boundary);

            println!("Target in Stored Boundary: {}", if target_in_stored { "✓" } else { "✗" });
            println!("Target in Actual Boundary: {}", if target_in_actual { "✓" } else { "✗" });

            let is_diagonal = is_at_diagonal_subcell(
                target, actor_pos, cell_width, cell_height, actual_psc.grid_size
            );

            println!("\nDirection Classification:");
            if target_in_actual {
                println!("  DIRECT - Target within actor's boundary rectangle");
            } else if is_diagonal {
                println!("  NOT DIRECT - Target at diagonal subcell position");
            } else {
                println!("  UNEXPECTED - Target outside boundary but not diagonal");
            }

            println!("\nExpected Behavior (DestinationDirect mode):");
            println!("  Target should be within actual boundary rectangle");
            println!("  for fluid movement toward destination");

            if !target_in_actual {
                println!("\n❌ BUG EXPOSED: Target placed incorrectly!");
                println!("   This causes non-direct movement when direct movement is expected");

                if psc_mismatch {
                    println!("   Root cause: Stale PSC caused wrong boundary calculation");
                }
            } else {
                println!("\n✓ Target placement correct for this scenario");
            }
        } else {
            println!("\nNo Locked Target: Actor may not have reserved a diagonal cell");
        }
    } else {
        println!("Stored PSC: None");
        println!("Actual PSC: cell({}, {}), sub({}, {})",
            actual_psc.cell_x, actual_psc.cell_y, actual_psc.sub_x, actual_psc.sub_y);
    }

    println!("\nDestination: cell({}, {})", destination.0, destination.1);
    let dest_world_x = destination.0 as f32 * cell_width;
    let dest_world_y = destination.1 as f32 * cell_height;
    println!("Destination World: ({:.1}, {:.1})", dest_world_x, dest_world_y);

    println!("\n{:=<70}\n", "");
}

// ============================================================================
// TEST CASES
// ============================================================================

/// Test the original bug case: actor at (175.0, 477.0) with destination (19, 28)
#[test]
fn test_original_bug_case_175_477_to_19_28() {
    let cell_width = 32.0;
    let cell_height = 32.0;
    let spawn_x = 175.0;
    let spawn_y = 477.0;
    let dest_cell_x = 19;
    let dest_cell_y = 28;

    // Create actor
    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0,  // size
        64.0,  // speed (pixels/second)
        8.0,   // collision_radius
        cell_width, cell_height,
        2,     // subcell_grid_size (2x2)
        0.0, 0.0,  // subcell offsets
        true,  // enable_lookahead
        0.5,   // psc_switch_threshold
    );
    actor.use_directing_v2 = true;

    // Calculate actual PSC from spawn position
    let actual_psc = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    // Set initial current_subcell (this will be the "stored" PSC)
    actor.current_subcell = Some(actual_psc.to_subpoint());

    // Set destination
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    // Setup reservation manager
    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(actual_psc.clone().to_subpoint(), 0);

    // Run one update to trigger reservation and affinity calculation
    actor.update_subcell_destination_direct(
        0.016,  // delta_time (~60 FPS)
        &mut reservation_mgr,
        false,  // enable_early_reservation
        false,  // filter_backward
            false, // enable_anti_cross
        false,  // track_movement
        0.0,    // reservation_threshold_distance
        ReservationEagerness::Center,
        ReleaseEagerness::Center,
    );

    // Calculate current actual PSC (after any movement)
    let current_actual_psc = SubCellCoord::from_screen_pos_with_offset(
        actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    // Print diagnostic report
    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    print_diagnostic_report(
        "original_bug_case_175_477_to_19_28",
        (actor.fpos_x, actor.fpos_y),
        &stored_psc_coord,
        &current_actual_psc,
        (dest_cell_x, dest_cell_y),
        actor.locked_target,
        cell_width,
        cell_height,
    );
}

/// Test actor at subcell boundary - edge case where PSC calculation is critical
#[test]
fn test_actor_at_subcell_boundary() {
    let cell_width = 32.0;
    let cell_height = 32.0;

    // Spawn actor exactly at a subcell boundary
    let spawn_x = 160.0;  // Exactly at subcell x=0 boundary (cell 5, sub 0)
    let spawn_y = 464.0;  // Exactly at subcell y=0 boundary (cell 14, sub 0)
    let dest_cell_x = 20;
    let dest_cell_y = 30;

    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0, 64.0, 8.0,
        cell_width, cell_height,
        2, 0.0, 0.0,
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    let actual_psc = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(actual_psc.to_subpoint());
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(actual_psc.clone().to_subpoint(), 0);

    actor.update_subcell_destination_direct(
        0.016, &mut reservation_mgr,
        false, false, false, // enable_early_reservation, filter_backward, track_movement
            false, // enable_anti_cross
        0.0, ReservationEagerness::Center, ReleaseEagerness::Center,
    );

    let current_actual_psc = SubCellCoord::from_screen_pos_with_offset(
        actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    print_diagnostic_report(
        "actor_at_subcell_boundary",
        (actor.fpos_x, actor.fpos_y),
        &stored_psc_coord,
        &current_actual_psc,
        (dest_cell_x, dest_cell_y),
        actor.locked_target,
        cell_width,
        cell_height,
    );
}

/// Test diagonal movement NE direction
#[test]
fn test_diagonal_movement_northeast() {
    let cell_width = 32.0;
    let cell_height = 32.0;
    let spawn_x = 100.0;
    let spawn_y = 100.0;
    let dest_cell_x = 15;  // Northeast destination
    let dest_cell_y = 5;

    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0, 64.0, 8.0,
        cell_width, cell_height,
        2, 0.0, 0.0,
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    let actual_psc = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(actual_psc.to_subpoint());
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(actual_psc.clone().to_subpoint(), 0);

    actor.update_subcell_destination_direct(
        0.016, &mut reservation_mgr,
        false, false, false, // enable_early_reservation, filter_backward, track_movement
            false, // enable_anti_cross
        0.0, ReservationEagerness::Center, ReleaseEagerness::Center,
    );

    let current_actual_psc = SubCellCoord::from_screen_pos_with_offset(
        actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    print_diagnostic_report(
        "diagonal_movement_northeast",
        (actor.fpos_x, actor.fpos_y),
        &stored_psc_coord,
        &current_actual_psc,
        (dest_cell_x, dest_cell_y),
        actor.locked_target,
        cell_width,
        cell_height,
    );
}

/// Test diagonal movement SW direction
#[test]
fn test_diagonal_movement_southwest() {
    let cell_width = 32.0;
    let cell_height = 32.0;
    let spawn_x = 300.0;
    let spawn_y = 200.0;
    let dest_cell_x = 3;  // Southwest destination
    let dest_cell_y = 12;

    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0, 64.0, 8.0,
        cell_width, cell_height,
        2, 0.0, 0.0,
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    let actual_psc = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(actual_psc.to_subpoint());
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(actual_psc.clone().to_subpoint(), 0);

    actor.update_subcell_destination_direct(
        0.016, &mut reservation_mgr,
        false, false, false, // enable_early_reservation, filter_backward, track_movement
            false, // enable_anti_cross
        0.0, ReservationEagerness::Center, ReleaseEagerness::Center,
    );

    let current_actual_psc = SubCellCoord::from_screen_pos_with_offset(
        actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    print_diagnostic_report(
        "diagonal_movement_southwest",
        (actor.fpos_x, actor.fpos_y),
        &stored_psc_coord,
        &current_actual_psc,
        (dest_cell_x, dest_cell_y),
        actor.locked_target,
        cell_width,
        cell_height,
    );
}

/// Test with multiple update iterations to expose PSC lag over time
#[test]
fn test_multiple_updates_psc_lag() {
    let cell_width = 32.0;
    let cell_height = 32.0;
    let spawn_x = 175.0;
    let spawn_y = 477.0;
    let dest_cell_x = 19;
    let dest_cell_y = 28;

    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0, 64.0, 8.0,
        cell_width, cell_height,
        2, 0.0, 0.0,
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    let initial_psc = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(initial_psc.to_subpoint());
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(initial_psc.clone().to_subpoint(), 0);

    println!("\n{:=<70}", "");
    println!("Test: multiple_updates_psc_lag (5 iterations)");
    println!("{:=<70}\n", "");

    // Run multiple updates to see PSC lag develop over time
    for iteration in 0..5 {
        println!("--- Iteration {} ---", iteration + 1);
        println!("Position before: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);

        let psc_before = actor.current_subcell.clone();
        let actual_psc_before = SubCellCoord::from_screen_pos_with_offset(
            actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
        );

        actor.update_subcell_destination_direct(
            0.016, &mut reservation_mgr,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            false, // enable_anti_cross
            0.0, ReservationEagerness::Center, ReleaseEagerness::Center,
        );

        println!("Position after:  ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);

        let psc_after = actor.current_subcell.clone();
        let actual_psc_after = SubCellCoord::from_screen_pos_with_offset(
            actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
        );

        if let (Some(stored), actual) = (&psc_after, &actual_psc_after) {
            let (stored_cx, stored_cy) = stored.to_cell(2);
            let (stored_sx, stored_sy) = stored.subcell_offset(2);
            let mismatch = stored_cx != actual.cell_x
                || stored_cy != actual.cell_y
                || stored_sx != actual.sub_x
                || stored_sy != actual.sub_y;

            if mismatch {
                println!("❗ PSC Mismatch: Stored=({},{},{},{}), Actual=({},{},{},{})",
                    stored_cx, stored_cy, stored_sx, stored_sy,
                    actual.cell_x, actual.cell_y, actual.sub_x, actual.sub_y);
            } else {
                println!("✓ PSC consistent");
            }
        }

        if let Some(target) = actor.locked_target {
            println!("Target: ({:.2}, {:.2})", target.0, target.1);
        }

        println!();
    }

    // Final diagnostic report
    let final_actual_psc = SubCellCoord::from_screen_pos_with_offset(
        actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    print_diagnostic_report(
        "multiple_updates_psc_lag_final_state",
        (actor.fpos_x, actor.fpos_y),
        &stored_psc_coord,
        &final_actual_psc,
        (dest_cell_x, dest_cell_y),
        actor.locked_target,
        cell_width,
        cell_height,
    );
}

/// Test with high speed to amplify PSC lag effect
#[test]
fn test_high_speed_amplified_psc_lag() {
    let cell_width = 32.0;
    let cell_height = 32.0;
    let spawn_x = 175.0;
    let spawn_y = 477.0;
    let dest_cell_x = 25;
    let dest_cell_y = 25;

    // High speed = larger movement per frame = more PSC lag
    let high_speed = 320.0;  // 5x normal speed

    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0, high_speed, 8.0,
        cell_width, cell_height,
        2, 0.0, 0.0,
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    let actual_psc = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(actual_psc.to_subpoint());
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(actual_psc.clone().to_subpoint(), 0);

    actor.update_subcell_destination_direct(
        0.016, &mut reservation_mgr,
        false, false, false, // enable_early_reservation, filter_backward, track_movement
            false, // enable_anti_cross
        0.0, ReservationEagerness::Center, ReleaseEagerness::Center,
    );

    let current_actual_psc = SubCellCoord::from_screen_pos_with_offset(
        actor.fpos_x, actor.fpos_y, cell_width, cell_height, 2, 0.0, 0.0
    );

    let stored_psc_coord = actor.current_subcell.map(|sp| SubCellCoord::from_subpoint(&sp, 2));
    print_diagnostic_report(
        "high_speed_amplified_psc_lag",
        (actor.fpos_x, actor.fpos_y),
        &stored_psc_coord,
        &current_actual_psc,
        (dest_cell_x, dest_cell_y),
        actor.locked_target,
        cell_width,
        cell_height,
    );
}
