use crate::grid::Grid;
use crate::ray::RayState;
use std::collections::HashSet;

/// Deferred cone state (matches C# PFContextNode)
#[derive(Clone, Debug)]
struct DeferredCone {
    ray_right: RayState,
    ray_left: RayState,
    curr_l_start_x: i32,
    curr_l_end_x: i32,
    curr_l_y: i32,
    prev_l_start_x: i32,
    prev_l_end_x: i32,
}

/// Main raycasting function
pub fn raycast(grid: &Grid, start_x: i32, start_y: i32) -> HashSet<i32> {
    let mut visible = HashSet::new();

    if grid.is_blocked(start_x, start_y) {
        return visible;
    }

    let mut lanes: Vec<Vec<(i32, i32)>> = vec![Vec::new(); grid.rows as usize];

    let (row_start_x, row_end_x) = find_walkable_bounds(grid, start_x, start_y);
    lanes[start_y as usize].push((row_start_x + 1, row_end_x + 1));

    scan_direction(grid, start_x, start_y, 1, row_start_x, row_end_x, &mut lanes);
    scan_direction(grid, start_x, start_y, -1, row_start_x, row_end_x, &mut lanes);

    // Debug output (disabled)
    // if (grid.rows == 12 && grid.cols == 12 && start_x == 3 && start_y == 3)
    //     || (grid.rows == 8 && grid.cols == 8 && start_x == 4 && start_y == 4) {
    //     println!("\n[RUST] lanes for start ({},{})",start_x, start_y);
    //     for (row, ranges) in lanes.iter().enumerate() {
    //         if !ranges.is_empty() {
    //             let range_str: Vec<String> = ranges.iter()
    //                 .map(|(s, e)| format!("({},{})", s, e))
    //                 .collect();
    //             println!("[RUST] Row {}: {} ranges: {}", row, ranges.len(), range_str.join(", "));
    //         }
    //     }
    // }

    for (row, ranges) in lanes.iter().enumerate() {
        for &(range_start, range_end) in ranges {
            for x in (range_start - 1)..=(range_end - 1) {
                if x >= 0 && x < grid.cols {
                    visible.insert(grid.get_id(x, row as i32));
                }
            }
        }
    }

    visible
}

fn find_walkable_bounds(grid: &Grid, x: i32, y: i32) -> (i32, i32) {
    let mut start_x = x;
    let mut end_x = x;

    while start_x > 0 && !grid.is_blocked(start_x - 1, y) {
        start_x -= 1;
    }

    while end_x < grid.cols - 1 && !grid.is_blocked(end_x + 1, y) {
        end_x += 1;
    }

    (start_x, end_x)
}

/// Scan in one direction - EXACT match to C# getBorders + stepNxt logic
fn scan_direction(
    grid: &Grid,
    start_x: i32,
    start_y: i32,
    dir: i32,
    row_start_x: i32,
    row_end_x: i32,
    lanes: &mut Vec<Vec<(i32, i32)>>,
) {
    // Debug disabled
    // let debug = grid.rows == 10 && grid.cols == 10 && start_x == 5 && start_y == 3;
    // if debug {
    //     println!("[SCAN_DIR] dir={}, start=({},{}), row_bounds=[{},{}]", dir, start_x, start_y, row_start_x, row_end_x);
    // }

    // C# getBorders lines 55-57: Find first segment in scan direction
    let next_y = start_y + dir;
    if next_y < 0 || next_y >= grid.rows {
        return;  // No rows in scan direction
    }

    let segments = find_all_segments_in_range(grid, next_y, row_start_x, row_end_x);
    if segments.is_empty() {
        return;  // No segments found
    }

    // C# getBorders LOOP logic (lines 222-240): Try each segment until we find one containing start_x
    let mut found_segment = None;
    for &(seg_start, seg_end) in &segments {
        // C# line 127: Check if start position is within this segment
        if start_x >= seg_start && start_x <= seg_end {
            found_segment = Some((seg_start, seg_end));
            break;
        }
    }

    let (first_seg_start, first_seg_end) = match found_segment {
        Some(seg) => seg,
        None => return,  // No segment contains start position, stop scanning
    };

    // Initialize first cone (C# getBorders lines 197-210)
    let ray_right = RayState::new(row_end_x - start_x, 1, -1, 0);
    let ray_left = RayState::new(start_x - row_start_x, 1, -1, 0);

    let initial_cone = DeferredCone {
        ray_right,
        ray_left,
        curr_l_start_x: first_seg_start,  // Use first segment bounds, not start row bounds!
        curr_l_end_x: first_seg_end,
        curr_l_y: next_y,
        prev_l_start_x: row_start_x,
        prev_l_end_x: row_end_x,
    };

    // C# getBorders line 218: pf.stepNxt(pfn, ...)
    let mut pfn = Vec::new(); // Deferred cones list
    process_cone(grid, start_x, start_y, dir, initial_cone, lanes, &mut pfn);

    // C# getBorders lines 244-248: Process deferred cones
    while let Some(deferred) = pfn.pop() {
        process_cone(grid, start_x, start_y, dir, deferred, lanes, &mut pfn);
    }
}

/// Process a single cone (matches C# PFContext.stepNxt)
fn process_cone(
    grid: &Grid,
    start_x: i32,
    start_y: i32,
    dir: i32,
    mut cone: DeferredCone,
    lanes: &mut Vec<Vec<(i32, i32)>>,
    pfn: &mut Vec<DeferredCone>,
) {
    // C# do-while loop (line 66)
    loop {
        // === C# lines 68-69: right(dir, currLR) || left(dir) ===

        // right() method
        cone.ray_right.increment_y_step();
        let calc_border_r = start_x + cone.ray_right.calculate_border();
        let mut border_x_r = calc_border_r.min(cone.prev_l_end_x);

        if border_x_r >= cone.curr_l_end_x {
            cone.ray_right.diff_x = cone.curr_l_end_x - start_x;
            cone.ray_right.diff_y = dir * (cone.curr_l_y - start_y);

            if cone.ray_right.diff_x >= 0 {
                cone.ray_right.diff_y += 1;
                cone.ray_right.y_step = -1;
                cone.ray_right.rounding = 0;
                border_x_r = cone.curr_l_end_x;
            } else {
                cone.ray_right.y_step = 1;
                cone.ray_right.diff_y -= 1;
                cone.ray_right.rounding = cone.ray_right.diff_y - 1;
                border_x_r = start_x + cone.ray_right.calculate_border();
            }
        } else if border_x_r < cone.curr_l_start_x {
            break;
        }

        // left() method
        cone.ray_left.increment_y_step();
        let mut border_x_l = start_x - cone.ray_left.calculate_border();

        if border_x_l <= cone.curr_l_start_x {
            cone.ray_left.diff_x = start_x - cone.curr_l_start_x;
            cone.ray_left.diff_y = dir * (cone.curr_l_y - start_y);

            if cone.ray_left.diff_x >= 0 {
                cone.ray_left.y_step = -1;
                cone.ray_left.diff_y += 1;
                cone.ray_left.rounding = 0;
                border_x_l = cone.curr_l_start_x;
            } else {
                cone.ray_left.y_step = 1;
                cone.ray_left.diff_y -= 1;
                cone.ray_left.rounding = cone.ray_left.diff_y - 1;
                border_x_l = start_x - cone.ray_left.calculate_border();
            }
        } else if border_x_l > cone.curr_l_end_x {
            break;
        }

        // === C# line 71-72: check if cone collapsed ===
        if border_x_r < border_x_l-1 {
            break;
        }

        // === C# line 73: Add range to lanes ===
        // CRITICAL: Add intersection of ray borders and current segment bounds
        // Ray borders show what's VISIBLE, segment bounds show what's WALKABLE
        let range_start = border_x_l.max(cone.curr_l_start_x);
        let range_end = border_x_r.min(cone.curr_l_end_x);

        if cone.curr_l_y >= 0 && cone.curr_l_y < grid.rows && range_start <= range_end {
            lanes[cone.curr_l_y as usize].push((range_start + 1, range_end + 1));
        } else {
            break;
        }

        // === C# lines 108-109: prevL = currL ===
        cone.prev_l_start_x = cone.curr_l_start_x;
        cone.prev_l_end_x = cone.curr_l_end_x;

        // === C# lines 88-92: Find next line ===
        let next_y = cone.curr_l_y + dir;
        if next_y < 0 || next_y >= grid.rows {
            break;
        }

        // C# lines 115-170: Segment loop - find ALL segments and handle splits
        let segments = find_all_segments_in_range(grid, next_y, cone.curr_l_start_x, cone.curr_l_end_x);

        if segments.is_empty() {
            break;
        }

        // C# lines 155-169: First segment continues, others are deferred
        let mut first = true;
        for &(seg_start, seg_end) in &segments {
            if first {
                // C# lines 155-159: lnn = (li, l2); first = true
                cone.curr_l_start_x = seg_start;
                cone.curr_l_end_x = seg_end;
                cone.curr_l_y = next_y;
                first = false;
            } else {
                // C# lines 161-168: Create new cone for split
                // CRITICAL: Deferred cone gets current ray state and starts at THIS row
                let deferred = DeferredCone {
                    ray_right: cone.ray_right, // Current ray state
                    ray_left: cone.ray_left,   // Current ray state
                    curr_l_start_x: seg_start,  // Segment bounds
                    curr_l_end_x: seg_end,
                    curr_l_y: next_y,          // Start at split row
                    prev_l_start_x: cone.prev_l_start_x, // Previous line bounds
                    prev_l_end_x: cone.prev_l_end_x,
                };
                pfn.push(deferred);
            }
        }
    }
}

/// Find ALL walkable segments in a row within range
fn find_all_segments_in_range(grid: &Grid, y: i32, range_start: i32, range_end: i32) -> Vec<(i32, i32)> {
    let mut segments = Vec::new();
    let mut x = range_start;

    while x <= range_end {
        // Skip blocked cells
        while x <= range_end && (x < 0 || x >= grid.cols || grid.is_blocked(x, y)) {
            x += 1;
        }

        if x > range_end {
            break;
        }

        // Found walkable cell - expand to get full segment
        let (seg_start, seg_end) = find_walkable_bounds(grid, x, y);

        // Only include segments that overlap with our range
        if seg_end >= range_start && seg_start <= range_end {
            segments.push((seg_start, seg_end));
        }

        x = seg_end + 1;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let grid = Grid::new(10, 10);
        let visible = raycast(&grid, 5, 5);
        assert_eq!(visible.len(), 100);
    }

    #[test]
    fn test_blocked_start() {
        let grid = Grid::with_blocked(10, 10, &[55]);
        let visible = raycast(&grid, 5, 5);
        assert_eq!(visible.len(), 0);
    }
}
