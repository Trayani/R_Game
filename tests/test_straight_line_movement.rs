// Comprehensive test suite for straight-line movement
// Tests various diagonal ratios to ensure actors always move directly toward destinations
// without zigzagging, backtracking, or inefficient paths

use rustgame3::Actor;
use rustgame3::SubPointReservationManager;
use rustgame3::subcell::{SubCellCoord, };
use rustgame3::pathfinding::Position;
use rustgame3::config::{ReservationEagerness, ReleaseEagerness};

/// Test parameters for a movement scenario
struct MovementTest {
    name: &'static str,
    dx_cells: i32,
    dy_cells: i32,
    min_efficiency: f32,
}

/// Helper function to test movement in a specific direction
fn test_movement_direction(test: &MovementTest) {
    println!("\n=== Testing {} ===", test.name);

    let cell_width = 32.0;
    let cell_height = 32.0;

    // Spawn at fixed position
    let spawn_x = 64.0;  // Cell (2, 2)
    let spawn_y = 64.0;
    let spawn_cell_x = (spawn_x / cell_width) as i32;
    let spawn_cell_y = (spawn_y / cell_height) as i32;

    // Destination is offset by dx_cells, dy_cells
    let dest_cell_x = spawn_cell_x + test.dx_cells;
    let dest_cell_y = spawn_cell_y + test.dy_cells;

    let dest_x = dest_cell_x as f32 * cell_width;
    let dest_y = dest_cell_y as f32 * cell_height;

    println!("Spawn: ({:.1}, {:.1}) -> Destination: ({:.1}, {:.1})",
        spawn_x, spawn_y, dest_x, dest_y);
    println!("Offset: {}X {}Y ({:.2} cells)",
        test.dx_cells, test.dy_cells,
        ((test.dx_cells * test.dx_cells + test.dy_cells * test.dy_cells) as f32).sqrt());

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

    // Set destination
    let dest = Position { x: dest_cell_x, y: dest_cell_y };
    actor.set_subcell_destination(dest);

    let mut reservation_mgr = SubPointReservationManager::new(2);
    reservation_mgr.set_current(start_subcell, 0);

    // Simulation parameters
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut iteration = 0;
    let mut reached = false;

    // Movement tracking
    let mut total_backwards_x = 0;
    let mut total_backwards_y = 0;
    let mut position_history = Vec::new();
    position_history.push((spawn_x, spawn_y));

    // Determine expected movement directions
    let expect_positive_x = test.dx_cells > 0;
    let expect_positive_y = test.dy_cells > 0;

    // Run simulation
    while iteration < max_iterations && !reached {
        let old_x = actor.fpos_x;
        let old_y = actor.fpos_y;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_mgr,
            false, // enable_early_reservation
            false, // filter_backward
            false, // enable_anti_cross
            false, // track_movement (disable for cleaner output)
            0.0,
            ReservationEagerness::Center,
            ReleaseEagerness::Center,
        );

        iteration += 1;
        position_history.push((actor.fpos_x, actor.fpos_y));

        // Check for backwards movement
        let dx_move = actor.fpos_x - old_x;
        let dy_move = actor.fpos_y - old_y;

        if test.dx_cells != 0 {
            if expect_positive_x && dx_move < -0.001 {
                total_backwards_x += 1;
            } else if !expect_positive_x && dx_move > 0.001 {
                total_backwards_x += 1;
            }
        }

        if test.dy_cells != 0 {
            if expect_positive_y && dy_move < -0.001 {
                total_backwards_y += 1;
            } else if !expect_positive_y && dy_move > 0.001 {
                total_backwards_y += 1;
            }
        }

        if reached {
            println!("✓ REACHED after {} iterations ({:.1}s)", iteration, iteration as f32 * delta_time);
            break;
        }
    }

    if !reached {
        println!("✗ DID NOT REACH after {} iterations", max_iterations);
    }

    // Analyze path efficiency
    let start_x = position_history[0].0;
    let start_y = position_history[0].1;
    let end_x = position_history[position_history.len() - 1].0;
    let end_y = position_history[position_history.len() - 1].1;

    let total_dx = end_x - start_x;
    let total_dy = end_y - start_y;

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
    println!("Backwards movements: X={}, Y={}", total_backwards_x, total_backwards_y);

    // Assertions
    assert!(reached, "{}: Actor should reach destination", test.name);
    assert_eq!(total_backwards_x, 0, "{}: No backwards X movement allowed", test.name);
    assert_eq!(total_backwards_y, 0, "{}: No backwards Y movement allowed", test.name);
    assert!(efficiency >= test.min_efficiency,
        "{}: Path efficiency should be >= {:.1}% (was {:.1}%)",
        test.name, test.min_efficiency, efficiency);

    println!("✓ {} PASSED\n", test.name);
}

#[test]
fn test_diagonal_45_degrees() {
    test_movement_direction(&MovementTest {
        name: "Diagonal 45° (+8X +8Y)",
        dx_cells: 8,
        dy_cells: 8,
        min_efficiency: 99.0,
    });
}

#[test]
fn test_horizontal_bias_strong() {
    test_movement_direction(&MovementTest {
        name: "Horizontal-biased (+10X +3Y)",
        dx_cells: 10,
        dy_cells: 3,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_vertical_bias_strong() {
    test_movement_direction(&MovementTest {
        name: "Vertical-biased (+3X +10Y)",
        dx_cells: 3,
        dy_cells: 10,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_extreme_horizontal() {
    test_movement_direction(&MovementTest {
        name: "Extreme horizontal (+15X +1Y)",
        dx_cells: 15,
        dy_cells: 1,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_extreme_vertical() {
    test_movement_direction(&MovementTest {
        name: "Extreme vertical (+1X +15Y)",
        dx_cells: 1,
        dy_cells: 15,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_original_case() {
    test_movement_direction(&MovementTest {
        name: "Original case (+5X +13Y)",
        dx_cells: 5,
        dy_cells: 13,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_negative_diagonal() {
    test_movement_direction(&MovementTest {
        name: "Negative diagonal (-5X -5Y)",
        dx_cells: -5,
        dy_cells: -5,
        min_efficiency: 99.0,
    });
}

#[test]
fn test_mixed_positive_x_negative_y() {
    test_movement_direction(&MovementTest {
        name: "Mixed (+8X -8Y)",
        dx_cells: 8,
        dy_cells: -8,
        min_efficiency: 99.0,
    });
}

#[test]
fn test_mixed_negative_x_positive_y() {
    test_movement_direction(&MovementTest {
        name: "Mixed (-8X +8Y)",
        dx_cells: -8,
        dy_cells: 8,
        min_efficiency: 99.0,
    });
}

#[test]
fn test_shallow_angle() {
    test_movement_direction(&MovementTest {
        name: "Shallow angle (+12X +2Y)",
        dx_cells: 12,
        dy_cells: 2,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_steep_angle() {
    test_movement_direction(&MovementTest {
        name: "Steep angle (+2X +12Y)",
        dx_cells: 2,
        dy_cells: 12,
        min_efficiency: 95.0,
    });
}

#[test]
fn test_medium_diagonal() {
    test_movement_direction(&MovementTest {
        name: "Medium diagonal (+6X +9Y)",
        dx_cells: 6,
        dy_cells: 9,
        min_efficiency: 95.0,
    });
}
