/// Grid structure for storing cell states
/// Cell values: 0=free/walkable, 1=blocked/wall, -1=chasm (unused for size=1 observer)
#[derive(Clone)]
pub struct Grid {
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<i32>,
    /// Revision number - incremented whenever grid cells change
    pub revision: u64,
}

impl Grid {
    /// Create a new grid with all cells set to free (0)
    pub fn new(rows: i32, cols: i32) -> Self {
        Grid {
            rows,
            cols,
            cells: vec![0; (rows * cols) as usize],
            revision: 0,
        }
    }

    /// Create a grid with specific blocked cells
    pub fn with_blocked(rows: i32, cols: i32, blocked: &[i32]) -> Self {
        let mut grid = Self::new(rows, cols);
        for &cell_id in blocked {
            if cell_id >= 0 && cell_id < (rows * cols) {
                grid.cells[cell_id as usize] = 1;
            }
        }
        grid
    }

    /// Check if a cell at (x, y) is blocked
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.cols || y < 0 || y >= self.rows {
            return true; // Out of bounds is considered blocked
        }
        self.cells[self.get_id(x, y) as usize] == 1
    }

    /// Check if a cell ID is blocked
    pub fn is_blocked_id(&self, id: i32) -> bool {
        if id < 0 || id >= (self.rows * self.cols) {
            return true;
        }
        self.cells[id as usize] == 1
    }

    /// Convert (x, y) coordinates to cell ID
    pub fn get_id(&self, x: i32, y: i32) -> i32 {
        x + y * self.cols
    }

    /// Convert cell ID to (x, y) coordinates
    pub fn get_coords(&self, id: i32) -> (i32, i32) {
        (id % self.cols, id / self.cols)
    }

    /// Get cell value at (x, y)
    pub fn get_cell(&self, x: i32, y: i32) -> i32 {
        if x < 0 || x >= self.cols || y < 0 || y >= self.rows {
            return 1; // Out of bounds is treated as blocked
        }
        self.cells[self.get_id(x, y) as usize]
    }

    /// Set cell value at (x, y)
    pub fn set_cell(&mut self, x: i32, y: i32, value: i32) {
        if x >= 0 && x < self.cols && y >= 0 && y < self.rows {
            let id = self.get_id(x, y);
            let old_value = self.cells[id as usize];
            if old_value != value {
                self.cells[id as usize] = value;
                self.revision += 1;
            }
        }
    }

    /// Get current grid revision number
    pub fn get_revision(&self) -> u64 {
        self.revision
    }
}
