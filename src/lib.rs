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
pub use subcell::{SubCellCoord, SubCellReservationManager, find_best_neighbors, find_square_reservation, spread_subcell_destinations, spread_cell_destinations, get_counter_diagonal_subpoints, calculate_rectangle_bounds_subpoint, is_within_rectangle, spread_subpoint_destinations, find_best_neighbors_subpoint, find_best_3_neighbors_subpoint, find_square_reservation_subpoint, distance_between_subpoints, is_horizontal_move_subpoint, is_vertical_move_subpoint, are_adjacent_subpoints, manhattan_distance_subpoints, clamp_to_rectangle, closest_subpoint_to_screen, is_subpoint_in_bounds, direction_between_subpoints, get_valid_neighbors_subpoint, subpoint_from_components, cell_center_subpoint, same_cell_subpoint, subpoint_to_cell, subpoint_subcell_offset, offset_subpoint, midpoint_subpoints, is_diagonal_move_subpoint, alignment_score_subpoint, neighbors_sorted_by_alignment, best_aligned_neighbor, distance_to_screen_pos, closest_subpoint_to_position, is_at_cell_boundary, cardinal_neighbors_subpoint, diagonal_neighbors_subpoint, subpoint_to_screen_with_offset, subpoint_to_screen, filter_subpoints_in_bounds, subpoints_in_cell, angle_between_subpoints, is_right_angle};
pub use subpoint::{SubPoint, SubPointReservationManager};
