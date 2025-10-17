/// Ray state for raycasting calculations
/// Tracks the trajectory of a ray as it expands outward from the observer
#[derive(Debug, Clone, Copy)]
pub struct RayState {
    /// Horizontal component of ray vector (always >= 0)
    pub diff_x: i32,
    /// Vertical component of ray vector (always >= 0, never 0 during raycasting)
    pub diff_y: i32,
    /// Ray advancement parameter: adjusts ray convergence/divergence
    /// Starts at -1 (convergent), increments each row
    pub y_step: i32,
    /// Integer rounding adjustment for conservative occlusion
    /// 0 for convergent rays, diff_y-1 for divergent rays
    pub rounding: i32,
}

impl RayState {
    /// Create a new ray state
    pub fn new(diff_x: i32, diff_y: i32, y_step: i32, rounding: i32) -> Self {
        RayState {
            diff_x,
            diff_y,
            y_step,
            rounding,
        }
    }

    /// Calculate the ray border position using integer arithmetic
    /// Formula: ((diff_y + y_step) * diff_x - rounding) / diff_y
    ///
    /// This formula determines how far the ray extends horizontally at the current row.
    /// All arithmetic is integer-based to avoid floating-point precision issues.
    pub fn calculate_border(&self) -> i32 {
        if self.diff_y == 0 {
            return 0; // Should never happen during raycasting, but safe default
        }
        ((self.diff_y + self.y_step) * self.diff_x - self.rounding) / self.diff_y
    }

    /// Increment the y_step parameter (call this when advancing to next row)
    pub fn increment_y_step(&mut self) {
        self.y_step += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_border_calculation() {
        // Test convergent ray (yStep = -1)
        let ray = RayState::new(5, 1, -1, 0);
        assert_eq!(ray.calculate_border(), 0); // ((1 + -1) * 5 - 0) / 1 = 0

        // Test after one increment
        let mut ray2 = ray;
        ray2.increment_y_step(); // y_step becomes 0
        assert_eq!(ray2.calculate_border(), 5); // ((1 + 0) * 5 - 0) / 1 = 5

        // Test after another increment
        ray2.increment_y_step(); // y_step becomes 1
        assert_eq!(ray2.calculate_border(), 10); // ((1 + 1) * 5 - 0) / 1 = 10
    }

    #[test]
    fn test_ray_with_rounding() {
        // Test divergent ray with rounding
        let ray = RayState::new(5, 2, 1, 1);
        assert_eq!(ray.calculate_border(), 7); // ((2 + 1) * 5 - 1) / 2 = 14 / 2 = 7
    }
}
