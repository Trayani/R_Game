use crate::ray::RayState;

/// Cone represents a visibility cone during raycasting
/// A cone is defined by two rays (left and right) that expand outward from the observer
/// and track the walkable bounds from the previous row
#[derive(Debug, Clone)]
pub struct Cone {
    /// Left boundary ray
    pub ray_left: RayState,
    /// Right boundary ray
    pub ray_right: RayState,
    /// Left X bound from previous row's walkable segment
    pub prev_line_start_x: i32,
    /// Right X bound from previous row's walkable segment
    pub prev_line_end_x: i32,
    /// Whether this cone is still active (not collapsed or blocked)
    pub is_active: bool,
}

impl Cone {
    /// Create a new cone
    pub fn new(
        ray_left: RayState,
        ray_right: RayState,
        prev_line_start_x: i32,
        prev_line_end_x: i32,
    ) -> Self {
        Cone {
            ray_left,
            ray_right,
            prev_line_start_x,
            prev_line_end_x,
            is_active: true,
        }
    }

    /// Clone this cone (needed for cone splitting)
    pub fn clone_cone(&self) -> Self {
        Self {
            ray_left: self.ray_left,
            ray_right: self.ray_right,
            prev_line_start_x: self.prev_line_start_x,
            prev_line_end_x: self.prev_line_end_x,
            is_active: self.is_active,
        }
    }
}
