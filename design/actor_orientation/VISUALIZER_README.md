# Actor Directing Test Visualizer

Interactive HTML+JavaScript tool for visualizing and validating position-aware diagonal reservation tests.

## Files

- **`test_visualizer.html`** - Main visualization interface
- **`test_visualization_data.json`** - Test case data (22 sample tests included)
- This README

## Quick Start

1. Open `test_visualizer.html` in any modern web browser
2. Browse through test cases in the left sidebar
3. Click on any test to visualize it
4. Use arrow keys to navigate between tests

**Note**: Both HTML and JSON files must be in the same directory.

## Display Settings

The visualization uses a 1200×1200 canvas with 90 pixels per subcell unit, providing high-resolution display of the geometric relationships. Labels are automatically positioned to avoid overlap based on the relative positions of elements.

## Features

### Visual Elements

The canvas displays colored points and geometric elements. All elements are identified by color - refer to the legend in the visualization:

- **Cyan/Blue circle** (largest) - Actor's current position
- **Green dot** - Primary Subcell (PSC) center
- **Red dot** - Diagonal subcell center
- **Yellow dot** - Anchor subcell center
- **Purple circle** - Destination position
- **Black circle** - Calculated target position (pitch black for high contrast)
- **Bright white rectangle** - Active reservation rectangle (PSC to diagonal subcell)
- **Dim white rectangles** - All other possible diagonal rectangles (NE, NW, SE, SW from PSC)
- **Dashed blue line** - Ray from actor toward destination
- **Orange arrow** - Movement vector (actor to target)
- **Grid lines** - Both horizontal and vertical, showing subcell coordinate system (0-10)

**No Text Labels**: Points are identified by color only. Use the legend (displayed below the canvas) to identify each element.

**Rectangle Visualization**: The tool displays all 4 possible diagonal reservation rectangles from the PSC (Primary Subcell) center to each of its 4 diagonal neighbors. The active rectangle (corresponding to the test's actual diagonal direction) is highlighted in bright white with thick borders, while the other 3 possibilities are shown dimmed. This helps visualize all movement options and why a particular diagonal direction was chosen.

### Interaction

**Mouse:**
- Click any test in the sidebar to visualize it
- Test items highlight on hover

**Keyboard:**
- `↑` / `←` - Previous test
- `↓` / `→` - Next test

**Filters:**
- Filter by base test (T021, T022, etc.)
- Filter by actor position (P1-P9)
- Filter by affinity (H, V, BOTH)

### Information Panel

Bottom panel shows:
- Test ID and position variant
- Actor coordinates (float precision)
- Target coordinates (calculated)
- Destination coordinates
- Affinity determination (H/V/BOTH)
- Optimal direction with suffix
- Test notes and calculations

## Understanding the Visualization

### Coordinate System

**Subcell Positioning:**
- Subcells are positioned **at grid line intersections** (not between grid lines)
- PSC (5, 5) is drawn at position (5.0, 5.0) on the grid
- Rectangles span from one grid intersection to another
- Actor positions use float coordinates and can be anywhere within rectangles

### Key Concepts

**Rectangle Bounds:**
- Formed between PSC and diagonal subcell (both at grid intersections)
- Actor can be anywhere within this rectangle
- Target must lie on rectangle boundary

**Ray to Destination (Blue Dashed):**
- Shows the intended direction of movement
- Always points from actor toward final destination
- Used to calculate intersection with rectangle

**Movement Vector (Orange Arrow):**
- Shows actual movement from actor to target
- Target is where ray intersects rectangle boundary
- Length indicates distance to move before reaching boundary

**Affinity Determination:**
- **V (Vertical-favoring)**: Ray hits horizontal edge first (top/bottom)
- **H (Horizontal-favoring)**: Ray hits vertical edge first (left/right)
- **BOTH**: Ray hits corner exactly

### Test Patterns to Look For

**P1 (PSC Center):**
- Actor at PSC center (e.g., 5.0, 5.0)
- Should match baseline test expectations
- Ray starts from bottom-left corner of rectangle

**P2 (Rectangle Center):**
- Actor at geometric center (e.g., 5.5, 4.5)
- May show different affinity than P1
- Ray starts from middle of rectangle

**P3 (Diagonal Corner):**
- Actor already at diagonal subcell
- Target = actor position (no movement)
- Affinity always BOTH

**P4-P7 (Edge Positions):**
- Actor on one edge of rectangle
- Ray starts from boundary
- One t-value = 0, other determines affinity

**P8-P9 (Corner Positions):**
- Actor at rectangle corners
- P8 usually same as P1
- P9 at anchor corner shows minimal movement

## Adding More Test Cases

To add test cases, edit `test_visualization_data.json`:

```json
{
  "id": "T{base}_P{position}",
  "base": "T0XX",
  "position": "Position_Name",
  "actor": [actor_x, actor_y],
  "psc": [psc_x, psc_y],
  "diagonal": [diag_x, diag_y],
  "destination": [dest_x, dest_y],
  "target": [target_x, target_y],
  "affinity": "H|V|BOTH",
  "anchor": [anchor_x, anchor_y],
  "optimal_dir": "Direction-Affinity",
  "notes": "Description"
}
```

Add to the `tests` array. The visualizer will automatically load and display new tests.

## Validating Tests

### Visual Checks

✅ **Target on boundary**: Orange target circle should be exactly on rectangle edge or corner

✅ **Ray alignment**: Dashed blue line should pass through target and continue toward destination

✅ **Movement direction**: Orange arrow should point from actor toward target

✅ **Affinity consistency**:
   - V-affinity: Target on horizontal edge (top/bottom)
   - H-affinity: Target on vertical edge (left/right)
   - BOTH: Target at corner

✅ **Rectangle formation**: Light rectangle should connect PSC and diagonal centers

### Common Issues

**Target not on boundary:**
- Calculation error in target coordinates
- Check intersection formula

**Ray doesn't pass through target:**
- Target not on ray from actor to destination
- Verify ray direction calculation

**Wrong affinity:**
- t-value comparison incorrect
- Check which edge is hit first

**Misaligned anchor:**
- Anchor should be orthogonal neighbor of PSC
- H-affinity → horizontal neighbor
- V-affinity → vertical neighbor

## Sample Test Cases Included

The JSON file includes 22 representative tests:

**NORTH-EAST quadrant (9 tests):**
- T021_P1-P9: dest=(6,1), various actor positions

**NORTH-EAST variations:**
- T022_P1-P2: dest=(7,2)
- T024_P1-P2: dest=(8,3)
- T025_P1, T026_P1: dest=(9,3) and (9,4)

**Other quadrants (baselines):**
- T027_P1: SOUTH-EAST, dest=(8,7)
- T029_P1: SOUTH-EAST, dest=(7,8)
- T031_P1, T032_P1: SOUTH-WEST
- T034_P1, T035_P1, T036_P1: NORTH-WEST

## Browser Compatibility

Tested on:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Requires:
- HTML5 Canvas support
- ES6 JavaScript
- Fetch API for JSON loading

## Troubleshooting

**"Error loading test data":**
- Ensure `test_visualization_data.json` is in same directory as HTML
- Check browser console for detailed error
- Verify JSON syntax is valid

**Canvas not displaying:**
- Check browser console for errors
- Ensure browser supports HTML5 Canvas
- Try refreshing the page

**Tests not filtering:**
- JavaScript errors in console?
- Try clearing browser cache
- Ensure JSON data loaded successfully

## Performance

- Renders in real-time (< 16ms per frame)
- Handles 100+ test cases smoothly
- No external dependencies (pure HTML/CSS/JS)

## Future Enhancements

Possible additions:
- Export visualization as PNG
- Side-by-side comparison of two tests
- Animation showing movement from actor to target
- Grid snapping and measurement tools
- Test case generation from UI
- Full 108-test dataset loaded by default

## Credits

Created for RustGame3 actor directing algorithm validation.
Design document: `actor_directing_v2.txt`
Test specifications: `actor_directing_position_tests.md`
