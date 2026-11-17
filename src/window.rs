//! Win32 Layered Window Management for Apple Dynamic Island
//! Zero-Idle CPU lifecycle, smooth 60 FPS spring physics, WinRT SMTC, Pager & Tray Settings

use std::mem::zeroed;
use std::ptr::null_mut;
use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::theme::*;
use crate::spring::*;
use crate::mascot::*;
use crate::pages::*;
use crate::render::*;
use crate::settings::{Mode, Settings};
use crate::tray::*;
use crate::media::*;
use crate::gesture::*;

const TIMER_ANIMATION: usize = 1;
const TIMER_MASCOT_BLINK: usize = 2;
const WIN_WM_MOUSELEAVE: u32 = 0x02A3;

pub struct WindowContext {
    pub hwnd: HWND,
    pub hdc_mem: HDC,
    #[allow(dead_code)]
    pub hbitmap: HBITMAP,
    pub p_bits: *mut u32,
    pub surface: Surface,
    pub width_spring: Spring,
    pub height_spring: Spring,
    pub mascot: Mascot,
    pub page_state: PageState,
    pub media_hook: MediaHook,
    pub media: MediaSnapshot,
    pub pager: Pager,
    pub settings: Settings,
    pub is_hovered: bool,
    pub is_animating: bool,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub hovered_hit: HitTarget,
    pub press_hit: HitTarget,
    pub tick_counter: u32,
}

static mut CONTEXT: Option<WindowContext> = None;

pub unsafe fn run_window() -> windows::core::Result<()> {
    let instance = GetModuleHandleW(None)?;

    let class_name = w!("DynamicIslandWindow");
    let mut wcx: WNDCLASSEXW = zeroed();
    wcx.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wcx.style = CS_HREDRAW | CS_VREDRAW;
    wcx.lpfnWndProc = Some(wndproc);
    wcx.hInstance = instance.into();
    wcx.hCursor = LoadCursorW(None, IDC_ARROW)?;
    wcx.lpszClassName = class_name;

    RegisterClassExW(&wcx);

    let settings = Settings::load();
    let (win_x, win_y) = settings.anchor(CANVAS_WIDTH, 10);

    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
        class_name,
        w!("Dynamic Island"),
        WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS,
        win_x,
        win_y,
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
        None,
        None,
        instance,
        None,
    )?;

    // 32-bit DIB Section for alpha blending
    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(hdc_screen);

    let mut bmi: BITMAPINFO = zeroed();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = CANVAS_WIDTH;
    bmi.bmiHeader.biHeight = -CANVAS_HEIGHT; // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;

    let mut p_bits: *mut std::ffi::c_void = null_mut();
    let hbitmap = CreateDIBSection(
        hdc_mem,
        &bmi,
        DIB_RGB_COLORS,
        &mut p_bits,
        None,
        0,
    )?;

    SelectObject(hdc_mem, hbitmap);
    ReleaseDC(None, hdc_screen);

    // Initial state: start expanded, fluidly spring into compact pill
    let mut w_spring = Spring::new(EXPANDED_WIDTH);
    let mut h_spring = Spring::new(EXPANDED_HEIGHT);
    w_spring.current = EXPANDED_WIDTH;
    h_spring.current = EXPANDED_HEIGHT;
    w_spring.set_target(COMPACT_WIDTH);
    h_spring.set_target(COMPACT_HEIGHT);

    let media_hook = MediaHook::start(hwnd);
    let media = media_hook.snapshot();
    let pager = Pager::new(EXPANDED_WIDTH);

    let ctx = WindowContext {
        hwnd,
        hdc_mem,
        hbitmap,
        p_bits: p_bits as *mut u32,
        surface: Surface::new(CANVAS_WIDTH, CANVAS_HEIGHT),
        width_spring: w_spring,
        height_spring: h_spring,
        mascot: Mascot::new(),
        page_state: PageState::default(),
        media_hook,
        media,
        pager,
        settings,
        is_hovered: false,
        is_animating: true,
        mouse_x: 0,
        mouse_y: 0,
        hovered_hit: HitTarget::None,
        press_hit: HitTarget::None,
        tick_counter: 0,
    };

    CONTEXT = Some(ctx);

    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        win_x,
        win_y,
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
        SWP_SHOWWINDOW | SWP_NOACTIVATE,
    )?;

    add_tray_icon(hwnd);
    update_window_surface();

    SetTimer(hwnd, TIMER_ANIMATION, 16, None);
    SetTimer(hwnd, TIMER_MASCOT_BLINK, 100, None);

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, None, 0, 0).into() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    Ok(())
}

unsafe fn update_window_surface() {
    let ctx_ptr = &raw mut CONTEXT;
    if let Some(ctx) = (*ctx_ptr).as_mut() {
        let expand_t = ((ctx.width_spring.current - COMPACT_WIDTH) / (EXPANDED_WIDTH - COMPACT_WIDTH)).clamp(0.0, 1.0);
        let smoothed_t = smoothstep(0.0, 1.0, expand_t);
        let is_flat_top = ctx.settings.mode == Mode::Notch;

        // 1. Vector/squircle rasterization pass
        render_apple_island(
            &mut ctx.surface,
            &ctx.width_spring,
            &ctx.height_spring,
            smoothed_t,
            &ctx.mascot,
            &ctx.page_state,
            &ctx.media,
            is_flat_top,
            ctx.hovered_hit,
        );

        // Copy rendered pixels into DIB Section
        std::ptr::copy_nonoverlapping(
            ctx.surface.pixels.as_ptr(),
            ctx.p_bits,
            (CANVAS_WIDTH * CANVAS_HEIGHT) as usize,
        );

        let cur_w = ctx.width_spring.current as i32;
        let x0 = (CANVAS_WIDTH - cur_w) / 2;
        let y0 = if is_flat_top { 0 } else { 10 };

        // 2. Crisp GDI Typography Pass for Expanded state
        if smoothed_t > 0.3 {
            render_gdi_text_pass(
                ctx.hdc_mem,
                x0,
                y0,
                cur_w,
                smoothed_t,
                &ctx.page_state,
                &ctx.media,
            );

            // Post-pass: Fix alpha channel for GDI painted text inside the island
            let text_y_start = (y0 + 12).max(0) as usize;
            let text_y_end = (y0 + 140).min(CANVAS_HEIGHT) as usize;
            let text_x_start = (x0 + 15).max(0) as usize;
            let text_x_end = (x0 + cur_w - 15).min(CANVAS_WIDTH) as usize;

            for y in text_y_start..text_y_end {
                let row_offset = y * (CANVAS_WIDTH as usize);
                for x in text_x_start..text_x_end {
                    let idx = row_offset + x;
                    let val = *ctx.p_bits.add(idx);
                    let rgb = val & 0x00FFFFFF;
                    if rgb > 0x001A1A1A {
                        *ctx.p_bits.add(idx) = 0xFF000000 | rgb;
                    }
                }
            }
        }

        // 3. UpdateLayeredWindow
        let (win_x, win_y) = ctx.settings.anchor(CANVAS_WIDTH, 10);
        let pt_dst = POINT { x: win_x, y: win_y };
        let pt_src = POINT { x: 0, y: 0 };
        let size = SIZE {
            cx: CANVAS_WIDTH,
            cy: CANVAS_HEIGHT,
        };

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            ctx.hwnd,
            None,
            Some(&pt_dst),
            Some(&size),
            ctx.hdc_mem,
            Some(&pt_src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
    }
}

fn hit_test(x: f32, y: f32, x0: f32, y0: f32, cur_w: f32, cur_h: f32, current_page: PageId) -> HitTarget {
    let center_x = x0 + cur_w * 0.5;

    // Check bottom pagination dots
    let dot_y = y0 + cur_h - 12.0;
    if (y - dot_y).abs() < 12.0 {
        let dots_count = PageId::ALL.len();
        let spacing = 14.0;
        let start_x = center_x - ((dots_count as f32 - 1.0) * spacing) * 0.5;
        for (i, _) in PageId::ALL.iter().enumerate() {
            let px = start_x + (i as f32) * spacing;
            if (x - px).abs() < 10.0 {
                return HitTarget::PageDot(i);
            }
        }
    }

    match current_page {
        PageId::Music => {
            let ctrl_cy = y0 + 116.0;
            if (y - ctrl_cy).abs() < 24.0 {
                if (x - center_x).abs() < 24.0 {
                    return HitTarget::MediaPlayPause;
                } else if (x - (center_x - 56.0)).abs() < 18.0 {
                    return HitTarget::MediaPrev;
                } else if (x - (center_x + 56.0)).abs() < 18.0 {
                    return HitTarget::MediaNext;
                }
            } else if (y - (y0 + 84.0)).abs() < 10.0 {
                return HitTarget::Scrubber;
            }
        }
        PageId::Pomodoro => {
            if y >= y0 + 70.0 && y <= y0 + 98.0 {
                let pill_w = 78.0;
                let spacing = 8.0;
                let total_w = 3.0 * pill_w + 2.0 * spacing;
                let start_x = x0 + (cur_w - total_w) * 0.5;
                for i in 0..3 {
                    let px = start_x + (i as f32) * (pill_w + spacing);
                    if x >= px && x <= px + pill_w {
                        return HitTarget::PomoMode(i);
                    }
                }
            } else if y >= y0 + 100.0 && y <= y0 + 130.0 {
                if x >= center_x - 52.0 && x <= center_x + 20.0 {
                    return HitTarget::PomoStart;
                } else if x >= center_x + 28.0 && x <= center_x + 80.0 {
                    return HitTarget::PomoReset;
                }
            }
        }
        PageId::Water => {
            if y >= y0 + 92.0 && y <= y0 + 124.0 {
                if x >= x0 + 75.0 && x <= x0 + 151.0 {
                    return HitTarget::WaterAdd250;
                } else if x >= x0 + 160.0 && x <= x0 + 236.0 {
                    return HitTarget::WaterAdd500;
                } else if x >= x0 + 245.0 && x <= x0 + 305.0 {
                    return HitTarget::WaterReset;
                }
            }
        }
        PageId::Todo => {
            for i in 0..3 {
                let row_y = y0 + 36.0 + (i as f32) * 28.0;
                if y >= row_y - 2.0 && y <= row_y + 24.0 {
                    return HitTarget::TaskItem(i);
                }
            }
        }
        PageId::Shelf => {
            for i in 0..2 {
                let card_y = y0 + 38.0 + (i as f32) * 38.0;
                if y >= card_y && y <= card_y + 34.0 {
                    return HitTarget::ShelfOpen(i);
                }
            }
        }
    }

    HitTarget::None
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ctx_ptr = &raw mut CONTEXT;
    match msg {
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;

            if let Some(ctx) = (*ctx_ptr).as_mut() {
                ctx.mouse_x = x as i32;
                ctx.mouse_y = y as i32;

                let center_x = (CANVAS_WIDTH as f32) * 0.5;
                ctx.mascot.update_gaze(x - center_x, y - 18.0);

                let cur_w = ctx.width_spring.current;
                let cur_h = ctx.height_spring.current;
                let is_flat_top = ctx.settings.mode == Mode::Notch;
                let x0 = center_x - cur_w * 0.5;
                let y0 = if is_flat_top { 0.0 } else { 10.0 };

                let inside_island = x >= (x0 - 6.0)
                    && x <= (x0 + cur_w + 6.0)
                    && y >= 0.0
                    && y <= (y0 + cur_h + 8.0);

                if inside_island {
                    if !ctx.is_hovered {
                        ctx.is_hovered = true;
                        ctx.width_spring.set_target(EXPANDED_WIDTH);
                        ctx.height_spring.set_target(EXPANDED_HEIGHT);

                        let mut tme: TRACKMOUSEEVENT = zeroed();
                        tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
                        tme.dwFlags = TME_LEAVE;
                        tme.hwndTrack = hwnd;
                        let _ = TrackMouseEvent(&mut tme);

                        if !ctx.is_animating {
                            ctx.is_animating = true;
                            SetTimer(hwnd, TIMER_ANIMATION, 16, None);
                        }
                    }

                    if ctx.pager.is_dragging() {
                        if ctx.pager.drag(x) {
                            ctx.hovered_hit = HitTarget::None;
                            update_window_surface();
                        }
                    } else {
                        let hit = hit_test(x, y, x0, y0, cur_w, cur_h, ctx.page_state.current_page);
                        if hit != ctx.hovered_hit {
                            ctx.hovered_hit = hit;
                            update_window_surface();
                        }
                    }
                } else if ctx.is_hovered && !ctx.pager.is_dragging() {
                    ctx.is_hovered = false;
                    ctx.hovered_hit = HitTarget::None;
                    ctx.width_spring.set_target(COMPACT_WIDTH);
                    ctx.height_spring.set_target(COMPACT_HEIGHT);

                    if !ctx.is_animating {
                        ctx.is_animating = true;
                        SetTimer(hwnd, TIMER_ANIMATION, 16, None);
                    }
                }
            }
            LRESULT(0)
        }

        WIN_WM_MOUSELEAVE => {
            if let Some(ctx) = (*ctx_ptr).as_mut() {
                if !ctx.pager.is_dragging() {
                    ctx.is_hovered = false;
                    ctx.hovered_hit = HitTarget::None;
                    ctx.width_spring.set_target(COMPACT_WIDTH);
                    ctx.height_spring.set_target(COMPACT_HEIGHT);

                    if !ctx.is_animating {
                        ctx.is_animating = true;
                        SetTimer(hwnd, TIMER_ANIMATION, 16, None);
                    }
                }
            }
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;

            if let Some(ctx) = (*ctx_ptr).as_mut() {
                let expand_t = ((ctx.width_spring.current - COMPACT_WIDTH) / (EXPANDED_WIDTH - COMPACT_WIDTH)).clamp(0.0, 1.0);
                if expand_t > 0.7 {
                    let cur_w = ctx.width_spring.current;
                    let cur_h = ctx.height_spring.current;
                    let is_flat_top = ctx.settings.mode == Mode::Notch;
                    let x0 = ((CANVAS_WIDTH as f32) - cur_w) * 0.5;
                    let y0 = if is_flat_top { 0.0 } else { 10.0 };

                    ctx.press_hit = hit_test(x, y, x0, y0, cur_w, cur_h, ctx.page_state.current_page);
                    ctx.pager.press(x);
                    let _ = SetCapture(hwnd);
                }
            }
            LRESULT(0)
        }

        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as f32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as f32;
            let _ = ReleaseCapture();

            if let Some(ctx) = (*ctx_ptr).as_mut() {
                let cur_w = ctx.width_spring.current;
                let cur_h = ctx.height_spring.current;
                let is_flat_top = ctx.settings.mode == Mode::Notch;
                let x0 = ((CANVAS_WIDTH as f32) - cur_w) * 0.5;
                let y0 = if is_flat_top { 0.0 } else { 10.0 };

                let release_hit = hit_test(x, y, x0, y0, cur_w, cur_h, ctx.page_state.current_page);

                match ctx.pager.release() {
                    Release::Swiped { to, .. } => {
                        ctx.page_state.current_page = PageId::from_index(to);
                        if !ctx.is_animating {
                            ctx.is_animating = true;
                            SetTimer(hwnd, TIMER_ANIMATION, 16, None);
                        }
                        update_window_surface();
                    }
                    Release::Click => {
                        if release_hit != HitTarget::None && release_hit == ctx.press_hit {
                            match release_hit {
                                HitTarget::PageDot(i) => {
                                    ctx.page_state.current_page = PageId::from_index(i);
                                    ctx.pager.jump_to(i);
                                    update_window_surface();
                                }
                                HitTarget::MediaPlayPause => {
                                    ctx.media_hook.send(MediaCmd::PlayPause);
                                    ctx.page_state.is_playing = !ctx.page_state.is_playing;
                                    update_window_surface();
                                }
                                HitTarget::MediaPrev => {
                                    ctx.media_hook.send(MediaCmd::Prev);
                                }
                                HitTarget::MediaNext => {
                                    ctx.media_hook.send(MediaCmd::Next);
                                }
                                HitTarget::Scrubber => {
                                    let bar_x = x0 + 20.0;
                                    let bar_w = cur_w - 40.0;
                                    let ratio = ((x - bar_x) / bar_w).clamp(0.0, 1.0);
                                    ctx.page_state.track_progress = ratio;
                                    update_window_surface();
                                }
                                HitTarget::PomoMode(i) => {
                                    ctx.page_state.set_pomo_mode(i);
                                    update_window_surface();
                                }
                                HitTarget::PomoStart => {
                                    ctx.page_state.toggle_pomo();
                                    update_window_surface();
                                }
                                HitTarget::PomoReset => {
                                    ctx.page_state.reset_pomo();
                                    update_window_surface();
                                }
                                HitTarget::WaterAdd250 => {
                                    ctx.page_state.add_water(250);
                                    update_window_surface();
                                }
                                HitTarget::WaterAdd500 => {
                                    ctx.page_state.add_water(500);
                                    update_window_surface();
                                }
                                HitTarget::WaterReset => {
                                    ctx.page_state.reset_water();
                                    update_window_surface();
                                }
                                HitTarget::TaskItem(i) => {
                                    ctx.page_state.toggle_task(i);
                                    update_window_surface();
                                }
                                HitTarget::ShelfOpen(_) => {
                                    let _ = std::process::Command::new("explorer").arg(r"D:\Downloads").spawn();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ctx.press_hit = HitTarget::None;
            }
            LRESULT(0)
        }

        WM_RBUTTONUP => {
            if let Some(ctx) = (*ctx_ptr).as_mut() {
                if let Some(cmd) = show_tray_menu(hwnd, &ctx.settings, Settings::monitor_count()) {
                    handle_tray_cmd(ctx, cmd);
                }
            }
            LRESULT(0)
        }

        WM_TRAY => {
            let ev = (lparam.0 as u32) & 0xFFFF;
            if ev == WM_RBUTTONUP || ev == WM_CONTEXTMENU {
                if let Some(ctx) = (*ctx_ptr).as_mut() {
                    if let Some(cmd) = show_tray_menu(hwnd, &ctx.settings, Settings::monitor_count()) {
                        handle_tray_cmd(ctx, cmd);
                    }
                }
            }
            LRESULT(0)
        }

        WM_MEDIA_CHANGED => {
            if let Some(ctx) = (*ctx_ptr).as_mut() {
                ctx.media = ctx.media_hook.snapshot();
                update_window_surface();
            }
            LRESULT(0)
        }

        WM_TIMER => {
            let timer_id = wparam.0;
            if timer_id == TIMER_ANIMATION {
                if let Some(ctx) = (*ctx_ptr).as_mut() {
                    let dt = 0.016;
                    let w_moving = ctx.width_spring.update(dt);
                    let h_moving = ctx.height_spring.update(dt);
                    let pager_moving = ctx.pager.tick(dt);

                    let is_playing = if ctx.media.has_session {
                        ctx.media.playing
                    } else {
                        ctx.page_state.is_playing
                    };

                    if is_playing {
                        for bar in &mut ctx.page_state.eq_bars {
                            *bar = (*bar + 0.12) % 1.0;
                        }
                    }

                    update_window_surface();

                    // Zero-Idle: kill timer if all springs settled and music not actively playing
                    if !w_moving && !h_moving && !pager_moving && (!is_playing || !ctx.is_hovered) {
                        ctx.is_animating = false;
                        let _ = KillTimer(hwnd, TIMER_ANIMATION);
                    }
                }
            } else if timer_id == TIMER_MASCOT_BLINK {
                if let Some(ctx) = (*ctx_ptr).as_mut() {
                    let need_redraw = ctx.mascot.update_blink(0.1);

                    ctx.tick_counter += 1;
                    let mut pomo_tick = false;
                    if ctx.tick_counter % 10 == 0 {
                        if ctx.page_state.pomo_running && ctx.page_state.pomo_seconds_left > 0 {
                            ctx.page_state.pomo_seconds_left -= 1;
                            pomo_tick = true;
                        }
                    }

                    if (need_redraw || pomo_tick) && !ctx.is_animating {
                        update_window_surface();
                    }
                }
            }
            LRESULT(0)
        }

        WM_SETCURSOR => {
            SetCursor(LoadCursorW(None, IDC_ARROW).unwrap_or_default());
            LRESULT(1)
        }

        WM_DESTROY => {
            remove_tray_icon(hwnd);
            let _ = KillTimer(hwnd, TIMER_ANIMATION);
            let _ = KillTimer(hwnd, TIMER_MASCOT_BLINK);
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn handle_tray_cmd(ctx: &mut WindowContext, cmd: TrayCmd) {
    match cmd {
        TrayCmd::Mode(m) => {
            ctx.settings.mode = m;
            ctx.settings.save();
            let (x, y) = ctx.settings.anchor(CANVAS_WIDTH, 10);
            unsafe {
                let _ = SetWindowPos(ctx.hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
            }
            unsafe { update_window_surface(); }
        }
        TrayCmd::Monitor(mon) => {
            ctx.settings.monitor = mon;
            ctx.settings.save();
            let (x, y) = ctx.settings.anchor(CANVAS_WIDTH, 10);
            unsafe {
                let _ = SetWindowPos(ctx.hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
            }
            unsafe { update_window_surface(); }
        }
        TrayCmd::ToggleStartup => {
            let on = !ctx.settings.autostart;
            ctx.settings.set_autostart(on);
        }
        TrayCmd::Quit => unsafe {
            let _ = DestroyWindow(ctx.hwnd);
        },
    }
}
