//! Horizontal pager gesture engine and spring-driven tab transitions.
//!
//! Tracks pointer press/drag/flick gestures across the expanded card's pages,
//! applies logarithmic rubber-band resistance past the first and last page,
//! and settles onto the target page with a damped spring.

use crate::spring::Spring;
use crate::theme;

pub const PAGE_COUNT: usize = 5;
pub const TAB_NAMES: [&str; PAGE_COUNT] = ["Media", "Focus", "Water", "Tasks", "Files"];

/// Minimum horizontal pointer travel (in pixels) before a press becomes a drag.
const DRAG_SLOP_PX: f32 = 6.0;

/// Flick velocity threshold (pixels/second) to advance/retreat a page.
const FLICK_VELOCITY_PX_S: f32 = 400.0;

/// Displacement fraction of `card_width` required to commit a slow drag.
const COMMIT_FRACTION: f32 = 0.35;

pub struct Pager {
    pub active: usize,
    pub scroll_x: Spring,
    pub card_width: f32,
    pub pressing: bool,
    pub dragging: bool,
    start_x: i32,
    start_time_ms: u64,
    start_scroll: f32,
}

impl Pager {
    pub fn new() -> Self {
        Self {
            active: 0,
            scroll_x: Spring::new(0.0, theme::motion::PAGER_OMEGA, theme::motion::PAGER_ZETA),
            card_width: theme::EXPANDED_W,
            pressing: false,
            dragging: false,
            start_x: 0,
            start_time_ms: 0,
            start_scroll: 0.0,
        }
    }

    /// Continuous page index in `0.0..=(PAGE_COUNT - 1) as f32` for pagination dots.
    pub fn page_pos(&self) -> f32 {
        (self.scroll_x.value / self.card_width.max(1.0)).clamp(0.0, (PAGE_COUNT - 1) as f32)
    }

    /// Capture initial press coordinate and timestamp.
    pub fn begin_press(&mut self, x: i32, now_ms: u64) {
        self.pressing = true;
        self.dragging = false;
        self.start_x = x;
        self.start_time_ms = now_ms;
        self.start_scroll = self.scroll_x.value;
    }

    /// Update drag offset with rubber-band resistance at the edges.
    /// Returns `true` if the pager is actively dragging.
    pub fn update_drag(&mut self, x: i32) -> bool {
        if !self.pressing {
            return false;
        }
        let dx = (x - self.start_x) as f32;
        if !self.dragging && dx.abs() > DRAG_SLOP_PX {
            self.dragging = true;
        }
        if self.dragging {
            let raw = self.start_scroll - dx;
            let max_scroll = (PAGE_COUNT - 1) as f32 * self.card_width;
            let clamped = if raw < 0.0 {
                theme::rubber_band(raw, self.card_width)
            } else if raw > max_scroll {
                max_scroll + theme::rubber_band(raw - max_scroll, self.card_width)
            } else {
                raw
            };
            self.scroll_x.set(clamped);
        }
        self.dragging
    }

    /// Finish a press/drag gesture. Returns `true` if it was a drag (so callers
    /// suppress click dispatch), or `false` if it was a tap/click.
    pub fn end_press(&mut self, x: i32, now_ms: u64) -> bool {
        let was_drag = self.dragging;
        self.pressing = false;
        self.dragging = false;

        if was_drag {
            let dx = (x - self.start_x) as f32;
            let dt = ((now_ms.saturating_sub(self.start_time_ms)) as f32 / 1000.0).max(0.001);
            let velocity = dx / dt;
            let threshold = COMMIT_FRACTION * self.card_width;

            if velocity < -FLICK_VELOCITY_PX_S || dx < -threshold {
                self.active = (self.active + 1).min(PAGE_COUNT - 1);
            } else if velocity > FLICK_VELOCITY_PX_S || dx > threshold {
                self.active = self.active.saturating_sub(1);
            }

            self.scroll_x.target(self.active as f32 * self.card_width);
            self.scroll_x.velocity = (-velocity).clamp(-2200.0, 2200.0);
        }

        was_drag
    }

    /// Cancel an in-progress drag (e.g. capture lost) and spring back.
    pub fn cancel_drag(&mut self) {
        self.pressing = false;
        self.dragging = false;
        self.scroll_x.target(self.active as f32 * self.card_width);
    }

    /// Jump smoothly to a specific tab index.
    pub fn select(&mut self, index: usize) {
        self.active = index.min(PAGE_COUNT - 1);
        self.scroll_x.target(self.active as f32 * self.card_width);
    }

    /// Advance the pager spring. Returns `true` while still moving.
    pub fn step(&mut self, dt: f32) -> bool {
        if self.dragging {
            return false;
        }
        self.scroll_x.step(dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubber_band_bounds_overscroll() {
        let mut p = Pager::new();
        p.begin_press(100, 1000);
        // Drag far to the right while already on page 0.
        assert!(p.update_drag(100 + 2000));
        assert!(p.scroll_x.value < 0.0);
        assert!(p.scroll_x.value > -p.card_width);
    }

    #[test]
    fn flick_advances_page() {
        let mut p = Pager::new();
        p.begin_press(300, 1000);
        p.update_drag(220); // -80 px
        let was_drag = p.end_press(220, 1100); // 100 ms -> -800 px/s
        assert!(was_drag);
        assert_eq!(p.active, 1);
        assert_eq!(p.scroll_x.target, p.card_width);
    }

    #[test]
    fn slow_large_drag_advances_page() {
        let mut p = Pager::new();
        p.begin_press(400, 1000);
        let dx = (p.card_width * 0.40) as i32;
        p.update_drag(400 - dx);
        let was_drag = p.end_press(400 - dx, 3000); // slow 2s drag
        assert!(was_drag);
        assert_eq!(p.active, 1);
    }

    #[test]
    fn small_slow_drag_springs_back() {
        let mut p = Pager::new();
        p.begin_press(400, 1000);
        p.update_drag(350); // -50 px
        let was_drag = p.end_press(350, 2000); // -50 px/s
        assert!(was_drag);
        assert_eq!(p.active, 0);
        assert_eq!(p.scroll_x.target, 0.0);
    }

    #[test]
    fn tiny_jitter_is_treated_as_click() {
        let mut p = Pager::new();
        p.begin_press(400, 1000);
        assert!(!p.update_drag(403));
        let was_drag = p.end_press(403, 1050);
        assert!(!was_drag);
        assert_eq!(p.active, 0);
    }
}
