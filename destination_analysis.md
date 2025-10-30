# Actor Destination and Locked Target Analysis

## Summary

**Finding**: The `locked_target` values are NOT pointing directly to the destination. Instead, they point to the intersection of the destination ray with the **rectangular boundary between the current subcell (PSC) and the diagonal candidate subcell**.

**This is by design** according to actor_directing_v2, but it creates a zigzag pattern rather than direct movement toward the destination.

---

## 1. Actual Destination Coordinates

### Destination Set at 2743ms
- **Cell coordinates**: (15, 23)
- **Raw screen position**: (465.0, 470.0) [cell center]
- **Quantized to subcell grid**: **(457.5, 465.0)**
- **Subcell grid point**: (30, 46)

### Actor Starting Position
- **Spawn position**: (235.0, 303.0)

### Ideal Path
- **Vector**: (222.5, 162.0)
- **Distance**: 275.2 pixels
- **Angle**: 36.1° (northeast)

---

## 2. Locked Target Values Analysis

### Key Finding: Targets Point to Rectangle Boundaries, Not Destination

From ActorDirecting logs, examining sample locked_target values:

#### Event at 2743ms
- **PSC**: (9, 18, 1, 1) → approx position (292.5, 375.0)
- **locked_target**: (292.5, 373.18)
- **Vector to target**: (0.0, -1.8) ← points UPWARD
- **Vector to dest**: (165.0, 90.0) ← should point NORTHEAST
- **Angle deviation**: **118.6°** ❌
- **Analysis**: Target is pointing in nearly opposite direction from destination!

#### Event at 2772ms
- **PSC**: (10, 19, 1, 1) → approx position (322.5, 395.0)
- **locked_target**: (307.5, 381.53)
- **Vector to target**: (-15.0, -13.5) ← points NORTHWEST
- **Vector to dest**: (135.0, 70.0) ← should point NORTHEAST
- **Angle deviation**: **165.5°** ❌

#### Event at 5625ms (near destination)
- **PSC**: (14, 23, 1, 1) → approx position (442.5, 475.0)
- **locked_target**: (427.5, 448.31)
- **Vector to target**: (-15.0, -26.7) ← points SOUTHWEST
- **Vector to dest**: (15.0, -10.0) ← should point NORTHEAST
- **Angle deviation**: **85.6°** ❌

### Pattern Detection: Zigzagging

Consecutive target movements show wild oscillation:

| Timestamp | Target Position | Movement Direction | Deviation from Ideal |
|-----------|----------------|-------------------|---------------------|
| 2743ms | (292.5, 373.2) | - | - |
| 2772ms | (307.5, 381.5) | 29.1° | 7.0° ✓ (aligned) |
| 2856ms | (236.0, 445.0) | 138.4° | **102.4°** ❌ |
| 3045ms | (244.1, 295.0) | -86.9° | **123.0°** ❌ |

The target jumps around wildly, deviating over 100° from the ideal direction!

---

## 3. Subcell Grid Geometry

### Parameters
- Cell size: 30×20 pixels
- Subcell size: 15×10 pixels
- Grid size: 2×2 subcells per cell
- Offset: (0.5, 0.5)

### Subcell Centers for Cell (0,0)
| Subcell | Screen Position |
|---------|----------------|
| (0,0) | (7.5, 5.0) |
| (1,0) | (22.5, 5.0) |
| (0,1) | (7.5, 15.0) |
| (1,1) | (22.5, 15.0) |

### Subcell Boundaries
- **Horizontal boundary** (between sub_y=0 and sub_y=1): y = 10.0 per cell
- **Vertical boundary** (between sub_x=0 and sub_x=1): x = 15.0 per cell

---

## 4. Path Geometry Analysis

### The Problem: Rectangle Intersection ≠ Direct Path

The `calculate_affinity_and_target()` function computes:

1. **Rectangle bounds** between PSC and diagonal subcell
2. **Ray** from actor position toward destination
3. **Intersection** of ray with rectangle boundary
4. **Target** = intersection point on boundary edge

**This is NOT the same as moving directly toward the destination!**

### Example: Why It Creates Zigzag

Consider actor at PSC (9, 18, 1, 1) moving toward (15, 23):

1. Actor position: (292.5, 375.0)
2. Destination: (457.5, 465.0)
3. Ray direction: northeast (36.1°)
4. Diagonal candidate: (10, 19, 0, 0)
5. **Rectangle**: between (292.5, 375.0) and (307.5, 395.0)
6. **Ray intersects rectangle** at vertical edge: (292.5, 373.18)
7. **This point is BEHIND the actor!**

The actor is told to move to (292.5, 373.18), which is:
- Slightly upward (dy = -1.8)
- Not toward the destination at all

---

## 5. Understanding actor_directing_v2 Target Calculation

### From actor.rs Lines 760-934

The algorithm:
```rust
// Step 1: Define rectangle bounds
let rect = Rectangle::between(PSC, diagonal);

// Step 2: Calculate ray direction
let ray = Ray::from_actor_to_dest(actor_pos, dest_pos);

// Step 3: Ray-rectangle intersection
let (t_vertical, t_horizontal) = ray.intersect(rect);

// Step 4: Determine affinity and target
if t_vertical < t_horizontal {
    // Horizontal affinity: hits vertical edge first
    target = ray.at(t_vertical);  // Point on vertical edge
    anchor = horizontal_neighbor;
} else {
    // Vertical affinity: hits horizontal edge first
    target = ray.at(t_horizontal);  // Point on horizontal edge
    anchor = vertical_neighbor;
}
```

### The Intent

The system is designed for **subcell-to-subcell navigation**, where:
- Each move is from one subcell to an adjacent subcell
- The rectangle represents the "movement area" for that step
- The target should guide smooth diagonal movement within that area

### The Issue

When destination is many cells away:
- The ray toward destination may intersect the current rectangle at awkward angles
- The intersection point may not align with direct progress toward destination
- Actor ends up targeting local rectangle boundaries instead of global destination

---

## 6. What "Subcell Boundary" Means

**Geometrically**: A subcell boundary is a **line between two subcell center points**.

For offset=(0.5, 0.5) and grid_size=2:
- **Vertical boundaries**: x = cell_x * 30 + 15 (midpoint of cell)
- **Horizontal boundaries**: y = cell_y * 20 + 10 (midpoint of cell)

These are the **edges of the rectangles** that actor_directing_v2 computes.

**In actor_directing_v2**: The "subcell boundary" is the edge of the rectangle formed by:
- Corner 1: PSC center
- Corner 2: Diagonal subcell center

The target is placed **on this boundary**, not on the path to the destination.

---

## 7. Are Locked Targets Guiding Directly? **NO**

### Evidence

1. **Angle deviations**: 85°-165° from ideal path (should be <10°)
2. **Cross products**: 299-914 (should be near 0 for collinear)
3. **Direction reversals**: Targets sometimes point backward from destination
4. **Oscillation**: Consecutive targets change direction by >100°

### Conclusion

The locked_target values are **NOT guiding actors directly toward the destination**. Instead:

- They guide actors to the nearest boundary of the current movement rectangle
- This creates a **staircase/zigzag pattern** as actors move from rectangle to rectangle
- Each step is toward a local target (rectangle boundary), not the global destination

---

## 8. Visual Proof

```
Destination: (457.5, 465.0)  ★
                              ↑
                              │ (Should move directly here)
                              │
Actor at (292.5, 375.0)  ●───┘
    │
    │ (But locked_target says...)
    ↓
Target: (292.5, 373.2)  ✕  (Moves UPWARD instead!)
```

The actor is at the bottom-right of a subcell rectangle. The destination is northeast. But the locked_target points upward (north), because that's where the destination ray exits the current rectangle's vertical boundary.

This is **geometrically correct for ray-rectangle intersection**, but it's **not correct for direct destination-based movement**.

---

## 9. Recommendations

### Issue
The current system uses rectangle-boundary targets, which create zigzag paths.

### Expected Behavior (from user)
> "If target type is DIRECT: locked_target should always point to intermediate subcell boundary (a vertical/horizontal line between subcell points!), and it should guide the actor directly to destination"

### Possible Solutions

1. **Use destination directly as target** (when far away)
   - Ignore rectangle intersection when destination is outside current subcell area
   - Only use rectangle boundaries for final approach (within 1-2 cells)

2. **Compute target on boundary that aligns with destination**
   - Instead of ray intersection, find point on rectangle boundary closest to destination ray
   - Ensures target is always progressing toward destination

3. **Switch algorithms based on distance**
   - Far from dest (>3 cells): Use destination directly
   - Near dest (<3 cells): Use rectangle intersection for smooth arrival

4. **Rethink "direct" mode**
   - Maybe "direct" should mean "ignore subcell rectangles entirely"
   - Move straight toward destination, only respecting collision avoidance

---

## Appendix: Log Data

### Destination Settings
- 1699ms: Cell (9, 25) → screen (277.5, 505.0)
- 2743ms: Cell (15, 23) → screen (457.5, 465.0) ← **Active during diagonal movement**

### Sample ActorDirecting Events
```
2743ms: affinity=H, target=(292.5, 373.18), reserved=(10,19,0,0), anchor=(10,18,0,1)
2772ms: affinity=H, target=(307.5, 381.53), reserved=(10,19,1,1), anchor=(10,19,1,0)
3147ms: affinity=H, target=(322.5, 389.88), reserved=(11,20,0,0), anchor=(11,19,0,1)
5625ms: affinity=H, target=(427.5, 448.31), reserved=(14,23,1,1), anchor=(14,23,1,0)
```

All show targets that deviate significantly from destination direction.

