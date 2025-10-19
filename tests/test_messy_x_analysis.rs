// Analysis test for RECI and 3FLIP with messy X
use rustgame3::{Grid, raycast};
use rustgame3::corners::{detect_all_corners, filter_interesting_corners_with_observer_corners};
use std::collections::HashSet;

fn parse_messy_x_test(text: &str) -> (Grid, i32, i32, HashSet<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let lines: Vec<&str> = text.trim().lines()
        .filter(|line| !line.is_empty() && !line.starts_with("s ...") && !line.starts_with("b ...")
                && !line.starts_with("x ...") && !line.starts_with("o ...") && !line.starts_with("c ...") && !line.starts_with("z ..."))
        .collect();

    let rows = lines.len() as i32;
    let cols = if rows > 0 { lines[0].chars().count() as i32 } else { 0 };

    let mut grid = Grid::new(rows, cols);
    let mut observer_x = -1;
    let mut observer_y = -1;
    let mut visible_positions = HashSet::new();
    let mut interesting_corners = Vec::new();
    let mut observer_corners = Vec::new();

    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let x = x as i32;
            let y = y as i32;

            match ch {
                'b' => grid.set_cell(x, y, 1),
                's' => {
                    if observer_x == -1 {
                        observer_x = x;
                        observer_y = y;
                    }
                    visible_positions.insert((x, y));
                }
                'o' => { visible_positions.insert((x, y)); }
                'c' => {
                    visible_positions.insert((x, y));
                    interesting_corners.push((x, y));
                }
                'z' => {
                    visible_positions.insert((x, y));
                    observer_corners.push((x, y));
                }
                'x' => {}
                _ => {}
            }
        }
    }

    (grid, observer_x, observer_y, visible_positions, interesting_corners, observer_corners)
}

#[test]
fn analyze_messy_x_reci() {
    println!("\n=== MESSY X + RECI Analysis ===");

    // Test with both 6_messy_x.txt and 7_messy_x2.txt
    for test_file in &["test_data/corners/6_messy_x.txt", "test_data/corners/7_messy_x2.txt"] {
        println!("\n>>> Testing: {} <<<", test_file);

        let test_data = std::fs::read_to_string(test_file)
            .expect(&format!("Failed to read {}", test_file));

    let grid_lines: Vec<&str> = test_data.lines()
        .skip_while(|line| line.starts_with("s ...") || line.starts_with("x ...")
                    || line.starts_with("o ...") || line.starts_with("c ...") || line.trim().is_empty())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let test_text = grid_lines.join("\n");
    let (grid, obs_x, obs_y, _expected_visible, _expected_interesting, observer_corners) =
        parse_messy_x_test(&test_text);

    println!("Grid: {}x{}", grid.cols, grid.rows);
    println!("Observer at ({},{}) + ({},{}) [messy X]", obs_x, obs_y, obs_x+1, obs_y);

    // Run messy X raycasting
    let visible_cells = raycast(&grid, obs_x, obs_y, true);
    let visible_positions: HashSet<(i32, i32)> = visible_cells.iter()
        .map(|&id| grid.get_coords(id))
        .collect();

    println!("Visible cells: {}", visible_positions.len());

    // Get interesting corners
    let all_corners = detect_all_corners(&grid);
    let interesting_corners = filter_interesting_corners_with_observer_corners(
        &all_corners, &visible_cells, &grid, obs_x, obs_y, true, &observer_corners);

    println!("Interesting corners: {}", interesting_corners.len());

    // RECI TEST: For each interesting corner, check if observer is visible from that corner
    println!("\n--- RECIPROCAL VISIBILITY (RECI) TEST ---");
    println!("Question: If corner C is visible from messy X observer O,");
    println!("          should O be visible from C?");
    println!();

    let mut sees_both = 0;
    let mut sees_one = 0;
    let mut sees_neither = 0;

    for corner in &interesting_corners {
        // Run raycast from corner (NOT messy X, just regular single-cell)
        let corner_visible = raycast(&grid, corner.x, corner.y, false);
        let corner_sees: HashSet<(i32, i32)> = corner_visible.iter()
            .map(|&id| grid.get_coords(id))
            .collect();

        // Check if corner can see EITHER of the observer cells
        let sees_left = corner_sees.contains(&(obs_x, obs_y));
        let sees_right = corner_sees.contains(&(obs_x + 1, obs_y));

        println!("Corner ({},{}):", corner.x, corner.y);
        println!("  Sees left cell ({},{})? {}", obs_x, obs_y, sees_left);
        println!("  Sees right cell ({},{})? {}", obs_x+1, obs_y, sees_right);

        if sees_left && sees_right {
            println!("  ✓ Sees BOTH observer cells");
            sees_both += 1;
        } else if sees_left || sees_right {
            println!("  ⚠️  Sees only ONE observer cell");
            sees_one += 1;
        } else {
            println!("  ❌ Sees NEITHER observer cell");
            sees_neither += 1;
        }
    }

    println!("\n--- RECI Summary ---");
    println!("Corners seeing both observer cells: {}", sees_both);
    println!("Corners seeing only one observer cell: {}", sees_one);
    println!("Corners seeing neither observer cell: {}", sees_neither);

    if sees_one > 0 || sees_neither > 0 {
        println!("\n⚠️  RECI ISSUE: Some corners don't see both observer cells!");
        println!("This happens because messy X uses INTERSECTION (conservative):");
        println!("  - Observer O sees corner C (C is in intersection of both cells' views)");
        println!("  - But corner C might only see ONE of the two observer cells");
        println!("  - This breaks the symmetry expected by RECI");
    } else {
        println!("\n✓ All corners see both observer cells - RECI would work!");
    }
    }  // End of loop over test files
}

#[test]
fn analyze_messy_x_3flip() {
    println!("\n=== MESSY X + 3FLIP Analysis ===");

    let test_data = std::fs::read_to_string("test_data/corners/6_messy_x.txt")
        .expect("Failed to read 6_messy_x.txt");

    let grid_lines: Vec<&str> = test_data.lines()
        .skip_while(|line| line.starts_with("s ...") || line.starts_with("x ...")
                    || line.starts_with("o ...") || line.starts_with("c ...") || line.trim().is_empty())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let test_text = grid_lines.join("\n");
    let (grid, obs_x, obs_y, _expected_visible, _expected_interesting, _observer_corners) =
        parse_messy_x_test(&test_text);

    println!("Grid: {}x{}", grid.cols, grid.rows);
    println!("Original observer: ({},{}) + ({},{}) [messy X]", obs_x, obs_y, obs_x+1, obs_y);
    println!();

    println!("HORIZONTAL FLIP:");
    let h_flip_left = grid.cols - 1 - (obs_x + 1);  // Right cell becomes left
    let h_flip_right = grid.cols - 1 - obs_x;       // Left cell becomes right
    println!("  Flipped observer: ({},{}) + ({},{}) [messy X]", h_flip_left, obs_y, h_flip_right, obs_y);
    println!("  ⚠️  ISSUE: The two cells are SWAPPED in order!");
    println!("            Original: left=({},{}), right=({},{})", obs_x, obs_y, obs_x+1, obs_y);
    println!("            Flipped:  left=({},{}), right=({},{})", h_flip_left, obs_y, h_flip_right, obs_y);
    println!("            The leftmost cell is no longer the 'left' observer cell");
    println!();

    println!("VERTICAL FLIP:");
    let v_flip_y = grid.rows - 1 - obs_y;
    println!("  Flipped observer: ({},{}) + ({},{}) [messy X]", obs_x, v_flip_y, obs_x+1, v_flip_y);
    println!("  ✓ OK: The horizontal adjacency is preserved");
    println!("        Left cell stays left, right cell stays right");
    println!();

    println!("BOTH FLIP:");
    println!("  Combines h-flip and v-flip, so has the same swap issue as h-flip");
    println!("  ⚠️  ISSUE: Cells are swapped");
    println!();

    println!("--- 3FLIP Summary ---");
    println!("Vertical flip (v_flip): ✓ Works - preserves observer cell order");
    println!("Horizontal flip (h_flip): ❌ Breaks - swaps observer cell order");
    println!("Both flip (hv_flip): ❌ Breaks - swaps observer cell order");
    println!();
    println!("CONCLUSION: Only 2 orientations work (original + v_flip), not all 4");
    println!("            This is fundamentally incompatible with the 3FLIP principle");
}
