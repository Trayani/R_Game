use crate::{Grid, raycast};
use crate::corners::{detect_all_corners, filter_interesting_corners, Corner};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;

// Trace logging flag - set to true to enable debug output
const TRACE_PATHFINDING: bool = true;

/// A position on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    /// Calculate Euclidean distance squared (avoid sqrt for performance)
    pub fn distance_squared(&self, other: &Position) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Calculate Euclidean distance
    pub fn distance(&self, other: &Position) -> f64 {
        ((self.distance_squared(other)) as f64).sqrt()
    }
}

/// A node in the pathfinding search
#[derive(Debug, Clone)]
struct PathNode {
    position: Position,
    total_distance: f64,
    path: Vec<Position>,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.total_distance == other.total_distance
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other.total_distance.partial_cmp(&self.total_distance)
            .unwrap_or(Ordering::Equal)
            // Tie-breaker: use position for deterministic ordering
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.position.y.cmp(&other.position.y))
    }
}

/// Cache for corner visibility computations
struct CornerCache {
    /// Maps corner position -> (visible corners, processed flag)
    cache: HashMap<Position, (Vec<Corner>, bool)>,
}

impl CornerCache {
    fn new() -> Self {
        CornerCache {
            cache: HashMap::new(),
        }
    }

    /// Get cached corners for a position, or compute if not cached
    fn get_or_compute(
        &mut self,
        pos: Position,
        grid: &Grid,
        messy_x: bool,
        messy_y: bool,
    ) -> Vec<Corner> {
        if let Some((corners, _)) = self.cache.get(&pos) {
            return corners.clone();
        }

        // Compute corners for this position
        let visible_cells = raycast(grid, pos.x, pos.y, messy_x, messy_y);
        let all_corners = detect_all_corners(grid);
        let interesting = filter_interesting_corners(&all_corners, &visible_cells, grid, pos.x, pos.y, messy_x);

        self.cache.insert(pos, (interesting.clone(), false));
        interesting
    }

    /// Mark a corner as processed
    fn mark_processed(&mut self, pos: Position) {
        if let Some((_corners, processed)) = self.cache.get_mut(&pos) {
            *processed = true;
        }
    }

    /// Check if a corner has been processed
    fn is_processed(&self, pos: &Position) -> bool {
        self.cache.get(pos).map(|(_, p)| *p).unwrap_or(false)
    }
}

/// Find path using cell IDs (for test compatibility with C# implementation)
/// Returns (path as cell IDs, total distance)
pub fn find_path_by_id(
    grid: &Grid,
    start_id: i32,
    dest_id: i32,
    messy_x: bool,
    messy_y: bool,
) -> Option<(Vec<i32>, f64)> {
    let (start_x, start_y) = grid.get_coords(start_id);
    let (dest_x, dest_y) = grid.get_coords(dest_id);

    let path_positions = find_path(grid, start_x, start_y, dest_x, dest_y, messy_x, messy_y)?;

    // Convert Position path to cell IDs
    let path_ids: Vec<i32> = path_positions
        .iter()
        .map(|p| grid.get_id(p.x, p.y))
        .collect();

    // Calculate total distance
    let mut total_dist = 0.0;
    for i in 1..path_positions.len() {
        total_dist += path_positions[i - 1].distance(&path_positions[i]);
    }

    Some((path_ids, total_dist))
}

/// Find path from start to destination using corner-based pathfinding
pub fn find_path(
    grid: &Grid,
    start_x: i32,
    start_y: i32,
    dest_x: i32,
    dest_y: i32,
    messy_x: bool,
    messy_y: bool,
) -> Option<Vec<Position>> {
    let start = Position::new(start_x, start_y);
    let dest = Position::new(dest_x, dest_y);

    if TRACE_PATHFINDING {
        println!("\n[find_path] START: ({},{}) -> ({},{}), messy_x={}, messy_y={}",
                 start_x, start_y, dest_x, dest_y, messy_x, messy_y);
        println!("[find_path] Start ID: {}, Dest ID: {}",
                 grid.get_id(start_x, start_y), grid.get_id(dest_x, dest_y));
    }

    // Step 1: Get visible cells and corners from observer
    let visible_cells = raycast(grid, start_x, start_y, messy_x, messy_y);
    let visible_positions: HashSet<Position> = visible_cells.iter()
        .map(|&id| {
            let (x, y) = grid.get_coords(id);
            Position::new(x, y)
        })
        .collect();

    if TRACE_PATHFINDING {
        println!("[find_path] Visible cells from start: {} cells", visible_cells.len());
    }

    // Step 2: Early exit - if destination is visible, return direct path
    if visible_positions.contains(&dest) {
        if TRACE_PATHFINDING {
            println!("[find_path] Destination is directly visible - returning direct path");
        }
        return Some(vec![start, dest]);
    }

    // Step 3: Determine target corners
    let all_corners = detect_all_corners(grid);
    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, grid, start_x, start_y, messy_x);

    if TRACE_PATHFINDING {
        println!("[find_path] Start interesting corners: {} corners", interesting_corners.len());
        for (i, corner) in interesting_corners.iter().enumerate().take(5) {
            println!("  [{}] Corner at ({},{}) = ID {}", i, corner.x, corner.y, grid.get_id(corner.x, corner.y));
        }
        if interesting_corners.len() > 5 {
            println!("  ... and {} more", interesting_corners.len() - 5);
        }
    }

    let targets = determine_targets(&dest, grid);
    if targets.is_empty() {
        if TRACE_PATHFINDING {
            println!("[find_path] No target corners found - no path possible");
        }
        return None; // No valid targets
    }

    if TRACE_PATHFINDING {
        println!("[find_path] Target corners: {} corners", targets.len());
        for (i, target) in targets.iter().enumerate().take(5) {
            println!("  [{}] Target at ({},{}) = ID {}", i, target.x, target.y, grid.get_id(target.x, target.y));
        }
        if targets.len() > 5 {
            println!("  ... and {} more", targets.len() - 5);
        }
    }

    // Step 4: Initialize search
    let mut cache = CornerCache::new();
    let mut queue: BinaryHeap<PathNode> = BinaryHeap::new();
    let mut best_distances: HashMap<Position, f64> = HashMap::new();
    let mut min_distance = f64::MAX;
    let mut optimal_path: Option<Vec<Position>> = None;

    // Enqueue initial interesting corners
    for corner in &interesting_corners {
        let pos = Position::new(corner.x, corner.y);
        let distance = start.distance(&pos);
        best_distances.insert(pos, distance);
        queue.push(PathNode {
            position: pos,
            total_distance: distance,
            path: vec![start, pos],
        });
    }

    // Step 5: Process queue
    let mut iterations = 0;
    while let Some(node) = queue.pop() {
        iterations += 1;
        let pos = node.position;

        if TRACE_PATHFINDING && iterations <= 10 {
            println!("[A*] Iteration {}: Expanding ({},{}) = ID {}, dist={:.2}",
                     iterations, pos.x, pos.y, grid.get_id(pos.x, pos.y), node.total_distance);
        }

        // Skip if already processed with better distance
        if let Some(&best_dist) = best_distances.get(&pos) {
            if node.total_distance > best_dist {
                continue;
            }
        }

        // Check if this is a target
        if targets.contains(&pos) {
            if TRACE_PATHFINDING {
                println!("[A*] Found target at ({},{}) = ID {}, dist={:.2}",
                         pos.x, pos.y, grid.get_id(pos.x, pos.y), node.total_distance);
            }
            if node.total_distance < min_distance {
                min_distance = node.total_distance;
                let mut path = node.path.clone();
                path.push(dest);
                optimal_path = Some(path);
            }
            continue;
        }

        // Skip if already processed
        if cache.is_processed(&pos) {
            continue;
        }

        // Mark as processed
        cache.mark_processed(pos);

        // Get this corner's interesting corners
        let next_corners = cache.get_or_compute(pos, grid, messy_x, messy_y);

        if TRACE_PATHFINDING && iterations <= 10 {
            println!("  Found {} next corners from ({},{})", next_corners.len(), pos.x, pos.y);
        }

        for next_corner in next_corners {
            let next_pos = Position::new(next_corner.x, next_corner.y);
            let distance_to_next = pos.distance(&next_pos);
            let total_distance = node.total_distance + distance_to_next;

            // Only enqueue if this is a better path
            let should_enqueue = match best_distances.get(&next_pos) {
                Some(&best_dist) => total_distance < best_dist,
                None => true,
            };

            if should_enqueue {
                best_distances.insert(next_pos, total_distance);
                let mut new_path = node.path.clone();
                new_path.push(next_pos);
                queue.push(PathNode {
                    position: next_pos,
                    total_distance,
                    path: new_path,
                });
            }
        }
    }

    if TRACE_PATHFINDING {
        if let Some(ref path) = optimal_path {
            println!("[find_path] FOUND PATH: {} waypoints, dist={:.2}", path.len(), min_distance);
            println!("[find_path] Path IDs: {:?}", path.iter().map(|p| grid.get_id(p.x, p.y)).collect::<Vec<_>>());
        } else {
            println!("[find_path] NO PATH FOUND after {} iterations", iterations);
        }
    }

    optimal_path
}

/// Determine target corners for pathfinding
/// These are corners from which the destination is visible
fn determine_targets(dest: &Position, grid: &Grid) -> HashSet<Position> {
    let mut targets = HashSet::new();

    // Raycast FROM the destination to find which corners can see it
    // This matches C# behavior where dest's interesting corners become targets
    let dest_visible = raycast(grid, dest.x, dest.y, false, false);

    // Detect all corners in the grid
    let all_corners = detect_all_corners(grid);

    // Filter for interesting corners visible from destination
    let dest_corners = filter_interesting_corners(
        &all_corners,
        &dest_visible,
        grid,
        dest.x,
        dest.y,
        false,
    );

    // These corners can "see" the destination, so they are valid targets
    for corner in dest_corners {
        targets.insert(Position::new(corner.x, corner.y));
    }

    // Check if destination itself is a corner - if so, add it as the only target
    for corner in &all_corners {
        if corner.x == dest.x && corner.y == dest.y {
            targets.clear();  // Clear other targets
            targets.insert(*dest);  // Dest is the only target
            break;
        }
    }

    targets
}

/// Format path for display
pub fn format_path(path: &[Position]) -> String {
    if path.is_empty() {
        return "No path".to_string();
    }

    let mut result = String::new();
    for (i, pos) in path.iter().enumerate() {
        if i > 0 {
            result.push_str(" -> ");
        }
        result.push_str(&format!("({},{})", pos.x, pos.y));
    }
    result
}
