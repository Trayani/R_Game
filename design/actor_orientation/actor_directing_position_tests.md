# Actor Directing Position-Variant Tests

Comprehensive test suite for position-aware diagonal reservation algorithm (Design V2).

## Test Structure

For each diagonal destination from baseline tests (T021-T036, excluding pure cardinal and BOTH-only cases), we test with **9 actor positions** within the reservation rectangle.

**Base diagonal tests**: 12 tests with single affinity (V or H)
**Position variants per test**: 9 positions
**Total tests**: 12 × 9 = **108 tests**

## 9 Standard Actor Positions

For a rectangle formed by PSC (5,5) and diagonal subcell (e.g., 6,4):

| Position | ID | Location | Coordinates Example |
|----------|----|-----------|--------------------|
| PSC Center | P1 | Primary subcell center | (5.0, 5.0) |
| Rectangle Center | P2 | Geometric center of rectangle | (5.5, 4.5) |
| Diagonal Corner | P3 | At diagonal subcell center | (6.0, 4.0) |
| Top Edge Center | P4 | Center of top edge | (5.5, 4.0) |
| Bottom Edge Center | P5 | Center of bottom edge | (5.5, 5.0) |
| Left Edge Center | P6 | Center of left edge | (5.0, 4.5) |
| Right Edge Center | P7 | Center of right edge | (6.0, 4.5) |
| Bottom-Left Corner | P8 | PSC corner (often same as P1) | (5.0, 5.0) |
| Bottom-Right Corner | P9 | Anchor corner | (6.0, 5.0) |

## Column Definitions

- **test_id**: Unique test identifier (format: T{base}_{pos})
- **base**: Original baseline test number
- **position**: Position variant (P1-P9)
- **actor_x, actor_y**: Actor's precise float coordinates
- **PSC_x, PSC_y**: Primary subcell coordinates
- **diag_x, diag_y**: Diagonal subcell coordinates
- **dest_x, dest_y**: Destination subcell center coordinates
- **target_x, target_y**: Calculated target position on rectangle boundary
- **affinity**: H (horizontal-favoring), V (vertical-favoring), or BOTH
- **anchor_x, anchor_y**: Reserved anchor subcell coordinates
- **optimal_dir**: Direction with affinity suffix
- **notes**: Calculation details

## Sample Test Cases (Fully Calculated)

### T021: Destination (6, 1) - Baseline V-affinity from PSC center

**Rectangle**: PSC (5,5) → Diagonal (6,4), bounds [5.0-6.0, 4.0-5.0]

| test_id | position | actor_x | actor_y | dest_x | dest_y | target_x | target_y | affinity | anchor_x | anchor_y | optimal_dir | notes |
|---------|----------|---------|---------|--------|--------|----------|----------|----------|----------|----------|-------------|-------|
| T021_P1 | PSC Center | 5.0 | 5.0 | 6 | 1 | 5.25 | 4.0 | V | 5 | 4 | NE-V | Baseline: t_h=0.25 < t_v=1.0 |
| T021_P2 | Rect Center | 5.5 | 4.5 | 6 | 1 | 5.54 | 4.0 | V | 5 | 4 | NE-V | t_h=0.182 < t_v=0.909 |
| T021_P3 | Diag Corner | 6.0 | 4.0 | 6 | 1 | 6.0 | 4.0 | BOTH | 5,4 or 6,5 | NE | Already at diagonal |
| T021_P4 | Top Edge | 5.5 | 4.0 | 6 | 1 | 5.5 | 4.0 | BOTH | 5,4 or 6,5 | NE | Already on top edge |
| T021_P5 | Bottom Edge | 5.5 | 5.0 | 6 | 1 | 5.71 | 4.0 | V | 5 | 4 | NE-V | t_h=0.364 < t_v=0.818 |
| T021_P6 | Left Edge | 5.0 | 4.5 | 6 | 1 | 5.33 | 4.0 | V | 5 | 4 | NE-V | t_h=0.286 < t_v=1.143 |
| T021_P7 | Right Edge | 6.0 | 4.5 | 6 | 1 | 6.0 | 4.0 | H | 6 | 5 | NE-H | t_h=0.5 > t_v=0 (at edge) |
| T021_P8 | BL Corner | 5.0 | 5.0 | 6 | 1 | 5.25 | 4.0 | V | 5 | 4 | NE-V | Same as P1 |
| T021_P9 | BR Corner | 6.0 | 5.0 | 6 | 1 | 6.0 | 4.0 | H | 6 | 5 | NE-H | t_v=0 (at edge), t_h=0.25 |

### T022: Destination (7, 2) - Baseline V-affinity

**Rectangle**: PSC (5,5) → Diagonal (6,4), bounds [5.0-6.0, 4.0-5.0]

| test_id | position | actor_x | actor_y | dest_x | dest_y | target_x | target_y | affinity | anchor_x | anchor_y | optimal_dir | notes |
|---------|----------|---------|---------|--------|--------|----------|----------|----------|----------|----------|-------------|-------|
| T022_P1 | PSC Center | 5.0 | 5.0 | 7 | 2 | 5.67 | 4.0 | V | 5 | 4 | NE-V | Baseline: t_h=0.333 < t_v=0.5 |
| T022_P2 | Rect Center | 5.5 | 4.5 | 7 | 2 | 5.83 | 4.0 | V | 5 | 4 | NE-V | t_h=0.25 < t_v=0.313 |
| T022_P3 | Diag Corner | 6.0 | 4.0 | 7 | 2 | 6.0 | 4.0 | BOTH | 5,4 or 6,5 | NE | Already at diagonal |
| T022_P4 | Top Edge | 5.5 | 4.0 | 7 | 2 | 5.5 | 4.0 | BOTH | 5,4 or 6,5 | NE | Already on top edge |
| T022_P5 | Bottom Edge | 5.5 | 5.0 | 7 | 2 | 6.0 | 4.33 | H | 6 | 5 | NE-H | t_v=0.25 < t_h=0.333 |
| T022_P6 | Left Edge | 5.0 | 4.5 | 7 | 2 | 5.8 | 4.0 | V | 5 | 4 | NE-V | t_h=0.4 < t_v=0.667 |
| T022_P7 | Right Edge | 6.0 | 4.5 | 7 | 2 | 6.0 | 4.0 | H | 6 | 5 | NE-H | t_v=0 (at edge) |
| T022_P8 | BL Corner | 5.0 | 5.0 | 7 | 2 | 5.67 | 4.0 | V | 5 | 4 | NE-V | Same as P1 |
| T022_P9 | BR Corner | 6.0 | 5.0 | 7 | 2 | 6.0 | 4.5 | H | 6 | 5 | NE-H | t_v=0 (at edge), t_h=0.25 |

### T024: Destination (8, 3) - Baseline H-affinity

**Rectangle**: PSC (5,5) → Diagonal (6,4), bounds [5.0-6.0, 4.0-5.0]

| test_id | position | actor_x | actor_y | dest_x | dest_y | target_x | target_y | affinity | anchor_x | anchor_y | optimal_dir | notes |
|---------|----------|---------|---------|--------|--------|----------|----------|----------|----------|----------|-------------|-------|
| T024_P1 | PSC Center | 5.0 | 5.0 | 8 | 3 | 6.0 | 4.33 | H | 6 | 5 | NE-H | Baseline: t_v=0.333 < t_h=0.5 |
| T024_P2 | Rect Center | 5.5 | 4.5 | 8 | 3 | 6.0 | 4.2 | H | 6 | 5 | NE-H | t_v=0.2 < t_h=0.286 |
| T024_P3 | Diag Corner | 6.0 | 4.0 | 8 | 3 | 6.0 | 4.0 | BOTH | 5,4 or 6,5 | NE | Already at diagonal |
| T024_P4 | Top Edge | 5.5 | 4.0 | 8 | 3 | 5.5 | 4.0 | BOTH | 5,4 or 6,5 | NE | Already on top edge |
| T024_P5 | Bottom Edge | 5.5 | 5.0 | 8 | 3 | 6.0 | 4.6 | H | 6 | 5 | NE-H | t_v=0.286 < t_h=0.5 |
| T024_P6 | Left Edge | 5.0 | 4.5 | 8 | 3 | 6.0 | 4.17 | H | 6 | 5 | NE-H | t_v=0.4 < t_h=0.667 |
| T024_P7 | Right Edge | 6.0 | 4.5 | 8 | 3 | 6.0 | 4.5 | H | 6 | 5 | NE-H | t_v=0 (at edge) |
| T024_P8 | BL Corner | 5.0 | 5.0 | 8 | 3 | 6.0 | 4.33 | H | 6 | 5 | NE-H | Same as P1 |
| T024_P9 | BR Corner | 6.0 | 5.0 | 8 | 3 | 6.0 | 5.0 | H | 6 | 5 | NE-H | t_v=0 (at edge), stays in place |

## Test Coverage by Quadrant

### North-East Tests (T021-T026, 6 base × 9 pos = 54 tests)
- T021_P1 through T021_P9: dest=(6,1)
- T022_P1 through T022_P9: dest=(7,2)
- T024_P1 through T024_P9: dest=(8,3)
- T025_P1 through T025_P9: dest=(9,3)
- T026_P1 through T026_P9: dest=(9,4)

### South-East Tests (T027-T029, 2 base × 9 pos = 18 tests)
- T027_P1 through T027_P9: dest=(8,7)
- T029_P1 through T029_P9: dest=(7,8)

### South-West Tests (T031-T032, 2 base × 9 pos = 18 tests)
- T031_P1 through T031_P9: dest=(2,7)
- T032_P1 through T032_P9: dest=(1,7)

### North-West Tests (T034-T036, 3 base × 9 pos = 27 tests)
- T034_P1 through T034_P9: dest=(2,3)
- T035_P1 through T035_P9: dest=(2,1)
- T036_P1 through T036_P9: dest=(1,2)

**Total: 108 position-variant tests**

## Expected Patterns

### Pattern 1: Actor at PSC Center (P1, P8)
- Should match baseline test results
- P8 often identical to P1 (both at bottom-left corner)

### Pattern 2: Actor at Rectangle Center (P2)
- Affinity may differ from baseline
- Often shows BOTH affinity for 45° destinations
- Target closer to diagonal corner than baseline

### Pattern 3: Actor at Diagonal Corner (P3)
- Always results in target = actor position
- Affinity = BOTH (already at destination subcell)
- No actual movement needed

### Pattern 4: Actor on Edges (P4-P7)
- P4 (top edge): Often BOTH or V-affinity
- P5 (bottom edge): Often maintains baseline affinity
- P6 (left edge): Increases distance to vertical edge
- P7 (right edge): t_vertical = 0, uses H-affinity

### Pattern 5: Actor at Corners (P9)
- Bottom-right (anchor) corner
- Often H-affinity due to being on right edge
- May result in minimal movement

## Calculation Notes

### Ray-Rectangle Intersection Formula

For each test, we calculate:

```python
# 1. Ray direction (normalized)
ray_x = (dest_x - actor_x) / distance
ray_y = (dest_y - actor_y) / distance

# 2. Parametric t values
t_vertical = (edge_x - actor_x) / ray_x  # For appropriate edge
t_horizontal = (edge_y - actor_y) / ray_y  # For appropriate edge

# 3. Compare t values
if t_vertical < t_horizontal:
    affinity = H
    target = (edge_x, actor_y + ray_y * t_vertical)
elif t_horizontal < t_vertical:
    affinity = V
    target = (actor_x + ray_x * t_horizontal, edge_y)
else:
    affinity = BOTH
    target = (edge_x, edge_y)  # corner
```

### Special Cases

**Case 1: Actor already on boundary (t=0)**
- Target = actor's current position
- Affinity based on which boundary

**Case 2: Actor at corner**
- Both t values = 0 or both very small
- Affinity = BOTH
- Target = corner position

**Case 3: Destination exactly 45° (equal X/Y difference)**
- From center positions: Usually BOTH affinity
- From edge positions: Single affinity based on which edge

## Test Validation Rules

For each test, verify:

1. ✅ Target lies on rectangle boundary
2. ✅ Ray from actor through target points toward destination
3. ✅ Anchor is adjacent to both PSC and diagonal
4. ✅ Affinity matches which edge was hit first
5. ✅ Target coordinates are within rectangle bounds
6. ✅ For t=0 cases (already at boundary), target = actor position

## Usage

Tests can be run in three modes:

1. **Baseline Only**: Run 40 original tests (actor always at PSC center)
2. **Critical Subset**: Run P1, P2, P3 variants (3×12 = 36 tests) + baselines = 76 total
3. **Comprehensive**: Run all 108 position-variant tests + 40 baselines = 148 total

Recommendation: Start with **Critical Subset** for initial implementation validation.
