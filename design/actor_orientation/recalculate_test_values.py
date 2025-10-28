#!/usr/bin/env python3
"""
Recalculate correct expected values for actor directing position tests.
Uses the algorithm from actor_directing_v2.txt to generate accurate test data.
"""

import math

def calculate_affinity_and_target(actor_x, actor_y, psc_x, psc_y, diag_x, diag_y, dest_x, dest_y):
    """
    Calculate affinity and target position using ray-rectangle intersection.
    Implements actor_directing_v2.txt Section B.
    """
    EPSILON = 1e-6

    # Step 1: Define rectangle bounds
    rect_min_x = min(psc_x, diag_x)
    rect_max_x = max(psc_x, diag_x)
    rect_min_y = min(psc_y, diag_y)
    rect_max_y = max(psc_y, diag_y)

    # Step 2: Calculate ray direction
    dir_x = dest_x - actor_x
    dir_y = dest_y - actor_y
    dir_len = math.sqrt(dir_x**2 + dir_y**2)

    if dir_len < EPSILON:
        # Actor at destination
        return "BOTH", actor_x, actor_y

    ray_x = dir_x / dir_len
    ray_y = dir_y / dir_len

    # Step 3: Ray-rectangle intersection
    t_vertical = float('inf')
    t_horizontal = float('inf')

    # Vertical edges
    if abs(ray_x) > EPSILON:
        if ray_x > 0:
            t_vertical = (rect_max_x - actor_x) / ray_x
        else:
            t_vertical = (rect_min_x - actor_x) / ray_x

    # Horizontal edges
    if abs(ray_y) > EPSILON:
        if ray_y > 0:
            t_horizontal = (rect_max_y - actor_y) / ray_y
        else:
            t_horizontal = (rect_min_y - actor_y) / ray_y

    # Handle edge cases
    if t_vertical <= EPSILON and t_horizontal <= EPSILON:
        # Actor at corner or outside
        return "BOTH", actor_x, actor_y
    elif t_vertical <= EPSILON:
        t_vertical = float('inf')
    elif t_horizontal <= EPSILON:
        t_horizontal = float('inf')

    # Step 4: Determine affinity
    if abs(t_vertical - t_horizontal) < EPSILON:
        affinity = "BOTH"
        t_min = t_vertical
    elif t_vertical < t_horizontal:
        affinity = "H"
        t_min = t_vertical
    else:
        affinity = "V"
        t_min = t_horizontal

    # Calculate target
    target_x = actor_x + ray_x * t_min
    target_y = actor_y + ray_y * t_min

    # Clamp to rectangle bounds
    target_x = max(rect_min_x, min(rect_max_x, target_x))
    target_y = max(rect_min_y, min(rect_max_y, target_y))

    return affinity, target_x, target_y

def get_anchor(psc_x, psc_y, diag_x, diag_y, affinity, actor_x, actor_y):
    """Get anchor subcell based on affinity."""
    if affinity == "H":
        # Horizontal anchor: shares Y with PSC, X with diagonal
        return diag_x, psc_y
    elif affinity == "V":
        # Vertical anchor: shares X with PSC, Y with diagonal
        return psc_x, diag_y
    else:  # BOTH
        # Choose based on actor position offset
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        if offset_x > offset_y:
            return diag_x, psc_y
        else:
            return psc_x, diag_y

# Test cases from TSV
test_cases = [
    # T021_P2
    {"test_id": "T021_P2", "actor_x": 5.5, "actor_y": 4.5,
     "psc": (5, 5), "diag": (6, 4), "dest": (6, 1)},
    # Add more as needed
]

print("# Recalculated Test Values\n")
print("test_id\tactortest_x\tactor_y\ttarget_x\ttarget_y\taffinity\tanchor_x\tanchor_y")

for test in test_cases:
    affinity, target_x, target_y = calculate_affinity_and_target(
        test["actor_x"], test["actor_y"],
        test["psc"][0], test["psc"][1],
        test["diag"][0], test["diag"][1],
        test["dest"][0], test["dest"][1]
    )

    anchor_x, anchor_y = get_anchor(
        test["psc"][0], test["psc"][1],
        test["diag"][0], test["diag"][1],
        affinity,
        test["actor_x"], test["actor_y"]
    )

    print(f"{test['test_id']}\t{test['actor_x']:.2f}\t{test['actor_y']:.2f}\t"
          f"{target_x:.2f}\t{target_y:.2f}\t{affinity}\t{anchor_x}\t{anchor_y}")

# T021_P2 detailed calculation
print("\n# Detailed T021_P2 Calculation")
print("Actor: (5.5, 4.5)")
print("PSC: (5, 5), Diagonal: (6, 4), Dest: (6, 1)")
print()
affinity, tx, ty = calculate_affinity_and_target(5.5, 4.5, 5, 5, 6, 4, 6, 1)
print(f"Calculated: target=({tx:.4f}, {ty:.4f}), affinity={affinity}")
print(f"TSV expected: target=(5.54, 4.00), affinity=V")
print(f"Difference: target_x = {abs(tx - 5.54):.4f}")
