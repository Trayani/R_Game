// Test user's affinity issue: spawn (136, 525) -> dest (16, 31)
// User reports NE-H was optimal but actor traveled as if NE-V was selected

use rustgame3::Actor;
use rustgame3::SubPointReservationManager;
use rustgame3::subcell::{SubCellCoord, };
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_affinity_spawn_136_525_dest_16_31() {
    println!("\n=== Testing Affinity Issue ===");

    let cell_width = 32.0;
    let cell_height = 32.0;

    let spawn_x = 136.0;
    let spawn_y = 525.0;
    let dest_cell_x = 16;
    let dest_cell_y = 31;

    let spawn_cell_x = (spawn_x / cell_width) as i32;
    let spawn_cell_y = (spawn_y / cell_height) as i32;

    println!("Spawn: ({:.1}, {:.1}) in cell ({}, {})", spawn_x, spawn_y, spawn_cell_x, spawn_cell_y);
    println!("Destination: cell ({}, {})", dest_cell_x, dest_cell_y);

    let dx = dest_cell_x - spawn_cell_x;
    let dy = dest_cell_y - spawn_cell_y;

    println!("Direction vector: dx={} dy={}", dx, dy);
    println!("Expected affinity: {} (abs_dx={} abs_dy={})\n",
        if dx.abs() > dy.abs() { "HORIZONTAL" } else { "VERTICAL" },
        dx.abs(), dy.abs());

    // Create actor with directing v2
    let mut actor = Actor::new(
        0, spawn_x, spawn_y,
        16.0, 64.0, 8.0,
        cell_width, cell_height,
        2, 0.0, 0.0,
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    let start_subcell = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(start_subcell.to_subpoint());

    let dest = Position { x: dest_cell_x, y: dest_cell_y };
    actor.set_subcell_destination(dest);

    let mut reservation_mgr = SubPointReservationManager::new(2, 1000, 1000);
    reservation_mgr.set_current(start_subcell.to_subpoint(), 0);

    // Run just a few iterations to see initial affinity choices
    let delta_time = 0.016;
    let max_iterations = 100;

    println!("Running {} iterations to observe affinity choices...\n", max_iterations);

    for iteration in 0..max_iterations {
        let _reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // enable_anti_cross
            iteration < 20, // track_movement for first 20 frames
            0.0,
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        // Log position every 20 frames
        if iteration > 0 && iteration % 20 == 0 {
            let dest_x = dest_cell_x as f32 * cell_width;
            let dest_y = dest_cell_y as f32 * cell_height;
            let curr_dx = dest_x - actor.fpos_x;
            let curr_dy = dest_y - actor.fpos_y;
            let angle = curr_dy.atan2(curr_dx).to_degrees();

            println!("[Iter {}] Pos: ({:.1}, {:.1}) | Direction to dest: dx={:.1} dy={:.1} angle={:.1}°",
                iteration, actor.fpos_x, actor.fpos_y, curr_dx, curr_dy, angle);
        }
    }

    println!("\n=== Analysis Complete ===");
    println!("Check the [RESERVE V2] logs above to see which affinity was chosen");
    println!("and compare with the expected affinity based on direction vector\n");
}
