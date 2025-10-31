pub mod action_log;
pub mod actor;
pub mod actor_directives;
pub mod actor_execution;
pub mod compact_log;
pub mod cone;
pub mod config;
pub mod corners;
pub mod grid;
pub mod mirror_triangle;
pub mod pathfinding;
pub mod ray;
pub mod raycast;
pub mod save_state;
pub mod subcell;
pub mod subpoint;  // New flat coordinate system

pub use action_log::{Action, ActionLog, ActionPhase, LoggedAction};
pub use actor::{Actor, CellPosition, MovementEvent, Affinity, AffinityResult};
pub use compact_log::{CompactLogWriter, CompactLogStats};
pub use config::{Config, ReservationEagerness, ReleaseEagerness};
pub use grid::Grid;
pub use raycast::raycast;
pub use save_state::SaveState;
pub use subcell::{SubCellCoord, SubCellReservationManager, find_best_neighbors, find_square_reservation, spread_subcell_destinations, spread_cell_destinations, get_counter_diagonal_subpoints, calculate_rectangle_bounds_subpoint, is_within_rectangle, spread_subpoint_destinations, find_best_neighbors_subpoint, find_best_3_neighbors_subpoint};
pub use subpoint::SubPoint;
