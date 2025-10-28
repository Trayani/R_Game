# Target Position Explanation for T021_P1

## The Question

"The target has to be placed on the TOP-LEFT rectangle's upper horizontal boundary. Why is it not on a vertical boundary?"

## The Answer

**The target IS correctly placed on the upper horizontal boundary at y=4.0.** This is correct for **V-affinity** (vertical-favoring).

## Important: Coordinate System Clarification

**Subcells are positioned at grid line intersections, not between them:**
- PSC (5, 5) is at position (5.0, 5.0) - ON the grid intersection
- Diagonal (6, 4) is at position (6.0, 4.0) - ON the grid intersection
- Rectangle spans from (5.0, 5.0) to (6.0, 4.0)
- No +0.5 offset is applied to subcell positions

## Understanding Affinity

### V-Affinity (Vertical-Favoring)
- Ray hits **HORIZONTAL edge first** (top or bottom)
- Target is placed on **HORIZONTAL boundary**
- Actor moves primarily **vertically** to reach the boundary
- Anchor is **vertical neighbor** of PSC

### H-Affinity (Horizontal-Favoring)
- Ray hits **VERTICAL edge first** (left or right)
- Target is placed on **VERTICAL boundary**
- Actor moves primarily **horizontally** to reach the boundary
- Anchor is **horizontal neighbor** of PSC

## Calculation for T021_P1

**Given:**
- Actor position: (5.0, 5.0)
- PSC: (5, 5), center at (5.5, 5.5)
- Diagonal: (6, 4), center at (6.5, 4.5)
- Destination: (6, 1) - subcell center at (6.5, 1.5)
- Rectangle: x ∈ [5.0, 6.0], y ∈ [4.0, 5.0]

**Ray direction (from actor to destination):**
```
dir_x = 6.5 - 5.0 = 1.5
dir_y = 1.5 - 5.0 = -3.5
length = sqrt(1.5² + 3.5²) = sqrt(14.5) ≈ 3.808

ray_x = 1.5 / 3.808 ≈ 0.394
ray_y = -3.5 / 3.808 ≈ -0.919
```

**Wait - I need to recalculate with subcell CENTERS, not integer coords!**

Actually, looking at the test data, destination is given as `[6, 1]` which are integer subcell coordinates. Let me check if these represent subcell centers or corners...

**From design doc**: "destination is always a subcell center (whole number)"

So destination (6, 1) means the CENTER of subcell (6, 1), which is at float position (6.5, 1.5) for a 2x2 grid!

**Recalculating:**
```
Actor: (5.0, 5.0)
Destination CENTER: (6.5, 1.5)

dir_x = 6.5 - 5.0 = 1.5
dir_y = 1.5 - 5.0 = -3.5
length = sqrt(1.5² + 3.5²) = sqrt(14.5) ≈ 3.808

ray_x = 1.5 / 3.808 ≈ 0.394
ray_y = -3.5 / 3.808 ≈ -0.919
```

**Rectangle intersection:**
```
Rectangle: x ∈ [5.0, 6.0], y ∈ [4.0, 5.0]

ray_x > 0: t_right = (6.0 - 5.0) / 0.394 ≈ 2.538
ray_y < 0: t_top = (4.0 - 5.0) / -0.919 ≈ 1.088

t_horizontal_edge (top) = 1.088
t_vertical_edge (right) = 2.538

t_horizontal < t_vertical → V-affinity (hits horizontal edge first)
```

**Target position:**
```
target_x = 5.0 + 0.394 × 1.088 ≈ 5.0 + 0.429 ≈ 5.43
target_y = 5.0 + (-0.919) × 1.088 ≈ 5.0 - 1.0 = 4.0
```

**Hmm, I'm getting 5.43, but the test says 5.25...**

Let me check if destination coordinates in the JSON are already centers or need offset...

## Issue Found

There's a discrepancy in coordinate interpretation:
- Test data shows destination as `[6, 1]`
- Design doc says "destination is always a subcell center"
- For 2x2 subcell grid, subcell (6,1) center should be at (6.5, 1.5)
- But calculations seem to use (6, 1) directly as float coordinates

**Need to clarify:** Are destination coordinates in the test JSON:
1. Subcell integer coordinates (need +0.5 offset to get center)?
2. Already float coordinates representing the center?

The calculation that produces target_x=5.25 suggests destinations are being treated as integer coordinates WITHOUT the +0.5 offset, which contradicts the subcell center principle.
