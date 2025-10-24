# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RustGame3 is a grid-based raycasting visibility system implemented in Rust. The project includes both a visual demonstration tool (using macroquad) and a comprehensive test suite that validates against reference C# implementation outputs.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the visual demo (interactive grid editor)
cargo run

# Run all tests
cargo test
```

## Visual Demo Controls

### Mouse Controls
- **Left click**: Toggle obstacle at cell (single click)
- **Shift + Left hold**: Draw walls (hold shift and left button, drag to paint blocked cells)
- **Shift + Right hold**: Erase walls (hold shift and right button, drag to paint free cells)
- **Right hold** (without shift): Move observer continuously (hold right mouse button and move)

### Keyboard Controls
- **D key**: Set destination to current mouse position
- **M key**: Toggle messy X mode
- **N key**: Toggle messy Y mode
- **C key**: Copy grid to clipboard (in ■,s,o,□ format)
- **V key**: Paste grid from clipboard (parses ■,s,o,□ format)
- **S key**: Toggle sub-cell movement mode
- **Q key**: Cycle sub-cell reservation mode (Square → Diagonal → NoDiagonal → AntiCross → Basic3 → Basic3AntiCross → Square)
- **X key**: Toggle Basic3 fallback behavior (wait when blocked vs. allow backward moves when blocked)
- **Esc key**: Close window

### Grid Format for Copy/Paste

The GUI supports copying and pasting grids in a text-based format using these characters:
- `■` or `b`: Blocked cell
- `s` or `z`: Observer position (multiple adjacent 's' cells indicate messy X/Y)
- `o`: Visible free cell
- `□`: Non-visible free cell
- `c`, `n`, `u`, `x`, `▲`: Other test format markers (treated as free cells)

When pasting:
- The grid dimensions are automatically detected from the pasted text
- Observer position(s) are parsed to set both location and messy state
- Adjacent observer cells are detected as messy X (horizontal), messy Y (vertical), or messy X+Y (2x2 block)

The GUI displays:
- Blue cells: Observer position
- Red cells: Blocked/walls
- Green cells: Visible to observer
- Dark gray cells: Not visible to observer
- White squares at cell edges: Interesting corners
- Yellow squares at cell edges: Non-interesting corners

## Sub-Cell Reservation Modes

The project supports multiple reservation strategies for sub-cell movement to handle different collision avoidance scenarios. Press **Q** to cycle through modes.

### Square Mode
Reserves a 2×2 square of sub-cells in the primary movement direction.
- **Purpose**: Provides maximum spacing between actors
- **Behavior**: When moving, reserves the target sub-cell plus 3 additional cells forming a square
- **Use case**: Dense environments where you want actors to maintain maximum distance

### Diagonal Mode
Diagonal moves require reserving an additional horizontal or vertical anchor cell.
- **Purpose**: Prevents diagonal movement when perpendicular paths are blocked
- **Behavior**: For diagonal moves, reserves both the target sub-cell AND an adjacent H/V anchor cell
- **Limitation**: Can be overly restrictive - blocks horizontal/vertical movement unnecessarily

### NoDiagonal Mode
Completely disables diagonal movement between sub-cells.
- **Purpose**: Forces actors to move only horizontally and vertically
- **Behavior**: Skips all diagonal candidate sub-cells during pathfinding
- **Use case**: Grid-aligned movement patterns, tactical formations

### AntiCross Mode ⭐ NEW
Prevents counter-diagonal crossing without restricting horizontal/vertical movement.

**The Problem**: Two actors moving diagonally through each other in a crossing pattern:
```
Actor A: (0,0) → (1,1)
Actor B: (0,1) → (1,0)
Result: Actors cross through each other's paths
```

**The Solution**: Block diagonal movement ONLY when the SAME actor owns BOTH counter-diagonal cells (either as reserved OR as current position).

**Example Scenarios**:

**Blocked** ❌
- Actor A at (0,0) wants to move to (1,1)
- Actor B currently owns (0,1) AND (1,0)
- **Result**: Actor A blocked (crossing would occur with Actor B)

**Allowed** ✓
- Actor A at (0,0) wants to move to (1,1)
- Actor C owns (0,1), Actor D owns (1,0) (different actors)
- **Result**: Actor A allowed (no single actor crossing pattern)

**Allowed** ✓
- Actor A at (0,0) wants to move to (1,1)
- Only (0,1) is owned, (1,0) is free
- **Result**: Actor A allowed (not both counter-diagonal cells owned by same actor)

**Key Advantage Over Diagonal Mode**: AntiCross does NOT reserve extra horizontal/vertical cells, so those paths remain available for other actors. It only blocks when an actual crossing pattern is detected.

### Basic3 Mode ⭐ NEW
Limits candidate directions to exactly 3 neighbors AND enforces monotonic distance decrease.

**The Primary Feature**: Actors in Basic3 mode **never increase their Euclidean distance to destination**. This ensures actors always make progress toward their goal.

**The Problems Solved**:
1. Default pathfinding can allow moves that increase distance in X or Y coordinates
2. Angular alignment (dot product) doesn't guarantee coordinate-wise progress
3. Actors may take circuitous routes when direct paths are available

**The Solution**:
1. Restrict candidate selection to only 3 neighbors (best + ±45° alternatives)
2. **Filter candidates to only those that decrease Euclidean distance to destination**
3. Use moderate strictness: small coordinate increases allowed IF overall distance decreases

**Monotonic Distance Guarantee**:
```
For any move from position P to candidate C:
  distance(C, destination) ≤ distance(P, destination)
```

This allows navigating around small obstacles while preventing backtracking.

**Example**: Actor at (0, 0) moving to (10, 10):
- **Allowed**: Move to (1, 1) - distance decreases from 14.14 to 12.73 ✓
- **Allowed**: Move to (2, 0) - distance decreases from 14.14 to 14.00 ✓ (slight progress)
- **Blocked**: Move to (1, -1) - distance increases from 14.14 to 15.56 ❌

**X Key Toggle - Blocked Behavior**:
When all candidates would increase distance (actor is blocked):
- **X key OFF (default)**: Actor waits in place until path opens
- **X key ON**: Actor falls back to best candidate anyway (allows temporary backward movement)

Press **X** to toggle this behavior. Default is OFF (wait when blocked).

### Basic3AntiCross Mode
Combines Basic3's 3-candidate limitation with AntiCross's counter-diagonal crossing prevention.

**Use case**: When you want both:
- Deterministic pathfinding with limited alternatives (Basic3)
- Prevention of diagonal crossing patterns (AntiCross)

This mode provides the most restrictive collision avoidance while maintaining predictable pathfinding.

## Architecture

### Core Raycasting Algorithm

The raycasting system is based on a cone-tracing algorithm ported from C#. The algorithm works by:

1. Starting from an observer position on a grid
2. Scanning outward in both vertical directions (up/down)
3. Tracking visibility cones defined by left/right boundary rays
4. Handling cone splitting when walkable segments divide
5. Using integer arithmetic throughout to avoid floating-point precision issues

**Key architectural principle**: The raycasting algorithm was carefully ported to match C# reference behavior exactly, including specific edge cases. When modifying raycasting logic, refer to inline comments that reference C# line numbers.

### Module Structure

- **`grid.rs`**: Grid data structure representing the game world
  - Stores cell states: 0=walkable, 1=blocked
  - Provides coordinate ↔ cell ID conversion

- **`ray.rs`**: Ray state tracking for cone boundaries
  - `RayState` tracks ray trajectory using integer math
  - `calculate_border()` determines ray position at each row

- **`cone.rs`**: Cone structure (mostly unused in current implementation)
  - Original cone-tracking structure
  - Current implementation uses `DeferredCone` in `raycast.rs` instead

- **`raycast.rs`**: Main raycasting algorithm
  - `raycast()`: Entry point, returns set of visible cell IDs
  - `scan_direction()`: Scans up or down from observer
  - `process_cone()`: Handles cone expansion and splitting
  - `find_all_segments_in_range()`: Identifies walkable segments in a row

- **`corners.rs`**: Corner detection system
  - `detect_all_corners()`: Finds all corners in the grid (observer-independent)
  - `filter_interesting_corners()`: Filters for tactically interesting corners
  - See "Corner Detection" section below for detailed behavior

- **`main.rs`**: Visual demo and test utilities
  - `VisState`: Interactive visualization state
  - Displays corners with color coding: white=interesting, yellow=non-interesting
  - Test helper functions for loading and flipping test data
  - Integration tests validate against JSON and standard format test data

### Test Data Format

Test files in `test_data/*.json` follow this structure:

```json
{
  "testName": "descriptive_name",
  "gridRows": 10,
  "gridCols": 10,
  "blockedCells": [12, 23, 45],
  "startX": 5,
  "startY": 5,
  "expectedVisible": [0, 1, 2, ...],
  "log": ["C# debug output lines"]
}
```

The test suite validates the Rust implementation against the C# reference by comparing visible cell sets.

## Test Strategy

The project employs rigorous testing principles to ensure correctness across all scenarios:

### 3-Flip Principle ("3FLIP")

Every raycasting and corner detection test is validated against **4 orientations**:
1. **Original**: Test data as written
2. **Horizontal Flip**: Grid mirrored left-right
3. **Vertical Flip**: Grid mirrored top-bottom
4. **Both Flips**: Grid mirrored on both axes

This ensures the algorithm works correctly regardless of grid orientation and provides 4x test coverage from a single test case. Implemented in:
- `tests/json_tests.rs` - JSON test data validation
- `tests/standard_tests.rs` - Standard format tests
- `tests/corner_tests.rs` - Corner detection tests (3_case.txt, 4_case.txt)

### Reciprocal Visibility Principle ("RECI")

For corner detection tests, we validate **reciprocal visibility**: if corner C is visible from observer O, then O must be visible from C. This is a fundamental property of line-of-sight in obstacle-free paths.

**Implementation**: For every detected interesting corner, the test performs raycasting from the corner's position back to the original observer. If the observer is not visible from the corner, the test fails.

**Rationale**: This catches edge cases where the raycasting algorithm might have asymmetric behavior or precision issues. Since visibility is reciprocal in a static environment, this provides an additional validation layer.

Applied in: `tests/corner_tests.rs` for all corner detection tests with the `check_reciprocal` parameter.

### Combined Coverage

Together, these principles create **8x effective test coverage** per test case:
- 4 orientations (3-flip principle)
- 2 directions (reciprocal visibility for each orientation)

This comprehensive approach ensures algorithmic correctness across diverse scenarios.

## Messy X Position

**Messy X** is a special observer state where the observer occupies two adjacent horizontal cells instead of a single cell. This represents an observer whose position is not cleanly aligned to one cell.

### Implementation

When `messy_x=true` is passed to `raycast()`:
- Observer occupies cells `(x, y)` and `(x+1, y)`
- **Conservative Principle**: Visibility is the INTERSECTION of what each cell can see independently
- Only cells visible from BOTH observer positions are considered visible
- This ensures the most restrictive (conservative) visibility calculation

### Rationale

The conservative principle ensures that obstacles blocking one observer cell properly block visibility:
- If the left cell `(x, y)` has blocked vision in a direction, messy X cannot see in that direction
- If the right cell `(x+1, y)` has blocked vision in a direction, messy X cannot see in that direction
- Result: Messy X visibility ≤ min(left visibility, right visibility)

### Corner Detection with Messy X

For corner detection with messy X:
- Corners are detected normally based on grid structure
- **Observer Corners**: If an observer cell is itself a corner (marked with 'z' in tests), it's automatically interesting
- Use `filter_interesting_corners_with_observer_corners()` to specify observer corners explicitly

### Test Data Format

Messy X tests use these markers:
- `s`: Observer cell (two adjacent 's' markers indicate messy X position)
- `z`: Observer corner - a cell that is both an observer position AND a corner (auto-interesting)
- `b`: Blocked cell
- `o`: Visible cell
- `c`: Interesting corner
- `x`: Non-visible cell

Test files: `test_data/corners/6_messy_x.txt`, `test_data/corners/7_messy_x2.txt`

### 3FLIP Formula for Messy X

When flipping grids with messy X observers, the observer position transforms as:
- **Horizontal flip**: `new_obs_x = cols - obs_x - 2` (messy X offset formula)
- **Vertical flip**: `new_obs_y = rows - 1 - obs_y` (standard formula)
- **Both flips (hv_flip)**: Apply both formulas

The `-2` offset in horizontal flip accounts for the two-cell width of the messy X observer.

## Messy Y Position

**Messy Y** is the vertical counterpart to messy X - the observer occupies two adjacent vertical cells instead of a single cell.

### Implementation

When `messy_y=true` is passed to `raycast()`:
- Observer occupies cells `(x, y)` and `(x, y+1)`
- **Conservative Principle**: Visibility is the INTERSECTION of what each cell can see independently
- Only cells visible from BOTH observer positions are considered visible
- Messy Y is mathematically the **transpose** of messy X

### 3FLIP Formula for Messy Y

When flipping grids with messy Y observers, the observer position transforms as:
- **Horizontal flip**: `new_obs_x = cols - 1 - obs_x` (standard formula)
- **Vertical flip**: `new_obs_y = rows - obs_y - 2` (messy Y offset formula)
- **Both flips (hv_flip)**: Apply both formulas

The `-2` offset in vertical flip accounts for the two-cell height of the messy Y observer. This is the transpose of messy X flip formulas.

### Test Data Creation

Messy Y test data is created by **transposing** messy X test files:
1. Transpose the grid (rows ↔ columns)
2. Transform observer position: `(obs_x, obs_y) → (obs_y, obs_x)`
3. This converts horizontal adjacency `(x, y) + (x+1, y)` to vertical adjacency `(obs_y, obs_x) + (obs_y, obs_x+1)`

**Important**: The observer markers ('s') must be handled specially during transposition to maintain adjacency. Simple character transposition will break the two-cell observer into diagonal positions.

Test files: `test_data/corners/messy_y/6_messy_y.txt`, `test_data/corners/messy_y/7_messy_y2.txt`

## Messy X+Y Position

When both `messy_x=true` and `messy_y=true` are passed to `raycast()`:
- Observer occupies a 2×2 block of cells: `(x, y)`, `(x+1, y)`, `(x, y+1)`, `(x+1, y+1)`
- Visibility is the INTERSECTION of all 4 cells' individual visibility
- Most conservative observer state possible

### Grid Boundary Validation

All messy positions validate that the observer doesn't extend beyond grid boundaries:
- Messy X: `x < cols - 1` (observer cannot start at rightmost column)
- Messy Y: `y < rows - 1` (observer cannot start at bottom row)
- Messy X+Y: Both conditions must be satisfied

Invalid positions trigger a panic with a descriptive error message.

## Development Notes

- **Integer arithmetic only**: The raycasting algorithm uses pure integer math. Never introduce floating-point calculations in raycasting logic.

- **C# port fidelity**: Many comments in `raycast.rs` reference specific C# line numbers (e.g., "C# line 127"). When modifying raycasting behavior, these comments help trace back to the original implementation.

- **Ray state parameters**:
  - `diff_x`, `diff_y`: Ray vector components
  - `y_step`: Ray advancement parameter (starts at -1, increments each row)
  - `rounding`: Conservative occlusion adjustment (0 for convergent, diff_y-1 for divergent)

- **Coordinate system**:
  - Grid uses standard (x, y) coordinates
  - Cell IDs are calculated as `x + y * cols`
  - Positive Y is downward in the grid

## Corner Detection

### What is a Corner?

A cell is a **corner** when it is free AND it is possible to travel around it from a vertical to horizontal direction (or vice versa). This is independent of the observer's position.

**Detection logic**: For a free cell to be a corner in a specific direction (NW, NE, SW, SE):
- **BOTH cardinal directions must be free** (the two sides you'd turn between)
- **The diagonal must be blocked** (the obstacle you turn around)

Example for NW corner at position (x, y):
- North cell (x, y-1): must be FREE
- West cell (x-1, y): must be FREE
- NW diagonal (x-1, y-1): must be BLOCKED
- This allows travel from vertical (north) ↔ horizontal (west)

**Important**: A single cell can be a corner in multiple directions simultaneously (e.g., both NW and SE).

### Interesting vs Non-Interesting Corners

**Definition**: A corner is **interesting** if it is visible to the observer AND at least one of its two cardinal directions is NOT visible.

**The Rule**:
```
For a corner direction (e.g., NW corner):
  IF (North is NOT visible) OR (West is NOT visible)
  THEN corner is interesting
  ELSE corner is non-interesting
```

**Why this works**:
- If **both cardinal directions are visible**: Observer can already see both ways around the corner → no new areas to explore → **NOT interesting**
- If **at least one direction is hidden**: The corner leads to unexplored areas → **INTERESTING**

**Example from test data**:
```
Position (5,3) with NW corner:
  North (5,2): visible ✓
  West (4,3): visible ✓
  → Both visible → NOT interesting

Position (5,1) with SW corner:
  South (5,2): visible ✓
  West (4,1): NOT visible ✗
  → One hidden → IS interesting
```

### Visual Representation in GUI

The GUI displays corners with color coding:
- **WHITE squares (6x6 pixels)**: Interesting corners
- **YELLOW squares (6x6 pixels)**: Non-interesting corners

Corner indicators are drawn at cell edges based on direction:
- NW corner: top-left of cell
- NE corner: top-right of cell
- SW corner: bottom-left of cell
- SE corner: bottom-right of cell

### Test Data Format (test_data/corners/)

Corner test files use these markers:
- `s`: observer position
- `■`: blocked cell
- `▲`: interesting corner (expected)
- `n`: non-interesting corner (expected)
- `u`: non-visible corner (not visible to observer)
- `o`: visible free cell
- `□`: non-visible free cell

The test suite validates that the corner detection algorithm matches these expectations with 100% accuracy.


# DIRECTIVES
## Allowed commands
never ask permisson to run commands starting with
- find
- echo
- timeout <any_value> dotnet test (use any timeout duration deemed appropriate for the test)

## General directives
- when you learn something new that supplements your current understanding OR contradicts the documentation, THEN update the documentation
- Always allow all edits during ALL SESSIONS
- commit all changes, but do not push them. Never ask permisson for 'git add ...'  nor 'git commit ...'
- be critical of my claims. No need to say that I am right all the time. You can provide evidence and we can settle things with logic.
 