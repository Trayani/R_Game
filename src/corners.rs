use crate::grid::Grid;
use std::collections::HashSet;

/// Corner direction types - a cell can be multiple corner types simultaneously
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerDirection {
    NW, // Northwest corner
    NE, // Northeast corner
    SW, // Southwest corner
    SE, // Southeast corner
}

/// A corner in the grid with its position and the directions it serves as a corner
#[derive(Debug, Clone)]
pub struct Corner {
    pub x: i32,
    pub y: i32,
    pub directions: HashSet<CornerDirection>,
}

impl Corner {
    pub fn new(x: i32, y: i32) -> Self {
        Corner {
            x,
            y,
            directions: HashSet::new(),
        }
    }

    pub fn add_direction(&mut self, dir: CornerDirection) {
        self.directions.insert(dir);
    }

    pub fn is_empty(&self) -> bool {
        self.directions.is_empty()
    }
}

/// Detect all corners in the grid, independent of observer position.
/// A cell is a corner when you can travel around it from vertical↔horizontal direction.
pub fn detect_all_corners(grid: &Grid) -> Vec<Corner> {
    let mut corners = Vec::new();

    for y in 0..grid.rows {
        for x in 0..grid.cols {
            // Only free cells can be corners
            if grid.is_blocked(x, y) {
                continue;
            }

            let mut corner = Corner::new(x, y);

            // Check each diagonal direction independently
            check_nw_corner(grid, x, y, &mut corner);
            check_ne_corner(grid, x, y, &mut corner);
            check_sw_corner(grid, x, y, &mut corner);
            check_se_corner(grid, x, y, &mut corner);

            if !corner.is_empty() {
                corners.push(corner);
            }
        }
    }

    corners
}

/// Check if cell (x,y) is a NW corner
/// NW corner: can turn from North→West or West→North around this cell
/// This happens when:
/// - North AND West are BOTH walkable (need both directions to turn between them)
/// - NW diagonal is blocked/boundary (obstacle around which to turn)
fn check_nw_corner(grid: &Grid, x: i32, y: i32, corner: &mut Corner) {
    let north_free = !grid.is_blocked(x, y - 1);
    let west_free = !grid.is_blocked(x - 1, y);
    let nw_blocked = grid.is_blocked(x - 1, y - 1);

    // Can turn if BOTH cardinal directions are free AND diagonal is blocked
    if north_free && west_free && nw_blocked {
        corner.add_direction(CornerDirection::NW);
    }
}

/// Check if cell (x,y) is a NE corner
fn check_ne_corner(grid: &Grid, x: i32, y: i32, corner: &mut Corner) {
    let north_free = !grid.is_blocked(x, y - 1);
    let east_free = !grid.is_blocked(x + 1, y);
    let ne_blocked = grid.is_blocked(x + 1, y - 1);

    if north_free && east_free && ne_blocked {
        corner.add_direction(CornerDirection::NE);
    }
}

/// Check if cell (x,y) is a SW corner
fn check_sw_corner(grid: &Grid, x: i32, y: i32, corner: &mut Corner) {
    let south_free = !grid.is_blocked(x, y + 1);
    let west_free = !grid.is_blocked(x - 1, y);
    let sw_blocked = grid.is_blocked(x - 1, y + 1);

    if south_free && west_free && sw_blocked {
        corner.add_direction(CornerDirection::SW);
    }
}

/// Check if cell (x,y) is a SE corner
fn check_se_corner(grid: &Grid, x: i32, y: i32, corner: &mut Corner) {
    let south_free = !grid.is_blocked(x, y + 1);
    let east_free = !grid.is_blocked(x + 1, y);
    let se_blocked = grid.is_blocked(x + 1, y + 1);

    if south_free && east_free && se_blocked {
        corner.add_direction(CornerDirection::SE);
    }
}

/// Filter corners to find "interesting" ones - those visible to observer AND
/// leading to directions that are not further visible (behind the corner)
pub fn filter_interesting_corners(
    all_corners: &[Corner],
    visible_cells: &HashSet<i32>,
    grid: &Grid,
    observer_x: i32,
    observer_y: i32,
) -> Vec<Corner> {
    let mut interesting = Vec::new();

    for corner in all_corners {
        let corner_id = grid.get_id(corner.x, corner.y);

        // Corner must be visible
        if !visible_cells.contains(&corner_id) {
            continue;
        }

        // Check if at least one direction the corner leads to is NOT visible
        let mut has_hidden_direction = false;

        for &dir in &corner.directions {
            if leads_to_hidden_area(grid, corner.x, corner.y, dir, visible_cells, observer_x, observer_y) {
                has_hidden_direction = true;
                break;
            }
        }

        if has_hidden_direction {
            interesting.push(corner.clone());
        }
    }

    interesting
}

/// Check if a corner direction leads to an area that's not visible (behind the corner)
/// A corner is only interesting if it's "facing away" from the observer
fn leads_to_hidden_area(
    grid: &Grid,
    corner_x: i32,
    corner_y: i32,
    dir: CornerDirection,
    visible_cells: &HashSet<i32>,
    observer_x: i32,
    observer_y: i32,
) -> bool {
    // Calculate vector from observer to corner
    let dx = corner_x - observer_x;
    let dy = corner_y - observer_y;

    // Check if this corner direction is "facing away" from the observer
    // i.e., the corner direction vector should point generally in the same direction
    // as the vector from observer to corner
    let is_facing_away = match dir {
        CornerDirection::NW => dx <= 0 && dy <= 0, // Corner is NW, observer should be SE of it
        CornerDirection::NE => dx >= 0 && dy <= 0, // Corner is NE, observer should be SW of it
        CornerDirection::SW => dx <= 0 && dy >= 0, // Corner is SW, observer should be NE of it
        CornerDirection::SE => dx >= 0 && dy >= 0, // Corner is SE, observer should be NW of it
    };

    // Corner must be facing away from observer to be interesting
    if !is_facing_away {
        return false;
    }

    // Now check if there are non-visible cells in the direction "behind" the corner
    let check_positions = match dir {
        CornerDirection::NW => {
            // Check cells to the northwest beyond the corner
            vec![
                (corner_x - 1, corner_y - 1), // diagonal
                (corner_x - 2, corner_y - 1), // further west
                (corner_x - 1, corner_y - 2), // further north
                (corner_x - 2, corner_y - 2), // further diagonal
            ]
        }
        CornerDirection::NE => {
            vec![
                (corner_x + 1, corner_y - 1),
                (corner_x + 2, corner_y - 1),
                (corner_x + 1, corner_y - 2),
                (corner_x + 2, corner_y - 2),
            ]
        }
        CornerDirection::SW => {
            vec![
                (corner_x - 1, corner_y + 1),
                (corner_x - 2, corner_y + 1),
                (corner_x - 1, corner_y + 2),
                (corner_x - 2, corner_y + 2),
            ]
        }
        CornerDirection::SE => {
            vec![
                (corner_x + 1, corner_y + 1),
                (corner_x + 2, corner_y + 1),
                (corner_x + 1, corner_y + 2),
                (corner_x + 2, corner_y + 2),
            ]
        }
    };

    // If any of the "beyond" cells are free but NOT visible, this corner leads to hidden area
    for (x, y) in check_positions {
        if x >= 0 && x < grid.cols && y >= 0 && y < grid.rows {
            let cell_id = grid.get_id(x, y);
            if !grid.is_blocked(x, y) && !visible_cells.contains(&cell_id) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_3x3_corner() {
        // 3x3 grid with blocked cell at (1,1) - center
        // □□□
        // □■□
        // □□□
        let grid = Grid::with_blocked(3, 3, &[4]); // cell ID 4 = (1,1)
        let corners = detect_all_corners(&grid);

        println!("Detected {} corners:", corners.len());
        for c in &corners {
            println!("  Corner at ({}, {}) with directions: {:?}", c.x, c.y, c.directions);
        }

        // Check that all 4 corner positions are detected
        let positions: HashSet<(i32, i32)> = corners.iter().map(|c| (c.x, c.y)).collect();
        assert!(positions.contains(&(0, 0))); // NW of grid
        assert!(positions.contains(&(2, 0))); // NE of grid
        assert!(positions.contains(&(0, 2))); // SW of grid
        assert!(positions.contains(&(2, 2))); // SE of grid

        // Might detect more corners at edges/sides - that's OK
        // The spec examples show corners at the 4 corners of the grid
    }

    #[test]
    fn test_4x3_two_blocks() {
        // 4x3 grid = width 4, height 3 = 4 columns, 3 rows
        // Grid layout:
        // □□□□  <- row 0
        // □□■■  <- row 1, blocks at x=2,3
        // □□□□  <- row 2
        let grid = Grid::with_blocked(3, 4, &[6, 7]); // rows=3, cols=4, blocks at (2,1) and (3,1)

        // Debug: Print grid
        println!("Grid: {}x{} (cols x rows)", grid.cols, grid.rows);
        for y in 0..grid.rows {
            for x in 0..grid.cols {
                let ch = if grid.is_blocked(x, y) { '■' } else { '□' };
                print!("{}", ch);
            }
            println!(" <- row {}", y);
        }
        println!("\nCell 6 coords: {:?}", grid.get_coords(6));
        println!("Cell 7 coords: {:?}", grid.get_coords(7));

        let corners = detect_all_corners(&grid);

        println!("\nDetected {} corners:", corners.len());
        for c in &corners {
            println!("  Corner at ({}, {}) with directions: {:?}", c.x, c.y, c.directions);
        }

        // The actual corners where you can turn around obstacles:
        // (1,0) with SE - can turn around the block at (2,1)
        // (1,2) with NE - can turn around the block at (2,1)
        let positions: HashSet<(i32, i32)> = corners.iter().map(|c| (c.x, c.y)).collect();
        assert!(positions.contains(&(1, 0)), "Should have corner at (1,0)");
        assert!(positions.contains(&(1, 2)), "Should have corner at (1,2)");
        assert_eq!(corners.len(), 2, "Should have exactly 2 corners");
    }

    #[test]
    fn test_multi_direction_corner() {
        // Create a cross pattern where center cell is corner in all 4 directions
        // ■ □ ■
        // □ X □  <- X should be NW, NE, SW, SE corner
        // ■ □ ■
        let grid = Grid::with_blocked(3, 3, &[0, 2, 6, 8]);
        let corners = detect_all_corners(&grid);

        // Find the center corner
        let center = corners.iter().find(|c| c.x == 1 && c.y == 1).unwrap();

        // Should have all 4 directions
        assert_eq!(center.directions.len(), 4);
        assert!(center.directions.contains(&CornerDirection::NW));
        assert!(center.directions.contains(&CornerDirection::NE));
        assert!(center.directions.contains(&CornerDirection::SW));
        assert!(center.directions.contains(&CornerDirection::SE));
    }
}
