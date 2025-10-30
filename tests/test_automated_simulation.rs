/// Automated simulation test to verify distance rule filter prevents backwards movements
///
/// IMPORTANT: Per actor_states.txt design document:
/// - Actors spawn in PSC_ALIGNMENT state and must first move to nearest subcell center
/// - PSC_ALIGNMENT movements CAN move backwards relative to destination (by design)
/// - Only AFTER alignment does distance rule filter apply to navigation movements
///
/// This test tracks PSC_ALIGNMENT vs navigation separately to verify:
/// 1. PSC_ALIGNMENT: May have backwards movements (acceptable)
/// 2. Navigation: Must have 0 backwards movements (enforced by distance rule filter)

use rustgame3::Actor;
use rustgame3::pathfinding::Position;
use rustgame3::subcell::SubCellReservationManager;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};
use std::collections::HashSet;

#[test]
fn test_automated_simulation_with_diagonal_fallback() {
    println!("\n========================================");
    println!("  AUTOMATED SIMULATION: Distance Rule");
    println!("========================================\n");

    let cell_width = 30.0;
    let cell_height = 20.0;
    let subcell_grid_size = 2;

    // Create multiple actors in northeast region
    let mut actors = Vec::new();
    for i in 0..10 {
        let start_x = (15 + i % 3) as f32 * cell_width + 7.5;
        let start_y = (10 + i / 3) as f32 * cell_height + 5.0;

        let mut actor = Actor::new(
            i,
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
        actors.push(actor);
    }

    println!("✓ Created {} actors in northeast region", actors.len());

    // Set SOUTHWEST destinations (similar to Actor 3 scenario)
    let dest_cell_x = 5;
    let dest_cell_y = 30;
    let dest_screen_x = dest_cell_x as f32 * cell_width;
    let dest_screen_y = dest_cell_y as f32 * cell_height;

    for actor in &mut actors {
        actor.set_subcell_destination(Position { x: dest_cell_x, y: dest_cell_y });
    }

    println!("✓ Set destination: cell ({}, {}) = pos ({:.1}, {:.1})",
        dest_cell_x, dest_cell_y, dest_screen_x, dest_screen_y);
    println!("✓ Direction: SOUTHWEST (left and down)\n");
    println!("NOTE: Per actor_states.txt, actors will first align to nearest subcell.");
    println!("      PSC_ALIGNMENT movements may go backwards - this is expected!\n");

    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);

    // Track which actors have completed PSC_ALIGNMENT
    let mut actors_aligned: HashSet<usize> = HashSet::new();

    // Run simulation
    let max_iterations = 1000;
    let delta_time = 0.016; // ~60 FPS
    let mut reached_count = 0;

    // Separate counters for alignment vs navigation
    let mut alignment_backwards = 0;
    let mut navigation_backwards = 0;
    let mut alignment_movements = 0;
    let mut navigation_movements = 0;

    println!("Running simulation for up to {} iterations...\n", max_iterations);

    for iteration in 0..max_iterations {
        let mut any_movement = false;

        for actor in &mut actors {
            if actor.subcell_destination.is_none() {
                continue; // Already reached
            }

            // Check if actor has valid reservation (indicates it's past PSC_ALIGNMENT)
            let has_reservation = actor.reserved_subcell.is_some();
            let is_navigating = actors_aligned.contains(&(actor.id as usize)) && has_reservation;

            let old_pos = (actor.fpos_x, actor.fpos_y);
            let old_dist_x = (dest_screen_x - old_pos.0).abs();
            let old_dist_y = (dest_screen_y - old_pos.1).abs();

            let reached = actor.update_subcell_destination_direct(
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
            let new_dist_x = (dest_screen_x - new_pos.0).abs();
            let new_dist_y = (dest_screen_y - new_pos.1).abs();

            // Mark actor as aligned once it has successfully reserved a subcell
            if has_reservation && !actors_aligned.contains(&(actor.id as usize)) {
                actors_aligned.insert(actor.id as usize);
                println!("[{:4}ms] Actor {} completed PSC_ALIGNMENT, now navigating",
                    iteration * 16, actor.id);
            }

            // Check for backwards movement
            let moved = (new_pos.0 - old_pos.0).abs() > 0.01 || (new_pos.1 - old_pos.1).abs() > 0.01;

            if moved {
                any_movement = true;

                // Check if distance increased in either direction
                let backwards_x = new_dist_x > old_dist_x + 0.1;
                let backwards_y = new_dist_y > old_dist_y + 0.1;

                if is_navigating {
                    navigation_movements += 1;

                    if backwards_x || backwards_y {
                        navigation_backwards += 1;
                        println!("⚠️  NAV BACKWARDS [{:4}ms] Actor {} moved from ({:.1},{:.1}) to ({:.1},{:.1})",
                            iteration * 16, actor.id, old_pos.0, old_pos.1, new_pos.0, new_pos.1);
                        println!("    Distance: X: {:.1}→{:.1} ({:+.1}) Y: {:.1}→{:.1} ({:+.1})",
                            old_dist_x, new_dist_x, new_dist_x - old_dist_x,
                            old_dist_y, new_dist_y, new_dist_y - old_dist_y);
                    }
                } else {
                    // PSC_ALIGNMENT phase
                    alignment_movements += 1;

                    if backwards_x || backwards_y {
                        alignment_backwards += 1;
                        if iteration < 100 {  // Only log first few alignment backwards
                            println!("[{:4}ms] Actor {} PSC_ALIGNMENT backwards (expected): ({:.1},{:.1}) → ({:.1},{:.1})",
                                iteration * 16, actor.id, old_pos.0, old_pos.1, new_pos.0, new_pos.1);
                        }
                    }
                }
            }

            if reached {
                reached_count += 1;
                println!("✓  [{:4}ms] Actor {} REACHED destination at ({:.1},{:.1})",
                    iteration * 16, actor.id, new_pos.0, new_pos.1);
            }

            // Clear diagnostic messages to avoid memory buildup
            actor.diagnostic_messages.clear();
        }

        // Progress report every 200 iterations
        if iteration % 200 == 0 && iteration > 0 {
            let active_actors = actors.iter().filter(|a| a.subcell_destination.is_some()).count();
            let aligned = actors_aligned.len();
            println!("[{:4}ms] {} actors moving, {} aligned, {} reached",
                iteration * 16, active_actors, aligned, reached_count);
            println!("       Alignment: {} movements ({} backwards)",
                alignment_movements, alignment_backwards);
            println!("       Navigation: {} movements ({} backwards)",
                navigation_movements, navigation_backwards);
        }

        // Stop if no movement for a while
        if !any_movement && iteration > 100 {
            println!("\n[{:4}ms] No movement detected, simulation complete", iteration * 16);
            break;
        }
    }

    println!("\n========================================");
    println!("  SIMULATION RESULTS");
    println!("========================================");
    println!("Actors reached destination: {}/{}", reached_count, actors.len());
    println!();
    println!("PSC_ALIGNMENT Phase:");
    println!("  Movements: {}", alignment_movements);
    println!("  Backwards: {} ({:.1}%) ← EXPECTED per actor_states.txt",
        alignment_backwards,
        if alignment_movements > 0 { (alignment_backwards as f32 / alignment_movements as f32) * 100.0 } else { 0.0 }
    );
    println!();
    println!("Navigation Phase:");
    println!("  Movements: {}", navigation_movements);
    println!("  Backwards: {} ({:.1}%) ← MUST BE 0 (distance rule filter)",
        navigation_backwards,
        if navigation_movements > 0 { (navigation_backwards as f32 / navigation_movements as f32) * 100.0 } else { 0.0 }
    );
    println!();

    if navigation_backwards == 0 && alignment_backwards > 0 {
        println!("✓✓✓ SUCCESS: Distance rule filter working correctly! ✓✓✓");
        println!("    - PSC_ALIGNMENT backwards: {} (expected)", alignment_backwards);
        println!("    - Navigation backwards: 0 (enforced by filter)");
    } else if navigation_backwards == 0 {
        println!("✓✓✓ SUCCESS: No backwards movements! ✓✓✓");
    } else {
        println!("⚠️⚠️⚠️  FAILURE: {} navigation backwards movements! ⚠️⚠⚠️", navigation_backwards);
    }

    println!("========================================\n");

    // Assertions
    assert!(reached_count > 0, "Expected at least some actors to reach destination");
    assert_eq!(navigation_backwards, 0,
        "Expected 0 navigation backwards movements, found {}. \
         PSC_ALIGNMENT backwards ({}) are acceptable per actor_states.txt design.",
        navigation_backwards, alignment_backwards);
}
