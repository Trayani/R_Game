use std::io::Result as IoResult;
use crate::action_log::{LoggedAction, Action, ActionPhase};

/// Compact binary action log format optimized for size and Claude readability
///
/// Format design:
/// - Variable-length integers (varint) for timestamps and IDs
/// - Delta encoding for timestamps (store differences, not absolute values)
/// - Action type as single byte enum
/// - Compact field encoding
///
/// Size comparison (typical entry):
/// - JSON: ~150 bytes per action (with formatting)
/// - Compact: ~8-20 bytes per action
/// - Compression ratio: ~90% reduction

pub struct CompactLogWriter {
    buffer: Vec<u8>,
    last_timestamp: u64,
}

impl CompactLogWriter {
    pub fn new() -> Self {
        CompactLogWriter {
            buffer: Vec::new(),
            last_timestamp: 0,
        }
    }

    /// Write an action to the compact log
    pub fn write_action(&mut self, action: &LoggedAction) -> IoResult<()> {
        // Write timestamp delta (varint encoded)
        let delta = action.timestamp_ms.saturating_sub(self.last_timestamp);
        self.write_varint(delta);
        self.last_timestamp = action.timestamp_ms;

        // Write action type and data
        self.write_action_data(&action.action, &action.phase)?;

        Ok(())
    }

    /// Encode action type and data in compact format
    fn write_action_data(&mut self, action: &Action, phase: &ActionPhase) -> IoResult<()> {
        // Encode phase as high bit (0=Start, 1=Finish)
        let phase_bit: u8 = match phase {
            ActionPhase::Start => 0,
            ActionPhase::Finish => 0x80,
        };

        match action {
            Action::SetBlocked { x, y } => {
                self.buffer.push(1 | phase_bit);
                self.write_i32(*x);
                self.write_i32(*y);
            }
            Action::SetFree { x, y } => {
                self.buffer.push(2 | phase_bit);
                self.write_i32(*x);
                self.write_i32(*y);
            }
            Action::ToggleCell { x, y } => {
                self.buffer.push(3 | phase_bit);
                self.write_i32(*x);
                self.write_i32(*y);
            }
            Action::MoveObserver { x, y, messy_x, messy_y } => {
                self.buffer.push(4 | phase_bit);
                self.write_i32(*x);
                self.write_i32(*y);
                self.buffer.push((*messy_x as u8) | ((*messy_y as u8) << 1));
            }
            Action::ToggleMessyX => {
                self.buffer.push(5 | phase_bit);
            }
            Action::ToggleMessyY => {
                self.buffer.push(6 | phase_bit);
            }
            Action::SetObserverDestination { x, y } => {
                self.buffer.push(7 | phase_bit);
                self.write_i32(*x);
                self.write_i32(*y);
            }
            Action::SpawnActor { x, y } => {
                self.buffer.push(8 | phase_bit);
                self.write_f32(*x);
                self.write_f32(*y);
            }
            Action::SetActorDestination { x, y, actor_count } => {
                self.buffer.push(9 | phase_bit);
                self.write_i32(*x);
                self.write_i32(*y);
                self.write_varint(*actor_count as u64);
            }
            Action::PasteGrid { rows, cols } => {
                self.buffer.push(10 | phase_bit);
                self.write_i32(*rows);
                self.write_i32(*cols);
            }
            Action::ActorStartMovingToCell { actor_id, cell_x, cell_y, cell_id } => {
                self.buffer.push(11 | phase_bit);
                self.write_varint(*actor_id as u64);
                self.write_i32(*cell_x);
                self.write_i32(*cell_y);
                self.write_i32(*cell_id);
            }
            Action::ActorReachedWaypoint { actor_id, cell_x, cell_y, cell_id, next_cell_x, next_cell_y, next_cell_id } => {
                self.buffer.push(12 | phase_bit);
                self.write_varint(*actor_id as u64);
                self.write_i32(*cell_x);
                self.write_i32(*cell_y);
                self.write_i32(*cell_id);
                self.write_i32(*next_cell_x);
                self.write_i32(*next_cell_y);
                self.write_i32(*next_cell_id);
            }
            Action::ActorReachedDestination { actor_id, cell_x, cell_y, cell_id } => {
                self.buffer.push(13 | phase_bit);
                self.write_varint(*actor_id as u64);
                self.write_i32(*cell_x);
                self.write_i32(*cell_y);
                self.write_i32(*cell_id);
            }
            Action::CalculateCorners { observer_x, observer_y, messy_x, messy_y, total_corners, interesting_corners } => {
                self.buffer.push(14 | phase_bit);
                self.write_i32(*observer_x);
                self.write_i32(*observer_y);
                self.buffer.push((*messy_x as u8) | ((*messy_y as u8) << 1));
                self.write_varint(*total_corners as u64);
                self.write_varint(*interesting_corners as u64);
            }
            Action::CalculatePath { from_x, from_y, to_x, to_y, messy_x, messy_y, path_length, success } => {
                self.buffer.push(15 | phase_bit);
                self.write_i32(*from_x);
                self.write_i32(*from_y);
                self.write_i32(*to_x);
                self.write_i32(*to_y);
                self.buffer.push((*messy_x as u8) | ((*messy_y as u8) << 1) | ((*success as u8) << 2));
                self.write_varint(*path_length as u64);
            }
            Action::ActorStayedDueToCollision { actor_id, fpos_x, fpos_y, blocking_actor_id } => {
                self.buffer.push(16 | phase_bit);
                self.write_varint(*actor_id as u64);
                self.write_f32(*fpos_x);
                self.write_f32(*fpos_y);
                self.write_varint(*blocking_actor_id as u64);
            }
        }

        Ok(())
    }

    /// Write variable-length integer (smaller values use fewer bytes)
    fn write_varint(&mut self, mut value: u64) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80; // More bytes follow
            }
            self.buffer.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    /// Write i32 in compact format (varint for small values)
    fn write_i32(&mut self, value: i32) {
        // ZigZag encoding: map signed to unsigned efficiently
        // 0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3, 2 -> 4, etc.
        let encoded = ((value << 1) ^ (value >> 31)) as u64;
        self.write_varint(encoded);
    }

    /// Write f32 (4 bytes, not compressed)
    fn write_f32(&mut self, value: f32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Get the complete binary log
    pub fn get_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Save to file
    pub fn save_to_file(&self, path: &str) -> IoResult<()> {
        std::fs::write(path, &self.buffer)?;
        Ok(())
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> CompactLogStats {
        CompactLogStats {
            binary_size: self.buffer.len(),
            entry_count: 0, // Caller should track this
        }
    }
}

pub struct CompactLogStats {
    pub binary_size: usize,
    pub entry_count: usize,
}

impl CompactLogStats {
    pub fn avg_bytes_per_entry(&self) -> f64 {
        if self.entry_count == 0 {
            0.0
        } else {
            self.binary_size as f64 / self.entry_count as f64
        }
    }
}
