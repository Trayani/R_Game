use serde::{Deserialize, Serialize};
use crate::{Grid, Actor};
use std::fs;

/// Save state containing grid and actor positions
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveState {
    /// Grid revision number
    pub grid_revision: u64,
    /// Grid dimensions
    pub grid_cols: i32,
    pub grid_rows: i32,
    /// Blocked cells (stored as cell IDs)
    pub blocked_cells: Vec<i32>,
    /// Actor positions (without movement state)
    pub actors: Vec<ActorSaveData>,
}

/// Minimal actor data for saving/loading
#[derive(Debug, Serialize, Deserialize)]
pub struct ActorSaveData {
    pub id: usize,
    pub fpos_x: f32,
    pub fpos_y: f32,
    pub size: f32,
    pub speed: f32,
    pub collision_radius: f32,
}

impl SaveState {
    /// Create a save state from current grid and actors
    pub fn from_grid_and_actors(grid: &Grid, actors: &[Actor], cell_width: f32, cell_height: f32) -> Self {
        // Collect all blocked cells
        let mut blocked_cells = Vec::new();
        for y in 0..grid.rows {
            for x in 0..grid.cols {
                if grid.get_cell(x, y) != 0 {
                    let cell_id = grid.get_id(x, y);
                    blocked_cells.push(cell_id);
                }
            }
        }

        // Extract minimal actor data (position only, no movement state)
        let actors_data: Vec<ActorSaveData> = actors
            .iter()
            .map(|actor| ActorSaveData {
                id: actor.id,
                fpos_x: actor.fpos_x,
                fpos_y: actor.fpos_y,
                size: actor.size,
                speed: actor.speed,
                collision_radius: actor.collision_radius,
            })
            .collect();

        SaveState {
            grid_revision: grid.get_revision(),
            grid_cols: grid.cols,
            grid_rows: grid.rows,
            blocked_cells,
            actors: actors_data,
        }
    }

    /// Save to file
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize save state: {}", e))?;

        fs::write(path, json)
            .map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load from file
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read save file: {}", e))?;

        let save_state: SaveState = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse save file: {}", e))?;

        Ok(save_state)
    }

    /// Restore grid from save state
    pub fn restore_grid(&self) -> Grid {
        let mut grid = Grid::new(self.grid_cols, self.grid_rows);

        // Restore blocked cells
        for &cell_id in &self.blocked_cells {
            let (x, y) = grid.get_coords(cell_id);
            grid.set_cell(x, y, 1);
        }

        grid
    }

    /// Restore actors from save state
    pub fn restore_actors(&self, cell_width: f32, cell_height: f32) -> Vec<Actor> {
        self.actors
            .iter()
            .map(|data| Actor::new(
                data.id,
                data.fpos_x,
                data.fpos_y,
                data.size,
                data.speed,
                data.collision_radius,
                cell_width,
                cell_height,
            ))
            .collect()
    }
}
