/// Mirror Triangle Mechanics for DestinationDirect Mode
///
/// This module implements the continuous triangle-chain navigation described in
/// design/subcell_triangles_revised.txt Section 2 (Q2.5-Q2.6).
///
/// Key Concepts:
/// - Current triangle: (PSC, reserved_diagonal, anchor)
/// - Mirror triangle: Next triangle in sequence that shares an edge with current
/// - Shared edge: The 2 subcells that both triangles have in common
/// - New diagonal: The subcell that completes the mirror triangle
/// - Redundant subcell: The old subcell no longer needed (opposite the shared edge)

use crate::subcell::{SubCellCoord, SubCellReservationManager};

/// Represents a triangle formed by 3 subcell coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    /// The three vertices (subcell centers) forming the triangle
    /// Order: [PSC/base, vertex2, vertex3]
    pub vertices: [SubCellCoord; 3],
}

impl Triangle {
    /// Create a new triangle from 3 subcell coordinates
    pub fn new(v1: SubCellCoord, v2: SubCellCoord, v3: SubCellCoord) -> Self {
        Triangle {
            vertices: [v1, v2, v3],
        }
    }

    /// Get the PSC (base vertex) of the triangle
    pub fn psc(&self) -> &SubCellCoord {
        &self.vertices[0]
    }

    /// Check if this triangle contains a given subcell as one of its vertices
    pub fn contains_vertex(&self, sc: &SubCellCoord) -> bool {
        self.vertices.iter().any(|v| v == sc)
    }

    /// Get the two vertices that are NOT the given vertex (finds the opposite edge)
    /// Returns None if the given vertex is not in this triangle
    pub fn get_opposite_edge(&self, vertex: &SubCellCoord) -> Option<[SubCellCoord; 2]> {
        let others: Vec<SubCellCoord> = self.vertices
            .iter()
            .filter(|v| *v != vertex)
            .copied()
            .collect();

        if others.len() == 2 {
            Some([others[0], others[1]])
        } else {
            None
        }
    }

    /// Find which vertex is the "redundant" one when transitioning to a mirror
    /// This is typically the PSC (base) vertex when moving forward
    pub fn find_redundant_for_direction(
        &self,
        dest_x: f32,
        dest_y: f32,
        cell_width: f32,
        cell_height: f32,
    ) -> SubCellCoord {
        // Calculate which vertex is furthest from destination
        let mut max_dist = -1.0;
        let mut redundant = self.vertices[0];

        for vertex in &self.vertices {
            let (vx, vy) = vertex.to_screen_center(cell_width, cell_height);
            let dx = dest_x - vx;
            let dy = dest_y - vy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > max_dist {
                max_dist = dist;
                redundant = *vertex;
            }
        }

        redundant
    }
}

/// Result of mirror triangle identification
#[derive(Debug, Clone)]
pub struct MirrorTriangleInfo {
    /// The current triangle configuration
    pub current_triangle: Triangle,
    /// The shared edge (2 subcells common to both current and mirror)
    pub shared_edge: [SubCellCoord; 2],
    /// The new diagonal subcell that completes the mirror triangle
    pub new_diagonal: SubCellCoord,
    /// The redundant subcell to release (from current triangle)
    pub redundant: SubCellCoord,
    /// The resulting mirror triangle
    pub mirror_triangle: Triangle,
}

/// Identify the mirror triangle for continuous boundary movement
///
/// Given:
/// - current_psc: The actor's current primary subcell
/// - reserved_diagonal: The diagonal subcell actor is moving toward
/// - anchor: The H/V anchor completing the current triangle
/// - dest_x, dest_y: Destination screen coordinates
///
/// Returns: MirrorTriangleInfo describing the mirror transition, or None if not applicable
pub fn identify_mirror_triangle(
    current_psc: &SubCellCoord,
    reserved_diagonal: &SubCellCoord,
    anchor: &SubCellCoord,
    dest_x: f32,
    dest_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> Option<MirrorTriangleInfo> {
    // Current triangle: (current_psc, reserved_diagonal, anchor)
    let current_triangle = Triangle::new(*current_psc, *reserved_diagonal, *anchor);

    // The shared edge should be the two vertices closest to destination
    // Typically: (reserved_diagonal, anchor) - the edge actor is approaching
    let redundant = current_triangle.find_redundant_for_direction(
        dest_x, dest_y, cell_width, cell_height
    );

    let shared_edge = current_triangle.get_opposite_edge(&redundant)?;

    // Now we need to find the new diagonal that forms a rectangle
    // The mirror triangle shares the edge (reserved_diagonal, anchor)
    // and adds a new diagonal on the opposite side

    // Strategy: The new diagonal is a neighbor of BOTH shared edge vertices
    // that continues in the direction toward destination

    let neighbors_of_first = shared_edge[0].get_neighbors();
    let neighbors_of_second = shared_edge[1].get_neighbors();

    // Find neighbors common to both shared edge vertices (excluding current triangle vertices)
    let mut candidates = Vec::new();
    for n1 in &neighbors_of_first {
        for n2 in &neighbors_of_second {
            if n1 == n2 && !current_triangle.contains_vertex(n1) {
                candidates.push(*n1);
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Among candidates, choose the one that:
    // 1. Is diagonal from at least one of the shared edge vertices
    // 2. Is closest to the destination direction

    let (diag_x, diag_y) = reserved_diagonal.to_screen_center(cell_width, cell_height);
    let dir_to_dest_x = dest_x - diag_x;
    let dir_to_dest_y = dest_y - diag_y;
    let dest_dir_len = (dir_to_dest_x * dir_to_dest_x + dir_to_dest_y * dir_to_dest_y).sqrt();

    if dest_dir_len < 0.001 {
        return None; // At destination
    }

    let norm_dest_x = dir_to_dest_x / dest_dir_len;
    let norm_dest_y = dir_to_dest_y / dest_dir_len;

    // Score each candidate by alignment with destination direction
    let mut best_candidate = candidates[0];
    let mut best_score = -2.0;

    for candidate in &candidates {
        // Check if candidate is diagonal from reserved_diagonal
        let dx_cells = (candidate.cell_x - reserved_diagonal.cell_x).abs();
        let dy_cells = (candidate.cell_y - reserved_diagonal.cell_y).abs();
        let dx_subs = (candidate.sub_x - reserved_diagonal.sub_x).abs();
        let dy_subs = (candidate.sub_y - reserved_diagonal.sub_y).abs();

        let is_diagonal = (dx_cells > 0 || dx_subs > 0) && (dy_cells > 0 || dy_subs > 0);

        if !is_diagonal {
            continue; // Must be diagonal for mirror
        }

        // Calculate alignment score (dot product)
        let score = reserved_diagonal.alignment_score(
            candidate,
            norm_dest_x,
            norm_dest_y,
            cell_width,
            cell_height,
        );

        if score > best_score {
            best_score = score;
            best_candidate = *candidate;
        }
    }

    // Construct the mirror triangle
    // The mirror uses: shared_edge[0], shared_edge[1], new_diagonal
    // We want the new diagonal to become the next PSC, so put it first
    let mirror_triangle = Triangle::new(best_candidate, shared_edge[0], shared_edge[1]);

    Some(MirrorTriangleInfo {
        current_triangle,
        shared_edge,
        new_diagonal: best_candidate,
        redundant,
        mirror_triangle,
    })
}

/// Try to reserve a mirror triangle atomically
///
/// Returns true if mirror reservation succeeded, false if blocked
pub fn try_reserve_mirror(
    mirror_info: &MirrorTriangleInfo,
    actor_id: usize,
    reservation_manager: &mut SubCellReservationManager,
) -> bool {
    // The shared edge should already be reserved by this actor
    // We only need to reserve the new diagonal

    // Verify shared edge is owned by actor (safety check)
    for sc in &mirror_info.shared_edge {
        match reservation_manager.get_owner(sc) {
            Some(owner) if owner == actor_id => { /* OK */ },
            _ => {
                println!("[MIRROR] WARNING: Shared edge {:?} not owned by actor {}", sc, actor_id);
                return false;
            }
        }
    }

    // Try to reserve the new diagonal
    if reservation_manager.try_reserve(mirror_info.new_diagonal, actor_id) {
        println!("[MIRROR] Actor {} reserved mirror diagonal: {:?}",
            actor_id, mirror_info.new_diagonal);
        println!("  Shared edge: {:?}, will release: {:?}",
            mirror_info.shared_edge, mirror_info.redundant);
        true
    } else {
        println!("[MIRROR] Actor {} BLOCKED - cannot reserve mirror diagonal: {:?}",
            actor_id, mirror_info.new_diagonal);
        false
    }
}

/// Release the redundant subcell from the old triangle
pub fn release_redundant(
    redundant: &SubCellCoord,
    actor_id: usize,
    reservation_manager: &mut SubCellReservationManager,
) {
    reservation_manager.release(*redundant, actor_id);
    println!("[MIRROR] Actor {} released redundant: {:?}", actor_id, redundant);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_creation() {
        let v1 = SubCellCoord::new(0, 0, 0, 0, 2);
        let v2 = SubCellCoord::new(0, 0, 1, 1, 2);
        let v3 = SubCellCoord::new(0, 0, 1, 0, 2);

        let triangle = Triangle::new(v1, v2, v3);

        assert_eq!(triangle.psc(), &v1);
        assert!(triangle.contains_vertex(&v2));
        assert!(triangle.contains_vertex(&v3));
    }

    #[test]
    fn test_opposite_edge() {
        let v1 = SubCellCoord::new(0, 0, 0, 0, 2);
        let v2 = SubCellCoord::new(0, 0, 1, 1, 2);
        let v3 = SubCellCoord::new(0, 0, 1, 0, 2);

        let triangle = Triangle::new(v1, v2, v3);

        // Get edge opposite to v1 (should be v2 and v3)
        let edge = triangle.get_opposite_edge(&v1).unwrap();
        assert!(edge.contains(&v2));
        assert!(edge.contains(&v3));
    }

    #[test]
    fn test_mirror_identification() {
        let cell_width = 30.0;
        let cell_height = 30.0;

        // Current triangle: (0,0,0,0) -> (0,0,1,1) with anchor (0,0,1,0)
        let psc = SubCellCoord::new(0, 0, 0, 0, 2);
        let diagonal = SubCellCoord::new(0, 0, 1, 1, 2);
        let anchor = SubCellCoord::new(0, 0, 1, 0, 2);

        // Destination: moving NE
        let dest_x = 100.0;
        let dest_y = 100.0;

        let mirror_info = identify_mirror_triangle(
            &psc, &diagonal, &anchor,
            dest_x, dest_y,
            cell_width, cell_height
        );

        assert!(mirror_info.is_some(), "Should identify mirror triangle");

        let info = mirror_info.unwrap();
        println!("Mirror identified:");
        println!("  Current: {:?}", info.current_triangle.vertices);
        println!("  Shared edge: {:?}", info.shared_edge);
        println!("  New diagonal: {:?}", info.new_diagonal);
        println!("  Redundant: {:?}", info.redundant);
        println!("  Mirror: {:?}", info.mirror_triangle.vertices);
    }
}
