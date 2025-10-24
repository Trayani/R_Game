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

### 4. Reservation Strategies

The system supports two mutually exclusive reservation strategies, toggled with the **Q** key:

#### 4a. Square Reservation Strategy (Default)

Actors attempt to reserve a 2×2 square of sub-cells in their primary movement direction.

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

#### 4b. Diagonal Reservation Strategy

When enabled, diagonal moves require also reserving a horizontal or vertical "anchor" cell.

**Algorithm:**
1. For each candidate neighbor, check if it's a diagonal move (both X and Y components change)
2. If diagonal:
   - Find an anchor cell (horizontal or vertical neighbor of current)
   - Try to atomically reserve BOTH anchor and diagonal cells
   - If successful: move to diagonal, track anchor in `extra_reserved_subcells`
   - If failed: skip this candidate, try next
3. If non-diagonal: reserve normally (single cell)

**Benefits:**
- More conservative movement through tight spaces
- Prevents actors from "cutting corners" too aggressively
- Actors maintain clearer horizontal/vertical paths

**Example:**
```
Current: (0, 0)
Best diagonal: (1, 1)
Anchor options: (1, 0) [horizontal] or (0, 1) [vertical]
Result: Reserve both (1, 0) and (1, 1), or both (0, 1) and (1, 1)
```

**Toggle:** Press **Q** to cycle between Square, Diagonal, and NoDiagonal modes

#### 4c. NoDiagonal Reservation Strategy

When enabled, diagonal moves are completely prohibited. Only horizontal and vertical moves are allowed.

**Algorithm:**
1. For each candidate neighbor, check if it's a diagonal move
2. If diagonal: **skip** this candidate entirely
3. If horizontal or vertical: attempt reservation normally

**Benefits:**
- Forces more structured, grid-aligned movement
- Eliminates "cutting corners" behavior completely
- More predictable pathfinding (Manhattan distance)
- Useful for scenarios requiring strict cardinal direction movement

**Example:**
```
Current: (0, 0)
Candidates: [(1,0), (1,1), (0,1), (-1,1), ...]
             ^H     ^D     ^V     ^D
             OK     SKIP   OK     SKIP

Result: Only horizontal (1,0) and vertical (0,1) are considered
```

**Toggle:** Press **Q** to cycle through modes: Square → Diagonal → NoDiagonal → Square

#### 4d. Early Reservation Mode

When enabled, actors immediately attempt to reserve the next sub-cell after switching from reserved to current, without waiting to center on the current cell.

**Normal Behavior (Early Reservation OFF):**
1. Actor moves toward reserved sub-cell
2. When closer to reserved than current: switch current = reserved
3. Actor continues moving toward **center** of new current cell
4. When within 1.0 pixel of center: attempt to reserve next cell
5. Move toward newly reserved cell

**Early Reservation Behavior (Early Reservation ON):**
1. Actor moves toward reserved sub-cell
2. When closer to reserved than current: switch current = reserved
3. **Immediately attempt to reserve next cell** (skip centering)
4. If reservation succeeds: move directly toward new reserved cell
5. If reservation fails: continue toward center of current (fallback to normal)

**Benefits:**
- Faster, more fluid movement (no pause at cell centers)
- Reduced overall travel time to destination
- More responsive navigation in open areas

**Trade-offs:**
- May appear less "deliberate" or controlled
- Actors move through cells without settling
- Can make congestion harder to observe visually

**Example:**
```
Normal mode:
  Current (5,5) → Reserved (5,6) → [CENTER on (5,6)] → Reserve (5,7) → Move

Early mode:
  Current (5,5) → Reserved (5,6) → [IMMEDIATE Reserve (5,7)] → Move
```

**Toggle:** Press **E** to enable/disable early reservation (default: OFF)

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
     - **Release all extra reserved cells** (from square/diagonal reservation)
     - Clear `reserved_subcell` and `extra_reserved_subcells`
     - **If early reservation enabled:** Immediately attempt next reservation (skip centering)
     - **If early reservation disabled:** Continue to centering phase

3. **Centering on Current Sub-Cell** (skipped if early reservation enabled and next reservation succeeds)
   - Actor has `current_subcell` only
   - Moves toward center of current sub-cell (with offset)
   - When within 1.0 pixel of center: proceed to reservation phase

4. **Reserving Next Sub-Cell**
   - Calculate direction to destination
   - **If Square reservation mode:**
     - Call `find_square_reservation()` to identify 2×2 square
     - Try `try_reserve_multiple()` for all 4 cells
     - If successful: set `reserved_subcell` and `extra_reserved_subcells`
     - If failed: fall back to single-cell
   - **If Diagonal reservation mode:**
     - Get 5 best candidate neighbors (best, ±1 CW, ±2 CW)
     - For each candidate:
       - If diagonal: find anchor, try reserve both atomically
       - If horizontal/vertical: reserve normally
     - If successful: set `reserved_subcell` and optionally `extra_reserved_subcells` (anchor)
   - **If all blocked:** Wait (no reservation made)

### 6. Candidate Selection

The `find_best_neighbors()` function returns up to 5 candidates in priority order:

```
Given current direction → target:

1. Best aligned neighbor (highest dot product with direction)
2. 1× clockwise from best
3. 1× counter-clockwise from best
4. 2× clockwise from best
5. 2× counter-clockwise from best
```

This provides intelligent fallback when the preferred sub-cell is occupied.

#### 6a. Backward Move Filtering (Default: ON)

The `filter_backward` parameter ensures actors never move away from their destination. When enabled:

**Principle:** "Any movement must decrease the distance to the destination. Otherwise don't move at all."

**Implementation:**
- Candidates with negative alignment scores (dot product < 0.0) are filtered out
- Dot product < 0.0 means the candidate direction is more than 90° away from destination
- Result: Actors may receive fewer than 5 candidates if many are backward-moving

**Example:**
```
Destination at 70°, normalized to nearest neighbor at 90°

Without filtering:
  Candidates: [90°, 45°, 135°, 0°, 180°]
  Scores:     [1.0, 0.71, 0.71, 0.34, -0.34]
  Result: All 5 returned, including 180° (moves away)

With filtering (default):
  Candidates: [90°, 45°, 135°, 0°]
  Scores:     [1.0, 0.71, 0.71, 0.34]
  Result: Only 4 returned, 180° filtered (score < 0.0)
```

**Benefits:**
- Prevents actors from ever increasing distance to destination
- Actors wait rather than making backward moves when all forward paths blocked
- Ensures monotonic progress toward destination (no detours)
- Especially important with greedy best-first navigation

**Trade-offs:**
- Actors may wait longer in congested areas (no backward escape routes)
- Can cause temporary "traffic jams" when forward movement is blocked
- Disabling filter allows more "wiggle room" for congestion resolution

**Toggle:** Press **F** to enable/disable (default: ON)

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
| **Q** | Cycle reservation mode: Square → Diagonal → NoDiagonal |
| **E** | Toggle early reservation ON/OFF |
| **F** | Toggle backward move filter ON/OFF |
| **B** | Toggle sub-cell markers (visual indicators) |
| **O** | Spawn actor at mouse position |
| **D** | Set destination for selected actor |
| **P** | Set destination for all actors |

### Visual Indicators

When sub-cell movement is enabled and markers are shown (**B** key):

- **Green circle** = Current sub-cell center
- **Yellow circle + line** = Reserved sub-cell center (primary target)
- **Black circles + lines** = Extra reserved sub-cells (from square or diagonal reservation)
- **Orange circle** = Final destination (cell-level)
- **Grid lines** = Sub-cell boundaries (when display mode is not None)

**Square Mode:** Yellow circle (primary) + 3 black circles (square formation)
**Diagonal Mode:** Yellow circle (diagonal target) + 1 black circle (H/V anchor), or just yellow (H/V move)
**NoDiagonal Mode:** Yellow circle only (horizontal/vertical moves only, no diagonals)

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
Reservation Mode: Square
Reservation Mode: Diagonal
Reservation Mode: NoDiagonal
Early Reservation: ON
Early Reservation: OFF
Filter Backward Moves: ON
Filter Backward Moves: OFF
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
    pub extra_reserved_subcells: Vec<SubCellCoord>,  // For square/diagonal reservation
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
    let enable_square = reservation_mode == ReservationMode::Square;
    let enable_diagonal = reservation_mode == ReservationMode::Diagonal;
    let enable_no_diagonal = reservation_mode == ReservationMode::NoDiagonal;

    for actor in &mut actors {
        actor.update_subcell(
            delta_time,
            &mut reservation_manager,
            enable_square,              // True for Square mode
            enable_diagonal,            // True for Diagonal mode
            enable_no_diagonal,         // True for NoDiagonal mode
            early_reservation_enabled,  // True to skip centering
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

### Reservation and Release Lifecycle

Understanding when reservations are made and released is critical to the system's behavior.

#### When New Reservations Happen

A new reservation is attempted when **all three conditions** are met:

1. **Actor has `current_subcell` but NO `reserved_subcell`** (not already moving to a reserved cell)
2. **Actor is centered on current sub-cell** (within 1.0 pixel of center)
3. **Actor has not reached destination** (distance to destination ≥ 2.0 pixels)

**Reservation Attempt Process:**
1. Calculate direction vector to destination: `(dx, dy) = (dest.x - actor.x, dest.y - actor.y)`
2. Normalize direction: `dir = (dx / magnitude, dy / magnitude)`
   - **Magnitude** = `sqrt(dx² + dy²)` (Euclidean distance to destination)
   - Used to normalize the direction vector to unit length
3. Get 5 best candidate neighbors aligned with direction
4. **Square Mode**: Try to reserve 2×2 square first, then fall back to single cells
5. **Diagonal Mode**: For diagonal candidates, try to reserve anchor + diagonal atomically
6. **Success**: Set `reserved_subcell` and optionally `extra_reserved_subcells`, actor begins moving toward reserved cell
7. **Failure**: No reservation made, actor waits at current position until next frame

#### When Releases Happen

Releases occur in **three distinct scenarios**:

**Scenario 1: Switching from Reserved to Current**

When actor crosses the midpoint between current and reserved sub-cells:

```rust
let dist_to_reserved = distance(actor.pos, reserved.center);
let dist_to_current = distance(actor.pos, current.center);

if dist_to_reserved <= dist_to_current {
    // RELEASE EVENTS:

    // 1. Release old current_subcell (if different from reserved)
    if current != reserved {
        reservation_manager.release(current, actor.id);
    }

    // 2. Release ALL extra reserved cells (from square/diagonal)
    for extra in extra_reserved_subcells {
        reservation_manager.release(extra, actor.id);
    }

    // 3. Update state
    current_subcell = reserved_subcell;
    reserved_subcell = None;
    extra_reserved_subcells.clear();
}
```

**Timing**: This happens mid-movement, as soon as actor is closer to reserved than current.

**Result**: Actor now holds only 1 reservation (the new current cell).

---

**Scenario 2: Reaching Final Destination**

When actor gets within 2.0 pixels of destination:

```rust
let dist_to_dest = distance(actor.pos, destination);

if dist_to_dest < 2.0 {
    let dest_subcell = calculate_subcell_at(destination);

    // RELEASE EVENTS:

    // 1. Release current_subcell (if different from destination subcell)
    if current_subcell != dest_subcell {
        reservation_manager.release(current_subcell, actor.id);
    }

    // 2. Release reserved_subcell (if any)
    if let Some(reserved) = reserved_subcell {
        reservation_manager.release(reserved, actor.id);
    }

    // 3. Release ALL extra reserved cells
    for extra in extra_reserved_subcells {
        reservation_manager.release(extra, actor.id);
    }

    // 4. Update state
    current_subcell = dest_subcell;
    reserved_subcell = None;
    extra_reserved_subcells.clear();

    return true; // Reached!
}
```

**Timing**: As soon as actor enters 2-pixel radius of destination.

**Result**: Actor holds only the destination sub-cell, freeing 1-3 cells for other actors.

---

**Scenario 3: Failed Square Reservation Fallback**

When square reservation fails and system falls back to single-cell:

```rust
// Try square reservation
if enable_square {
    if let Some((best, extras)) = find_square_reservation(...) {
        if reservation_manager.try_reserve_multiple(&[best, ...extras], actor.id) {
            reserved_subcell = Some(best);
            extra_reserved_subcells = vec![...extras];
            return; // Success!
        }
    }
}

// FALLBACK: Try single-cell reservation
for candidate in candidates {
    if reservation_manager.try_reserve(candidate, actor.id) {
        // RELEASE EVENT:
        // Clear any partially tracked extras (should be empty in practice)
        extra_reserved_subcells.clear();

        reserved_subcell = Some(candidate);
        return; // Success!
    }
}
```

**Timing**: Same frame as failed square reservation attempt.

**Result**: Actor reserves only 1 cell instead of 4.

---

#### Summary Table: Reservation States

| State | Current | Reserved | Extras | Description |
|-------|---------|----------|--------|-------------|
| **At Rest** | 1 cell | None | 0 | Centered, no destination or reached destination |
| **Moving to Reserved** | 1 cell | 1 cell | 0-3 | Mid-transit between sub-cells |
| **At Destination** | 1 cell | None | 0 | Within 2.0 pixels of final destination |
| **Waiting (Blocked)** | 1 cell | None | 0 | Centered but all neighbors reserved |

**Key Invariant**: An actor **always** holds at least 1 reservation (current_subcell) except during initialization.

#### Magnitude Explained

**Magnitude** refers to the **Euclidean distance** (straight-line distance) from the actor's position to the destination:

```rust
let dx = dest_x - actor_x;
let dy = dest_y - actor_y;
let magnitude = sqrt(dx * dx + dy * dy);
```

**Usage**:
- **Direction normalization**: Dividing direction vector by magnitude gives unit vector
  - `dir_x = dx / magnitude`
  - `dir_y = dy / magnitude`
- **Primary direction determination**: Comparing `abs(dx)` vs `abs(dy)` determines if movement is more horizontal or vertical
  - If `abs(dx) > abs(dy)`: Primary direction is horizontal (X)
  - If `abs(dy) >= abs(dx)`: Primary direction is vertical (Y)

**Example**:
```
Actor at (100, 100)
Destination at (130, 140)

dx = 30, dy = 40
magnitude = sqrt(30² + 40²) = sqrt(900 + 1600) = sqrt(2500) = 50.0

Normalized direction:
dir_x = 30 / 50 = 0.6
dir_y = 40 / 50 = 0.8

Primary direction: Vertical (because abs(40) > abs(30))
```

This normalized direction vector is used by `find_best_neighbors()` to select sub-cells that best align with the target direction.

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
- Reservation mode: **Square** (default), try **Diagonal** for tighter navigation, **NoDiagonal** for cardinal-only movement
- Early reservation: **OFF** (default), try **ON** for faster movement
- Backward filter: **ON** (default), ensures actors never move away from destination
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
3. **Square Reservation**: Visual confirmation of 4-cell blocks (1 yellow + 3 black)
4. **Diagonal Reservation**: Visual confirmation of diagonal + anchor (1 yellow + 1 black)
5. **NoDiagonal Reservation**: Observe horizontal/vertical-only movement (no diagonal shortcuts)
6. **Early Reservation**: Observe faster, more fluid movement (no centering pauses)
7. **Offset Modes**: Different grid alignments produce different behavior
8. **Mode Switching**: Clean transitions between reservation modes (Q key), early reservation (E key)

## Performance Characteristics

- **Space:** O(R) where R = number of active reservations
  - Square mode: up to 4R for R actors (1 primary + 3 extras)
  - Diagonal mode: up to 2R for R actors (1 target + 1 anchor)
  - Typical: 1-2 reservations per actor
- **Time per frame:**
  - Single-cell: O(A × 8) neighbor checking
  - Square: O(A × 16) (neighbors + square formation)
  - Diagonal: O(A × 8) neighbor checking + diagonal detection
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
- **Triple Reservation Strategies**:
  - **Square Mode**: Reserve 2×2 blocks for smoother, more spacious movement
  - **Diagonal Mode**: Reserve diagonal + anchor for tighter, more conservative navigation
  - **NoDiagonal Mode**: Horizontal/vertical moves only, no diagonal shortcuts
- **Early Reservation**: Optional mode for faster, more fluid movement (skip centering)
- **Backward Move Filtering**: Ensures actors never increase distance to destination (default: ON)
- **Atomic Operations**: All-or-nothing reservations prevent deadlocks
- **Single-Cell at Rest**: Actors free up space when reaching destinations

The implementation successfully handles multiple actors converging on destinations through first-come-first-served reservation with intelligent fallback strategies.
