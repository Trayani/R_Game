/// Test that 6 actors spawning at the same location properly handle subcell overflow
/// With a 2x2 subcell grid, only 4 subcells exist per cell.
/// The 5th and 6th actors should either:
/// - Wait in NoSubcell state until a subcell becomes available, OR
/// - Reserve subcells in adjacent cells
///
/// This tests the edge case of more actors than available subcells.

use rustgame3::Actor;
use rustgame3::subcell::SubCellReservationManager;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};
use std::collections::HashSet;

#[test]
fn test_six_actors_at_same_location() {
    println!("\n========================================");
    println!("  Six Actors Subcell Reservation Test");
    println!("========================================\n");

    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    // Spawn 6 actors at THE SAME position (more than 4 subcells available in one cell)
    let start_x = 157.3;
    let start_y = 207.8;

    let mut actors = Vec::new();
    for i in 0..6 {
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
    println!("Expected behavior:");
    println!("  - Cell (5,6) has 4 subcells: (0,0), (0,1), (1,0), (1,1)");
    println!("  - First 4 actors should reserve these subcells");
    println!("  - Actors 5 and 6 should either:");
    println!("    a) Wait in NoSubcell state, OR");
    println!("    b) Reserve subcells in adjacent cells\n");

    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);

    // Run simulation to let actors reserve subcells
    let max_iterations = 200;
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

        // Check progress every 50 iterations
        if iteration % 50 == 0 && iteration > 0 {
            let reserved_count = actors.iter().filter(|a| a.current_subcell.is_some()).count();
            println!("[{:4}ms] Reserved: {}/{} actors", iteration * 16, reserved_count, actors.len());
        }
    }

    println!("\n========================================");
    println!("  RESERVATION RESULTS");
    println!("========================================");

    // Collect all reserved subcells
    let mut reserved_subcells = HashSet::new();
    let mut actors_in_no_subcell = 0;

    for actor in &actors {
        if let Some(sc) = actor.current_subcell {
            let (cell_x, cell_y) = sc.to_cell(actor.subcell_grid_size);
            let (sub_x, sub_y) = sc.subcell_offset(actor.subcell_grid_size);
            println!("Actor {}: subcell ({},{},{},{}) @ state {:?}",
                actor.id, cell_x, cell_y, sub_x, sub_y, actor.alignment_state);

            let key = (cell_x, cell_y, sub_x, sub_y);
            if reserved_subcells.contains(&key) {
                println!("  ⚠️ ERROR: This subcell is already used by another actor!");
            }
            reserved_subcells.insert(key);
        } else {
            println!("Actor {}: NO SUBCELL (state: {:?})", actor.id, actor.alignment_state);
            actors_in_no_subcell += 1;
        }
    }

    println!();
    println!("Unique subcells reserved: {}", reserved_subcells.len());
    println!("Actors in NoSubcell state: {}", actors_in_no_subcell);
    println!("Total actors: {}", actors.len());

    // Group by cell
    let mut cells: std::collections::HashMap<(i32, i32), Vec<(i32, i32, usize)>> = std::collections::HashMap::new();
    for actor in &actors {
        if let Some(sc) = actor.current_subcell {
            let (cell_x, cell_y) = sc.to_cell(actor.subcell_grid_size);
            let (sub_x, sub_y) = sc.subcell_offset(actor.subcell_grid_size);
            cells.entry((cell_x, cell_y))
                .or_insert_with(Vec::new)
                .push((sub_x, sub_y, actor.id));
        }
    }

    println!("\nSubcells by cell:");
    for ((cell_x, cell_y), subcells) in cells.iter() {
        println!("  Cell ({},{}) - {} subcells:", cell_x, cell_y, subcells.len());
        for (sub_x, sub_y, actor_id) in subcells {
            println!("    ({},{}) = Actor {}", sub_x, sub_y, actor_id);
        }
    }

    println!("\n========================================");

    // Assertions
    assert!(reserved_subcells.len() <= 6, "Should reserve at most 6 subcells (one per actor)");

    // All reserved subcells must be unique
    assert_eq!(reserved_subcells.len(), actors.iter().filter(|a| a.current_subcell.is_some()).count(),
        "All reserved subcells must be unique");

    // Either all 6 actors have subcells, OR some are waiting in NoSubcell state
    let actors_with_subcells = actors.iter().filter(|a| a.current_subcell.is_some()).count();
    if actors_with_subcells == 6 {
        println!("✓✓✓ SUCCESS: All 6 actors got unique subcells! ✓✓✓");
    } else {
        println!("✓✓✓ SUCCESS: {} actors got subcells, {} waiting in NoSubcell ✓✓✓",
            actors_with_subcells, actors_in_no_subcell);

        // Verify actors without subcells are in NoSubcell state
        for actor in &actors {
            if actor.current_subcell.is_none() {
                assert_eq!(actor.alignment_state, rustgame3::actor::AlignmentState::NoSubcell,
                    "Actor {} should be in NoSubcell state", actor.id);
            }
        }
    }

    println!("========================================\n");
}
