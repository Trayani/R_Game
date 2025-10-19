# Messy Positions: How They Work

## Core Concept

**Messy positions ONLY affect the starting/observer position, NEVER the destination.**

Once a path reaches any corner from the starting position, all subsequent navigation is "cleanly aligned" to the grid.

## What Are Messy Positions?

Messy positions represent entities that are **not aligned to the grid**.

### messyX (Horizontal Misalignment)
- Entity's left edge is between grid columns (e.g., X=5.5 instead of X=5 or X=6)
- Entity spans from X to X+1 horizontally
- Example: A size=1 entity at messyX=true occupies partial cells at both X and X+1

### messyY (Vertical Misalignment)
- Entity's top edge is between grid rows (e.g., Y=10.5 instead of Y=10 or Y=11)
- Entity spans from Y to Y+1 vertically
- Example: A size=1 entity at messyY=true occupies partial cells at both Y and Y+1

## Where Messy Flags Apply

### ✅ MUST Use Messy Flags

1. **Raycasting from START position**
   ```rust
   let visible_cells = raycast(grid, start_x, start_y, messy_x, messy_y);
   ```
   - The observer (starting entity) has a fractional position
   - Rays must originate from the correct fractional coordinates
   - Vision cone is affected by misalignment

2. **Corner filtering from START position**
   ```rust
   let interesting_corners = filter_interesting_corners(
       &all_corners, &visible_cells, grid, start_x, start_y, messy_x
   );
   ```
   - Which corners are "interesting" depends on observer's fractional position
   - A corner at the observer's own position needs special handling
   - Adjacent cell visibility is affected by misalignment

### ❌ NEVER Use Messy Flags

1. **Raycasting from DESTINATION**
   ```rust
   // CORRECT: Always use false, false for destination
   let dest_visible = raycast(grid, dest_x, dest_y, false, false);
   ```
   - Destination is just a target point, not an entity position
   - We only care if corners can "see" the destination as a grid-aligned point

2. **Corner filtering from DESTINATION**
   ```rust
   // CORRECT: Always use false for destination corner filtering
   let dest_corners = filter_interesting_corners(
       &all_corners, &dest_visible, grid, dest_x, dest_y, false
   );
   ```

## Why Messy Flags Affect Corner Discovery

### Scenario: messyY=true (entity between rows Y and Y+1)

The entity simultaneously occupies two rows vertically:
- Partial occupation of row Y
- Partial occupation of row Y+1

**Impact on corners**:
- A corner at the entity's exact position (Y or Y+1) might be "occupied" by the entity itself
- The entity has different visibility from Y vs Y+1
- Adjacent cell checks must account for spanning two rows

### Example: Test messy_001

```
Start: 6388 at (80, 76) with messyY=true
- Entity spans rows 76 and 77 vertically
- Raycasting must originate from fractional Y position (76.5)
- Corner 6389 at (81, 76) is discovered as interesting

Dest: 6139 at (80, 73) - NO messy flags
- Destination is just a target point
- Raycasting from dest uses clean grid-aligned position (80, 73)
- Discovers corners 6140 and 6141 that can see the destination
```

## Current Bug in Rust Implementation

### Bug Location: `src/corners.rs:138`

```rust
pub fn filter_interesting_corners_with_observer_corners(
    all_corners: &[Corner],
    visible_cells: &HashSet<i32>,
    grid: &Grid,
    observer_x: i32,
    observer_y: i32,
    _messy_x: bool,  // ⚠️ UNDERSCORE PREFIX - NOT USED!
    observer_corners: &[(i32, i32)],
) -> Vec<Corner> {
```

**Problem**: The `_messy_x` parameter has an underscore, meaning it's **intentionally ignored**.

**Impact**: Messy position filtering logic is NOT implemented. The function currently:
1. Checks if corner is visible ✓
2. Checks if corner leads to hidden areas ✓
3. **Does NOT account for messy position edge cases** ✗

### What Should messyX Do?

When `messy_x=true`, the observer spans columns X and X+1:

1. **Corners at observer's position**:
   - Corner at (X, observer_y) might be occupied by observer's left half
   - Corner at (X+1, observer_y) might be occupied by observer's right half
   - Need special handling to determine if these are "interesting"

2. **Adjacent cell visibility**:
   - Cell at (X-1, Y) might be partially blocked by observer's left edge
   - Cell at (X+2, Y) might be partially blocked by observer's right edge
   - Affects which corners lead to "hidden" areas

### What Should messyY Do?

When `messy_y=true`, the observer spans rows Y and Y+1:

1. **Corners at observer's position**:
   - Corner at (observer_x, Y) might be occupied by observer's top half
   - Corner at (observer_x, Y+1) might be occupied by observer's bottom half
   - Need special handling to determine if these are "interesting"

2. **Adjacent cell visibility**:
   - Cell at (X, Y-1) might be partially blocked by observer's top edge
   - Cell at (X, Y+2) might be partially blocked by observer's bottom edge
   - Affects which corners lead to "hidden" areas

## C# Reference

In the C# codebase, messy positions affect:

1. **Line segment discovery** (`PathFinder.cs:calcRow`):
   - messyX/messyY determine which line segments the entity occupies
   - Multiple line segments might be needed for spanning entities

2. **Corner computation** (`PathFinder.cs:computeConer`):
   - messyX/messyY affect which corners are discovered from the starting position
   - Used in `getBorders()` to handle fractional positions

3. **Ray-casting** (`PFContext.cs`):
   - Rays originate from fractional coordinates when messy flags are true
   - Integer ray calculations must account for fractional starting positions

## Path Navigation Flow

```
START (with messyX/Y)
  |
  | [Raycast with messy flags]
  | [Find interesting corners with messy flags]
  v
First Corner (grid-aligned)
  |
  | [Normal corner-to-corner navigation]
  | [NO messy flags needed anymore]
  v
Second Corner (grid-aligned)
  |
  | [Normal corner-to-corner navigation]
  v
...
  |
  v
Last Corner (grid-aligned)
  |
  | [Normal movement to destination]
  | [NO messy flags - dest is just a target point]
  v
DESTINATION (never messy)
```

## Test Case Analysis

### test2x_messy_001: 6388 -> 6139 (messyY=true)

**Start: 6388 at (80, 76) with messyY=true**
- Raycast from (80, 76.5) - fractional Y position
- Should discover corners considering vertical misalignment
- C# finds 1 corner: 6389 at (81, 76)

**Destination: 6139 at (80, 73) - NO messy flags**
- Raycast from (80, 73) - clean grid position
- Should discover corners normally
- C# finds 2 corners: 6140 at (81, 73), 6141 at (82, 73)

**Current Rust Bug**:
- Start raycast works (finds corners)
- Dest raycast returns 0 cells - **THIS IS THE BUG**
- Without dest corners, no path can be found

## Why Dest Raycast Returns 0 Cells

**HYPOTHESIS**: The bug is NOT in messy flag handling (dest correctly uses false, false).

The bug is likely in the **raycast implementation itself** for this specific grid position (80, 73):
- Possibly a boundary condition bug
- Possibly an edge case in ray angle calculations
- Possibly related to blocked cells near (80, 73)

**Next step**: Debug raycast(grid, 80, 73, false, false) directly to see why it returns empty.

## Summary

- ✅ Messy flags ONLY apply to START position (observer)
- ✅ Messy flags affect raycasting and corner filtering from start
- ❌ Messy flags NEVER apply to DESTINATION
- ❌ Destination is always treated as grid-aligned target point
- 🐛 Current bug: Messy flag parameter in corner filtering is ignored (`_messy_x`)
- 🐛 Current bug: Raycast returns 0 cells for position (80, 73) - needs investigation
