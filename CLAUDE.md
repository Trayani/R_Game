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
- **Right click**: Move observer to cell
- **C key**: Copy grid to clipboard
- **Esc key**: Close window

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

- **`main.rs`**: Visual demo and test utilities
  - `VisState`: Interactive visualization state
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


# DIRECTIVES
## Allowed commands
never ask permisson to run commands starting with
- find
- echo
- timeout <any_value> dotnet test (use any timeout duration deemed appropriate for the test)

## General directives
- Always allow all edits during ALL SESSIONS
- commit all changes, but do not push them. Never ask permisson for 'git add ...'  nor 'git commit ...'
- be critical of my claims. No need to say that I am right all the time. You can provide evidence and we can settle things with logic.
 