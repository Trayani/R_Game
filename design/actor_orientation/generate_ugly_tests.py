#!/usr/bin/env python3
"""
Generate ugly position test cases for actor directing tests.

Creates test cases with non-aligned actor positions (5.9, 5.3, 5.1, 4.7, etc.)
to ensure the algorithm handles imprecise coordinates correctly.

Generates 24 test cases: 12 bases (T021-T036) × 2 ugly positions (U1, U2)
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

    # Step 4: Determine affinity and target based on which edge hits first
    if abs(t_vertical - t_horizontal) < EPSILON:
        # BOTH: Hits corner (both edges at same t)
        affinity = "BOTH"
        target_x = actor_x + ray_x * t_vertical
        target_y = actor_y + ray_y * t_vertical
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        anchor_x = diag_x if offset_x > offset_y else psc_x
        anchor_y = psc_y if offset_x > offset_y else diag_y
    elif t_vertical < t_horizontal:
        # V: Hits vertical edge first
        affinity = "V"
        target_x = actor_x + ray_x * t_vertical
        target_y = actor_y + ray_y * t_vertical
        anchor_x = diag_x
        anchor_y = psc_y
    else:
        # H: Hits horizontal edge first
        affinity = "H"
        target_x = actor_x + ray_x * t_horizontal
        target_y = actor_y + ray_y * t_horizontal
        anchor_x = psc_x
        anchor_y = diag_y

    return (affinity, target_x, target_y, anchor_x, anchor_y, t_vertical, t_horizontal)


def flip_affinity(affinity_str):
    """Flip affinity for opposite anchor."""
    if affinity_str == "H":
        return "V"
    elif affinity_str == "V":
        return "H"
    else:
        return "BOTH"


def load_baseline_tests():
    """Load baseline test data (P1 positions) from TSV."""
    baseline_tests = {}

    with open("design/actor_orientation/actor_directing_position_tests.tsv", "r") as f:
        reader = csv.DictReader(f, delimiter='\t')
        for row in reader:
            test_id = row['test_id']
            if test_id.endswith('_P1'):  # Only baseline positions
                base = test_id.split('_')[0]  # e.g., T021
                baseline_tests[base] = {
                    'base': row['base'],
                    'psc_x': int(row['PSC_x']),
                    'psc_y': int(row['PSC_y']),
                    'diag_x': int(row['diag_x']),
                    'diag_y': int(row['diag_y']),
                    'dest_x': float(row['dest_x']),
                    'dest_y': float(row['dest_y']),
                    'optimal_dir': row['optimal_dir']
                }

    return baseline_tests


def generate_ugly_positions():
    """
    Generate ugly actor positions for U1 (near-boundary) and U2 (mid-range).

    Positions are within the rectangle between PSC(5,5) and each diagonal.
    U1: Near PSC or diagonal boundaries (within 0.1-0.2)
    U2: Mid-range positions (0.3-0.7 between intersections)

    Returns dict mapping base to (u1_name, u1_x, u1_y, u2_name, u2_x, u2_y)
    """
    # All positions are within rectangle bounds between PSC (5,5) and diagonal
    ugly_positions = {
        # NE direction: rect [5-6] x [4-5]
        'T021': ('Ugly_Near_PSC', 5.1, 4.9, 'Ugly_Mid_Range', 5.3, 4.7),
        'T022': ('Ugly_Near_Diag', 5.9, 4.15, 'Ugly_Offset', 5.6, 4.3),
        'T024': ('Ugly_Asymmetric', 5.15, 4.85, 'Ugly_Far', 5.7, 4.3),
        'T025': ('Ugly_NE_Offset', 5.2, 4.8, 'Ugly_NE_Mid', 5.6, 4.4),

        # E direction: rect [5-6] x [4-5]
        'T026': ('Ugly_East_Near', 5.9, 4.5, 'Ugly_East_Mid', 5.3, 4.7),

        # SE direction: rect [5-6] x [5-6]
        'T027': ('Ugly_SE_Near', 5.1, 5.9, 'Ugly_SE_Mid', 5.3, 5.7),
        'T029': ('Ugly_SE_Alt', 5.9, 5.2, 'Ugly_SE_Center', 5.7, 5.3),

        # SW direction: rect [4-5] x [5-6]
        'T031': ('Ugly_SW_Near', 4.1, 5.1, 'Ugly_SW_Mid', 4.3, 5.6),
        'T032': ('Ugly_SW_Alt', 4.15, 5.9, 'Ugly_SW_Center', 4.6, 5.3),

        # NW direction: rect [4-5] x [4-5]
        'T034': ('Ugly_NW_Near', 4.9, 4.1, 'Ugly_NW_Mid', 4.7, 4.3),
        'T035': ('Ugly_NW_Alt', 4.85, 4.2, 'Ugly_NW_Center', 4.4, 4.6),
        'T036': ('Ugly_NW_Corner', 4.2, 4.15, 'Ugly_NW_Far', 4.3, 4.3),
    }

    return ugly_positions


def generate_test_case(base, baseline, position_name, actor_x, actor_y, position_id):
    """Generate a complete test case row."""
    psc_x = baseline['psc_x']
    psc_y = baseline['psc_y']
    diag_x = baseline['diag_x']
    diag_y = baseline['diag_y']
    dest_x = baseline['dest_x']
    dest_y = baseline['dest_y']

    # Calculate optimal direction
    affinity, target_x, target_y, anchor_x, anchor_y, t_v, t_h = \
        calculate_affinity_and_target(actor_x, actor_y, psc_x, psc_y, diag_x, diag_y, dest_x, dest_y)

    # Determine optimal direction name
    base_dir = baseline['optimal_dir'].split('-')[0]  # NE, SE, etc.
    if affinity == "BOTH":
        optimal_dir = base_dir
    else:
        optimal_dir = f"{base_dir}-{affinity}"

    # Calculate alternative direction (opposite affinity)
    if affinity == "BOTH":
        # For BOTH, use position offset to determine which anchor to block
        offset_x = abs(actor_x - psc_x)
        offset_y = abs(actor_y - psc_y)
        # Block the anchor that was chosen first
        if offset_x > offset_y:
            # H anchor was chosen, so alternative is V
            alt_affinity = "V"
        else:
            # V anchor was chosen, so alternative is H
            alt_affinity = "H"
    else:
        alt_affinity = flip_affinity(affinity)

    alt_dir = f"{base_dir}-{alt_affinity}"

    # Calculate alternative target with opposite affinity
    # Recalculate with same geometry - the target will be the same,
    # just the affinity/anchor differs
    alt_target_x = target_x
    alt_target_y = target_y

    # Calculate alternative anchor
    if alt_affinity == "H":
        alt_anchor_x = psc_x
        alt_anchor_y = diag_y
    elif alt_affinity == "V":
        alt_anchor_x = diag_x
        alt_anchor_y = psc_y
    else:  # BOTH
        # Use opposite anchor from optimal
        if affinity == "H":
            alt_anchor_x = diag_x
            alt_anchor_y = psc_y
        else:
            alt_anchor_x = psc_x
            alt_anchor_y = diag_y

    # Format test row
    test_id = f"{base}_{position_id}"
    notes = f"Ugly position test"

    row = [
        test_id,
        baseline['base'],
        position_name,
        f"{actor_x:.1f}",
        f"{actor_y:.1f}",
        str(psc_x),
        str(psc_y),
        str(diag_x),
        str(diag_y),
        f"{dest_x:.1f}",
        f"{dest_y:.1f}",
        f"{target_x:.2f}",
        f"{target_y:.2f}",
        affinity,
        str(anchor_x),
        str(anchor_y),
        optimal_dir,
        notes,
        alt_dir,
        f"{alt_target_x:.2f}",
        f"{alt_target_y:.2f}",
        alt_affinity,
        str(alt_anchor_x),
        str(alt_anchor_y)
    ]

    return '\t'.join(row)


def main():
    """Generate ugly position test cases."""
    baseline_tests = load_baseline_tests()
    ugly_positions = generate_ugly_positions()

    print("# Generated ugly position test cases", file=sys.stderr)
    print(f"# Total cases: {len(ugly_positions) * 2}", file=sys.stderr)

    # Generate test cases
    for base in sorted(baseline_tests.keys()):
        if base not in ugly_positions:
            print(f"# Warning: No ugly positions defined for {base}", file=sys.stderr)
            continue

        baseline = baseline_tests[base]
        u1_name, u1_x, u1_y, u2_name, u2_x, u2_y = ugly_positions[base]

        # Generate U1 test case
        u1_row = generate_test_case(base, baseline, u1_name, u1_x, u1_y, 'U1')
        print(u1_row)

        # Generate U2 test case
        u2_row = generate_test_case(base, baseline, u2_name, u2_x, u2_y, 'U2')
        print(u2_row)

    print(f"# Successfully generated {len(ugly_positions) * 2} test cases", file=sys.stderr)


if __name__ == '__main__':
    main()
