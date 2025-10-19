# Test messy_010 Detailed Analysis

## Test Information

**C# Source**: ProtoTests.cs:243
```csharp
test2(6637, 6552, 1, false, 6636, 6637);
```

**Parameters**:
- Start: 6637 at (80, 79)
- Dest: 6552 at (78, 78)
- Size: 1
- Reverse: false (not used)
- messyX: false
- messyY: true
- Expected waypoints: [6636, 6637]

**Full expected path**: [6637, 6636, 6637, 6552]

## Grid Layout Visualization

```
Grid: 104 cols × 83 rows
ID = x + y * cols

Position 6637: x=80, y=79 → ID = 80 + 79*104 = 6637
Position 6636: x=79, y=79 → ID = 79 + 79*104 = 6636
Position 6639: x=82, y=79 → ID = 82 + 79*104 = 6639
Position 6552: x=78, y=78 → ID = 78 + 78*104 = 6552
```

### Visual Layout
```
Row 78:   ... 6552 . . . . ...
              (78)
              [DEST]

Row 79:   ... 6636 6637 . 6639 ...
              (79) (80)   (82)
              [WP1][START][ALT]

Row 80:   ... . 6721 . ...
                (81)
```

### Messy Position Explanation

**Start 6637 with messyY=true**:
- Entity is at position (80, 79.5) - BETWEEN rows 79 and 80
- Entity spans cells at Y=79 and Y=80 (occupies partial space in both rows)
- Not aligned to grid - this is the "messy" state

**Clean position 6637**:
- Same cell ID, but entity fully aligned at (80, 79) - ON row 79
- No longer spanning two rows
- This is the "clean" state

## Path Analysis

### C# Expected Path: [6637, 6636, 6637, 6552]

**Step-by-step movement**:
1. **6637 (messy)** → Start at (80, 79.5) - entity spans rows 79-80
2. **6636** → Move to (79, 79) - still messy? Or first waypoint?
3. **6637 (clean)** → Return to (80, 79.0) - NOW aligned to grid
4. **6552** → Move to destination (78, 78)

**Why does start appear twice?**
- First 6637: Messy starting position (80, 79.5)
- Second 6637: Alignment waypoint (80, 79.0) - clean position
- Same cell ID, different semantic meaning!

### Rust Path: [6637, 6636, 6639, 6552]

**Step-by-step movement**:
1. **6637** → Start at (80, 79.5)
2. **6636** → Move to (79, 79)
3. **6639** → Move to (82, 79) ← WRONG! Should go back to 6637
4. **6552** → Move to destination (78, 78)

**Why does Rust choose 6639?**
- Rust A* finds a geometrically valid path through 6639
- But doesn't enforce the alignment principle
- Missing: The requirement to return to start position (clean version)

## C# Code Reference

### 1. Path Computation (PathFinder.cs:383-388)

```csharp
var r = pfxXX(start, dest, size, messyStartX, messyStartY);
if (appendStart)
    r.path?.Add(start);
if (appendEnd)
    r.path?.Insert(0, dest);
return r;
```

**Note**: `appendStart` is always false in visible code, so start is NOT appended here.

### 2. Corner Discovery (PathFinder.pfxXXXS.cs)

From C# trace output:
```
[pfxXX] corner has 3 corners
[pfxXX] First few corners: id=6637, dist=1.00, tag=Base
```

**Key insight**: Position 6637 sees ITSELF as a corner at distance 1.00!
- This is the alignment target
- The messy position can "see" the clean version of itself as a reachable corner
- Distance 1.0 suggests it's treated as a direct neighbor

### 3. A* Search (PathFinder.pfxXXXS.cs)

C# trace shows:
```
[pfxXXS] Starting main search loop from corner 6637
[pfxXXS] Loop 1: deq=6637, Priority=0.00, minDist=2.41
[pfxXXS]   Processing 4 corners from 6637
[pfxXXS]     Corner[0]: id=6636, dist=1.00, total_dist=1.00
[pfxXXS]       Found finished corner: finDist=1.41, total=2.41
```

**Path found**: 6637 → 6636 → (dest)
- Total distance: 2.41
- Waypoints returned: [6636]

But wait - the EXPECTED waypoints are [6636, 6637]! Where does the second 6637 come from?

### 4. Path Reconstruction Mystery

Looking at C# test result:
```
Waypoints: [6636, 6637]
Full path: [6637, 6636, 6637, 6552]
```

The waypoints list has BOTH 6636 and 6637. But the A* search only returned [6636].

**Hypothesis**: There must be post-processing that adds the alignment waypoint!

Let me check pfxXX method for corner iteration...

### 5. pfxXX Method (PathFinder.pfxXX.cs)

From C# trace:
```
[pfxXX] Testing corner 0: id=6637, dist=1.00
[pfxXX]   pfxXXS from corner 6637 returned: dist=2.41, path=1 nodes
[pfxXX]   Path: 6636
[pfxXX]   NEW BEST: total_dist=3.41, path_len=2
```

So pfxXX tests MULTIPLE corners from the start position:
1. **Corner 0 (6637)**: Distance 1.00 from start
   - Runs A* from 6637 to dest
   - Returns path: [6636]
   - Total distance: 1.00 (start→6637) + 2.41 (6637→dest) = 3.41

The corner 6637 IS being tested! And the path FROM that corner is [6636].

**Path composition**:
- Start → Corner 6637: implicit (distance 1.00)
- Corner 6637 → Dest: [6636] (waypoints from A*)
- Full waypoints: [6637, 6636]

But wait, the expected is [6636, 6637], which is REVERSED!

### 6. Waypoint Reversal

C# code often reverses waypoints. Let me check...

Looking at the test expectation:
```csharp
test2(6637, 6552, 1, false, 6636, 6637);
//    start  dest  size reverse  waypoints...
```

Expected waypoints: [6636, 6637]
Full path: [6637, 6636, 6637, 6552]

**Aha!** The full path is:
- Start: 6637
- Waypoint 1: 6636
- Waypoint 2: 6637
- Dest: 6552

So waypoints are NOT reversed! They're in travel order:
1. Start at 6637 (messy)
2. Go to 6636 (first corner)
3. Return to 6637 (alignment - clean version)
4. Go to 6552 (destination)

## The Missing Piece

The C# algorithm structure is:

```
FOR each corner reachable from messy start:
    path_to_corner = [corner]  (implicit first waypoint)
    path_from_corner = A*(corner → dest)
    full_path = path_to_corner + path_from_corner

    IF best_distance improved:
        best_path = full_path
```

For corner 6637 at distance 1.00:
- path_to_corner = [6637]
- path_from_corner = [6636] (from A* search)
- full_path = [6637, 6636]

But we need to reverse this to get the actual travel order!

**WAIT NO** - let me re-read the C# trace...

```
[pfxXX] Testing corner 0: id=6637, dist=1.00
```

This tests STARTING from corner 6637, not going TO corner 6637!

Let me reconsider...

## Corrected Understanding

Looking at ProtoTests.cs:242-243:
```csharp
test2(6552, 6720, 1, false, 6637, 6636);  // Line 242: REVERSE direction
test2(6637, 6552, 1, false, 6636, 6637);  // Line 243: Our test
```

Notice line 242 (reverse direction):
- Start 6552, Dest 6720
- Expected waypoints: [6637, 6636]

Line 243 (our test):
- Start 6637, Dest 6552
- Expected waypoints: [6636, 6637]

These are SYMMETRIC paths! The waypoint order is different.

**In test messy_010 (6637 → 6552)**:
- Path: [6637, 6636, 6637, 6552]
- First move: 6637 → 6636 (move left)
- Second move: 6636 → 6637 (move right - BACK to start!)
- Third move: 6637 → 6552 (move to dest)

This means:
1. From messy 6637, move to 6636
2. From 6636, return to clean 6637 (alignment waypoint)
3. From clean 6637, move to 6552

## Why Does Rust Choose 6639 Instead?

Rust finds path: [6637, 6636, 6639, 6552]

This suggests:
- From 6637, move to 6636 ✓ (same as C#)
- From 6636, move to 6639 ✗ (should be 6637)
- From 6639, move to 6552

**Geometric analysis**:
- 6636 is at (79, 79)
- 6637 is at (80, 79) - distance 1.0
- 6639 is at (82, 79) - distance 3.0
- 6552 is at (78, 78) - distance sqrt(2) = 1.41

From 6636:
- Distance to 6637: 1.0
- Distance to 6639: 3.0

So why would Rust choose 6639 over 6637?

**Hypothesis**: Rust's A* is computing total path cost, and the path through 6639 might have a better total cost due to heuristic or corner visibility.

**Likely cause**: Rust doesn't have 6637 in the "finished corners" list!

When we added 6637 as an alignment corner, it was added to the START interesting corners. But it may not be in the path from 6636 to dest.

## Root Cause

The issue is that C# treats the messy→clean transition specially:

**C# Structure**:
1. Compute corners from MESSY start (includes clean start as corner)
2. For each corner, run A* from that corner to dest
3. When testing corner = START (6637):
   - A* from 6637 to 6552
   - Finds path: [6636]
   - Prepends 6637 to waypoints: [6637, 6636]

**Rust Structure**:
1. Compute corners from start (now includes start itself)
2. Run SINGLE A* from start through all corners to dest
3. A* expands from start and finds best path through corners
4. Doesn't force going back to start position after first corner

## C# Algorithm - The ACTUAL Implementation

### pfxXX Method (PathFinder.pfxXX.cs:82-109)

```csharp
for (int i = 0; i < corner.corners.Length; ++i) {
    ref var cor = ref corner.corners[i];
    // cor.dist = distance from START to this corner

    if (cor.ci == null || cor.dist > minDist) {
        break;  // Early termination
    }

    // Run A* from THIS CORNER to dest
    var (d, p, cached) = pfxXXS(cor.ci, destCorner, size, minDist);
    // d = distance from corner to dest
    // p = waypoints from corner to dest

    if (p != null && (d + cor.dist < minDist)) {
        minDist = d + cor.dist;  // Total: start→corner + corner→dest
        path = p;
        p.Add(cor.id);  // APPEND THE CORNER TO THE PATH!
    }
}
```

**Key insight at line 107**: `p.Add(cor.id);`

The corner ID is APPENDED to the path returned from pfxXXS!

### Path Composition for test messy_010

**Iteration i=0: Testing corner 6637 (distance 1.00)**

1. `cor.id = 6637`, `cor.dist = 1.00` (distance from start to this corner)
2. Run `pfxXXS(6637_corner_info, destCorner, size, ∞)`
   - This runs A* FROM corner 6637 TO dest 6552
   - Returns: `d=2.41`, `p=[6636]` (waypoints from 6637 to 6552)
3. Total distance: `1.00 + 2.41 = 3.41`
4. **Line 107**: `p.Add(6637)` → path becomes `[6636, 6637]`
5. `minDist = 3.41`, `path = [6636, 6637]`

**Final result**: waypoints `[6636, 6637]`

**Full path construction**:
- Start: 6637
- Waypoints: [6636, 6637]
- Dest: 6552
- **Full path**: [6637, 6636, 6637, 6552] ✓

## The Magic of Line 107

```csharp
p.Add(cor.id);  // Append corner to path
```

This line adds the corner AFTER the waypoints from that corner to dest!

So for corner 6637:
- A* from 6637 to dest returns: [6636] (go to 6636 first)
- Then append 6637: [6636, 6637]

Reading the path left-to-right as waypoints:
1. First waypoint: 6636
2. Second waypoint: 6637

Travel order from start 6637:
1. Start at 6637 (messy)
2. Go to 6636 (first waypoint)
3. Go to 6637 (second waypoint - the alignment corner!)
4. Go to 6552 (dest)

**This is why 6637 appears twice!**

The corner is appended AFTER its own path, creating the alignment pattern:
- Start (messy) → intermediate corners → back to start (clean) → continue to dest

## Why This Works

When A* runs FROM corner 6637 (treated as clean) TO dest 6552:
1. It finds that going through 6636 is optimal
2. Returns path: [6636]
3. The corner 6637 is then appended: [6636, 6637]

This creates the travel order:
- From messy 6637, go to 6636 (sideways move)
- From 6636, return to clean 6637 (alignment!)
- From clean 6637, go to dest 6552

## Solution for Rust

Rust needs to implement the SAME corner iteration approach:

```rust
// Rust equivalent of C# pfxXX
let mut min_dist = f64::INFINITY;
let mut best_path = None;

for (i, corner) in interesting_corners.iter().enumerate() {
    let corner_pos = Position::new(corner.x, corner.y);
    let corner_dist = start.distance(&corner_pos);

    if corner_dist > min_dist {
        break;  // Early termination
    }

    // Run A* from THIS CORNER to dest (treating corner as clean)
    let path_from_corner = a_star_from_corner(corner_pos, dest, grid);

    if let Some((mut waypoints, dist)) = path_from_corner {
        let total_dist = corner_dist + dist;

        if total_dist < min_dist {
            min_dist = total_dist;
            // APPEND THE CORNER to waypoints (line 107 equivalent!)
            waypoints.push(corner_pos);
            best_path = Some(waypoints);
        }
    }
}

// Build full path: start + waypoints + dest
let mut full_path = vec![start];
if let Some(waypoints) = best_path {
    full_path.extend(waypoints);
}
full_path.push(dest);
```

**Critical**: The corner is APPENDED to the path from that corner, not prepended!
