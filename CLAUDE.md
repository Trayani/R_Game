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

- **Left click**: Toggle obstacle at cell
- **Right hold**: Move observer continuously (hold right mouse button and move)
- **C key**: Copy grid to clipboard
- **Esc key**: Close window

The GUI displays:
- Blue cells: Observer position
- Red cells: Blocked/walls
- Green cells: Visible to observer
- Dark gray cells: Not visible to observer
- White squares at cell edges: Interesting corners
- Yellow squares at cell edges: Non-interesting corners

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

### 3-Flip Principle

Every raycasting and corner detection test is validated against **4 orientations**:
1. **Original**: Test data as written
2. **Horizontal Flip**: Grid mirrored left-right
3. **Vertical Flip**: Grid mirrored top-bottom
4. **Both Flips**: Grid mirrored on both axes

This ensures the algorithm works correctly regardless of grid orientation and provides 4x test coverage from a single test case. Implemented in:
- `tests/json_tests.rs` - JSON test data validation
- `tests/standard_tests.rs` - Standard format tests
- `tests/corner_tests.rs` - Corner detection tests (3_case.txt, 4_case.txt)

### Reciprocal Visibility Principle

For corner detection tests, we validate **reciprocal visibility**: if corner C is visible from observer O, then O must be visible from C. This is a fundamental property of line-of-sight in obstacle-free paths.

**Implementation**: For every detected interesting corner, the test performs raycasting from the corner's position back to the original observer. If the observer is not visible from the corner, the test fails.

**Rationale**: This catches edge cases where the raycasting algorithm might have asymmetric behavior or precision issues. Since visibility is reciprocal in a static environment, this provides an additional validation layer.

Applied in: `tests/corner_tests.rs` for all corner detection tests with the `check_reciprocal` parameter.

### Combined Coverage

Together, these principles create **8x effective test coverage** per test case:
- 4 orientations (3-flip principle)
- 2 directions (reciprocal visibility for each orientation)

This comprehensive approach ensures algorithmic correctness across diverse scenarios.

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
 