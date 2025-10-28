#!/usr/bin/env python3
"""
Calculate alternative directions for actor directing position tests.

When the optimal direction is blocked, the algorithm falls back to alternatives:
1. For diagonal with affinity: Try opposite affinity (e.g., NE-H → NE-V)
2. For cardinal: Try adjacent diagonals

This script calculates the expected alternative direction, target position,
affinity, and anchor for each test case.
"""

import math
import csv
import sys

def calculate_affinity_and_target(actor_x, actor_y, psc_x, psc_y, diag_x, diag_y, dest_x, dest_y):
    """
    Calculate affinity and target position using ray-rectangle intersection.
    Implements actor_directing_v2.txt Section B.

    Returns: (affinity, target_x, target_y, anchor_x, anchor_y, t_v, t_h)
    """
    EPSILON = 1e-6

    # Step 1: Define rectangle bounds (grid intersections)
    rect_min_x = min(psc_x, diag_x)
    rect_max_x = max(psc_x, diag_x)
    rect_min_y = min(psc_y, diag_y)
    rect_max_y = max(psc_y, diag_y)

    # Step 2: Calculate ray direction
    dir_x = dest_x - actor_x
    dir_y = dest_y - actor_y
    dir_len = math.sqrt(dir_x**2 + dir_y**2)

    if dir_len < EPSILON:
        # Actor at destination - use position offset for anchor
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        anchor_x = diag_x if offset_x > offset_y else psc_x
        anchor_y = psc_y if offset_x > offset_y else diag_y
        return ("BOTH", actor_x, actor_y, anchor_x, anchor_y, 0.0, 0.0)

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
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        anchor_x = diag_x if offset_x > offset_y else psc_x
        anchor_y = psc_y if offset_x > offset_y else diag_y
        target_x = max(rect_min_x, min(rect_max_x, actor_x))
        target_y = max(rect_min_y, min(rect_max_y, actor_y))
        return ("BOTH", target_x, target_y, anchor_x, anchor_y, 0.0, 0.0)

    # Handle both t infinite (ray perpendicular to boundary)
    if math.isinf(t_vertical) and math.isinf(t_horizontal):
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        anchor_x = diag_x if offset_x > offset_y else psc_x
        anchor_y = psc_y if offset_x > offset_y else diag_y
        return ("BOTH", actor_x, actor_y, anchor_x, anchor_y, t_vertical, t_horizontal)

    # Step 4: Determine affinity
    if abs(t_vertical - t_horizontal) < EPSILON:
        affinity = "BOTH"
        t_min = t_vertical
        # Choose anchor based on position offset
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        anchor_x = diag_x if offset_x > offset_y else psc_x
        anchor_y = psc_y if offset_x > offset_y else diag_y
    elif t_vertical < t_horizontal:
        affinity = "H"
        t_min = t_vertical
        anchor_x = diag_x  # H-affinity: shares Y with PSC, X with diagonal
        anchor_y = psc_y
    else:
        affinity = "V"
        t_min = t_horizontal
        anchor_x = psc_x  # V-affinity: shares X with PSC, Y with diagonal
        anchor_y = diag_y

    # Calculate target
    target_x = actor_x + ray_x * t_min
    target_y = actor_y + ray_y * t_min

    # Clamp to rectangle bounds
    target_x = max(rect_min_x, min(rect_max_x, target_x))
    target_y = max(rect_min_y, min(rect_max_y, target_y))

    return (affinity, target_x, target_y, anchor_x, anchor_y, t_vertical, t_horizontal)


def parse_direction(dir_str):
    """Parse direction string like 'NE-H' into components."""
    if not dir_str or dir_str.strip() == "":
        return None, None

    parts = dir_str.strip().split('-')
    base_dir = parts[0]  # NE, SE, SW, NW
    affinity = parts[1] if len(parts) > 1 else "BOTH"
    return base_dir, affinity


def get_diagonal_coords(psc_x, psc_y, direction):
    """Get diagonal subcell coordinates from PSC given direction."""
    offsets = {
        'NE': (1, -1),
        'SE': (1, 1),
        'SW': (-1, 1),
        'NW': (-1, -1),
    }

    if direction not in offsets:
        return None, None

    dx, dy = offsets[direction]
    return psc_x + dx, psc_y + dy


def flip_affinity(affinity):
    """Flip H ↔ V affinity."""
    if affinity == "H":
        return "V"
    elif affinity == "V":
        return "H"
    else:
        return "BOTH"


def calculate_alternative(test_row):
    """
    Calculate alternative direction for a test case.

    Returns: (alt1_dir, alt1_target_x, alt1_target_y, alt1_affinity, alt1_anchor_x, alt1_anchor_y)
    """
    # Parse test data
    actor_x = float(test_row['actor_x'])
    actor_y = float(test_row['actor_y'])
    psc_x = int(test_row['PSC_x'])
    psc_y = int(test_row['PSC_y'])
    dest_x = float(test_row['dest_x'])
    dest_y = float(test_row['dest_y'])
    optimal_dir = test_row['optimal_dir']

    # Parse optimal direction
    base_dir, optimal_affinity = parse_direction(optimal_dir)
    if base_dir is None:
        return ("", "", "", "", "", "")

    # Get diagonal coordinates
    diag_x, diag_y = get_diagonal_coords(psc_x, psc_y, base_dir)
    if diag_x is None:
        return ("", "", "", "", "", "")

    # Calculate alternative: Try opposite affinity for same diagonal
    if optimal_affinity in ["H", "V"]:
        alt1_affinity_str = flip_affinity(optimal_affinity)
        alt1_dir = f"{base_dir}-{alt1_affinity_str}"

        # Recalculate affinity and target for the same diagonal
        # (The algorithm will pick opposite anchor)
        affinity, target_x, target_y, anchor_x, anchor_y, t_v, t_h = calculate_affinity_and_target(
            actor_x, actor_y, psc_x, psc_y, diag_x, diag_y, dest_x, dest_y
        )

        # Note: We force the opposite affinity, so anchor is calculated for opposite
        if alt1_affinity_str == "H":
            anchor_x = diag_x
            anchor_y = psc_y
        else:  # V
            anchor_x = psc_x
            anchor_y = diag_y

        return (
            alt1_dir,
            f"{target_x:.2f}",
            f"{target_y:.2f}",
            alt1_affinity_str,
            str(anchor_x),
            str(anchor_y)
        )

    elif optimal_affinity == "BOTH":
        # For BOTH affinity, alternative depends on which anchor was blocked
        # Try opposite anchor choice
        affinity, target_x, target_y, anchor_x, anchor_y, t_v, t_h = calculate_affinity_and_target(
            actor_x, actor_y, psc_x, psc_y, diag_x, diag_y, dest_x, dest_y
        )

        # Alternative is same direction but different anchor choice
        # (Implementation dependent - could be H or V based on position offset)
        alt1_dir = base_dir  # Keep as BOTH
        alt1_affinity_str = "BOTH"

        return (
            alt1_dir,
            f"{target_x:.2f}",
            f"{target_y:.2f}",
            alt1_affinity_str,
            str(anchor_x),
            str(anchor_y)
        )

    return ("", "", "", "", "", "")


def main():
    input_file = "design/actor_orientation/actor_directing_position_tests.tsv"
    output_file = "design/actor_orientation/actor_directing_position_tests_with_alt.tsv"

    print(f"Reading {input_file}...")

    with open(input_file, 'r') as f:
        reader = csv.DictReader(f, delimiter='\t')
        fieldnames = reader.fieldnames
        rows = list(reader)

    print(f"Loaded {len(rows)} test cases")

    # Add new fields for alternatives
    new_fieldnames = list(fieldnames) + ['alt1_dir', 'alt1_target_x', 'alt1_target_y',
                                          'alt1_affinity', 'alt1_anchor_x', 'alt1_anchor_y']

    # Calculate alternatives for each test
    print("\nCalculating alternatives...")
    for i, row in enumerate(rows):
        alt_data = calculate_alternative(row)
        row['alt1_dir'] = alt_data[0]
        row['alt1_target_x'] = alt_data[1]
        row['alt1_target_y'] = alt_data[2]
        row['alt1_affinity'] = alt_data[3]
        row['alt1_anchor_x'] = alt_data[4]
        row['alt1_anchor_y'] = alt_data[5]

        if (i + 1) % 20 == 0:
            print(f"  Processed {i + 1}/{len(rows)} tests...")

    print(f"\nWriting {output_file}...")
    with open(output_file, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=new_fieldnames, delimiter='\t')
        writer.writeheader()
        writer.writerows(rows)

    print(f"✓ Done! Created {output_file}")
    print(f"  Added {len([r for r in rows if r['alt1_dir']])} alternatives")

    # Print sample
    print("\nSample alternatives:")
    for i, row in enumerate(rows[:5]):
        if row['alt1_dir']:
            print(f"  {row['test_id']}: optimal={row['optimal_dir']} → alt1={row['alt1_dir']}")


if __name__ == "__main__":
    main()
