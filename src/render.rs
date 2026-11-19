//! Software rasterizer for Isle's canvas.
//!
//! The island is drawn into a 32-bit **premultiplied BGRA** buffer, which is
//! exactly what `UpdateLayeredWindow`/`ULW_ALPHA` expects. Everything here is
//! CPU-side: shapes are evaluated as signed distance fields so edges are
//! analytically antialiased, and alpha compositing happens in premultiplied
//! space to avoid halos around translucent edges.
//!
//! ## Why superellipses
//!
//! A circular corner radius creates a *tangent break*: curvature jumps from
//! 1/r at the corner to 0 on the straight edge. Apple's hardware curves avoid
//! this by using a superellipse (Lamé curve) whose curvature decays smoothly
//! to zero at the extremes. [`RoundRect`] evaluates an Lp-norm distance field
//! with exponent `n >= 4`, which reproduces that behaviour: continuity of
//! curvature at the straight-edge tangent points.

use std::f32::consts::PI;

/// Straight (non-premultiplied) 8-bit RGBA color.
#[derive(Clone, Copy, Debug)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Linear interpolation in straight-alpha space.
    pub fn lerp(self, other: Rgba, t: f32) -> Rgba {
        let t = t.clamp(0.0, 1.0);
        let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
        Rgba {
            r: m(self.r, other.r),
            g: m(self.g, other.g),
            b: m(self.b, other.b),
            a: m(self.a, other.a),
        }
    }
}

/// Axis-aligned floating-point rect.
#[derive(Clone, Copy, Debug)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectF {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    pub fn center_x(&self) -> f32 {
        self.x + self.w * 0.5
    }
    pub fn center_y(&self) -> f32 {
        self.y + self.h * 0.5
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }
}

/// Rounded rectangle with independently controllable corner radii and a
/// superellipse (Lp-norm) corner profile.
///
/// Radii are ordered `[top-left, top-right, bottom-right, bottom-left]`.
/// A radius of `0` yields a mathematically sharp corner.
#[derive(Clone, Copy, Debug)]
pub struct RoundRect {
    pub center_x: f32,
    pub center_y: f32,
    pub half_w: f32,
    pub half_h: f32,
    pub radii: [f32; 4],
    /// Lp-norm exponent. 2 = circular, 4+ = Apple-style squircle.
    pub exponent: f32,
}

impl RoundRect {
    pub fn new(center_x: f32, center_y: f32, half_w: f32, half_h: f32, radii: [f32; 4], exponent: f32) -> Self {
        let limit = half_w.min(half_h);
        let radii = [
            radii[0].min(limit),
            radii[1].min(limit),
            radii[2].min(limit),
            radii[3].min(limit),
        ];
        Self { center_x, center_y, half_w, half_h, radii, exponent }
    }

    /// Radius for the quadrant containing `(px, py)` relative to the center.
    #[inline]
    fn radius_for(&self, px: f32, py: f32) -> f32 {
        let sx = if px < 0.0 { 0 } else { 1 };
        let sy = if py < 0.0 { 0 } else { 1 };
        // tl=0, tr=1, br=2, bl=3
        self.radii[if sy == 0 { sx } else { 3 - sx }]
    }

    /// Signed distance to the shape. Negative inside, positive outside.
    ///
    /// Uses the classic rounded-box field with the Euclidean corner length
    /// replaced by an Lp-norm, so the corner profile becomes a superellipse.
    pub fn distance(&self, x: f32, y: f32) -> f32 {
        let px = x - self.center_x;
        let py = y - self.center_y;
        let r = self.radius_for(px, py);

        let qx = px.abs() - (self.half_w - r);
        let qy = py.abs() - (self.half_h - r);

        let ox = qx.max(0.0);
        let oy = qy.max(0.0);
        let corner = lnorm(ox, oy, self.exponent);
        let inner = qx.max(qy).min(0.0);

        corner + inner - r
    }

    /// Antialiased coverage of the shape at a pixel center, in 0..=1.
    pub fn coverage(&self, x: f32, y: f32) -> f32 {
        let d = self.distance(x, y);
        if d.abs() > 2.5 {
            return if d < 0.0 { 1.0 } else { 0.0 };
        }
        // Normalize by the gradient magnitude so the AA band has a constant
        // width in pixels even where the superellipse curve is steep.
        let eps = 0.35;
        let gx = self.distance(x + eps, y) - self.distance(x - eps, y);
        let gy = self.distance(x, y + eps) - self.distance(x, y - eps);
        let grad = (gx * gx + gy * gy).sqrt();
        if grad < 1e-4 {
            return if d < 0.0 { 1.0 } else { 0.0 };
        }
        (0.5 - d / grad).clamp(0.0, 1.0)
    }

    /// Bounding box of the shape, for cheap iteration bounds.
    pub fn bounds(&self) -> RectF {
        RectF::new(
            self.center_x - self.half_w,
            self.center_y - self.half_h,
            self.half_w * 2.0,
            self.half_h * 2.0,
        )
    }
}

/// Lp-norm of a 2D vector: `(a^p + b^p)^(1/p)`.
#[inline]
pub fn lnorm(a: f32, b: f32, p: f32) -> f32 {
    if p == 2.0 {
        (a * a + b * b).sqrt()
    } else if p == 4.0 {
        (a * a * a * a + b * b * b * b).sqrt().sqrt()
    } else {
        (a.powf(p) + b.powf(p)).powf(1.0 / p)
    }
}

/// A 32-bit premultiplied BGRA pixel buffer.
///
/// This is the exact pixel format `UpdateLayeredWindow` consumes, so the
/// canvas is blitted to screen with zero conversion.
#[derive(Clone)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    /// BGRA, premultiplied, row-major, tightly packed.
    pub data: Vec<u8>,
    /// Optional axis-aligned clip rectangle `(x0, y0, x1, y1)`.
    pub clip_rect: Option<(i32, i32, i32, i32)>,
    /// Optional superellipse clip silhouette.
    pub clip_shape: Option<RoundRect>,
    /// Global opacity multiplier applied to subsequent draws.
    pub alpha_scale: f32,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0; width * height * 4],
            clip_rect: None,
            clip_shape: None,
            alpha_scale: 1.0,
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
        self.clip_rect = None;
        self.clip_shape = None;
        self.alpha_scale = 1.0;
    }

    pub fn set_clip(&mut self, rect: Option<(i32, i32, i32, i32)>, shape: Option<RoundRect>, alpha: f32) {
        self.clip_rect = rect;
        self.clip_shape = shape;
        self.alpha_scale = alpha.clamp(0.0, 1.0);
    }

    #[inline]
    fn clip_coverage(&self, x: i32, y: i32) -> f32 {
        if let Some((x0, y0, x1, y1)) = self.clip_rect {
            if x < x0 || y < y0 || x >= x1 || y >= y1 {
                return 0.0;
            }
        }
        if let Some(ref rr) = self.clip_shape {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let dx = (px - rr.center_x).abs();
            let dy = (py - rr.center_y).abs();
            if dx > rr.half_w + 1.5 || dy > rr.half_h + 1.5 {
                return 0.0;
            }
            let max_r = rr.radii[0].max(rr.radii[1]).max(rr.radii[2]).max(rr.radii[3]);
            if (dx < rr.half_w - max_r - 2.0 && dy < rr.half_h - 2.0)
                || (dy < rr.half_h - max_r - 2.0 && dx < rr.half_w - 2.0)
            {
                return 1.0;
            }
            return rr.coverage(px, py);
        }
        1.0
    }

    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        (y * self.width + x) * 4
    }

    /// Source-over blend of a straight-alpha color. Returns false if out of bounds.
    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, color: Rgba) -> bool {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return false;
        }
        let clip = self.clip_coverage(x, y);
        if clip <= 0.0 {
            return true;
        }
        let sa = (color.a as f32 / 255.0) * clip * self.alpha_scale;
        if sa <= 0.0 {
            return true;
        }
        let i = self.index(x as usize, y as usize);
        let inv = 1.0 - sa;
        let d = &mut self.data[i..i + 4];
        // Premultiplied source.
        let sr = color.r as f32 * sa;
        let sg = color.g as f32 * sa;
        let sb = color.b as f32 * sa;
        let da = d[3] as f32 / 255.0;
        d[0] = (sb + d[0] as f32 * inv).round().clamp(0.0, 255.0) as u8;
        d[1] = (sg + d[1] as f32 * inv).round().clamp(0.0, 255.0) as u8;
        d[2] = (sr + d[2] as f32 * inv).round().clamp(0.0, 255.0) as u8;
        d[3] = ((sa + da * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
        true
    }

    /// Blit a premultiplied BGRA block (e.g. decoded album art) with an
    /// optional rounded mask.
    #[allow(dead_code)]
    pub fn blit_bgra(&mut self, x: i32, y: i32, src: &[u8], src_w: usize, src_h: usize, mask: Option<&RoundRect>) {
        for sy in 0..src_h {
            for sx in 0..src_w {
                let i = (sy * src_w + sx) * 4;
                let a = src[i + 3] as f32 / 255.0;
                if a <= 0.0 {
                    continue;
                }
                let mut cov = 1.0;
                if let Some(m) = mask {
                    cov = m.coverage((sx as f32) + 0.5, (sy as f32) + 0.5);
                    if cov <= 0.0 {
                        continue;
                    }
                }
                let a = a * cov;
                let dx = x + sx as i32;
                let dy = y + sy as i32;
                if dx < 0 || dy < 0 || dx as usize >= self.width || dy as usize >= self.height {
                    continue;
                }
                let di = self.index(dx as usize, dy as usize);
                let inv = 1.0 - a;
                let d = &mut self.data[di..di + 4];
                let pr = src[i + 2] as f32 * a;
                let pg = src[i + 1] as f32 * a;
                let pb = src[i] as f32 * a;
                let da = d[3] as f32 / 255.0;
                d[0] = (pb + d[0] as f32 * inv).round().clamp(0.0, 255.0) as u8;
                d[1] = (pg + d[1] as f32 * inv).round().clamp(0.0, 255.0) as u8;
                d[2] = (pr + d[2] as f32 * inv).round().clamp(0.0, 255.0) as u8;
                d[3] = ((a + da * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    /// Blit a premultiplied BGRA block scaled to a destination size, with an
    /// optional rounded mask applied in destination coordinates.
    pub fn blit_bgra_scaled(
        &mut self,
        dst_x: i32, dst_y: i32,
        dst_w: usize, dst_h: usize,
        src: &[u8], src_w: usize, src_h: usize,
        mask: Option<&RoundRect>,
    ) {
        if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
            return;
        }
        for dy in 0..dst_h {
            for dx in 0..dst_w {
                let sx = (dx * src_w / dst_w).min(src_w - 1);
                let sy = (dy * src_h / dst_h).min(src_h - 1);
                let si = (sy * src_w + sx) * 4;
                if si + 3 >= src.len() {
                    continue;
                }
                let sa = src[si + 3] as f32 / 255.0;
                if sa <= 0.0 {
                    continue;
                }
                let mut cov = 1.0;
                if let Some(m) = mask {
                    cov = m.coverage(dx as f32 + 0.5, dy as f32 + 0.5);
                    if cov <= 0.0 {
                        continue;
                    }
                }
                let px = dst_x + dx as i32;
                let py = dst_y + dy as i32;
                if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {
                    continue;
                }
                let clip = self.clip_coverage(px, py) * self.alpha_scale;
                if clip <= 0.0 {
                    continue;
                }
                let eff_cov = cov * clip;
                let eff_a = sa * eff_cov;
                let di = self.index(px as usize, py as usize);
                let inv = 1.0 - eff_a;
                let d = &mut self.data[di..di + 4];
                d[0] = (src[si] as f32 * eff_cov + d[0] as f32 * inv).round().clamp(0.0, 255.0) as u8;
                d[1] = (src[si + 1] as f32 * eff_cov + d[1] as f32 * inv).round().clamp(0.0, 255.0) as u8;
                d[2] = (src[si + 2] as f32 * eff_cov + d[2] as f32 * inv).round().clamp(0.0, 255.0) as u8;
                d[3] = ((eff_a + d[3] as f32 / 255.0 * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    /// Fill a rounded rect with a single color.
    pub fn fill_round_rect(&mut self, shape: &RoundRect, color: Rgba) {
        self.fill_round_rect_gradient(shape, color, None);
    }

    /// Fill a rounded rect with an optional vertical gradient.
    ///
    /// `gradient` is `(top_color, bottom_color)`; when supplied, the fill color
    /// is interpolated across the shape's vertical extent.
    pub fn fill_round_rect_gradient(&mut self, shape: &RoundRect, color: Rgba, gradient: Option<(Rgba, Rgba)>) {
        let b = shape.bounds();
        let x0 = b.x.floor() as i32;
        let y0 = b.y.floor() as i32;
        let x1 = b.right().ceil() as i32;
        let y1 = b.bottom().ceil() as i32;
        let span = shape.half_h * 2.0;

        for py in y0..y1 {
            for px in x0..x1 {
                let cov = shape.coverage(px as f32 + 0.5, py as f32 + 0.5);
                if cov <= 0.0 {
                    continue;
                }
                let c = match gradient {
                    Some((top, bottom)) => {
                        let t = if span > 0.0 {
                            ((py as f32 + 0.5) - (shape.center_y - shape.half_h)) / span
                        } else {
                            0.0
                        };
                        top.lerp(bottom, t.clamp(0.0, 1.0))
                    }
                    None => color,
                };
                let mut c = c;
                c.a = (c.a as f32 * cov).round() as u8;
                self.blend(px, py, c);
            }
        }
    }

    /// Draw a hairline stroke just inside a rounded rect.
    pub fn stroke_round_rect(&mut self, shape: &RoundRect, color: Rgba, thickness: f32) {
        let b = shape.bounds();
        let x0 = b.x.floor() as i32 - 1;
        let y0 = b.y.floor() as i32 - 1;
        let x1 = b.right().ceil() as i32 + 1;
        let y1 = b.bottom().ceil() as i32 + 1;

        for py in y0..y1 {
            for px in x0..x1 {
                let d = shape.distance(px as f32 + 0.5, py as f32 + 0.5);
                // Band from -thickness to 0 (inside the edge).
                let t = ((0.0 - d) / thickness).clamp(0.0, 1.0);
                if t <= 0.0 {
                    continue;
                }
                let mut c = color;
                c.a = (color.a as f32 * t).round() as u8;
                self.blend(px, py, c);
            }
        }
    }

    /// Fill an axis-aligned rect.
    pub fn fill_rect(&mut self, rect: RectF, color: Rgba) {
        let x0 = rect.x.floor() as i32;
        let y0 = rect.y.floor() as i32;
        let x1 = rect.right().ceil() as i32;
        let y1 = rect.bottom().ceil() as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                self.blend(px, py, color);
            }
        }
    }

    /// Fill a circle.
    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgba) {
        let x0 = (cx - radius - 1.0).floor() as i32;
        let y0 = (cy - radius - 1.0).floor() as i32;
        let x1 = (cx + radius + 1.0).ceil() as i32;
        let y1 = (cy + radius + 1.0).ceil() as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt() - radius;
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let mut c = color;
                c.a = (color.a as f32 * cov).round() as u8;
                self.blend(px, py, c);
            }
        }
    }

    /// Draw an antialiased line segment with rounded caps.
    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, thickness: f32, color: Rgba) {
        let r = thickness * 0.5;
        let min_x = (x0.min(x1) - r - 1.0).floor() as i32;
        let max_x = (x0.max(x1) + r + 1.0).ceil() as i32;
        let min_y = (y0.min(y1) - r - 1.0).floor() as i32;
        let max_y = (y0.max(y1) + r + 1.0).ceil() as i32;
        let vx = x1 - x0;
        let vy = y1 - y0;
        let len2 = vx * vx + vy * vy;

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let cx = px as f32 + 0.5;
                let cy = py as f32 + 0.5;
                let t = if len2 > 1e-6 {
                    (((cx - x0) * vx + (cy - y0) * vy) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let dx = cx - (x0 + vx * t);
                let dy = cy - (y0 + vy * t);
                let d = (dx * dx + dy * dy).sqrt() - r;
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let mut c = color;
                c.a = (color.a as f32 * cov).round() as u8;
                self.blend(px, py, c);
            }
        }
    }

    /// Draw an antialiased circular progress ring starting at 12 o'clock clockwise.
    pub fn draw_ring_arc(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        thickness: f32,
        progress: f32,
        track_color: Rgba,
        fill_color: Rgba,
    ) {
        let cap_r = thickness * 0.5;
        let outer = radius + cap_r + 1.5;
        let x0 = (cx - outer).floor() as i32;
        let y0 = (cy - outer).floor() as i32;
        let x1 = (cx + outer).ceil() as i32;
        let y1 = (cy + outer).ceil() as i32;

        let p = progress.clamp(0.0, 1.0);
        let end_a = p * 2.0 * PI;
        let sx = cx;
        let sy = cy - radius;
        let ex = cx + radius * end_a.sin();
        let ey = cy - radius * end_a.cos();

        for py in y0..=y1 {
            for px in x0..=x1 {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let dx = fx - cx;
                let dy = fy - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let ring_d = (dist - radius).abs() - cap_r;
                let ring_cov = (0.5 - ring_d).clamp(0.0, 1.0);
                if ring_cov <= 0.0 {
                    continue;
                }

                let mut tc = track_color;
                tc.a = (track_color.a as f32 * ring_cov).round() as u8;
                self.blend(px, py, tc);

                if p > 0.001 {
                    let fill_cov = if p >= 0.999 {
                        ring_cov
                    } else {
                        let mut angle = dx.atan2(-dy);
                        if angle < 0.0 {
                            angle += 2.0 * PI;
                        }
                        let frac = angle / (2.0 * PI);
                        let sector_cov = if frac <= p { ring_cov } else { 0.0 };
                        let sd = ((fx - sx) * (fx - sx) + (fy - sy) * (fy - sy)).sqrt() - cap_r;
                        let ed = ((fx - ex) * (fx - ex) + (fy - ey) * (fy - ey)).sqrt() - cap_r;
                        let cap_cov = (0.5 - sd.min(ed)).clamp(0.0, 1.0).min(ring_cov);
                        sector_cov.max(cap_cov)
                    };
                    if fill_cov > 0.0 {
                        let mut fc = fill_color;
                        fc.a = (fill_color.a as f32 * fill_cov).round() as u8;
                        self.blend(px, py, fc);
                    }
                }
            }
        }
    }

    /// Composite a coverage mask (0..=1 per pixel) as a solid color.
    #[allow(dead_code)]
    pub fn blend_coverage(&mut self, x: i32, y: i32, w: usize, h: usize, mask: &[f32], color: Rgba) {
        for my in 0..h {
            for mx in 0..w {
                let idx = my * w + mx;
                if idx >= mask.len() {
                    break;
                }
                let cov = mask[idx];
                if cov <= 0.0 {
                    continue;
                }
                let mut c = color;
                c.a = (color.a as f32 * cov).round() as u8;
                self.blend(x + mx as i32, y + my as i32, c);
            }
        }
    }
}

/// A single-channel coverage buffer, used to build drop shadows.
#[derive(Clone)]
pub struct Mask {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl Mask {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, data: vec![0.0; width * height] }
    }

    pub fn clear(&mut self) {
        self.data.fill(0.0);
    }

    /// Rasterize a rounded rect into this mask.
    pub fn fill_round_rect(&mut self, shape: &RoundRect, value: f32) {
        let b = shape.bounds();
        let x0 = b.x.floor() as i32;
        let y0 = b.y.floor() as i32;
        let x1 = b.right().ceil() as i32;
        let y1 = b.bottom().ceil() as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                if px < 0 || py < 0 || px as usize >= self.width || py as usize >= self.height {
                    continue;
                }
                let cov = shape.coverage(px as f32 + 0.5, py as f32 + 0.5);
                let i = py as usize * self.width + px as usize;
                self.data[i] = self.data[i].max(cov * value);
            }
        }
    }

    /// Separable box blur, applied three times to approximate a Gaussian.
    #[allow(dead_code)]
    pub fn blur(&mut self, radius: usize) {
        if radius == 0 {
            return;
        }
        let r = radius as i32;
        let w = self.width as i32;
        let h = self.height as i32;
        let mut tmp = vec![0.0f32; self.data.len()];

        for _ in 0..3 {
            // Horizontal.
            for y in 0..h {
                for x in 0..w {
                    let mut sum = 0.0;
                    let mut count = 0.0;
                    for k in -r..=r {
                        let sx = x + k;
                        if sx >= 0 && sx < w {
                            sum += self.data[y as usize * self.width + sx as usize];
                            count += 1.0;
                        }
                    }
                    tmp[y as usize * self.width + x as usize] = sum / count;
                }
            }
            // Vertical.
            for y in 0..h {
                for x in 0..w {
                    let mut sum = 0.0;
                    let mut count = 0.0;
                    for k in -r..=r {
                        let sy = y + k;
                        if sy >= 0 && sy < h {
                            sum += tmp[sy as usize * self.width + x as usize];
                            count += 1.0;
                        }
                    }
                    self.data[y as usize * self.width + x as usize] = sum / count;
                }
            }
        }
    }

    /// Separable box blur restricted to a sub-rectangle.
    ///
    /// The shadow only ever needs softening near the island, and a full-canvas
    /// blur would cost tens of milliseconds per frame. Blurring just the
    /// interesting region keeps the 60 FPS budget intact.
    pub fn blur_in(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, radius: usize) {
        if radius == 0 {
            return;
        }
        let x0 = x0.clamp(0, self.width as i32);
        let y0 = y0.clamp(0, self.height as i32);
        let x1 = x1.clamp(0, self.width as i32);
        let y1 = y1.clamp(0, self.height as i32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let r = radius as i32;
        let mut tmp = vec![0.0f32; self.data.len()];

        // Horizontal pass.
        for y in y0..y1 {
            for x in x0..x1 {
                let mut sum = 0.0;
                let mut count = 0.0;
                for k in -r..=r {
                    let sx = x + k;
                    if sx >= x0 && sx < x1 {
                        sum += self.data[y as usize * self.width + sx as usize];
                        count += 1.0;
                    }
                }
                tmp[y as usize * self.width + x as usize] = sum / count;
            }
        }
        // Vertical pass.
        for y in y0..y1 {
            for x in x0..x1 {
                let mut sum = 0.0;
                let mut count = 0.0;
                for k in -r..=r {
                    let sy = y + k;
                    if sy >= y0 && sy < y1 {
                        sum += tmp[sy as usize * self.width + x as usize];
                        count += 1.0;
                    }
                }
                self.data[y as usize * self.width + x as usize] = sum / count;
            }
        }
    }

    /// Shift the mask contents by (dx, dy).
    #[allow(dead_code)]
    pub fn offset(&self, dx: i32, dy: i32) -> Mask {
        let mut out = Mask::new(self.width, self.height);
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let sx = x - dx;
                let sy = y - dy;
                if sx < 0 || sy < 0 || sx >= self.width as i32 || sy >= self.height as i32 {
                    continue;
                }
                out.data[y as usize * self.width + x as usize] =
                    self.data[sy as usize * self.width + sx as usize];
            }
        }
        out
    }
}

/// Ease a value between 0 and 1 with a smooth curve.
#[allow(dead_code)]
#[inline]
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Convert degrees to radians.
#[allow(dead_code)]
#[inline]
pub fn radians(deg: f32) -> f32 {
    deg * PI / 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squircle_coverage_is_antialiased() {
        let shape = RoundRect::new(50.0, 50.0, 40.0, 20.0, [20.0; 4], 4.0);
        assert!(shape.coverage(50.0, 50.0) > 0.99);
        assert!(shape.coverage(0.0, 0.0) < 0.01);
        // Edge pixels should be partial, not binary.
        let edge = shape.coverage(50.0, 70.0);
        assert!(edge > 0.0 && edge < 1.0, "edge coverage {edge}");
    }

    #[test]
    fn corner_is_fuller_than_circle() {
        // At the 45-degree diagonal, a superellipse corner should extend
        // further from the corner point than a circular one.
        let squircle = RoundRect::new(0.0, 0.0, 100.0, 100.0, [30.0; 4], 4.0);
        let circle = RoundRect::new(0.0, 0.0, 100.0, 100.0, [30.0; 4], 2.0);
        let d = 30.0f32 / std::f32::consts::SQRT_2;
        let probe = 100.0 - d + 1.0;
        assert!(squircle.distance(probe, probe) < circle.distance(probe, probe));
    }

    #[test]
    fn sharp_corner_zero_radius() {
        let shape = RoundRect::new(0.0, 0.0, 50.0, 50.0, [0.0; 4], 4.0);
        assert!(shape.distance(49.0, 49.0) < 0.0, "inside");
        assert!(shape.distance(51.0, 51.0) > 0.0, "outside");
    }

    #[test]
    fn canvas_blend_premultiplies() {
        let mut c = Canvas::new(4, 4);
        c.blend(0, 0, Rgba::rgba(255, 0, 0, 128));
        assert_eq!(c.data[3], 128);
        assert_eq!(c.data[2], 128, "red channel premultiplied");
    }

    #[test]
    fn blur_spreads_mask() {
        let mut m = Mask::new(32, 32);
        let shape = RoundRect::new(16.0, 16.0, 6.0, 6.0, [6.0; 4], 4.0);
        m.fill_round_rect(&shape, 1.0);
        let before = m.data[16 * 32 + 16];
        m.blur(3);
        let after = m.data[16 * 32 + 16];
        assert!(after < before, "blur should reduce peak");
        assert!(m.data[16 * 32 + 26] > 0.0, "blur should spread outward");
    }
}
