//! Gesture Pager: Drag, Flick, and Apple-grade Rubber-Band Physics
use crate::spring::Spring;
use std::time::Instant;

pub const PAGE_COUNT: usize = 5;
const SLOP_PX: f32 = 6.0;
const FLICK_PROJECTION_S: f32 = 0.12;

struct Drag {
    start_x: f32,
    last_x: f32,
    last_t: Instant,
    vel: f32,
    base: f32,
    moved: bool,
}

pub enum Release {
    Click,
    Swiped { from: usize, to: usize },
}

pub struct Pager {
    pub page: usize,
    pub x: Spring,
    page_w: f32,
    drag: Option<Drag>,
}

fn rubber(over: f32, dim: f32) -> f32 {
    (1.0 - 1.0 / (over.abs() * 0.55 / dim + 1.0)) * dim * over.signum()
}

impl Pager {
    pub fn new(page_w: f32) -> Self {
        Self {
            page: 0,
            page_w,
            drag: None,
            x: Spring {
                current: 0.0,
                target: 0.0,
                velocity: 0.0,
                omega: 16.0,
                zeta: 0.72,
            },
        }
    }

    pub fn content_x(&self) -> f32 {
        self.x.current
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.as_ref().map_or(false, |d| d.moved)
    }

    pub fn is_moving(&self) -> bool {
        self.is_dragging()
            || (self.x.current - self.x.target).abs() > 0.5
            || self.x.velocity.abs() > 0.5
    }

    pub fn press(&mut self, x: f32) {
        self.drag = Some(Drag {
            start_x: x,
            last_x: x,
            last_t: Instant::now(),
            vel: 0.0,
            base: self.x.current,
            moved: false,
        });
    }

    pub fn drag(&mut self, x: f32) -> bool {
        let min_x = -((PAGE_COUNT - 1) as f32) * self.page_w;
        let w = self.page_w;
        let Some(d) = self.drag.as_mut() else {
            return false;
        };
        let dx = x - d.start_x;
        if !d.moved && dx.abs() < SLOP_PX {
            return false;
        }
        d.moved = true;

        let now = Instant::now();
        let dt = now.duration_since(d.last_t).as_secs_f32().max(0.001);
        d.vel = 0.6 * d.vel + 0.4 * ((x - d.last_x) / dt);
        d.last_x = x;
        d.last_t = now;

        let raw = d.base + dx;
        let pos = if raw > 0.0 {
            rubber(raw, w)
        } else if raw < min_x {
            min_x + rubber(raw - min_x, w)
        } else {
            raw
        };
        self.x.current = pos;
        self.x.target = pos;
        self.x.velocity = 0.0;
        true
    }

    pub fn release(&mut self) -> Release {
        let Some(d) = self.drag.take() else {
            return Release::Click;
        };
        if !d.moved {
            return Release::Click;
        }
        let vel = if d.last_t.elapsed().as_secs_f32() > 0.08 {
            0.0
        } else {
            d.vel
        };
        let projected = self.x.current + vel * FLICK_PROJECTION_S;
        let from = self.page;
        let to = (((-projected / self.page_w).round().max(0.0)) as usize)
            .clamp(from.saturating_sub(1), (from + 1).min(PAGE_COUNT - 1));
        self.page = to;
        self.x.target = -(to as f32) * self.page_w;
        self.x.velocity = vel;
        Release::Swiped { from, to }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.x.update(dt.min(1.0 / 30.0))
    }

    pub fn jump_to(&mut self, target_page: usize) {
        if target_page < PAGE_COUNT {
            self.page = target_page;
            self.x.set_target(-(target_page as f32) * self.page_w);
        }
    }
}
