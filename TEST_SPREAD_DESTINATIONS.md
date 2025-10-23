# Test: Actors Don't Merge at Destination

## Objective
Verify that when multiple actors are sent to the same destination, they do NOT merge into the same position. Each actor should occupy a unique sub-cell.

## Setup

1. Run the visual demo:
   ```bash
   cargo run --bin rustgame3
   ```

2. Enable sub-cell movement:
   - Press **S** to toggle sub-cell movement ON
   - Status line should show "Sub-cell movement: enabled"

3. Enable sub-cell visualization:
   - Press **G** to cycle to 3x3 sub-cell grid display
   - You should see a 3x3 grid within each cell

## Test Scenario: 6 Actors Close Together

### Step 1: Spawn 6 Actors
Press **O** and click to spawn actors at these approximate positions:
1. Click near (300, 300) - center of grid
2. Click near (330, 300) - slightly right
3. Click near (300, 330) - slightly down
4. Click near (330, 330) - diagonal
5. Click near (360, 300) - further right
6. Click near (300, 360) - further down

You should now have 6 actors clustered in a 3x3 cell area.

### Step 2: Set Common Destination
Press **P** and click on a distant cell (e.g., around (600, 600))

**Expected Behavior:**
- Console should print: "Sub-cell destinations set: (X, Y) for 6 actors (spread across 6 sub-cells)"
- All 6 actors should start moving toward the destination

### Step 3: Observe Movement
Watch the actors as they approach the destination:

**What to Look For:**

✓ **CORRECT** - Actors spread out as they approach:
  - Each actor targets a different sub-cell in the destination area
  - Actor 1 → center sub-cell (1, 1)
  - Actor 2 → surrounding sub-cell (0, 0)
  - Actor 3 → surrounding sub-cell (1, 0)
  - Actor 4 → surrounding sub-cell (2, 0)
  - Actors 5-6 → other surrounding sub-cells or neighboring cells

✓ **CORRECT** - No merging:
  - When actors arrive, they occupy different positions
  - With 3x3 grid visible, you can see each actor in a different sub-cell
  - Actors maintain separation (no overlapping circles)

✓ **CORRECT** - Reservation system working:
  - Status line shows: "Sub-cell reservations: 6" (one per actor)
  - Actors don't fight for the same sub-cell
  - Smooth movement without jittering

✗ **WRONG** - All actors merge:
  - All 6 actors end up at exactly the same position
  - Only one actor circle visible (others hidden underneath)
  - This should NOT happen with the new implementation

✗ **WRONG** - Actors stuck/jittering:
  - Actors freeze before reaching destination
  - Rapid back-and-forth movement
  - This indicates reservation conflicts

## Advanced Test: 10+ Actors

For a more challenging test:
1. Spawn 10-15 actors in a cluster
2. Set destination to a single cell
3. Observe spread pattern:
   - First 9 actors → 9 sub-cells in target cell
   - Remaining actors → sub-cells in surrounding cells (N, E, S, W, NE, SE, SW, NW)

**Expected:** Spiral distribution pattern with no merging

## Debug Information

### Console Output
When you press P, look for:
```
Sub-cell destinations set: (20, 20) for 6 actors (spread across 6 sub-cells)
```

The number of sub-cells should match the number of actors (up to 81 for very large groups).

### Visual Indicators
- **Blue circles**: Actors
- **3x3 grid**: Sub-cell boundaries (when G is toggled to 3x3)
- **Status line**: Shows reservation count

### Common Issues

**Issue**: Actors still merge at destination
- **Cause**: Sub-cell movement not enabled
- **Fix**: Press S to enable

**Issue**: Can't see sub-cell positions clearly
- **Cause**: Sub-cell grid not visible
- **Fix**: Press G until "3x3" mode is active

**Issue**: Actors don't move at all
- **Cause**: Destination is blocked or no path
- **Fix**: Click on a free (green/dark gray) cell

## Success Criteria

Test PASSES if:
1. All 6 actors reach the destination area
2. Each actor occupies a different sub-cell (no overlapping)
3. No jittering or stuck behavior
4. Reservation count matches actor count
5. Visual separation is maintained (you can see 6 distinct circles)

Test FAILS if:
1. Actors merge into same position (only 1 circle visible)
2. Actors get stuck before reaching destination
3. Rapid jittering/fighting for position
4. Any actor disappears or behaves erratically
