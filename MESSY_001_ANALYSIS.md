# Test messy_001 Failure Analysis

## Test Information

**Test**: `test2x_messy_001_6388_to_6139`
**C# Source**: ProtoTests.cs:235
**C# Test Call**: `test2(6388, 6139, 1, false, 6141, 6307, 6306, 6389)`
**Mapping**: `test2 => test3(start, dest, size, reverse=false, messyY=true, messyX=false, exPath)`

**Parameters**:
- Start: 6388 at (80, 76)
- Dest: 6139 at (80, 73)
- messyX: false
- messyY: **true**
- Expected waypoints: [6141, 6307, 6306, 6389]

## C# Behavior (CORRECT)

C# successfully finds the path with waypoints `[6141, 6307, 6306, 6389]`:

```
[pfxXX] corner has 1 corners
[pfxXX] First few corners: id=6389, dist=1.41, tag=Base
[pfxXX] destCorner has 2 corners
[pfxXX] Dest corners: id=6140, dist=1.00, id=6141, dist=2.00
```

**Key observations**:
- Start (6388) discovers 1 interesting corner: 6389
- **Dest (6139) discovers 2 interesting corners: 6140, 6141**
- Pathfinding successfully connects through: 6388 → 6389 → 6306 → 6307 → 6141 → 6139
- Total distance: 7.41

## Rust Behavior (FAILS)

Rust returns `None` (no path found):

```
--- Corners from start 6388 ---
Start visible cells: 4
Interesting corners from start: 1
  [0] ID 6389 at (81,76) - dist 1.00

--- Corners from dest 6139 ---
Dest visible cells: 0          ⚠️ PROBLEM!
Interesting corners from dest: 0
⚠️  DEST HAS NO INTERESTING CORNERS!
```

## ROOT CAUSE: Raycasting Bug with messyY=true

The issue is in the **raycasting function** when `messy_y=true`.

**Symptom**: `raycast(&grid, dest_x, dest_y, messy_x=false, messy_y=true)` returns **0 visible cells**.

**Why this causes path failure**:
1. Destination raycast returns empty set
2. No interesting corners can be computed for destination (requires visible cells)
3. No "finished corners" exist (finished corners = corners visible from dest)
4. Bidirectional A* search cannot find any target to reach
5. Pathfinding returns None

**Comparison**:
- C# with messyY=true: dest sees 6140 and 6141 (at least 2+ cells visible)
- Rust with messyY=true: dest sees **nothing** (0 cells visible)

## Expected Waypoints Are Valid

All expected waypoints exist as corners and are on free cells:

```
WP 6141 at (82,73): cell_val=0, is_corner=true ✓
WP 6307 at (82,75): cell_val=0, is_corner=true ✓
WP 6306 at (81,75): cell_val=0, is_corner=true ✓
WP 6389 at (81,76): cell_val=0, is_corner=true ✓
```

The path is geometrically valid. The problem is purely in the raycasting logic.

## Detailed Bug Location

**File**: `/home/jan/hobby/RustGame3/src/raycast.rs`
**Function**: `raycast(grid, x, y, messy_x, messy_y)`

**Bug**: When `messy_y=true`, the raycasting logic is incorrectly handling:
1. Starting position calculation (y-coordinate adjustment for messy position)
2. Ray angle calculations from non-integer y-positions
3. Cell inclusion checks for fractional vertical positions

**Evidence**: Start position with messyY=true works (4 visible cells), but dest position with messyY=true fails (0 visible cells). This suggests the bug may be position-dependent or related to how the grid boundaries are handled.

## Reproduction Steps

### C# (works)
```csharp
var grid = Grid.init(104, 83, blocked);
var pf = new PathFinder(grid, 1);
var result = pf.pfx2(6388, 6139, 1, messyX: false, messyY: true);
// Returns: waypoints [6141, 6307, 6306, 6389], dist=7.41
```

### Rust (fails)
```rust
let grid = Grid::with_blocked(104, 83, &blocked);
let result = find_path_by_id(&grid, 6388, 6139, false, true);
// Returns: None
```

### Debug Output
```bash
cd /home/jan/hobby/RustGame3
cargo run --example debug_messy_001 --release
```

## Impact on Other Tests

**All 15 tests that return None likely have the same root cause**:
- Raycasting with messyY=true or messyX=true is broken
- This prevents corner discovery from either start or dest
- Without corners, pathfinding cannot proceed

**Tests that pass (14 tests)**: Likely have configurations where:
- Both start and dest happen to produce visible cells despite the bug
- Or the bug only affects certain grid positions/configurations

## Next Steps

1. **Fix raycasting for messyY=true**:
   - Review C# raycast implementation for messy position handling
   - Port the correct logic to Rust raycast.rs
   - Focus on how fractional y-positions affect ray calculations

2. **Test raycast directly**:
   - Create unit test: `raycast(grid, 80, 73, false, true)` should return non-empty set
   - Compare with C#: `pf.raycast(...)` or equivalent visibility calculation

3. **Verify fix**:
   - Re-run debug_messy_001.rs - should show visible cells > 0
   - Re-run test suite - expect more tests to pass

## Related Files

- `/home/jan/hobby/RustGame3/src/raycast.rs` - Bug location
- `/home/jan/hobby/RustGame3/examples/debug_messy_001.rs` - Reproduction script
- `/home/jan/hobby/GameLib2/GameLib/entry/Program.cs` - C# verification (lines 9-57)
- `/home/jan/hobby/GameLib2/GameLib/tests/ProtoTests.cs` - Original test (line 235)
