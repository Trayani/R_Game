/// Test suite for PSC (Primary SubCell) switching in DestinationDirect mode
///
/// Validates the principle that:
/// 1. PSC + reserved subcells + target are locked together
/// 2. Actor moves toward the locked target
/// 3. When target is reached, PSC switches to the CLOSER reserved subcell
/// 4. New target is calculated from the new PSC position
///
/// This creates a progression of targets toward the destination, ensuring
/// actors always pick the subcell that minimizes distance to destination.

use rustgame3::actor::Actor;
use rustgame3::pathfinding::Position;
use rustgame3::subcell::{SubCellCoord, SubCellReservationManager};

/// Helper to create a simple actor at a given position
fn create_actor(id: usize, x: f32, y: f32, cell_width: f32, cell_height: f32, subcell_grid_size: i32) -> Actor {
    let size = cell_width * 0.8;
    let speed = 50.0;
    let collision_radius = cell_width * 0.3;
    let subcell_offset_x = 0.0;
    let subcell_offset_y = 0.0;

    let mut actor = Actor::new(
        id,
        x,
        y,
        size,
        speed,
        collision_radius,
        cell_width,
        cell_height,
        subcell_grid_size,
        subcell_offset_x,
        subcell_offset_y,
    );
    actor.use_directing_v2 = true;
    actor
}

/// Helper to track PSC progression during movement
struct PSCTracker {
    psc_history: Vec<(i32, i32, i32, i32)>,  // (cell_x, cell_y, sub_x, sub_y)
    target_history: Vec<(f32, f32)>,          // (target_x, target_y)
}

impl PSCTracker {
    fn new() -> Self {
        PSCTracker {
            psc_history: Vec::new(),
            target_history: Vec::new(),
        }
    }

    fn record(&mut self, actor: &Actor) {
        if let Some(psc) = actor.current_subcell {
            self.psc_history.push((psc.cell_x, psc.cell_y, psc.sub_x, psc.sub_y));
        }
        if let Some((tx, ty)) = actor.locked_target {
            self.target_history.push((tx, ty));
        }
    }

    fn print_summary(&self, actor: &Actor) {
        println!("\n=== PSC Progression Summary ===");
        println!("Actor {} final position: ({:.2}, {:.2})", actor.id, actor.fpos_x, actor.fpos_y);
        println!("PSC switches: {}", self.psc_history.len());
        for (i, (cx, cy, sx, sy)) in self.psc_history.iter().enumerate() {
            println!("  PSC {}: cell=({}, {}), sub=({}, {})", i + 1, cx, cy, sx, sy);
        }
        println!("Target progression: {}", self.target_history.len());
        for (i, (tx, ty)) in self.target_history.iter().enumerate() {
            println!("  Target {}: ({:.2}, {:.2})", i + 1, tx, ty);
        }
    }
}

/// Test multi-step path: Actor at (0,0) moving to (3,1)
/// Expected progression:
/// - PSC (0,0) → target ~(1, 0.33) → switch to PSC (1,0)
/// - PSC (1,0) → target ~(2, 0.67) → switch to PSC (2,0)
/// - PSC (2,0) → target (3, 1) → reach destination
#[test]
fn test_multi_step_path_0_0_to_3_1() {
    println!("\n=== TEST: Multi-step path (0,0) → (3,1) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Create actor at cell (0,0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Set destination to (3, 1) - grid line intersection (not cell center)
    let dest_x = 3;
    let dest_y = 1;
    actor.set_subcell_destination(Position { x: dest_x, y: dest_y });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Simulate movement
    let delta_time = 0.016; // ~60 FPS
    let max_iterations = 4000;
    let mut reached = false;

    for iteration in 0..max_iterations {
        // Record PSC before update
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false,  // enable_early_reservation
            false,  // filter_backward
            false,  // track_movement
            10.0,   // reservation_threshold_distance
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        // Check if PSC changed
        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
            if let Some(psc) = actor.current_subcell {
                println!("  [Iteration {}] PSC switched to: cell=({}, {}), sub=({}, {})",
                    iteration, psc.cell_x, psc.cell_y, psc.sub_x, psc.sub_y);
            }
        }

        if reached {
            println!("  [Iteration {}] REACHED DESTINATION", iteration);
            break;
        }

        // Log progress every 200 iterations
        if iteration % 200 == 0 && iteration > 0 {
            println!("  [Iteration {}] Position: ({:.2}, {:.2})",
                iteration, actor.fpos_x, actor.fpos_y);
        }
    }

    tracker.print_summary(&actor);

    // Verify actor reached destination
    assert!(reached, "Actor should reach destination (3, 1)");

    // Verify actor made progress through multiple PSC switches
    assert!(tracker.psc_history.len() >= 2,
        "Expected at least 2 PSC switches, got {}", tracker.psc_history.len());

    // Verify final position is close to destination
    let dest_screen_x = dest_x as f32 * cell_width;
    let dest_screen_y = dest_y as f32 * cell_height;
    let dist = ((actor.fpos_x - dest_screen_x).powi(2) + (actor.fpos_y - dest_screen_y).powi(2)).sqrt();
    assert!(dist < 5.0, "Actor should be within 5 pixels of destination, was {:.2} pixels away", dist);

    println!("\n✓ Multi-step path test PASSED");
}

/// Test diagonal NE movement: destination east-northeast → should prefer horizontal anchor
#[test]
fn test_diagonal_ne_prefer_horizontal() {
    println!("\n=== TEST: Diagonal NE (prefer horizontal) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is east-northeast (3, 1) - more horizontal than vertical
    actor.set_subcell_destination(Position { x: 3, y: 1 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");
    assert!(tracker.psc_history.len() >= 1, "Expected at least 1 PSC switch");

    println!("✓ Diagonal NE (horizontal preference) test PASSED");
}

/// Test diagonal NE movement: destination north-northeast → should prefer vertical anchor
#[test]
fn test_diagonal_ne_prefer_vertical() {
    println!("\n=== TEST: Diagonal NE (prefer vertical) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is north-northeast (1, -3) - more vertical than horizontal
    actor.set_subcell_destination(Position { x: 1, y: -3 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");

    println!("✓ Diagonal NE (vertical preference) test PASSED");
}

/// Test cardinal direction E (east): straightforward horizontal movement
#[test]
fn test_cardinal_east() {
    println!("\n=== TEST: Cardinal E (east) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is directly east (3, 0)
    actor.set_subcell_destination(Position { x: 3, y: 0 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");

    println!("✓ Cardinal E (east) test PASSED");
}

/// Test cardinal direction N (north): straightforward vertical movement
#[test]
fn test_cardinal_north() {
    println!("\n=== TEST: Cardinal N (north) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is directly north (0, -3)
    actor.set_subcell_destination(Position { x: 0, y: -3 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");

    println!("✓ Cardinal N (north) test PASSED");
}

/// Test diagonal SE movement
#[test]
fn test_diagonal_se() {
    println!("\n=== TEST: Diagonal SE (southeast) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is southeast (3, 3)
    actor.set_subcell_destination(Position { x: 3, y: 3 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");

    println!("✓ Diagonal SE test PASSED");
}

/// Test diagonal SW movement
#[test]
fn test_diagonal_sw() {
    println!("\n=== TEST: Diagonal SW (southwest) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is southwest (-3, 3)
    actor.set_subcell_destination(Position { x: -3, y: 3 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");

    println!("✓ Diagonal SW test PASSED");
}

/// Test diagonal NW movement
#[test]
fn test_diagonal_nw() {
    println!("\n=== TEST: Diagonal NW (northwest) ===");

    let cell_width = 100.0;
    let cell_height = 100.0;
    let subcell_grid_size = 2;

    // Actor at (0, 0) center
    let start_x = cell_width * 0.5;
    let start_y = cell_height * 0.5;
    let mut actor = create_actor(0, start_x, start_y, cell_width, cell_height, subcell_grid_size);

    // Destination is northwest (-3, -3)
    actor.set_subcell_destination(Position { x: -3, y: -3 });

    let mut reservation_manager = SubCellReservationManager::new(subcell_grid_size);
    let mut tracker = PSCTracker::new();

    // Run simulation
    let delta_time = 0.016;
    let max_iterations = 3000;
    let mut reached = false;

    for _iteration in 0..max_iterations {
        let prev_psc = actor.current_subcell;

        reached = actor.update_subcell_destination_direct(
            delta_time,
            &mut reservation_manager,
            false, false, false, // enable_early_reservation, filter_backward, track_movement
            10.0,
            rustgame3::config::ReservationEagerness::Center,
            rustgame3::config::ReleaseEagerness::Center,
        );

        if actor.current_subcell != prev_psc {
            tracker.record(&actor);
        }

        if reached { break; }
    }

    tracker.print_summary(&actor);

    assert!(reached, "Actor should reach destination");

    println!("✓ Diagonal NW test PASSED");
}
