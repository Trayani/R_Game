// Test exact user scenario from action log
// Spawn at (122.0, 359.0), destination cell (11, 39)

use rustgame3::Actor;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_user_scenario_spawn_122_359_dest_11_39() {
    println!("\n=== Testing User Scenario ===");
    println!("Spawn: (122.0, 359.0)");
    println!("Destination: cell (11, 39)\n");

    let cell_width = 32.0;
    let cell_height = 32.0;

    let spawn_x = 122.0;
    let spawn_y = 359.0;
    let dest_cell_x = 11;
    let dest_cell_y = 39;

    // Calculate which cell the actor spawned in
    let spawn_cell_x = (spawn_x / cell_width) as i32;
    let spawn_cell_y = (spawn_y / cell_height) as i32;

    println!("Spawn cell: ({}, {})", spawn_cell_x, spawn_cell_y);
    println!("Destination cell: ({}, {})", dest_cell_x, dest_cell_y);

    let dx = dest_cell_x - spawn_cell_x;
    let dy = dest_cell_y - spawn_cell_y;
    let distance = ((dx * dx + dy * dy) as f32).sqrt();

    println!("Distance: {:.1} cells ({} X, {} Y)", distance, dx.abs(), dy.abs());
    println!("Direction: {}diagonal ({}X, {}Y)\n",
        if dx != 0 && dy != 0 { "" } else { "axis-aligned " },
        if dx > 0 { "+" } else if dx < 0 { "-" } else { "" },
        if dy > 0 { "+" } else if dy < 0 { "-" } else { "" });

    // Create actor with v2 directing enabled
    let mut actor = Actor::new(
        0,
        spawn_x,
        spawn_y,
        16.0,
        64.0,   // speed
        8.0,    // collision_radius
        cell_width,
        cell_height,
        2,      // subcell_grid_size
        0.0,    // offset_x
        0.0,    // offset_y
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

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 4000;
    let mut iteration = 0;
    let mut reached = false;
    let mut last_report = 0;

    println!("Starting simulation...\n");

    while iteration < max_iterations && !reached {
        let old_x = actor.fpos_x;
        let old_y = actor.fpos_y;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation (default, should work now)
            false, // filter_backward
            false, // enable_anti_cross
            false, // track_movement
            0.0,
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        let moved = (actor.fpos_x - old_x).abs() > 0.001 || (actor.fpos_y - old_y).abs() > 0.001;

        // Report every second or when stopped
        if iteration % 60 == 0 || (!moved && iteration - last_report > 10) {
            let dest_x = dest_cell_x as f32 * cell_width;
            let dest_y = dest_cell_y as f32 * cell_height;
            let dist = ((actor.fpos_x - dest_x).powi(2) + (actor.fpos_y - dest_y).powi(2)).sqrt();

            println!("[{:4} iter / {:.1}s] Pos: ({:.1}, {:.1}) | Dist: {:.1}px | {}",
                iteration,
                iteration as f32 * delta_time,
                actor.fpos_x,
                actor.fpos_y,
                dist,
                if moved { "MOVING" } else { "STOPPED" });

            if let Some(ref sc) = actor.current_subcell {
                println!("                   Subcell: ({},{},{},{}) | Reservations: {}",
                    sc.cell_x, sc.cell_y, sc.sub_x, sc.sub_y,
                    reservation_mgr.reservation_count());
            }

            last_report = iteration;

            // Log if stopped but don't exit yet (let it try to recover)
            if !moved && iteration == 283 {
                println!("\n!!! ACTOR STOPPED at iter {} ({:.1}s) !!!", iteration, iteration as f32 * delta_time);
                println!("Position: ({:.1}, {:.1})", actor.fpos_x, actor.fpos_y);
                println!("Distance remaining: {:.1}px ({:.1} cells)", dist, dist / cell_width);
                println!("Continuing to see if it recovers...\n");
            }

            // Don't abort - let it run to completion to see if it reaches destination
        }

        if reached {
            println!("\n✓ Actor REACHED destination after {} iterations ({:.1}s)",
                iteration, iteration as f32 * delta_time);
            break;
        }
    }

    if !reached && iteration >= max_iterations {
        println!("\n✗ Actor DID NOT reach destination after {} iterations", max_iterations);
    }

    println!("\n=== Test Complete ===\n");

    // Test should pass if actor reaches destination
    assert!(reached, "Actor should reach destination");
}
