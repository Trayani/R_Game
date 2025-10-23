# Test: Actors Spread to Different Cells

## Objective
Verify that when multiple actors are sent to the same destination point, they spread out to different CELLS (not just different sub-cells). Sub-cells are only for intermediate movement; final destinations must be at cell level.

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
- Console should print: "Sub-cell destinations set: (X, Y) for 6 actors (spread across 6 cells)"
- All 6 actors should start moving toward different cells around that point

### Step 3: Observe Movement
Watch the actors as they approach the destination:

**What to Look For:**

✓ **CORRECT** - Actors spread to different CELLS:
  - Actor 1 → target cell (clicked position)
  - Actor 2 → cell to the North
  - Actor 3 → cell to the East
  - Actor 4 → cell to the South
  - Actor 5 → cell to the West
  - Actor 6 → cell to the Northeast
  - (Pattern continues in spiral for more actors)

✓ **CORRECT** - Each actor stops at different cell:
  - When actors arrive, they occupy different CELLS, not just different sub-cells
  - Actors are visibly separated by at least one cell width
  - Final positions form a cluster pattern around target

✓ **CORRECT** - Sub-cell navigation during movement:
  - Actors use sub-cells to navigate smoothly DURING movement
  - But final resting positions are at cell centers (different cells)
  - Reservation system prevents collisions during travel

✗ **WRONG** - All actors go to same cell:
  - All 6 actors end up in the same grid cell
  - Actors overlap at destination
  - This should NOT happen with cell-level spreading

✗ **WRONG** - Actors stuck/jittering:
  - Actors freeze before reaching destination
  - Rapid back-and-forth movement
  - This indicates reservation conflicts

## Advanced Test: 10+ Actors

For a more challenging test:
1. Spawn 10-15 actors in a cluster
2. Set destination to a single cell
3. Observe spread pattern:
   - Actor 1 → target cell
   - Actors 2-9 → 8 surrounding cells (ring 1)
   - Actors 10-24 → next ring of cells (ring 2)
   - Pattern continues spiraling outward

**Expected:** Spiral distribution across CELLS, each actor gets its own cell

## Debug Information

### Console Output
When you press P, look for:
```
Sub-cell destinations set: (20, 20) for 6 actors (spread across 6 cells)
```

The number of cells should match the number of actors.

### Visual Indicators
- **Blue circles**: Actors
- **Grid cells**: Each large square is one cell
- **3x3 sub-grid**: Only for visualizing intermediate movement (when G is toggled)
- **Status line**: Shows reservation count during movement

### Common Issues

**Issue**: Actors still merge at destination
- **Cause**: Sub-cell movement not enabled
- **Fix**: Press S to enable

**Issue**: Want to see sub-cell navigation
- **Cause**: Sub-cell grid not visible
- **Fix**: Press G until "3x3" mode is active (helps visualize intermediate movement)

**Issue**: Actors don't move at all
- **Cause**: Destination is blocked or no path
- **Fix**: Click on a free (green/dark gray) cell

## Success Criteria

Test PASSES if:
1. All 6 actors reach different destination CELLS
2. Each actor stops at a different cell (clearly separated by cell boundaries)
3. Destination cells form a spiral pattern around the clicked point
4. No jittering or stuck behavior during travel
5. Sub-cells are used smoothly for navigation during movement
6. You can see 6 distinct circles at 6 different cells when movement completes

Test FAILS if:
1. Multiple actors merge into same CELL (not just same sub-cell)
2. Actors get stuck before reaching destination
3. Rapid jittering/fighting for position
4. All actors aim for the exact same cell instead of spreading
