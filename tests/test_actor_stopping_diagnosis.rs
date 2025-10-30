// Diagnose why actor stops moving during diagonal movement
// Simulates user scenario: spawn actor, set diagonal destination, observe stopping

use rustgame3::Actor;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_diagnose_actor_stopping() {
    println!("\n=== Diagnosing Actor Stopping Issue ===\n");

    // Recreate user's scenario from action_log
    // Actor spawned at (310.0, 203.0)
    // Destination set to cell (17, 26)

    let cell_width = 32.0;  // Typical GUI cell size
    let cell_height = 32.0;

    let spawn_x = 310.0;
    let spawn_y = 203.0;

    // Calculate which cell the actor spawned in
    let spawn_cell_x = (spawn_x / cell_width) as i32;
    let spawn_cell_y = (spawn_y / cell_height) as i32;

    println!("Spawn position: ({:.1}, {:.1})", spawn_x, spawn_y);
    println!("Spawn cell: ({}, {})", spawn_cell_x, spawn_cell_y);

    let dest_cell_x = 17;
    let dest_cell_y = 26;

    println!("Destination cell: ({}, {})", dest_cell_x, dest_cell_y);

    // Calculate distance
    let dx = dest_cell_x - spawn_cell_x;
    let dy = dest_cell_y - spawn_cell_y;
    let distance = ((dx * dx + dy * dy) as f32).sqrt();

    println!("Distance: {:.1} cells ({} cells X, {} cells Y)", distance, dx.abs(), dy.abs());
    println!("Direction: {}diagonal ({}X, {}Y)\n",
        if dx != 0 && dy != 0 { "" } else { "axis-aligned " },
        if dx > 0 { "+" } else if dx < 0 { "-" } else { "" },
        if dy > 0 { "+" } else if dy < 0 { "-" } else { "" });

    // Create actor
    let mut actor = Actor::new(
        0,                  // id
        spawn_x,            // fpos_x
        spawn_y,            // fpos_y
        16.0,               // size (typical)
        64.0,               // speed (2 cells/second at 32px cells)
        8.0,                // collision_radius
        cell_width,
        cell_height,
        2,                  // subcell_grid_size (2x2)
        0.0,                // subcell_offset_x
        0.0,                // subcell_offset_y
        false, 0.0,
    );
    actor.use_directing_v2 = true;

    // Set starting subcell
    let start_subcell = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(start_subcell.clone());

    println!("Starting subcell: ({},{},{},{})\n",
        start_subcell.cell_x, start_subcell.cell_y,
        start_subcell.sub_x, start_subcell.sub_y);

    // Set destination
    let dest = Position { x: dest_cell_x, y: dest_cell_y };
    actor.set_subcell_destination(dest);

    let mut reservation_mgr = SubCellReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Simulation parameters
    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 2000;  // Long enough to reach destination

    // Track movement
    let mut iteration = 0;
    let mut reached = false;
    let mut last_report_iteration = 0;
    let mut distance_to_dest = distance * cell_width;

    println!("Starting simulation...\n");

    while iteration < max_iterations && !reached {
        let old_x = actor.fpos_x;
        let old_y = actor.fpos_y;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // enable_anti_cross
            iteration >= 140 && iteration <= 160, // track_movement - only log around failure
            0.0,   // reservation_threshold_distance
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        // Check if actor moved
        let moved = (actor.fpos_x - old_x).abs() > 0.001 || (actor.fpos_y - old_y).abs() > 0.001;

        // Calculate distance to destination
        let dest_x = dest_cell_x as f32 * cell_width;
        let dest_y = dest_cell_y as f32 * cell_height;
        let new_distance = ((actor.fpos_x - dest_x).powi(2) + (actor.fpos_y - dest_y).powi(2)).sqrt();

        // Report every 60 frames (1 second) or when stopped
        if iteration % 60 == 0 || (!moved && iteration - last_report_iteration > 10) {
            let time_sec = iteration as f32 * delta_time;
            let travel_distance = distance_to_dest - new_distance;

            println!("[{:5} iter / {:.1}s] Pos: ({:.1}, {:.1}) | Dist to dest: {:.1}px | Travel: {:.1}px | {}",
                iteration, time_sec,
                actor.fpos_x, actor.fpos_y,
                new_distance,
                travel_distance,
                if moved { "MOVING" } else { "STOPPED" });

            if let Some(ref subcell) = actor.current_subcell {
                println!("                    Subcell: ({},{},{},{}) | Reservations: {}",
                    subcell.cell_x, subcell.cell_y, subcell.sub_x, subcell.sub_y,
                    reservation_mgr.reservation_count());
            }

            last_report_iteration = iteration;

            // If stopped for too long, warn but continue for debugging
            if !moved && iteration == 145 {  // Log once at iteration 145
                println!("\n!!! ACTOR STOPPED MOVING after {:.1}s !!!", time_sec);
                println!("Current position: ({:.1}, {:.1})", actor.fpos_x, actor.fpos_y);
                println!("Distance remaining: {:.1}px ({:.1} cells)",
                    new_distance, new_distance / cell_width);
                println!("Progress: {:.1}%\n", (travel_distance / distance_to_dest) * 100.0);
            }
        }

        distance_to_dest = distance_to_dest.min(new_distance);

        if reached {
            println!("\n✓ Actor reached destination after {} iterations ({:.1}s)",
                iteration, iteration as f32 * delta_time);
            break;
        }
    }

    if !reached && iteration >= max_iterations {
        println!("\n✗ Actor did NOT reach destination after {} iterations", max_iterations);
    }

    println!("\n=== Diagnosis Complete ===\n");
}
