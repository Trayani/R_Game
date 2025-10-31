/// Test that multiple actors spawning at the same location
/// do NOT align to the same subcell - each gets their own subcell

use rustgame3::Actor;
use rustgame3::SubPointReservationManager;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};
use std::collections::HashSet;

#[test]
fn test_actors_get_unique_subcells() {
    println!("\n========================================");
    println!("  Unique Subcell Reservation Test");
    println!("========================================\n");

    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Spawn 4 actors at THE SAME position (same as number of subcells in a cell)
    let start_x = 157.3;
    let start_y = 207.8;

    let mut actors = Vec::new();
    for i in 0..4 {
        let mut actor = Actor::new(
            i,
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
        actors.push(actor);
    }

    println!("Spawned {} actors at SAME position ({:.1}, {:.1})", actors.len(), start_x, start_y);
    println!("Each actor should reserve a DIFFERENT subcell\n");

    let mut reservation_mgr = SubPointReservationManager::new(subcell_grid_size, 1000, 1000);

    // Run simulation to let actors reserve subcells
    let max_iterations = 100;
    let delta_time = 0.016;

    println!("Running simulation for up to {} iterations...\n", max_iterations);

    for iteration in 0..max_iterations {
        for actor in &mut actors {
            let _reached = actor.update_subcell_destination_direct(
                delta_time,
                &mut reservation_mgr,
                false, false, false, false, 0.0,
                ReservationEagerness::Center,
                ReleaseEagerness::Center,
            );
        }

        // Check if all actors have reserved subcells
        let all_have_subcells = actors.iter().all(|a| a.current_subcell.is_some());
        if all_have_subcells {
            println!("\n[{:4}ms] All actors have reserved subcells", iteration * 16);
            break;
        }
    }

    println!("\n========================================");
    println!("  RESERVATION RESULTS");
    println!("========================================");

    // Collect all reserved subcells
    let mut reserved_subcells = HashSet::new();
    for actor in &actors {
        if let Some(sc) = actor.current_subcell {
            let (cell_x, cell_y) = sc.to_cell(subcell_grid_size);
            let (sub_x, sub_y) = sc.subcell_offset(subcell_grid_size);
            println!("Actor {}: subcell ({},{},{},{})",
                actor.id, cell_x, cell_y, sub_x, sub_y);

            let key = (cell_x, cell_y, sub_x, sub_y);
            if reserved_subcells.contains(&key) {
                println!("  ⚠️ ERROR: This subcell is already used by another actor!");
            }
            reserved_subcells.insert(key);
        } else {
            println!("Actor {}: NO SUBCELL (could not reserve)", actor.id);
        }
    }

    println!();
    println!("Unique subcells reserved: {}/{}", reserved_subcells.len(), actors.len());

    if reserved_subcells.len() == actors.len() {
        println!("✓✓✓ SUCCESS: All actors got unique subcells! ✓✓✓");
    } else {
        println!("⚠️⚠️⚠️  FAILURE: Some actors share subcells ⚠️⚠️⚠️");
    }

    println!("========================================\n");

    // Assertions
    assert_eq!(reserved_subcells.len(), actors.len(),
        "Expected {} unique subcells, got {}",
        actors.len(), reserved_subcells.len());

    // Verify no two actors have the same subcell
    for actor in &actors {
        assert!(actor.current_subcell.is_some(),
            "Actor {} should have a reserved subcell", actor.id);
    }
}
