//! The living-notch mascot.
//!
//! A procedural character that lives inside the island. It has two eyes that
//! track the cursor with a soft leash, blinks on a randomized cadence, and
//! drifts through idle "moods" (watching, resting, sleepy) depending on how
//! long it has been without attention.

use std::f32::consts::PI;

/// Maximum pupil displacement from the eye center, in pixels.
pub const MAX_PUPIL_TRAVEL: f32 = 3.4;

/// How strongly the pupil is attracted to the cursor (0..1).
pub const GAZE_STRENGTH: f32 = 0.16;

/// The mascot's mood, which drives pupil behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    /// Awake and following the cursor.
    Watching,
    /// Cursor gone for a while; pupils drift gently.
    Resting,
    /// Very long idle; eyes nearly closed.
    Sleepy,
}

/// Procedural blink state machine.
#[derive(Clone, Copy, Debug)]
pub struct Blink {
    /// Seconds until the next blink.
    pub next_in: f32,
    /// 0 = fully open, 1 = fully closed.
    pub closure: f32,
    /// Total duration of the current blink, in seconds.
    pub duration: f32,
    /// Elapsed time within the current blink.
    pub elapsed: f32,
    pub active: bool,
}

impl Blink {
    pub fn new() -> Self {
        // First blink comes a little after start-up so it doesn't fire
        // immediately on launch.
        Self { next_in: 2.2, closure: 0.0, duration: 0.0, elapsed: 0.0, active: false }
    }

    /// Advance the blink timer. Returns true if the state changed (i.e. the
    /// renderer needs to redraw).
    pub fn update(&mut self, dt: f32, rand: &mut u32) -> bool {
        if self.active {
            self.elapsed += dt;
            let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
            // Fast close, slower open — reads as a natural blink.
            self.closure = if t < 0.4 { t / 0.4 } else { 1.0 - (t - 0.4) / 0.6 };
            if t >= 1.0 {
                self.active = false;
                self.closure = 0.0;
                self.next_in = 3.5 + rand_next(rand) * 2.5;
                return true;
            }
            return true;
        }

        self.next_in -= dt;
        if self.next_in <= 0.0 {
            self.active = true;
            self.elapsed = 0.0;
            self.duration = 0.09 + rand_next(rand) * 0.05;
            self.closure = 0.0;
            return true;
        }
        false
    }
}

/// Deterministic pseudo-random helper, so the mascot behaves reproducibly
/// for a given session rather than depending on an RNG crate.
///
/// Xorshift32 over a `u32` state.
pub fn rand_next(state: &mut u32) -> f32 {
    let mut x = *state;
    if x == 0 {
        x = 0x9E37_79B9;
    }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    // Map the full u32 range onto 0..1.
    (x >> 8) as f32 / (u32::MAX >> 8) as f32
}

/// Eye gaze state.
#[derive(Clone, Copy, Debug)]
pub struct Eyes {
    /// Pupil offset from the eye center, in pixels.
    pub pupil: (f32, f32),
    /// Current mood.
    pub mood: Mood,
    /// Seconds since the cursor last moved.
    pub idle_time: f32,
}

impl Eyes {
    pub fn new() -> Self {
        Self { pupil: (0.0, 0.0), mood: Mood::Resting, idle_time: 0.0 }
    }

    /// Recompute pupil position and mood.
    ///
    /// `cursor` is the cursor position in *screen* coordinates; `origin` is
    /// the mascot's eye origin in screen coordinates. When the cursor is
    /// absent, the pupil drifts back to center.
    pub fn update(
        &mut self,
        dt: f32,
        cursor: Option<(f32, f32)>,
        origin: (f32, f32),
        blink: &Blink,
    ) -> bool {
        let mut changed = false;

        match cursor {
            Some((mx, my)) => {
                let dx = mx - origin.0;
                let dy = my - origin.1;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 0.001 {
                    // Leashed gaze: the further away the cursor, the closer to
                    // the leash limit — but the travel is capped.
                    let travel = (dist * GAZE_STRENGTH).min(MAX_PUPIL_TRAVEL);
                    let tx = (dx / dist) * travel;
                    let ty = (dy / dist) * travel;
                    changed |= self.approach(tx, ty, dt);
                }
                self.idle_time = 0.0;
                if self.mood != Mood::Watching {
                    self.mood = Mood::Watching;
                    changed = true;
                }
            }
            None => {
                self.idle_time += dt;
                // Drift back toward center, with a slow wander once resting.
                let (tx, ty) = if self.idle_time > 12.0 {
                    self.mood = Mood::Sleepy;
                    (0.0, 0.0)
                } else if self.idle_time > 3.0 {
                    self.mood = Mood::Resting;
                    // A lazy figure-eight drift.
                    let t = self.idle_time * 0.6;
                    ((t.sin() * 1.6), (t.cos() * 2.0 * t.sin()).sin() * 0.6)
                } else {
                    (0.0, 0.0)
                };
                changed |= self.approach(tx, ty, dt);
                if self.mood != Mood::Resting && self.mood != Mood::Sleepy {
                    self.mood = Mood::Resting;
                    changed = true;
                }
            }
        }

        // A sleeping mascot keeps its eyes shut regardless of the cursor.
        if blink.closure > 0.0 {
            changed = true;
        }
        changed
    }

    /// Move the pupil toward a target with a critically-damped feel.
    fn approach(&mut self, tx: f32, ty: f32, dt: f32) -> bool {
        const RATE: f32 = 11.0;
        let k = (RATE * dt).min(1.0);
        let nx = self.pupil.0 + (tx - self.pupil.0) * k;
        let ny = self.pupil.1 + (ty - self.pupil.1) * k;
        let moved = (nx - self.pupil.0).abs() > 0.01 || (ny - self.pupil.1).abs() > 0.01;
        self.pupil = (nx, ny);
        moved
    }

    /// Vertical scale of the eye opening, 0 (shut) .. 1 (wide).
    pub fn aperture(&self, blink: &Blink) -> f32 {
        let base = match self.mood {
            Mood::Sleepy => 0.55,
            Mood::Resting => 0.85,
            Mood::Watching => 1.0,
        };
        base * (1.0 - blink.closure)
    }
}

impl Default for Eyes {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Blink {
    fn default() -> Self {
        Self::new()
    }
}

/// Geometry of the mascot's face, in local coordinates relative to the
/// island's center.
#[derive(Clone, Copy, Debug)]
pub struct MascotGeometry {
    /// Distance between the two eye centers.
    pub eye_spacing: f32,
    /// Eye radius.
    pub eye_radius: f32,
    /// Pupil radius.
    pub pupil_radius: f32,
}

impl Default for MascotGeometry {
    fn default() -> Self {
        Self { eye_spacing: 13.0, eye_radius: 5.4, pupil_radius: 2.5 }
    }
}

impl MascotGeometry {
    /// Left eye center, relative to the mascot origin.
    pub fn left_eye(&self) -> (f32, f32) {
        (-self.eye_spacing * 0.5, 0.0)
    }

    /// Right eye center, relative to the mascot origin.
    pub fn right_eye(&self) -> (f32, f32) {
        (self.eye_spacing * 0.5, 0.0)
    }
}

/// Draw the mascot's eyes into the canvas.
///
/// The eyes are the island's "living" signal — they should read as glassy
/// domes rather than flat dots, so each eye gets a specular highlight.
pub fn draw_mascot(
    canvas: &mut crate::render::Canvas,
    origin: (f32, f32),
    geom: &MascotGeometry,
    eyes: &Eyes,
    blink: &Blink,
    scale: f32,
) {
    use crate::render::Rgba;

    let aperture = eyes.aperture(blink);
    let base = Rgba::rgb(238, 242, 248);
    let pupil_color = Rgba::rgb(10, 11, 14);

    for eye in [geom.left_eye(), geom.right_eye()] {
        let (ex, ey) = (origin.0 + eye.0, origin.1 + eye.1);
        let r = geom.eye_radius * scale;

        if aperture <= 0.06 {
            // Closed: draw a soft downward arc suggesting a squinted lid.
            canvas.fill_circle(ex, ey, r * 0.85, Rgba::rgba(150, 156, 168, 140));
            continue;
        }

        // Eye white, vertically squashed by the aperture.
        let eye_h = r * (0.55 + 0.45 * aperture);
        for py in (ey - r - 1.0).floor() as i32..=(ey + r + 1.0).ceil() as i32 {
            for px in (ex - r - 1.0).floor() as i32..=(ex + r + 1.0).ceil() as i32 {
                let dx = px as f32 + 0.5 - ex;
                let dy = (py as f32 + 0.5 - ey) / eye_h.max(0.01);
                let d = (dx * dx + dy * dy).sqrt() * eye_h - r;
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let mut c = base;
                c.a = (base.a as f32 * cov).round() as u8;
                canvas.blend(px, py, c);
            }
        }

        // Pupil, offset by gaze.
        let pr = geom.pupil_radius * scale * (0.7 + 0.3 * aperture);
        let px = ex + eyes.pupil.0 * scale;
        let py = ey + eyes.pupil.1 * scale;
        canvas.fill_circle(px, py, pr, pupil_color);

        // Specular highlight — upper-left, as if lit from above.
        canvas.fill_circle(
            px - pr * 0.42,
            py - pr * 0.52,
            pr * 0.36,
            Rgba::rgba(255, 255, 255, 210),
        );
        canvas.fill_circle(
            px + pr * 0.38,
            py + pr * 0.45,
            pr * 0.18,
            Rgba::rgba(255, 255, 255, 90),
        );
    }
}

/// A gentle "wave" animation used when the island expands — the mascot
/// acknowledges the user.
#[allow(dead_code)]
pub fn wave_phase(elapsed: f32) -> f32 {
    (elapsed * 2.4).sin() * 0.5 + 0.5
}

/// Angle of the mascot's gaze direction, for rendering the highlight offset.
#[allow(dead_code)]
pub fn gaze_angle(eyes: &Eyes) -> f32 {
    (eyes.pupil.1).atan2(eyes.pupil.0)
}

/// Cosine of the gaze angle, used to bias the specular highlight.
#[allow(dead_code)]
pub fn gaze_cos(eyes: &Eyes) -> f32 {
    gaze_angle(eyes).cos()
}

/// Sine of the gaze angle.
#[allow(dead_code)]
pub fn gaze_sin(eyes: &Eyes) -> f32 {
    gaze_angle(eyes).sin()
}

/// Full rotation of a radial gradient, used by the water tracker.
#[allow(dead_code)]
pub fn radial_phase(t: f32) -> f32 {
    (t * 0.5).rem_euclid(2.0 * PI)
}
