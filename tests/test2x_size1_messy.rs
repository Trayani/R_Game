// Test2X test cases with messy positions (messyX=true and/or messyY=true)
// Source: GameLib2/GameLib/tests/ProtoTests.cs::Test2X()
// Lines 235-311 (before messyY=false at line 312)

use rustgame3::{Grid, pathfinding::find_path_by_id};
use std::fs;
use serde_json;

fn load_test_grid() -> Grid {
    let json = fs::read_to_string("test_data/grid2_blocked.json")
        .expect("Failed to load grid2_blocked.json");
    let blocked: Vec<i32> = serde_json::from_str(&json)
        .expect("Failed to parse grid2_blocked.json");
    Grid::with_blocked(104, 83, &blocked)
}

// Helper to run pathfinding and validate
// Note: expected_waypoints contains ONLY intermediate waypoints (not start/dest)
// The full path returned by find_path_by_id should be: [start, ...waypoints, dest]
fn assert_path(
    grid: &Grid,
    start: i32,
    dest: i32,
    expected_waypoints: Option<&[i32]>,
    messy_x: bool,
    messy_y: bool,
    test_name: &str,
) {
    let result = find_path_by_id(grid, start, dest, messy_x, messy_y);

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

// ===== TESTS WITH messyY=true, messyX=false (Lines 235-311 before messyY=false) =====

// Lines 235-246: test2() calls with messyY=true (from line 193), messyX=false (hardcoded in test2)
// test2(start, dest, size, reverse, ...path) => test3(start, dest, size, reverse, messyY=true, messyX=false, ...path)

#[test]
fn test2x_messy_001_6388_to_6139() {
    // test2(6388, 6139, 1, false, 6141, 6307, 6306, 6389);
    let grid = load_test_grid();
    assert_path(&grid, 6388, 6139, Some(&[6141, 6307, 6306, 6389]), false, true, "messy_001");
}

#[test]
fn test2x_messy_002_1205_to_1208() {
    // test2(1205, 1208, 1, false);
    let grid = load_test_grid();
    assert_path(&grid, 1205, 1208, None, false, true, "messy_002");
}

#[test]
fn test2x_messy_003_1208_to_1205() {
    // test2(1208, 1205, 1, false);
    let grid = load_test_grid();
    assert_path(&grid, 1208, 1205, None, false, true, "messy_003");
}

#[test]
fn test2x_messy_004_1207_to_1210() {
    // test2(1207, 1210, 1, false, 1209);
    let grid = load_test_grid();
    assert_path(&grid, 1207, 1210, Some(&[1209]), false, true, "messy_004");
}

#[test]
fn test2x_messy_005_4638_to_4719() {
    // test2(4638, 4719, 1, false, 4553, 4555);
    let grid = load_test_grid();
    assert_path(&grid, 4638, 4719, Some(&[4553, 4555]), false, true, "messy_005");
}

#[test]
fn test2x_messy_006_4638_to_4641() {
    // test2(4638, 4641, 1, false, 4558, 4556);
    let grid = load_test_grid();
    assert_path(&grid, 4638, 4641, Some(&[4558, 4556]), false, true, "messy_006");
}

#[test]
fn test2x_messy_007_6302_to_6383() {
    // test2(6302, 6383, 1, false, 6217, 6219);
    let grid = load_test_grid();
    assert_path(&grid, 6302, 6383, Some(&[6217, 6219]), false, true, "messy_007");
}

#[test]
fn test2x_messy_008_6388_to_6139_again() {
    // test2(6388, 6139, 1, false, 6141, 6307, 6306, 6389);
    let grid = load_test_grid();
    assert_path(&grid, 6388, 6139, Some(&[6141, 6307, 6306, 6389]), false, true, "messy_008");
}

#[test]
fn test2x_messy_009_6552_to_6720() {
    // test2(6552, 6720, 1, false, 6637, 6636);
    let grid = load_test_grid();
    assert_path(&grid, 6552, 6720, Some(&[6637, 6636]), false, true, "messy_009");
}

#[test]
fn test2x_messy_010_6637_to_6552() {
    // test2(6637, 6552, 1, false, 6636, 6637);
    let grid = load_test_grid();
    assert_path(&grid, 6637, 6552, Some(&[6636, 6637]), false, true, "messy_010");
}

#[test]
fn test2x_messy_011_4062_to_4231() {
    // test2(4062, 4231, 1, false, 4065, 4063);
    let grid = load_test_grid();
    assert_path(&grid, 4062, 4231, Some(&[4065, 4063]), false, true, "messy_011");
}

#[test]
fn test2x_messy_012_5476_to_5556() {
    // test2(5476, 5556, 1, false, 5474, 5476);
    let grid = load_test_grid();
    assert_path(&grid, 5476, 5556, Some(&[5474, 5476]), false, true, "messy_012");
}

// Lines 253-261: test() calls with messyY=true (from line 193), messyX=false
// test(start, dest, size, ...path) => test2(start, dest, size, reverse=true, ...path)
//                                 => test3(start, dest, size, reverse=true, messyY=true, messyX=false, ...path)

#[test]
fn test2x_messy_013_4887_to_5051() {
    // test(4887, 5051, 1, 5300, 5303, 5137);
    let grid = load_test_grid();
    assert_path(&grid, 4887, 5051, Some(&[5300, 5303, 5137]), false, true, "messy_013");
}

#[test]
fn test2x_messy_014_4888_to_5051() {
    // test(4888, 5051, 1, 5300, 5303);
    let grid = load_test_grid();
    assert_path(&grid, 4888, 5051, Some(&[5300, 5303]), false, true, "messy_014");
}

#[test]
fn test2x_messy_015_4890_to_5142() {
    // test(4890, 5142, 1, 5308, 5305);
    let grid = load_test_grid();
    assert_path(&grid, 4890, 5142, Some(&[5308, 5305]), false, true, "messy_015");
}

#[test]
fn test2x_messy_016_4891_to_5142() {
    // test(4891, 5142, 1, 5308, 5305, 5139);
    let grid = load_test_grid();
    assert_path(&grid, 4891, 5142, Some(&[5308, 5305, 5139]), false, true, "messy_016");
}

#[test]
fn test2x_messy_017_5721_to_5719() {
    // test(5721, 5719, 1, 5968, 5970);
    let grid = load_test_grid();
    assert_path(&grid, 5721, 5719, Some(&[5968, 5970]), false, true, "messy_017");
}

#[test]
fn test2x_messy_018_5721_to_5725() {
    // test(5721, 5725, 1, 5891, 5889);
    let grid = load_test_grid();
    assert_path(&grid, 5721, 5725, Some(&[5891, 5889]), false, true, "messy_018");
}

#[test]
fn test2x_messy_019_3768_to_2440() {
    // test(3768, 2440, 1);
    let grid = load_test_grid();
    assert_path(&grid, 3768, 2440, None, false, true, "messy_019");
}

#[test]
fn test2x_messy_020_2341_to_2591() {
    // test2(2341, 2591, 1, false, 2424);
    let grid = load_test_grid();
    assert_path(&grid, 2341, 2591, Some(&[2424]), false, true, "messy_020");
}

// Lines 266-285: Mix of test2() and test3() - still with messyY=true

#[test]
fn test2x_messy_021_2679_to_2768() {
    // test2(2679, 2768, 1, false, 2851, 2845);
    let grid = load_test_grid();
    assert_path(&grid, 2679, 2768, Some(&[2851, 2845]), false, true, "messy_021");
}

#[test]
fn test2x_messy_022_2762_to_2768() {
    // test2(2762, 2768, 1, false, 2851, 2845);
    let grid = load_test_grid();
    assert_path(&grid, 2762, 2768, Some(&[2851, 2845]), false, true, "messy_022");
}

#[test]
fn test2x_messy_023_2596_to_2768() {
    // test2(2596, 2768, 1, false, 2851, 2845);
    let grid = load_test_grid();
    assert_path(&grid, 2596, 2768, Some(&[2851, 2845]), false, true, "messy_023");
}

#[test]
fn test2x_messy_024_2679_to_2684() {
    // test2(2679, 2684, 1, false, 2679);
    let grid = load_test_grid();
    assert_path(&grid, 2679, 2684, Some(&[2679]), false, true, "messy_024");
}

#[test]
fn test2x_messy_025_2679_to_2270() {
    // test2(2679, 2270, 1, false, 2269, 2599, 2679);
    let grid = load_test_grid();
    assert_path(&grid, 2679, 2270, Some(&[2269, 2599, 2679]), false, true, "messy_025");
}

#[test]
fn test2x_messy_026_2762_to_2270() {
    // test2(2762, 2270, 1, false, 2269, 2599, 2679);
    let grid = load_test_grid();
    assert_path(&grid, 2762, 2270, Some(&[2269, 2599, 2679]), false, true, "messy_026");
}

#[test]
fn test2x_messy_027_2845_to_2270() {
    // test2(2845, 2270, 1, false, 2269, 2599, 2679);
    let grid = load_test_grid();
    assert_path(&grid, 2845, 2270, Some(&[2269, 2599, 2679]), false, true, "messy_027");
}

// Lines 298-309: test3() calls with explicit messyX flags

#[test]
fn test2x_messy_028_1348_to_1185_messyx() {
    // test3(1348, 1185, 1, false, true, false, 1182);
    // messyY=true, messyX=false
    let grid = load_test_grid();
    assert_path(&grid, 1348, 1185, Some(&[1182]), false, true, "messy_028");
}

#[test]
fn test2x_messy_029_3008_to_2674_messyx() {
    // test3(3008, 2674, 1, false, true, false, 2759);
    // messyY=true, messyX=false
    let grid = load_test_grid();
    assert_path(&grid, 3008, 2674, Some(&[2759]), false, true, "messy_029");
}

// ===== TESTS WITH messyX=true (Lines 313-318 after messyY=false) =====
// Note: messyY becomes false at line 312

#[test]
fn test2x_messy_030_6061_to_6309_messyx() {
    // test3(6061, 6309, 1, false, false, true, 6312, 6146);
    // messyY=false, messyX=true
    let grid = load_test_grid();
    assert_path(&grid, 6061, 6309, Some(&[6312, 6146]), true, false, "messy_030");
}

#[test]
fn test2x_messy_031_6062_to_6309_messyx() {
    // test3(6062, 6309, 1, false, false, true, 6312, 6146);
    // messyY=false, messyX=true
    let grid = load_test_grid();
    assert_path(&grid, 6062, 6309, Some(&[6312, 6146]), true, false, "messy_031");
}

#[test]
fn test2x_messy_032_2596_to_2768_both() {
    // test3(2596, 2768, 1, false, true, true, 2851, 2845, 2679);
    // messyY=true, messyX=true
    let grid = load_test_grid();
    assert_path(&grid, 2596, 2768, Some(&[2851, 2845, 2679]), true, true, "messy_032");
}
