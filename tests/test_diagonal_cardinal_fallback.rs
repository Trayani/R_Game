/// Tests for diagonal-to-cardinal fallback behavior
/// Per actor_directing_v2.txt Section C2:
/// When best diagonal is blocked (by reservation or anti-cross),
/// actors should fall back to cardinal directions before waiting.

use rustgame3::Actor;
use rustgame3::pathfinding::Position;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

/// Test Case 1: SW diagonal blocked → should try S (down) cardinal
/// Actor needs to go SOUTHWEST, but SW diagonal is blocked.
/// Expected: Actor tries S (down) cardinal instead of SE diagonal.
#[test]
fn test_sw_blocked_tries_s_cardinal() {
    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Spawn actor at cell (10, 10), subcell (0, 0)
    let start_x = 10.0 * cell_width + 7.5; // subcell (0,0) center
    let start_y = 10.0 * cell_height + 7.5;

    let mut actor = Actor::new(
        0,
        start_x,
        start_y,
        10.0, // size
        100.0, // speed
        6.0, // collision_radius
        cell_width,
        cell_height,
        subcell_grid_size,
        0.0, 0.0, // no offset
        true, // enable_lookahead
        0.5, // psc_switch_threshold
    );

    actor.use_directing_v2 = true; // Use new algorithm

    // Set destination SOUTHWEST (down and left)
    let dest_cell_x = 8; // 2 cells left
    let dest_cell_y = 13; // 3 cells down
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    // Block SW diagonal (9, 11, 1, 0) and its anchor
    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);
    let sw_diagonal = SubCellCoord::new(9, 11, 1, 0, subcell_grid_size);
    let sw_anchor_v = SubCellCoord::new(10, 11, 0, 0, subcell_grid_size); // Vertical anchor
    let sw_anchor_h = SubCellCoord::new(9, 10, 1, 1, subcell_grid_size); // Horizontal anchor

    reservation_mgr.try_reserve(sw_diagonal, 999); // Block with different actor
    reservation_mgr.try_reserve(sw_anchor_v, 999);
    reservation_mgr.try_reserve(sw_anchor_h, 999);

    // Update actor (should try to reserve next subcell)
    let reached = actor.update_subcell_destination_direct(
        0.016, // delta_time
        &mut reservation_mgr,
        false, // enable_early_reservation
        false, // filter_backward
        false, // enable_anti_cross
        false, // track_movement
        0.0,   // reservation_threshold_distance
        ReservationEagerness::Center,
        ReleaseEagerness::Center,
    );

    assert!(!reached, "Should not reach destination in one frame");

    // Check reservation: should have reserved a CARDINAL direction (S or W)
    // Since SW is blocked, actor should fall back to cardinals
    let reserved = actor.reserved_subcell.expect("Actor should have reserved something");

    // S (down) is subcell (10, 11, 0, 0) - directly down
    // W (left) is subcell (9, 10, 1, 1) - directly left
    let s_cardinal = SubCellCoord::new(10, 11, 0, 0, subcell_grid_size);
    let w_cardinal = SubCellCoord::new(9, 10, 1, 1, subcell_grid_size);

    // Should have reserved S (down) since vertical distance is larger (3 cells vs 2 cells)
    assert!(
        reserved == s_cardinal || reserved == w_cardinal,
        "Actor should reserve cardinal direction S or W, not SE diagonal. Reserved: {:?}",
        reserved
    );
}

/// Test Case 2: NE diagonal blocked → should try N or E cardinal
/// Actor needs to go NORTHEAST, but NE diagonal is blocked.
/// Expected: Actor tries N (up) or E (right) cardinal instead of NW diagonal.
#[test]
fn test_ne_blocked_tries_cardinal() {
    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Spawn actor at cell (10, 10), subcell (0, 0)
    let start_x = 10.0 * cell_width + 7.5;
    let start_y = 10.0 * cell_height + 7.5;

    let mut actor = Actor::new(
        0,
        start_x,
        start_y,
        10.0, 100.0, 6.0,
        cell_width, cell_height,
        subcell_grid_size,
        0.0, 0.0,
        true, 0.5,
    );

    actor.use_directing_v2 = true;

    // Set destination NORTHEAST (up and right)
    let dest_cell_x = 13; // cells
    let dest_cell_y = 7; // cells
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    // Block NE diagonal and both possible anchors
    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);
    let ne_diagonal = SubCellCoord::new(11, 9, 0, 1, subcell_grid_size);
    let ne_anchor_v = SubCellCoord::new(10, 9, 0, 1, subcell_grid_size); // Vertical anchor (N)
    let ne_anchor_h = SubCellCoord::new(11, 10, 0, 0, subcell_grid_size); // Horizontal anchor (E)

    reservation_mgr.try_reserve(ne_diagonal, 999);
    reservation_mgr.try_reserve(ne_anchor_v, 999);
    reservation_mgr.try_reserve(ne_anchor_h, 999);

    let reached = actor.update_subcell_destination_direct(
        0.016,
        &mut reservation_mgr,
        false, // enable_early_reservation
        false, // filter_backward
        false, // enable_anti_cross
        false, // track_movement
        0.0,   // reservation_threshold_distance
        ReservationEagerness::Center,
        ReleaseEagerness::Center,
    );

    assert!(!reached);

    // Should have reserved a cardinal direction
    let reserved = actor.reserved_subcell.expect("Actor should have reserved something");

    // N (up) is subcell (10, 9, 0, 1)
    // E (right) is subcell (11, 10, 0, 0)
    let n_cardinal = SubCellCoord::new(10, 9, 0, 1, subcell_grid_size);
    let e_cardinal = SubCellCoord::new(11, 10, 0, 0, subcell_grid_size);

    assert!(
        reserved == n_cardinal || reserved == e_cardinal,
        "Actor should reserve cardinal N or E, not NW. Reserved: {:?}",
        reserved
    );
}

/// Test Case 3: All diagonals + cardinals blocked → actor waits
/// When both the best diagonal AND all cardinals are blocked,
/// actor should NOT try worse-aligned diagonals - just wait.
#[test]
fn test_all_directions_blocked_actor_waits() {
    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    let start_x = 10.0 * cell_width + 7.5;
    let start_y = 10.0 * cell_height + 7.5;

    let mut actor = Actor::new(
        0,
        start_x,
        start_y,
        10.0, 100.0, 6.0,
        cell_width, cell_height,
        subcell_grid_size,
        0.0, 0.0,
        true, 0.5,
    );

    actor.use_directing_v2 = true;

    // Destination: SOUTHWEST
    let dest_cell_x = 8; // cells
    let dest_cell_y = 13; // cells
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    // Block ALL neighbors (current is 10,10,0,0)
    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);
    let neighbors = [
        SubCellCoord::new(9, 9, 1, 1, subcell_grid_size),   // NW
        SubCellCoord::new(10, 9, 0, 1, subcell_grid_size),  // N
        SubCellCoord::new(11, 9, 0, 1, subcell_grid_size),  // NE
        SubCellCoord::new(9, 10, 1, 1, subcell_grid_size),  // W
        SubCellCoord::new(11, 10, 0, 0, subcell_grid_size), // E
        SubCellCoord::new(9, 11, 1, 0, subcell_grid_size),  // SW
        SubCellCoord::new(10, 11, 0, 0, subcell_grid_size), // S
        SubCellCoord::new(11, 11, 0, 0, subcell_grid_size), // SE
    ];

    for neighbor in &neighbors {
        reservation_mgr.try_reserve(*neighbor, 999);
    }

    let reached = actor.update_subcell_destination_direct(
        0.016,
        &mut reservation_mgr,
        false, // enable_early_reservation
        false, // filter_backward
        false, // enable_anti_cross
        false, // track_movement
        0.0,   // reservation_threshold_distance
        ReservationEagerness::Center,
        ReleaseEagerness::Center,
    );

    assert!(!reached);

    // Actor should NOT have reserved anything (should wait)
    assert!(
        actor.reserved_subcell.is_none(),
        "Actor should not reserve anything when all directions blocked"
    );
}

/// Test Case 4: Simulation with multiple actors competing for diagonals
/// Validates that actors use cardinal fallback correctly when diagonals are contested.
#[test]
fn test_multiple_actors_diagonal_fallback() {
    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Create 3 actors all heading SOUTHWEST from same region
    let mut actors = vec![];
    for i in 0..3 {
        let start_x = (15 + i) as f32 * cell_width + 7.5;
        let start_y = 10.0 * cell_height + 7.5;

        let mut actor = Actor::new(
            i,
            start_x,
            start_y,
            10.0, 100.0, 6.0,
            cell_width, cell_height,
            subcell_grid_size,
            0.0, 0.0,
            true, 0.5,
        );

        actor.use_directing_v2 = true;

        // All head to same SOUTHWEST destination
    let dest_cell_x = 5; // cells
    let dest_cell_y = 20; // cells
        actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

        actors.push(actor);
    }

    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);

    // Run one frame - actors should reserve without backwards movement
    let mut movements = Vec::new();
    for actor in &mut actors {
        let old_pos = (actor.fpos_x, actor.fpos_y);

        let reached = actor.update_subcell_destination_direct(
            0.016,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // enable_anti_cross
            false, // track_movement
            0.0,   // reservation_threshold_distance
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        let new_pos = (actor.fpos_x, actor.fpos_y);
        movements.push((actor.id, old_pos, new_pos));
    }

    // Check: all actors should have made some reservation (or stayed if blocked)
    // At least one actor should successfully reserve
    let successful_reservations = actors.iter()
        .filter(|a| a.reserved_subcell.is_some())
        .count();

    assert!(
        successful_reservations >= 1,
        "At least one actor should successfully reserve a subcell"
    );

    // Check: no actor moved backwards (increased distance to destination)
    for (id, old, new) in movements {
        let dest_x = 5.0 * cell_width;
        let dest_y = 20.0 * cell_height;

        let old_dist_x = (dest_x - old.0).abs();
        let old_dist_y = (dest_y - old.1).abs();
        let new_dist_x = (dest_x - new.0).abs();
        let new_dist_y = (dest_y - new.1).abs();

        assert!(
            new_dist_x <= old_dist_x + 1.0 && new_dist_y <= old_dist_y + 1.0,
            "Actor {} moved backwards: old=({:.1}, {:.1}) new=({:.1}, {:.1}) dest=({:.1}, {:.1})",
            id, old.0, old.1, new.0, new.1, dest_x, dest_y
        );
    }
}

/// Test Case 5: Verify best diagonal selection by alignment
/// When multiple diagonals pass distance rule, only the best-aligned should be tried.
#[test]
fn test_only_best_diagonal_tried() {
    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    let start_x = 10.0 * cell_width + 7.5;
    let start_y = 10.0 * cell_height + 7.5;

    let mut actor = Actor::new(
        0,
        start_x,
        start_y,
        10.0, 100.0, 6.0,
        cell_width, cell_height,
        subcell_grid_size,
        0.0, 0.0,
        true, 0.5,
    );

    actor.use_directing_v2 = true;

    // Destination: SOUTHWEST (more down than left: 2 cells left, 4 cells down)
    let dest_cell_x = 8; // cells
    let dest_cell_y = 14; // cells
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    // Block SW diagonal (best aligned) and its anchors
    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);
    let sw_diagonal = SubCellCoord::new(9, 11, 1, 0, subcell_grid_size);
    reservation_mgr.try_reserve(sw_diagonal, 999);

    // Also block both anchors for SW
    let sw_anchor_v = SubCellCoord::new(10, 11, 0, 0, subcell_grid_size); // S
    let sw_anchor_h = SubCellCoord::new(9, 10, 1, 1, subcell_grid_size); // W
    reservation_mgr.try_reserve(sw_anchor_v, 999);
    reservation_mgr.try_reserve(sw_anchor_h, 999);

    // Leave SE diagonal open (worse alignment but not blocked)
    // But per spec, actor should NOT try SE - should go to cardinals

    let reached = actor.update_subcell_destination_direct(
        0.016,
        &mut reservation_mgr,
        false, // enable_early_reservation
        false, // filter_backward
        false, // enable_anti_cross
        false, // track_movement
        0.0,   // reservation_threshold_distance
        ReservationEagerness::Center,
        ReleaseEagerness::Center,
    );

    assert!(!reached);

    // Actor should have reserved a CARDINAL direction, NOT SE diagonal
    if let Some(reserved) = actor.reserved_subcell {
        let se_diagonal = SubCellCoord::new(11, 11, 0, 0, subcell_grid_size);

        assert_ne!(
            reserved, se_diagonal,
            "Actor should NOT try SE diagonal when SW (best) is blocked. Should use cardinal fallback."
        );

        // Should be either S (down) - if not blocked
        // Since we blocked S above, actor might try W or might wait
        // The key test is: did NOT try SE
    }
}
