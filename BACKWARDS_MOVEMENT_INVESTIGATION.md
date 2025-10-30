# Backwards Movement Investigation Report

**Date**: 2025-10-30
**Investigation**: Distance rule filter effectiveness and backwards movement causes

## Executive Summary

Investigated reported backwards movements in actor navigation. Found **3 distinct categories** with one major discovery:

1. ⚠️ **"PSC_ALIGNMENT" backwards** (124 cases, 55.4% of initial moves): **MISNAMED** - DestinationDirect mode does NOT implement PSC_ALIGNMENT state. These movements need re-investigation.
2. ✅ **Distance rule filter** for diagonal reservation: WORKING CORRECTLY (0 violations)
3. ⚠️  **Destination overshoot** (10 cases, 0.3% of navigation moves): MINOR BUG - actors overshoot destination by 1-4 pixels

**Critical Finding**: DestinationDirect implementation does not follow `actor_states.txt` 4-state design. PSC_ALIGNMENT state is not implemented.

## Investigation Details

### 1. "PSC_ALIGNMENT" Backwards Movements (MISNAMED - See Follow-up)

**Count**: 124 out of 224 initial movements (55.4%)
**Status**: ⚠️ **REQUIRES CLARIFICATION** - See Follow-up Discovery section

#### What the Test Actually Measured

The automated simulation test (test_automated_simulation.rs) categorized movements as:
- "PSC_ALIGNMENT phase": Movements before actor first reserves a subcell
- "Navigation phase": Movements after actor has a reservation

**However**: DestinationDirect mode does NOT implement PSC_ALIGNMENT state (see Follow-up Discovery below). These "PSC_ALIGNMENT backwards" movements are actually:
1. Initial movements in the very first reserved subcell
2. Movements that happen to increase distance due to subcell boundary constraints
3. **Not** movements toward nearest subcell center (as design document describes)

#### Design Document vs Implementation

**Design document** (actor_states.txt) describes PSC_ALIGNMENT as:
- Actor moves to **nearest** subcell center first
- Destination is **ignored** during alignment
- Backwards movement relative to destination is **expected** during this phase

**Actual implementation** (DestinationDirect):
- Actor reserves subcells **toward destination** immediately
- No alignment-to-center phase exists
- "Backwards" movements in early frames are likely subcell boundary effects, not intentional alignment

#### Conclusion

The 124 "PSC_ALIGNMENT backwards" movements are **not actually PSC_ALIGNMENT** behavior. The categorization in test_automated_simulation.rs is a misnomer. These should be re-investigated as potential navigation issues, not accepted as "by design".

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

###  1. "PSC_ALIGNMENT" Backwards Movements (REVISED)

**Priority**: HIGH - requires investigation

**Issue**: 124 backwards movements (55.4%) in early frames, previously assumed to be "PSC_ALIGNMENT by design", but PSC_ALIGNMENT state is not implemented.

**Action needed**:
1. Re-investigate these 124 movements with correct understanding
2. Determine if they are:
   - Legitimate subcell boundary effects (acceptable)
   - Bugs in initial reservation logic (needs fix)
   - Side effects of missing PSC_ALIGNMENT implementation (needs design decision)

### 2. PSC_ALIGNMENT State Implementation

**Priority**: MEDIUM - design decision required

**Options**:
1. **Implement PSC_ALIGNMENT**: Follow actor_states.txt 4-state design
   - Actors align to nearest subcell center before navigating
   - More predictable spawn behavior
   - Adds alignment phase overhead

2. **Accept current behavior**: DestinationDirect as-is
   - Update actor_states.txt to match implementation
   - Remove test_psc_alignment_only.rs
   - Document that DestinationDirect skips alignment

### 3. Distance Rule Filter

**No action needed** - filter is working correctly (0 violations).

**Consider**: Add similar distance checks to other movement contexts (e.g., H/V movement).

### 4. Destination Overshoot Bug

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

The original concern about backwards movements during diagonal reservation was **resolved**: the distance rule filter is working correctly (0 violations).

**However**, we discovered a critical gap between design and implementation:
1. **PSC_ALIGNMENT state is not implemented** in DestinationDirect mode
2. The 124 "PSC_ALIGNMENT backwards" movements were misnamed - they are not the intentional alignment behavior described in actor_states.txt
3. These 124 backwards movements (55.4% of initial movements) **require re-investigation** with correct understanding

**Minor findings**:
- Destination overshoot bug affects 0.3% of movements near destination (low priority)
- Distance rule filter successfully prevents backwards diagonal movements during navigation

## Follow-up Discovery: PSC_ALIGNMENT Not Implemented

**Date**: 2025-10-30 (continued investigation)

### Finding

During test creation for PSC_ALIGNMENT behavior, discovered that **DestinationDirect mode does not implement the PSC_ALIGNMENT state** described in `design/actor_states/actor_states.txt`.

**Design Document** (actor_states.txt Section 3):
- State 1: NO_SUBCELL → Actor tries to reserve **nearest** subcell (within 4-cell rectangle around position)
- State 2: PSC_ALIGNMENT → Actor moves to **center** of reserved subcell
- State 3: IDLE → Actor waits at center, ready to navigate
- State 4: MOVE → Actor navigates toward destination

**Actual Implementation** (src/actor.rs:3099+):
- NO_SUBCELL → Actor tries to reserve subcell **toward destination** (not nearest)
- MOVE → Actor immediately navigates (no alignment phase)

**Impact**:
- No PSC_ALIGNMENT phase exists in current code
- Actors never align to nearest subcell center before navigating
- "PSC_ALIGNMENT backwards movements" category from this report actually refers to **manual testing with different mode** or **conceptual design**, not actual DestinationDirect behavior

### Test Status

**test_psc_alignment_only.rs**: Created to test PSC_ALIGNMENT, but fails because DestinationDirect doesn't implement this state. Test expects actors to align to nearest subcell centers, but actors immediately start reserving toward destination instead.

### Recommendation

**Option 1**: Accept that DestinationDirect doesn't follow actor_states.txt design
- Mark actor_states.txt as "proposed design" not implemented
- Remove test_psc_alignment_only.rs as testing non-existent behavior

**Option 2**: Implement PSC_ALIGNMENT in DestinationDirect mode
- Add initial alignment phase when actor first reserves a subcell
- Follow actor_states.txt 4-state machine design
- Retest with test_psc_alignment_only.rs
