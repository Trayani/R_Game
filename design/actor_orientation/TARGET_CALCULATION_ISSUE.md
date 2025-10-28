# Target Calculation Issue - T021_P6

## Problem

The target position for T021_P6 does not lie on the ray from actor to destination, indicating a calculation error in the test data.

## Test Data

- **Test ID:** T021_P6
- **Actor:** (5.0, 4.5)
- **PSC:** (5, 5)
- **Diagonal:** (6, 4)
- **Destination:** (6, 1)
- **Target (test data):** (5.33, 4.0)
- **Affinity:** V
- **Notes:** t_h=0.286 < t_v=1.143

## Correct Calculation

### Step 1: Ray Direction

```
dir_x = 6.0 - 5.0 = 1.0
dir_y = 1.0 - 4.5 = -3.5
length = sqrt(1.0² + 3.5²) = sqrt(13.25) ≈ 3.640

ray_x = 1.0 / 3.640 ≈ 0.2747
ray_y = -3.5 / 3.640 ≈ -0.9615
```

### Step 2: Rectangle Intersection

```
Rectangle: x ∈ [5.0, 6.0], y ∈ [4.0, 5.0]

ray_x > 0: t_right = (6.0 - 5.0) / 0.2747 ≈ 3.640
ray_y < 0: t_top = (4.0 - 4.5) / -0.9615 ≈ 0.520

t_horizontal_edge (top) = 0.520
t_vertical_edge (right) = 3.640

Since 0.520 < 3.640 → V-affinity ✓ (matches test data)
```

### Step 3: Target Position

```
target_x = 5.0 + 0.2747 × 0.520 = 5.0 + 0.143 ≈ 5.143
target_y = 4.5 + (-0.9615) × 0.520 = 4.5 - 0.500 = 4.0 ✓
```

## Comparison

| Value | Test Data | Calculated | Difference |
|-------|-----------|------------|------------|
| target_x | 5.33 | 5.143 | 0.187 |
| target_y | 4.0 | 4.0 | 0.0 ✓ |
| t_h (from notes) | 0.286 | 0.520 | 0.234 |

## Verification with T021_P1

To confirm the formula is correct, let's verify T021_P1:

- Actor: (5.0, 5.0), Dest: (6, 1)
- Ray: (1.0, -4.0), normalized: (0.2425, -0.9701)
- t_top = (4.0 - 5.0) / -0.9701 ≈ 1.031
- target_x = 5.0 + 0.2425 × 1.031 = 5.0 + 0.25 = **5.25** ✓
- **Matches test data perfectly!**

## Coordinate Interpretation (Confirmed)

**Destination coordinates represent subcell integer coordinates directly:**
- Destination (6, 1) = point at (6.0, 1.0)
- NOT subcell center with offset (6.5, 1.5)

This has been confirmed by the user and matches the calculations.

## Root Cause

The test data for T021_P6 has an incorrect target_x value. The error appears to be in the original calculation when the test data was generated.

## Possible Cause of Error

Looking at the pattern, it seems like the calculation may have used an incorrect ray direction or t-value. The note says `t_h=0.286` but the correct value is `t_h=0.520`.

If we use t=0.286 (the incorrect value from notes):
```
target_x = 5.0 + 0.2747 × 0.286 = 5.0 + 0.079 ≈ 5.079 (still doesn't match 5.33!)
```

Working backwards from 5.33:
```
5.33 = 5.0 + ray_x × t
0.33 = ray_x × t

With t=0.520: ray_x = 0.33 / 0.520 = 0.635 (but actual is 0.2747!)
```

**Conclusion:** The test data has a calculation error. The target_x should be **5.143**, not 5.33.

## T021_P5 - Second Error Found

### Test Data (Before Fix)
- Actor: (5.5, 5.0) - bottom edge
- Destination: (6, 1) → (6.0, 1.0)
- Target (test data): (5.71, 4.0) ❌
- Notes claimed: t_h=0.364 < t_v=0.818

### Correct Calculation

```
Ray direction:
  dir_x = 6.0 - 5.5 = 0.5
  dir_y = 1.0 - 5.0 = -4.0
  length = sqrt(0.25 + 16) = 4.031

  ray_x = 0.5 / 4.031 = 0.1240
  ray_y = -4.0 / 4.031 = -0.9923

Intersection:
  t_right = (6.0 - 5.5) / 0.1240 = 4.032
  t_top = (4.0 - 5.0) / -0.9923 = 1.008

  t_top < t_right → V-affinity ✓

Target:
  target_x = 5.5 + 0.1240 × 1.008 = 5.625 ≈ 5.63
  target_y = 5.0 + (-0.9923) × 1.008 = 4.0 ✓
```

### Comparison

| Value | Test Data | Calculated | Difference |
|-------|-----------|------------|------------|
| target_x | 5.71 | 5.63 | 0.08 |
| target_y | 4.0 | 4.0 | 0.0 ✓ |
| t_h (from notes) | 0.364 | 4.032 | 3.668 (!) |
| t_v (from notes) | 0.818 | 1.008 | 0.190 |

### Root Cause

Same issue as T021_P6 - incorrect calculation in original test data generation. The t-values in the notes are completely wrong, suggesting a fundamental error in the calculation formula used.

**Pattern:** Both edge position tests (P5, P6) had calculation errors, suggesting the original generator had bugs when handling actors on rectangle boundaries.

### Fix Applied

- Updated target_x from 5.71 to 5.63 in both JSON and TSV
- Corrected t-values in notes: t_h=4.032 < t_v=1.008
- Marked as "(corrected)"

## Recommendation

Recalculate ALL test data in actor_directing_position_tests.tsv to ensure accuracy. Use the algorithm from actor_directing_v2.txt consistently.

### Tests Fixed So Far
- ✅ T021_P6 (Left_Edge): 5.33 → 5.14
- ✅ T021_P5 (Bottom_Edge): 5.71 → 5.63

## Tests to Recalculate

Priority check these tests for similar errors:
- All T021_P2 through T021_P9 (positions within same rectangle)
- All other position-variant tests (T022_P2-P9, T024_P2-P9, etc.)
