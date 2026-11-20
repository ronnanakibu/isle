//! Hydration Tracking Tab

#[derive(Debug, Clone)]
pub struct HydrationTabState {
    pub intake_ml: u32,
    pub goal_ml: u32,
    pub streak_days: u32,
}

impl Default for HydrationTabState {
    fn default() -> Self {
        Self {
            intake_ml: 1000,
            goal_ml: 2000,
            streak_days: 3,
        }
    }
}

impl HydrationTabState {
    pub fn add(&mut self, ml: u32) {
        self.intake_ml = (self.intake_ml + ml).min(self.goal_ml * 2);
    }

    pub fn reset(&mut self) {
        self.intake_ml = 0;
    }

    pub fn progress_pct(&self) -> f32 {
        if self.goal_ml > 0 {
            ((self.intake_ml as f32 / self.goal_ml as f32) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }
}
