//! Island chrome: the glass body, the light it catches, the shadow it
//! throws onto the wallpaper, and the interactive card pages.

use crate::activities::{FilesState, FocusState, TasksState, WaterState};
use crate::lyrics::{format_duration, Lyrics};
use crate::mascot::{Blink, Eyes, MascotGeometry};
use crate::media::Snapshot;
use crate::pager::{PAGE_COUNT, TAB_NAMES};
use crate::render::{Canvas, Mask, RectF, Rgba, RoundRect};
use crate::text::{Align, TextRenderer, FAMILY_DISPLAY, FAMILY_UI};
use crate::theme::{self, TextRole};

/// The island's geometric description for one frame.
#[derive(Clone, Copy, Debug)]
pub struct IslandShape {
    /// Outer silhouette.
    pub body: RoundRect,
    /// Vertical gap between the top of the canvas and the top of the island.
    pub gap: f32,
    /// 0 = compact pill, 1 = expanded card.
    pub expand: f32,
}

impl IslandShape {
    pub fn left(&self) -> f32 {
        self.body.center_x - self.body.half_w
    }
    pub fn right(&self) -> f32 {
        self.body.center_x + self.body.half_w
    }
    pub fn top(&self) -> f32 {
        self.body.center_y - self.body.half_h
    }
    pub fn bottom(&self) -> f32 {
        self.body.center_y + self.body.half_h
    }
}

/// How the island sits against the top of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mount {
    /// Flush against the bezel, top edge flat.
    Notch,
    /// Slightly below the bezel, every corner rounded.
    Floating,
}

impl Mount {
    pub fn gap(self) -> f32 {
        match self {
            Mount::Notch => theme::NOTCH_Y as f32,
            Mount::Floating => theme::FLOATING_Y as f32,
        }
    }
}

/// Build the silhouette for a given size and state.
pub fn build_shape(width: f32, height: f32, mount: Mount, expand: f32) -> IslandShape {
    let half_w = width * 0.5;
    let half_h = height * 0.5;
    let gap = mount.gap();

    let ratio = theme::PILL_CORNER_RATIO
        + (theme::CARD_CORNER_RATIO - theme::PILL_CORNER_RATIO) * expand.clamp(0.0, 1.0);
    let radius = (height * ratio).min(half_h);

    let radii = match mount {
        Mount::Notch => [0.0, 0.0, radius, radius],
        Mount::Floating => [radius; 4],
    };

    let body = RoundRect::new(
        theme::CANVAS_W as f32 * 0.5,
        gap + half_h,
        half_w,
        half_h,
        radii,
        theme::SQUIRCLE_EXPONENT,
    );

    IslandShape { body, gap, expand }
}

/// Draw the shadow the island casts on the wallpaper.
pub fn draw_shadow(canvas: &mut Canvas, mask: &mut Mask, shape: &IslandShape) {
    let offset = 9.0f32;
    let moved = RoundRect::new(
        shape.body.center_x,
        shape.body.center_y + offset,
        shape.body.half_w,
        shape.body.half_h,
        shape.body.radii,
        shape.body.exponent,
    );

    mask.clear();
    mask.fill_round_rect(&moved, 1.0);

    let b = moved.bounds();
    let pad = 26;
    mask.blur_in(
        (b.x as i32) - pad,
        (b.y as i32) - pad,
        (b.right() as i32) + pad,
        (b.bottom() as i32) + pad,
        11,
    );

    let shadow = theme::SHADOW;
    for y in 0..mask.height {
        for x in 0..mask.width {
            let i = y * mask.width + x;
            let cov = mask.data[i];
            if cov <= 0.004 {
                continue;
            }
            let mut c = shadow;
            c.a = (shadow.a as f32 * cov).round() as u8;
            canvas.blend(x as i32, y as i32, c);
        }
    }
}

/// Draw the glass body with its lighting.
pub fn draw_glass(canvas: &mut Canvas, shape: &IslandShape) {
    let body = &shape.body;
    let b = body.bounds();
    let x0 = b.x.floor() as i32;
    let y0 = b.y.floor() as i32;
    let x1 = b.right().ceil() as i32;
    let y1 = b.bottom().ceil() as i32;
    let span = body.half_h * 2.0;
    let top = body.center_y - body.half_h;

    for py in y0..y1 {
        for px in x0..x1 {
            let cov = body.coverage(px as f32 + 0.5, py as f32 + 0.5);
            if cov <= 0.0 {
                continue;
            }

            let t = ((py as f32 + 0.5) - top) / span.max(1.0);

            let mut c = theme::GLASS_TOP.lerp(theme::GLASS_BOTTOM, t);
            c.a = (c.a as f32 * cov).round() as u8;
            canvas.blend(px, py, c);

            if t < 0.44 {
                let fall = (1.0 - t / 0.44).powf(2.1);
                let mut s = theme::SPECULAR;
                s.a = (s.a as f32 * fall * cov).round() as u8;
                canvas.blend(px, py, s);
            }
            if t < 0.035 {
                let hot = (1.0 - t / 0.035).powf(1.4);
                let mut s = theme::SPECULAR_HOT;
                s.a = (s.a as f32 * hot * cov).round() as u8;
                canvas.blend(px, py, s);
            }
            if t > 0.82 {
                let fall = ((t - 0.82) / 0.18).powf(1.6);
                let mut s = theme::BOTTOM_SHADE;
                s.a = (s.a as f32 * fall * cov).round() as u8;
                canvas.blend(px, py, s);
            }
        }
    }

    canvas.stroke_round_rect(body, theme::HAIRLINE, 1.0);
}

/// Draw the compact (pill) contents: mascot, title, and a live meter.
pub fn draw_compact(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    shape: &IslandShape,
    eyes: &Eyes,
    blink: &Blink,
    geom: &MascotGeometry,
    title: &str,
    subtitle: &str,
    meter: Option<(f32, bool)>,
) {
    let alpha = (1.0 - shape.expand * 2.2).clamp(0.0, 1.0);
    if alpha <= 0.01 {
        return;
    }
    canvas.set_clip(None, Some(shape.body), alpha);

    let left = shape.left();
    let cy = shape.body.center_y;

    let mascot_x = left + 30.0;
    crate::mascot::draw_mascot(canvas, (mascot_x, cy), geom, eyes, blink, 1.0);

    let text_left = mascot_x + 20.0;
    let meter_reserve = if meter.is_some() { 56.0 } else { 0.0 };
    let right_edge = shape.right() - 16.0 - meter_reserve;

    if !title.is_empty() {
        text.draw(
            canvas,
            title,
            text_left,
            cy - 11.0,
            theme::TEXT_PRIMARY,
            FAMILY_DISPLAY,
            12.5,
            600,
            Align::Left,
            Some((right_edge - text_left).max(20.0)),
        );
    }
    if !subtitle.is_empty() {
        text.draw(
            canvas,
            subtitle,
            text_left,
            cy + 3.0,
            theme::TEXT_TERTIARY,
            FAMILY_UI,
            10.5,
            500,
            Align::Left,
            Some((right_edge - text_left).max(20.0)),
        );
    }

    if let Some((value, live)) = meter {
        let bar_w = 42.0;
        let x = shape.right() - 16.0 - bar_w;
        let y = cy;
        if live {
            for i in 0..4 {
                let h = 6.0 + 8.0 * value * (0.55 + 0.45 * ((i as f32) * 1.7).sin());
                let bx = x + 14.0 + i as f32 * 6.0;
                canvas.fill_rect(
                    RectF::new(bx, y - h * 0.5, 3.0, h),
                    theme::ACCENT,
                );
            }
        } else {
            canvas.fill_rect(
                RectF::new(x, y - 1.5, bar_w, 3.0),
                Rgba::rgba(255, 255, 255, 28),
            );
            let w = (bar_w * value.clamp(0.0, 1.0)).max(2.0);
            canvas.fill_rect(RectF::new(x, y - 1.5, w, 3.0), theme::ACCENT);
        }
    }

    canvas.set_clip(None, None, 1.0);
}

// ---------------------------------------------------------------------------
// Hit targets & actions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct TransportButtons {
    pub previous: (f32, f32),
    pub play_pause: (f32, f32),
    pub next: (f32, f32),
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportAction {
    Previous,
    PlayPause,
    Next,
}

impl TransportButtons {
    pub fn hit_test(&self, x: f32, y: f32) -> Option<TransportAction> {
        let r2 = (self.radius + 4.0) * (self.radius + 4.0);
        let dist2 = |c: (f32, f32)| (x - c.0) * (x - c.0) + (y - c.1) * (y - c.1);
        if dist2(self.previous) <= r2 {
            Some(TransportAction::Previous)
        } else if dist2(self.play_pause) <= r2 {
            Some(TransportAction::PlayPause)
        } else if dist2(self.next) <= r2 {
            Some(TransportAction::Next)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardAction {
    Transport(TransportAction),
    SelectTab(usize),
    FocusToggle,
    FocusReset,
    FocusSkip,
    WaterAdd(u32),
    WaterUndo,
    WaterReset,
    ToggleTask(usize),
    OpenFile(usize),
    ClearFiles,
    Collapse,
}

#[derive(Clone, Debug)]
pub struct CardHitTargets {
    pub transport: Option<TransportButtons>,
    pub tab_rects: [RectF; PAGE_COUNT],
    pub collapse_rect: RectF,
    pub focus_toggle: Option<RectF>,
    pub focus_reset: Option<RectF>,
    pub focus_skip: Option<RectF>,
    pub water_add: Option<RectF>,
    pub water_undo: Option<RectF>,
    pub water_reset: Option<RectF>,
    pub task_rows: Vec<(usize, RectF)>,
    pub file_rows: Vec<(usize, RectF)>,
    pub files_clear: Option<RectF>,
}

impl CardHitTargets {
    pub fn is_transport_hit(&self, x: f32, y: f32) -> bool {
        self.transport
            .as_ref()
            .and_then(|t| t.hit_test(x, y))
            .is_some()
    }

    pub fn hit_test(&self, x: f32, y: f32, active_page: usize) -> Option<CardAction> {
        if self.collapse_rect.contains(x, y) {
            return Some(CardAction::Collapse);
        }
        for (i, rect) in self.tab_rects.iter().enumerate() {
            if rect.contains(x, y) {
                return Some(CardAction::SelectTab(i));
            }
        }
        match active_page {
            0 => {
                if let Some(t) = &self.transport {
                    if let Some(act) = t.hit_test(x, y) {
                        return Some(CardAction::Transport(act));
                    }
                }
            }
            1 => {
                if let Some(r) = &self.focus_reset {
                    if r.contains(x, y) {
                        return Some(CardAction::FocusReset);
                    }
                }
                if let Some(r) = &self.focus_skip {
                    if r.contains(x, y) {
                        return Some(CardAction::FocusSkip);
                    }
                }
                if let Some(r) = &self.focus_toggle {
                    if r.contains(x, y) {
                        return Some(CardAction::FocusToggle);
                    }
                }
            }
            2 => {
                if let Some(r) = &self.water_add {
                    if r.contains(x, y) {
                        return Some(CardAction::WaterAdd(250));
                    }
                }
                if let Some(r) = &self.water_undo {
                    if r.contains(x, y) {
                        return Some(CardAction::WaterUndo);
                    }
                }
                if let Some(r) = &self.water_reset {
                    if r.contains(x, y) {
                        return Some(CardAction::WaterReset);
                    }
                }
            }
            3 => {
                for (idx, r) in &self.task_rows {
                    if r.contains(x, y) {
                        return Some(CardAction::ToggleTask(*idx));
                    }
                }
            }
            4 => {
                if let Some(r) = &self.files_clear {
                    if r.contains(x, y) {
                        return Some(CardAction::ClearFiles);
                    }
                }
                for (idx, r) in &self.file_rows {
                    if r.contains(x, y) {
                        return Some(CardAction::OpenFile(*idx));
                    }
                }
            }
            _ => {}
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Vector glyphs
// ---------------------------------------------------------------------------

fn fill_triangle(canvas: &mut Canvas, a: (f32, f32), b: (f32, f32), c: (f32, f32), color: Rgba) {
    let min_x = a.0.min(b.0).min(c.0).floor() as i32;
    let max_x = a.0.max(b.0).max(c.0).ceil() as i32;
    let min_y = a.1.min(b.1).min(c.1).floor() as i32;
    let max_y = a.1.max(b.1).max(c.1).ceil() as i32;

    let edge = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
        (x - p.0) * (q.1 - p.1) - (y - p.1) * (q.0 - p.0)
    };

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let x = px as f32 + 0.5;
            let y = py as f32 + 0.5;
            let w0 = edge(a, b, x, y);
            let w1 = edge(b, c, x, y);
            let w2 = edge(c, a, x, y);
            let inside =
                (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if !inside {
                continue;
            }
            canvas.blend(px, py, color);
        }
    }
}

fn draw_play_pause(canvas: &mut Canvas, center: (f32, f32), size: f32, playing: bool) {
    let (cx, cy) = center;
    if playing {
        let w = size * 0.26;
        let h = size * 0.86;
        canvas.fill_rect(
            RectF::new(cx - size * 0.44, cy - h * 0.5, w, h),
            theme::TEXT_PRIMARY,
        );
        canvas.fill_rect(
            RectF::new(cx + size * 0.18, cy - h * 0.5, w, h),
            theme::TEXT_PRIMARY,
        );
    } else {
        let r = size * 0.5;
        fill_triangle(
            canvas,
            (cx - r * 0.55, cy - r),
            (cx - r * 0.55, cy + r),
            (cx + r * 0.8, cy),
            theme::TEXT_PRIMARY,
        );
    }
}

fn draw_skip(canvas: &mut Canvas, center: (f32, f32), size: f32, forward: bool) {
    let (cx, cy) = center;
    let r = size * 0.42;
    let dir = if forward { 1.0 } else { -1.0 };
    for i in 0..2 {
        let off = (i as f32 - 0.5) * r * 1.15 * dir;
        let x = cx + off;
        fill_triangle(
            canvas,
            (x - r * 0.5 * dir, cy - r),
            (x - r * 0.5 * dir, cy + r),
            (x + r * 0.5 * dir, cy),
            theme::TEXT_SECONDARY,
        );
    }
}

fn draw_reset_glyph(canvas: &mut Canvas, cx: f32, cy: f32, color: Rgba) {
    canvas.draw_ring_arc(
        cx,
        cy,
        6.0,
        1.8,
        0.76,
        Rgba::rgba(0, 0, 0, 0),
        color,
    );
    fill_triangle(
        canvas,
        (cx - 1.5, cy - 9.0),
        (cx - 1.5, cy - 3.0),
        (cx + 3.0, cy - 6.0),
        color,
    );
}

// ---------------------------------------------------------------------------
// Expanded card & pages
// ---------------------------------------------------------------------------

pub struct CardContext<'a> {
    pub snapshot: Option<&'a Snapshot>,
    pub lyrics: Option<&'a Lyrics>,
    pub focus: &'a FocusState,
    pub water: &'a WaterState,
    pub tasks: &'a TasksState,
    pub files: &'a FilesState,
    pub pager_scroll_x: f32,
    pub pager_page_pos: f32,
    pub active_page: usize,
    pub card_width: f32,
}

/// Draw the expanded card with horizontal pager pages, tab rail, and pagination dots.
pub fn draw_card(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    shape: &IslandShape,
    eyes: &Eyes,
    blink: &Blink,
    geom: &MascotGeometry,
    ctx: &CardContext<'_>,
) -> CardHitTargets {
    let left = shape.left();
    let top = shape.top();
    let width = shape.body.half_w * 2.0;
    let bottom = shape.bottom();

    let mut hits = CardHitTargets {
        transport: None,
        tab_rects: [RectF::new(0.0, 0.0, 0.0, 0.0); PAGE_COUNT],
        collapse_rect: RectF::new(left, top, width, 14.0),
        focus_toggle: None,
        focus_reset: None,
        focus_skip: None,
        water_add: None,
        water_undo: None,
        water_reset: None,
        task_rows: Vec::new(),
        file_rows: Vec::new(),
        files_clear: None,
    };

    let card_alpha = ((shape.expand - 0.20) / 0.80).clamp(0.0, 1.0);
    if card_alpha <= 0.01 {
        return hits;
    }
    let clip_rect = (
        (left + 2.0).floor() as i32,
        (top + 2.0).floor() as i32,
        (shape.right() - 2.0).ceil() as i32,
        (bottom - 2.0).ceil() as i32,
    );
    canvas.set_clip(Some(clip_rect), Some(shape.body), card_alpha);

    // Subtle top notch pill handle so users know clicking the top edge collapses.
    let handle = RoundRect::new(
        shape.body.center_x,
        top + 7.0,
        18.0,
        1.8,
        [1.8; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    canvas.fill_round_rect(&handle, Rgba::rgba(255, 255, 255, 36));

    // Clip page content strictly above the tab rail separator.
    let content_clip = (
        (left + 4.0).floor() as i32,
        (top + 12.0).floor() as i32,
        (shape.right() - 4.0).ceil() as i32,
        (bottom - 46.0).floor() as i32,
    );
    canvas.set_clip(Some(content_clip), Some(shape.body), card_alpha);

    for page_idx in 0..PAGE_COUNT {
        let page_dx = page_idx as f32 * ctx.card_width - ctx.pager_scroll_x;
        if page_dx.abs() > ctx.card_width + 24.0 {
            continue;
        }
        let page_left = left + page_dx;
        match page_idx {
            0 => {
                let btns = draw_media_page(
                    canvas,
                    text,
                    page_left,
                    top,
                    width,
                    eyes,
                    blink,
                    geom,
                    ctx.snapshot,
                    ctx.lyrics,
                );
                if page_idx == ctx.active_page {
                    hits.transport = Some(btns);
                }
            }
            1 => {
                let (toggle, reset, skip) =
                    draw_focus_page(canvas, text, page_left, top, width, ctx.focus);
                if page_idx == ctx.active_page {
                    hits.focus_toggle = Some(toggle);
                    hits.focus_reset = Some(reset);
                    hits.focus_skip = Some(skip);
                }
            }
            2 => {
                let (add, undo, reset) =
                    draw_water_page(canvas, text, page_left, top, width, ctx.water);
                if page_idx == ctx.active_page {
                    hits.water_add = Some(add);
                    hits.water_undo = Some(undo);
                    hits.water_reset = Some(reset);
                }
            }
            3 => {
                let rows = draw_tasks_page(canvas, text, page_left, top, width, ctx.tasks);
                if page_idx == ctx.active_page {
                    hits.task_rows = rows;
                }
            }
            4 => {
                let (rows, clear) =
                    draw_files_page(canvas, text, page_left, top, width, ctx.files);
                if page_idx == ctx.active_page {
                    hits.file_rows = rows;
                    hits.files_clear = clear;
                }
            }
            _ => {}
        }
    }

    // Restore full card clip for the fixed bottom chrome (separator, tab rail, dots).
    canvas.set_clip(Some(clip_rect), Some(shape.body), card_alpha);

    // Hairline separator above the tab rail.
    let sep_y = bottom - 46.0;
    canvas.fill_rect(
        RectF::new(left + 22.0, sep_y, width - 44.0, 1.0),
        theme::SEPARATOR,
    );

    // --- Tab rail -----------------------------------------------------------
    let rail_y = bottom - 40.0;
    let spacing = 84.0;
    let start = shape.body.center_x - (PAGE_COUNT as f32 - 1.0) * spacing * 0.5;
    for (i, name) in TAB_NAMES.iter().enumerate() {
        let x = start + i as f32 * spacing;
        let active = i == ctx.active_page;
        hits.tab_rects[i] = RectF::new(x - 38.0, rail_y - 2.0, 76.0, 20.0);
        text.draw(
            canvas,
            name,
            x,
            rail_y,
            if active {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_TERTIARY
            },
            FAMILY_UI,
            10.5,
            if active { 600 } else { 500 },
            Align::Center,
            Some(74.0),
        );
    }

    // --- Apple-style pagination dots ---------------------------------------
    let dots_y = bottom - 13.0;
    let page_pos = ctx.pager_page_pos;
    let min_w = 5.0f32;
    let max_w = 18.0f32;
    let dot_h = 5.0f32;
    let dot_gap = 6.0f32;

    let mut weights = [0.0f32; PAGE_COUNT];
    let mut widths = [min_w; PAGE_COUNT];
    let mut total_w = (PAGE_COUNT as f32 - 1.0) * dot_gap;
    for i in 0..PAGE_COUNT {
        let w = (1.0 - (page_pos - i as f32).abs()).clamp(0.0, 1.0);
        weights[i] = w;
        widths[i] = min_w + (max_w - min_w) * w;
        total_w += widths[i];
    }

    let mut cursor_x = shape.body.center_x - total_w * 0.5;
    let inactive_dot = Rgba::rgba(255, 255, 255, 58);
    for i in 0..PAGE_COUNT {
        let dw = widths[i];
        let cx = cursor_x + dw * 0.5;
        let color = inactive_dot.lerp(theme::TEXT_PRIMARY, weights[i]);
        let pill = RoundRect::new(
            cx,
            dots_y,
            dw * 0.5,
            dot_h * 0.5,
            [dot_h * 0.5; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&pill, color);
        cursor_x += dw + dot_gap;
    }

    canvas.set_clip(None, None, 1.0);
    hits
}

// ---------------------------------------------------------------------------
// Page 0: Media
// ---------------------------------------------------------------------------

fn draw_media_page(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    left: f32,
    top: f32,
    width: f32,
    eyes: &Eyes,
    blink: &Blink,
    geom: &MascotGeometry,
    snapshot: Option<&Snapshot>,
    lyrics: Option<&Lyrics>,
) -> TransportButtons {
    let pad = 24.0;
    let right = left + width;

    let (title, subtitle, playing, pos_ms, duration_ms, art) = match snapshot {
        Some(s) => {
            let dur = if s.duration_ms > 0 {
                s.duration_ms
            } else {
                lyrics.and_then(|l| l.duration_ms).unwrap_or(0)
            };
            (
                s.title.as_str(),
                s.artist.as_str(),
                s.playing,
                s.position_now(),
                dur,
                s.art.as_ref(),
            )
        }
        None => ("", "", false, 0, 0, None),
    };

    let progress = if duration_ms > 0 {
        (pos_ms as f32 / duration_ms as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let art_size = 56.0f32;
    let art_x = left + pad;
    let art_y = top + 16.0;
    let art_center = (art_x + art_size * 0.5, art_y + art_size * 0.5);

    if let Some(a) = art {
        let art_rr = RoundRect::new(
            art_size * 0.5,
            art_size * 0.5,
            art_size * 0.5,
            art_size * 0.5,
            [8.0; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.blit_bgra_scaled(
            art_x as i32,
            art_y as i32,
            art_size as usize,
            art_size as usize,
            &a.data,
            a.width,
            a.height,
            Some(&art_rr),
        );
    } else {
        crate::mascot::draw_mascot(canvas, art_center, geom, eyes, blink, 1.55);
    }

    let text_x = art_x + art_size + 16.0;
    let transport_x = right - pad - 86.0;
    let display_title = if title.is_empty() { "Nothing Playing" } else { title };
    let display_sub = if !subtitle.is_empty() {
        subtitle
    } else if title.is_empty() {
        "Play audio in any app"
    } else {
        "Playing"
    };

    text.draw(
        canvas,
        display_title,
        text_x,
        top + 20.0,
        theme::TEXT_PRIMARY,
        FAMILY_DISPLAY,
        TextRole::Title.size(),
        TextRole::Title.weight(),
        Align::Left,
        Some((transport_x - text_x - 12.0).max(40.0)),
    );
    text.draw(
        canvas,
        display_sub,
        text_x,
        top + 42.0,
        theme::TEXT_SECONDARY,
        FAMILY_UI,
        TextRole::Caption.size(),
        TextRole::Caption.weight(),
        Align::Left,
        Some((transport_x - text_x - 12.0).max(40.0)),
    );

    let btn_y = art_center.1;
    let radius = 15.0;
    let previous = (transport_x + 4.0, btn_y);
    let play_pause = (transport_x + 38.0, btn_y);
    let next = (transport_x + 72.0, btn_y);

    for (i, center) in [previous, play_pause, next].iter().enumerate() {
        let fill = if i == 1 && playing {
            theme::CONTROL_FILL_ACTIVE
        } else {
            theme::CONTROL_FILL
        };
        let pill = RoundRect::new(
            center.0,
            center.1,
            radius,
            radius,
            [radius; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&pill, fill);
    }

    draw_skip(canvas, previous, 9.0, false);
    draw_play_pause(canvas, play_pause, 9.0, playing);
    draw_skip(canvas, next, 9.0, true);

    // Progress bar + timestamps.
    let bar_y = top + 86.0;
    let bar_w = width - pad * 2.0;
    let track = RoundRect::new(
        left + pad + bar_w * 0.5,
        bar_y,
        bar_w * 0.5,
        2.0,
        [2.0; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    canvas.fill_round_rect(&track, Rgba::rgba(255, 255, 255, 26));

    let fill_w = (bar_w * progress).max(0.0);
    if fill_w > 0.5 {
        let fill = RoundRect::new(
            left + pad + fill_w * 0.5,
            bar_y,
            fill_w * 0.5,
            2.0,
            [2.0; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&fill, theme::ACCENT);
    }

    if duration_ms > 0 {
        let pos_str = format_duration(pos_ms);
        let dur_str = format_duration(duration_ms);
        text.draw(
            canvas,
            &pos_str,
            left + pad,
            bar_y + 6.0,
            theme::TEXT_TERTIARY,
            FAMILY_UI,
            10.0,
            500,
            Align::Left,
            None,
        );
        text.draw(
            canvas,
            &dur_str,
            right - pad,
            bar_y + 6.0,
            theme::TEXT_TERTIARY,
            FAMILY_UI,
            10.0,
            500,
            Align::Right,
            None,
        );
    }

    // Active & upcoming synced lyrics.
    let active_lyric = lyrics.and_then(|l| l.active_text(pos_ms));
    let next_lyric = lyrics.and_then(|l| l.next_text(pos_ms));
    let (lyric_text, lyric_color) = match active_lyric {
        Some(line) if !line.is_empty() => (line, theme::TEXT_PRIMARY),
        _ if title.is_empty() => ("Swipe or click tabs below  ·  Isle", theme::TEXT_TERTIARY),
        _ => ("Synced lyrics appear here", theme::TEXT_TERTIARY),
    };
    let lyric_max_w = (bar_w - 96.0).max(120.0);
    text.draw(
        canvas,
        lyric_text,
        left + width * 0.5,
        bar_y + 14.0,
        lyric_color,
        FAMILY_UI,
        12.0,
        500,
        Align::Center,
        Some(lyric_max_w),
    );
    if let Some(next_line) = next_lyric {
        if !next_line.is_empty() {
            text.draw(
                canvas,
                next_line,
                left + width * 0.5,
                bar_y + 32.0,
                theme::TEXT_TERTIARY,
                FAMILY_UI,
                10.5,
                400,
                Align::Center,
                Some(lyric_max_w),
            );
        }
    }

    TransportButtons {
        previous,
        play_pause,
        next,
        radius,
    }
}

// ---------------------------------------------------------------------------
// Page 1: Focus / Pomodoro
// ---------------------------------------------------------------------------

fn draw_focus_page(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    left: f32,
    top: f32,
    width: f32,
    focus: &FocusState,
) -> (RectF, RectF, RectF) {
    let pad = 26.0;
    let right = left + width;
    let phase_color = focus.phase.color();

    // Circular progress ring on the left.
    let ring_cx = left + pad + 40.0;
    let ring_cy = top + 76.0;
    canvas.draw_ring_arc(
        ring_cx,
        ring_cy,
        32.0,
        5.5,
        focus.progress(),
        Rgba::rgba(255, 255, 255, 24),
        phase_color,
    );
    let session_label = format!("#{}", focus.completed_sessions + 1);
    text.draw(
        canvas,
        &session_label,
        ring_cx,
        ring_cy - 8.0,
        theme::TEXT_SECONDARY,
        FAMILY_DISPLAY,
        13.0,
        600,
        Align::Center,
        None,
    );

    // Phase badge pill (clickable to skip phase).
    let info_x = ring_cx + 54.0;
    let badge_w = 96.0;
    let badge_h = 20.0;
    let badge_rect = RectF::new(info_x, top + 22.0, badge_w, badge_h);
    let badge_rr = RoundRect::new(
        badge_rect.center_x(),
        badge_rect.center_y(),
        badge_w * 0.5,
        badge_h * 0.5,
        [badge_h * 0.5; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    let mut badge_bg = phase_color;
    badge_bg.a = 36;
    canvas.fill_round_rect(&badge_rr, badge_bg);
    text.draw(
        canvas,
        focus.phase.label(),
        badge_rect.center_x(),
        badge_rect.y + 3.0,
        phase_color,
        FAMILY_UI,
        TextRole::Micro.size(),
        TextRole::Micro.weight(),
        Align::Center,
        Some(badge_w - 8.0),
    );

    // Large digital countdown.
    let clock = focus.formatted_time();
    text.draw(
        canvas,
        &clock,
        info_x,
        top + 46.0,
        theme::TEXT_PRIMARY,
        FAMILY_DISPLAY,
        TextRole::Display.size(),
        TextRole::Display.weight(),
        Align::Left,
        None,
    );

    let hint = if focus.running {
        "Running  ·  Click to pause"
    } else {
        "Click to start  ·  Right-click to reset"
    };
    text.draw(
        canvas,
        hint,
        info_x,
        top + 104.0,
        theme::TEXT_TERTIARY,
        FAMILY_UI,
        11.0,
        500,
        Align::Left,
        Some(260.0),
    );

    // Right-side controls: Start/Pause pill + Reset icon button.
    let reset_r = 17.0;
    let reset_cx = right - pad - reset_r;
    let reset_cy = top + 76.0;
    let reset_rect = RectF::new(
        reset_cx - reset_r,
        reset_cy - reset_r,
        reset_r * 2.0,
        reset_r * 2.0,
    );
    let reset_rr = RoundRect::new(
        reset_cx,
        reset_cy,
        reset_r,
        reset_r,
        [reset_r; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    canvas.fill_round_rect(&reset_rr, theme::CONTROL_FILL);
    draw_reset_glyph(canvas, reset_cx, reset_cy, theme::TEXT_SECONDARY);

    let btn_w = 96.0;
    let btn_h = 34.0;
    let btn_x = reset_rect.x - 12.0 - btn_w;
    let btn_y = reset_cy - btn_h * 0.5;
    let btn_rect = RectF::new(btn_x, btn_y, btn_w, btn_h);
    let btn_rr = RoundRect::new(
        btn_rect.center_x(),
        btn_rect.center_y(),
        btn_w * 0.5,
        btn_h * 0.5,
        [btn_h * 0.5; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    canvas.fill_round_rect(
        &btn_rr,
        if focus.running {
            theme::CONTROL_FILL_ACTIVE
        } else {
            theme::CONTROL_FILL_HOVER
        },
    );
    text.draw(
        canvas,
        if focus.running { "Pause" } else { "Start" },
        btn_rect.center_x(),
        btn_rect.y + 8.0,
        theme::TEXT_PRIMARY,
        FAMILY_UI,
        12.5,
        600,
        Align::Center,
        None,
    );

    // Clicking either the timer/ring area or the Start/Pause button toggles Focus.
    let toggle_rect = RectF::new(left + pad, top + 20.0, btn_rect.right() - (left + pad), 112.0);
    (toggle_rect, reset_rect, badge_rect)
}

// ---------------------------------------------------------------------------
// Page 2: Hydration (Water Tracker)
// ---------------------------------------------------------------------------

fn draw_water_page(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    left: f32,
    top: f32,
    width: f32,
    water: &WaterState,
) -> (RectF, RectF, RectF) {
    let pad = 26.0;
    let right = left + width;
    let progress = water.progress();
    let goal_met = water.intake_ml >= water.goal_ml;
    let accent = if goal_met { theme::SUCCESS } else { theme::ACCENT };

    // Percentage ring on the left.
    let ring_cx = left + pad + 40.0;
    let ring_cy = top + 76.0;
    canvas.draw_ring_arc(
        ring_cx,
        ring_cy,
        32.0,
        5.5,
        progress,
        Rgba::rgba(255, 255, 255, 24),
        accent,
    );
    let pct = format!("{}%", (progress * 100.0).round() as u32);
    text.draw(
        canvas,
        &pct,
        ring_cx,
        ring_cy - 9.0,
        theme::TEXT_PRIMARY,
        FAMILY_DISPLAY,
        14.0,
        600,
        Align::Center,
        None,
    );

    // Center readout + fluid level bar.
    let info_x = ring_cx + 54.0;
    text.draw(
        canvas,
        if goal_met { "HYDRATION  ·  GOAL MET" } else { "HYDRATION" },
        info_x,
        top + 24.0,
        accent,
        FAMILY_UI,
        TextRole::Micro.size(),
        TextRole::Micro.weight(),
        Align::Left,
        None,
    );

    let total_str = water.formatted_total();
    text.draw(
        canvas,
        &total_str,
        info_x,
        top + 44.0,
        theme::TEXT_PRIMARY,
        FAMILY_DISPLAY,
        22.0,
        600,
        Align::Left,
        None,
    );

    let meter_w = 260.0;
    let meter_h = 8.0;
    let meter_y = top + 84.0;
    let meter_track = RoundRect::new(
        info_x + meter_w * 0.5,
        meter_y,
        meter_w * 0.5,
        meter_h * 0.5,
        [meter_h * 0.5; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    canvas.fill_round_rect(&meter_track, Rgba::rgba(255, 255, 255, 22));
    let fill_w = (meter_w * progress).max(0.0);
    if fill_w > 1.0 {
        let meter_fill = RoundRect::new(
            info_x + fill_w * 0.5,
            meter_y,
            fill_w * 0.5,
            meter_h * 0.5,
            [meter_h * 0.5; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&meter_fill, accent);
    }

    let sub = if goal_met {
        "Daily hydration target reached".to_string()
    } else {
        format!("{} ml remaining today", water.goal_ml.saturating_sub(water.intake_ml))
    };
    text.draw(
        canvas,
        &sub,
        info_x,
        top + 100.0,
        theme::TEXT_TERTIARY,
        FAMILY_UI,
        11.0,
        500,
        Align::Left,
        Some(260.0),
    );

    // Right-side tactile +250 ml button (with spring bounce) + Undo / Reset.
    let bounce = water.bounce.value.clamp(-0.3, 1.2);
    let add_w = 116.0 + bounce * 6.0;
    let add_h = 36.0 + bounce * 3.0;
    let add_cx = right - pad - 58.0;
    let add_cy = top + 54.0;
    let add_rect = RectF::new(add_cx - add_w * 0.5, add_cy - add_h * 0.5, add_w, add_h);
    let add_rr = RoundRect::new(
        add_cx,
        add_cy,
        add_w * 0.5,
        add_h * 0.5,
        [add_h * 0.5; 4],
        theme::SQUIRCLE_EXPONENT,
    );
    let add_fill = theme::CONTROL_FILL_ACTIVE.lerp(theme::ACCENT, (bounce * 0.45).clamp(0.0, 0.6));
    canvas.fill_round_rect(&add_rr, add_fill);
    text.draw(
        canvas,
        "+250 ml",
        add_cx,
        add_cy - 9.0,
        theme::TEXT_PRIMARY,
        FAMILY_DISPLAY,
        13.0,
        600,
        Align::Center,
        None,
    );

    let sub_w = 54.0;
    let sub_h = 24.0;
    let sub_y = top + 86.0;
    let undo_rect = RectF::new(right - pad - 116.0, sub_y, sub_w, sub_h);
    let reset_rect = RectF::new(right - pad - sub_w, sub_y, sub_w, sub_h);

    for (rect, label) in [(undo_rect, "Undo"), (reset_rect, "Reset")] {
        let rr = RoundRect::new(
            rect.center_x(),
            rect.center_y(),
            sub_w * 0.5,
            sub_h * 0.5,
            [sub_h * 0.5; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&rr, theme::CONTROL_FILL);
        text.draw(
            canvas,
            label,
            rect.center_x(),
            rect.y + 4.0,
            theme::TEXT_SECONDARY,
            FAMILY_UI,
            10.5,
            500,
            Align::Center,
            None,
        );
    }

    (add_rect, undo_rect, reset_rect)
}

// ---------------------------------------------------------------------------
// Page 3: Quick Reminders / Tasks
// ---------------------------------------------------------------------------

fn draw_tasks_page(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    left: f32,
    top: f32,
    width: f32,
    tasks: &TasksState,
) -> Vec<(usize, RectF)> {
    let pad = 24.0;
    let right = left + width;
    let row_w = width - pad * 2.0;

    text.draw(
        canvas,
        "REMINDERS",
        left + pad,
        top + 18.0,
        theme::TEXT_SECONDARY,
        FAMILY_UI,
        TextRole::Micro.size(),
        TextRole::Micro.weight(),
        Align::Left,
        None,
    );
    let summary = format!("{}/{} done", tasks.completed_count(), tasks.items.len());
    text.draw(
        canvas,
        &summary,
        right - pad,
        top + 17.0,
        theme::TEXT_TERTIARY,
        FAMILY_UI,
        11.0,
        500,
        Align::Right,
        None,
    );

    let mut rows = Vec::new();
    let start_y = top + 38.0;
    let row_h = 22.0;
    let row_gap = 3.0;

    for (i, item) in tasks.items.iter().take(4).enumerate() {
        let ry = start_y + i as f32 * (row_h + row_gap);
        let rect = RectF::new(left + pad, ry, row_w, row_h);
        let rr = RoundRect::new(
            rect.center_x(),
            rect.center_y(),
            row_w * 0.5,
            row_h * 0.5,
            [8.0; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&rr, Rgba::rgba(255, 255, 255, 10));

        let cx = rect.x + 14.0;
        let cy = rect.center_y();
        if item.completed {
            canvas.fill_circle(cx, cy, 6.5, item.color);
            canvas.draw_line(cx - 3.0, cy + 0.2, cx - 0.6, cy + 2.6, 1.6, Rgba::rgb(12, 13, 17));
            canvas.draw_line(cx - 0.6, cy + 2.6, cx + 3.4, cy - 2.2, 1.6, Rgba::rgb(12, 13, 17));
        } else {
            canvas.draw_ring_arc(cx, cy, 5.8, 1.6, 1.0, item.color, item.color);
        }

        let title_x = rect.x + 30.0;
        let title_color = if item.completed {
            theme::TEXT_TERTIARY
        } else {
            theme::TEXT_PRIMARY
        };
        let adv = text.draw(
            canvas,
            &item.title,
            title_x,
            ry + 3.0,
            title_color,
            FAMILY_UI,
            12.0,
            if item.completed { 400 } else { 500 },
            Align::Left,
            Some(row_w - 56.0),
        );
        if item.completed && adv > 1.0 {
            canvas.draw_line(
                title_x,
                cy + 0.5,
                title_x + adv,
                cy + 0.5,
                1.1,
                theme::TEXT_TERTIARY,
            );
        }

        // Category dot on the right.
        let mut dot_c = item.color;
        if item.completed {
            dot_c.a = 90;
        }
        canvas.fill_circle(rect.right() - 12.0, cy, 3.0, dot_c);

        rows.push((i, rect));
    }

    rows
}

// ---------------------------------------------------------------------------
// Page 4: Quick File Shelf
// ---------------------------------------------------------------------------

fn draw_files_page(
    canvas: &mut Canvas,
    text: &mut TextRenderer,
    left: f32,
    top: f32,
    width: f32,
    files: &FilesState,
) -> (Vec<(usize, RectF)>, Option<RectF>) {
    let pad = 24.0;
    let right = left + width;
    let row_w = width - pad * 2.0;

    text.draw(
        canvas,
        "QUICK SHELF",
        left + pad,
        top + 18.0,
        theme::TEXT_SECONDARY,
        FAMILY_UI,
        TextRole::Micro.size(),
        TextRole::Micro.weight(),
        Align::Left,
        None,
    );

    let mut clear_rect = None;
    if !files.items.is_empty() {
        let cr = RectF::new(right - pad - 48.0, top + 14.0, 48.0, 18.0);
        let rr = RoundRect::new(
            cr.center_x(),
            cr.center_y(),
            24.0,
            9.0,
            [9.0; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&rr, theme::CONTROL_FILL);
        text.draw(
            canvas,
            "Clear",
            cr.center_x(),
            cr.y + 2.0,
            theme::DANGER,
            FAMILY_UI,
            10.0,
            500,
            Align::Center,
            None,
        );
        clear_rect = Some(cr);
    }

    let mut rows = Vec::new();
    if files.items.is_empty() {
        let box_rect = RectF::new(left + pad, top + 42.0, row_w, 82.0);
        let rr = RoundRect::new(
            box_rect.center_x(),
            box_rect.center_y(),
            row_w * 0.5,
            41.0,
            [12.0; 4],
            theme::SQUIRCLE_EXPONENT,
        );
        canvas.fill_round_rect(&rr, Rgba::rgba(255, 255, 255, 8));
        canvas.stroke_round_rect(&rr, Rgba::rgba(255, 255, 255, 22), 1.0);
        text.draw(
            canvas,
            "Drop files onto the island to pin them here",
            box_rect.center_x(),
            box_rect.y + 24.0,
            theme::TEXT_SECONDARY,
            FAMILY_UI,
            TextRole::Body.size(),
            TextRole::Body.weight(),
            Align::Center,
            Some(row_w - 24.0),
        );
        text.draw(
            canvas,
            "Click any pinned file to launch it",
            box_rect.center_x(),
            box_rect.y + 44.0,
            theme::TEXT_TERTIARY,
            FAMILY_UI,
            10.5,
            400,
            Align::Center,
            Some(row_w - 24.0),
        );
    } else {
        let start_y = top + 38.0;
        let row_h = 22.0;
        let row_gap = 3.0;
        for (i, file) in files.items.iter().take(4).enumerate() {
            let ry = start_y + i as f32 * (row_h + row_gap);
            let rect = RectF::new(left + pad, ry, row_w, row_h);
            let rr = RoundRect::new(
                rect.center_x(),
                rect.center_y(),
                row_w * 0.5,
                row_h * 0.5,
                [8.0; 4],
                theme::SQUIRCLE_EXPONENT,
            );
            canvas.fill_round_rect(&rr, Rgba::rgba(255, 255, 255, 12));
            canvas.fill_circle(rect.x + 12.0, rect.center_y(), 3.0, theme::ACCENT);
            text.draw(
                canvas,
                &file.name,
                rect.x + 24.0,
                ry + 3.0,
                theme::TEXT_PRIMARY,
                FAMILY_UI,
                12.0,
                500,
                Align::Left,
                Some(row_w - 80.0),
            );
            text.draw(
                canvas,
                "Open",
                rect.right() - 10.0,
                ry + 4.0,
                theme::TEXT_TERTIARY,
                FAMILY_UI,
                10.5,
                500,
                Align::Right,
                None,
            );
            rows.push((i, rect));
        }
    }

    (rows, clear_rect)
}

/// Fill a region with a nearly invisible but *hit-testable* alpha.
pub fn draw_hit_rail(canvas: &mut Canvas, shape: &IslandShape) {
    if shape.gap <= 0.5 {
        return;
    }
    let rail = Rgba::rgba(0, 0, 0, 1);
    let x0 = shape.left() as i32;
    let x1 = shape.right() as i32;
    for py in 0..shape.gap.floor() as i32 {
        for px in x0..x1 {
            canvas.blend(px, py, rail);
        }
    }
}
