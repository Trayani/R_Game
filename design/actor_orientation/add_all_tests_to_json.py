#!/usr/bin/env python3
"""
Add all remaining tests from TSV to JSON visualization data with verification.
"""

import json
import math
import sys

def calculate_target(actor_x, actor_y, psc_x, psc_y, diag_x, diag_y, dest_x, dest_y):
    """Calculate correct target position using ray-rectangle intersection."""

    # Rectangle bounds (at grid intersections)
    rect_min_x = min(psc_x, diag_x)
    rect_max_x = max(psc_x, diag_x)
    rect_min_y = min(psc_y, diag_y)
    rect_max_y = max(psc_y, diag_y)

    # Ray direction
    dir_x = dest_x - actor_x
    dir_y = dest_y - actor_y
    length = math.sqrt(dir_x**2 + dir_y**2)

    if length < 1e-9:
        return actor_x, actor_y, "BOTH"

    ray_x = dir_x / length
    ray_y = dir_y / length

    # Calculate t-values for rectangle edges
    INF = float('inf')

    # Vertical edges
    if ray_x > 1e-9:
        t_right = (rect_max_x - actor_x) / ray_x
        t_left = INF
    elif ray_x < -1e-9:
        t_left = (rect_min_x - actor_x) / ray_x
        t_right = INF
    else:
        t_left = t_right = INF

    # Horizontal edges
    if ray_y > 1e-9:
        t_bottom = (rect_max_y - actor_y) / ray_y
        t_top = INF
    elif ray_y < -1e-9:
        t_top = (rect_min_y - actor_y) / ray_y
        t_bottom = INF
    else:
        t_top = t_bottom = INF

    t_vertical = min(t_left, t_right)
    t_horizontal = min(t_top, t_bottom)

    # Determine affinity and target
    epsilon = 1e-6

    if abs(t_vertical - t_horizontal) < epsilon:
        # Corner hit
        t = t_vertical
        affinity = "BOTH"
    elif t_vertical < t_horizontal:
        # Hits vertical edge first -> H-affinity
        t = t_vertical
        affinity = "H"
    else:
        # Hits horizontal edge first -> V-affinity
        t = t_horizontal
        affinity = "V"

    # Calculate target
    target_x = actor_x + ray_x * t
    target_y = actor_y + ray_y * t

    # Round to 2 decimal places
    target_x = round(target_x, 2)
    target_y = round(target_y, 2)

    return target_x, target_y, affinity


def calculate_anchor(psc_x, psc_y, diag_x, diag_y, affinity):
    """Calculate correct anchor based on affinity and diagonal direction."""

    if affinity == "H":
        # Horizontal-favoring: anchor is horizontal neighbor of PSC toward diagonal
        if diag_x > psc_x:
            return [psc_x + 1, psc_y]
        else:
            return [psc_x - 1, psc_y]
    elif affinity == "V":
        # Vertical-favoring: anchor is vertical neighbor of PSC toward diagonal
        if diag_y > psc_y:
            return [psc_x, psc_y + 1]
        else:
            return [psc_x, psc_y - 1]
    else:
        # BOTH affinity: return PSC (can use either)
        return [psc_x, psc_y]


def load_tsv(tsv_path):
    """Load tests from TSV file."""
    tests = []
    with open(tsv_path, 'r') as f:
        lines = f.readlines()
        for line in lines[1:]:  # Skip header
            parts = line.strip().split('\t')
            if len(parts) < 17:
                continue

            test = {
                'id': parts[0],
                'base': parts[1],
                'position': parts[2],
                'actor': [float(parts[3]), float(parts[4])],
                'psc': [int(parts[5]), int(parts[6])],
                'diagonal': [int(parts[7]), int(parts[8])],
                'destination': [float(parts[9]), float(parts[10])],
                'target_tsv': [float(parts[11]), float(parts[12])],
                'affinity': parts[13],
                'anchor_raw': parts[14],
                'optimal_dir': parts[16],
                'notes': parts[17] if len(parts) > 17 else ""
            }

            # Parse anchor (handle "5-4" format)
            if '-' in test['anchor_raw']:
                anchor_parts = test['anchor_raw'].split('-')
                test['anchor'] = [int(anchor_parts[0]), int(anchor_parts[1])]
            else:
                anchor_parts = test['anchor_raw'].split()
                if len(anchor_parts) == 2:
                    test['anchor'] = [int(anchor_parts[0]), int(anchor_parts[1])]
                else:
                    test['anchor'] = test['psc']  # fallback

            tests.append(test)

    return tests


def verify_and_create_json_entry(test):
    """Verify test calculation and create JSON entry."""

    # Calculate correct target
    calc_target_x, calc_target_y, calc_affinity = calculate_target(
        test['actor'][0], test['actor'][1],
        test['psc'][0], test['psc'][1],
        test['diagonal'][0], test['diagonal'][1],
        test['destination'][0], test['destination'][1]
    )

    # Calculate correct anchor
    calc_anchor = calculate_anchor(
        test['psc'][0], test['psc'][1],
        test['diagonal'][0], test['diagonal'][1],
        calc_affinity
    )

    # Check if TSV data matches calculation
    tsv_target_x = test['target_tsv'][0]
    tsv_target_y = test['target_tsv'][1]

    diff_x = abs(calc_target_x - tsv_target_x)
    diff_y = abs(calc_target_y - tsv_target_y)

    corrected = False
    anchor_corrected = False

    # Check anchor correctness
    if test['anchor'] != calc_anchor:
        anchor_corrected = True
        print(f"  📍 {test['id']}: Anchor corrected from {test['anchor']} to {calc_anchor}")

    if diff_x > 0.01 or diff_y > 0.01:
        corrected = True
        notes = f"CORRECTED: was ({tsv_target_x}, {tsv_target_y}), now ({calc_target_x}, {calc_target_y})"
        print(f"  ⚠️  {test['id']}: {notes}")
    else:
        notes = test['notes']

    # Create JSON entry
    json_entry = {
        "id": test['id'],
        "base": test['base'],
        "position": test['position'],
        "actor": test['actor'],
        "psc": test['psc'],
        "diagonal": test['diagonal'],
        "destination": test['destination'],
        "target": [calc_target_x, calc_target_y],
        "affinity": calc_affinity,
        "anchor": calc_anchor,
        "optimal_dir": test['optimal_dir'],
        "notes": notes
    }

    return json_entry, (corrected or anchor_corrected)


def main():
    tsv_path = "/home/jan/hobby/bcp/RustGame3/design/actor_orientation/actor_directing_position_tests.tsv"
    json_path = "/home/jan/hobby/bcp/RustGame3/design/actor_orientation/test_visualization_data.json"

    print("Loading TSV data...")
    tsv_tests = load_tsv(tsv_path)
    print(f"Found {len(tsv_tests)} tests in TSV")

    print("\nLoading existing JSON for metadata...")
    with open(json_path, 'r') as f:
        json_data = json.load(f)

    # Process ALL tests (regenerate everything)
    print("\nVerifying and regenerating all tests...")
    all_tests = []
    corrected_count = 0

    for test in tsv_tests:
        json_entry, corrected = verify_and_create_json_entry(test)
        all_tests.append(json_entry)
        if corrected:
            corrected_count += 1

    print(f"\n✅ Processed {len(all_tests)} tests")
    print(f"   {corrected_count} corrections made")
    print(f"   {len(all_tests) - corrected_count} verified correct")

    # Replace all tests in JSON
    json_data['tests'] = all_tests

    # Sort by ID for readability
    json_data['tests'].sort(key=lambda t: t['id'])

    # Write updated JSON
    print(f"\nWriting updated JSON with {len(json_data['tests'])} total tests...")
    with open(json_path, 'w') as f:
        json.dump(json_data, f, indent=2)

    print("✅ Done!")
    print(f"\nTotal tests in visualization: {len(json_data['tests'])}")


if __name__ == "__main__":
    main()
