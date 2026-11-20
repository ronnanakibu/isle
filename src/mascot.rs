//! Living Mascot Engine
//! Procedural eye animations, pupil cursor-tracking, burst blinking, and ThinkPad F4 mic mute tape cross ( · 🗙 · )

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotMood {
    Normal,
    Music,
    Happy,
    Zen,
    Muted,
}

#[derive(Debug, Clone)]
pub struct Mascot {
    pub eye_radius: f32,
    pub pupil_radius: f32,
    pub eye_spacing: f32,
    pub blink_progress: f32, // 1.0 = fully open, 0.05 = closed
    pub is_blinking: bool,
    pub blink_timer: f32,
    pub burst_remaining: u8,
    pub next_blink_interval: f32,
    pub pupil_offset_x: f32,
    pub pupil_offset_y: f32,
    pub mic_muted: bool,
    pub mood: MascotMood,
}

impl Mascot {
    pub fn new() -> Self {
        Self {
            eye_radius: 4.5,
            pupil_radius: 2.2,
            eye_spacing: 13.0,
            blink_progress: 1.0,
            is_blinking: false,
            blink_timer: 0.0,
            burst_remaining: 0,
            next_blink_interval: 3.5,
            pupil_offset_x: 0.0,
            pupil_offset_y: 0.0,
            mic_muted: false,
            mood: MascotMood::Normal,
        }
    }

    pub fn set_mic_muted(&mut self, muted: bool) {
        self.mic_muted = muted;
        if muted {
            self.mood = MascotMood::Muted;
        } else if self.mood == MascotMood::Muted {
            self.mood = MascotMood::Normal;
        }
    }

    /// Update gaze towards mouse cursor relative to mascot center
    pub fn update_gaze(&mut self, rel_mx: f32, rel_my: f32) {
        let dist = (rel_mx * rel_mx + rel_my * rel_my).sqrt();
        let max_offset = self.eye_radius - self.pupil_radius - 0.5;
        if dist > 0.01 {
            let factor = (dist / 120.0).clamp(0.0, 1.0);
            self.pupil_offset_x = (rel_mx / dist) * max_offset * factor;
            self.pupil_offset_y = (rel_my / dist) * max_offset * factor;
        } else {
            self.pupil_offset_x = 0.0;
            self.pupil_offset_y = 0.0;
        }
    }

    /// Update blinking animation (dt in seconds) with natural blink bursts
    pub fn update_blink(&mut self, dt: f32) -> bool {
        self.blink_timer += dt;
        if !self.is_blinking {
            if self.blink_timer >= self.next_blink_interval {
                self.is_blinking = true;
                self.blink_timer = 0.0;
            }
            false
        } else {
            // Blink cycle takes ~0.20 seconds
            let duration = 0.20;
            let t = self.blink_timer / duration;
            if t < 0.5 {
                // Closing
                self.blink_progress = 1.0 - (t * 2.0) * 0.95;
            } else if t < 1.0 {
                // Opening
                self.blink_progress = 0.05 + ((t - 0.5) * 2.0) * 0.95;
            } else {
                self.is_blinking = false;
                self.blink_progress = 1.0;
                self.blink_timer = 0.0;

                // Handle burst blinking (e.g. quick double-blink)
                if self.burst_remaining > 0 {
                    self.burst_remaining -= 1;
                    self.next_blink_interval = 0.12; // Rapid burst re-trigger
                } else {
                    // Seed next burst: 30% chance of double-blink burst
                    let pseudo_rand = ((self.pupil_offset_x * 100.0).abs() as u32 + 7) % 10;
                    if pseudo_rand < 3 {
                        self.burst_remaining = 1;
                    }
                    self.next_blink_interval = 2.8 + (pseudo_rand as f32) * 0.35;
                }
            }
            true
        }
    }
}
