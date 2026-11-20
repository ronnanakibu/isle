//! Zen Breathing Tab (4-7-8 Deep Relaxation Cadence)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreathPhase {
    Inhale,  // 4s
    Hold,    // 7s
    Exhale,  // 8s
}

#[derive(Debug, Clone)]
pub struct ZenTabState {
    pub active: bool,
    pub phase: BreathPhase,
    pub phase_seconds_left: u32,
    pub progress: f32, // 0.0 to 1.0 expansion of the breath circle
    pub completed_cycles: u32,
}

impl Default for ZenTabState {
    fn default() -> Self {
        Self {
            active: false,
            phase: BreathPhase::Inhale,
            phase_seconds_left: 4,
            progress: 0.2,
            completed_cycles: 0,
        }
    }
}

impl ZenTabState {
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if self.active {
            self.phase = BreathPhase::Inhale;
            self.phase_seconds_left = 4;
            self.progress = 0.2;
        }
    }

    pub fn tick_step(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        match self.phase {
            BreathPhase::Inhale => {
                self.progress = (self.progress + dt / 4.0).min(1.0);
                if self.progress >= 1.0 {
                    self.phase = BreathPhase::Hold;
                    self.phase_seconds_left = 7;
                }
            }
            BreathPhase::Hold => {
                // Keep circle at peak
                self.progress = 1.0;
            }
            BreathPhase::Exhale => {
                self.progress = (self.progress - dt / 8.0).max(0.2);
                if self.progress <= 0.2 {
                    self.phase = BreathPhase::Inhale;
                    self.phase_seconds_left = 4;
                    self.completed_cycles += 1;
                }
            }
        }
    }
}
