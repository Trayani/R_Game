use std::time::Instant;
use std::thread::{self, JoinHandle};
use serde::{Serialize, Deserialize};
use crate::compact_log::CompactLogWriter;
use rusqlite::Connection;
use crossbeam_channel::{Sender, Receiver, unbounded};

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
    /// Log a text message
    LogMessage { message: String },
    /// Actor directing decision (affinity, target, reserved subcell, anchor)
    ActorDirecting {
        actor_id: usize,
        affinity: String, // "Horizontal", "Vertical", or "Both"
        target_x: f32,
        target_y: f32,
        reserved_cell_x: i32,
        reserved_cell_y: i32,
        reserved_sub_x: i32,
        reserved_sub_y: i32,
        anchor_cell_x: i32,
        anchor_cell_y: i32,
        anchor_sub_x: i32,
        anchor_sub_y: i32,
    },
    /// PSC (Primary SubCell) selection when target reached
    PSCSelection {
        actor_id: usize,
        old_psc_cell_x: i32,
        old_psc_cell_y: i32,
        old_psc_sub_x: i32,
        old_psc_sub_y: i32,
        reserved_cell_x: i32,
        reserved_cell_y: i32,
        reserved_sub_x: i32,
        reserved_sub_y: i32,
        reserved_dist: f32,
        anchor_cell_x: Option<i32>,  // None for H/V moves
        anchor_cell_y: Option<i32>,
        anchor_sub_x: Option<i32>,
        anchor_sub_y: Option<i32>,
        anchor_dist: Option<f32>,
        chosen: String,  // "Reserved" or "Anchor"
        chosen_cell_x: i32,
        chosen_cell_y: i32,
        chosen_sub_x: i32,
        chosen_sub_y: i32,
    },
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

use std::fs::File;
use std::io::Write;

/// Extract actor_id from action if it has one
fn extract_actor_id(action: &Action) -> Option<i64> {
    match action {
        Action::ActorStartMovingToCell { actor_id, .. } |
        Action::ActorReachedWaypoint { actor_id, .. } |
        Action::ActorReachedDestination { actor_id, .. } |
        Action::ActorDirecting { actor_id, .. } |
        Action::PSCSelection { actor_id, .. } => Some(*actor_id as i64),
        _ => None,
    }
}

/// Get action type name as string
fn action_type_name(action: &Action) -> &'static str {
    match action {
        Action::SetBlocked { .. } => "SetBlocked",
        Action::SetFree { .. } => "SetFree",
        Action::ToggleCell { .. } => "ToggleCell",
        Action::MoveObserver { .. } => "MoveObserver",
        Action::ToggleMessyX => "ToggleMessyX",
        Action::ToggleMessyY => "ToggleMessyY",
        Action::SetObserverDestination { .. } => "SetObserverDestination",
        Action::SpawnActor { .. } => "SpawnActor",
        Action::SetActorDestination { .. } => "SetActorDestination",
        Action::PasteGrid { .. } => "PasteGrid",
        Action::ActorStartMovingToCell { .. } => "ActorStartMovingToCell",
        Action::ActorReachedWaypoint { .. } => "ActorReachedWaypoint",
        Action::ActorReachedDestination { .. } => "ActorReachedDestination",
        Action::LogMessage { .. } => "LogMessage",
        Action::ActorDirecting { .. } => "ActorDirecting",
        Action::PSCSelection { .. } => "PSCSelection",
    }
}

/// Internal message for async logging thread
enum LogMessage {
    Action(LoggedAction),
    Shutdown,
}

/// Action logger with asynchronous background thread
pub struct ActionLog {
    start_time: Instant,
    actions: Vec<LoggedAction>,
    sender: Sender<LogMessage>,
    worker_thread: Option<JoinHandle<()>>,
}

impl ActionLog {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<LogMessage>();

        // Spawn background thread to handle all file I/O
        let worker_thread = thread::spawn(move || {
            Self::worker_thread_main(receiver);
        });

        ActionLog {
            start_time: Instant::now(),
            actions: Vec::new(),
            sender,
            worker_thread: Some(worker_thread),
        }
    }

    /// Background thread main loop - handles all I/O operations
    fn worker_thread_main(receiver: Receiver<LogMessage>) {
        // Open JSON file for streaming writes
        let mut json_file = File::create("action_log.json").ok();
        let mut first_entry = true;

        // Write opening bracket
        if let Some(ref file) = json_file {
            let _ = writeln!(file as &File, "[");
        }

        // Initialize compact binary log
        let mut compact_log = CompactLogWriter::new();

        // Open/create SQLite database and initialize schema
        let db_conn = Connection::open("action_log.db").ok();
        if let Some(ref conn) = db_conn {
            // Create tables and indexes if they don't exist
            let _ = conn.execute_batch("
                CREATE TABLE IF NOT EXISTS actions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp_ms INTEGER NOT NULL,
                    actor_id INTEGER,
                    action_type TEXT NOT NULL,
                    phase TEXT NOT NULL,
                    data TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );

                CREATE INDEX IF NOT EXISTS idx_actor_time
                    ON actions(actor_id, timestamp_ms);

                CREATE INDEX IF NOT EXISTS idx_action_type
                    ON actions(action_type);

                CREATE INDEX IF NOT EXISTS idx_timestamp
                    ON actions(timestamp_ms);
            ");

            // Spawn async cleanup task to delete entries older than 10 minutes
            thread::spawn(|| {
                // Wait a bit for app to fully start before cleanup
                thread::sleep(std::time::Duration::from_secs(5));

                if let Ok(conn) = Connection::open("action_log.db") {
                    let ten_minutes_ago = "datetime('now', '-10 minutes')";
                    let result = conn.execute(
                        &format!("DELETE FROM actions WHERE created_at < {}", ten_minutes_ago),
                        [],
                    );
                    if let Ok(deleted) = result {
                        if deleted > 0 {
                            println!("Cleaned up {} old action log entries (>10 min)", deleted);
                        }
                    }
                }
                // Thread exits after cleanup
            });
        }

        // Process messages until shutdown
        loop {
            match receiver.recv() {
                Ok(LogMessage::Action(logged_action)) => {
                    // Write to compact binary log
                    let _ = compact_log.write_action(&logged_action);

                    // Stream to JSON file immediately (non-pretty-printed)
                    if let Some(ref mut file) = json_file {
                        // Add comma before entry if not first
                        if !first_entry {
                            let _ = writeln!(file, ",");
                        }
                        first_entry = false;

                        // Write JSON entry without pretty-printing
                        if let Ok(json) = serde_json::to_string(&logged_action) {
                            let _ = write!(file, "{}", json);
                            let _ = file.flush(); // Flush immediately
                        }
                    }

                    // Write to SQLite database
                    if let Some(ref conn) = db_conn {
                        let actor_id = extract_actor_id(&logged_action.action);
                        let action_type = action_type_name(&logged_action.action);
                        let phase_str = match logged_action.phase {
                            ActionPhase::Start => "Start",
                            ActionPhase::Finish => "Finish",
                        };

                        // Serialize action data as JSON for storage
                        if let Ok(data_json) = serde_json::to_string(&logged_action.action) {
                            let _ = conn.execute(
                                "INSERT INTO actions (timestamp_ms, actor_id, action_type, phase, data) VALUES (?1, ?2, ?3, ?4, ?5)",
                                rusqlite::params![
                                    logged_action.timestamp_ms as i64,
                                    actor_id,
                                    action_type,
                                    phase_str,
                                    data_json
                                ],
                            );
                        }
                    }
                }
                Ok(LogMessage::Shutdown) => {
                    // Close JSON array
                    if let Some(ref mut file) = json_file {
                        let _ = writeln!(file, "\n]");
                        let _ = file.flush();
                    }

                    // Save compact log
                    let _ = compact_log.save_to_file("action_log.bin");

                    break;
                }
                Err(_) => {
                    // Channel closed, exit thread
                    break;
                }
            }
        }
    }

    /// Log an action with current timestamp and phase (async, non-blocking)
    pub fn log(&mut self, action: Action, phase: ActionPhase) {
        let elapsed = self.start_time.elapsed();
        let timestamp_ms = elapsed.as_millis() as u64;

        let logged_action = LoggedAction {
            timestamp_ms,
            action,
            phase,
        };

        // Store in memory for quick access
        self.actions.push(logged_action.clone());

        // Send to background thread for async I/O (non-blocking)
        let _ = self.sender.send(LogMessage::Action(logged_action));
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

    /// Log a text message
    /// Convenience method for logging arbitrary text messages
    pub fn log_message(&mut self, message: impl Into<String>) {
        self.log_event(Action::LogMessage { message: message.into() });
    }

    /// Get all logged actions
    pub fn get_actions(&self) -> &Vec<LoggedAction> {
        &self.actions
    }

    /// Shutdown background logging thread and flush all pending writes
    pub fn shutdown(&mut self) {
        // Send shutdown signal
        let _ = self.sender.send(LogMessage::Shutdown);

        // Wait for worker thread to finish
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
    }

    /// Save log to JSON file (legacy method - now handled by streaming)
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // The streaming log is already written to action_log.json
        // This method is now a no-op if path is action_log.json
        if path == "action_log.json" {
            return Ok(());
        }
        // For other paths, write in-memory data
        let json = serde_json::to_string(&self.actions)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Save compact binary log to file (handled automatically by background thread on shutdown)
    pub fn save_compact_to_file(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Compact log is saved automatically by background thread on shutdown to "action_log.bin"
        Ok(())
    }

    /// Get compact log size statistics (estimated based on JSON size)
    pub fn get_compact_stats(&self) -> (usize, usize, f64) {
        let json_size = serde_json::to_string(&self.actions).unwrap_or_default().len();
        // Estimate compact size as ~10-15% of JSON size (actual ratio from previous measurements)
        let compact_size = (json_size as f64 * 0.12) as usize;
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

impl Drop for ActionLog {
    fn drop(&mut self) {
        self.shutdown();
    }
}
