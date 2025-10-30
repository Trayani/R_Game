# Backwards Movement Investigation Report

**Date**: 2025-10-30
**Investigation**: Distance rule filter effectiveness and backwards movement causes

## Executive Summary

Investigated reported backwards movements in actor navigation. Found **3 distinct categories** of backwards movement with different causes and significance:

1. ✅ **PSC_ALIGNMENT backwards** (124 cases, 55.4% of alignment moves): EXPECTED per `actor_states.txt` design
2. ✅ **Distance rule filter** for diagonal reservation: WORKING CORRECTLY (0 violations)
3. ⚠️  **Destination overshoot** (10 cases, 0.3% of navigation moves): MINOR BUG - actors overshoot destination by 1-4 pixels

## Investigation Details

### 1. PSC_ALIGNMENT Backwards Movements (EXPECTED)

**Count**: 124 out of 224 alignment movements (55.4%)
**Status**: ✅ **CORRECT BEHAVIOR per actor_states.txt**

#### Design Document Reference

From `design/actor_states/actor_states.txt` Section 2:

> **State 2: PRIMARY SUBCELL ALIGNMENT "PSC_ALIGNMENT"**
>
> Actor has reserved a primary subcell and must align to its center.
>
> **EVERY FRAME Behavior**:
> Actor moves **DIRECTLY** toward the primary subcell center.
>
> **Destination Handling**: If destination is set during PSC_ALIGNMENT, it is stored and **ignored until alignment completes**.

#### Why This Causes Backwards Movement

1. Actor spawns at arbitrary float position (e.g., 477.0, 250.0)
2. Destination is set (e.g., 150.0, 600.0) - southwest direction
3. Actor MUST first move to nearest subcell center (e.g., 457.5, 245.0)
4. This center might be in the OPPOSITE direction from destination
5. Actor moves toward center (backwards relative to destination)
6. Only AFTER reaching center can navigation begin

#### Example from Test

```
Actor at (477.0, 250.0)
Destination: (150.0, 600.0) - needs to go LEFT and DOWN
Nearest subcell center: (457.5, 245.0)
Move: (477.0, 250.0) → (475.5, 249.6)
  - X: correct direction (left toward 150.0)
  - Y: BACKWARDS (up from 250→249.6, away from 600.0)

Result: Y distance INCREASES from 350.0 to 350.4 (+0.4px)
Status: ✅ EXPECTED - PSC_ALIGNMENT ignores destination
```

### 2. Distance Rule Filter for Diagonal Reservation (WORKING)

**Count**: 0 violations in diagonal reservation
**Status**: ✅ **WORKING CORRECTLY**

#### Test Coverage

Created `tests/test_actor3_backwards.rs` to reproduce specific scenario from action_log.db:
- Actor 3 at subcell (9,13,1,1)
- Destination southwest (270.0, 750.0)
- SE diagonal candidate (10,14,0,0) would increase X distance by +15.0px
- **Result**: SE diagonal correctly FILTERED, actor fell back to vertical movement

#### Distance Rule Implementation

From `src/subcell.rs:168-209` (`violates_distance_rule`):

```rust
let (curr_x, curr_y) = self.to_screen_center_with_offset(...);
let (other_x, other_y) = other.to_screen_center_with_offset(...);

let curr_dist_x = (dest_x - curr_x).abs();
let new_dist_x = (dest_x - other_x).abs();

let tolerance = subcell_width * tolerance_multiplier;  // 15.0 * 0.6 = 9.0

// Violates if EITHER distance increases beyond tolerance
(new_dist_x - curr_dist_x) > tolerance || (new_dist_y - curr_dist_y) > tolerance
```

#### Verification

- Manual Python calculation: SE should be filtered ✓
- Unit test result: SE was filtered ✓
- Simulation test: 0 diagonal reservation violations ✓

**Conclusion**: Distance rule filter is functioning correctly. The diagonal-to-cardinal fallback algorithm prevents backwards diagonal movements.

### 3. Destination Overshoot During Navigation (MINOR BUG)

**Count**: 10 out of 3059 navigation movements (0.3%)
**Status**: ⚠️ **MINOR BUG** - needs investigation

#### Characteristics

All 10 cases occurred:
- Very close to destination (within 1-3 pixels)
- At Y≈600 (destination Y=600)
- Moving LEFT (correct toward X=150) but DOWN (overshooting Y=600)

#### Example Cases

```
Actor 9: (161.9, 600.0) → (160.9, 601.2)
  X: 161.9 → 160.9 (-1.0) ✓ toward dest 150.0
  Y: 600.0 → 601.2 (+1.2) ✗ away from dest 600.0

Actor 7: (162.1, 600.0) → (161.0, 601.2)
  X: 162.1 → 161.0 (-1.1) ✓ toward dest 150.0
  Y: 600.0 → 601.2 (+1.2) ✗ away from dest 600.0
```

#### Hypothesis

Actors have a **locked target position** from diagonal movement calculation. They continue moving toward this locked target even after one coordinate passes the destination, causing overshoot in that dimension.

**Not checked by distance rule** because:
- Distance rule only applies during reservation (IDLE → MOVE transition)
- Once in MOVE state, actor follows locked target without recalculation
- No mid-flight destination proximity check

#### Severity

**LOW** - only 0.3% of navigation movements, magnitude 1-4 pixels, occurs at destination approach

## Test Infrastructure

### New Tests Created

1. **`tests/test_actor3_backwards.rs`**
   - Reproduces specific Actor 3 scenario from action_log.db
   - Verifies SE diagonal is filtered when X distance would increase
   - Status: ✅ PASSING

2. **`tests/test_automated_simulation.rs`**
   - Spawns 10 actors, sets southwest destinations
   - Tracks PSC_ALIGNMENT vs navigation separately
   - Reports backwards movements by phase
   - Status: ⚠️ FAILING (10 navigation backwards due to overshoot bug)

### Diagnostic Logging Added

**File**: `src/actor.rs:1376-1384`

Added detailed distance check logging for diagonal candidates:
- Current and candidate subcell centers
- Current and new distances (X and Y)
- Distance changes and tolerance
- Violation decision

Messages written to `diagnostic_messages` → `action_log.db`

## Recommendations

###  1. PSC_ALIGNMENT Backwards Movements

**No action needed** - this is correct behavior per design document.

**Documentation**: Updated test file header to explain PSC_ALIGNMENT phase.

### 2. Distance Rule Filter

**No action needed** - filter is working correctly.

**Consider**: Add similar distance checks to other movement contexts (e.g., H/V movement).

### 3. Destination Overshoot Bug

**Priority**: Low (0.3% occurrence, minimal impact)

**Investigation needed**:
1. Check if locked target calculation considers destination proximity
2. Consider adding mid-flight destination check in MOVE state
3. Evaluate if overshooting by 1-4 pixels is acceptable tolerance

**Potential fix**:
- Clamp movement to not pass destination in any dimension
- OR add destination proximity check before following locked target
- OR consider this acceptable behavior (actor will correct on next PSC switch)

## Conclusion

The original concern about backwards movements during diagonal reservation was **resolved**: the distance rule filter is working correctly.

We discovered two additional categories:
1. PSC_ALIGNMENT backwards movements are **by design** and necessary
2. Destination overshoot is a **minor bug** affecting 0.3% of movements near destination

The diagonal-to-cardinal fallback algorithm successfully prevents backwards diagonal movements during navigation.
