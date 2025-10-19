# Messy Position Alignment Principle

## Core Concept

**When starting from a messy position, the entity must first ALIGN to a clean grid position before traveling.**

## Why Alignment is Necessary

### Messy Position Constraints

A messy position means the entity is **not aligned to the grid**:
- **messyX=true**: Entity spans columns X and X+1 (between grid columns)
- **messyY=true**: Entity spans rows Y and Y+1 (between grid rows)

### Visibility Difference

**Critical insight**: A corner that is visible from a clean position may NOT be visible from the messy position!

**Example**: Position 2679 at (23, 32) with messyY=true
- Entity spans rows 32 and 33 (between them)
- From messy (23, 32.5): Limited visibility due to fractional position
- From clean (23, 32): Full visibility from grid-aligned position

### The Alignment Step

Before the entity can travel to any destination, it must:
1. **Align to start position** - Move from messy to clean at same coordinates
2. From aligned position, it can then navigate normally

**Path structure with messy start**:
```
MESSY_START → ALIGN_TO_START → ... corners ... → DESTINATION
   2679          2679              waypoints        2684
 (23, 32.5)    (23, 32)
```

This is why start position 2679 appears as a waypoint - it's the **alignment target**!

## Same-Line Special Case

### Scenario: Start and Dest on Same Row

**Test messy_024**: 2679 → 2684
- Start: 2679 at (23, 32) with messyY=true (entity at Y=32.5)
- Dest: 2684 at (28, 32) - same row!

### C# Logic (PathFinder.cs:345-372)

```csharp
if (onLine) {  // Start and dest on same horizontal line
    if (!messyStartY)
        return empty waypoints;  // Already aligned - direct path

    // messyStartY=true: Must align before traveling
    var lx = getLineFromRow(start + grid.cols, size);  // Get line from row BELOW
    var res = new List<int>();

    // Determine alignment waypoint
    if (start > dest) {
        if (dest.X < lx2.startX)
            res.Add(grid.getId(lx.startX, l.y));  // Align to line start
    }
    else if (dest.X > lx.endX) {
        res.Add(grid.getId(lx.endX, l.y));  // Align to line end
    }

    return res;  // Alignment waypoint(s)
}
```

### Why Check Row Below?

When messyY=true, the entity spans rows Y and Y+1:
- Current row: Y=32
- Row below: Y+1=33

The entity needs to check if there's a valid line segment on row 33 that allows alignment.

**For test messy_024**:
1. Start at messy position 2679 (23, 32.5)
2. Check line on row 33 from position 2679+83=2762
3. Line boundaries determine alignment point
4. If dest (28, 32) is beyond line.endX, align to line.endX
5. Alignment waypoint happens to be at position 2679 (same ID, but clean position)
6. Path: [2679_messy → 2679_clean → 2684]

**Waypoints returned**: `[2679]`
**Full path**: `[2679, 2679, 2684]` - First 2679 is messy start, second is clean alignment!

## General Case: Messy Position Pathfinding

### Non-Same-Line Paths

For paths where start and dest are NOT on same line:

1. **Compute corners from messy start position**
   - Use messyX, messyY flags for raycasting
   - Get interesting corners from messy viewpoint

2. **Navigate through corners**
   - Once at first corner, entity is grid-aligned
   - All subsequent navigation is clean (no messy flags)

3. **Path structure**:
   ```
   MESSY_START → first_corner → ... → last_corner → DESTINATION
   ```

### Same-Line Paths

For paths where start and dest ARE on same line:

**Without messy flags**:
```
START → DESTINATION (direct path, no waypoints)
```

**With messy flags**:
```
MESSY_START → ALIGNMENT_WAYPOINT → DESTINATION
```

The alignment waypoint is determined by checking the line from the adjacent row.

## Implementation in Rust

### Required Changes

Rust needs to implement the same-line special case:

```rust
fn find_path(...) {
    // ... existing code ...

    // Check if start and dest are on same line
    if start_y == dest_y {
        return handle_same_line_path(start, dest, messy_x, messy_y, grid);
    }

    // ... rest of pathfinding ...
}

fn handle_same_line_path(start, dest, messy_x, messy_y, grid) -> Option<Vec<Position>> {
    // If no messy flags, return direct path
    if !messy_x && !messy_y {
        return Some(vec![start, dest]);
    }

    // With messy flags, need alignment waypoint
    let mut waypoints = Vec::new();

    if messy_y {
        // Check line from row below
        let below_row = start.y + 1;
        if let Some(line_below) = get_line_at(grid, start.x, below_row) {
            // Determine if alignment waypoint needed
            if start.x > dest.x {
                if dest.x < line_below.start_x {
                    waypoints.push(Position::new(line_below.start_x, start.y));
                }
            } else if dest.x > line_below.end_x {
                waypoints.push(Position::new(line_below.end_x, start.y));
            }
        }
    }

    if messy_x {
        // Similar logic for horizontal alignment
        // Check line from column to the right
        // ...
    }

    // Build full path: start + waypoints + dest
    let mut path = vec![start];
    path.extend(waypoints);
    path.push(dest);

    Some(path)
}
```

## Key Principles

1. **Messy → Clean Transition**: Messy positions must align to clean before traveling
2. **Visibility Difference**: Corners visible from clean may not be visible from messy
3. **Alignment Waypoint**: Same ID but different semantic position (messy vs clean)
4. **Same-Line Special Case**: Requires checking adjacent row/column line segments
5. **Once Aligned**: All subsequent navigation is clean (no messy flags needed)

## Test messy_024 Explanation

**C# Expected**: `[2679, 2679, 2684]`
- First 2679: Messy start at (23, 32.5)
- Second 2679: Clean alignment at (23, 32)
- 2684: Destination at (28, 32)

**Rust Current**: `[2679, 2684]`
- Missing the alignment waypoint!
- Treats it as direct path from messy position
- **INCORRECT** - must add alignment waypoint

**Fix**: Implement same-line special case with messy flag handling in Rust.
