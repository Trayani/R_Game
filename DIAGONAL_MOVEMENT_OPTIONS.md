# Diagonal Movement Problem and Solutions

## Problem: "Drunk" Diagonal Movement

Actors moving diagonally through the sub-cell system exhibit a "drunk" or "wobbling" appearance, constantly changing direction.

### Root Cause

The sub-cell movement algorithm uses **myopic greedy selection**:

1. **Direction recalculated every frame** (src/actor.rs lines 595-596)
   - Target direction = vector from current position to destination
   - Changes as actor moves, causing alignment scores to fluctuate

2. **Greedy local selection** (src/subcell.rs find_best_neighbors)
   - Only looks at immediate next sub-cell
   - No path planning or lookahead
   - Picks "best aligned" neighbor based on current frame's direction

3. **Diagonal ambiguity in 3×3 grid**
   - When moving diagonally (e.g., northeast), multiple neighbors have similar scores
   - Actor switches between: diagonal, right-then-up, up-then-right
   - Creates visible zigzag pattern

4. **No path memory**
   - Each sub-cell reservation decision is independent
   - No commitment to a direction
   - Results in constant micro-corrections

### Visual Example

Moving from southwest (0,2) to northeast (2,0) in a 3×3 sub-cell grid:

```
[0,0] [1,0] [2,0] ← Goal
[0,1] [1,1] [2,1]
[0,2] [1,2] [2,2] ← Start

Ideal path:     (0,2) → (1,1) → (2,0)  [smooth diagonal]
Actual path:    (0,2) → (1,2) → (1,1) → (2,1) → (2,0)  [staircase zigzag]
Or:             (0,2) → (0,1) → (1,1) → (1,0) → (2,0)  [different zigzag]
Or:             (0,2) → (1,1) → (1,2) → (2,1) → (2,0)  [wobble]
```

The actor constantly switches between these paths frame-by-frame, creating the "drunk" appearance.

## Solution Options

### Option 1: Sub-Cell Pathfinding

**Description:** Calculate a complete sub-cell path at the start using A* or similar algorithm, then follow it.

**Implementation:**
- Add A* pathfinding at sub-cell granularity
- Calculate path when destination is set
- Store path in actor state
- Follow path waypoints sequentially
- Recalculate when blocked or destination changes

**Pros:**
- Smoothest, most predictable movement
- Proper diagonal handling (no zigzag)
- Most realistic looking
- Can plan around predicted obstacles

**Cons:**
- Most complex implementation (~200+ lines)
- Need A* or Theta* for sub-cell grid
- Path recalculation overhead when blocked
- More memory per actor (store full path)
- Balancing path quality vs computation cost

**Best for:** Final production quality, when movement must look perfect

---

### Option 2: Directional Momentum/Smoothing ⭐ RECOMMENDED

**Description:** Add momentum to movement direction - commit to a direction for several frames instead of recalculating every frame.

**Implementation:**
- Add fields to Actor:
  ```rust
  subcell_movement_direction: Option<(f32, f32)>,  // Committed direction
  subcell_direction_frames: u32,  // Frames since direction set
  ```
- Keep using same direction for N frames (e.g., 8-10)
- Only recalculate direction when:
  - Blocked (reservation fails)
  - Direction not yet established
  - Stuck/no progress detected
- Optionally: Interpolate between old and new direction for smooth rotation

**Pros:**
- Simple implementation (~50 lines)
- Maintains reactive collision avoidance
- Significantly reduces wobble
- Works perfectly with existing reservation system
- Good balance of smoothness vs responsiveness

**Cons:**
- Still some minor zigzagging possible (much less visible)
- Not as perfect as full pathfinding
- Slightly less reactive to dynamic obstacles

**Best for:** Good quality with minimal complexity, recommended for this project

**Tuning parameters:**
- `DIRECTION_COMMIT_FRAMES`: How many frames to keep direction (8-15)
- `DIRECTION_BLEND_FRAMES`: How many frames to smoothly rotate (2-3)
- `MIN_PROGRESS_THRESHOLD`: Minimum movement to avoid "stuck" detection

---

### Option 3: Larger Sub-Cell Grid (Quick Fix)

**Description:** Change from 3×3 to 5×5 or 7×7 sub-cells per cell, reducing diagonal ambiguity.

**Implementation:**
- Change constants in SubCellCoord::from_screen_pos:
  ```rust
  let sub_x = (cell_local_x * 5.0).floor() as i32;  // Was 3.0
  let sub_y = (cell_local_y * 5.0).floor() as i32;  // Was 3.0
  ```
- Update bounds checks from 0..=2 to 0..=4
- Update get_neighbors() to handle larger grid

**Pros:**
- Extremely simple (change a few numbers)
- More granular movement
- Diagonal ambiguity reduced (more directional options)
- No algorithm changes needed

**Cons:**
- More memory for reservations (25 or 49 sub-cells per cell)
- Slower reservation HashMap lookups
- Doesn't fully eliminate the problem, just reduces it
- May need to adjust actor speed for smooth appearance

**Best for:** Quick test to see if granularity helps, temporary fix

**Grid size recommendations:**
- 5×5: Good balance, 25 sub-cells per cell
- 7×7: Very smooth, 49 sub-cells per cell (may be overkill)
- 9×9: Probably too fine, performance concerns

---

## Hybrid Approach (Advanced)

Combine Option 2 + Option 3:
- Use 5×5 grid for better granularity
- Add directional momentum for smooth committed movement
- Best of both worlds

## Implementation Status

**Current:** Option 2 (Directional Momentum) - IN PROGRESS

**Files to modify:**
- `src/actor.rs`: Add momentum fields, update update_subcell() logic
- `src/subcell.rs`: Possibly add direction smoothing utilities

**Testing approach:**
1. Implement basic momentum (commit for N frames)
2. Test with diagonal movement
3. Add interpolation if needed
4. Tune parameters for best feel
