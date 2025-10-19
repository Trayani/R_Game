use crate::{Grid, raycast};
use crate::corners::{detect_all_corners, filter_interesting_corners, Corner};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;

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
        if let Some((corners, processed)) = self.cache.get_mut(&pos) {
            *processed = true;
        }
    }

    /// Check if a corner has been processed
    fn is_processed(&self, pos: &Position) -> bool {
        self.cache.get(pos).map(|(_, p)| *p).unwrap_or(false)
    }
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

    // Step 1: Get visible cells and corners from observer
    let visible_cells = raycast(grid, start_x, start_y, messy_x, messy_y);
    let visible_positions: HashSet<Position> = visible_cells.iter()
        .map(|&id| {
            let (x, y) = grid.get_coords(id);
            Position::new(x, y)
        })
        .collect();

    // Step 2: Early exit - if destination is visible, return direct path
    if visible_positions.contains(&dest) {
        return Some(vec![start, dest]);
    }

    // Step 3: Determine target corners
    let all_corners = detect_all_corners(grid);
    let interesting_corners = filter_interesting_corners(&all_corners, &visible_cells, grid, start_x, start_y, messy_x);

    let targets = determine_targets(&dest, &all_corners, grid);
    if targets.is_empty() {
        return None; // No valid targets
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
    while let Some(node) = queue.pop() {
        let pos = node.position;

        // Skip if already processed with better distance
        if let Some(&best_dist) = best_distances.get(&pos) {
            if node.total_distance > best_dist {
                continue;
            }
        }

        // Check if this is a target
        if targets.contains(&pos) {
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

    optimal_path
}

/// Determine target corners for pathfinding
fn determine_targets(dest: &Position, all_corners: &[Corner], grid: &Grid) -> HashSet<Position> {
    let mut targets = HashSet::new();

    // Check if destination itself is a corner
    for corner in all_corners {
        if corner.x == dest.x && corner.y == dest.y {
            targets.insert(*dest);
            return targets; // If destination is a corner, it's the only target
        }
    }

    // If destination is not a corner, we need to find corners from which
    // the destination is visible. These become our targets.
    // We'll accept any corner that could "see" the destination.

    // For now, use a simple heuristic: any corner within reasonable range
    // A better approach: actually check visibility from each corner to dest
    for corner in all_corners {
        let dx = (corner.x - dest.x).abs();
        let dy = (corner.y - dest.y).abs();
        // Consider corners within a reasonable distance as potential targets
        let dist_sq = dx * dx + dy * dy;
        if dist_sq <= 100 {  // Within ~10 cells
            targets.insert(Position::new(corner.x, corner.y));
        }
    }

    // If no targets found nearby, use all corners as potential targets
    if targets.is_empty() {
        for corner in all_corners {
            targets.insert(Position::new(corner.x, corner.y));
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
