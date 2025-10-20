/// Test that verifies destinations are reachable even when not "interesting"
///
/// This test documents the important distinction between:
/// 1. "Interesting corners" - corners that lead to unexplored areas (for exploration)
/// 2. "Destination corners" - corners that are the pathfinding goal (must always be reachable)
///
/// A corner can be visible and NOT interesting (both cardinal directions visible),
/// but if it's the destination, it must still be included in the path.

use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use rustgame3::pathfinding::find_path;

#[test]
fn test_destination_reachable_even_if_not_interesting() {
    // Setup grid with the scenario from the bug report
    let grid_str = r#"o□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
oo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
ooo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
oooo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
ooooo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□
oooooo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□o
ooooooo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□□oo
oooooooo□□□□□□□□□□□□□□□□□□□□□□□□□□□□□ooo
ooooooooo□□□□□□□□□□□□□□□□□□□□□□□□□□ooooo
oooooooooo□□□□□□□□□□□□□□□□□□□□□□□□oooooo
ooooooooooo□□□□□□□□□□□□□□□□□□□□□□ooooooo
oooooooooooo□□□□□□□□□□□■□□□□□□□□oooooooo
ooooooooooooo□□□□□□■□□□□□□□□□□oooooooooo
oooooooooooooo□■□□□□□■■□□□■□□ooooooooooo
ooooooooooooooooo□■□■□o■□□□□oooooooooooo
oooooooooooooooooooooooo■□□ooooooooooooo
oooooooooooooooooooooooooooooooooooooooo
oooooooooooooooooooooooooooooooooooooooo
oooooooooooooooooooooooooooooooooooooooo
oooooooooooooooooooooooooooooooooooooooo
oooooooooooooooooooosooooooooooooooooooo
oooooooooooooooooooooooooooooooooooooooo
ooooooooooooooooooooooooo■□□□□□□oooooooo
oooo□□□□□□■oooooooooooooo□□□□□□□□□□□□□oo
□□□□□□□□□□□ooooooo□■oooooo□□□□□□□□□□□□□□
□□□□□□□□□oooooooo□□□ooooooo□□□□□□□□□□□□□
□□□□□□□oooooooooo□□□ooooooooo□□□□□□□□□□□
□□□□□ooooooooooo□□□□oooooooooo□□□□□□□□□□
□□ooooooooooooo□□□□□ooooooooooo□□□□□□□□□
ooooooooooooooo□□□□□ooooooooooooo□□□□□□□
oooooooooooo□■□□□□□□oooooooooooooo□□□□□□
ooooooooooo□□□□□□□□□ooooooooooooooo□□□□□
oooooooooo□□□□□□□□□□ooooooooooooooooo□□□
ooooooooo□□□□□□□□□□□oooooooooooooooooo□□
oooooooo□□□□□□□□□□□□ooooooooooooooooooo□
ooooooo□□□□□□□□□□□□□oooooooooooooooooooo
oooooo□□□□□□□□□□■□□□oooooooooooooooooooo
ooooo□□□□□□□□□□□□□□□oooooooooooooooooooo
ooooo□□□□□□□□□□□□□□□oooooooooooooooooooo
oooo□□□□□□□□□□□□□□□□oooooooooooooooooooo"#;

    let lines: Vec<&str> = grid_str.lines().collect();
    let rows = lines.len();
    let cols = lines[0].chars().count();

    let mut grid = Grid::new(cols as i32, rows as i32);
    let observer = (20, 20);
    let destination = (24, 12); // Cell 504

    // Parse grid
    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == '■' || ch == 'b' {
                let id = grid.get_id(x as i32, y as i32);
                grid.cells[id as usize] = 1;
            }
        }
    }

    // Verify the scenario: from waypoint 585, destination 504 is visible but not interesting
    let waypoint_585 = (25, 14);
    let visible_from_585 = raycast(&grid, waypoint_585.0, waypoint_585.1, false, false);
    let all_corners = detect_all_corners(&grid);
    let interesting_from_585 = filter_interesting_corners(
        &all_corners,
        &visible_from_585,
        &grid,
        waypoint_585.0,
        waypoint_585.1,
        false,
    );

    // Assert: 504 IS visible from 585
    assert!(
        visible_from_585.contains(&grid.get_id(destination.0, destination.1)),
        "Destination 504 should be visible from waypoint 585"
    );

    // Assert: 504 is NOT in interesting corners list (this is the key insight)
    let is_interesting = interesting_from_585
        .iter()
        .any(|c| c.x == destination.0 && c.y == destination.1);
    assert!(
        !is_interesting,
        "Destination 504 should NOT be interesting from 585 (both cardinal directions visible)"
    );

    // The critical test: pathfinding should still find the direct path
    let path = find_path(
        &grid,
        observer.0,
        observer.1,
        destination.0,
        destination.1,
        false,
        false,
    )
    .expect("Path should exist");

    // Convert path to IDs for easier verification
    let path_ids: Vec<i32> = path.iter().map(|p| grid.get_id(p.x, p.y)).collect();

    // Assert: Path should NOT contain 503 (the unnecessary waypoint)
    assert!(
        !path_ids.contains(&503),
        "Path should not contain cell 503 (23,12) - Expected: 820->665->585->504, Got: {:?}",
        path_ids
    );

    // Assert: Path should go directly from 585 to 504
    let has_direct_585_to_504 = path_ids.windows(2).any(|w| w[0] == 585 && w[1] == 504);
    assert!(
        has_direct_585_to_504,
        "Path should go directly from 585 to 504. Path IDs: {:?}",
        path_ids
    );

    // Expected optimal path: 820 (observer) -> 665 -> 585 -> 504 (destination)
    assert_eq!(
        path_ids,
        vec![820, 665, 585, 504],
        "Expected optimal path without unnecessary waypoint 503"
    );
}
