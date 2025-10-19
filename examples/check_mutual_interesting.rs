use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
use std::fs;

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== CHECKING MUTUAL INTERESTING CORNERS FOR C# PATH =====\n");
    println!("C# path: 1211 → 4056 → 4310 → 4396\n");

    let all_corners = detect_all_corners(&grid);

    // Check 1211 → 4056
    println!("1. From 1211: Is 4056 an interesting corner?");
    let (x1, y1) = grid.get_coords(1211);
    let visible_from_1211 = raycast(&grid, x1, y1, false, false);
    let interesting_from_1211 = filter_interesting_corners(
        &all_corners, &visible_from_1211, &grid, x1, y1, false
    );
    let has_4056 = interesting_from_1211.iter().any(|c| grid.get_id(c.x, c.y) == 4056);
    println!("   {} {}", if has_4056 { "✓" } else { "✗" }, if has_4056 { "YES" } else { "NO" });

    // Check 4056 → 4310
    println!("\n2. From 4056: Is 4310 an interesting corner?");
    let (x2, y2) = grid.get_coords(4056);
    let visible_from_4056 = raycast(&grid, x2, y2, false, false);
    let interesting_from_4056 = filter_interesting_corners(
        &all_corners, &visible_from_4056, &grid, x2, y2, false
    );
    let has_4310 = interesting_from_4056.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
    println!("   {} {}", if has_4310 { "✓" } else { "✗" }, if has_4310 { "YES" } else { "NO" });

    if !has_4310 {
        // Check if 4310 is at least visible
        let is_4310_visible = visible_from_4056.contains(&4310);
        println!("   Note: 4310 is {} from 4056", if is_4310_visible { "VISIBLE" } else { "NOT VISIBLE" });

        // Check if 4310 is in ALL visible corners (not just interesting)
        let all_visible_corners: Vec<_> = all_corners.iter()
            .filter(|corner| {
                let corner_id = grid.get_id(corner.x, corner.y);
                visible_from_4056.contains(&corner_id)
            })
            .collect();
        let in_all_visible = all_visible_corners.iter().any(|c| grid.get_id(c.x, c.y) == 4310);
        println!("   Note: 4310 is in ALL visible corners: {}", if in_all_visible { "YES" } else { "NO" });
    }

    // Check 4310 → 4396 (destination)
    println!("\n3. From 4310: Is 4396 (destination) an interesting corner?");
    let (x3, y3) = grid.get_coords(4310);
    let visible_from_4310 = raycast(&grid, x3, y3, false, false);
    let interesting_from_4310 = filter_interesting_corners(
        &all_corners, &visible_from_4310, &grid, x3, y3, false
    );
    let has_4396 = interesting_from_4310.iter().any(|c| grid.get_id(c.x, c.y) == 4396);
    println!("   {} {}", if has_4396 { "✓" } else { "✗" }, if has_4396 { "YES" } else { "NO" });

    let is_4396_visible = visible_from_4310.contains(&4396);
    println!("   Note: 4396 is {} from 4310", if is_4396_visible { "VISIBLE" } else { "NOT VISIBLE" });
    println!("   (4396 is the destination, not necessarily an interesting corner)");

    // Also check reverse direction: 4056 → 1211
    println!("\n4. Reverse check - From 4056: Is 1211 (start) an interesting corner?");
    let has_1211 = interesting_from_4056.iter().any(|c| grid.get_id(c.x, c.y) == 1211);
    println!("   {} {}", if has_1211 { "✓" } else { "✗" }, if has_1211 { "YES" } else { "NO" });

    // Check 4310 → 4056
    println!("\n5. Reverse check - From 4310: Is 4056 an interesting corner?");
    let has_4056_from_4310 = interesting_from_4310.iter().any(|c| grid.get_id(c.x, c.y) == 4056);
    println!("   {} {}", if has_4056_from_4310 { "✓" } else { "✗" }, if has_4056_from_4310 { "YES" } else { "NO" });

    println!("\n\n===== SUMMARY =====");
    println!("Mutual interesting corner relationships:");
    println!("  1211 → 4056: {}", if has_4056 { "✓ YES" } else { "✗ NO" });
    println!("  4056 → 4310: {}", if has_4310 { "✓ YES" } else { "✗ NO" });
    println!("  4310 → 4396: {} (dest)", if has_4396 { "✓ YES" } else { "✗ NO" });
    println!("\nReverse:");
    println!("  4056 → 1211: {} (start)", if has_1211 { "✓ YES" } else { "✗ NO" });
    println!("  4310 → 4056: {}", if has_4056_from_4310 { "✓ YES" } else { "✗ NO" });

    let all_interesting = has_4056 && has_4310;
    println!("\n{}", if all_interesting {
        "✓ All forward corners are mutually interesting"
    } else {
        "✗ NOT all forward corners are mutually interesting"
    });
}
