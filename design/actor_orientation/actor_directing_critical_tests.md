# Critical Test Subset for Actor Directing Implementation

This document identifies the most critical tests to validate the position-aware diagonal reservation algorithm.

## Testing Strategy

### Phase 1: Baseline Validation (40 tests)
**Purpose**: Verify algorithm works correctly when actor is at PSC center (traditional assumption)

**Tests**: All baseline tests from `actor_directing_tests.tsv`
- T001-T016: Cardinal directions (16 tests)
- T017-T036: Diagonal directions from PSC center (20 tests)
- T037-T040: Special cases (4 tests)

**Success Criteria**:
- ✅ All 40 baseline tests pass
- ✅ Targets match expected values within epsilon
- ✅ Affinities match expected values
- ✅ Anchors are correctly selected

---

### Phase 2: Position-Aware Core (36 tests)
**Purpose**: Validate that actor position affects affinity and target calculation

**Test Selection**: For each of 12 single-affinity diagonal tests, run 3 critical positions

**3 Critical Positions**:
1. **P1 (PSC Center)**: Baseline comparison
2. **P2 (Rectangle Center)**: Most common after partial movement
3. **P3 (Diagonal Corner)**: Boundary condition (already at target)

**Test IDs**:
```
T021_P1, T021_P2, T021_P3  # dest=(6,1) V-affinity baseline
T022_P1, T022_P2, T022_P3  # dest=(7,2) V-affinity baseline
T024_P1, T024_P2, T024_P3  # dest=(8,3) H-affinity baseline
T025_P1, T025_P2, T025_P3  # dest=(9,3) H-affinity baseline
T026_P1, T026_P2, T026_P3  # dest=(9,4) H-affinity baseline
T027_P1, T027_P2, T027_P3  # dest=(8,7) H-affinity baseline
T029_P1, T029_P2, T029_P3  # dest=(7,8) V-affinity baseline
T031_P1, T031_P2, T031_P3  # dest=(2,7) H-affinity baseline
T032_P1, T032_P2, T032_P3  # dest=(1,7) H-affinity baseline
T034_P1, T034_P2, T034_P3  # dest=(2,3) H-affinity baseline
T035_P1, T035_P2, T035_P3  # dest=(2,1) V-affinity baseline
T036_P1, T036_P2, T036_P3  # dest=(1,2) H-affinity baseline
```

**Success Criteria**:
- ✅ P1 results match baseline tests (Phase 1)
- ✅ P2 shows different affinity/target for some cases
- ✅ P3 always results in target = actor position, affinity = BOTH
- ✅ All targets lie on rectangle boundaries
- ✅ Ray from actor through target points toward destination

---

### Phase 3: Edge Cases (24 tests)
**Purpose**: Validate behavior when actor is on rectangle edges

**Test Selection**: For 4 representative diagonal cases, test all 4 edge positions

**4 Representative Cases**:
- **T021** (dest=6,1): V-affinity baseline, NORTH-EAST, close range
- **T024** (dest=8,3): H-affinity baseline, NORTH-EAST, medium range
- **T029** (dest=7,8): V-affinity baseline, SOUTH-EAST, medium range
- **T034** (dest=2,3): H-affinity baseline, NORTH-WEST, close range

**4 Edge Positions per case**:
- **P4 (Top Edge)**: Tests behavior on horizontal boundary
- **P5 (Bottom Edge)**: Tests behavior on opposite horizontal boundary
- **P6 (Left Edge)**: Tests behavior on vertical boundary
- **P7 (Right Edge)**: Tests behavior on opposite vertical boundary

**Test IDs**:
```
T021_P4, T021_P5, T021_P6, T021_P7
T024_P4, T024_P5, T024_P6, T024_P7
T029_P4, T029_P5, T029_P6, T029_P7
T034_P4, T034_P5, T034_P6, T034_P7
```

**Success Criteria**:
- ✅ Actor on edge → t=0 for that edge
- ✅ Target = actor position OR on perpendicular edge
- ✅ Affinity determined by non-zero t value
- ✅ No division by zero or NaN values

---

### Phase 4: Corner Cases (16 tests)
**Purpose**: Validate behavior at rectangle corners

**Test Selection**: For 4 representative diagonal cases, test corner positions

**2 Corner Positions per case**:
- **P8 (Bottom-Left Corner)**: Usually same as PSC center
- **P9 (Bottom-Right Corner)**: At anchor position

**Test IDs**:
```
T021_P8, T021_P9
T024_P8, T024_P9
T029_P8, T029_P9
T034_P8, T034_P9
T031_P8, T031_P9  # Additional SW case
T032_P8, T032_P9  # Additional SW case
T035_P8, T035_P9  # Additional NW case
T036_P8, T036_P9  # Additional NW case
```

**Success Criteria**:
- ✅ P8 matches P1 when both are at PSC center
- ✅ P9 (anchor corner) produces minimal movement
- ✅ Corner positions result in single or BOTH affinity
- ✅ No unexpected behavior at boundaries

---

## Testing Progression

### Minimal Viable Test Suite (76 tests)
**Phases**: 1 + 2
**Tests**: 40 baseline + 36 position-aware core
**Time**: ~1-2 hours implementation + testing
**Confidence**: Medium - validates core algorithm

### Recommended Test Suite (100 tests)
**Phases**: 1 + 2 + 3
**Tests**: 40 baseline + 36 core + 24 edge cases
**Time**: ~2-4 hours implementation + testing
**Confidence**: High - validates edge cases

### Comprehensive Test Suite (148 tests)
**Phases**: 1 + 2 + 3 + 4 + selection from remaining
**Tests**: All critical tests + selected exhaustive tests
**Time**: ~4-8 hours implementation + testing
**Confidence**: Very High - validates all edge cases

---

## Test Execution Order

### Recommended Sequence:

1. **Cardinal Direction Tests (T001-T016)**: 16 tests
   - Purpose: Verify basic direction selection
   - No position variance (always center)
   - Should pass quickly

2. **Diagonal Baseline P1 Tests**: 12 tests
   - Purpose: Verify position-aware algorithm matches baseline when actor at PSC center
   - Tests: T021_P1, T022_P1, T024_P1, ..., T036_P1
   - Should match original baseline results exactly

3. **Rectangle Center P2 Tests**: 12 tests
   - Purpose: Verify affinity changes based on actor position
   - Tests: T021_P2, T022_P2, T024_P2, ..., T036_P2
   - Expected: Some affinity differences from P1

4. **Diagonal Corner P3 Tests**: 12 tests
   - Purpose: Verify boundary condition handling
   - Tests: T021_P3, T022_P3, T024_P3, ..., T036_P3
   - Expected: All result in target = actor position, affinity = BOTH

5. **Edge Position Tests (P4-P7)**: 24 tests (if doing Phase 3)
   - Purpose: Verify t=0 handling
   - Expected: No crashes, correct affinity determination

6. **Corner Position Tests (P8-P9)**: 16 tests (if doing Phase 4)
   - Purpose: Verify corner cases
   - Expected: P8 matches P1, P9 produces minimal movement

---

## Critical Test Cases (Must Pass)

These specific tests are the most important for validating correctness:

### 1. T021_P1 → T021_P2 Comparison
**Why**: Shows how rectangle center position changes V-affinity
- P1: (5.0, 5.0) → (5.25, 4.0) V-affinity
- P2: (5.5, 4.5) → (5.54, 4.0) V-affinity (still V, but different target)
- **Validates**: Target calculation from non-PSC position

### 2. T024_P1 → T024_P2 Comparison
**Why**: Shows H-affinity maintained from different positions
- P1: (5.0, 5.0) → (6.0, 4.33) H-affinity
- P2: (5.5, 4.5) → (6.0, 4.2) H-affinity (still H, closer to diagonal)
- **Validates**: Affinity consistency across positions

### 3. T021_P7 (Actor on Right Edge)
**Why**: Tests t=0 case (already at vertical boundary)
- Actor: (6.0, 4.5) - on right edge
- Expected: target = (6.0, 4.0), H-affinity, t_vertical = 0
- **Validates**: Edge case handling

### 4. T024_P5 vs T024_P6
**Why**: Shows affinity difference based on which edge actor is on
- P5 (bottom edge): H-affinity (hits right edge first)
- P6 (left edge): H-affinity (hits right edge first)
- **Validates**: Ray-casting from different edge positions

### 5. T022_P5 (Bottom Edge, Affinity Flip)
**Why**: Position can cause affinity to flip from baseline
- Baseline (P1): V-affinity from (5.0, 5.0)
- P5: H-affinity from (5.5, 5.0) - bottom edge
- **Validates**: Dynamic affinity determination

### 6. T021_P3, T024_P3, T029_P3, T034_P3 (All Diagonal Corners)
**Why**: Boundary condition - actor already at target
- All should produce: target = actor position, affinity = BOTH
- **Validates**: No-movement case

---

## Debugging Strategy

### If Tests Fail:

**Symptom**: Wrong affinity
- **Check**: t-value calculation (vertical vs horizontal)
- **Verify**: Ray direction normalization
- **Confirm**: Rectangle bounds definition

**Symptom**: Wrong target position
- **Check**: Parametric ray intersection formula
- **Verify**: t-value used matches affinity
- **Confirm**: Target clamping to rectangle bounds

**Symptom**: NaN or infinity values
- **Check**: Division by zero in ray direction
- **Verify**: Destination is not inside rectangle (should never happen)
- **Confirm**: Actor position is within or clamped to rectangle

**Symptom**: P1 doesn't match baseline
- **Check**: Offset calculation (subcell center positioning)
- **Verify**: Rectangle bounds calculation
- **Confirm**: Epsilon tolerance for floating-point comparison

---

## Test Output Format

Recommended test output for debugging:

```
Test: T021_P2 (Rectangle Center, dest=6,1)
  Actor: (5.5, 4.5) in PSC (5,5)
  Destination: (6, 1)
  Rectangle: [5.0-6.0, 4.0-5.0]

  Ray Direction: (0.196, -0.981) normalized

  t-values:
    t_right: 0.909 (hits right edge at x=6.0)
    t_top: 0.182 (hits top edge at y=4.0)

  Result: t_top < t_right → V-affinity
  Target: (5.54, 4.0)
  Anchor: (5, 4)

  Expected: (5.54, 4.0), V, (5,4)
  Status: ✅ PASS
```

---

## Success Metrics

### Phase 1 Complete (Baseline):
- 40/40 tests pass
- All cardinal directions work
- All diagonal directions from PSC center work

### Phase 2 Complete (Position-Aware Core):
- 76/76 tests pass (40 + 36)
- P1 results match baseline
- P2 shows position-dependent behavior
- P3 handles boundary conditions

### Phase 3 Complete (Edge Cases):
- 100/100 tests pass (40 + 36 + 24)
- Edge positions handled correctly
- No division errors or NaN values

### Phase 4 Complete (Comprehensive):
- 148+/148+ tests pass
- Corner cases validated
- System ready for production use
