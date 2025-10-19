# Messy Position Tests Status

## Overview

Ported 32 messy position pathfinding tests from C# Test2X to Rust.

**Results**: 14 passing (43.75%), 18 failing (56.25%)

These failures are EXPECTED - messy positions have known issues that need investigation.

## Test Categories

### messyY=true, messyX=false (28 tests)
Tests where the entity's vertical position is between grid rows (non-aligned).

### messyY=false, messyX=true (2 tests)
Tests where the entity's horizontal position is between grid columns (non-aligned).

### messyY=true, messyX=true (2 tests)
Tests where the entity is non-aligned both horizontally and vertically.

## Passing Tests (14)

1. ✅ test2x_messy_002_1205_to_1208 (messyY=true)
2. ✅ test2x_messy_003_1208_to_1205 (messyY=true)
3. ✅ test2x_messy_004_1207_to_1210 (messyY=true)
4. ✅ test2x_messy_013_4887_to_5051 (messyY=true)
5. ✅ test2x_messy_014_4888_to_5051 (messyY=true)
6. ✅ test2x_messy_015_4890_to_5142 (messyY=true)
7. ✅ test2x_messy_016_4891_to_5142 (messyY=true)
8. ✅ test2x_messy_019_3768_to_2440 (messyY=true)
9. ✅ test2x_messy_020_2341_to_2591 (messyY=true)
10. ✅ test2x_messy_021_2679_to_2768 (messyY=true)
11. ✅ test2x_messy_022_2762_to_2768 (messyY=true)
12. ✅ test2x_messy_023_2596_to_2768 (messyY=true)
13. ✅ test2x_messy_028_1348_to_1185_messyx (messyY=true)
14. ✅ test2x_messy_029_3008_to_2674_messyx (messyY=true)

## Failing Tests (18)

### Type 1: Path Not Found (15 tests)

These tests expect a path but Rust returns None (no path found):

1. ❌ test2x_messy_001_6388_to_6139 - Expected: [6141, 6307, 6306, 6389]
2. ❌ test2x_messy_007_6302_to_6383 - Expected: [6217, 6219]
3. ❌ test2x_messy_008_6388_to_6139_again - Expected: [6141, 6307, 6306, 6389]
4. ❌ test2x_messy_009_6552_to_6720 - Expected: [6637, 6636]
5. ❌ test2x_messy_010_6637_to_6552 - Expected: [6636, 6637]
6. ❌ test2x_messy_011_4062_to_4231 - Expected: [4065, 4063]
7. ❌ test2x_messy_012_5476_to_5556 - Expected: [5474, 5476]
8. ❌ test2x_messy_017_5721_to_5719 - Expected: [5968, 5970]
9. ❌ test2x_messy_018_5721_to_5725 - Expected: [5891, 5889]
10. ❌ test2x_messy_025_2679_to_2270 - Expected: [2269, 2599, 2679]
11. ❌ test2x_messy_026_2762_to_2270 - Expected: [2269, 2599, 2679]
12. ❌ test2x_messy_027_2845_to_2270 - Expected: [2269, 2599, 2679]
13. ❌ test2x_messy_030_6061_to_6309_messyx - Expected: [6312, 6146] (messyX=true)
14. ❌ test2x_messy_031_6062_to_6309_messyx - Expected: [6312, 6146] (messyX=true)
15. ❌ test2x_messy_032_2596_to_2768_both - Expected: [2851, 2845, 2679] (messyX=true, messyY=true)

### Type 2: Wrong Path Found (3 tests)

These tests find a different path than expected:

1. ❌ test2x_messy_005_4638_to_4719
   - Expected: [4638, 4553, 4555, 4719]
   - Got: [4638, 4553, 4226, 4719]

2. ❌ test2x_messy_006_4638_to_4641
   - Expected: [4638, 4558, 4556, 4641]
   - Got: [4638, 4644, 4310, 4555, 4641]

3. ❌ test2x_messy_024_2679_to_2684
   - Expected: [2679, 2679, 2684] (waypoint includes start position!)
   - Got: [2679, 2684] (direct path)

## Analysis

### messyX Tests (messyX=true)
- **Total**: 4 tests (030, 031, 032, and implicitly 005/006 may be affected)
- **Passing**: 0
- **Failing**: 4 (all return None)
- **Issue**: Appears that messyX pathfinding may not be fully implemented

### messyY Tests (messyY=true)
- **Total**: 28 tests
- **Passing**: 14 (50%)
- **Failing**: 14 (50%)
- **Issue**: Some corner discovery or visibility problems with vertical messy positions

### Curious Cases

**test2x_messy_024**: Expected path includes the start position as a waypoint: `[2679, 2679, 2684]`
- This seems unusual - why would start be a waypoint?
- Rust returns the more intuitive direct path: `[2679, 2684]`
- Need to verify C# actually produces this expected result

## Next Steps

1. Verify C# produces the expected paths for failing tests
2. Investigate why messyX tests all return None (potential missing implementation?)
3. Debug corner discovery for messyY tests that return None
4. Understand the asymmetry: why do some messyY tests pass while others fail?
5. Investigate test2x_messy_024's unusual expected waypoint (start position as waypoint)

## File Location

`/home/jan/hobby/RustGame3/tests/test2x_size1_messy.rs`
