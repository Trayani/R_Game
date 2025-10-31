/// Test that PSC_ALIGNMENT happens automatically after spawn WITHOUT setting destination
///
/// Per actor_states.txt Section 3.1:
/// - When actor spawns, it should automatically align to nearest subcell center
/// - This should happen EVEN if no destination is set
/// - Actor should transition: NoSubcell → PscAlignment → Idle
/// - Actor should stay idle at subcell center until destination is set

use rustgame3::Actor;
use rustgame3::SubPointReservationManager;
use rustgame3::subcell::;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_automatic_alignment_without_destination() {
    println!("\n========================================");
    println!("  Automatic PSC_ALIGNMENT Test");
    println!("  (No Destination Required)");
    println!("========================================\n");

    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Spawn actor at arbitrary position (NOT at subcell center)
    let start_x = 157.3;
    let start_y = 207.8;

    let mut actor = Actor::new(
        0,
        start_x,
        start_y,
        10.0,   // size
        100.0,  // speed
        6.0,    // collision_radius
        cell_width,
        cell_height,
        subcell_grid_size,
        0.0, 0.0, // no offset
        true,   // enable_lookahead
        0.5,    // psc_switch_threshold
    );

    actor.use_directing_v2 = true;

    println!("Spawned Actor {} at ({:.1}, {:.1})", actor.id, start_x, start_y);
    println!("NO DESTINATION SET - actor should align automatically\n");

    let mut reservation_mgr = SubPointReservationManager::new(subcell_grid_size, 1000, 1000);

    // Run simulation WITHOUT setting destination
    let max_iterations = 200;
    let delta_time = 0.016;    // ~60 FPS

    let mut aligned = false;
    let mut alignment_frame = None;

    println!("Running simulation for up to {} iterations...\n", max_iterations);

    for iteration in 0..max_iterations {
        let old_pos = (actor.fpos_x, actor.fpos_y);

        // Update WITHOUT destination
        let _reached = actor.update_subcell_destination_direct(
            delta_time,
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

        // Check alignment to current subcell center
        if let Some(current) = actor.current_subcell {
            let (center_x, center_y) = current.to_screen_center(cell_width, cell_height, subcell_grid_size);
            let dist = ((new_pos.0 - center_x).powi(2) + (new_pos.1 - center_y).powi(2)).sqrt();

            // Check if aligned (distance < 2.0 pixels)
            if !aligned && dist < 2.0 {
                aligned = true;
                alignment_frame = Some(iteration);
                let (cell_x, cell_y) = current.to_cell(subcell_grid_size);
                let (sub_x, sub_y) = current.subcell_offset(subcell_grid_size);
                println!("[{:4}ms] Actor {} ALIGNED to subcell ({},{},{},{}) at ({:.1},{:.1}) - dist={:.3}px",
                    iteration * 16, actor.id,
                    cell_x, cell_y, sub_x, sub_y,
                    new_pos.0, new_pos.1, dist);
                println!("[{:4}ms] Actor should now be IDLE at subcell center", iteration * 16);
            }

            // Log initial movements to show alignment in action
            if iteration < 20 && !aligned {
                let moved = (new_pos.0 - old_pos.0).abs() > 0.01 || (new_pos.1 - old_pos.1).abs() > 0.01;
                if moved {
                    let move_dist = ((new_pos.0 - old_pos.0).powi(2) + (new_pos.1 - old_pos.1).powi(2)).sqrt();
                    println!("[{:4}ms] PSC_ALIGNMENT: ({:.1},{:.1}) → ({:.1},{:.1}) [move={:.2}px, dist_to_center={:.2}px]",
                        iteration * 16,
                        old_pos.0, old_pos.1, new_pos.0, new_pos.1,
                        move_dist, dist);
                }
            }
        }

        // Stop if aligned and actor has stopped moving
        if aligned {
            let moved = (new_pos.0 - old_pos.0).abs() > 0.01 || (new_pos.1 - old_pos.1).abs() > 0.01;
            if !moved {
                println!("\n[{:4}ms] Actor aligned and stable at subcell center", iteration * 16);
                break;
            }
        }
    }

    println!("\n========================================");
    println!("  ALIGNMENT RESULT");
    println!("========================================");

    if let Some(current) = actor.current_subcell {
        let (center_x, center_y) = current.to_screen_center(cell_width, cell_height, subcell_grid_size);
        let dist = ((actor.fpos_x - center_x).powi(2) + (actor.fpos_y - center_y).powi(2)).sqrt();
        let aligned_ms = alignment_frame.unwrap_or(0) * 16;

        let (cell_x, cell_y) = current.to_cell(subcell_grid_size);
        let (sub_x, sub_y) = current.subcell_offset(subcell_grid_size);
        println!("Current Subcell: ({},{},{},{})", cell_x, cell_y, sub_x, sub_y);
        println!("Position: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);
        println!("Center: ({:.2}, {:.2})", center_x, center_y);
        println!("Distance: {:.3}px", dist);
        println!("Aligned: {} (at {}ms)", if aligned { "✓" } else { "✗" }, aligned_ms);
        println!();

        if aligned {
            println!("✓✓✓ SUCCESS: Actor aligned automatically without destination! ✓✓✓");
        } else {
            println!("⚠️⚠️⚠️  FAILURE: Actor did not align ⚠️⚠️⚠️");
        }

        println!("========================================\n");

        // Assertions
        assert!(aligned, "Actor should align automatically after spawn");
        assert!(dist < 2.0, "Actor distance {:.3}px should be < 2.0px", dist);
        assert!(actor.subcell_destination.is_none(), "Actor should have NO destination");
    } else {
        panic!("Actor has no current_subcell after spawn");
    }
}
