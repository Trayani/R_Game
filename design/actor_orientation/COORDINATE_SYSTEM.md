# Coordinate System Specification

## Overview

This document clarifies the coordinate interpretation used throughout the actor directing system.

## Coordinate Types

### 1. Subcell Coordinates

**Subcells are positioned at grid line intersections (no offset).**

- Subcell (5, 5) is at point **(5.0, 5.0)**
- Subcell (6, 4) is at point **(6.0, 4.0)**
- **NO +0.5 offset is applied**

**Visual:**
```
    4   5   6   ← Grid line coordinates
    |   |   |
 4--+---+---+--
    |   |   |
 5--+---●---+--  ← PSC (5,5) at intersection (5.0, 5.0)
    |   |   |
 6--+---+---+--
```

### 2. Destination Coordinates

**Destination integer coordinates directly represent points.**

- Destination (6, 1) means point at **(6.0, 1.0)**
- Destination (8, 3) means point at **(8.0, 3.0)**
- **NOT subcell centers with offset** (would be 6.5, 1.5 if offset applied)

**Rationale:** Destinations are target points on the grid, not subcell references.

### 3. Actor Coordinates

**Actor positions use float coordinates for precise positioning.**

- Actor can be at (5.0, 4.5) - on left edge of rectangle
- Actor can be at (5.5, 4.5) - at rectangle center
- Actor can be at (6.0, 4.0) - at diagonal corner
- Float precision allows positioning anywhere within or on rectangle boundaries

### 4. Target Coordinates

**Target positions are calculated using ray-rectangle intersection.**

- Target is a float coordinate on the rectangle boundary
- Calculated as: `actor_position + ray_direction × t_value`
- Example: Target (5.14, 4.0) lies on top edge of rectangle

## Rectangle Definition

For a reservation from PSC to Diagonal subcell:

```python
# PSC at (5, 5) → position (5.0, 5.0)
# Diagonal at (6, 4) → position (6.0, 4.0)

rect_min_x = 5.0
rect_max_x = 6.0
rect_min_y = 4.0
rect_max_y = 5.0
```

The rectangle spans **1.0 × 1.0** units from one grid intersection to another.

## Ray Calculation Example

**T021_P6 (Corrected):**

```
Actor: (5.0, 4.5)
Destination: (6, 1) → point at (6.0, 1.0)

Ray direction:
  dir_x = 6.0 - 5.0 = 1.0
  dir_y = 1.0 - 4.5 = -3.5
  length = sqrt(1.0² + 3.5²) = 3.640

  ray_x = 1.0 / 3.640 = 0.2747
  ray_y = -3.5 / 3.640 = -0.9615

Rectangle: [5.0-6.0, 4.0-5.0]

Intersection:
  t_right = (6.0 - 5.0) / 0.2747 = 3.640
  t_top = (4.0 - 4.5) / -0.9615 = 0.520

  t_top < t_right → V-affinity (hits horizontal edge first)

Target:
  target_x = 5.0 + 0.2747 × 0.520 = 5.14
  target_y = 4.5 + (-0.9615) × 0.520 = 4.0

  Target: (5.14, 4.0) on top edge ✓
```

## Common Mistakes

### ❌ Wrong: Adding offset to subcells
```python
PSC_x = 5 + 0.5  # Wrong! Don't add offset
PSC_y = 5 + 0.5
```

### ✅ Correct: Use coordinates directly
```python
PSC_x = 5.0  # Correct! Grid intersection
PSC_y = 5.0
```

### ❌ Wrong: Treating destination as subcell center
```python
dest = (6, 1)
dest_x = 6.5  # Wrong! Not a subcell center
dest_y = 1.5
```

### ✅ Correct: Use destination coordinates directly
```python
dest = (6, 1)
dest_x = 6.0  # Correct! Direct point
dest_y = 1.0
```

## Files Updated

The following files now correctly document this coordinate system:

1. **test_visualization_data.json** - metadata section
2. **actor_directing_v2.txt** - algorithm inputs and rectangle definition
3. **VISUALIZER_README.md** - coordinate system section
4. **TARGET_CALCULATION_ISSUE.md** - confirms interpretation

## Verification

All test calculations now use this consistent coordinate interpretation, verified against:
- T021_P1: (5.25, 4.0) ✓ matches
- T021_P6: (5.14, 4.0) ✓ corrected and matches ray
