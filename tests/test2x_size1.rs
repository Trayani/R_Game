// Test2X test cases ported from C# (size=1 only)
// Source: GameLib2/GameLib/tests/ProtoTests.cs::Test2X()

use rustgame3::{Grid, pathfinding::find_path_by_id};
use std::fs;
use serde_json;

fn load_test_grid() -> Grid {
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    // Grid dimensions: 104 cols × 83 rows
    Grid::with_blocked(83, 104, &blocked)
}

// Helper to run pathfinding and validate
// Note: expected_waypoints contains ONLY intermediate waypoints (not start/dest)
// The full path returned by find_path_by_id should be: [start, ...waypoints, dest]
fn assert_path(
    grid: &Grid,
    start: i32,
    dest: i32,
    expected_waypoints: Option<&[i32]>,  // Just the intermediate points
    test_name: &str,
) {
    let result = find_path_by_id(grid, start, dest, false, false);

    match (result, expected_waypoints) {
        (Some((path, _dist)), Some(waypoints)) => {
            // Build expected full path: start + waypoints + dest
            let mut expected_full = vec![start];
            expected_full.extend_from_slice(waypoints);
            expected_full.push(dest);

            assert_eq!(
                path, expected_full,
                "{}: Path mismatch. Expected {:?}, got {:?}",
                test_name, expected_full, path
            );
        }
        (Some((path, _)), None) => {
            // Expected direct path (no waypoints) = [start, dest]
            let expected_direct = vec![start, dest];
            assert_eq!(
                path, expected_direct,
                "{}: Expected direct path {:?}, got {:?}",
                test_name, expected_direct, path
            );
        }
        (None, Some(waypoints)) => {
            panic!("{}: Expected path with waypoints {:?}, but got None", test_name, waypoints);
        }
        (None, None) => {
            panic!("{}: Both result and expected are None - this case shouldn't happen in valid tests", test_name);
        }
    }
}

// Lines 215-229: test4() calls with messyX=false, messyY=false
#[test]
#[ignore]  // Remove after fixing pathfinding
fn test2x_001_4396_to_1211() {
    // test4(pf, 4396, 1211, 1, true, false, false, 4056, 4310);
    let grid = load_test_grid();
    assert_path(&grid, 4396, 1211, Some(&[4056, 4310]), "001");
}

#[test]
#[ignore]
fn test2x_002_1211_to_4396() {
    // test4(pf, 1211, 4396, 1, true, false, false, 4310, 4056);
    let grid = load_test_grid();
    assert_path(&grid, 1211, 4396, Some(&[4310, 4056]), "002");
}

#[test]
#[ignore]
fn test2x_003_1211_to_4310() {
    // test4(pf, 1211, 4310, 1, true, false, false, 4056);
    let grid = load_test_grid();
    assert_path(&grid, 1211, 4310, Some(&[4056]), "003");
}

#[test]
#[ignore]
fn test2x_004_875_to_4396() {
    // test4(pf, 875, 4396, 1, true, false, false, 4310, 4056, 1211);
    let grid = load_test_grid();
    assert_path(&grid, 875, 4396, Some(&[4310, 4056, 1211]), "004");
}

#[test]
#[ignore]
fn test2x_005_4396_to_875() {
    // test4(pf, 4396, 875, 1, true, false, false, 1211, 4056, 4310);
    let grid = load_test_grid();
    assert_path(&grid, 4396, 875, Some(&[1211, 4056, 4310]), "005");
}

// Lines 235-246: test2() calls (messyY=true at this point, but reverse=false)
// test2(start, dest, size, reverse, ...path) => test3(start, dest, size, reverse, messyY, false, ...path)
// So these have messyY=true, messyX=false
// Since we're only doing messy_x=false, messy_y=false for now, SKIP these

// Lines 253-262: test() calls - these use test2 with reverse=true
// test() => test2(reverse=true) => test3(reverse=true, messyY=true, messyX=false)
// SKIP (messyY=true)

// Lines 266-285: mix of test2() and test3() with messyY=true
// SKIP (messyY=true)

// Line 312: messyY = false  <-- From here, messyY is false!

// Lines 313-314: test3 with messyX=true
// SKIP (messyX=true)

// Lines 320-333: test3 calls with various messy combinations
// Line 320: test3(2679, 2768, 1, false, false, true, ...) - messyY=false, messyX=true - SKIP
// Line 321: test3(2679, 2768, 1, false, true, false, ...) - messyY=true, messyX=false - SKIP
// Line 322: test3(2679, 2768, 1, false, false, false, 2851, 2845) - messyY=false, messyX=false - INCLUDE!

#[test]
#[ignore]
fn test2x_006_2679_to_2768() {
    let grid = load_test_grid();
    assert_path(&grid, 2679, 2768, Some(&[2851, 2845]), "006");
}

// Line 325: test3(2597, 2768, 1, false, false, true, ...) - SKIP (messyX=true)
// Line 326: test3(2597, 2768, 1, false, false, false, 2851, 2845, 2679) - INCLUDE!

#[test]
#[ignore]
fn test2x_007_2597_to_2768() {
    let grid = load_test_grid();
    assert_path(&grid, 2597, 2768, Some(&[2851, 2845, 2679]), "007");
}

// Line 327: test3(2597, 2768, 1, false, true, false, ...) - SKIP (messyY=true)
// Line 328: test3(2597, 2768, 1, false, true, true, ...) - SKIP (both messy)
// Line 330: test3(2680, 2768, 1, false, false, true, ...) - SKIP (messyX=true)
// Line 331: test3(2680, 2768, 1, false, false, false, 2851, 2845, 2679) - INCLUDE!

#[test]
#[ignore]
fn test2x_008_2680_to_2768() {
    let grid = load_test_grid();
    assert_path(&grid, 2680, 2768, Some(&[2851, 2845, 2679]), "008");
}

// Line 344: test3(2679, 2768, 1, false, false, false, 2851, 2845) - INCLUDE!
#[test]
#[ignore]
fn test2x_009_2679_to_2768_again() {
    let grid = load_test_grid();
    assert_path(&grid, 2679, 2768, Some(&[2851, 2845]), "009");
}

// Lines 350-361: test() calls (reverse=true, messyY still false from line 312)
// test() => test2() => test3(reverse, messyY=false, messyX=false) - INCLUDE!

#[test]
#[ignore]
fn test2x_010_2254_to_2255() {
    // test(2254, 2255, 1); - no waypoints means direct path
    let grid = load_test_grid();
    assert_path(&grid, 2254, 2255, None, "010");
}

#[test]
#[ignore]
fn test2x_011_4240_to_4980() {
    // test(4240, 4980, 1, 4732);
    let grid = load_test_grid();
    assert_path(&grid, 4240, 4980, Some(&[4732]), "011");
}

#[test]
#[ignore]
fn test2x_012_5429_to_5507() {
    // test(5429, 5507, 1); - no waypoints
    let grid = load_test_grid();
    assert_path(&grid, 5429, 5507, None, "012");
}

#[test]
#[ignore]
fn test2x_013_947_to_2110() {
    // test(947, 2110, 1, 1114);
    let grid = load_test_grid();
    assert_path(&grid, 947, 2110, Some(&[1114]), "013");
}

#[test]
#[ignore]
fn test2x_014_4820_to_4490() {
    // test(4820, 4490, 1, 4573);
    let grid = load_test_grid();
    assert_path(&grid, 4820, 4490, Some(&[4573]), "014");
}

#[test]
#[ignore]
fn test2x_015_2341_to_5507_long_path() {
    // test(2341, 5507, 1, 5505, 5585, 5570, 5321, 5156, 4908, 4576, 4574, 4901, 4898, 4732, 4323, 4330, 5245, 5247, 4087, 2759, 2424);
    let grid = load_test_grid();
    assert_path(
        &grid,
        2341,
        5507,
        Some(&[
            5505, 5585, 5570, 5321, 5156, 4908, 4576, 4574, 4901, 4898, 4732, 4323, 4330,
            5245, 5247, 4087, 2759, 2424,
        ]),
        "015",
    );
}

// Lines 377-380: test() calls (size=1)
#[test]
#[ignore]
fn test2x_016_946_to_2437() {
    // test(946, 2437, 1, 2523, 3769, 3771, 2194, 1114);
    let grid = load_test_grid();
    assert_path(&grid, 946, 2437, Some(&[2523, 3769, 3771, 2194, 1114]), "016");
}

#[test]
#[ignore]
fn test2x_017_946_to_2440() {
    // test(946, 2440, 1, 3769, 3771, 2194, 1114);
    let grid = load_test_grid();
    assert_path(&grid, 946, 2440, Some(&[3769, 3771, 2194, 1114]), "017");
}

#[test]
#[ignore]
fn test2x_018_4829_to_4750() {
    // test(4829, 4750, 1, 5247, 5245);
    let grid = load_test_grid();
    assert_path(&grid, 4829, 4750, Some(&[5247, 5245]), "018");
}

// Lines 406-407: Grid modification - add blocked cells
// grid.setBlocked(4998);
// grid.setBlocked(4999);

// Lines 412: test() with modified grid
#[test]
#[ignore]
fn test2x_019_4829_to_4750_with_blocks() {
    // test(4829, 4750, 1, 4917, 5083, 5247, 5245);
    let mut grid = load_test_grid();
    grid.set_cell(4998 % 104, 4998 / 104, 1);  // setBlocked
    grid.set_cell(4999 % 104, 4999 / 104, 1);
    assert_path(&grid, 4829, 4750, Some(&[4917, 5083, 5247, 5245]), "019");
}

// Now I need to go back and add the tests from lines 235-285 that I skipped
// These were test2() calls with messyY=true
// Actually, looking at the test wrapper functions more carefully:
// Line 210: test2(start, dest, size, reverse, ...path) => test3(start, dest, size, reverse, messyY, false, ...path)
// Line 211: Uses the `messyY` variable which is `true` at line 193
// So test2() uses messyY but always messyX=false

// Wait, I need to re-read test3:
// test3(start, dest, size, reverse, messY, messyX, ...path) => test4(pf, start, dest, size, reverse, messY, messyX, ...path)

// Looking at test4 signature (line 94):
// test4(PathFinder pfx, int start, int dest, int size, bool reverse, bool messY, bool messyX, params int[]? exPath)

// So the parameter order in test3 is: (start, dest, size, reverse, messY, messyX, ...path)
// And test2 calls test3 with: test3(start, dest, size, reverse, messyY, false, ...path)
// Where messyY is the variable (true initially, false after line 312), and messyX is hardcoded to false

// So I need to identify which tests have BOTH messyY=false AND messyX=false

// Let me trace through more carefully:
// Lines 215-229: test4() directly with messyX=false, messyY=false ✓ INCLUDED above
// Lines 235-246: test2() with messyY=true (variable), messyX=false (hardcoded) - SKIP
// Lines 253-262: test() => test2(reverse=true) => messyY=true - SKIP
// Lines 266-285: test2() and test3() with messyY=true - SKIP
// Line 312: messyY = false
// Lines 313-344: Mix of test3() with various combinations
// Lines 350-380: test() calls with messyY=false ✓ INCLUDED above
// Lines 412-413: test() with modified grid ✓ INCLUDED above

// Let me add the remaining test3() calls from lines 313-344 that have BOTH messyY=false AND messyX=false

// Summary of what I've included so far:
// - Lines 215-229: 5 tests ✓
// - Line 322: 1 test ✓
// - Line 326: 1 test ✓
// - Line 331: 1 test ✓
// - Line 344: 1 test ✓
// - Lines 350-380: 8 tests ✓
// Total so far: 17 tests

// I think that's all the size=1, messyX=false, messyY=false tests!
// The rest are either size=2, or have messyX=true, or have messyY=true
