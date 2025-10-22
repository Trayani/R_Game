use std::time::Instant;
use serde::{Serialize, Deserialize};
use crate::compact_log::CompactLogWriter;

/// Action phase - whether the action is starting or finishing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionPhase {
    Start,
    Finish,
}

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
    /// Actor starts moving to next waypoint (actor_id, cell_x, cell_y, cell_id)
    ActorStartMovingToCell { actor_id: usize, cell_x: i32, cell_y: i32, cell_id: i32 },
    /// Actor reached waypoint and proceeds to next (actor_id, cell_x, cell_y, cell_id, next_cell_x, next_cell_y, next_cell_id)
    ActorReachedWaypoint { actor_id: usize, cell_x: i32, cell_y: i32, cell_id: i32, next_cell_x: i32, next_cell_y: i32, next_cell_id: i32 },
    /// Actor reached final destination (actor_id, cell_x, cell_y, cell_id)
    ActorReachedDestination { actor_id: usize, cell_x: i32, cell_y: i32, cell_id: i32 },
    /// Corner calculation (observer_x, observer_y, messy_x, messy_y, total_corners, interesting_corners)
    CalculateCorners { observer_x: i32, observer_y: i32, messy_x: bool, messy_y: bool, total_corners: usize, interesting_corners: usize },
    /// Path calculation (from_x, from_y, to_x, to_y, messy_x, messy_y, path_length, success)
    CalculatePath { from_x: i32, from_y: i32, to_x: i32, to_y: i32, messy_x: bool, messy_y: bool, path_length: usize, success: bool },
    /// Actor stayed due to collision radius violation (actor_id, fpos_x, fpos_y, blocking_actor_id)
    ActorStayedDueToCollision { actor_id: usize, fpos_x: f32, fpos_y: f32, blocking_actor_id: usize },
}

/// Logged action with timestamp and phase
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggedAction {
    /// Milliseconds since start
    pub timestamp_ms: u64,
    /// The action
    pub action: Action,
    /// Whether this is the start or finish of the action
    pub phase: ActionPhase,
}

/// Action logger
pub struct ActionLog {
    start_time: Instant,
    actions: Vec<LoggedAction>,
    compact_log: CompactLogWriter,
}

impl ActionLog {
    pub fn new() -> Self {
        ActionLog {
            start_time: Instant::now(),
            actions: Vec::new(),
            compact_log: CompactLogWriter::new(),
        }
    }

    /// Log an action with current timestamp and phase
    pub fn log(&mut self, action: Action, phase: ActionPhase) {
        let elapsed = self.start_time.elapsed();
        let timestamp_ms = elapsed.as_millis() as u64;

        let logged_action = LoggedAction {
            timestamp_ms,
            action,
            phase,
        };

        // Write to both JSON and compact binary logs
        self.actions.push(logged_action.clone());
        let _ = self.compact_log.write_action(&logged_action);
    }

    /// Log the start of an action
    pub fn log_start(&mut self, action: Action) {
        self.log(action, ActionPhase::Start);
    }

    /// Log the finish of an action
    pub fn log_finish(&mut self, action: Action) {
        self.log(action, ActionPhase::Finish);
    }

    /// Log a single event (finish only, no duration tracking)
    /// Use this for events that don't need start/finish pairs to reduce log size
    pub fn log_event(&mut self, action: Action) {
        self.log(action, ActionPhase::Finish);
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

    /// Save compact binary log to file
    pub fn save_compact_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.compact_log.save_to_file(path)?;
        Ok(())
    }

    /// Get compact log size statistics
    pub fn get_compact_stats(&self) -> (usize, usize, f64) {
        let json_size = serde_json::to_string(&self.actions).unwrap_or_default().len();
        let compact_size = self.compact_log.get_bytes().len();
        let compression_ratio = if json_size > 0 {
            (json_size - compact_size) as f64 / json_size as f64 * 100.0
        } else {
            0.0
        };
        (json_size, compact_size, compression_ratio)
    }

    /// Print log to console
    pub fn print(&self) {
        println!("\n=== Action Log ({} events) ===", self.actions.len());
        for (i, logged) in self.actions.iter().enumerate() {
            let phase_str = match logged.phase {
                ActionPhase::Start => "START ",
                ActionPhase::Finish => "FINISH",
            };
            println!("[{:6}ms] #{:3} {} {:?}", logged.timestamp_ms, i + 1, phase_str, logged.action);
        }
        println!("=== End of Log ===\n");
    }

    /// Print log with duration analysis
    pub fn print_with_durations(&self) {
        use std::collections::HashMap;

        println!("\n=== Action Log with Durations ===");

        // Track start times for each action type
        let mut start_times: HashMap<String, u64> = HashMap::new();

        for (i, logged) in self.actions.iter().enumerate() {
            let action_key = format!("{:?}", logged.action);

            match logged.phase {
                ActionPhase::Start => {
                    start_times.insert(action_key.clone(), logged.timestamp_ms);
                    println!("[{:6}ms] #{:3} START  {:?}", logged.timestamp_ms, i + 1, logged.action);
                }
                ActionPhase::Finish => {
                    if let Some(start_ms) = start_times.remove(&action_key) {
                        let duration = logged.timestamp_ms - start_ms;
                        println!("[{:6}ms] #{:3} FINISH {:?} [duration: {}ms]",
                                logged.timestamp_ms, i + 1, logged.action, duration);
                    } else {
                        println!("[{:6}ms] #{:3} FINISH {:?} [no matching start]",
                                logged.timestamp_ms, i + 1, logged.action);
                    }
                }
            }
        }
        println!("=== End of Log ===\n");
    }

    /// Get summary statistics (including compact log info)
    pub fn summary(&self) -> String {
        let mut blocked_count = 0;
        let mut free_count = 0;
        let mut toggle_count = 0;
        let mut actor_spawns = 0;
        let mut destination_sets = 0;
        let mut total_actors_commanded = 0;

        // Only count finish events to get actual completed action counts
        for logged in &self.actions {
            if matches!(logged.phase, ActionPhase::Finish) {
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
        }

        let duration = if let Some(last) = self.actions.last() {
            last.timestamp_ms
        } else {
            0
        };

        let (json_size, compact_size, compression_ratio) = self.get_compact_stats();

        format!(
            "Session Duration: {}ms\n\
             Total Events: {} ({} action pairs)\n\
             Grid Modifications: {} blocked, {} freed, {} toggled\n\
             Actor Operations: {} spawned, {} destination commands ({} total actors commanded)\n\
             Log Sizes: JSON={} bytes, Compact={} bytes ({:.1}% reduction)",
            duration,
            self.actions.len(),
            self.actions.len() / 2,
            blocked_count,
            free_count,
            toggle_count,
            actor_spawns,
            destination_sets,
            total_actors_commanded,
            json_size,
            compact_size,
            compression_ratio
        )
    }
}
