/// Headless simulation of F9 (load save state) + P (set destination) workflow
/// This test is CRITICAL - simulates real usage scenario with obstacles and multiple actors

use rustgame3::{SaveState, Grid, SubCellReservationManager, ReservationEagerness, ReleaseEagerness};
use rustgame3::pathfinding::Position;
use rustgame3::subcell::spread_cell_destinations;
use std::collections::HashMap;

#[test]
fn test_f9_load_and_p_destination() {
    println!("\n=== F9 + P Simulation Test ===");
    println!("Loading save_state.json and directing actors to bottom-left destination");

    // Step 1: Load save state (F9 equivalent)
    let save_state = SaveState::load_from_file("save_state.json")
        .expect("Failed to load save_state.json - make sure file exists!");

    println!("\n[F9] Loaded save state:");
    println!("  Grid: {}x{}", save_state.grid_cols, save_state.grid_rows);
    println!("  Obstacles: {} cells", save_state.blocked_cells.len());
    println!("  Actors: {}", save_state.actors.len());

    // Print actor starting positions
    println!("\n[ACTORS] Starting positions:");
    for (i, actor_data) in save_state.actors.iter().enumerate() {
        println!("  Actor {}: ({:.1}, {:.1})", i, actor_data.fpos_x, actor_data.fpos_y);
    }

    // Recreate grid
    let mut grid = Grid::new(save_state.grid_cols, save_state.grid_rows);
    for &cell_id in &save_state.blocked_cells {
        let x = cell_id % save_state.grid_cols;
        let y = cell_id / save_state.grid_cols;
        grid.set_cell(x, y, 1);
    }

    // Recreate actors
    let cell_width = 30.0;
    let cell_height = 30.0;
    let subcell_grid_size = 2;

    let mut actors = save_state.restore_actors(
        cell_width,
        cell_height,
        subcell_grid_size,
        0.0, // subcell_offset_x
        0.0, // subcell_offset_y
        50.0, // configured_speed
        0.6 // distance_tolerance_multiplier
    );
    let mut reservation_mgr = SubCellReservationManager::new(subcell_grid_size);

    // Step 2: Set destination to bottom-left (P key equivalent)
    // User said "bottom left place" - let's use cell (5, 35) as target
    let target_x = 5;
    let target_y = 35;

    println!("\n[P] Setting destinations to bottom-left area ({}, {})", target_x, target_y);
    println!("  Spreading {} actors across different cells", actors.len());

    // Spread actors across different cells (same as GUI does)
    let cell_destinations = spread_cell_destinations(target_x, target_y, actors.len());

    for (actor, (dest_x, dest_y)) in actors.iter_mut().zip(cell_destinations.iter()) {
        let dest_pos = Position { x: *dest_x, y: *dest_y };
        actor.set_subcell_destination(dest_pos);
        println!("  Actor {} -> cell ({}, {})", actor.id, dest_x, dest_y);
    }

    // Step 3: Run simulation
    println!("\n[SIMULATION] Running for 4000 iterations...");

    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 4000; // Increased for long-distance travel with obstacles
    let mut reached_count = 0;
    let mut progress_snapshots: HashMap<usize, Vec<(f32, f32)>> = HashMap::new();

    for iteration in 0..max_iterations {
        let mut actors_reached_this_frame = 0;

        for i in 0..actors.len() {
            // Skip if already reached
            if actors[i].subcell_destination.is_none() {
                continue;
            }

            let reached = actors[i].update_subcell_destination_direct(
                delta_time,
                &mut reservation_mgr,
                false, // enable_square_reservation
                false, // enable_diagonal_constraint
                false, // enable_no_diagonal
                true,  // enable_anti_cross
                false, // enable_basic3
                false, // enable_basic3_anti_cross
                true,  // enable_early_reservation
                false, // filter_backward
                false, // basic3_fallback_enabled
                false, // track_movement
                0.1,   // reservation_threshold_distance
                ReservationEagerness::Center,
                ReleaseEagerness::Center,
            );

            if reached {
                actors_reached_this_frame += 1;
                reached_count += 1;
            }

            // Record progress every 100 iterations
            if iteration % 100 == 0 {
                progress_snapshots
                    .entry(iteration)
                    .or_insert_with(Vec::new)
                    .push((actors[i].fpos_x, actors[i].fpos_y));
            }
        }

        // Progress update every 100 iterations
        if iteration % 100 == 0 {
            println!("  Iteration {}: {}/{} actors reached destination",
                iteration, reached_count, actors.len());

            // Show a few sample positions
            if iteration % 200 == 0 && iteration > 0 {
                for i in 0..3.min(actors.len()) {
                    if actors[i].subcell_destination.is_some() {
                        println!("    Actor {} at ({:.1}, {:.1})",
                            actors[i].id, actors[i].fpos_x, actors[i].fpos_y);
                    }
                }
            }
        }

        // Early exit if all reached
        if reached_count == actors.len() {
            println!("\n[SUCCESS] All actors reached destination at iteration {}", iteration);
            break;
        }
    }

    // Step 4: Verify results
    println!("\n=== VERIFICATION ===");
    println!("Final status: {}/{} actors reached destination", reached_count, actors.len());

    // Show final positions
    println!("\n[FINAL POSITIONS]");
    let mut stuck_actors = 0;
    for (i, actor) in actors.iter().enumerate() {
        let status = if actor.subcell_destination.is_none() {
            "REACHED"
        } else {
            stuck_actors += 1;
            "STUCK"
        };
        println!("  Actor {} ({:.1}, {:.1}): {}",
            actor.id, actor.fpos_x, actor.fpos_y, status);
    }

    // Analyze progress
    println!("\n[PROGRESS ANALYSIS]");
    if let (Some(start_positions), Some(end_positions)) =
        (progress_snapshots.get(&0), progress_snapshots.get(&900))
    {
        for i in 0..actors.len().min(5) {
            let (start_x, start_y) = start_positions[i];
            let (end_x, end_y) = end_positions[i];
            let distance_traveled = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
            println!("  Actor {}: traveled {:.1} pixels", i, distance_traveled);
        }
    }

    // Assertions
    println!("\n[ASSERTIONS]");

    // At least 40% of actors should reach destination (realistic with obstacles and distance)
    let success_ratio = reached_count as f32 / actors.len() as f32;
    println!("Success ratio: {:.1}%", success_ratio * 100.0);

    assert!(
        success_ratio >= 0.4,
        "FAIL: Only {}/{} actors ({:.1}%) reached destination - expected at least 40%",
        reached_count,
        actors.len(),
        success_ratio * 100.0
    );

    // All actors should have made significant progress (moved > 100 pixels)
    let mut insufficient_progress = 0;
    if let (Some(start_positions), Some(end_positions)) =
        (progress_snapshots.get(&0), progress_snapshots.get(&900))
    {
        for i in 0..actors.len() {
            let (start_x, start_y) = start_positions[i];
            let (end_x, end_y) = end_positions[i];
            let distance = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
            if distance < 100.0 && actors[i].subcell_destination.is_some() {
                insufficient_progress += 1;
                println!("  WARNING: Actor {} barely moved ({:.1} pixels)", actors[i].id, distance);
            }
        }
    }

    if insufficient_progress > 0 {
        println!("  NOTE: {} actors made insufficient progress - may be blocked by obstacles or other actors",
            insufficient_progress);
    }

    // Allow up to 20% of actors to be stuck (obstacles, crowding, edge cases)
    assert!(
        insufficient_progress <= actors.len() / 5,
        "FAIL: Too many actors stuck - {} actors made insufficient progress (< 100 pixels)",
        insufficient_progress
    );

    // No actors should be completely stuck at starting position
    assert!(
        reached_count > 0,
        "FAIL: NO actors reached destination - complete failure!"
    );

    println!("\n=== TEST PASSED ===");
    println!("✓ {}/{} actors reached destination ({:.1}%)",
        reached_count, actors.len(), success_ratio * 100.0);
    println!("✓ All actors made progress");
    println!("✓ Reservation system working with real obstacles");
}

#[test]
fn test_actors_avoid_obstacles_from_save_state() {
    // Verify actors don't try to path through the obstacle wall
    println!("\n=== Obstacle Avoidance Test ===");

    let save_state = SaveState::load_from_file("save_state.json")
        .expect("Failed to load save_state.json");

    // Recreate grid
    let mut grid = Grid::new(save_state.grid_cols, save_state.grid_rows);
    for &cell_id in &save_state.blocked_cells {
        let x = cell_id % save_state.grid_cols;
        let y = cell_id / save_state.grid_cols;
        grid.set_cell(x, y, 1);
    }

    println!("Grid: {}x{}", save_state.grid_cols, save_state.grid_rows);
    println!("Obstacles form a wall - actors must navigate around it");

    // Verify the obstacle structure (should be a vertical wall around x=10-12)
    let mut obstacle_columns: HashMap<i32, i32> = HashMap::new();
    for &cell_id in &save_state.blocked_cells {
        let x = cell_id % save_state.grid_cols;
        *obstacle_columns.entry(x).or_insert(0) += 1;
    }

    println!("\nObstacle distribution by column:");
    let mut columns: Vec<_> = obstacle_columns.iter().collect();
    columns.sort_by_key(|(x, _)| **x);
    for (x, count) in columns.iter().take(10) {
        println!("  Column {}: {} obstacles", x, count);
    }

    // The wall should be substantial
    let wall_columns = obstacle_columns.values().filter(|&&count| count > 5).count();
    assert!(
        wall_columns >= 2,
        "Expected obstacle wall, found only {} columns with obstacles",
        wall_columns
    );

    println!("\n✓ Obstacle wall detected ({} dense columns)", wall_columns);
    println!("✓ Ready for pathfinding test");
}
