use rustgame3::{Grid, raycast, corners::{detect_all_corners, filter_interesting_corners}};
use std::fs;

#[test]
#[ignore]
fn debug_corner_4056_and_4310() {
    // Load grid
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    let grid = Grid::with_blocked(83, 104, &blocked);

    // Start position
    let start_id = 4396;
    let (start_x, start_y) = grid.get_coords(start_id);
    println!("Start: ID {} = ({}, {})", start_id, start_x, start_y);

    // Expected waypoints
    let wp1_id = 4056;
    let wp2_id = 4310;
    let (wp1_x, wp1_y) = grid.get_coords(wp1_id);
    let (wp2_x, wp2_y) = grid.get_coords(wp2_id);
    println!("Expected WP1: ID {} = ({}, {})", wp1_id, wp1_x, wp1_y);
    println!("Expected WP2: ID {} = ({}, {})", wp2_id, wp2_x, wp2_y);

    // Detect all corners
    let all_corners = detect_all_corners(&grid);
    println!("\nTotal corners in grid: {}", all_corners.len());

    // Check if wp1 and wp2 are corners
    let wp1_is_corner = all_corners.iter().any(|c| c.x == wp1_x && c.y == wp1_y);
    let wp2_is_corner = all_corners.iter().any(|c| c.x == wp2_x && c.y == wp2_y);
    println!("WP1 ({},{}) is a corner? {}", wp1_x, wp1_y, wp1_is_corner);
    println!("WP2 ({},{}) is a corner? {}", wp2_x, wp2_y, wp2_is_corner);

    if wp1_is_corner {
        let corner = all_corners.iter().find(|c| c.x == wp1_x && c.y == wp1_y).unwrap();
        println!("WP1 corner directions: {:?}", corner.directions);
    }
    if wp2_is_corner {
        let corner = all_corners.iter().find(|c| c.x == wp2_x && c.y == wp2_y).unwrap();
        println!("WP2 corner directions: {:?}", corner.directions);
    }

    // Raycast from start
    let visible_cells = raycast(&grid, start_x, start_y, false, false);
    println!("\nVisible cells from start: {}", visible_cells.len());

    // Check if wp1 and wp2 are visible
    let wp1_visible = visible_cells.contains(&wp1_id);
    let wp2_visible = visible_cells.contains(&wp2_id);
    println!("WP1 ({},{}) visible from start? {}", wp1_x, wp1_y, wp1_visible);
    println!("WP2 ({},{}) visible from start? {}", wp2_x, wp2_y, wp2_visible);

    // Filter interesting corners
    let interesting = filter_interesting_corners(&all_corners, &visible_cells, &grid, start_x, start_y, false);
    println!("\nInteresting corners from start: {}", interesting.len());

    let wp1_interesting = interesting.iter().any(|c| c.x == wp1_x && c.y == wp1_y);
    let wp2_interesting = interesting.iter().any(|c| c.x == wp2_x && c.y == wp2_y);
    println!("WP1 ({},{}) is interesting? {}", wp1_x, wp1_y, wp1_interesting);
    println!("WP2 ({},{}) is interesting? {}", wp2_x, wp2_y, wp2_interesting);
}
