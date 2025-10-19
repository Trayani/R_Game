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
/// For messy X with observer_corners: corners explicitly marked as observer corners are auto-added
pub fn filter_interesting_corners(
    all_corners: &[Corner],
    visible_cells: &HashSet<i32>,
    grid: &Grid,
    observer_x: i32,
    observer_y: i32,
    messy_x: bool,
) -> Vec<Corner> {
    filter_interesting_corners_with_observer_corners(all_corners, visible_cells, grid, observer_x, observer_y, messy_x, &[])
}

/// Extended version that allows specifying observer corners (corners occupied by observer that should be auto-added)
pub fn filter_interesting_corners_with_observer_corners(
    all_corners: &[Corner],
    visible_cells: &HashSet<i32>,
    grid: &Grid,
    observer_x: i32,
    observer_y: i32,
    _messy_x: bool,
    observer_corners: &[(i32, i32)],
) -> Vec<Corner> {
    let mut interesting = Vec::new();

    for corner in all_corners {
        let corner_id = grid.get_id(corner.x, corner.y);

        // Corner must be visible
        if !visible_cells.contains(&corner_id) {
            continue;
        }

        // If this corner is marked as an observer corner (e.g., 'z' marker), auto-add as interesting
        if observer_corners.contains(&(corner.x, corner.y)) {
            interesting.push(corner.clone());
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
///
/// A corner is interesting if at least one of its two cardinal directions is NOT visible.
/// If both cardinal directions are visible, you can already see both ways around the corner,
/// so it doesn't lead to unexplored areas.
fn leads_to_hidden_area(
    grid: &Grid,
    corner_x: i32,
    corner_y: i32,
    dir: CornerDirection,
    visible_cells: &HashSet<i32>,
    _observer_x: i32,
    _observer_y: i32,
) -> bool {
    // Get the two cardinal direction cells for this corner
    let (cardinal1_x, cardinal1_y, cardinal2_x, cardinal2_y) = match dir {
        CornerDirection::NW => (corner_x, corner_y - 1, corner_x - 1, corner_y), // North, West
        CornerDirection::NE => (corner_x, corner_y - 1, corner_x + 1, corner_y), // North, East
        CornerDirection::SW => (corner_x, corner_y + 1, corner_x - 1, corner_y), // South, West
        CornerDirection::SE => (corner_x, corner_y + 1, corner_x + 1, corner_y), // South, East
    };

    // Check if each cardinal direction is visible (and walkable)
    let cardinal1_visible = {
        if cardinal1_x >= 0 && cardinal1_x < grid.cols && cardinal1_y >= 0 && cardinal1_y < grid.rows {
            !grid.is_blocked(cardinal1_x, cardinal1_y) &&
            visible_cells.contains(&grid.get_id(cardinal1_x, cardinal1_y))
        } else {
            false // Out of bounds counts as not visible
        }
    };

    let cardinal2_visible = {
        if cardinal2_x >= 0 && cardinal2_x < grid.cols && cardinal2_y >= 0 && cardinal2_y < grid.rows {
            !grid.is_blocked(cardinal2_x, cardinal2_y) &&
            visible_cells.contains(&grid.get_id(cardinal2_x, cardinal2_y))
        } else {
            false // Out of bounds counts as not visible
        }
    };

    // Corner is interesting if at least one cardinal direction is NOT visible
    // (meaning it leads to unexplored areas)
    !cardinal1_visible || !cardinal2_visible
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
