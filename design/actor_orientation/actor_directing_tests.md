# Actor Directing Test Definitions

Complete test table for the actor directing algorithm defined in `actor_directing.txt`.

## Test Setup

- **Actor starting position**: subcell (5, 5) for all tests
- **Subcell grid**: Normalized coordinates representing logical units
- **Target calculation**: Uses intersection formula from lines 29-43 of design document

## Test Table (Markdown Format)

| ID | dest_x | dest_y | optimal | target_x | target_y | affinity | anchor_x | anchor_y | alt1 | alt2 | alt3 | notes |
|----|--------|--------|---------|----------|----------|----------|----------|----------|------|------|------|-------|
| T001 | 5 | 4 | NORTH | 5.0 | 4.0 | N/A | N/A | N/A | NORTH-EAST-V | NORTH-WEST-V | - | Pure cardinal |
| T002 | 5 | 3 | NORTH | 5.0 | 4.0 | N/A | N/A | N/A | NORTH-EAST-V | NORTH-WEST-V | - | Pure cardinal |
| T003 | 5 | 2 | NORTH | 5.0 | 4.0 | N/A | N/A | N/A | NORTH-EAST-V | NORTH-WEST-V | - | Pure cardinal |
| T004 | 5 | 1 | NORTH | 5.0 | 4.0 | N/A | N/A | N/A | NORTH-EAST-V | NORTH-WEST-V | - | Pure cardinal |
| T005 | 5 | 6 | SOUTH | 5.0 | 6.0 | N/A | N/A | N/A | SOUTH-EAST-V | SOUTH-WEST-V | - | Pure cardinal |
| T006 | 5 | 7 | SOUTH | 5.0 | 6.0 | N/A | N/A | N/A | SOUTH-EAST-V | SOUTH-WEST-V | - | Pure cardinal |
| T007 | 5 | 8 | SOUTH | 5.0 | 6.0 | N/A | N/A | N/A | SOUTH-EAST-V | SOUTH-WEST-V | - | Pure cardinal |
| T008 | 5 | 9 | SOUTH | 5.0 | 6.0 | N/A | N/A | N/A | SOUTH-EAST-V | SOUTH-WEST-V | - | Pure cardinal |
| T009 | 4 | 5 | WEST | 4.0 | 5.0 | N/A | N/A | N/A | SOUTH-WEST-H | NORTH-WEST-H | - | Pure cardinal, fixed from line 119 |
| T010 | 3 | 5 | WEST | 4.0 | 5.0 | N/A | N/A | N/A | SOUTH-WEST-H | NORTH-WEST-H | - | Pure cardinal |
| T011 | 2 | 5 | WEST | 4.0 | 5.0 | N/A | N/A | N/A | SOUTH-WEST-H | NORTH-WEST-H | - | Pure cardinal |
| T012 | 1 | 5 | WEST | 4.0 | 5.0 | N/A | N/A | N/A | SOUTH-WEST-H | NORTH-WEST-H | - | Pure cardinal |
| T013 | 6 | 5 | EAST | 6.0 | 5.0 | N/A | N/A | N/A | SOUTH-EAST-H | NORTH-EAST-H | - | Pure cardinal, fixed alternatives line 120 |
| T014 | 7 | 5 | EAST | 6.0 | 5.0 | N/A | N/A | N/A | SOUTH-EAST-H | NORTH-EAST-H | - | Pure cardinal, fixed alternatives |
| T015 | 8 | 5 | EAST | 6.0 | 5.0 | N/A | N/A | N/A | SOUTH-EAST-H | NORTH-EAST-H | - | Pure cardinal, fixed alternatives |
| T016 | 9 | 5 | EAST | 6.0 | 5.0 | N/A | N/A | N/A | SOUTH-EAST-H | NORTH-EAST-H | - | Pure cardinal, fixed alternatives |
| T017 | 6 | 4 | NORTH-EAST | 6.0 | 4.0 | BOTH | 6,5 or 5,4 | - | NORTH | EAST | - | One subcell diagonal, t_side=t_top=1.0 |
| T018 | 4 | 4 | NORTH-WEST | 4.0 | 4.0 | BOTH | 4,5 or 5,4 | - | NORTH | WEST | - | One subcell diagonal, t_side=t_top=1.0 |
| T019 | 6 | 6 | SOUTH-EAST | 6.0 | 6.0 | BOTH | 6,5 or 5,6 | - | SOUTH | EAST | - | One subcell diagonal, t_side=t_top=1.0 |
| T020 | 4 | 6 | SOUTH-WEST | 4.0 | 6.0 | BOTH | 4,5 or 5,6 | - | SOUTH | WEST | - | One subcell diagonal, t_side=t_top=1.0 |
| T021 | 6 | 1 | NORTH-EAST-V | 5.25 | 4.0 | V | 5 | 4 | NORTH-EAST-H | NORTH | EAST | t_top=0.25 < t_side=1.0, hits horizontal first |
| T022 | 7 | 2 | NORTH-EAST-V | 5.67 | 4.0 | V | 5 | 4 | NORTH-EAST-H | NORTH | EAST | t_top=0.33 < t_side=0.5, hits horizontal first |
| T023 | 7 | 3 | NORTH-EAST | 6.0 | 4.0 | BOTH | 6,5 or 5,4 | - | NORTH | EAST | - | t_side=0.5, t_top=0.5, equal |
| T024 | 8 | 3 | NORTH-EAST-H | 6.0 | 4.33 | H | 6 | 5 | NORTH-EAST-V | EAST | NORTH | t_side=0.33 < t_top=0.5, hits vertical first |
| T025 | 9 | 3 | NORTH-EAST-H | 6.0 | 4.50 | H | 6 | 5 | NORTH-EAST-V | EAST | NORTH | t_side=0.25 < t_top=0.5, hits vertical first |
| T026 | 9 | 4 | NORTH-EAST-H | 6.0 | 4.75 | H | 6 | 5 | NORTH-EAST-V | EAST | NORTH | Fixed direction, t_side=0.25 < t_top=1.0 |
| T027 | 8 | 7 | SOUTH-EAST-H | 6.0 | 5.67 | H | 6 | 5 | SOUTH-EAST-V | EAST | SOUTH | t_side=0.33 < t_top=0.5, hits vertical first |
| T028 | 7 | 7 | SOUTH-EAST | 6.0 | 6.0 | BOTH | 6,5 or 5,6 | - | SOUTH | EAST | - | t_side=0.5, t_top=0.5, equal |
| T029 | 7 | 8 | SOUTH-EAST-V | 5.67 | 6.0 | V | 5 | 6 | SOUTH-EAST-H | SOUTH | EAST | t_top=0.33 < t_side=0.5, hits horizontal first |
| T030 | 3 | 7 | SOUTH-WEST | 4.0 | 6.0 | BOTH | 4,5 or 5,6 | - | SOUTH | WEST | - | t_side=0.5, t_top=0.5, equal |
| T031 | 2 | 7 | SOUTH-WEST-H | 4.0 | 5.67 | H | 4 | 5 | SOUTH-WEST-V | WEST | SOUTH | t_side=0.33 < t_top=0.5, hits vertical first |
| T032 | 1 | 7 | SOUTH-WEST-H | 4.0 | 5.50 | H | 4 | 5 | SOUTH-WEST-V | WEST | SOUTH | t_side=0.25 < t_top=0.5, hits vertical first |
| T033 | 3 | 3 | NORTH-WEST | 4.0 | 4.0 | BOTH | 4,5 or 5,4 | - | NORTH | WEST | - | t_side=0.5, t_top=0.5, equal |
| T034 | 2 | 3 | NORTH-WEST-H | 4.0 | 4.33 | H | 4 | 5 | NORTH-WEST-V | WEST | NORTH | t_side=0.33 < t_top=0.5, hits vertical first |
| T035 | 2 | 1 | NORTH-WEST-V | 4.25 | 4.0 | V | 5 | 4 | NORTH-WEST-H | NORTH | WEST | t_top=0.25 < t_side=0.33, hits horizontal first |
| T036 | 1 | 2 | NORTH-WEST-H | 4.0 | 4.25 | H | 4 | 5 | NORTH-WEST-V | WEST | NORTH | t_side=0.25 < t_top=0.33, hits vertical first |
| T037 | 5 | 5 | NONE | 5.0 | 5.0 | N/A | N/A | N/A | - | - | - | At destination, no movement needed |
| T038 | 10 | 10 | SOUTH-EAST | 6.0 | 6.0 | BOTH | 6,5 or 5,6 | - | SOUTH | EAST | - | Far diagonal, t_side=t_top=0.2, 45° angle |
| T039 | 0 | 0 | NORTH-WEST | 4.0 | 4.0 | BOTH | 4,5 or 5,4 | - | NORTH | WEST | - | Far diagonal, t_side=t_top=0.2, 45° angle |
| T040 | 8 | 2 | NORTH-EAST | 6.0 | 4.0 | BOTH | 6,5 or 5,4 | - | NORTH | EAST | - | t_side=t_top=0.33, 45° angle |

## Column Definitions

- **ID**: Test identifier
- **dest_x, dest_y**: Destination subcell coordinates
- **optimal**: Optimal direction with affinity suffix (-V, -H, or no suffix for BOTH/N/A)
- **target_x, target_y**: Calculated target position using intersection formula
- **affinity**: H (horizontal-favoring), V (vertical-favoring), BOTH (equal), N/A (cardinal)
- **anchor_x, anchor_y**: Anchor subcell for diagonal reservations (N/A for cardinal)
- **alt1, alt2, alt3**: Alternative directions in priority order
- **notes**: Calculation details and corrections

## Test Coverage

### Pure Cardinal Directions (16 tests)
- **NORTH**: T001-T004 (4 tests)
- **SOUTH**: T005-T008 (4 tests)
- **WEST**: T009-T012 (4 tests)
- **EAST**: T013-T016 (4 tests)

### Diagonal with Single Affinity (12 tests)
- **NORTH-EAST-V**: T021, T022 (2 tests)
- **NORTH-EAST-H**: T024, T025, T026 (3 tests)
- **SOUTH-EAST-H**: T027 (1 test)
- **SOUTH-EAST-V**: T029 (1 test)
- **SOUTH-WEST-H**: T031, T032 (2 tests)
- **NORTH-WEST-H**: T034, T036 (2 tests)
- **NORTH-WEST-V**: T035 (1 test)

### Diagonal with BOTH Affinity (8 tests)
- **Adjacent diagonal**: T017, T018, T019, T020 (4 tests, t=1.0)
- **Equal angle**: T023, T028, T030, T033 (4 tests, t_side=t_top)

### Special Cases (4 tests)
- **At destination**: T037 (1 test)
- **Far 45° diagonals**: T038, T039, T040 (3 tests)

## Intersection Formula Reference

From actor_directing.txt lines 29-43:

```python
def intersection(Ax, Ay, w, h, Bx, By):
    # Actor at (Ax, Ay), moving toward destination (Bx, By)
    # w = 1 (subcell width), h = 1 (subcell height)

    t_side = w / abs(Bx - Ax)  # Time to hit vertical edge
    t_top = h / abs(By - Ay)   # Time to hit horizontal edge

    if t_side < t_top:
        # Hits vertical edge first → H-affinity
        y_side = Ay + t_side * (By - Ay)
        return (Ax ± w, y_side)
    elif t_top < t_side:
        # Hits horizontal edge first → V-affinity
        x_top = Ax + t_top * (Bx - Ax)
        return (x_top, Ay ± h)
    else:
        # Equal → BOTH affinity (corner)
        return (Ax ± w, Ay ± h)
```

## Corrections from Original Document

1. **Line 119**: WEST alternatives were missing
2. **Line 120**: EAST alternatives incorrectly listed as "SW-V | NW-V", fixed to "SE-H | NE-H"
3. **Line 127**: Destination (9,4) was labeled "SOUTH-WEST-V", corrected to "NORTH-EAST-H"
4. **Line 128**: Typo "NORT-WEST" fixed to "NORTH-WEST"
5. **Line 130**: Target coordinates corrected from (4.333, 4) to (4.25, 4.0)
