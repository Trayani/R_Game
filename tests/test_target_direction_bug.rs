// Test to reproduce target direction bug
// Spawn at (208, 473), destination cell (20, 27)
// Expected: move RIGHT and DOWN
// Actual: target points UP (wrong!)

use rustgame3::Actor;
use rustgame3::SubPointReservationManager;
use rustgame3::subcell::{SubCellCoord, };
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_target_direction_spawn_208_473_dest_20_27() {
    println!("\n=== Testing Target Direction Bug ===\n");

    let cell_width = 32.0;
    let cell_height = 32.0;

    let spawn_x = 208.0;
    let spawn_y = 473.0;
    let dest_cell_x = 20;
    let dest_cell_y = 27;

    println!("Spawn: ({:.1}, {:.1})", spawn_x, spawn_y);
    println!("Destination: cell ({}, {}) = screen ({:.1}, {:.1})",
        dest_cell_x, dest_cell_y,
        dest_cell_x as f32 * cell_width,
        dest_cell_y as f32 * cell_height);

    let expected_dx = dest_cell_x as f32 * cell_width - spawn_x;
    let expected_dy = dest_cell_y as f32 * cell_height - spawn_y;
    println!("Expected direction: dx={:.1} dy={:.1} ({} and {})",
        expected_dx, expected_dy,
        if expected_dx > 0.0 { "RIGHT" } else { "LEFT" },
        if expected_dy > 0.0 { "DOWN" } else { "UP" });

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

    let mut reservation_mgr = SubPointReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Run just a few iterations to see first directing decision
    let delta_time = 0.016;
    let max_iterations = 5;

    println!("\nRunning {} iterations...\n", max_iterations);

    for iteration in 0..max_iterations {
        let _reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // enable_anti_cross
            true,  // track_movement = true to see logs
            0.0,
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        if iteration == 0 {
            // Check first directing decision
            if let Some(ref directing) = actor.last_directing_info {
                println!("\n=== FIRST DIRECTING DECISION ===");
                println!("Affinity: {:?}", directing.affinity);
                println!("Target: ({:.1}, {:.1})", directing.target_x, directing.target_y);
                
                let target_dx = directing.target_x - spawn_x;
                let target_dy = directing.target_y - spawn_y;
                println!("Target direction: dx={:.1} dy={:.1} ({} and {})",
                    target_dx, target_dy,
                    if target_dx > 0.0 { "RIGHT" } else { "LEFT" },
                    if target_dy > 0.0 { "DOWN" } else { "UP" });

                // Check if directions match
                let x_correct = (expected_dx > 0.0) == (target_dx > 0.0);
                let y_correct = (expected_dy > 0.0) == (target_dy > 0.0);
                
                println!("\nX direction: {}", if x_correct { "✓ CORRECT" } else { "✗ WRONG" });
                println!("Y direction: {}", if y_correct { "✓ CORRECT" } else { "✗ WRONG" });

                if !x_correct || !y_correct {
                    panic!("Target direction is WRONG! Expected ({} and {}), got ({} and {})",
                        if expected_dx > 0.0 { "RIGHT" } else { "LEFT" },
                        if expected_dy > 0.0 { "DOWN" } else { "UP" },
                        if target_dx > 0.0 { "RIGHT" } else { "LEFT" },
                        if target_dy > 0.0 { "DOWN" } else { "UP" });
                }
            }
            break;
        }
    }

    println!("\n=== Test Complete ===\n");
}
