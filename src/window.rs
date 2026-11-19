//! The overlay window: a layered, click-through, top-most surface that hosts
//! the island.
//!
//! Two properties of this window are non-negotiable:
//!
//! * **It never resizes.** The OS window is a fixed 740x220 surface for the
//!   entire life of the process. All morphing happens inside the transparent
//!   buffer, because resizing a layered window makes DWM re-allocate and
//!   re-composite the surface, which shows up as a visible flash.
//! * **It sleeps.** The render timer is destroyed the instant every spring
//!   settles, and re-armed only when something actually moves. That is what
//!   makes the idle CPU figure 0.0%.

use std::ffi::c_void;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, AC_SRC_ALPHA,
    AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC,
    HGDIOBJ,
};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TrackMouseEvent, TRACKMOUSEEVENT, TME_LEAVE,
};
use windows::Win32::UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, ShellExecuteW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GWLP_USERDATA, KillTimer,
    PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow, SetTimer, SetWindowLongPtrW,
    SetWindowPos, ShowWindow, TrackPopupMenu, TranslateMessage, UpdateLayeredWindow, HCURSOR,
    HICON, HWND_TOPMOST, MF_STRING, SM_CXSCREEN, SWP_NOACTIVATE, SW_SHOWNORMAL, SW_SHOWNOACTIVATE,
    TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, ULW_ALPHA, WM_APP, WM_CAPTURECHANGED,
    WM_DESTROY, WM_DROPFILES, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NULL, WM_RBUTTONUP,
    WM_TIMER, WS_CLIPSIBLINGS, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP, WNDCLASSW,
};

use crate::activities::{FilesState, FocusState, TasksState, WaterState};
use crate::mascot::{Blink, Eyes, MascotGeometry};
use crate::pager::Pager;
use crate::render::{Canvas, Mask};
use crate::spring::Spring;
use crate::text::TextRenderer;
use crate::theme;
use crate::ui::{self, CardAction, CardContext, CardHitTargets, Mount, TransportAction};

/// Message posted by the SMTC worker thread when the media session changes.
pub const WM_MEDIA_CHANGED: u32 = WM_APP + 1;

/// Message posted by the lyrics worker thread when LRCLIB lyrics arrive.
pub const WM_LYRICS_LOADED: u32 = WM_APP + 2;

/// `WM_MOUSELEAVE` is not exposed by the crate.
const WM_MOUSELEAVE: u32 = 0x02A3;

/// Timer id for the 60 FPS render loop.
const TIMER_ID: usize = 1;

/// Timer id for the 1 Hz playback / Pomodoro tick.
const SECOND_TIMER_ID: usize = 2;

/// The window class name.
const CLASS_NAME: &str = "NotchWindow";

/// Global mutex name used to enforce a single running instance.
const MUTEX_NAME: &str = "Global\\Isle_SingleInstance";

// ---------------------------------------------------------------------------
// Spring-driven state
// ---------------------------------------------------------------------------

struct Morph {
    width: Spring,
    height: Spring,
    expand: Spring,
    gap: Spring,
}

impl Morph {
    fn new() -> Self {
        Self {
            width: Spring::new(theme::COMPACT_W, theme::motion::MORPH_OMEGA, theme::motion::MORPH_ZETA),
            height: Spring::new(theme::COMPACT_H, theme::motion::MORPH_OMEGA, theme::motion::MORPH_ZETA),
            expand: Spring::new(0.0, theme::motion::MORPH_OMEGA, theme::motion::MORPH_ZETA),
            gap: Spring::new(0.0, theme::motion::MORPH_OMEGA, theme::motion::MORPH_ZETA),
        }
    }

    fn step(&mut self, dt: f32) -> bool {
        let mut moving = false;
        moving |= self.width.step(dt);
        moving |= self.height.step(dt);
        moving |= self.expand.step(dt);
        moving |= self.gap.step(dt);
        moving
    }
}

// ---------------------------------------------------------------------------
// The application
// ---------------------------------------------------------------------------

pub struct Island {
    hwnd: HWND,
    _mutex: windows::Win32::Foundation::HANDLE,

    canvas: Canvas,
    shadow_mask: Mask,
    text: TextRenderer,

    mem_dc: HDC,
    mem_bmp: HBITMAP,
    mem_old: HGDIOBJ,
    mem_bits: *mut u8,

    morph: Morph,
    mount: Mount,
    expanded: bool,
    hovering: bool,
    menu_open: bool,
    cursor: Option<(f32, f32)>,

    eyes: Eyes,
    blink: Blink,
    geom: MascotGeometry,
    rand: u32,

    elapsed: f32,
    last_tick: u64,
    timer_armed: bool,
    second_timer_armed: bool,

    snapshot: Option<crate::media::Snapshot>,
    track_key: (String, String),
    lyrics: Option<crate::lyrics::Lyrics>,

    pager: Pager,
    focus: FocusState,
    water: WaterState,
    tasks: TasksState,
    files: FilesState,
    last_hits: Option<CardHitTargets>,

    win_x: i32,
    win_y: i32,
}

impl Island {
    fn new(hwnd: HWND, mutex: windows::Win32::Foundation::HANDLE) -> Result<Self, String> {
        let canvas = Canvas::new(theme::CANVAS_W as usize, theme::CANVAS_H as usize);
        let shadow_mask = Mask::new(canvas.width, canvas.height);
        let text = TextRenderer::new(hwnd)?;

        let (mem_dc, mem_bmp, mem_old, mem_bits) = unsafe { create_blit_surface(hwnd)? };

        Ok(Self {
            hwnd,
            _mutex: mutex,
            canvas,
            shadow_mask,
            text,
            mem_dc,
            mem_bmp,
            mem_old,
            mem_bits,
            morph: Morph::new(),
            mount: Mount::Notch,
            expanded: false,
            hovering: false,
            menu_open: false,
            cursor: None,
            eyes: Eyes::new(),
            blink: Blink::new(),
            geom: MascotGeometry::default(),
            rand: 0x1234_5678,
            elapsed: 0.0,
            last_tick: unsafe { GetTickCount64() },
            timer_armed: false,
            second_timer_armed: false,
            snapshot: None,
            track_key: (String::new(), String::new()),
            lyrics: None,
            pager: Pager::new(),
            focus: FocusState::new(),
            water: WaterState::load(),
            tasks: TasksState::load(),
            files: FilesState::new(),
            last_hits: None,
            win_x: 0,
            win_y: 0,
        })
    }

    fn arm_timer(&mut self) {
        if self.timer_armed {
            return;
        }
        unsafe {
            SetTimer(self.hwnd, TIMER_ID, 16, None);
        }
        self.timer_armed = true;
    }

    fn disarm_timer(&mut self) {
        if !self.timer_armed {
            return;
        }
        unsafe {
            let _ = KillTimer(self.hwnd, TIMER_ID);
        }
        self.timer_armed = false;
    }

    /// Keep the 1 Hz timer active only when expanded media is playing or Focus is running.
    fn sync_second_timer(&mut self) {
        let media_playing = self.expanded && self.snapshot.as_ref().map_or(false, |s| s.playing);
        let should_run = media_playing || self.focus.running;
        if should_run && !self.second_timer_armed {
            unsafe {
                SetTimer(self.hwnd, SECOND_TIMER_ID, 1000, None);
            }
            self.second_timer_armed = true;
        } else if !should_run && self.second_timer_armed {
            unsafe {
                let _ = KillTimer(self.hwnd, SECOND_TIMER_ID);
            }
            self.second_timer_armed = false;
        }
    }

    fn reposition(&mut self) {
        let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) } as i32;
        self.win_x = (screen_w - theme::CANVAS_W as i32) / 2;
        self.win_y = 0;
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                self.win_x,
                self.win_y,
                theme::CANVAS_W as i32,
                theme::CANVAS_H as i32,
                SWP_NOACTIVATE,
            );
        }
    }

    fn set_expanded(&mut self, expanded: bool) {
        if self.expanded == expanded {
            return;
        }
        self.expanded = expanded;
        if expanded {
            self.morph.width.target = theme::EXPANDED_W;
            self.morph.height.target = theme::EXPANDED_H;
            self.morph.expand.target = 1.0;
        } else {
            self.morph.width.target = theme::COMPACT_W;
            self.morph.height.target = theme::COMPACT_H;
            self.morph.expand.target = 0.0;
            self.pager.cancel_drag();
        }
        self.sync_second_timer();
        self.arm_timer();
    }

    fn set_mount(&mut self, mount: Mount) {
        if self.mount == mount {
            return;
        }
        self.mount = mount;
        self.morph.gap.target = mount.gap();
        self.arm_timer();
    }

    fn on_mouse_move(&mut self, x: i32, _y: i32) {
        self.hovering = true;
        if self.pager.pressing {
            self.pager.update_drag(x);
        }
        let mut p = POINT { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut p) }.is_ok() {
            self.cursor = Some((p.x as f32, p.y as f32));
        }
        self.arm_timer();
    }

    fn on_mouse_leave(&mut self) {
        if self.pager.pressing || self.menu_open {
            return;
        }
        self.hovering = false;
        self.cursor = None;
        self.set_expanded(false);
    }

    fn on_lbutton_down(&mut self, x: i32, y: i32) {
        if self.expanded {
            let shape = self.current_shape();
            let fx = x as f32;
            let fy = y as f32;
            let in_content = fx >= shape.left()
                && fx <= shape.right()
                && fy >= shape.top() + 14.0
                && fy <= shape.bottom();
            let on_transport = self
                .last_hits
                .as_ref()
                .map_or(false, |h| h.is_transport_hit(fx, fy));
            if in_content && !on_transport {
                let now = unsafe { GetTickCount64() };
                self.pager.begin_press(x, now);
                unsafe {
                    SetCapture(self.hwnd);
                }
            }
        }
        self.arm_timer();
    }

    fn on_lbutton_up(&mut self, x: i32, y: i32) {
        if self.pager.pressing {
            unsafe {
                let _ = ReleaseCapture();
            }
            let now = unsafe { GetTickCount64() };
            if self.pager.end_press(x, now) {
                self.arm_timer();
                return;
            }
        }
        self.on_click(x, y);
    }

    fn on_rbutton_up(&mut self, _x: i32, _y: i32) {
        if self.expanded {
            if self.pager.active == 1 {
                self.focus.reset();
                self.sync_second_timer();
                self.arm_timer();
            }
            return;
        }
        self.show_context_menu();
    }

    fn show_context_menu(&mut self) {
        unsafe {
            let Ok(menu) = CreatePopupMenu() else {
                return;
            };
            let mount_label = match self.mount {
                Mount::Notch => w!("Switch to Floating Pill"),
                Mount::Floating => w!("Switch to Notch Mount"),
            };
            let _ = AppendMenuW(menu, MF_STRING, 1, mount_label);
            let _ = AppendMenuW(menu, MF_STRING, 2, w!("Quit Isle"));

            let mut pt = POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);

            self.menu_open = true;
            let _ = SetForegroundWindow(self.hwnd);
            let cmd = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );
            let _ = PostMessageW(self.hwnd, WM_NULL, WPARAM(0), LPARAM(0));
            let _ = DestroyMenu(menu);
            self.menu_open = false;

            match cmd.0 {
                1 => {
                    let next = match self.mount {
                        Mount::Notch => Mount::Floating,
                        Mount::Floating => Mount::Notch,
                    };
                    self.set_mount(next);
                }
                2 => {
                    PostQuitMessage(0);
                }
                _ => {}
            }
        }
    }

    fn on_click(&mut self, x: i32, y: i32) {
        if !self.expanded {
            self.set_expanded(true);
            return;
        }
        let Some(hits) = self.last_hits.clone() else {
            return;
        };
        match hits.hit_test(x as f32, y as f32, self.pager.active) {
            Some(CardAction::Collapse) => self.set_expanded(false),
            Some(CardAction::SelectTab(idx)) => {
                self.pager.select(idx);
                self.arm_timer();
            }
            Some(CardAction::Transport(TransportAction::Previous)) => crate::media::previous(),
            Some(CardAction::Transport(TransportAction::PlayPause)) => {
                crate::media::toggle_play_pause()
            }
            Some(CardAction::Transport(TransportAction::Next)) => crate::media::next(),
            Some(CardAction::FocusToggle) => {
                self.focus.toggle();
                self.sync_second_timer();
                self.arm_timer();
            }
            Some(CardAction::FocusReset) => {
                self.focus.reset();
                self.sync_second_timer();
                self.arm_timer();
            }
            Some(CardAction::FocusSkip) => {
                self.focus.advance_phase();
                self.sync_second_timer();
                self.arm_timer();
            }
            Some(CardAction::WaterAdd(ml)) => {
                self.water.add(ml);
                self.arm_timer();
            }
            Some(CardAction::WaterUndo) => {
                self.water.undo();
                self.arm_timer();
            }
            Some(CardAction::WaterReset) => {
                self.water.reset();
                self.arm_timer();
            }
            Some(CardAction::ToggleTask(idx)) => {
                self.tasks.toggle(idx);
                self.arm_timer();
            }
            Some(CardAction::OpenFile(idx)) => {
                if let Some(file) = self.files.items.get(idx) {
                    let wide_path: Vec<u16> =
                        file.path.encode_utf16().chain(std::iter::once(0)).collect();
                    unsafe {
                        ShellExecuteW(
                            self.hwnd,
                            w!("open"),
                            PCWSTR(wide_path.as_ptr()),
                            PCWSTR::null(),
                            PCWSTR::null(),
                            SW_SHOWNORMAL,
                        );
                    }
                }
            }
            Some(CardAction::ClearFiles) => {
                self.files.clear();
                self.arm_timer();
            }
            None => {}
        }
    }

    fn on_drop_files(&mut self, hdrop: HDROP) {
        unsafe {
            let count = DragQueryFileW(hdrop, u32::MAX, None);
            for i in 0..count {
                let len = DragQueryFileW(hdrop, i, None);
                if len == 0 {
                    continue;
                }
                let mut buf = vec![0u16; (len + 1) as usize];
                let copied = DragQueryFileW(hdrop, i, Some(&mut buf));
                if copied > 0 {
                    let path = String::from_utf16_lossy(&buf[..copied as usize]);
                    self.files.add_path(path);
                }
            }
            DragFinish(hdrop);
        }
        self.pager.select(4);
        self.set_expanded(true);
        self.arm_timer();
    }

    fn on_media_changed(&mut self) {
        self.snapshot = crate::media::snapshot();
        let new_key = match &self.snapshot {
            Some(s) => (s.title.clone(), s.artist.clone()),
            None => (String::new(), String::new()),
        };
        if new_key != self.track_key {
            self.track_key = new_key.clone();
            self.lyrics = None;
            if !new_key.0.is_empty() {
                crate::lyrics::fetch_async(self.hwnd, new_key.0, new_key.1);
            }
        }
        self.sync_second_timer();
        self.arm_timer();
    }

    fn on_lyrics_loaded(&mut self) {
        if let Some((title, artist, lrc)) = crate::lyrics::take_fetched() {
            if (title, artist) == self.track_key {
                self.lyrics = Some(lrc);
                self.arm_timer();
            }
        }
    }

    fn on_second_tick(&mut self) {
        if self.focus.running {
            self.focus.tick_one_second();
            self.sync_second_timer();
        }
        self.render();
    }

    fn tick(&mut self) {
        let now = unsafe { GetTickCount64() };
        let dt = ((now.saturating_sub(self.last_tick)) as f32 / 1000.0).clamp(0.0, 0.25);
        self.last_tick = now;
        self.elapsed += dt;

        let _blink_changed = self.blink.update(dt, &mut self.rand);
        let shape = self.current_shape();
        let origin = (
            self.win_x as f32 + shape.left() + 30.0,
            self.win_y as f32 + shape.body.center_y,
        );
        let cursor = self.cursor;
        let eyes_changed = self.eyes.update(dt, cursor, origin, &self.blink);

        let moving = self.morph.step(dt);
        let pager_moving = self.pager.step(dt);
        let water_moving = self.water.bounce.step(dt);

        self.render();

        if !moving
            && !pager_moving
            && !water_moving
            && !eyes_changed
            && !self.blink.active
        {
            self.disarm_timer();
        }
    }

    fn current_shape(&self) -> ui::IslandShape {
        let mount = if self.morph.gap.value < 0.5 {
            Mount::Notch
        } else {
            Mount::Floating
        };
        ui::build_shape(
            self.morph.width.value,
            self.morph.height.value,
            mount,
            self.morph.expand.value,
        )
    }

    fn render(&mut self) {
        let shape = self.current_shape();
        self.canvas.clear();
        ui::draw_shadow(&mut self.canvas, &mut self.shadow_mask, &shape);
        ui::draw_glass(&mut self.canvas, &shape);

        let playing = self.snapshot.as_ref().map_or(false, |s| s.playing);
        let (compact_title, compact_sub, compact_meter) = if playing {
            let s = self.snapshot.as_ref().unwrap();
            (
                s.title.clone(),
                s.artist.clone(),
                Some((0.5 + 0.5 * (self.elapsed * 6.0).sin(), true)),
            )
        } else if self.focus.running {
            (
                format!("Focus  ·  {}", self.focus.formatted_time()),
                self.focus.phase.label().to_string(),
                Some((self.focus.progress(), false)),
            )
        } else if let Some(s) = &self.snapshot {
            let p = s.progress();
            (
                s.title.clone(),
                s.artist.clone(),
                if p > 0.0 { Some((p, false)) } else { None },
            )
        } else {
            (String::new(), String::new(), None)
        };

        if self.morph.expand.value < 0.999 {
            ui::draw_compact(
                &mut self.canvas,
                &mut self.text,
                &shape,
                &self.eyes,
                &self.blink,
                &self.geom,
                &compact_title,
                &compact_sub,
                compact_meter,
            );
        }
        if self.morph.expand.value > 0.001 {
            let ctx = CardContext {
                snapshot: self.snapshot.as_ref(),
                lyrics: self.lyrics.as_ref(),
                focus: &self.focus,
                water: &self.water,
                tasks: &self.tasks,
                files: &self.files,
                pager_scroll_x: self.pager.scroll_x.value,
                pager_page_pos: self.pager.page_pos(),
                active_page: self.pager.active,
                card_width: self.pager.card_width,
            };
            self.last_hits = Some(ui::draw_card(
                &mut self.canvas,
                &mut self.text,
                &shape,
                &self.eyes,
                &self.blink,
                &self.geom,
                &ctx,
            ));
        } else {
            self.last_hits = None;
        }

        ui::draw_hit_rail(&mut self.canvas, &shape);
        self.blit();
    }

    fn blit(&mut self) {
        let len = self.canvas.data.len();
        unsafe {
            std::ptr::copy_nonoverlapping(self.canvas.data.as_ptr(), self.mem_bits, len);

            let mut dst = POINT { x: self.win_x, y: self.win_y };
            let size = SIZE { cx: theme::CANVAS_W as i32, cy: theme::CANVAS_H as i32 };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let _ = UpdateLayeredWindow(
                self.hwnd,
                None,
                Some(&mut dst),
                Some(&size),
                self.mem_dc,
                Some(&src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
        }
    }
}

impl Drop for Island {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.mem_dc, self.mem_old);
            let _ = DeleteObject(self.mem_bmp);
            let _ = DeleteDC(self.mem_dc);
        }
    }
}

// ---------------------------------------------------------------------------
// Window procedure
// ---------------------------------------------------------------------------

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Island;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let island = &mut *ptr;

    match msg {
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                island.tick();
            } else if wparam.0 == SECOND_TIMER_ID {
                island.on_second_tick();
            }
        }
        WM_MOUSEMOVE => {
            let (x, y) = lparam_xy(lparam);
            island.on_mouse_move(x, y);
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tme);
        }
        WM_MOUSELEAVE => {
            island.on_mouse_leave();
        }
        WM_LBUTTONDOWN => {
            let (x, y) = lparam_xy(lparam);
            island.on_lbutton_down(x, y);
        }
        WM_LBUTTONUP => {
            let (x, y) = lparam_xy(lparam);
            island.on_lbutton_up(x, y);
        }
        WM_RBUTTONUP => {
            let (x, y) = lparam_xy(lparam);
            island.on_rbutton_up(x, y);
        }
        WM_CAPTURECHANGED => {
            if island.pager.pressing {
                island.pager.cancel_drag();
                island.arm_timer();
            }
        }
        WM_DROPFILES => {
            island.on_drop_files(HDROP(wparam.0 as *mut _));
        }
        WM_MEDIA_CHANGED => {
            island.on_media_changed();
        }
        WM_LYRICS_LOADED => {
            island.on_lyrics_loaded();
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn lparam_xy(lparam: LPARAM) -> (i32, i32) {
    let x = (lparam.0 as u16) as i16 as i32;
    let y = ((lparam.0 >> 16) as u16) as i16 as i32;
    (x, y)
}

// ---------------------------------------------------------------------------
// Startup
// ---------------------------------------------------------------------------

unsafe fn create_blit_surface(hwnd: HWND) -> Result<(HDC, HBITMAP, HGDIOBJ, *mut u8), String> {
    let screen_dc = windows::Win32::Graphics::Gdi::GetDC(hwnd);
    if screen_dc.is_invalid() {
        return Err("could not acquire a device context".into());
    }
    let dc = CreateCompatibleDC(screen_dc);
    windows::Win32::Graphics::Gdi::ReleaseDC(hwnd, screen_dc);
    if dc.is_invalid() {
        return Err("could not create a compatible device context".into());
    }

    let w = theme::CANVAS_W as i32;
    let h = theme::CANVAS_H as i32;
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
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

    let mut bits: *mut c_void = std::ptr::null_mut();
    let bmp = CreateDIBSection(
        dc,
        &mut bmi as *mut BITMAPINFO,
        DIB_RGB_COLORS,
        &mut bits,
        windows::Win32::Foundation::HANDLE(std::ptr::null_mut()),
        0,
    )
    .map_err(|e| format!("CreateDIBSection failed: {e}"))?;
    let old = SelectObject(dc, bmp);

    Ok((dc, bmp, old, bits as *mut u8))
}

pub fn run() -> Result<(), String> {
    unsafe {
        let mutex_wide: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();
        let mutex = CreateMutexW(None, true, PCWSTR(mutex_wide.as_ptr()))
            .map_err(|e| format!("could not create the instance mutex: {e}"))?;
        if windows::Win32::Foundation::GetLastError()
            == windows::Win32::Foundation::ERROR_ALREADY_EXISTS
        {
            let _ = windows::Win32::Foundation::CloseHandle(mutex);
            return Err("Isle is already running".into());
        }

        let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
            .map_err(|e| format!("could not get the module handle: {e}"))?;

        let class_name: Vec<u16> = CLASS_NAME.encode_utf16().chain(std::iter::once(0)).collect();
        let wc = WNDCLASSW {
            style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance.into(),
            hIcon: HICON(std::ptr::null_mut()),
            hCursor: HCURSOR(std::ptr::null_mut()),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(std::ptr::null_mut()),
            lpszMenuName: PCWSTR(std::ptr::null()),
            lpszClassName: PCWSTR(class_name.as_ptr()),
        };
        if RegisterClassW(&wc) == 0 {
            let _ = windows::Win32::Foundation::CloseHandle(mutex);
            return Err("could not register the window class".into());
        }

        let ex = WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_NOACTIVATE;
        let style = WS_POPUP | WS_CLIPSIBLINGS;

        let hwnd = CreateWindowExW(
            ex,
            PCWSTR(class_name.as_ptr()),
            w!("Isle"),
            style,
            0,
            0,
            theme::CANVAS_W as i32,
            theme::CANVAS_H as i32,
            HWND(std::ptr::null_mut()),
            None,
            hinstance,
            None,
        )
        .map_err(|e| format!("could not create the overlay window: {e}"))?;

        DragAcceptFiles(hwnd, true);

        let island = Box::new(Island::new(hwnd, mutex)?);
        let raw = Box::into_raw(island);
        SetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            raw as isize,
        );

        {
            let island = &mut *raw;
            island.reposition();
            island.render();
            island.arm_timer();
        }

        crate::media::init(hwnd);
        crate::media::refresh();

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        loop {
            let r = GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0);
            if r.0 <= 0 {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let island = Box::from_raw(raw);
        drop(island);
        let _ = windows::Win32::Foundation::CloseHandle(mutex);
    }

    Ok(())
}
