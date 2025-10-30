/// Test PSC_ALIGNMENT state in isolation
///
/// Per actor_states.txt Section 3.1:
/// - NO_SUBCELL state: Actor tries to reserve nearby subcell every frame
/// - PSC_ALIGNMENT state: Actor moves directly to subcell center
/// - IDLE state: Actor waits at subcell center (transitions to MOVE when destination set)
///
/// This test verifies:
/// 1. Actors spawn and reserve nearest subcell (NO_SUBCELL → PSC_ALIGNMENT)
/// 2. Actors move to subcell centers via PSC_ALIGNMENT
/// 3. Actors align to subcell centers (distance < 1.0 pixels per PSC_ALIGNMENT_THRESHOLD)
/// 4. Test stops immediately after all actors aligned, before navigation begins
///
/// Note: DestinationDirect mode requires destination to be set (far away at 30,30).
/// Actors will align first, then would start navigating, but test stops at alignment.

use rustgame3::Actor;
use rustgame3::subcell::SubCellReservationManager;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_four_actors_psc_alignment_no_destination() {
    println!("\n========================================");
    println!("  PSC_ALIGNMENT Test (No Destination)");
    println!("========================================\n");

    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Spawn 4 actors at arbitrary positions (NOT at subcell centers)
    let actor_configs = vec![
        (0, 157.3, 207.8),  // Actor 0: offset from center
        (1, 192.1, 215.4),  // Actor 1: offset from center
        (2, 223.7, 241.9),  // Actor 2: offset from center
        (3, 188.5, 268.2),  // Actor 3: offset from center
    ];

    let mut actors = Vec::new();
    for (id, x, y) in actor_configs {
        let mut actor = Actor::new(
            id,
            x,
            y,
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
        actors.push(actor);

        println!("Spawned Actor {} at ({:.1}, {:.1})", id, x, y);
    }

    // Set destinations far away to trigger PSC_ALIGNMENT
    // Note: DestinationDirect mode requires destination to be set
    // Actors will first align to nearest subcell (PSC_ALIGNMENT state)
    // Then navigate toward destination (MOVE state)
    use rustgame3::pathfinding::Position;

    let dest_cell_x = 30;  // Far away
    let dest_cell_y = 30;

    for actor in &mut actors {
        actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });
    }

    println!("\n✓ Created {} actors at arbitrary positions", actors.len());
    println!("✓ Set destination cell ({}, {}) - triggers PSC_ALIGNMENT", dest_cell_x, dest_cell_y);
    println!("✓ Actors will first align to nearest subcells before navigation\n");

    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);

    // Run simulation
    let max_iterations = 200;  // Should complete alignment quickly
    let delta_time = 0.016;    // ~60 FPS

    println!("Running simulation for up to {} iterations...\n", max_iterations);

    let mut actors_aligned = vec![false; actors.len()];
    let mut alignment_complete_frame = vec![None; actors.len()];

    for iteration in 0..max_iterations {
        let mut any_movement = false;

        for (i, actor) in actors.iter_mut().enumerate() {
            let old_pos = (actor.fpos_x, actor.fpos_y);

            // Update without destination
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

            // Check if actor has a reservation (successfully transitioned from NO_SUBCELL)
            if let Some(reserved) = actor.reserved_subcell {
                // Calculate distance to reserved subcell center
                let (center_x, center_y) = reserved.to_screen_center(cell_width, cell_height);
                let dist_to_center = ((new_pos.0 - center_x).powi(2) + (new_pos.1 - center_y).powi(2)).sqrt();

                // Check if aligned (distance < 1.0 pixels per actor_states.txt PSC_ALIGNMENT_THRESHOLD)
                if !actors_aligned[i] && dist_to_center < 1.0 {
                    actors_aligned[i] = true;
                    alignment_complete_frame[i] = Some(iteration);
                    println!("[{:4}ms] Actor {} ALIGNED to subcell ({},{},{},{}) at ({:.1},{:.1}) - dist={:.3}px",
                        iteration * 16, actor.id,
                        reserved.cell_x, reserved.cell_y, reserved.sub_x, reserved.sub_y,
                        new_pos.0, new_pos.1, dist_to_center);
                }

                // Track movement
                let moved = (new_pos.0 - old_pos.0).abs() > 0.01 || (new_pos.1 - old_pos.1).abs() > 0.01;
                if moved {
                    any_movement = true;

                    // Log initial movements to show PSC_ALIGNMENT in action
                    if iteration < 20 && !actors_aligned[i] {
                        let move_dist = ((new_pos.0 - old_pos.0).powi(2) + (new_pos.1 - old_pos.1).powi(2)).sqrt();
                        println!("[{:4}ms] Actor {} PSC_ALIGNMENT: ({:.1},{:.1}) → ({:.1},{:.1}) [move={:.2}px, dist_to_center={:.2}px]",
                            iteration * 16, actor.id,
                            old_pos.0, old_pos.1, new_pos.0, new_pos.1,
                            move_dist, dist_to_center);
                    }
                }
            } else {
                // Actor is in NO_SUBCELL state, trying to reserve
                if iteration < 10 {
                    println!("[{:4}ms] Actor {} NO_SUBCELL: attempting to reserve at ({:.1},{:.1})",
                        iteration * 16, actor.id, new_pos.0, new_pos.1);
                }
            }
        }

        // Stop immediately after all actors have aligned (before they start navigating away)
        let all_aligned = actors_aligned.iter().all(|&aligned| aligned);
        if all_aligned {
            println!("\n[{:4}ms] All actors aligned, stopping before navigation phase", iteration * 16);
            break;
        }
    }

    println!("\n========================================");
    println!("  ALIGNMENT RESULTS");
    println!("========================================");

    let aligned_count = actors_aligned.iter().filter(|&&x| x).count();
    println!("Actors aligned: {}/{}", aligned_count, actors.len());
    println!();

    for (i, actor) in actors.iter().enumerate() {
        if let Some(reserved) = actor.reserved_subcell {
            let (center_x, center_y) = reserved.to_screen_center(cell_width, cell_height);
            let dist = ((actor.fpos_x - center_x).powi(2) + (actor.fpos_y - center_y).powi(2)).sqrt();
            let aligned_frame = alignment_complete_frame[i].unwrap_or(0);
            let aligned_ms = aligned_frame * 16;

            println!("Actor {}:", actor.id);
            println!("  Subcell: ({},{},{},{})", reserved.cell_x, reserved.cell_y, reserved.sub_x, reserved.sub_y);
            println!("  Position: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);
            println!("  Center: ({:.2}, {:.2})", center_x, center_y);
            println!("  Distance: {:.3}px", dist);
            println!("  Aligned: {} (at {}ms)", if actors_aligned[i] { "✓" } else { "✗" }, aligned_ms);
            println!();
        } else {
            println!("Actor {}: ✗ No subcell reserved", actor.id);
            println!();
        }
    }

    if aligned_count == actors.len() {
        println!("✓✓✓ SUCCESS: All actors aligned to subcell centers! ✓✓✓");
    } else {
        println!("⚠️⚠️⚠️  FAILURE: Only {}/{} actors aligned ⚠️⚠️⚠️", aligned_count, actors.len());
    }

    println!("========================================\n");

    // Assertions
    assert_eq!(aligned_count, actors.len(),
        "Expected all {} actors to align to subcell centers, only {} aligned",
        actors.len(), aligned_count);

    // Verify all actors are within alignment threshold (1.0 pixels)
    for actor in &actors {
        if let Some(reserved) = actor.reserved_subcell {
            let (center_x, center_y) = reserved.to_screen_center(cell_width, cell_height);
            let dist = ((actor.fpos_x - center_x).powi(2) + (actor.fpos_y - center_y).powi(2)).sqrt();
            assert!(dist < 1.0,
                "Actor {} not properly aligned: distance {:.3}px > 1.0px threshold",
                actor.id, dist);
        } else {
            panic!("Actor {} has no reserved subcell after alignment", actor.id);
        }
    }

    // Verify actors have destination set (required for DestinationDirect mode)
    for actor in &actors {
        assert!(actor.subcell_destination.is_some(),
            "Actor {} should have destination set",
            actor.id);
    }
}
