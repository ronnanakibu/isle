//! Apple Dynamic Island High-Fidelity Software Rasterizer
//! Features continuous squircle blending, specular rim lighting, and crisp GDI typography

use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;

use crate::theme::*;
use crate::spring::*;
use crate::mascot::*;
use crate::pages::*;
use crate::media::MediaSnapshot;

pub struct Surface {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>,
}

impl Surface {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(COLOR_TRANSPARENT);
    }

    /// Blend a premultiplied ARGB pixel onto existing pixel
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, src: u32) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        let dst = self.pixels[idx];

        let sa = (src >> 24) & 0xFF;
        if sa == 0 {
            return;
        }
        if sa == 255 {
            self.pixels[idx] = src;
            return;
        }

        let inv_sa = 255 - sa;
        let sr = (src >> 16) & 0xFF;
        let sg = (src >> 8) & 0xFF;
        let sb = src & 0xFF;

        let da = (dst >> 24) & 0xFF;
        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let out_a = sa + (da * inv_sa + 127) / 255;
        let out_r = sr + (dr * inv_sa + 127) / 255;
        let out_g = sg + (dg * inv_sa + 127) / 255;
        let out_b = sb + (db * inv_sa + 127) / 255;

        self.pixels[idx] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
    }

    /// Fill an anti-aliased rounded rectangle (Apple Squircle approximation)
    pub fn fill_rounded_rect(&mut self, x0: f32, y0: f32, w: f32, h: f32, r: f32, color: u32, flat_top: bool) {
        let min_x = (x0 - 1.0).max(0.0) as i32;
        let max_x = (x0 + w + 1.0).min(self.width as f32) as i32;
        let min_y = (y0 - 1.0).max(0.0) as i32;
        let max_y = (y0 + h + 1.0).min(self.height as f32) as i32;

        let ca = ((color >> 24) & 0xFF) as f32 / 255.0;
        let cr = ((color >> 16) & 0xFF) as f32 / 255.0;
        let cg = ((color >> 8) & 0xFF) as f32 / 255.0;
        let cb = (color & 0xFF) as f32 / 255.0;

        let effective_r = r.min(w * 0.5).min(h * 0.5);
        let cx_left = x0 + effective_r;
        let cx_right = x0 + w - effective_r;
        let cy_top = if flat_top { y0 } else { y0 + effective_r };
        let cy_bottom = y0 + h - effective_r;

        for y in min_y..max_y {
            let py = y as f32 + 0.5;
            for x in min_x..max_x {
                let px = x as f32 + 0.5;

                let dx = if px < cx_left {
                    cx_left - px
                } else if px > cx_right {
                    px - cx_right
                } else {
                    0.0
                };

                let dy = if !flat_top && py < cy_top {
                    cy_top - py
                } else if py > cy_bottom {
                    py - cy_bottom
                } else {
                    0.0
                };

                let dist = (dx * dx + dy * dy).sqrt();
                let alpha = (effective_r + 0.5 - dist).clamp(0.0, 1.0);

                if alpha > 0.001 {
                    let final_a = (ca * alpha * 255.0).round() as u32;
                    let final_r = (cr * ca * alpha * 255.0).round() as u32;
                    let final_g = (cg * ca * alpha * 255.0).round() as u32;
                    let final_b = (cb * ca * alpha * 255.0).round() as u32;

                    self.blend_pixel(x, y, (final_a << 24) | (final_r << 16) | (final_g << 8) | final_b);
                }
            }
        }
    }

    /// Stroke an anti-aliased rounded rectangle border (Apple specular outline)
    pub fn stroke_rounded_rect(&mut self, x0: f32, y0: f32, w: f32, h: f32, r: f32, thickness: f32, color: u32, flat_top: bool) {
        let min_x = (x0 - thickness - 1.0).max(0.0) as i32;
        let max_x = (x0 + w + thickness + 1.0).min(self.width as f32) as i32;
        let min_y = (y0 - thickness - 1.0).max(0.0) as i32;
        let max_y = (y0 + h + thickness + 1.0).min(self.height as f32) as i32;

        let ca = ((color >> 24) & 0xFF) as f32 / 255.0;
        let cr = ((color >> 16) & 0xFF) as f32 / 255.0;
        let cg = ((color >> 8) & 0xFF) as f32 / 255.0;
        let cb = (color & 0xFF) as f32 / 255.0;

        let effective_r = r.min(w * 0.5).min(h * 0.5);
        let cx_left = x0 + effective_r;
        let cx_right = x0 + w - effective_r;
        let cy_top = if flat_top { y0 } else { y0 + effective_r };
        let cy_bottom = y0 + h - effective_r;

        let half_t = thickness * 0.5;

        for y in min_y..max_y {
            let py = y as f32 + 0.5;
            for x in min_x..max_x {
                let px = x as f32 + 0.5;

                let dx = if px < cx_left {
                    cx_left - px
                } else if px > cx_right {
                    px - cx_right
                } else {
                    0.0
                };

                let dy = if !flat_top && py < cy_top {
                    cy_top - py
                } else if py > cy_bottom {
                    py - cy_bottom
                } else {
                    0.0
                };

                let dist = (dx * dx + dy * dy).sqrt();
                let diff = (dist - effective_r).abs();
                let alpha = (half_t + 0.5 - diff).clamp(0.0, 1.0);

                if alpha > 0.001 {
                    let final_a = (ca * alpha * 255.0).round() as u32;
                    let final_r = (cr * ca * alpha * 255.0).round() as u32;
                    let final_g = (cg * ca * alpha * 255.0).round() as u32;
                    let final_b = (cb * ca * alpha * 255.0).round() as u32;

                    self.blend_pixel(x, y, (final_a << 24) | (final_r << 16) | (final_g << 8) | final_b);
                }
            }
        }
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, color: u32) {
        self.fill_rounded_rect(cx - r, cy - r, r * 2.0, r * 2.0, r, color, false);
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, r: f32, thickness: f32, color: u32) {
        self.stroke_rounded_rect(cx - r, cy - r, r * 2.0, r * 2.0, r, thickness, color, false);
    }

    /// Apple Specular Top Highlight Pass (1.5px subtle top sheen)
    pub fn specular_top_pass(&mut self, x0: f32, y0: f32, w: f32, radius: f32, strength: f32) {
        let cx = x0 + w * 0.5;
        let y_start = (y0 - 1.0).max(0.0) as i32;
        let y_end = ((y0 + radius * 1.5).ceil() as i32).min(self.height);
        let x_start = (x0 - 1.0).max(0.0) as i32;
        let x_end = ((x0 + w + 1.0).ceil() as i32).min(self.width);

        for y in y_start..y_end {
            let py = y as f32 + 0.5;
            let fall = (1.0 - (py - y0) / (radius * 1.5)).clamp(0.0, 1.0);
            if fall <= 0.0 {
                continue;
            }
            let fall2 = fall * fall;

            for x in x_start..x_end {
                let px = x as f32 + 0.5;
                let nx = ((px - cx) / (w * 0.5 * 0.85)).abs();
                if nx > 1.0 {
                    continue;
                }
                let horiz = (1.0 - nx * nx).powi(2);
                let a = horiz * fall2 * strength;
                if a <= 0.002 {
                    continue;
                }

                let alpha_u = (a * 255.0).round() as u32;
                let white_premul = (alpha_u << 24) | (alpha_u << 16) | (alpha_u << 8) | alpha_u;
                self.blend_pixel(x, y, white_premul);
            }
        }
    }

    /// Blit decoded album art with rounded corner mask
    pub fn blit_album_art(&mut self, dst_x: i32, dst_y: i32, art: &[u8], art_dim: u32, radius: f32) {
        let half = art_dim as f32 * 0.5;
        let cx = dst_x as f32 + half;
        let cy = dst_y as f32 + half;

        for sy in 0..art_dim as i32 {
            let dy = dst_y + sy;
            if dy < 0 || dy >= self.height {
                continue;
            }

            for sx in 0..art_dim as i32 {
                let dx = dst_x + sx;
                if dx < 0 || dx >= self.width {
                    continue;
                }

                let rel_x = (dx as f32 + 0.5 - cx).abs();
                let rel_y = (dy as f32 + 0.5 - cy).abs();
                let qx = (rel_x - (half - radius)).max(0.0);
                let qy = (rel_y - (half - radius)).max(0.0);
                let dist = (qx * qx + qy * qy).sqrt();
                let cov = (radius + 0.5 - dist).clamp(0.0, 1.0);
                if cov <= 0.01 {
                    continue;
                }

                let src_idx = ((sy * art_dim as i32 + sx) * 4) as usize;
                if src_idx + 3 < art.len() {
                    let b = (art[src_idx] as f32 * cov) as u32;
                    let g = (art[src_idx + 1] as f32 * cov) as u32;
                    let r = (art[src_idx + 2] as f32 * cov) as u32;
                    let a = (art[src_idx + 3] as f32 * cov) as u32;
                    let premul = (a << 24) | (r << 16) | (g << 8) | b;
                    self.blend_pixel(dx, dy, premul);
                }
            }
        }
    }
}

pub fn render_apple_island(
    surface: &mut Surface,
    width_spring: &Spring,
    height_spring: &Spring,
    expand_t: f32,
    mascot: &Mascot,
    page_state: &PageState,
    media: &MediaSnapshot,
    flat_top: bool,
    hovered: HitTarget,
) {
    surface.clear();

    let cur_w = width_spring.current;
    let cur_h = height_spring.current;

    let x0 = ((CANVAS_WIDTH as f32) - cur_w) * 0.5;
    let y0 = if flat_top { 0.0 } else { 10.0 };

    let radius = COMPACT_RADIUS + (EXPANDED_RADIUS - COMPACT_RADIUS) * expand_t;

    // 1. Soft Ambient Drop Shadow under island
    let shadow_offset_y = 2.0 + 8.0 * expand_t;
    let shadow_spread = 2.0 + 6.0 * expand_t;
    surface.fill_rounded_rect(
        x0 - shadow_spread * 0.5,
        y0 + shadow_offset_y,
        cur_w + shadow_spread,
        cur_h + shadow_spread,
        radius + shadow_spread * 0.5,
        fade_color(COLOR_ISLAND_SHADOW, 0.4 + 0.4 * expand_t),
        flat_top,
    );

    // 2. Main Apple OLED True Black Body
    surface.fill_rounded_rect(x0, y0, cur_w, cur_h, radius, COLOR_ISLAND_BLACK, flat_top);

    // 3. Apple Specular Glass Border (Subtle white rim light)
    surface.stroke_rounded_rect(
        x0,
        y0,
        cur_w,
        cur_h,
        radius,
        1.0,
        COLOR_ISLAND_BORDER,
        flat_top,
    );

    // 4. Apple Specular Top Highlight Sheen (only in floating mode)
    if !flat_top {
        surface.specular_top_pass(x0, y0, cur_w, radius, 0.28);
    }

    // RENDER COMPACT PILL STATE (< 0.7 expansion)
    if expand_t < 0.7 {
        let alpha = (1.0 - expand_t / 0.7).clamp(0.0, 1.0);

        let lead_x = x0 + 10.0;
        let lead_y = y0 + 8.0;
        let lead_s = 20.0;
        let lead_r = 6.0;

        match page_state.current_page {
            PageId::Music => {
                if let Some(ref art) = media.art {
                    surface.blit_album_art(lead_x as i32, lead_y as i32, &art.bgra_premul, art.size, lead_r);
                } else {
                    surface.fill_rounded_rect(lead_x, lead_y, lead_s, lead_s, lead_r, fade_color(0xFF1E1B4B, alpha), false);
                    surface.fill_circle(lead_x + 10.0, lead_y + 10.0, 4.0, fade_color(APPLE_CYAN, alpha));
                }
            }
            PageId::Pomodoro => {
                surface.fill_circle(lead_x + 10.0, lead_y + 10.0, 9.0, fade_color(APPLE_ORANGE, alpha));
                surface.fill_circle(lead_x + 10.0, lead_y + 10.0, 5.0, fade_color(COLOR_ISLAND_BLACK, alpha));
            }
            PageId::Water => {
                surface.fill_rounded_rect(lead_x + 3.0, lead_y + 2.0, 14.0, 16.0, 4.0, fade_color(APPLE_CYAN, alpha), false);
            }
            PageId::Todo => {
                surface.fill_circle(lead_x + 10.0, lead_y + 10.0, 8.0, fade_color(APPLE_BLUE, alpha));
                render_check_icon(surface, lead_x + 10.0, lead_y + 10.0, fade_color(COLOR_TEXT_WHITE, alpha));
            }
            PageId::Shelf => {
                surface.fill_rounded_rect(lead_x + 2.0, lead_y + 3.0, 16.0, 14.0, 3.0, fade_color(APPLE_PURPLE, alpha), false);
            }
            PageId::Hardware => {
                surface.fill_rounded_rect(lead_x + 2.0, lead_y + 2.0, 16.0, 16.0, 4.0, fade_color(APPLE_GREEN, alpha), false);
            }
            PageId::Zen => {
                surface.fill_circle(lead_x + 10.0, lead_y + 10.0, 7.0, fade_color(APPLE_CYAN, alpha));
            }
        }

        // Center Mascot Eyes
        let mascot_cx = x0 + cur_w * 0.5;
        let mascot_cy = y0 + cur_h * 0.5;
        render_mascot_eyes(surface, mascot_cx, mascot_cy, mascot, alpha);

        // Trailing View: Equalizer Waveform
        let wave_right = x0 + cur_w - 12.0;
        let wave_cy = y0 + cur_h * 0.5;
        let is_playing = if media.has_session { media.playing } else { page_state.is_playing };

        for i in 0..4 {
            let bx = wave_right - ((3 - i) as f32 * 4.5);
            let bar_h = 4.0 + page_state.eq_bars[i] * 14.0;
            let by = wave_cy - bar_h * 0.5;
            let wave_col = if is_playing { APPLE_ORANGE } else { COLOR_TEXT_TERTIARY };
            surface.fill_rounded_rect(bx, by, 2.5, bar_h, 1.25, fade_color(wave_col, alpha), false);
        }
    }

    // RENDER EXPANDED STATE (> 0.25 expansion)
    if expand_t > 0.25 {
        let alpha = ((expand_t - 0.25) / 0.75).clamp(0.0, 1.0);
        let center_x = x0 + cur_w * 0.5;
        let head_y = y0 + 16.0;

        // Mascot sitting atop the card
        let mascot_cx = center_x;
        let mascot_cy = head_y + 8.0;
        render_mascot_eyes(surface, mascot_cx, mascot_cy, mascot, alpha);

        match page_state.current_page {
            PageId::Music => {
                render_expanded_music(surface, x0, y0, cur_w, alpha, page_state, media, hovered);
            }
            PageId::Pomodoro => {
                render_expanded_pomodoro(surface, x0, y0, cur_w, alpha, page_state, hovered);
            }
            PageId::Water => {
                render_expanded_water(surface, x0, y0, cur_w, alpha, page_state, hovered);
            }
            PageId::Todo => {
                render_expanded_todo(surface, x0, y0, cur_w, alpha, page_state, hovered);
            }
            PageId::Shelf => {
                render_expanded_shelf(surface, x0, y0, cur_w, alpha, page_state, hovered);
            }
            PageId::Hardware => {
                render_expanded_hardware(surface, x0, y0, cur_w, alpha, page_state, hovered);
            }
            PageId::Zen => {
                render_expanded_zen(surface, x0, y0, cur_w, alpha, page_state, hovered);
            }
        }

        // Bottom Page Indicator Pill (Apple style)
        let pill_y = y0 + cur_h - 12.0;
        let dots_count = PageId::ALL.len();
        let spacing = 14.0;
        let start_x = center_x - ((dots_count as f32 - 1.0) * spacing) * 0.5;

        for (i, p) in PageId::ALL.iter().enumerate() {
            let px = start_x + (i as f32) * spacing;
            let is_active = *p == page_state.current_page;
            let is_dot_hover = hovered == HitTarget::PageDot(i);

            if is_active {
                surface.fill_rounded_rect(px - 6.0, pill_y - 2.0, 12.0, 4.0, 2.0, fade_color(COLOR_TEXT_WHITE, alpha), false);
            } else if is_dot_hover {
                surface.fill_circle(px, pill_y, 3.0, fade_color(COLOR_TEXT_WHITE, alpha * 0.8));
            } else {
                surface.fill_circle(px, pill_y, 2.0, fade_color(COLOR_TEXT_TERTIARY, alpha));
            }
        }
    }
}

fn render_expanded_music(
    surface: &mut Surface,
    x0: f32,
    y0: f32,
    cur_w: f32,
    alpha: f32,
    page_state: &PageState,
    media: &MediaSnapshot,
    hovered: HitTarget,
) {
    let head_y = y0 + 16.0;

    // Album Artwork on Left
    let art_x = x0 + 18.0;
    let art_s = 52.0;
    let art_r = 12.0;
    surface.fill_rounded_rect(art_x, head_y + 2.0, art_s, art_s, art_r, fade_color(0x55000000, alpha), false);

    if let Some(ref art) = media.art {
        surface.blit_album_art(art_x as i32, head_y as i32, &art.bgra_premul, art.size, art_r);
        surface.stroke_rounded_rect(art_x, head_y, art_s, art_s, art_r, 1.0, fade_color(0x33FFFFFF, alpha), false);
    } else {
        surface.fill_rounded_rect(art_x, head_y, art_s, art_s, art_r, fade_color(0xFF1E1B4B, alpha), false);
        surface.stroke_rounded_rect(art_x, head_y, art_s, art_s, art_r, 1.0, fade_color(0x33FFFFFF, alpha), false);
        surface.fill_circle(art_x + 26.0, head_y + 26.0, 18.0, fade_color(0xFF0F0F12, alpha));
        surface.fill_circle(art_x + 26.0, head_y + 26.0, 6.0, fade_color(APPLE_CYAN, alpha));
    }

    // Right side: Equalizer Waveform
    let wave_right = x0 + cur_w - 22.0;
    let is_playing = if media.has_session { media.playing } else { page_state.is_playing };

    for i in 0..5 {
        let bx = wave_right - ((4 - i) as f32 * 5.0);
        let bar_h = 5.0 + page_state.eq_bars[i] * 20.0;
        let by = (head_y + 26.0) - bar_h * 0.5;
        let col = if is_playing { APPLE_ORANGE } else { COLOR_TEXT_TERTIARY };
        surface.fill_rounded_rect(bx, by, 3.0, bar_h, 1.5, fade_color(col, alpha), false);
    }

    // Scrubber Progress Bar
    let bar_x = x0 + 20.0;
    let bar_y = y0 + 84.0;
    let bar_w = cur_w - 40.0;
    let bar_h = 4.0;

    let progress = if media.has_session && media.duration_ms > 0 {
        (media.position_ms_now() as f32 / media.duration_ms as f32).clamp(0.0, 1.0)
    } else {
        page_state.track_progress
    };

    surface.fill_rounded_rect(bar_x, bar_y, bar_w, bar_h, 2.0, fade_color(COLOR_TRACK_BG, alpha), false);
    let fill_w = bar_w * progress;
    surface.fill_rounded_rect(bar_x, bar_y, fill_w, bar_h, 2.0, fade_color(COLOR_TEXT_WHITE, alpha), false);
    surface.fill_circle(bar_x + fill_w, bar_y + 2.0, 3.5, fade_color(COLOR_TEXT_WHITE, alpha));

    // Controls
    let ctrl_cy = y0 + 116.0;
    let center_x = x0 + cur_w * 0.5;

    // Prev button
    let prev_bg = if hovered == HitTarget::MediaPrev { COLOR_CONTROL_HOVER } else { COLOR_CONTROL_BG };
    surface.fill_circle(center_x - 56.0, ctrl_cy, 15.0, fade_color(prev_bg, alpha));
    render_prev_icon(surface, center_x - 56.0, ctrl_cy, fade_color(COLOR_TEXT_WHITE, alpha));

    // Play / Pause prominent button
    let play_bg = if hovered == HitTarget::MediaPlayPause { 0xFFFFFFFF } else { fade_color(COLOR_TEXT_WHITE, 0.95) };
    surface.fill_circle(center_x, ctrl_cy, 20.0, fade_color(play_bg, alpha));
    if is_playing {
        surface.fill_rounded_rect(center_x - 5.0, ctrl_cy - 7.0, 3.5, 14.0, 1.5, fade_color(COLOR_ISLAND_BLACK, alpha), false);
        surface.fill_rounded_rect(center_x + 1.5, ctrl_cy - 7.0, 3.5, 14.0, 1.5, fade_color(COLOR_ISLAND_BLACK, alpha), false);
    } else {
        render_play_icon(surface, center_x + 1.0, ctrl_cy, fade_color(COLOR_ISLAND_BLACK, alpha));
    }

    // Next button
    let next_bg = if hovered == HitTarget::MediaNext { COLOR_CONTROL_HOVER } else { COLOR_CONTROL_BG };
    surface.fill_circle(center_x + 56.0, ctrl_cy, 15.0, fade_color(next_bg, alpha));
    render_next_icon(surface, center_x + 56.0, ctrl_cy, fade_color(COLOR_TEXT_WHITE, alpha));
}

fn render_expanded_pomodoro(surface: &mut Surface, x0: f32, y0: f32, cur_w: f32, alpha: f32, page_state: &PageState, hovered: HitTarget) {
    let head_y = y0 + 16.0;

    let ring_cx = x0 + 44.0;
    let ring_cy = head_y + 32.0;
    let ring_r = 24.0;

    surface.stroke_circle(ring_cx, ring_cy, ring_r, 4.0, fade_color(COLOR_TRACK_BG, alpha));
    surface.stroke_circle(ring_cx, ring_cy, ring_r, 4.0, fade_color(APPLE_ORANGE, alpha));
    surface.fill_circle(ring_cx, ring_cy, 10.0, fade_color(0x33FF9500, alpha));

    // Mode Selector Pills
    let mode_y = y0 + 74.0;
    let modes = ["25m Focus", "5m Break", "15m Rest"];
    let pill_w = 78.0;
    let spacing = 8.0;
    let total_w = 3.0 * pill_w + 2.0 * spacing;
    let start_x = x0 + (cur_w - total_w) * 0.5;

    for (i, _) in modes.iter().enumerate() {
        let px = start_x + (i as f32) * (pill_w + spacing);
        let is_selected = i == page_state.pomo_mode;
        let is_hov = hovered == HitTarget::PomoMode(i);

        if is_selected {
            surface.fill_rounded_rect(px, mode_y, pill_w, 22.0, 11.0, fade_color(0x44FF9500, alpha), false);
            surface.stroke_rounded_rect(px, mode_y, pill_w, 22.0, 11.0, 1.0, fade_color(APPLE_ORANGE, alpha), false);
        } else if is_hov {
            surface.fill_rounded_rect(px, mode_y, pill_w, 22.0, 11.0, fade_color(0x33FFFFFF, alpha), false);
        } else {
            surface.fill_rounded_rect(px, mode_y, pill_w, 22.0, 11.0, fade_color(0x18FFFFFF, alpha), false);
        }
    }

    // Action Buttons
    let btn_y = y0 + 104.0;
    let center_x = x0 + cur_w * 0.5;

    let start_col = if page_state.pomo_running { APPLE_ORANGE } else { APPLE_GREEN };
    surface.fill_rounded_rect(center_x - 52.0, btn_y, 72.0, 24.0, 12.0, fade_color(start_col, alpha), false);

    let reset_bg = if hovered == HitTarget::PomoReset { COLOR_CONTROL_HOVER } else { 0x2AFFFFFF };
    surface.fill_rounded_rect(center_x + 28.0, btn_y, 52.0, 24.0, 12.0, fade_color(reset_bg, alpha), false);
}

fn render_expanded_water(surface: &mut Surface, x0: f32, y0: f32, cur_w: f32, alpha: f32, page_state: &PageState, hovered: HitTarget) {
    let head_y = y0 + 16.0;

    let cup_x = x0 + 26.0;
    let cup_y = head_y + 4.0;
    let cup_w = 34.0;
    let cup_h = 50.0;
    surface.stroke_rounded_rect(cup_x, cup_y, cup_w, cup_h, 8.0, 2.0, fade_color(COLOR_TRACK_BG, alpha), false);

    let fill_ratio = (page_state.water_taken_ml as f32 / page_state.water_goal_ml as f32).clamp(0.0, 1.0);
    let fill_h = (cup_h - 4.0) * fill_ratio;
    if fill_h > 2.0 {
        surface.fill_rounded_rect(
            cup_x + 2.0,
            cup_y + cup_h - 2.0 - fill_h,
            cup_w - 4.0,
            fill_h,
            6.0,
            fade_color(APPLE_CYAN, alpha * 0.85),
            false,
        );
    }

    let bar_x = x0 + 75.0;
    let bar_y = y0 + 64.0;
    let bar_w = cur_w - 95.0;
    surface.fill_rounded_rect(bar_x, bar_y, bar_w, 6.0, 3.0, fade_color(COLOR_TRACK_BG, alpha), false);
    let progress_w = bar_w * fill_ratio;
    surface.fill_rounded_rect(bar_x, bar_y, progress_w, 6.0, 3.0, fade_color(APPLE_CYAN, alpha), false);

    // Quick Action Pill Buttons
    let btn_y = y0 + 96.0;
    let b1_bg = if hovered == HitTarget::WaterAdd250 { 0x5538BDF8 } else { 0x3338BDF8 };
    surface.fill_rounded_rect(x0 + 75.0, btn_y, 76.0, 24.0, 12.0, fade_color(b1_bg, alpha), false);
    surface.stroke_rounded_rect(x0 + 75.0, btn_y, 76.0, 24.0, 12.0, 1.0, fade_color(APPLE_CYAN, alpha), false);

    let b2_bg = if hovered == HitTarget::WaterAdd500 { 0x5538BDF8 } else { 0x3338BDF8 };
    surface.fill_rounded_rect(x0 + 160.0, btn_y, 76.0, 24.0, 12.0, fade_color(b2_bg, alpha), false);
    surface.stroke_rounded_rect(x0 + 160.0, btn_y, 76.0, 24.0, 12.0, 1.0, fade_color(APPLE_CYAN, alpha), false);

    let b3_bg = if hovered == HitTarget::WaterReset { COLOR_CONTROL_HOVER } else { 0x20FFFFFF };
    surface.fill_rounded_rect(x0 + 245.0, btn_y, 60.0, 24.0, 12.0, fade_color(b3_bg, alpha), false);
}

fn render_expanded_todo(surface: &mut Surface, x0: f32, y0: f32, cur_w: f32, alpha: f32, page_state: &PageState, hovered: HitTarget) {
    let head_y = y0 + 20.0;

    for (i, (_, done)) in page_state.tasks.iter().take(3).enumerate() {
        let row_y = head_y + 16.0 + (i as f32) * 28.0;
        let check_cx = x0 + 34.0;
        let check_cy = row_y + 10.0;

        if *done {
            surface.fill_circle(check_cx, check_cy, 8.0, fade_color(APPLE_BLUE, alpha));
            render_check_icon(surface, check_cx, check_cy, fade_color(COLOR_TEXT_WHITE, alpha));
        } else {
            surface.stroke_circle(check_cx, check_cy, 8.0, 1.5, fade_color(COLOR_TEXT_TERTIARY, alpha));
        }

        let is_hov = hovered == HitTarget::TaskItem(i);
        let row_bg = if is_hov { 0x1EFFFFFF } else { 0x08FFFFFF };
        surface.fill_rounded_rect(x0 + 18.0, row_y, cur_w - 36.0, 22.0, 6.0, fade_color(row_bg, alpha), false);
    }
}

fn render_expanded_shelf(surface: &mut Surface, x0: f32, y0: f32, cur_w: f32, alpha: f32, page_state: &PageState, hovered: HitTarget) {
    let head_y = y0 + 22.0;

    for (i, (_, _)) in page_state.shelf_items.iter().take(2).enumerate() {
        let card_y = head_y + 16.0 + (i as f32) * 38.0;
        let card_x = x0 + 20.0;
        let card_w = cur_w - 40.0;
        let card_h = 32.0;

        let is_hov = hovered == HitTarget::ShelfOpen(i);
        let card_bg = if is_hov { 0x2AFFFFFF } else { 0x18FFFFFF };

        surface.fill_rounded_rect(card_x, card_y, card_w, card_h, 8.0, fade_color(card_bg, alpha), false);
        surface.stroke_rounded_rect(card_x, card_y, card_w, card_h, 8.0, 1.0, fade_color(0x22FFFFFF, alpha), false);

        let badge_col = if i == 0 { APPLE_PURPLE } else { APPLE_ORANGE };
        surface.fill_rounded_rect(card_x + 8.0, card_y + 6.0, 20.0, 20.0, 5.0, fade_color(badge_col, alpha), false);

        let open_x = card_x + card_w - 60.0;
        surface.fill_rounded_rect(open_x, card_y + 5.0, 52.0, 22.0, 11.0, fade_color(0x33FFFFFF, alpha), false);
    }
}

/// Crisp GDI Typography Pass
pub unsafe fn render_gdi_text_pass(
    hdc: HDC,
    x0: i32,
    y0: i32,
    cur_w: i32,
    expand_t: f32,
    page_state: &PageState,
    media: &MediaSnapshot,
) {
    if expand_t < 0.3 {
        return;
    }

    SetBkMode(hdc, TRANSPARENT);

    match page_state.current_page {
        PageId::Music => {
            render_gdi_music(hdc, x0, y0, cur_w, page_state, media);
        }
        PageId::Pomodoro => {
            render_gdi_pomodoro(hdc, x0, y0, cur_w, page_state);
        }
        PageId::Water => {
            render_gdi_water(hdc, x0, y0, cur_w, page_state);
        }
        PageId::Todo => {
            render_gdi_todo(hdc, x0, y0, cur_w, page_state);
        }
        PageId::Shelf => {
            render_gdi_shelf(hdc, x0, y0, cur_w, page_state);
        }
        PageId::Hardware => {
            render_gdi_hardware(hdc, x0, y0, cur_w, page_state);
        }
        PageId::Zen => {
            render_gdi_zen(hdc, x0, y0, cur_w, page_state);
        }
    }
}

unsafe fn create_font(height: i32, weight: i32) -> HFONT {
    let mut lf: LOGFONTW = std::mem::zeroed();
    lf.lfHeight = -height;
    lf.lfWeight = weight;
    let face = "Segoe UI".encode_utf16().collect::<Vec<u16>>();
    for (i, &c) in face.iter().enumerate() {
        if i < lf.lfFaceName.len() - 1 {
            lf.lfFaceName[i] = c;
        }
    }
    CreateFontIndirectW(&lf)
}

fn fmt_time(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

unsafe fn render_gdi_music(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState, media: &MediaSnapshot) {
    let hfont_title = create_font(15, 700);
    let old_font = SelectObject(hdc, hfont_title);

    let (title, artist) = if media.has_session {
        (&media.title, &media.artist)
    } else {
        (&page_state.track_title, &page_state.track_artist)
    };

    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let mut rect_title = RECT {
        left: x0 + 82,
        top: y0 + 18,
        right: x0 + cur_w - 70,
        bottom: y0 + 38,
    };
    let mut title_utf16 = title.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut title_utf16, &mut rect_title, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    let hfont_sub = create_font(12, 500);
    SelectObject(hdc, hfont_sub);
    SetTextColor(hdc, COLORREF(0x00A1A1AA));
    let mut rect_artist = RECT {
        left: x0 + 82,
        top: y0 + 38,
        right: x0 + cur_w - 70,
        bottom: y0 + 54,
    };
    let mut artist_utf16 = artist.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut artist_utf16, &mut rect_artist, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    // Lyric / Source Line
    SetTextColor(hdc, COLORREF(0x00F8BD38));
    let mut rect_lyric = RECT {
        left: x0 + 82,
        top: y0 + 54,
        right: x0 + cur_w - 70,
        bottom: y0 + 72,
    };
    let source_str = if media.has_session && !media.source.is_empty() {
        format!("Playing via {}", media.source)
    } else {
        page_state.current_lyric.clone()
    };
    let mut lyric_utf16 = source_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut lyric_utf16, &mut rect_lyric, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    // Timestamps
    let hfont_mono = create_font(10, 500);
    SelectObject(hdc, hfont_mono);
    SetTextColor(hdc, COLORREF(0x0071717A));

    let (pos_str, dur_str) = if media.has_session && media.duration_ms > 0 {
        (fmt_time(media.position_ms_now()), fmt_time(media.duration_ms))
    } else {
        ("1:24".to_string(), "-2:36".to_string())
    };

    let mut rect_time_l = RECT {
        left: x0 + 20,
        top: y0 + 92,
        right: x0 + 80,
        bottom: y0 + 106,
    };
    let mut time_l = pos_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut time_l, &mut rect_time_l, DT_LEFT | DT_SINGLELINE);

    let mut rect_time_r = RECT {
        left: x0 + cur_w - 80,
        top: y0 + 92,
        right: x0 + cur_w - 20,
        bottom: y0 + 106,
    };
    let mut time_r = dur_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut time_r, &mut rect_time_r, DT_RIGHT | DT_SINGLELINE);

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont_title);
    let _ = DeleteObject(hfont_sub);
    let _ = DeleteObject(hfont_mono);
}

unsafe fn render_gdi_pomodoro(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState) {
    let hfont_title = create_font(22, 700);
    let old_font = SelectObject(hdc, hfont_title);

    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let mut rect_time = RECT {
        left: x0 + 84,
        top: y0 + 22,
        right: x0 + 220,
        bottom: y0 + 50,
    };
    let time_str = format!("{:02}:{:02}", page_state.pomo_seconds_left / 60, page_state.pomo_seconds_left % 60);
    let mut time_utf16 = time_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut time_utf16, &mut rect_time, DT_LEFT | DT_SINGLELINE);

    let hfont_sub = create_font(12, 500);
    SelectObject(hdc, hfont_sub);
    SetTextColor(hdc, COLORREF(0x000095FF));
    let mut rect_status = RECT {
        left: x0 + 84,
        top: y0 + 48,
        right: x0 + cur_w - 20,
        bottom: y0 + 66,
    };
    let mut status_utf16 = "Focus Session • 25m interval".encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut status_utf16, &mut rect_status, DT_LEFT | DT_SINGLELINE);

    let hfont_mode = create_font(11, 600);
    SelectObject(hdc, hfont_mode);
    let modes = ["25m Focus", "5m Break", "15m Rest"];
    let pill_w = 78;
    let spacing = 8;
    let total_w = 3 * pill_w + 2 * spacing;
    let start_x = x0 + (cur_w - total_w) / 2;

    for (i, label) in modes.iter().enumerate() {
        let px = start_x + (i as i32) * (pill_w + spacing);
        let mut rect_mode = RECT {
            left: px,
            top: y0 + 77,
            right: px + pill_w,
            bottom: y0 + 95,
        };
        let is_selected = i == page_state.pomo_mode;
        SetTextColor(hdc, if is_selected { COLORREF(0x000095FF) } else { COLORREF(0x00A1A1AA) });
        let mut text_utf16 = label.encode_utf16().collect::<Vec<u16>>();
        DrawTextW(hdc, &mut text_utf16, &mut rect_mode, DT_CENTER | DT_SINGLELINE);
    }

    let center_x = x0 + cur_w / 2;
    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let mut rect_start = RECT {
        left: center_x - 52,
        top: y0 + 107,
        right: center_x + 20,
        bottom: y0 + 128,
    };
    let start_lbl = if page_state.pomo_running { "Pause" } else { "Start" };
    let mut start_utf16 = start_lbl.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut start_utf16, &mut rect_start, DT_CENTER | DT_SINGLELINE);

    let mut rect_reset = RECT {
        left: center_x + 28,
        top: y0 + 107,
        right: center_x + 80,
        bottom: y0 + 128,
    };
    let mut reset_utf16 = "Reset".encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut reset_utf16, &mut rect_reset, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont_title);
    let _ = DeleteObject(hfont_sub);
    let _ = DeleteObject(hfont_mode);
}

unsafe fn render_gdi_water(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState) {
    let hfont_title = create_font(18, 700);
    let old_font = SelectObject(hdc, hfont_title);

    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let mut rect_stat = RECT {
        left: x0 + 75,
        top: y0 + 20,
        right: x0 + cur_w - 20,
        bottom: y0 + 44,
    };
    let stat_str = format!("{} / {} ml", page_state.water_taken_ml, page_state.water_goal_ml);
    let mut stat_utf16 = stat_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut stat_utf16, &mut rect_stat, DT_LEFT | DT_SINGLELINE);

    let hfont_sub = create_font(12, 500);
    SelectObject(hdc, hfont_sub);
    SetTextColor(hdc, COLORREF(0x00F8BD38));
    let mut rect_sub = RECT {
        left: x0 + 75,
        top: y0 + 44,
        right: x0 + cur_w - 20,
        bottom: y0 + 60,
    };
    let pct = (page_state.water_taken_ml * 100) / page_state.water_goal_ml;
    let sub_str = format!("{}% of daily hydration goal achieved", pct);
    let mut sub_utf16 = sub_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut sub_utf16, &mut rect_sub, DT_LEFT | DT_SINGLELINE);

    let hfont_btn = create_font(11, 600);
    SelectObject(hdc, hfont_btn);
    SetTextColor(hdc, COLORREF(0x00FFFFFF));

    let mut rect_b1 = RECT { left: x0 + 75, top: y0 + 100, right: x0 + 151, bottom: y0 + 120 };
    let mut b1_utf16 = "+250 ml".encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut b1_utf16, &mut rect_b1, DT_CENTER | DT_SINGLELINE);

    let mut rect_b2 = RECT { left: x0 + 160, top: y0 + 100, right: x0 + 236, bottom: y0 + 120 };
    let mut b2_utf16 = "+500 ml".encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut b2_utf16, &mut rect_b2, DT_CENTER | DT_SINGLELINE);

    SetTextColor(hdc, COLORREF(0x00A1A1AA));
    let mut rect_b3 = RECT { left: x0 + 245, top: y0 + 100, right: x0 + 305, bottom: y0 + 120 };
    let mut b3_utf16 = "Reset".encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut b3_utf16, &mut rect_b3, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont_title);
    let _ = DeleteObject(hfont_sub);
    let _ = DeleteObject(hfont_btn);
}

unsafe fn render_gdi_todo(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState) {
    let hfont_head = create_font(12, 700);
    let old_font = SelectObject(hdc, hfont_head);

    SetTextColor(hdc, COLORREF(0x00FF7A00));
    let mut rect_head = RECT {
        left: x0 + 20,
        top: y0 + 16,
        right: x0 + cur_w - 20,
        bottom: y0 + 32,
    };
    let done_count = page_state.tasks.iter().filter(|t| t.1).count();
    let head_str = format!("Reminders • {} of {} completed", done_count, page_state.tasks.len());
    let mut head_utf16 = head_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut head_utf16, &mut rect_head, DT_LEFT | DT_SINGLELINE);

    let hfont_task = create_font(13, 500);
    SelectObject(hdc, hfont_task);

    for (i, (text, done)) in page_state.tasks.iter().take(3).enumerate() {
        let row_y = y0 + 36 + (i as i32) * 28;
        let mut rect_t = RECT {
            left: x0 + 52,
            top: row_y + 3,
            right: x0 + cur_w - 30,
            bottom: row_y + 22,
        };
        if *done {
            SetTextColor(hdc, COLORREF(0x0071717A));
        } else {
            SetTextColor(hdc, COLORREF(0x00FFFFFF));
        }
        let mut t_utf16 = text.encode_utf16().collect::<Vec<u16>>();
        DrawTextW(hdc, &mut t_utf16, &mut rect_t, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);
    }

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont_head);
    let _ = DeleteObject(hfont_task);
}

unsafe fn render_gdi_shelf(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState) {
    let hfont_head = create_font(12, 700);
    let old_font = SelectObject(hdc, hfont_head);

    SetTextColor(hdc, COLORREF(0x00DE52AF));
    let mut rect_head = RECT {
        left: x0 + 20,
        top: y0 + 16,
        right: x0 + cur_w - 20,
        bottom: y0 + 32,
    };
    let head_str = "File Shelf • Drag & Drop Workspace".to_string();
    let mut head_utf16 = head_str.encode_utf16().collect::<Vec<u16>>();
    DrawTextW(hdc, &mut head_utf16, &mut rect_head, DT_LEFT | DT_SINGLELINE);

    let hfont_name = create_font(13, 600);
    let hfont_size = create_font(11, 400);

    for (i, (name, size)) in page_state.shelf_items.iter().take(2).enumerate() {
        let row_y = y0 + 38 + (i as i32) * 38;

        SelectObject(hdc, hfont_name);
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
        let mut rect_n = RECT {
            left: x0 + 56,
            top: row_y + 2,
            right: x0 + cur_w - 80,
            bottom: row_y + 18,
        };
        let mut n_utf16 = name.encode_utf16().collect::<Vec<u16>>();
        DrawTextW(hdc, &mut n_utf16, &mut rect_n, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

        SelectObject(hdc, hfont_size);
        SetTextColor(hdc, COLORREF(0x0071717A));
        let mut rect_s = RECT {
            left: x0 + 56,
            top: row_y + 18,
            right: x0 + cur_w - 80,
            bottom: row_y + 32,
        };
        let mut s_utf16 = size.encode_utf16().collect::<Vec<u16>>();
        DrawTextW(hdc, &mut s_utf16, &mut rect_s, DT_LEFT | DT_SINGLELINE);

        let mut rect_btn = RECT {
            left: x0 + cur_w - 80,
            top: row_y + 9,
            right: x0 + cur_w - 28,
            bottom: row_y + 27,
        };
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
        let mut btn_utf16 = "Open".encode_utf16().collect::<Vec<u16>>();
        DrawTextW(hdc, &mut btn_utf16, &mut rect_btn, DT_CENTER | DT_SINGLELINE);
    }

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont_head);
    let _ = DeleteObject(hfont_name);
    let _ = DeleteObject(hfont_size);
}

fn render_mascot_eyes(surface: &mut Surface, cx: f32, cy: f32, mascot: &Mascot, alpha: f32) {
    let half_spacing = mascot.eye_spacing * 0.5;
    let eye_h = (mascot.eye_radius * mascot.blink_progress).max(0.6);

    for sign in [-1.0, 1.0] {
        let ecx = cx + sign * half_spacing;
        let ecy = cy;

        surface.fill_rounded_rect(
            ecx - mascot.eye_radius,
            ecy - eye_h,
            mascot.eye_radius * 2.0,
            eye_h * 2.0,
            eye_h,
            fade_color(COLOR_TEXT_WHITE, alpha),
            false,
        );

        if mascot.blink_progress > 0.3 {
            let px = ecx + mascot.pupil_offset_x;
            let py = ecy + mascot.pupil_offset_y * mascot.blink_progress;
            let pupil_h = (mascot.pupil_radius * mascot.blink_progress).max(0.5);
            surface.fill_rounded_rect(
                px - mascot.pupil_radius,
                py - pupil_h,
                mascot.pupil_radius * 2.0,
                pupil_h * 2.0,
                pupil_h,
                fade_color(0xFF0369A1, alpha),
                false,
            );
        }
    }
}

pub fn render_check_icon(surface: &mut Surface, cx: f32, cy: f32, color: u32) {
    for t in 0..4 {
        let x = cx - 3.0 + t as f32 * 0.7;
        let y = cy - 0.5 + t as f32 * 0.8;
        surface.fill_rounded_rect(x, y, 1.6, 1.6, 0.8, color, false);
    }
    for t in 0..7 {
        let x = cx - 1.0 + t as f32 * 0.75;
        let y = cy + 2.0 - t as f32 * 0.8;
        surface.fill_rounded_rect(x, y, 1.6, 1.6, 0.8, color, false);
    }
}

fn render_play_icon(surface: &mut Surface, cx: f32, cy: f32, color: u32) {
    for i in 0..10 {
        let bar_h = (i as f32) * 1.3;
        let bx = cx - 4.0 + (i as f32);
        surface.fill_rounded_rect(bx, cy - bar_h * 0.5, 1.2, bar_h, 0.6, color, false);
    }
}

fn render_prev_icon(surface: &mut Surface, cx: f32, cy: f32, color: u32) {
    surface.fill_rounded_rect(cx - 5.0, cy - 5.0, 2.0, 10.0, 1.0, color, false);
    for i in 0..6 {
        let bar_h = (5 - i) as f32 * 1.6;
        let bx = cx - 2.0 + (i as f32);
        surface.fill_rounded_rect(bx, cy - bar_h * 0.5, 1.2, bar_h, 0.6, color, false);
    }
}

fn render_next_icon(surface: &mut Surface, cx: f32, cy: f32, color: u32) {
    surface.fill_rounded_rect(cx + 4.0, cy - 5.0, 2.0, 10.0, 1.0, color, false);
    for i in 0..6 {
        let bar_h = (i as f32) * 1.6;
        let bx = cx - 4.0 + (i as f32);
        surface.fill_rounded_rect(bx, cy - bar_h * 0.5, 1.2, bar_h, 0.6, color, false);
    }
}

pub fn fade_color(color: u32, alpha_factor: f32) -> u32 {
    let a = ((color >> 24) & 0xFF) as f32 * alpha_factor;
    let r = ((color >> 16) & 0xFF) as f32 * alpha_factor;
    let g = ((color >> 8) & 0xFF) as f32 * alpha_factor;
    let b = (color & 0xFF) as f32 * alpha_factor;
    ((a.round() as u32) << 24)
        | ((r.round() as u32) << 16)
        | ((g.round() as u32) << 8)
        | (b.round() as u32)
}

fn render_expanded_hardware(
    surface: &mut Surface,
    x0: f32,
    y0: f32,
    cur_w: f32,
    alpha: f32,
    page_state: &PageState,
    _hovered: HitTarget,
) {
    let card_x = x0 + 16.0;
    let card_w = cur_w - 32.0;
    let card_y = y0 + 60.0;

    // CPU multi-core distribution bars
    let cores = &page_state.hw_tab.cores;
    let bar_spacing = card_w / 8.0;
    for (i, &load) in cores.iter().enumerate() {
        let bx = card_x + (i as f32) * bar_spacing + 4.0;
        let bw = (bar_spacing - 8.0).max(4.0);
        let max_h = 44.0;
        let h = (load as f32 / 100.0 * max_h).max(3.0);
        let by = card_y + max_h - h + 18.0;
        let col = if load > 75 {
            APPLE_RED
        } else if load > 50 {
            APPLE_ORANGE
        } else {
            APPLE_GREEN
        };
        surface.fill_rounded_rect(bx, by, bw, h, 2.0, fade_color(col, alpha), false);
    }
}

fn render_expanded_zen(
    surface: &mut Surface,
    x0: f32,
    y0: f32,
    cur_w: f32,
    alpha: f32,
    page_state: &PageState,
    _hovered: HitTarget,
) {
    let cx = x0 + cur_w * 0.5;
    let cy = y0 + 104.0;
    let base_r = 18.0 + page_state.zen_tab.progress * 26.0;

    // Concentric breathing rings
    surface.fill_circle(cx, cy, base_r, fade_color(0x3338BDF8, alpha));
    surface.fill_circle(cx, cy, base_r * 0.7, fade_color(0x662DD4BF, alpha));
    surface.fill_circle(cx, cy, 6.0, fade_color(COLOR_TEXT_WHITE, alpha));
}

unsafe fn render_gdi_hardware(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState) {
    let hfont = create_font(13, 600);
    let old_font = SelectObject(hdc, hfont);

    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let mut title = format!(
        "CPU: {}%   RAM: {:.1}/{:.0} GB ({}%)",
        page_state.hw_tab.cpu_pct,
        page_state.hw_tab.ram_used_gb,
        page_state.hw_tab.ram_total_gb,
        page_state.hw_tab.ram_pct
    )
    .encode_utf16()
    .collect::<Vec<u16>>();

    let mut rect = RECT {
        left: x0 + 16,
        top: y0 + 44,
        right: x0 + cur_w - 16,
        bottom: y0 + 64,
    };
    DrawTextW(hdc, &mut title, &mut rect, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont);
}

unsafe fn render_gdi_zen(hdc: HDC, x0: i32, y0: i32, cur_w: i32, page_state: &PageState) {
    let hfont = create_font(14, 600);
    let old_font = SelectObject(hdc, hfont);

    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let phase_str = match page_state.zen_tab.phase {
        BreathPhase::Inhale => "Inhale (Deeply)",
        BreathPhase::Hold => "Hold Breath",
        BreathPhase::Exhale => "Exhale (Slowly)",
    };
    let mut text = format!("Zen: {}  •  Cycle {}", phase_str, page_state.zen_tab.completed_cycles)
        .encode_utf16()
        .collect::<Vec<u16>>();

    let mut rect = RECT {
        left: x0 + 16,
        top: y0 + 44,
        right: x0 + cur_w - 16,
        bottom: y0 + 66,
    };
    DrawTextW(hdc, &mut text, &mut rect, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, old_font);
    let _ = DeleteObject(hfont);
}

