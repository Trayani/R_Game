// Test direct movement from subcell to diagonal destination
// Validates actor state transitions and subcell crossing behavior

use rustgame3::Actor;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

#[test]
fn test_simple_diagonal_movement_ne() {
    println!("\n=== Test: Simple Diagonal Movement (NE Direction) ===\n");

    // Setup: Actor at subcell (5,5,0,0) moving to destination (9,1) - 4+ cells diagonally NE
    let mut actor = Actor::new(
        0,                  // id
        5.0,                // fpos_x (at grid intersection 5,5)
        5.0,                // fpos_y
        0.5,                // size
        1.0,                // speed (1 unit per second)
        0.25,               // collision_radius
        1.0,                // cell_width
        1.0,                // cell_height
        2,                  // subcell_grid_size (2x2)
        0.0,                // subcell_offset_x
        0.0,                // subcell_offset_y
        true,               // enable_lookahead
            0.5,  // psc_switch_threshold
    );
    actor.use_directing_v2 = true;

    // Set starting subcell
    let start_subcell = SubCellCoord::new(5, 5, 0, 0, 2);
    actor.current_subcell = Some(start_subcell.clone());

    // Set diagonal destination (NE direction, 4+ cells away)
    let dest = Position { x: 9, y: 1 };
    actor.set_subcell_destination(dest);

    println!("Start: ({:.2}, {:.2}) in subcell ({},{},{},{})",
        actor.fpos_x, actor.fpos_y,
        start_subcell.cell_x, start_subcell.cell_y, start_subcell.sub_x, start_subcell.sub_y);
    println!("Destination: ({}, {})", dest.x, dest.y);
    println!("Distance: {:.2} cells",
        ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());

    let mut reservation_mgr = SubCellReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Simulation parameters
    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 500;

    // Track movement
    let mut positions = Vec::new();
    let mut subcells_visited = Vec::new();
    let mut iteration = 0;
    let mut reached = false;

    positions.push((actor.fpos_x, actor.fpos_y));
    subcells_visited.push(start_subcell.clone());

    // Run simulation
    println!("\nSimulating movement...\n");

    while iteration < max_iterations && !reached {
        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // track_movement
            0.0,   // reservation_threshold_distance
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        // Record position every 10 iterations
        if iteration % 10 == 0 {
            positions.push((actor.fpos_x, actor.fpos_y));

            if let Some(current) = &actor.current_subcell {
                // Check if subcell changed
                if subcells_visited.last().map(|last| {
                    last.cell_x != current.cell_x ||
                    last.cell_y != current.cell_y ||
                    last.sub_x != current.sub_x ||
                    last.sub_y != current.sub_y
                }).unwrap_or(true) {
                    subcells_visited.push(current.clone());
                    println!("  [Iter {}] Position: ({:.2}, {:.2}) → Subcell ({},{},{},{})",
                        iteration, actor.fpos_x, actor.fpos_y,
                        current.cell_x, current.cell_y, current.sub_x, current.sub_y);
                }
            }
        }

        if reached {
            println!("\n✓ Reached destination after {} iterations ({:.2}s)",
                iteration, iteration as f32 * delta_time);
            positions.push((actor.fpos_x, actor.fpos_y));
            break;
        }
    }

    if !reached {
        println!("\n✗ Did NOT reach destination after {} iterations", max_iterations);
        println!("Final position: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);
        println!("Distance remaining: {:.2}",
            ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());
        panic!("Actor did not reach destination");
    }

    // Validation 1: Check movement is direct (minimal deviation from straight line)
    println!("\n=== Validation: Movement Directness ===");

    let start_x = positions[0].0;
    let start_y = positions[0].1;
    let end_x = positions.last().unwrap().0;
    let end_y = positions.last().unwrap().1;

    let ideal_dir_x = end_x - start_x;
    let ideal_dir_y = end_y - start_y;
    let ideal_len = (ideal_dir_x * ideal_dir_x + ideal_dir_y * ideal_dir_y).sqrt();

    let mut max_deviation = 0.0_f32;
    let mut total_deviation = 0.0_f32;

    for (i, (px, py)) in positions.iter().enumerate().skip(1) {
        // Calculate perpendicular distance from point to ideal line
        // Using cross product: |cross| / |line_vec|
        let vec_to_point_x = px - start_x;
        let vec_to_point_y = py - start_y;

        let cross = ideal_dir_x * vec_to_point_y - ideal_dir_y * vec_to_point_x;
        let deviation = cross.abs() / ideal_len;

        max_deviation = max_deviation.max(deviation);
        total_deviation += deviation;

        if deviation > 0.5 {
            println!("  [Point {}] Position ({:.2}, {:.2}): deviation = {:.3}",
                i, px, py, deviation);
        }
    }

    let avg_deviation = total_deviation / (positions.len() - 1) as f32;

    println!("Max deviation from straight line: {:.3}", max_deviation);
    println!("Avg deviation from straight line: {:.3}", avg_deviation);

    if max_deviation > 1.0 {
        println!("✗ FAIL: Movement not direct (max deviation {:.3} > 1.0)", max_deviation);
        panic!("Movement deviated significantly from straight line");
    } else {
        println!("✓ PASS: Movement is direct (max deviation {:.3} <= 1.0)", max_deviation);
    }

    // Validation 2: Check subcell crossings are progressive (no backtracking)
    println!("\n=== Validation: Subcell Progression ===");
    println!("Total subcells visited: {}", subcells_visited.len());

    let mut backtracked = false;
    for i in 1..subcells_visited.len() {
        let prev = &subcells_visited[i-1];
        let curr = &subcells_visited[i];

        // Check if we moved closer to destination
        let prev_dist = ((dest.x as f32 - prev.cell_x as f32).powi(2) +
                        (dest.y as f32 - prev.cell_y as f32).powi(2)).sqrt();
        let curr_dist = ((dest.x as f32 - curr.cell_x as f32).powi(2) +
                        (dest.y as f32 - curr.cell_y as f32).powi(2)).sqrt();

        if curr_dist > prev_dist + 0.5 {
            println!("  ✗ Backtrack detected: ({},{}) → ({},{}), dist: {:.2} → {:.2}",
                prev.cell_x, prev.cell_y, curr.cell_x, curr.cell_y, prev_dist, curr_dist);
            backtracked = true;
        }
    }

    if backtracked {
        println!("✗ FAIL: Actor backtracked during movement");
        panic!("Actor moved away from destination");
    } else {
        println!("✓ PASS: No backtracking detected");
    }

    // Validation 3: Check actual path length vs ideal
    println!("\n=== Validation: Path Efficiency ===");

    let mut actual_distance = 0.0;
    for i in 1..positions.len() {
        let dx = positions[i].0 - positions[i-1].0;
        let dy = positions[i].1 - positions[i-1].1;
        actual_distance += (dx * dx + dy * dy).sqrt();
    }

    let ideal_distance = ideal_len;
    let efficiency = ideal_distance / actual_distance;

    println!("Ideal distance: {:.2}", ideal_distance);
    println!("Actual distance: {:.2}", actual_distance);
    println!("Efficiency: {:.2}% ({:.3})", efficiency * 100.0, efficiency);

    if efficiency < 0.95 {
        println!("✗ FAIL: Path inefficient (efficiency {:.2}% < 95%)", efficiency * 100.0);
        panic!("Actor took a very indirect path");
    } else {
        println!("✓ PASS: Path is efficient (efficiency {:.2}% >= 95%)", efficiency * 100.0);
    }

    println!("\n=== Test Complete: All Validations Passed ===\n");
}

#[test]
fn test_simple_diagonal_movement_se() {
    println!("\n=== Test: Simple Diagonal Movement (SE Direction) ===\n");

    // Setup: Actor at subcell (5,5,0,0) moving to destination (9,9) - 4+ cells diagonally SE
    let mut actor = Actor::new(
        0,                  // id
        5.0,                // fpos_x (at grid intersection 5,5)
        5.0,                // fpos_y
        0.5,                // size
        1.0,                // speed (1 unit per second)
        0.25,               // collision_radius
        1.0,                // cell_width
        1.0,                // cell_height
        2,                  // subcell_grid_size (2x2)
        0.0,                // subcell_offset_x
        0.0,                // subcell_offset_y
        true,               // enable_lookahead
            0.5,  // psc_switch_threshold
    );
    actor.use_directing_v2 = true;

    // Set starting subcell
    let start_subcell = SubCellCoord::new(5, 5, 0, 0, 2);
    actor.current_subcell = Some(start_subcell.clone());

    // Set diagonal destination (SE direction, 4+ cells away)
    let dest = Position { x: 9, y: 9 };
    actor.set_subcell_destination(dest);

    println!("Start: ({:.2}, {:.2}) in subcell ({},{},{},{})",
        actor.fpos_x, actor.fpos_y,
        start_subcell.cell_x, start_subcell.cell_y, start_subcell.sub_x, start_subcell.sub_y);
    println!("Destination: ({}, {})", dest.x, dest.y);
    println!("Distance: {:.2} cells",
        ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());

    let mut reservation_mgr = SubCellReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Simulation parameters
    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 500;

    // Track movement
    let mut positions = Vec::new();
    let mut subcells_visited = Vec::new();
    let mut iteration = 0;
    let mut reached = false;

    positions.push((actor.fpos_x, actor.fpos_y));
    subcells_visited.push(start_subcell.clone());

    // Run simulation
    println!("\nSimulating movement...\n");

    while iteration < max_iterations && !reached {
        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // track_movement
            0.0,   // reservation_threshold_distance
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        // Record position every 10 iterations
        if iteration % 10 == 0 {
            positions.push((actor.fpos_x, actor.fpos_y));

            if let Some(current) = &actor.current_subcell {
                // Check if subcell changed
                if subcells_visited.last().map(|last| {
                    last.cell_x != current.cell_x ||
                    last.cell_y != current.cell_y ||
                    last.sub_x != current.sub_x ||
                    last.sub_y != current.sub_y
                }).unwrap_or(true) {
                    subcells_visited.push(current.clone());
                    println!("  [Iter {}] Position: ({:.2}, {:.2}) → Subcell ({},{},{},{})",
                        iteration, actor.fpos_x, actor.fpos_y,
                        current.cell_x, current.cell_y, current.sub_x, current.sub_y);
                }
            }
        }

        if reached {
            println!("\n✓ Reached destination after {} iterations ({:.2}s)",
                iteration, iteration as f32 * delta_time);
            positions.push((actor.fpos_x, actor.fpos_y));
            break;
        }
    }

    if !reached {
        println!("\n✗ Did NOT reach destination after {} iterations", max_iterations);
        println!("Final position: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);
        println!("Distance remaining: {:.2}",
            ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());
        panic!("Actor did not reach destination");
    }

    // Validation 1: Check movement is direct (minimal deviation from straight line)
    println!("\n=== Validation: Movement Directness ===");

    let start_x = positions[0].0;
    let start_y = positions[0].1;
    let end_x = positions.last().unwrap().0;
    let end_y = positions.last().unwrap().1;

    let ideal_dir_x = end_x - start_x;
    let ideal_dir_y = end_y - start_y;
    let ideal_len = (ideal_dir_x * ideal_dir_x + ideal_dir_y * ideal_dir_y).sqrt();

    let mut max_deviation = 0.0_f32;
    let mut total_deviation = 0.0_f32;

    for (i, (px, py)) in positions.iter().enumerate().skip(1) {
        // Calculate perpendicular distance from point to ideal line
        // Using cross product: |cross| / |line_vec|
        let vec_to_point_x = px - start_x;
        let vec_to_point_y = py - start_y;

        let cross = ideal_dir_x * vec_to_point_y - ideal_dir_y * vec_to_point_x;
        let deviation = cross.abs() / ideal_len;

        max_deviation = max_deviation.max(deviation);
        total_deviation += deviation;

        if deviation > 0.5 {
            println!("  [Point {}] Position ({:.2}, {:.2}): deviation = {:.3}",
                i, px, py, deviation);
        }
    }

    let avg_deviation = total_deviation / (positions.len() - 1) as f32;

    println!("Max deviation from straight line: {:.3}", max_deviation);
    println!("Avg deviation from straight line: {:.3}", avg_deviation);

    if max_deviation > 1.0 {
        println!("✗ FAIL: Movement not direct (max deviation {:.3} > 1.0)", max_deviation);
        panic!("Movement deviated significantly from straight line");
    } else {
        println!("✓ PASS: Movement is direct (max deviation {:.3} <= 1.0)", max_deviation);
    }

    // Validation 2: Check subcell crossings are progressive (no backtracking)
    println!("\n=== Validation: Subcell Progression ===");
    println!("Total subcells visited: {}", subcells_visited.len());

    let mut backtracked = false;
    for i in 1..subcells_visited.len() {
        let prev = &subcells_visited[i-1];
        let curr = &subcells_visited[i];

        // Check if we moved closer to destination
        let prev_dist = ((dest.x as f32 - prev.cell_x as f32).powi(2) +
                        (dest.y as f32 - prev.cell_y as f32).powi(2)).sqrt();
        let curr_dist = ((dest.x as f32 - curr.cell_x as f32).powi(2) +
                        (dest.y as f32 - curr.cell_y as f32).powi(2)).sqrt();

        if curr_dist > prev_dist + 0.5 {
            println!("  ✗ Backtrack detected: ({},{}) → ({},{}), dist: {:.2} → {:.2}",
                prev.cell_x, prev.cell_y, curr.cell_x, curr.cell_y, prev_dist, curr_dist);
            backtracked = true;
        }
    }

    if backtracked {
        println!("✗ FAIL: Actor backtracked during movement");
        panic!("Actor moved away from destination");
    } else {
        println!("✓ PASS: No backtracking detected");
    }

    // Validation 3: Check actual path length vs ideal
    println!("\n=== Validation: Path Efficiency ===");

    let mut actual_distance = 0.0;
    for i in 1..positions.len() {
        let dx = positions[i].0 - positions[i-1].0;
        let dy = positions[i].1 - positions[i-1].1;
        actual_distance += (dx * dx + dy * dy).sqrt();
    }

    let ideal_distance = ideal_len;
    let efficiency = ideal_distance / actual_distance;

    println!("Ideal distance: {:.2}", ideal_distance);
    println!("Actual distance: {:.2}", actual_distance);
    println!("Efficiency: {:.2}% ({:.3})", efficiency * 100.0, efficiency);

    if efficiency < 0.95 {
        println!("✗ FAIL: Path inefficient (efficiency {:.2}% < 95%)", efficiency * 100.0);
        panic!("Actor took a very indirect path");
    } else {
        println!("✓ PASS: Path is efficient (efficiency {:.2}% >= 95%)", efficiency * 100.0);
    }

    println!("\n=== Test Complete: All Validations Passed ===\n");
}

#[test]
fn test_simple_diagonal_movement_sw() {
    println!("\n=== Test: Simple Diagonal Movement (SW Direction) ===\n");

    // Setup: Actor at subcell (5,5,0,0) moving to destination (1,9) - 4+ cells diagonally SW
    let mut actor = Actor::new(
        0,                  // id
        5.0,                // fpos_x (at grid intersection 5,5)
        5.0,                // fpos_y
        0.5,                // size
        1.0,                // speed (1 unit per second)
        0.25,               // collision_radius
        1.0,                // cell_width
        1.0,                // cell_height
        2,                  // subcell_grid_size (2x2)
        0.0,                // subcell_offset_x
        0.0,                // subcell_offset_y
        true,               // enable_lookahead
            0.5,  // psc_switch_threshold
    );
    actor.use_directing_v2 = true;

    // Set starting subcell
    let start_subcell = SubCellCoord::new(5, 5, 0, 0, 2);
    actor.current_subcell = Some(start_subcell.clone());

    // Set diagonal destination (SW direction, 4+ cells away)
    let dest = Position { x: 1, y: 9 };
    actor.set_subcell_destination(dest);

    println!("Start: ({:.2}, {:.2}) in subcell ({},{},{},{})",
        actor.fpos_x, actor.fpos_y,
        start_subcell.cell_x, start_subcell.cell_y, start_subcell.sub_x, start_subcell.sub_y);
    println!("Destination: ({}, {})", dest.x, dest.y);
    println!("Distance: {:.2} cells",
        ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());

    let mut reservation_mgr = SubCellReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Simulation parameters
    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 500;

    // Track movement
    let mut positions = Vec::new();
    let mut subcells_visited = Vec::new();
    let mut iteration = 0;
    let mut reached = false;

    positions.push((actor.fpos_x, actor.fpos_y));
    subcells_visited.push(start_subcell.clone());

    // Run simulation
    println!("\nSimulating movement...\n");

    while iteration < max_iterations && !reached {
        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // track_movement
            0.0,   // reservation_threshold_distance
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        // Record position every 10 iterations
        if iteration % 10 == 0 {
            positions.push((actor.fpos_x, actor.fpos_y));

            if let Some(current) = &actor.current_subcell {
                // Check if subcell changed
                if subcells_visited.last().map(|last| {
                    last.cell_x != current.cell_x ||
                    last.cell_y != current.cell_y ||
                    last.sub_x != current.sub_x ||
                    last.sub_y != current.sub_y
                }).unwrap_or(true) {
                    subcells_visited.push(current.clone());
                    println!("  [Iter {}] Position: ({:.2}, {:.2}) → Subcell ({},{},{},{})",
                        iteration, actor.fpos_x, actor.fpos_y,
                        current.cell_x, current.cell_y, current.sub_x, current.sub_y);
                }
            }
        }

        if reached {
            println!("\n✓ Reached destination after {} iterations ({:.2}s)",
                iteration, iteration as f32 * delta_time);
            positions.push((actor.fpos_x, actor.fpos_y));
            break;
        }
    }

    if !reached {
        println!("\n✗ Did NOT reach destination after {} iterations", max_iterations);
        println!("Final position: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);
        println!("Distance remaining: {:.2}",
            ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());
        panic!("Actor did not reach destination");
    }

    // Validation 1: Check movement is direct (minimal deviation from straight line)
    println!("\n=== Validation: Movement Directness ===");

    let start_x = positions[0].0;
    let start_y = positions[0].1;
    let end_x = positions.last().unwrap().0;
    let end_y = positions.last().unwrap().1;

    let ideal_dir_x = end_x - start_x;
    let ideal_dir_y = end_y - start_y;
    let ideal_len = (ideal_dir_x * ideal_dir_x + ideal_dir_y * ideal_dir_y).sqrt();

    let mut max_deviation = 0.0_f32;
    let mut total_deviation = 0.0_f32;

    for (i, (px, py)) in positions.iter().enumerate().skip(1) {
        // Calculate perpendicular distance from point to ideal line
        // Using cross product: |cross| / |line_vec|
        let vec_to_point_x = px - start_x;
        let vec_to_point_y = py - start_y;

        let cross = ideal_dir_x * vec_to_point_y - ideal_dir_y * vec_to_point_x;
        let deviation = cross.abs() / ideal_len;

        max_deviation = max_deviation.max(deviation);
        total_deviation += deviation;

        if deviation > 0.5 {
            println!("  [Point {}] Position ({:.2}, {:.2}): deviation = {:.3}",
                i, px, py, deviation);
        }
    }

    let avg_deviation = total_deviation / (positions.len() - 1) as f32;

    println!("Max deviation from straight line: {:.3}", max_deviation);
    println!("Avg deviation from straight line: {:.3}", avg_deviation);

    if max_deviation > 1.0 {
        println!("✗ FAIL: Movement not direct (max deviation {:.3} > 1.0)", max_deviation);
        panic!("Movement deviated significantly from straight line");
    } else {
        println!("✓ PASS: Movement is direct (max deviation {:.3} <= 1.0)", max_deviation);
    }

    // Validation 2: Check subcell crossings are progressive (no backtracking)
    println!("\n=== Validation: Subcell Progression ===");
    println!("Total subcells visited: {}", subcells_visited.len());

    let mut backtracked = false;
    for i in 1..subcells_visited.len() {
        let prev = &subcells_visited[i-1];
        let curr = &subcells_visited[i];

        // Check if we moved closer to destination
        let prev_dist = ((dest.x as f32 - prev.cell_x as f32).powi(2) +
                        (dest.y as f32 - prev.cell_y as f32).powi(2)).sqrt();
        let curr_dist = ((dest.x as f32 - curr.cell_x as f32).powi(2) +
                        (dest.y as f32 - curr.cell_y as f32).powi(2)).sqrt();

        if curr_dist > prev_dist + 0.5 {
            println!("  ✗ Backtrack detected: ({},{}) → ({},{}), dist: {:.2} → {:.2}",
                prev.cell_x, prev.cell_y, curr.cell_x, curr.cell_y, prev_dist, curr_dist);
            backtracked = true;
        }
    }

    if backtracked {
        println!("✗ FAIL: Actor backtracked during movement");
        panic!("Actor moved away from destination");
    } else {
        println!("✓ PASS: No backtracking detected");
    }

    // Validation 3: Check actual path length vs ideal
    println!("\n=== Validation: Path Efficiency ===");

    let mut actual_distance = 0.0;
    for i in 1..positions.len() {
        let dx = positions[i].0 - positions[i-1].0;
        let dy = positions[i].1 - positions[i-1].1;
        actual_distance += (dx * dx + dy * dy).sqrt();
    }

    let ideal_distance = ideal_len;
    let efficiency = ideal_distance / actual_distance;

    println!("Ideal distance: {:.2}", ideal_distance);
    println!("Actual distance: {:.2}", actual_distance);
    println!("Efficiency: {:.2}% ({:.3})", efficiency * 100.0, efficiency);

    if efficiency < 0.95 {
        println!("✗ FAIL: Path inefficient (efficiency {:.2}% < 95%)", efficiency * 100.0);
        panic!("Actor took a very indirect path");
    } else {
        println!("✓ PASS: Path is efficient (efficiency {:.2}% >= 95%)", efficiency * 100.0);
    }

    println!("\n=== Test Complete: All Validations Passed ===\n");
}

#[test]
fn test_simple_diagonal_movement_nw() {
    println!("\n=== Test: Simple Diagonal Movement (NW Direction) ===\n");

    // Setup: Actor at subcell (5,5,0,0) moving to destination (1,1) - 4+ cells diagonally NW
    let mut actor = Actor::new(
        0,                  // id
        5.0,                // fpos_x (at grid intersection 5,5)
        5.0,                // fpos_y
        0.5,                // size
        1.0,                // speed (1 unit per second)
        0.25,               // collision_radius
        1.0,                // cell_width
        1.0,                // cell_height
        2,                  // subcell_grid_size (2x2)
        0.0,                // subcell_offset_x
        0.0,                // subcell_offset_y
        true,               // enable_lookahead
            0.5,  // psc_switch_threshold
    );
    actor.use_directing_v2 = true;

    // Set starting subcell
    let start_subcell = SubCellCoord::new(5, 5, 0, 0, 2);
    actor.current_subcell = Some(start_subcell.clone());

    // Set diagonal destination (NW direction, 4+ cells away)
    let dest = Position { x: 1, y: 1 };
    actor.set_subcell_destination(dest);

    println!("Start: ({:.2}, {:.2}) in subcell ({},{},{},{})",
        actor.fpos_x, actor.fpos_y,
        start_subcell.cell_x, start_subcell.cell_y, start_subcell.sub_x, start_subcell.sub_y);
    println!("Destination: ({}, {})", dest.x, dest.y);
    println!("Distance: {:.2} cells",
        ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());

    let mut reservation_mgr = SubCellReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Simulation parameters
    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 500;

    // Track movement
    let mut positions = Vec::new();
    let mut subcells_visited = Vec::new();
    let mut iteration = 0;
    let mut reached = false;

    positions.push((actor.fpos_x, actor.fpos_y));
    subcells_visited.push(start_subcell.clone());

    // Run simulation
    println!("\nSimulating movement...\n");

    while iteration < max_iterations && !reached {
        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // track_movement
            0.0,   // reservation_threshold_distance
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;

        // Record position every 10 iterations
        if iteration % 10 == 0 {
            positions.push((actor.fpos_x, actor.fpos_y));

            if let Some(current) = &actor.current_subcell {
                // Check if subcell changed
                if subcells_visited.last().map(|last| {
                    last.cell_x != current.cell_x ||
                    last.cell_y != current.cell_y ||
                    last.sub_x != current.sub_x ||
                    last.sub_y != current.sub_y
                }).unwrap_or(true) {
                    subcells_visited.push(current.clone());
                    println!("  [Iter {}] Position: ({:.2}, {:.2}) → Subcell ({},{},{},{})",
                        iteration, actor.fpos_x, actor.fpos_y,
                        current.cell_x, current.cell_y, current.sub_x, current.sub_y);
                }
            }
        }

        if reached {
            println!("\n✓ Reached destination after {} iterations ({:.2}s)",
                iteration, iteration as f32 * delta_time);
            positions.push((actor.fpos_x, actor.fpos_y));
            break;
        }
    }

    if !reached {
        println!("\n✗ Did NOT reach destination after {} iterations", max_iterations);
        println!("Final position: ({:.2}, {:.2})", actor.fpos_x, actor.fpos_y);
        println!("Distance remaining: {:.2}",
            ((dest.x as f32 - actor.fpos_x).powi(2) + (dest.y as f32 - actor.fpos_y).powi(2)).sqrt());
        panic!("Actor did not reach destination");
    }

    // Validation 1: Check movement is direct (minimal deviation from straight line)
    println!("\n=== Validation: Movement Directness ===");

    let start_x = positions[0].0;
    let start_y = positions[0].1;
    let end_x = positions.last().unwrap().0;
    let end_y = positions.last().unwrap().1;

    let ideal_dir_x = end_x - start_x;
    let ideal_dir_y = end_y - start_y;
    let ideal_len = (ideal_dir_x * ideal_dir_x + ideal_dir_y * ideal_dir_y).sqrt();

    let mut max_deviation = 0.0_f32;
    let mut total_deviation = 0.0_f32;

    for (i, (px, py)) in positions.iter().enumerate().skip(1) {
        // Calculate perpendicular distance from point to ideal line
        // Using cross product: |cross| / |line_vec|
        let vec_to_point_x = px - start_x;
        let vec_to_point_y = py - start_y;

        let cross = ideal_dir_x * vec_to_point_y - ideal_dir_y * vec_to_point_x;
        let deviation = cross.abs() / ideal_len;

        max_deviation = max_deviation.max(deviation);
        total_deviation += deviation;

        if deviation > 0.5 {
            println!("  [Point {}] Position ({:.2}, {:.2}): deviation = {:.3}",
                i, px, py, deviation);
        }
    }

    let avg_deviation = total_deviation / (positions.len() - 1) as f32;

    println!("Max deviation from straight line: {:.3}", max_deviation);
    println!("Avg deviation from straight line: {:.3}", avg_deviation);

    if max_deviation > 1.0 {
        println!("✗ FAIL: Movement not direct (max deviation {:.3} > 1.0)", max_deviation);
        panic!("Movement deviated significantly from straight line");
    } else {
        println!("✓ PASS: Movement is direct (max deviation {:.3} <= 1.0)", max_deviation);
    }

    // Validation 2: Check subcell crossings are progressive (no backtracking)
    println!("\n=== Validation: Subcell Progression ===");
    println!("Total subcells visited: {}", subcells_visited.len());

    let mut backtracked = false;
    for i in 1..subcells_visited.len() {
        let prev = &subcells_visited[i-1];
        let curr = &subcells_visited[i];

        // Check if we moved closer to destination
        let prev_dist = ((dest.x as f32 - prev.cell_x as f32).powi(2) +
                        (dest.y as f32 - prev.cell_y as f32).powi(2)).sqrt();
        let curr_dist = ((dest.x as f32 - curr.cell_x as f32).powi(2) +
                        (dest.y as f32 - curr.cell_y as f32).powi(2)).sqrt();

        if curr_dist > prev_dist + 0.5 {
            println!("  ✗ Backtrack detected: ({},{}) → ({},{}), dist: {:.2} → {:.2}",
                prev.cell_x, prev.cell_y, curr.cell_x, curr.cell_y, prev_dist, curr_dist);
            backtracked = true;
        }
    }

    if backtracked {
        println!("✗ FAIL: Actor backtracked during movement");
        panic!("Actor moved away from destination");
    } else {
        println!("✓ PASS: No backtracking detected");
    }

    // Validation 3: Check actual path length vs ideal
    println!("\n=== Validation: Path Efficiency ===");

    let mut actual_distance = 0.0;
    for i in 1..positions.len() {
        let dx = positions[i].0 - positions[i-1].0;
        let dy = positions[i].1 - positions[i-1].1;
        actual_distance += (dx * dx + dy * dy).sqrt();
    }

    let ideal_distance = ideal_len;
    let efficiency = ideal_distance / actual_distance;

    println!("Ideal distance: {:.2}", ideal_distance);
    println!("Actual distance: {:.2}", actual_distance);
    println!("Efficiency: {:.2}% ({:.3})", efficiency * 100.0, efficiency);

    if efficiency < 0.95 {
        println!("✗ FAIL: Path inefficient (efficiency {:.2}% < 95%)", efficiency * 100.0);
        panic!("Actor took a very indirect path");
    } else {
        println!("✓ PASS: Path is efficient (efficiency {:.2}% >= 95%)", efficiency * 100.0);
    }

    println!("\n=== Test Complete: All Validations Passed ===\n");
}
