use std::time::Instant;
use serde::{Serialize, Deserialize};

/// User actions that interact with the grid or actors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    /// Set a cell to blocked (x, y)
    SetBlocked { x: i32, y: i32 },
    /// Set a cell to free (x, y)
    SetFree { x: i32, y: i32 },
    /// Toggle a cell (x, y)
    ToggleCell { x: i32, y: i32 },
    /// Move observer to position (x, y, messy_x, messy_y)
    MoveObserver { x: i32, y: i32, messy_x: bool, messy_y: bool },
    /// Toggle messy X
    ToggleMessyX,
    /// Toggle messy Y
    ToggleMessyY,
    /// Set observer destination (x, y)
    SetObserverDestination { x: i32, y: i32 },
    /// Spawn actor at floating position (x, y)
    SpawnActor { x: f32, y: f32 },
    /// Set destination for all actors (x, y, actor_count)
    SetActorDestination { x: i32, y: i32, actor_count: usize },
    /// Paste grid from clipboard (rows, cols)
    PasteGrid { rows: i32, cols: i32 },
}

/// Logged action with timestamp
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggedAction {
    /// Milliseconds since start
    pub timestamp_ms: u64,
    /// The action
    pub action: Action,
}

/// Action logger
pub struct ActionLog {
    start_time: Instant,
    actions: Vec<LoggedAction>,
}

impl ActionLog {
    pub fn new() -> Self {
        ActionLog {
            start_time: Instant::now(),
            actions: Vec::new(),
        }
    }

    /// Log an action with current timestamp
    pub fn log(&mut self, action: Action) {
        let elapsed = self.start_time.elapsed();
        let timestamp_ms = elapsed.as_millis() as u64;

        self.actions.push(LoggedAction {
            timestamp_ms,
            action,
        });
    }

    /// Get all logged actions
    pub fn get_actions(&self) -> &Vec<LoggedAction> {
        &self.actions
    }

    /// Save log to JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.actions)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Print log to console
    pub fn print(&self) {
        println!("\n=== Action Log ({} actions) ===", self.actions.len());
        for (i, logged) in self.actions.iter().enumerate() {
            println!("[{:6}ms] #{:3} {:?}", logged.timestamp_ms, i + 1, logged.action);
        }
        println!("=== End of Log ===\n");
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        let mut blocked_count = 0;
        let mut free_count = 0;
        let mut toggle_count = 0;
        let mut actor_spawns = 0;
        let mut destination_sets = 0;
        let mut total_actors_commanded = 0;

        for logged in &self.actions {
            match &logged.action {
                Action::SetBlocked { .. } => blocked_count += 1,
                Action::SetFree { .. } => free_count += 1,
                Action::ToggleCell { .. } => toggle_count += 1,
                Action::SpawnActor { .. } => actor_spawns += 1,
                Action::SetActorDestination { actor_count, .. } => {
                    destination_sets += 1;
                    total_actors_commanded += actor_count;
                }
                _ => {}
            }
        }

        let duration = if let Some(last) = self.actions.last() {
            last.timestamp_ms
        } else {
            0
        };

        format!(
            "Session Duration: {}ms\n\
             Total Actions: {}\n\
             Grid Modifications: {} blocked, {} freed, {} toggled\n\
             Actor Operations: {} spawned, {} destination commands ({} total actors commanded)",
            duration,
            self.actions.len(),
            blocked_count,
            free_count,
            toggle_count,
            actor_spawns,
            destination_sets,
            total_actors_commanded
        )
    }
}
