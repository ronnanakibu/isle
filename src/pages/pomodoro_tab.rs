//! Focus / Pomodoro Timer Tab

#[derive(Debug, Clone)]
pub struct PomodoroTabState {
    pub seconds_left: u32,
    pub running: bool,
    pub mode: usize, // 0 = 25m work, 1 = 5m short break, 2 = 15m long break
    pub completed_sessions: u32,
}

impl Default for PomodoroTabState {
    fn default() -> Self {
        Self {
            seconds_left: 25 * 60,
            running: false,
            mode: 0,
            completed_sessions: 0,
        }
    }
}

impl PomodoroTabState {
    pub fn toggle(&mut self) {
        self.running = !self.running;
    }

    pub fn set_mode(&mut self, mode: usize) {
        self.mode = mode;
        self.seconds_left = match mode {
            0 => 25 * 60,
            1 => 5 * 60,
            2 => 15 * 60,
            _ => 25 * 60,
        };
        self.running = false;
    }

    pub fn reset(&mut self) {
        self.set_mode(self.mode);
    }

    pub fn tick(&mut self) {
        if self.running && self.seconds_left > 0 {
            self.seconds_left -= 1;
            if self.seconds_left == 0 {
                self.running = false;
                if self.mode == 0 {
                    self.completed_sessions += 1;
                }
            }
        }
    }
}
