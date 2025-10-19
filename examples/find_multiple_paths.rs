use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners};
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

fn find_multiple_paths(
    grid: &Grid,
    start_x: i32,
    start_y: i32,
    dest_x: i32,
    dest_y: i32,
    max_paths: usize,
) -> Vec<(Vec<i32>, f64)> {
    let start = Position::new(start_x, start_y);
    let dest = Position::new(dest_x, dest_y);

    // Get interesting corners from start
    let visible_cells = raycast(grid, start_x, start_y, false, false);
    let all_corners = detect_all_corners(grid);
    let interesting_corners = filter_interesting_corners(
        &all_corners, &visible_cells, grid, start_x, start_y, false
    );

    // Determine targets (corners visible from dest)
    let dest_visible = raycast(grid, dest_x, dest_y, false, false);
    let dest_corners = filter_interesting_corners(
        &all_corners, &dest_visible, grid, dest_x, dest_y, false
    );

    let mut targets = HashSet::new();
    for corner in dest_corners {
        targets.insert(Position::new(corner.x, corner.y));
    }

    // A* search - collect multiple paths
    let mut queue: BinaryHeap<PathNode> = BinaryHeap::new();
    let mut best_distances: HashMap<Position, Vec<f64>> = HashMap::new();
    let mut found_paths: Vec<(Vec<Position>, f64)> = Vec::new();

    queue.push(PathNode {
        position: start,
        total_distance: 0.0,
        path: vec![start],
    });

    let mut iterations = 0;
    let max_iterations = 100000;

    while let Some(node) = queue.pop() {
        iterations += 1;
        if iterations > max_iterations {
            break;
        }

        let pos = node.position;

        // Check if this is a target
        if targets.contains(&pos) {
            let mut path = node.path.clone();
            if pos != dest {
                path.push(dest);
            }
            found_paths.push((path, node.total_distance));

            if found_paths.len() >= max_paths {
                break;
            }
            continue; // Keep searching for more paths
        }

        // Get next corners
        let next_corners = if pos == start {
            interesting_corners.clone()
        } else {
            let visible = raycast(grid, pos.x, pos.y, false, false);
            filter_interesting_corners(&all_corners, &visible, grid, pos.x, pos.y, false)
        };

        for next_corner in next_corners {
            let next_pos = Position::new(next_corner.x, next_corner.y);
            let distance_to_next = pos.distance(&next_pos);
            let total_distance = node.total_distance + distance_to_next;

            // Allow multiple paths to same position with different distances
            let should_enqueue = match best_distances.get(&next_pos) {
                Some(dists) => {
                    // Only enqueue if this distance is significantly different
                    !dists.iter().any(|&d| (d - total_distance).abs() < 0.01)
                },
                None => true,
            };

            if should_enqueue {
                best_distances.entry(next_pos)
                    .or_insert_with(Vec::new)
                    .push(total_distance);

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

    // Convert to cell IDs and sort by distance
    let mut result: Vec<(Vec<i32>, f64)> = found_paths.iter().map(|(path, dist)| {
        let ids: Vec<i32> = path.iter().map(|p| grid.get_id(p.x, p.y)).collect();
        (ids, *dist)
    }).collect();

    result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    result
}

fn main() {
    let json = fs::read_to_string("test_data/grid2_blocked.json").unwrap();
    let blocked: Vec<i32> = serde_json::from_str(&json).unwrap();
    let grid = Grid::with_blocked(104, 83, &blocked);

    println!("===== FINDING MULTIPLE PATHS: 1211 → 4396 =====\n");

    let paths = find_multiple_paths(&grid, 49, 14, 80, 52, 1000);

    println!("Found {} paths:\n", paths.len());

    let expected_path = vec![1211, 4056, 4310, 4396];

    for (i, (path, dist)) in paths.iter().enumerate() {
        let matches_expected = path == &expected_path;
        let marker = if matches_expected { "👉" } else { "  " };

        // Only print first 10 and last 10, plus any matching expected path
        let should_print = i < 10 || i >= paths.len() - 10 || matches_expected;

        if should_print {
            println!("{} Path {}: {:?}", marker, i + 1, path);
            println!("   Distance: {:.2}", dist);

            if matches_expected {
                println!("   ⭐ THIS IS THE C# PATH!");
            }

            // Show coordinates for first few paths
            if i < 5 || matches_expected {
                print!("   Coords: ");
                for &id in path {
                    let (x, y) = grid.get_coords(id);
                    print!("{}({},{}) ", id, x, y);
                }
                println!();
            }
            println!();
        } else if i == 10 {
            println!("   ... ({} paths omitted) ...\n", paths.len() - 20);
        }
    }

    // Check if expected path is in the list
    let has_expected = paths.iter().any(|(p, _)| p == &expected_path);

    println!("===== SUMMARY =====");
    println!("Total paths found: {}", paths.len());
    println!("C# path [1211, 4056, 4310, 4396] found: {}",
             if has_expected { "✓ YES" } else { "✗ NO" });

    if has_expected {
        let pos = paths.iter().position(|(p, _)| p == &expected_path).unwrap();
        println!("C# path rank: {} out of {}", pos + 1, paths.len());
        println!("C# path distance: {:.2}", paths[pos].1);
    }

    if !paths.is_empty() {
        println!("\nBest path found:");
        println!("  {:?}", paths[0].0);
        println!("  Distance: {:.2}", paths[0].1);
    }
}
