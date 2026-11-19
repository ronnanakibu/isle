//! Text rendering.
//!
//! Typography quality is the difference between "looks like a utility" and
//! "looks like Apple hardware", so text gets real attention here.
//!
//! GDI's `DrawTextW` cannot write into our premultiplied buffer directly —
//! it composites in straight-alpha and would destroy the alpha channel. So we
//! render glyphs into a dedicated 32-bit scratch DIB, then transfer the
//! result into the canvas as a coverage mask. That gives us two things GDI
//! alone cannot: correct premultiplied compositing, and the ability to tint
//! text with any color while keeping the glyph's antialiased edges.

use std::collections::HashMap;

use windows::Win32::Foundation::{HWND, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateFontW, DeleteDC, DeleteObject, GetTextExtentPoint32W, SelectObject,
    SetBkMode, SetTextColor, SetStretchBltMode, DrawTextW, CreateDIBSection, BITMAPINFO,
    BITMAPINFOHEADER, DIB_RGB_COLORS, CreateSolidBrush, FillRect, HBRUSH, ANTIALIASED_QUALITY,
    DEFAULT_CHARSET, DEFAULT_PITCH, FF_DONTCARE, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, TRANSPARENT,
    LOGFONTW, HDC, HGDIOBJ, HFONT,
};

use crate::render::{Canvas, Rgba};

/// Font families. Segoe UI Variable is the modern Windows 11 system face and
/// has genuinely better optical sizing than the legacy Segoe UI.
pub const FAMILY_UI: &str = "Segoe UI Variable Text";
pub const FAMILY_DISPLAY: &str = "Segoe UI Variable Display";
#[allow(dead_code)]
pub const FALLBACK_FAMILY: &str = "Segoe UI";

/// A cached font at a specific size and weight.
#[derive(PartialEq, Eq, Hash)]
struct FontKey {
    family: &'static str,
    size: u32,
    weight: u16,
}

/// Horizontal alignment for a text run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// A text renderer owning its GDI scratch resources.
///
/// All resources are created once at startup and reused, so text rendering
/// allocates nothing during steady-state animation.
pub struct TextRenderer {
    /// Memory DC the scratch bitmap is selected into.
    scratch_dc: HDC,
    /// The scratch 32-bit DIB backing the text surface.
    scratch_bmp: windows::Win32::Graphics::Gdi::HBITMAP,
    /// Previous bitmap, restored before deletion.
    old_bmp: HGDIOBJ,
    /// Direct pointer to the scratch DIB pixels.
    scratch_bits: *mut u8,
    scratch_w: i32,
    scratch_h: i32,
    /// Font cache keyed by family/size/weight.
    fonts: HashMap<FontKey, HFONT>,
    /// Currently selected font, so we can restore before deleting.
    current_font: Option<HFONT>,
    /// Brush used to clear the scratch surface.
    brush: HBRUSH,
}

impl TextRenderer {
    /// Create the renderer. `hwnd` is used only to obtain a compatible DC.
    pub fn new(hwnd: HWND) -> Result<Self, String> {
        unsafe {
            let screen_dc = windows::Win32::Graphics::Gdi::GetDC(hwnd);
            if screen_dc.is_invalid() {
                return Err("could not acquire a device context".into());
            }
            let dc = CreateCompatibleDC(screen_dc);
            windows::Win32::Graphics::Gdi::ReleaseDC(hwnd, screen_dc);
            if dc.is_invalid() {
                return Err("could not create a compatible device context".into());
            }

            // A 1x1 placeholder is replaced on first use with a right-sized
            // DIB; text is measured before rendering so we always know the
            // needed extent.
            let (bmp, bits) = create_dib(dc, 8, 8)?;
            let old_bmp = SelectObject(dc, bmp);

            // ClearType is disabled: our scratch surface has no real
            // background, and subpixel rendering produces color fringes that
            // look wrong when tinted. Grayscale AA is the correct choice for
            // a translucent layered window.
            SetBkMode(dc, TRANSPARENT);
            SetStretchBltMode(dc, windows::Win32::Graphics::Gdi::HALFTONE);

            let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(0x0000_0000));

            Ok(Self {
                scratch_dc: dc,
                scratch_bmp: bmp,
                old_bmp,
                scratch_bits: bits,
                scratch_w: 8,
                scratch_h: 8,
                fonts: HashMap::new(),
                current_font: None,
                brush,
            })
        }
    }

    /// Get (or create) a font.
    fn font(&mut self, family: &'static str, size: f32, weight: u16) -> HFONT {
        let key = FontKey { family, size: size.round() as u32, weight };
        if let Some(f) = self.fonts.get(&key) {
            return *f;
        }
        unsafe {
            // Negative height requests a character cell height rather than a
            // cell height, which is what we want for typographic sizing.
            let h = -(size.round() as i32);
            let mut face = [0u16; 32];
            for (i, c) in family.encode_utf16().take(31).enumerate() {
                face[i] = c;
            }

            let lf = LOGFONTW {
                lfHeight: h,
                lfWidth: 0,
                lfEscapement: 0,
                lfOrientation: 0,
                lfWeight: weight as i32,
                lfItalic: 0,
                lfUnderline: 0,
                lfStrikeOut: 0,
                lfCharSet: DEFAULT_CHARSET,
                lfOutPrecision: OUT_TT_PRECIS,
                lfClipPrecision: CLIP_DEFAULT_PRECIS,
                lfQuality: ANTIALIASED_QUALITY,
                // PITCH_AND_FAMILY packs a pitch in the low byte and a family
                // in the high byte; FONT_PITCH and FONT_FAMILY are separate
                // typed constants, so combine them numerically.
                lfPitchAndFamily: DEFAULT_PITCH.0 | (FF_DONTCARE.0 << 4),
                lfFaceName: face,
            };
            let font = CreateFontW(
                lf.lfHeight,
                lf.lfWidth,
                lf.lfEscapement,
                lf.lfOrientation,
                lf.lfWeight,
                lf.lfItalic as u32,
                lf.lfUnderline as u32,
                lf.lfStrikeOut as u32,
                lf.lfCharSet.0 as u32,
                lf.lfOutPrecision.0 as u32,
                lf.lfClipPrecision.0 as u32,
                lf.lfQuality.0 as u32,
                lf.lfPitchAndFamily as u32,
                windows::core::PCWSTR(lf.lfFaceName.as_ptr()),
            );
            self.fonts.insert(key, font);
            font
        }
    }

    /// Ensure the scratch surface is at least `w` x `h` pixels.
    fn ensure_scratch(&mut self, w: i32, h: i32) -> Result<(), String> {
        if w <= self.scratch_w && h <= self.scratch_h {
            return Ok(());
        }
        unsafe {
            // Restore the original bitmap before deleting the scratch one.
            SelectObject(self.scratch_dc, self.old_bmp);
            let _ = DeleteObject(self.scratch_bmp);

            let nw = w.max(self.scratch_w);
            let nh = h.max(self.scratch_h);
            let (bmp, bits) = create_dib(self.scratch_dc, nw, nh)?;
            self.old_bmp = SelectObject(self.scratch_dc, bmp);
            self.scratch_bmp = bmp;
            self.scratch_bits = bits;
            self.scratch_w = nw;
            self.scratch_h = nh;
        }
        Ok(())
    }

    /// Measure a string's pixel extent.
    pub fn measure(&mut self, text: &str, family: &'static str, size: f32, weight: u16) -> (i32, i32) {
        if text.is_empty() {
            return (0, 0);
        }
        unsafe {
            let font = self.font(family, size, weight);
            let old = SelectObject(self.scratch_dc, HGDIOBJ(font.0));
            self.current_font = Some(font);

            let wide_text = wide(text);
            let mut size_out = SIZE { cx: 0, cy: 0 };
            let _ = GetTextExtentPoint32W(self.scratch_dc, &wide_text, &mut size_out);

            SelectObject(self.scratch_dc, old);
            (size_out.cx, size_out.cy)
        }
    }

    /// Render `text` into the canvas at `x`, `y` with the given color.
    ///
    /// Returns the advance width of the rendered text.
    pub fn draw(
        &mut self,
        canvas: &mut Canvas,
        text: &str,
        x: f32,
        y: f32,
        color: Rgba,
        family: &'static str,
        size: f32,
        weight: u16,
        align: Align,
        max_width: Option<f32>,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        // Truncate with an ellipsis if the run exceeds its box.
        let (text, advance) = match max_width {
            Some(maxw) => self.ellipsize(text, family, size, weight, maxw),
            None => {
                let w = self.measure(text, family, size, weight).0 as f32;
                (text.to_string(), w)
            }
        };

        if text.is_empty() {
            return 0.0;
        }

        let draw_x = match align {
            Align::Left => x,
            Align::Center => x - advance * 0.5,
            Align::Right => x - advance,
        };

        unsafe {
            let font = self.font(family, size, weight);
            let old = SelectObject(self.scratch_dc, HGDIOBJ(font.0));
            self.current_font = Some(font);

            let wide_text = wide(&text);
            let text_w = (advance.ceil() as i32).max(wide_text.len() as i32);
            let cell_h = (size * 1.6).ceil() as i32 + 8;

            if let Err(_e) = self.ensure_scratch(text_w + 8, cell_h) {
                SelectObject(self.scratch_dc, old);
                return advance;
            }

            // Clear the scratch region to opaque black; GDI needs a defined
            // background for grayscale antialiasing to blend against.
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: self.scratch_w,
                bottom: self.scratch_h,
            };
            FillRect(self.scratch_dc, &mut rect, self.brush);

            SetTextColor(self.scratch_dc, windows::Win32::Foundation::COLORREF(0x00FF_FFFF));

            let mut draw_rect = RECT {
                left: 0,
                top: 0,
                right: self.scratch_w,
                bottom: cell_h,
            };
            // DT_LEFT | DT_TOP | DT_SINGLELINE | DT_NOPREFIX
            let format = windows::Win32::Graphics::Gdi::DRAW_TEXT_FORMAT(
                0x0000 | 0x0000 | 0x0020 | 0x0800,
            );
            let drawn = DrawTextW(self.scratch_dc, &mut wide_text.clone(), &mut draw_rect, format);
            let _ = drawn;

            // Transfer the glyph coverage into the canvas.
            self.transfer(canvas, draw_x as i32, y as i32, text_w + 8, cell_h, color);

            SelectObject(self.scratch_dc, old);
        }

        advance
    }

    /// Copy the scratch alpha coverage into the canvas as tinted text.
    ///
    /// GDI renders the glyph as a luminance mask on black, so the red channel
    /// is the coverage. We rebuild the pixel in the requested color with that
    /// coverage as alpha, which keeps edges smooth and correctly premultiplied.
    unsafe fn transfer(&mut self, canvas: &mut Canvas, ox: i32, oy: i32, w: i32, h: i32, color: Rgba) {
        let bits = self.scratch_bits;
        if bits.is_null() {
            return;
        }
        let src_w = self.scratch_w as usize;
        for sy in 0..h {
            for sx in 0..w {
                let idx = (sy as usize) * src_w + (sx as usize);
                let p = bits.add(idx * 4);
                let coverage = (*p as f32) / 255.0;
                if coverage <= 0.002 {
                    continue;
                }
                let mut c = color;
                c.a = (color.a as f32 * coverage).round() as u8;
                canvas.blend(ox + sx, oy + sy, c);
            }
        }
    }

    /// Truncate text to fit `max_width`, appending an ellipsis if cut.
    fn ellipsize(
        &mut self,
        text: &str,
        family: &'static str,
        size: f32,
        weight: u16,
        max_width: f32,
    ) -> (String, f32) {
        let full_w = self.measure(text, family, size, weight).0 as f32;
        if full_w <= max_width {
            return (text.to_string(), full_w);
        }

        // Binary search the longest prefix that fits alongside the ellipsis.
        let ell = "…";
        let ell_w = self.measure(ell, family, size, weight).0 as f32;
        let budget = max_width - ell_w;
        if budget <= 0.0 {
            return (String::new(), 0.0);
        }

        let chars: Vec<char> = text.chars().collect();
        let mut lo = 0usize;
        let mut hi = chars.len();
        let mut best = 0usize;
        while lo <= hi {
            let mid = (lo + hi) / 2;
            let candidate: String = chars[..mid].iter().collect();
            let w = self.measure(&candidate, family, size, weight).0 as f32;
            if w <= budget {
                best = mid;
                lo = mid + 1;
            } else {
                if mid == 0 {
                    break;
                }
                hi = mid - 1;
            }
        }

        let mut out: String = chars[..best].iter().collect();
        out.push('…');
        let w = self.measure(&out, family, size, weight).0 as f32;
        (out, w)
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(font) = self.current_font {
                SelectObject(self.scratch_dc, HGDIOBJ(font.0));
            }
            SelectObject(self.scratch_dc, self.old_bmp);
            let _ = DeleteObject(self.scratch_bmp);
            let _ = DeleteObject(self.brush);
            let _ = DeleteDC(self.scratch_dc);
        }
    }
}

/// Create a 32-bit bottom-up BGRA DIB and return it with a pointer to its
/// pixels.
unsafe fn create_dib(dc: HDC, w: i32, h: i32) -> Result<(windows::Win32::Graphics::Gdi::HBITMAP, *mut u8), String> {
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            // Negative height selects a top-down DIB, so row 0 is the top row.
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    };

    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    let bmp = match CreateDIBSection(
        dc,
        &mut bmi as *mut BITMAPINFO,
        DIB_RGB_COLORS,
        &mut bits,
        windows::Win32::Foundation::HANDLE(std::ptr::null_mut()),
        0,
    ) {
        Ok(b) => b,
        Err(e) => return Err(format!("CreateDIBSection failed: {e}")),
    };
    Ok((bmp, bits as *mut u8))
}

/// Convert a Rust string to a null-terminated UTF-16 buffer.
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
