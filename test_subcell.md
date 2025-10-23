# Sub-Cell Movement Test Plan

## Test 1: Single Actor to Single Destination

**Setup:**
1. Start the GUI (`cargo run --bin rustgame3`)
2. Press 'G' until sub-cell grid shows "3x3"
3. Press 'S' to enable sub-cell movement
4. Press 'O' at position (10, 10) to spawn an actor

**Test Steps:**
1. Press 'P' at position (20, 10) to set destination
2. Observe:
   - Actor should move in sub-cell steps (3x3 grid)
   - Green circle = current sub-cell
   - Yellow circle = reserved sub-cell
   - Orange circle = final destination
   - Status shows "SubCell Movement: ON (reservations: X)"

**Expected Behavior:**
- Actor moves smoothly from sub-cell to sub-cell
- When centered on current sub-cell, it reserves next sub-cell toward destination
- When closer to reserved than current, it switches and centers on new sub-cell
- Eventually reaches destination and reservations drop to 0

## Test 2: Multiple Actors to Same Destination

**Setup:**
1. Enable sub-cell movement (press 'S')
2. Enable 3x3 sub-cell display (press 'G')
3. Spawn 3 actors at different positions:
   - Actor 1: (5, 10)
   - Actor 2: (10, 15)
   - Actor 3: (15, 10)

**Test Steps:**
1. Press 'P' at position (10, 10) to send all to same destination
2. Observe:
   - All actors navigate toward same cell
   - Sub-cells get reserved on first-come-first-served basis
   - Actors that can't reserve preferred sub-cell try alternatives
   - Actors wait if all neighboring sub-cells are reserved

**Expected Behavior:**
- Multiple actors converge on destination
- Sub-cell reservations prevent collisions
- Actors take alternative routes when blocked
- Reservation count increases as actors move
- All actors eventually reach vicinity of destination

## Test 3: Toggling Between Modes

**Setup:**
1. Spawn 2 actors
2. Normal mode: Press 'P' to set destination
3. Observe pathfinding behavior

**Test Steps:**
1. Press 'S' to enable sub-cell movement
2. Observe actors switch to sub-cell mode
3. Press 'S' again to disable
4. Observe actors switch back to pathfinding

**Expected Behavior:**
- Toggle clears/sets appropriate state
- Status line updates correctly
- Actors transition smoothly between modes

## Test 4: Sub-Cell Navigation Around Obstacles

**Setup:**
1. Enable sub-cell movement
2. Draw a wall blocking direct path
3. Spawn actor on one side
4. Set destination on other side

**Test Steps:**
1. Observe actor navigating around obstacle using sub-cells
2. Note: As per spec, collisions and blocked cells are IGNORED (prototype)

**Expected Behavior:**
- Actor may walk through walls (this is intentional for the prototype!)
- Focus is on sub-cell reservation system, not collision detection

## Visual Legend

When sub-cell movement is enabled:
- **Purple square**: Actor body
- **Magenta dot**: Actor center position
- **Green circle**: Current sub-cell center
- **Yellow circle**: Reserved sub-cell center
- **Yellow line**: Path from actor to reserved sub-cell
- **Orange circle**: Final destination

## Debug Information

Status line shows:
- `SubCell: None/2x2/3x3` - Display mode
- `SubCell Movement: ON/OFF (reservations: N)` - Movement mode and active reservations

Console output shows:
- "Sub-cell movement: ENABLED/DISABLED"
- "Sub-cell destination set: (x, y) for N actors"
