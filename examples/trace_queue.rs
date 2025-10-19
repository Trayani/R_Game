use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners, Corner};
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    fn distance(&self, other: &Position) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

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
        other.total_distance.partial_cmp(&self.total_distance)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.position.y.cmp(&other.position.y))
    }
}

fn find_path_traced(
    grid: &Grid,
    start_id: i32,
    dest_id: i32,
) -> Option<(Vec<i32>, f64)> {
    let (start_x, start_y) = grid.get_coords(start_id);
    let (dest_x, dest_y) = grid.get_coords(dest_id);

    let start = Position::new(start_x, start_y);
    let dest = Position::new(dest_x, dest_y);

    println!("\n===== DETAILED QUEUE TRACE FOR TEST 002: {} → {} =====\n", start_id, dest_id);
    println!("Start: {} at ({}, {})", start_id, start_x, start_y);
    println!("Dest:  {} at ({}, {})", dest_id, dest_x, dest_y);

    // Calculate straight-line distance as heuristic estimate
    let straight_dist = start.distance(&dest);
    println!("Straight-line distance: {:.2}\n", straight_dist);

    // Step 1: Get interesting corners from start
    let visible_cells = raycast(grid, start_x, start_y, false, false);
    let all_corners = detect_all_corners(grid);
    let interesting_corners = filter_interesting_corners(
        &all_corners, &visible_cells, grid, start_x, start_y, false
    );

    println!("Found {} interesting corners from start", interesting_corners.len());

    // Step 2: Determine targets (corners visible from dest)
    let dest_visible = raycast(grid, dest_x, dest_y, false, false);
    let dest_corners = filter_interesting_corners(
        &all_corners, &dest_visible, grid, dest_x, dest_y, false
    );

    let mut targets = HashSet::new();
    for corner in dest_corners {
        targets.insert(Position::new(corner.x, corner.y));
    }

    println!("Found {} target corners from dest\n", targets.len());
    println!("===== STARTING A* SEARCH =====\n");

    // Step 3: Initialize A* search
    let mut queue: BinaryHeap<PathNode> = BinaryHeap::new();
    let mut best_distances: HashMap<Position, f64> = HashMap::new();
    let mut processed: HashSet<Position> = HashSet::new();
    let mut corner_cache: HashMap<Position, Vec<Corner>> = HashMap::new();
    let mut min_distance = f64::MAX;
    let mut optimal_path: Option<Vec<Position>> = None;

    best_distances.insert(start, 0.0);
    queue.push(PathNode {
        position: start,
        total_distance: 0.0,
        path: vec![start],
    });

    let mut loop_count = 0;

    while let Some(node) = queue.pop() {
        loop_count += 1;
        let pos = node.position;
        let pos_id = grid.get_id(pos.x, pos.y);

        println!("[A*] Loop {}: deq={}, Priority={:.2}, minDist={:.2}",
                 loop_count, pos_id, node.total_distance, min_distance);

        // Skip if already processed with better distance
        if let Some(&best_dist) = best_distances.get(&pos) {
            if node.total_distance > best_dist {
                println!("     Skipping: Already have better distance {:.2}", best_dist);
                continue;
            }
        }

        // Check if this is a target
        if targets.contains(&pos) {
            println!("     ⭐ REACHED TARGET CORNER!");
            if node.total_distance < min_distance {
                println!("     NEW SHORTEST PATH: old={:.2}, new={:.2}", min_distance, node.total_distance);
                min_distance = node.total_distance;
                let mut path = node.path.clone();
                if pos != dest {
                    path.push(dest);
                }
                optimal_path = Some(path);
            }
            continue;
        }

        // Skip if already processed
        if processed.contains(&pos) {
            println!("     Skipping: Already processed");
            continue;
        }

        // Mark as processed
        processed.insert(pos);

        // Get next corners
        let next_corners = if pos == start {
            interesting_corners.clone()
        } else {
            // Check cache first
            if let Some(cached) = corner_cache.get(&pos) {
                cached.clone()
            } else {
                let visible = raycast(grid, pos.x, pos.y, false, false);
                let corners = filter_interesting_corners(&all_corners, &visible, grid, pos.x, pos.y, false);
                corner_cache.insert(pos, corners.clone());
                corners
            }
        };

        println!("     Processing {} corners from {}", next_corners.len(), pos_id);

        for (i, next_corner) in next_corners.iter().enumerate() {
            let next_pos = Position::new(next_corner.x, next_corner.y);
            let next_id = grid.get_id(next_corner.x, next_corner.y);
            let distance_to_next = pos.distance(&next_pos);
            let total_distance = node.total_distance + distance_to_next;

            // Check if should enqueue
            let should_enqueue = match best_distances.get(&next_pos) {
                Some(&best_dist) => total_distance < best_dist,
                None => true,
            };

            if should_enqueue {
                // Highlight important corners (4056 and 4310)
                let marker = if next_id == 4056 || next_id == 4310 { "👉" } else { "  " };

                println!("{}   Corner[{}]: id={}, dist={:.2}, total_dist={:.2}",
                         marker, i, next_id, distance_to_next, total_distance);

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

        println!(); // Blank line between iterations

        // Stop after 20 iterations for readability
        if loop_count >= 20 {
            println!("... (stopping trace after 20 iterations)\n");
            break;
        }
    }

    println!("\n===== RESULT =====");
    if let Some(ref path) = optimal_path {
        let path_ids: Vec<i32> = path.iter().map(|p| grid.get_id(p.x, p.y)).collect();
        println!("Distance: {:.2}", min_distance);
        println!("Waypoints: {:?}", path_ids);

        // Calculate distances for each segment
        println!("\nSegment distances:");
        for i in 1..path.len() {
            let from = &path[i-1];
            let to = &path[i];
            let from_id = grid.get_id(from.x, from.y);
            let to_id = grid.get_id(to.x, to.y);
            let dist = from.distance(to);
            println!("  {} ({},{}) -> {} ({},{}): {:.2}",
                     from_id, from.x, from.y,
                     to_id, to.x, to.y,
                     dist);
        }

        Some((path_ids, min_distance))
    } else {
        println!("NO PATH FOUND");
        None
    }
}

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    find_path_traced(&grid, 1211, 4396);
}
