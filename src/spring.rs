//! Apple-Style Analytical Spring Physics
//! Implements damping ratio (zeta) and natural frequency (omega) matching iOS UIKit/SwiftUI withAnimation(.spring)

#[derive(Debug, Clone)]
pub struct Spring {
    pub current: f32,
    pub target: f32,
    pub velocity: f32,
    pub omega: f32, // Natural frequency (rad/s) ~ 15.0 for snappy Apple response
    pub zeta: f32,  // Damping ratio ~ 0.72 for slight Apple overshoot
}

impl Spring {
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            velocity: 0.0,
            omega: 16.0,
            zeta: 0.72,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Step spring physics by dt seconds (Euler-Cromer integration for energy conservation)
    /// Returns true if still animating, false if settled at target.
    pub fn update(&mut self, dt: f32) -> bool {
        let displacement = self.current - self.target;

        // a = -omega^2 * x - 2 * zeta * omega * v
        let spring_acc = -self.omega * self.omega * displacement;
        let damping_acc = -2.0 * self.zeta * self.omega * self.velocity;
        let acceleration = spring_acc + damping_acc;

        self.velocity += acceleration * dt;
        self.current += self.velocity * dt;

        // Check resting condition
        if (self.current - self.target).abs() < 0.25 && self.velocity.abs() < 0.3 {
            self.current = self.target;
            self.velocity = 0.0;
            false
        } else {
            true
        }
    }
}

/// Cubic Hermite Smoothstep S(t) = 3t^2 - 2t^3
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
