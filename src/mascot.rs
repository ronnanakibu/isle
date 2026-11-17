//! Living Mascot Engine
//! Procedural eye animations, pupil cursor-tracking, and blinking

#[derive(Debug, Clone)]
pub struct Mascot {
    pub eye_radius: f32,
    pub pupil_radius: f32,
    pub eye_spacing: f32,
    pub blink_progress: f32, // 1.0 = fully open, 0.05 = closed
    pub is_blinking: bool,
    pub blink_timer: f32,
    pub pupil_offset_x: f32,
    pub pupil_offset_y: f32,
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
            pupil_offset_x: 0.0,
            pupil_offset_y: 0.0,
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

    /// Update blinking animation (dt in seconds)
    pub fn update_blink(&mut self, dt: f32) -> bool {
        self.blink_timer += dt;
        if !self.is_blinking {
            // Trigger blink every ~3.5 seconds
            if self.blink_timer >= 3.5 {
                self.is_blinking = true;
                self.blink_timer = 0.0;
            }
            false
        } else {
            // Blink cycle takes ~0.22 seconds
            let t = self.blink_timer / 0.22;
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
            }
            true
        }
    }
}
