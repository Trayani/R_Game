# Sub-Cell Movement Implementation

## Overview

This implementation provides a fine-grained movement system where each grid cell is subdivided into a configurable grid of sub-cells (2×2, 3×3, or 1×1). Actors navigate by reserving and moving between sub-cells, ensuring collision-free movement through a reservation-based system.

**Current Baseline:** 2×2 sub-cell grid is considered the standard configuration.

## Architecture

### 1. Sub-Cell Coordinate System (`src/subcell.rs`)

```rust
pub struct SubCellCoord {
    cell_x: i32,      // Grid cell X
    cell_y: i32,      // Grid cell Y
    sub_x: i32,       // Sub-cell index (0-1 for 2×2, 0-2 for 3×3)
    sub_y: i32,       // Sub-cell index (0-1 for 2×2, 0-2 for 3×3)
    grid_size: i32,   // Grid size (2 or 3)
}
```

Each grid cell contains multiple sub-cells. For 2×2 (baseline):
```
┌─────┬─────┐
│ 0,0 │ 1,0 │
├─────┼─────┤
│ 0,1 │ 1,1 │
└─────┴─────┘
```

**Key Methods:**
- `from_screen_pos()`: Convert pixel coordinates to sub-cell (uses default no-offset)
- `from_screen_pos_with_offset()`: Convert with configurable X/Y offset
- `to_screen_center()`: Get pixel coordinates of sub-cell center (default no-offset)
- `to_screen_center_with_offset()`: Get center position with X/Y offset
- `get_neighbors()`: Return all 8 adjacent sub-cells
- `alignment_score()`: Calculate how well a neighbor aligns with target direction

### 2. Sub-Cell Offset System

**NEW:** Sub-cell grids can be offset by 0.5 sub-cells in X and/or Y directions.

```rust
enum SubCellOffset {
    None,    // Standard alignment (e.g., 0.25, 0.75 for 2×2)
    X,       // Horizontal offset by 0.5 sub-cells
    Y,       // Vertical offset by 0.5 sub-cells
    XY,      // Both X and Y offsets
}
```

**Example for 2×2 with X offset:**
```
Cell: |-----------|
None: .     x     x     .  (at 0.25, 0.75)
X:    x     .     x     .  (at 0.0, 0.5)
                ^
         center sub-cell
```

With X offset:
- One sub-cell at cell center
- Two "half" sub-cells on left/right edges

**Use Cases:**
- **None**: Standard grid alignment
- **X/Y**: Creates a center sub-cell with edge half-cells
- **XY**: Sub-cells align with cell corners and center

### 3. Reservation System

```rust
pub struct SubCellReservationManager {
    reservations: HashMap<SubCellCoord, usize>,  // Maps sub-cell to actor ID
    grid_size: i32,
}
```

**Purpose:** Prevents multiple actors from occupying the same sub-cell.

**Operations:**
- `try_reserve(subcell, actor_id)`: Attempt to reserve a single sub-cell
- `try_reserve_multiple(subcells, actor_id)`: Atomically reserve multiple sub-cells
- `release(subcell, actor_id)`: Release a reservation
- `is_reserved(subcell)`: Check reservation status
- `clear()`: Release all reservations (used when switching modes)

### 4. Square Reservation Strategy

**NEW:** Actors can attempt to reserve a 2×2 square of sub-cells in their primary movement direction.

**Algorithm:**
1. Determine primary direction (X or Y, whichever has larger magnitude)
2. Find best aligned neighbor sub-cell (primary move target)
3. Identify 3 additional sub-cells that form a 2×2 square:
   - Best neighbor (primary target)
   - Perpendicular from current
   - Perpendicular from best
   - Diagonal corner
4. Try to reserve all 4 atomically
5. If successful: move to best, track 3 extras in `extra_reserved_subcells`
6. If failed: fall back to single-cell reservation

**Benefits:**
- Actors get more "space" during movement
- Reduces congestion in crowded areas
- Smoother flow around obstacles

**Lifecycle:**
- During movement: Actor may hold 1-4 sub-cell reservations
- When switching from reserved to current: Release extra 3 cells
- At destination: Release all except destination sub-cell

**Toggle:** Press **Q** to enable/disable square reservation (default: ON)

### 5. Movement Algorithm

Implemented in `Actor::update_subcell()`:

**State Machine:**

1. **Destination Reached Check**
   - If distance to destination < 2.0 pixels:
     - Calculate destination sub-cell
     - Release all cells except destination sub-cell
     - Keep destination as `current_subcell`
     - Clear `reserved_subcell` and `extra_reserved_subcells`
     - Return `true` (reached)

2. **Moving to Reserved Sub-Cell**
   - Actor has both `current_subcell` and `reserved_subcell`
   - Moves toward reserved sub-cell center (with offset)
   - When closer to reserved than current:
     - Switch: `current_subcell = reserved_subcell`
     - Release current (if different from reserved)
     - **Release all extra reserved cells** (from square reservation)
     - Clear `reserved_subcell` and `extra_reserved_subcells`
     - Continue to centering phase

3. **Centering on Current Sub-Cell**
   - Actor has `current_subcell` only
   - Moves toward center of current sub-cell (with offset)
   - When within 1.0 pixel of center: proceed to reservation phase

4. **Reserving Next Sub-Cell**
   - Calculate direction to destination
   - **If square reservation enabled:**
     - Call `find_square_reservation()` to identify 2×2 square
     - Try `try_reserve_multiple()` for all 4 cells
     - If successful: set `reserved_subcell` and `extra_reserved_subcells`
   - **Fallback to single-cell:**
     - Get 5 best candidate neighbors (best, ±1 CW, ±2 CW)
     - Try to reserve first available candidate
     - If successful: set `reserved_subcell`, clear `extra_reserved_subcells`
   - **If all blocked:** Wait (no reservation made)

### 6. Candidate Selection

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

### 7. Square Formation

The `find_square_reservation()` function:

```rust
pub fn find_square_reservation(
    current: &SubCellCoord,
    target_dir_x: f32,
    target_dir_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> Option<(SubCellCoord, [SubCellCoord; 3])>
```

Returns `Some((best, [3 additional cells]))` if square can be formed.

**Logic:**
1. Determine primary direction (abs(dir_x) vs abs(dir_y))
2. Find best aligned neighbor
3. Find neighbors of best that are also neighbors of current
4. Select perpendicular neighbor (best alignment to perpendicular direction)
5. Find fourth corner (adjacent to both best and perpendicular neighbor)
6. Return `(best, [perp_from_current, fourth_corner, best])`

## GUI Integration

### Controls

| Key | Action |
|-----|--------|
| **S** | Toggle sub-cell movement ON/OFF |
| **G** | Cycle sub-cell display: None → 1×1 → 2×2 → 3×3 |
| **T** | Cycle sub-cell offset: None → X → Y → XY |
| **Q** | Toggle square reservation (2×2) ON/OFF |
| **B** | Toggle sub-cell markers (visual indicators) |
| **O** | Spawn actor at mouse position |
| **D** | Set destination for selected actor |
| **P** | Set destination for all actors |

### Visual Indicators

When sub-cell movement is enabled and markers are shown (**B** key):

- **Green circle** = Current sub-cell center
- **Yellow circle + line** = Reserved sub-cell center (primary target)
- **Black circles + lines** = Extra reserved sub-cells (from square reservation)
- **Orange circle** = Final destination (cell-level)
- **Grid lines** = Sub-cell boundaries (when display mode is not None)

### Status Display

```
SubCell: 2x2 | SubCell Movement: ON (reservations: 12)
```

Shows:
- Current display grid mode (None/1×1/2×2/3×3)
- Movement mode status
- Number of active reservations

Console output on toggle:
```
Sub-cell markers: ON
Square reservation (2×2): ON
SubCell Offset: X
```

## Implementation Details

### Actor State Fields

```rust
pub struct Actor {
    // ... existing fields ...

    // Sub-cell movement state
    pub subcell_grid_size: i32,
    pub subcell_offset_x: f32,
    pub subcell_offset_y: f32,
    pub current_subcell: Option<SubCellCoord>,
    pub reserved_subcell: Option<SubCellCoord>,
    pub extra_reserved_subcells: Vec<SubCellCoord>,  // For square reservation
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
    actor.extra_reserved_subcells.clear();
}
```

### Update Loop

```rust
if subcell_movement_enabled {
    for actor in &mut actors {
        actor.update_subcell(
            delta_time,
            &mut reservation_manager,
            square_reservation_enabled,  // Toggle for 2×2 square strategy
        );
    }
} else {
    // Normal pathfinding with NPV...
}
```

### Offset Management

When offset changes (T key pressed):
```rust
state.subcell_offset = state.subcell_offset.next();
let (offset_x, offset_y) = state.subcell_offset.get_offsets();

// Update all actors
for actor in &mut state.actors {
    actor.subcell_offset_x = offset_x;
    actor.subcell_offset_y = offset_y;

    // Recalculate current sub-cell with new offset
    if actor.current_subcell.is_some() {
        actor.current_subcell = Some(SubCellCoord::from_screen_pos_with_offset(
            actor.fpos_x,
            actor.fpos_y,
            actor.cell_width,
            actor.cell_height,
            actor.subcell_grid_size,
            offset_x,
            offset_y,
        ));
    }
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

### 4. Atomic Square Reservation

Square reservation is atomic: either all 4 cells are reserved, or none are. This prevents partial reservations that could cause deadlocks.

### 5. Single Sub-Cell at Destination

When an actor reaches its destination, it releases all reservations except the destination sub-cell. This frees up space for other actors while maintaining the actor's final position.

### 6. No Collision Detection with Blocked Cells

As specified, the prototype **ignores blocked cells**. Actors will navigate through walls based purely on sub-cell reservations. This focuses on validating the reservation mechanism without complicating it with obstacle avoidance.

## Configuration Baseline

**Recommended Settings (2×2 Baseline):**
- Grid size: **2×2** (press G to cycle to 2×2)
- Offset: **None** initially, experiment with **X**, **Y**, **XY**
- Square reservation: **ON** (default)
- Markers: **ON** for debugging, **OFF** for cleaner visuals

**Why 2×2?**
- Simpler than 3×3 (fewer states)
- More granular than 1×1
- Square reservation works naturally (2×2 sub-cells = 4 cells)
- Good balance between precision and performance

## Testing

### Unit Tests (`src/actor.rs`, `src/subcell.rs`)

- ✅ `test_subcell_from_screen_pos()`: Coordinate conversion
- ✅ `test_subcell_to_screen_center()`: Reverse conversion
- ✅ `test_reservation_manager()`: Reservation logic
- ✅ `test_actor_clean_position()`: Clean cell positioning
- ✅ `test_actor_messy_x/y/xy()`: Messy position handling
- ✅ `test_actor_path_following()`: Sub-cell destination logic

### Manual Testing

1. **Single Actor**: Smooth sub-cell navigation to destination
2. **Multiple Actors**: First-come-first-served reservation
3. **Square Reservation**: Visual confirmation of 4-cell blocks
4. **Offset Modes**: Different grid alignments produce different behavior
5. **Mode Switching**: Clean transitions between modes

## Performance Characteristics

- **Space:** O(R) where R = number of active reservations
  - With square reservation: up to 4R for R actors
  - Typical: 1-2 reservations per actor
- **Time per frame:**
  - Single-cell: O(A × 8) neighbor checking
  - Square: O(A × 16) (neighbors + square formation)
- **Worst case:** O(A × 5) reservation attempts per actor when heavily congested

## Comparison with Normal Movement

| Feature | Normal Pathfinding | Sub-Cell Movement (2×2) |
|---------|-------------------|-------------------|
| **Granularity** | Cell centers | 4 points per cell |
| **Collision** | Radius-based | Reservation-based |
| **Path** | A* with corners | Greedy best-first |
| **Blocked cells** | Respected | Ignored (prototype) |
| **Multiple actors** | Radius avoidance | Sub-cell reservation |
| **Space per actor** | 1 position | 1-4 sub-cell reservations |
| **Congestion handling** | Push/slide | Wait-and-retry |

## Example Scenario: Square Reservation

**Setup:** 2 actors, 2×2 sub-cells, square reservation ON, same destination

```
Frame 1: Actor 1 at cell (10, 10), Actor 2 at cell (12, 10), Dest at cell (11, 10)

Actor 1:
- current_subcell: (10, 10, 1, 1) [bottom-right of cell 10,10]
- reserved_subcell: None
- Moving toward cell center, trying to reserve next...

Actor 1 reserves square:
- reserved_subcell: (11, 10, 0, 0) [top-left of cell 11,10]
- extra_reserved_subcells: [(10, 10, 0, 0), (11, 10, 1, 0), (10, 10, 1, 0)]
  (forms 2×2 block in direction of movement)

Actor 2:
- current_subcell: (12, 10, 0, 0) [top-left of cell 12,10]
- Tries to reserve square toward (11, 10)
- BLOCKED! (Actor 1 already has those cells)
- Falls back to single-cell reservation
- reserved_subcell: (12, 10, 1, 0) [top-right]
- Moving right

Frame 10: Actor 1 crosses into reserved cell

Actor 1:
- Switches: current_subcell = (11, 10, 0, 0)
- Releases: old current + 3 extra cells
- reserved_subcell: None
- Centering on new current...

Actor 2:
- Can now try to reserve square again
- reserved_subcell: (11, 10, 1, 1) [bottom-right]
- extra_reserved_subcells: [3 cells forming square]
- Moving left

Frame 20: Actor 1 reaches destination sub-cell

Actor 1:
- Distance to dest < 2.0 pixels
- Releases all except destination sub-cell
- current_subcell: (11, 10, 1, 1) [cell center - destination]
- REACHED!

Actor 2:
- Continues moving toward reserved square
- Will eventually reach destination after Actor 1
```

## Future Enhancements

Potential improvements (not in current scope):

1. **Collision Detection**: Respect blocked cells, integrate with raycasting
2. **Sub-Cell Pathfinding**: A* at sub-cell granularity for complex navigation
3. **Dynamic Grid Size**: Runtime changes to grid size with actor adaptation
4. **Reservation Timeout**: Prevent deadlocks from abandoned reservations
5. **Priority System**: Important actors get reservation priority
6. **Adaptive Square Size**: Use 3×3 or 4×4 squares based on congestion
7. **Offset Transitions**: Smooth animation when changing offset modes

## Known Limitations

1. **No Obstacle Avoidance**: Actors navigate through walls
2. **Greedy Algorithm**: Local decisions may not be globally optimal
3. **Potential Deadlocks**: Head-on collisions can cause waiting loops
4. **Grid Boundary Issues**: Actors near edges may have limited reservation options
5. **Mode Changes**: Changing offset while actors are moving causes position recalculation

## Conclusion

The sub-cell movement system provides fine-grained actor control through a reservation-based mechanism. Key features:

- **Flexible Grid**: 2×2 baseline, supports 1×1 and 3×3
- **Offset Modes**: Four alignment options for different behaviors
- **Square Reservation**: Actors reserve 2×2 blocks for smoother movement
- **Atomic Operations**: All-or-nothing reservations prevent deadlocks
- **Single-Cell at Rest**: Actors free up space when reaching destinations

The implementation successfully handles multiple actors converging on destinations through first-come-first-served reservation with intelligent fallback strategies.
