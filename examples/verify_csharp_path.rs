use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== VERIFYING C# PATH SEGMENTS IN RUST =====\n");

    // C# path: 1211 -> 4310 -> 4056 -> 4396
    let path = vec![1211, 4310, 4056, 4396];

    println!("C# path: {:?}\n", path);

    for &id in &path {
        let (x, y) = grid.get_coords(id);
        println!("  {} at ({}, {})", id, x, y);
    }

    println!("\n===== VISIBILITY CHECK =====\n");

    // Check 1211 -> 4310
    println!("1. Does 1211 see 4310?");
    let (x1, y1) = grid.get_coords(1211);
    let visible_from_1211 = raycast(&grid, x1, y1, false, false);
    let sees_4310 = visible_from_1211.contains(&4310);
    println!("   {} {}", if sees_4310 { "✓" } else { "✗" }, if sees_4310 { "YES" } else { "NO" });

    // Check 4310 -> 4056
    println!("\n2. Does 4310 see 4056?");
    let (x2, y2) = grid.get_coords(4310);
    let visible_from_4310 = raycast(&grid, x2, y2, false, false);
    let sees_4056 = visible_from_4310.contains(&4056);
    println!("   {} {}", if sees_4056 { "✓" } else { "✗" }, if sees_4056 { "YES" } else { "NO" });

    // Check 4056 -> 4396
    println!("\n3. Does 4056 see 4396?");
    let (x3, y3) = grid.get_coords(4056);
    let visible_from_4056 = raycast(&grid, x3, y3, false, false);
    let sees_4396 = visible_from_4056.contains(&4396);
    println!("   {} {}", if sees_4396 { "✓" } else { "✗" }, if sees_4396 { "YES" } else { "NO" });

    println!("\n===== CORNER DISCOVERY CHECK =====\n");

    let all_corners = detect_all_corners(&grid);

    // Check if 4310 and 4056 are corners at all
    let is_4310_corner = all_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    let is_4056_corner = all_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4056);

    println!("Is 4310 a corner? {}", if is_4310_corner { "YES" } else { "NO" });
    println!("Is 4056 a corner? {}", if is_4056_corner { "YES" } else { "NO" });

    // Check if they're in interesting corners from 1211
    println!("\n4. Is 4310 discovered as interesting corner from 1211?");
    let interesting_from_1211 = filter_interesting_corners(
        &all_corners, &visible_from_1211, &grid, x1, y1, false
    );
    let is_4310_interesting = interesting_from_1211.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    println!("   {} {}", if is_4310_interesting { "✓" } else { "✗" }, if is_4310_interesting { "YES" } else { "NO" });

    println!("\n5. Is 4056 discovered as interesting corner from 1211?");
    let is_4056_interesting = interesting_from_1211.iter().any(|c| grid.get_id(c.x, c.y) == 4056);
    println!("   {} {}", if is_4056_interesting { "✓" } else { "✗" }, if is_4056_interesting { "YES" } else { "NO" });

    // Check if 4056 is in interesting corners from 4310
    println!("\n6. Is 4056 discovered as interesting corner from 4310?");
    let interesting_from_4310 = filter_interesting_corners(
        &all_corners, &visible_from_4310, &grid, x2, y2, false
    );
    let is_4056_from_4310 = interesting_from_4310.iter().any(|c| grid.get_id(c.x, c.y) == 4056);
    println!("   {} {}", if is_4056_from_4310 { "✓" } else { "✗" }, if is_4056_from_4310 { "YES" } else { "NO" });

    // Check if 4396 is in interesting corners from 4056
    println!("\n7. Is 4396 discovered as interesting corner from 4056?");
    let interesting_from_4056 = filter_interesting_corners(
        &all_corners, &visible_from_4056, &grid, x3, y3, false
    );
    let is_4396_from_4056 = interesting_from_4056.iter().any(|c| grid.get_id(c.x, c.y) == 4396);
    println!("   {} {}", if is_4396_from_4056 { "✓" } else { "✗" }, if is_4396_from_4056 { "YES" } else { "NO" });

    println!("\n===== SUMMARY =====");
    println!("All visibility checks: {}", if sees_4310 && sees_4056 && sees_4396 { "✓ PASS" } else { "✗ FAIL" });
    println!("All corners discovered: {}", if is_4310_interesting && is_4056_interesting && is_4056_from_4310 { "✓ PASS" } else { "✗ FAIL" });
}
