# Sub-Cell Movement Implementation

## Overview

This implementation provides a fine-grained movement system where each grid cell is subdivided into a 3×3 grid of sub-cells. Actors navigate by reserving and moving between sub-cells, ensuring that only one actor occupies each sub-cell at a time.

## Architecture

### 1. Sub-Cell Coordinate System (`src/subcell.rs`)

```rust
pub struct SubCellCoord {
    cell_x: i32,      // Grid cell X
    cell_y: i32,      // Grid cell Y
    sub_x: i32,       // Sub-cell index (0-2)
    sub_y: i32,       // Sub-cell index (0-2)
}
```

Each grid cell contains 9 sub-cells arranged in a 3×3 pattern:
```
┌───┬───┬───┐
│0,0│1,0│2,0│
├───┼───┼───┤
│0,1│1,1│2,1│
├───┼───┼───┤
│0,2│1,2│2,2│
└───┴───┴───┘
```

**Key Methods:**
- `from_screen_pos()`: Convert pixel coordinates to sub-cell
- `to_screen_center()`: Get pixel coordinates of sub-cell center
- `get_neighbors()`: Return all 8 adjacent sub-cells
- `alignment_score()`: Calculate how well a neighbor aligns with target direction

### 2. Reservation System

```rust
pub struct SubCellReservationManager {
    reservations: HashMap<SubCellCoord, usize>  // Maps sub-cell to actor ID
}
```

**Purpose:** Prevents multiple actors from occupying the same sub-cell.

**Operations:**
- `try_reserve(subcell, actor_id)`: Attempt to reserve a sub-cell
- `release(subcell, actor_id)`: Release a reservation
- `is_reserved(subcell)`: Check reservation status

### 3. Movement Algorithm

Implemented in `Actor::update_subcell()`, following the specification exactly:

```
IF distance_to_reserved_cell <= distance_to_current_cell {
    release(current_cell)
    current_cell = reserved_cell
    go_to_current_cell_center()
}
```

**State Machine:**

1. **Moving to Reserved Sub-Cell**
   - Actor has both `current_subcell` and `reserved_subcell`
   - Moves toward reserved sub-cell
   - When closer to reserved than current: switch!

2. **Centering on Current Sub-Cell**
   - Actor has `current_subcell` only
   - Moves toward center of current sub-cell
   - When centered: try to reserve next sub-cell

3. **Reserving Next Sub-Cell**
   - Calculate direction to destination
   - Get 5 best candidates (best, ±1 clockwise, ±2 clockwise)
   - Try to reserve first available candidate
   - If all blocked: wait

4. **Destination Reached**
   - When very close to destination (< 2 pixels)
   - Release all reservations
   - Clear sub-cell state

### 4. Candidate Selection

The `find_best_neighbors()` function returns 5 candidates in priority order:

```
Given current direction → target:

1. Best aligned neighbor (highest dot product with direction)
2. 1× clockwise from best
3. 1× counter-clockwise from best
4. 2× clockwise from best
5. 2× counter-clockwise from best
```

This provides intelligent fallback when the preferred sub-cell is occupied.

## GUI Integration

### Controls

| Key | Action |
|-----|--------|
| **S** | Toggle sub-cell movement ON/OFF |
| **G** | Cycle sub-cell display: None → 2×2 → 3×3 |
| **O** | Spawn actor at mouse position |
| **P** | Set destination for all actors |

### Visual Indicators

When sub-cell movement is enabled:

- **Green circle** = Current sub-cell center
- **Yellow circle** = Reserved sub-cell center
- **Yellow line** = Movement vector to reserved
- **Orange circle** = Final destination
- **White grid lines** = 3×3 sub-cell boundaries

### Status Display

```
SubCell: 3x3 | SubCell Movement: ON (reservations: 5)
```

Shows:
- Current display grid mode
- Movement mode status
- Number of active reservations

## Implementation Details

### Actor State Fields

```rust
pub struct Actor {
    // ... existing fields ...

    // Sub-cell movement state
    pub current_subcell: Option<SubCellCoord>,
    pub reserved_subcell: Option<SubCellCoord>,
    pub subcell_destination: Option<Position>,
}
```

### Mode Switching

**Enabling sub-cell movement:**
```rust
for actor in &mut actors {
    if let Some(dest) = actor.destination {
        actor.set_subcell_destination(dest);
        actor.clear_path();  // Clear pathfinding path
    }
}
```

**Disabling sub-cell movement:**
```rust
reservation_manager.clear();  // Release all reservations
for actor in &mut actors {
    actor.subcell_destination = None;
    actor.reserved_subcell = None;
}
```

### Update Loop

```rust
if subcell_movement_enabled {
    for actor in &mut actors {
        actor.update_subcell(delta_time, &mut reservation_manager);
    }
} else {
    // Normal pathfinding with NPV...
}
```

## Design Decisions

### 1. Conservative Distance Check

```rust
if dist_to_reserved <= dist_to_current {
    // Switch to reserved
}
```

Using `<=` ensures we switch as soon as we're equidistant, preventing oscillation.

### 2. Centering Threshold

```rust
if dist_to_center > 1.0 {
    // Keep moving to center
}
```

Actors must be within 1 pixel of center before reserving next sub-cell.

### 3. Destination Threshold

```rust
if dist_to_dest < 2.0 {
    // Reached destination
}
```

2-pixel tolerance prevents infinite micro-adjustments at destination.

### 4. No Collision Detection

As specified, the prototype **ignores collisions and blocked cells**. This focuses on validating the sub-cell reservation mechanism without complicating it with obstacle avoidance.

## Testing

### Unit Tests (`src/subcell.rs`)

- ✅ `test_subcell_from_screen_pos()`: Coordinate conversion
- ✅ `test_subcell_to_screen_center()`: Reverse conversion
- ✅ `test_reservation_manager()`: Reservation logic

### Manual Testing (`test_subcell.md`)

1. **Single Actor**: Smooth sub-cell navigation to destination
2. **Multiple Actors**: First-come-first-served reservation
3. **Mode Switching**: Clean transitions between modes
4. **Obstacle Handling**: Validates that collisions are ignored (as intended)

## Performance Characteristics

- **Space:** O(A) where A = number of active reservations (typically 2A for A actors)
- **Time per frame:** O(A × 8) for neighbor checking
- **Worst case:** O(A × 5) reservation attempts per actor when heavily congested

## Future Enhancements

Potential improvements (not in current scope):

1. **Collision Detection**: Respect blocked cells
2. **Sub-Cell Pathfinding**: A* at sub-cell granularity
3. **Priority System**: Important actors get reservation priority
4. **Waiting Strategy**: Smarter wait-and-retry logic
5. **Variable Grid Size**: Support 2×2, 4×4, etc.
6. **Reservation Timeout**: Prevent deadlocks from abandoned reservations

## Comparison with Normal Movement

| Feature | Normal Pathfinding | Sub-Cell Movement |
|---------|-------------------|-------------------|
| **Granularity** | Cell centers | 9 points per cell |
| **Collision** | Radius-based | Reservation-based |
| **Path** | A* with corners | Greedy best-first |
| **Blocked cells** | Respected | Ignored (prototype) |
| **Multiple actors** | Radius avoidance | Sub-cell reservation |

## Example Scenario

**Setup:** 2 actors, 3×3 sub-cells, same destination

```
Frame 1: Actor 1 at (10, 10), Actor 2 at (12, 10), Dest at (11, 10)

Actor 1:
- current_subcell: (10, 10, 1, 1) [center of cell 10,10]
- reserved_subcell: (10, 10, 2, 1) [right sub-cell]
- Moving right toward reserved

Actor 2:
- current_subcell: (12, 10, 1, 1) [center of cell 12,10]
- reserved_subcell: (12, 10, 0, 1) [left sub-cell]
- Moving left toward reserved

Frame 10: Actors have moved closer

Actor 1:
- current_subcell: (10, 10, 2, 1) [switched!]
- reserved_subcell: (11, 10, 0, 1) [reserved next cell]
- Moving right

Actor 2:
- current_subcell: (12, 10, 0, 1) [switched!]
- reserved_subcell: (11, 10, 2, 1) [reserved next cell]
- Moving left

Frame 20: Both in destination cell, different sub-cells

Actor 1:
- current_subcell: (11, 10, 0, 1) [left side]
- reserved_subcell: (11, 10, 1, 1) [center - trying!]
- Blocked! Waiting...

Actor 2:
- current_subcell: (11, 10, 2, 1) [right side]
- reserved_subcell: (11, 10, 1, 1) [center - reserved!]
- Moving to center

Frame 30: Actor 2 reaches destination

Actor 2: REACHED! (releases reservations)

Frame 31: Actor 1 can now reserve center

Actor 1:
- reserved_subcell: (11, 10, 1, 1) [reserved!]
- Moving to center

Frame 40: Actor 1 reaches destination

Actor 1: REACHED!
```

## Conclusion

The sub-cell movement system provides fine-grained actor control through a simple reservation mechanism. The implementation closely follows the specification, using a state machine approach with distance-based switching and intelligent fallback candidate selection.

The system successfully handles multiple actors converging on the same destination, demonstrating the effectiveness of the first-come-first-served reservation strategy.
