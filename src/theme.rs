//! Isle's design system: the single source of truth for color, type, spacing
//! and motion.
//!
//! The visual language targets a "living" hardware object rather than a flat
//! app window: an OLED-black glass slab with a soft directional highlight,
//! a diffused shadow onto the wallpaper, and typography that stays razor
//! sharp at small sizes.

use crate::render::Rgba;

// ---------------------------------------------------------------------------
// Canvas geometry
// ---------------------------------------------------------------------------

/// The fixed transparent buffer size. Never resized at runtime — all morphing
/// happens inside this buffer so the OS window handle stays stable (resizing
/// a layered window causes DWM stutter).
pub const CANVAS_W: u32 = 740;
pub const CANVAS_H: u32 = 220;

/// Vertical offset from the top of the monitor's work area.
pub const NOTCH_Y: i32 = 0;
pub const FLOATING_Y: i32 = 10;

// ---------------------------------------------------------------------------
// Shape metrics
// ---------------------------------------------------------------------------

/// Superellipse exponent. 4 is the sweet spot: visibly squircle, still reads
/// as a rounded rectangle at pill sizes.
pub const SQUIRCLE_EXPONENT: f32 = 4.0;

/// Corner radius as a fraction of half-height, for the compact pill.
pub const PILL_CORNER_RATIO: f32 = 0.5;

/// Corner radius as a fraction of height, for the expanded card.
pub const CARD_CORNER_RATIO: f32 = 0.28;

/// Compact (idle) island size.
pub const COMPACT_W: f32 = 320.0;
pub const COMPACT_H: f32 = 44.0;

/// Expanded (active) island size.
pub const EXPANDED_W: f32 = 700.0;
pub const EXPANDED_H: f32 = 196.0;

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

/// The glass body. Near-black with a faint cool cast so it reads as a physical
/// object on an OLED panel rather than a hole punched in the wallpaper.
pub const GLASS_TOP: Rgba = Rgba::rgba(30, 32, 38, 242);
pub const GLASS_BOTTOM: Rgba = Rgba::rgba(12, 13, 17, 244);

/// Specular sheen along the upper boundary — simulates a directional light
/// catching the curve of the glass.
pub const SPECULAR: Rgba = Rgba::rgba(255, 255, 255, 46);

/// A second, tighter highlight right at the top lip.
pub const SPECULAR_HOT: Rgba = Rgba::rgba(255, 255, 255, 82);

/// Hairline inner border, keeps the silhouette legible against bright
/// wallpaper.
pub const HAIRLINE: Rgba = Rgba::rgba(255, 255, 255, 24);

/// Ambient occlusion at the bottom lip.
pub const BOTTOM_SHADE: Rgba = Rgba::rgba(0, 0, 0, 92);

/// Drop shadow cast onto the wallpaper.
pub const SHADOW: Rgba = Rgba::rgba(0, 0, 0, 150);

/// Primary typography.
pub const TEXT_PRIMARY: Rgba = Rgba::rgb(245, 247, 250);
/// Secondary typography — track titles, secondary values.
pub const TEXT_SECONDARY: Rgba = Rgba::rgb(154, 162, 174);
/// Tertiary typography — timestamps, hints, disabled.
pub const TEXT_TERTIARY: Rgba = Rgba::rgb(96, 104, 116);

/// Accent for active/progress states.
pub const ACCENT: Rgba = Rgba::rgb(94, 200, 255);
/// Success — completed tasks, hydration goal met.
pub const SUCCESS: Rgba = Rgba::rgb(74, 222, 128);
/// Warning — running/paused timers.
pub const WARNING: Rgba = Rgba::rgb(251, 191, 36);
/// Destructive — stop buttons.
pub const DANGER: Rgba = Rgba::rgb(248, 113, 113);

/// Interactive control surfaces.
pub const CONTROL_FILL: Rgba = Rgba::rgba(255, 255, 255, 22);
pub const CONTROL_FILL_HOVER: Rgba = Rgba::rgba(255, 255, 255, 42);
pub const CONTROL_FILL_ACTIVE: Rgba = Rgba::rgba(94, 200, 255, 62);

/// Separator lines.
pub const SEPARATOR: Rgba = Rgba::rgba(255, 255, 255, 18);

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Type ramp. Sizes are in logical pixels; the renderer scales them by DPI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextRole {
    /// Large numeric display — clock, timer countdown.
    Display,
    /// Section titles, track titles.
    Title,
    /// Body copy.
    Body,
    /// Small labels, metadata.
    Caption,
    /// Micro labels, tab names, units.
    Micro,
}

impl TextRole {
    /// Font size in logical pixels.
    pub fn size(self) -> f32 {
        match self {
            TextRole::Display => 40.0,
            TextRole::Title => 15.0,
            TextRole::Body => 13.0,
            TextRole::Caption => 11.5,
            TextRole::Micro => 10.0,
        }
    }

    /// Font weight (400 = regular, 500 = medium, 600 = semibold, 700 = bold).
    pub fn weight(self) -> u16 {
        match self {
            TextRole::Display => 600.0 as u16,
            TextRole::Title => 600.0 as u16,
            TextRole::Body => 400.0 as u16,
            TextRole::Caption => 500.0 as u16,
            TextRole::Micro => 600.0 as u16,
        }
    }

    /// Letter spacing in pixels (positive = tracked out).
    #[allow(dead_code)]
    pub fn tracking(self) -> f32 {
        match self {
            TextRole::Display => -0.5,
            TextRole::Title => -0.2,
            TextRole::Body => 0.0,
            TextRole::Caption => 0.1,
            TextRole::Micro => 0.6,
        }
    }

    /// Uppercase rendering (micro labels read better tracked out and caps).
    #[allow(dead_code)]
    pub fn uppercase(self) -> bool {
        matches!(self, TextRole::Micro)
    }
}

// ---------------------------------------------------------------------------
// Motion
// ---------------------------------------------------------------------------

/// Spring tuning, matching the brief's stated ranges.
///
/// * Morph (width/height): `omega 16.0-18.0`, `zeta 0.70-0.75`
/// * Pager slide: `omega 15.0-16.0`, `zeta 0.75`
pub mod motion {
    /// Width/height morph — snappy with a whisper of overshoot.
    pub const MORPH_OMEGA: f32 = 17.0;
    pub const MORPH_ZETA: f32 = 0.72;

    /// Pager horizontal slide.
    pub const PAGER_OMEGA: f32 = 15.5;
    pub const PAGER_ZETA: f32 = 0.75;

    /// Opacity cross-fades.
    #[allow(dead_code)]
    pub const FADE_OMEGA: f32 = 18.0;
    #[allow(dead_code)]
    pub const FADE_ZETA: f32 = 1.0;

    /// Micro-interactions (hover, press).
    pub const MICRO_OMEGA: f32 = 26.0;
    pub const MICRO_ZETA: f32 = 0.85;

    /// Settle threshold for the render timer to be killed.
    pub const REST_EPSILON: f32 = 0.5;
}

/// Rubber-banding resistance for swiping past the first/last page.
///
/// Returns a damped offset given the raw overscroll distance: the further you
/// pull, the less the content moves, asymptotically approaching `limit`.
pub fn rubber_band(overscroll: f32, limit: f32) -> f32 {
    if overscroll.abs() < f32::EPSILON {
        return 0.0;
    }
    let sign = overscroll.signum();
    let x = overscroll.abs() / limit.max(1.0);
    sign * limit * (1.0 - 1.0 / (1.0 + x * 0.55))
}
