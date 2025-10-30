/// Test to reproduce Actor 3's backwards movement from action_log.db
/// Actor 3 at (9,13,1,1) reserved SE diagonal (10,14,0,0) which increased X distance
/// This should have been filtered by the distance rule but wasn't.

use rustgame3::Actor;
use rustgame3::pathfinding::Position;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_actor3_se_diagonal_should_be_filtered() {
    // Configuration from action_log.db
    let cell_width = 30.0;
    let cell_height = 20.0;  // Note: height is 20, not 30!
    let subcell_grid_size = 2;

    // Actor 3 at subcell (9,13,1,1)
    // BR corner at (277.5, 265.0) per log
    let start_x = 277.5;
    let start_y = 265.0;

    let mut actor = Actor::new(
        3,
        start_x,
        start_y,
        10.0, // size
        100.0, // speed
        6.0, // collision_radius
        cell_width,
        cell_height,
        subcell_grid_size,
        0.5, 0.5, // offset
        true, // enable_lookahead
        0.5, // psc_switch_threshold
    );

    actor.use_directing_v2 = true;

    // Destination: (270.0, 750.0) - SOUTHWEST (left and down)
    let dest_cell_x = 9;  // 270 / 30 = 9
    let dest_cell_y = 37; // 750 / 20 = 37.5, round to 37
    actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });

    // Don't block anything - let the distance filter do its job
    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);

    // Update actor
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

    assert!(!reached, "Should not reach destination in one frame");

    // Check what was reserved
    if let Some(reserved) = actor.reserved_subcell {
        let se_diagonal = SubCellCoord::new(10, 14, 0, 0, subcell_grid_size);

        // Calculate if SE would increase distance
        let curr = SubCellCoord::new(9, 13, 1, 1, subcell_grid_size);
        let (curr_x, curr_y) = curr.to_screen_center_with_offset(
            cell_width, cell_height, 0.5, 0.5
        );
        let (se_x, se_y) = se_diagonal.to_screen_center_with_offset(
            cell_width, cell_height, 0.5, 0.5
        );

        let dest_x = dest_cell_x as f32 * cell_width;
        let dest_y = dest_cell_y as f32 * cell_height;

        let curr_dist_x = (dest_x - curr_x).abs();
        let new_dist_x = (dest_x - se_x).abs();
        let x_change = new_dist_x - curr_dist_x;

        println!("\n=== Actor 3 Scenario Analysis ===");
        println!("Current: (9,13,1,1) at ({:.1},{:.1})", curr_x, curr_y);
        println!("SE diagonal: (10,14,0,0) at ({:.1},{:.1})", se_x, se_y);
        println!("Destination: ({:.1},{:.1})", dest_x, dest_y);
        println!("X distance change: {:.1} → {:.1} = {:+.1}", curr_dist_x, new_dist_x, x_change);
        println!("Reserved: {:?}", reserved);
        println!();

        assert_ne!(
            reserved, se_diagonal,
            "SE diagonal should NOT be reserved - it increases X distance by {:.1}px",
            x_change
        );
    } else {
        panic!("Actor should have reserved something");
    }
}
