// Test direct movement to +5X +13Y destination
// Verifies that actor moves directly without zigzagging

use rustgame3::Actor;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_direct_movement_plus_5x_plus_13y() {
    println!("\n=== Testing Direct Movement +5X +13Y ===\n");

    let cell_width = 32.0;
    let cell_height = 32.0;

    // Spawn at origin
    let spawn_x = 64.0;  // Cell (2, 2)
    let spawn_y = 64.0;
    let spawn_cell_x = (spawn_x / cell_width) as i32;
    let spawn_cell_y = (spawn_y / cell_height) as i32;

    // Destination is +5X +13Y from spawn cell
    let dest_cell_x = spawn_cell_x + 5;  // Cell (7, 15)
    let dest_cell_y = spawn_cell_y + 13;

    println!("Spawn cell: ({}, {})", spawn_cell_x, spawn_cell_y);
    println!("Spawn position: ({:.1}, {:.1})", spawn_x, spawn_y);
    println!("Destination cell: ({}, {})", dest_cell_x, dest_cell_y);
    println!("Destination position: ({:.1}, {:.1})",
        dest_cell_x as f32 * cell_width,
        dest_cell_y as f32 * cell_height);

    let dx = dest_cell_x - spawn_cell_x;
    let dy = dest_cell_y - spawn_cell_y;
    let distance = ((dx * dx + dy * dy) as f32).sqrt();

    println!("Distance: {:.2} cells ({}X, {}Y)", distance, dx, dy);
    println!("Direction: diagonal (RIGHT and DOWN)\n");

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
        false,  // enable_lookahead
        0.0,    // psc_switch_threshold
    );
    actor.use_directing_v2 = true;

    // Set starting subcell
    let start_subcell = SubCellCoord::from_screen_pos_with_offset(
        spawn_x, spawn_y, cell_width, cell_height, 2, 0.0, 0.0
    );
    actor.current_subcell = Some(start_subcell.to_subpoint());

    println!("Starting subcell: ({},{},{},{})\n",
        start_subcell.cell_x, start_subcell.cell_y,
        start_subcell.sub_x, start_subcell.sub_y);

    // Set destination
    let dest = Position { x: dest_cell_x, y: dest_cell_y };
    actor.set_subcell_destination(dest);

    let mut reservation_mgr = SubCellReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Track movement for analysis
    let delta_time = 0.016;
    let max_iterations = 2000;
    let mut iteration = 0;
    let mut reached = false;

    // Movement analysis
    let mut total_backwards_x = 0;
    let mut total_backwards_y = 0;
    let mut position_history = Vec::new();
    position_history.push((spawn_x, spawn_y));

    println!("Starting simulation...\n");

    while iteration < max_iterations && !reached {
        let old_x = actor.fpos_x;
        let old_y = actor.fpos_y;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // enable_anti_cross (disabled to test if 3-cell reservation prevents crossing)
            true,  // track_movement - ENABLE for detailed logs
            0.0,
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        // Track position
        position_history.push((actor.fpos_x, actor.fpos_y));

        // Check for backwards movement
        let dx_move = actor.fpos_x - old_x;
        let dy_move = actor.fpos_y - old_y;

        // We expect rightward (+X) and downward (+Y) movement
        if dx_move < -0.001 {
            total_backwards_x += 1;
            println!("  [WARN] Backwards X movement at iter {}: {:.3} -> {:.3} (delta: {:.3})",
                iteration, old_x, actor.fpos_x, dx_move);
        }
        if dy_move < -0.001 {
            total_backwards_y += 1;
            println!("  [WARN] Backwards Y movement at iter {}: {:.3} -> {:.3} (delta: {:.3})",
                iteration, old_y, actor.fpos_y, dy_move);
        }

        let moved = (actor.fpos_x - old_x).abs() > 0.001 || (actor.fpos_y - old_y).abs() > 0.001;

        // Report every second
        if iteration % 60 == 0 {
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

    // Analyze movement
    println!("\n=== Movement Analysis ===");
    println!("Total iterations: {}", iteration);
    println!("Backwards X movements: {}", total_backwards_x);
    println!("Backwards Y movements: {}", total_backwards_y);

    let start_x = position_history[0].0;
    let start_y = position_history[0].1;
    let end_x = position_history[position_history.len() - 1].0;
    let end_y = position_history[position_history.len() - 1].1;

    let total_dx = end_x - start_x;
    let total_dy = end_y - start_y;

    println!("Total displacement: ({:.1}, {:.1})", total_dx, total_dy);
    println!("Expected direction: (+X, +Y)");
    println!("Actual direction: ({}X, {}Y)",
        if total_dx > 0.0 { "+" } else { "-" },
        if total_dy > 0.0 { "+" } else { "-" });

    // Calculate path efficiency
    let straight_line_distance = ((total_dx).powi(2) + (total_dy).powi(2)).sqrt();
    let mut actual_path_length = 0.0;
    for i in 1..position_history.len() {
        let dx = position_history[i].0 - position_history[i-1].0;
        let dy = position_history[i].1 - position_history[i-1].1;
        actual_path_length += (dx * dx + dy * dy).sqrt();
    }

    let efficiency = (straight_line_distance / actual_path_length) * 100.0;
    println!("Path efficiency: {:.1}% (straight: {:.1}px, actual: {:.1}px)",
        efficiency, straight_line_distance, actual_path_length);

    println!("\n=== Test Complete ===\n");

    // Assertions
    assert!(reached, "Actor should reach destination");
    assert_eq!(total_backwards_x, 0, "Actor should never move backwards in X");
    assert_eq!(total_backwards_y, 0, "Actor should never move backwards in Y");
    assert!(efficiency > 95.0, "Path efficiency should be > 95% (was {:.1}%)", efficiency);
}
