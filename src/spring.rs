//! Euler-Cromer damped harmonic oscillator.
//!
//! Drives every morph in the island: width, height, opacity, corner radius,
//! pager slide offset. Each spring is independent so different properties can
//! settle at different times, which is what gives the motion its organic feel.

/// A single spring-settling value.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    /// Undamped angular frequency. Higher = snappier.
    pub omega: f32,
    /// Damping ratio. 1.0 = critically damped, <1.0 = overshoots.
    pub zeta: f32,
}

impl Spring {
    pub const fn new(value: f32, omega: f32, zeta: f32) -> Self {
        Self { value, target: value, velocity: 0.0, omega, zeta }
    }

    /// Jump instantly, killing all motion.
    pub fn set(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }

    /// Retarget without touching current value/velocity.
    pub fn target(&mut self, target: f32) {
        self.target = target;
    }

    /// True once the spring is visually at rest.
    pub fn is_settled(&self) -> bool {
        let eps = crate::theme::motion::REST_EPSILON;
        (self.value - self.target).abs() < eps && self.velocity.abs() < eps
    }

    /// Advance the simulation. Returns true while still in motion.
    ///
    /// `dt` is clamped to 1/30s: during a system hitch or a window drag the
    /// real frame delta can spike, and an unclamped Euler step would
    /// inject enough energy to make the spring explode.
    pub fn step(&mut self, dt: f32) -> bool {
        let dt = dt.clamp(0.0, 1.0 / 30.0);
        if self.is_settled() {
            // Snap to target and stop accumulating energy.
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }

        let displacement = self.value - self.target;
        let acceleration =
            -self.omega * self.omega * displacement - 2.0 * self.zeta * self.omega * self.velocity;

        // Semi-implicit (symplectic) Euler: update velocity first, then
        // integrate position with the *new* velocity. More stable than
        // explicit Euler for the same step size.
        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;

        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        true
    }
}

/// A group of springs that must all settle before the app can go idle.
///
/// The window's render timer is killed only when every spring in the group
/// reports settled, which is what drives the app to a true 0.00% CPU state.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct SpringSet<const N: usize> {
    springs: [Spring; N],
}

#[allow(dead_code)]
impl<const N: usize> SpringSet<N> {
    pub const fn new(springs: [Spring; N]) -> Self {
        Self { springs }
    }

    pub fn get(&self, index: usize) -> &Spring {
        &self.springs[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Spring {
        &mut self.springs[index]
    }

    pub fn values(&self) -> [f32; N] {
        let mut out = [0.0; N];
        let mut i = 0;
        while i < N {
            out[i] = self.springs[i].value;
            i += 1;
        }
        out
    }

    /// Step every spring. Returns true if any is still moving.
    pub fn step(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for s in &mut self.springs {
            if s.step(dt) {
                moving = true;
            }
        }
        moving
    }

    pub fn is_settled(&self) -> bool {
        self.springs.iter().all(|s| s.is_settled())
    }

    pub fn set_all(&mut self, value: f32) {
        for s in &mut self.springs {
            s.set(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settles_at_target() {
        let mut s = Spring::new(0.0, 16.0, 0.72);
        s.target = 100.0;
        let mut t = 0.0;
        for _ in 0..600 {
            s.step(1.0 / 60.0);
            t += 1.0 / 60.0;
            if !s.is_settled() {
                assert!(t < 5.0, "spring failed to settle in time");
            }
        }
        assert!(s.is_settled());
        assert!((s.value - 100.0).abs() < 0.5);
    }

    #[test]
    fn overshoots_with_underdamping() {
        let mut s = Spring::new(0.0, 16.0, 0.4);
        s.target = 100.0;
        let mut peak: f32 = 0.0;
        for _ in 0..600 {
            s.step(1.0 / 60.0);
            peak = peak.max(s.value);
        }
        assert!(peak > 100.5, "underdamped spring should overshoot, peak={peak}");
    }

    #[test]
    fn huge_dt_does_not_explode() {
        let mut s = Spring::new(0.0, 16.0, 0.72);
        s.target = 100.0;
        s.step(10.0);
        assert!(s.value.is_finite());
        assert!(s.value.abs() < 10_000.0);
    }
}
