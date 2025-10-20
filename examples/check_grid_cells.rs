use rustgame3::Grid;

fn main() {
    let mut grid = Grid::new(12, 10);

    // Create obstacle
    for y in 3..7 {
        grid.set_cell(6, y, 1);
    }

    println!("Grid: {}x{} (cols x rows)", grid.cols, grid.rows);
    println!();

    // Check the rightmost columns
    for y in 0..10 {
        for x in 8..12 {
            let cell = grid.get_cell(x, y);
            print!("({},{})={} ", x, y, cell);
        }
        println!();
    }
}
