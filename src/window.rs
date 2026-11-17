//! Win32 Layered Window Management for Apple Dynamic Island
//! Zero-Idle CPU lifecycle, smooth 60 FPS spring physics, and crisp typography

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

const TIMER_ANIMATION: usize = 1;
const TIMER_MASCOT_BLINK: usize = 2;
const WIN_WM_MOUSELEAVE: u32 = 0x02A3;

pub struct WindowContext {
    pub hwnd: HWND,
    pub hdc_mem: HDC,
    pub hbitmap: HBITMAP,
    pub p_bits: *mut u32,
    pub surface: Surface,
    pub width_spring: Spring,
    pub height_spring: Spring,
    pub mascot: Mascot,
    pub page_state: PageState,
    pub is_hovered: bool,
    pub is_animating: bool,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub tick_counter: u32,
}

static mut CONTEXT: Option<WindowContext> = None;

unsafe fn send_media_key(key: VIRTUAL_KEY) {
    keybd_event(key.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
    keybd_event(key.0 as u8, 0, KEYEVENTF_KEYUP, 0);
}

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

    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let win_x = (screen_w - CANVAS_WIDTH) / 2;
    let win_y = 0;

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

    // Initial state: Start expanded and fluidly spring-contract into the compact pill on launch!
    let mut w_spring = Spring::new(EXPANDED_WIDTH);
    let mut h_spring = Spring::new(EXPANDED_HEIGHT);
    w_spring.current = EXPANDED_WIDTH;
    h_spring.current = EXPANDED_HEIGHT;
    w_spring.set_target(COMPACT_WIDTH);
    h_spring.set_target(COMPACT_HEIGHT);

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
        is_hovered: false,
        is_animating: true,
        mouse_x: 0,
        mouse_y: 0,
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

    update_window_surface();

    // Start 60 FPS animation timer for the intro spring transition
    SetTimer(hwnd, TIMER_ANIMATION, 16, None);

    // Start mascot blink timer (100ms check)
    SetTimer(hwnd, TIMER_MASCOT_BLINK, 100, None);

    // Message Loop (Zero-idle: Sleep in GetMessageW until an event arrives)
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

        // 1. High-fidelity vector/squircle rasterization pass
        render_apple_island(
            &mut ctx.surface,
            &ctx.width_spring,
            &ctx.height_spring,
            smoothed_t,
            &ctx.mascot,
            &ctx.page_state,
        );

        // Copy rendered pixels into DIB Section
        std::ptr::copy_nonoverlapping(
            ctx.surface.pixels.as_ptr(),
            ctx.p_bits,
            (CANVAS_WIDTH * CANVAS_HEIGHT) as usize,
        );

        let cur_w = ctx.width_spring.current as i32;
        let x0 = (CANVAS_WIDTH - cur_w) / 2;
        let y0 = 10;

        // 2. Crisp GDI Typography Pass for Expanded state
        if smoothed_t > 0.3 {
            render_gdi_text_pass(
                ctx.hdc_mem,
                x0,
                y0,
                cur_w,
                smoothed_t,
                &ctx.page_state,
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
                    // If GDI drew non-black text onto the surface, ensure full 255 alpha
                    if rgb > 0x001A1A1A {
                        *ctx.p_bits.add(idx) = 0xFF000000 | rgb;
                    }
                }
            }
        }

        // 3. UpdateLayeredWindow
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let pt_dst = POINT {
            x: (screen_w - CANVAS_WIDTH) / 2,
            y: 0,
        };
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

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ctx_ptr = &raw mut CONTEXT;
    match msg {
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if let Some(ctx) = (*ctx_ptr).as_mut() {
                ctx.mouse_x = x;
                ctx.mouse_y = y;

                let center_x = (CANVAS_WIDTH as f32) * 0.5;
                ctx.mascot.update_gaze((x as f32) - center_x, (y as f32) - 18.0);

                let cur_w = ctx.width_spring.current;
                let cur_h = ctx.height_spring.current;
                let x0 = center_x - cur_w * 0.5;
                let y0 = 10.0;

                // Sensitive hit area: covers top bezel y >= 0 down to island base + margin
                let inside_island = (x as f32) >= (x0 - 6.0)
                    && (x as f32) <= (x0 + cur_w + 6.0)
                    && (y as f32) >= 0.0
                    && (y as f32) <= (y0 + cur_h + 8.0);

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
                } else if ctx.is_hovered {
                    ctx.is_hovered = false;
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
                ctx.is_hovered = false;
                ctx.width_spring.set_target(COMPACT_WIDTH);
                ctx.height_spring.set_target(COMPACT_HEIGHT);

                if !ctx.is_animating {
                    ctx.is_animating = true;
                    SetTimer(hwnd, TIMER_ANIMATION, 16, None);
                }
            }
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if let Some(ctx) = (*ctx_ptr).as_mut() {
                let expand_t = ((ctx.width_spring.current - COMPACT_WIDTH) / (EXPANDED_WIDTH - COMPACT_WIDTH)).clamp(0.0, 1.0);
                if expand_t > 0.7 {
                    let cur_h = ctx.height_spring.current;
                    let cur_w = ctx.width_spring.current;
                    let y0 = 10.0;
                    let center_x = (CANVAS_WIDTH as f32) * 0.5;
                    let x0 = center_x - cur_w * 0.5;

                    // 1. Check Bottom Page Indicator Dots
                    let dot_y = y0 + cur_h - 12.0;
                    if ((y as f32) - dot_y).abs() < 12.0 {
                        let dots_count = PageId::ALL.len();
                        let spacing = 14.0;
                        let start_x = center_x - ((dots_count as f32 - 1.0) * spacing) * 0.5;
                        let mut clicked_dot = false;
                        for (i, p) in PageId::ALL.iter().enumerate() {
                            let px = start_x + (i as f32) * spacing;
                            if ((x as f32) - px).abs() < 10.0 {
                                ctx.page_state.current_page = *p;
                                clicked_dot = true;
                                break;
                            }
                        }
                        if !clicked_dot {
                            if (x as f32) < center_x {
                                ctx.page_state.current_page = ctx.page_state.current_page.prev();
                            } else {
                                ctx.page_state.current_page = ctx.page_state.current_page.next();
                            }
                        }
                        update_window_surface();
                        return LRESULT(0);
                    }

                    // 2. Page Specific Interactive Clicks
                    match ctx.page_state.current_page {
                        PageId::Music => {
                            let ctrl_cy = y0 + 116.0;
                            if ((y as f32) - ctrl_cy).abs() < 24.0 {
                                // Play / Pause
                                if ((x as f32) - center_x).abs() < 24.0 {
                                    ctx.page_state.is_playing = !ctx.page_state.is_playing;
                                    send_media_key(VK_MEDIA_PLAY_PAUSE);
                                    update_window_surface();
                                }
                                // Prev
                                else if ((x as f32) - (center_x - 56.0)).abs() < 18.0 {
                                    send_media_key(VK_MEDIA_PREV_TRACK);
                                    ctx.page_state.track_progress = (ctx.page_state.track_progress - 0.1).max(0.0);
                                    update_window_surface();
                                }
                                // Next
                                else if ((x as f32) - (center_x + 56.0)).abs() < 18.0 {
                                    send_media_key(VK_MEDIA_NEXT_TRACK);
                                    ctx.page_state.track_progress = (ctx.page_state.track_progress + 0.1).min(1.0);
                                    update_window_surface();
                                }
                            } else if ((y as f32) - (y0 + 84.0)).abs() < 10.0 {
                                // Scrubber bar clicked
                                let bar_x = x0 + 20.0;
                                let bar_w = cur_w - 40.0;
                                let ratio = (((x as f32) - bar_x) / bar_w).clamp(0.0, 1.0);
                                ctx.page_state.track_progress = ratio;
                                update_window_surface();
                            }
                        }
                        PageId::Pomodoro => {
                            // Mode Pills at y0 + 74
                            if (y as f32) >= y0 + 70.0 && (y as f32) <= y0 + 98.0 {
                                let pill_w = 78.0;
                                let spacing = 8.0;
                                let total_w = 3.0 * pill_w + 2.0 * spacing;
                                let start_x = x0 + (cur_w - total_w) * 0.5;
                                for i in 0..3 {
                                    let px = start_x + (i as f32) * (pill_w + spacing);
                                    if (x as f32) >= px && (x as f32) <= px + pill_w {
                                        ctx.page_state.set_pomo_mode(i);
                                        update_window_surface();
                                        break;
                                    }
                                }
                            }
                            // Action Buttons at y0 + 104
                            else if (y as f32) >= y0 + 100.0 && (y as f32) <= y0 + 130.0 {
                                if (x as f32) >= center_x - 52.0 && (x as f32) <= center_x + 20.0 {
                                    ctx.page_state.toggle_pomo();
                                    update_window_surface();
                                } else if (x as f32) >= center_x + 28.0 && (x as f32) <= center_x + 80.0 {
                                    ctx.page_state.reset_pomo();
                                    update_window_surface();
                                }
                            }
                        }
                        PageId::Water => {
                            // Buttons at y0 + 96
                            if (y as f32) >= y0 + 92.0 && (y as f32) <= y0 + 124.0 {
                                if (x as f32) >= x0 + 75.0 && (x as f32) <= x0 + 151.0 {
                                    ctx.page_state.add_water(250);
                                    update_window_surface();
                                } else if (x as f32) >= x0 + 160.0 && (x as f32) <= x0 + 236.0 {
                                    ctx.page_state.add_water(500);
                                    update_window_surface();
                                } else if (x as f32) >= x0 + 245.0 && (x as f32) <= x0 + 305.0 {
                                    ctx.page_state.reset_water();
                                    update_window_surface();
                                }
                            }
                        }
                        PageId::Todo => {
                            // Tasks at y0 + 36 + i * 28
                            for i in 0..ctx.page_state.tasks.len().min(3) {
                                let row_y = y0 + 36.0 + (i as f32) * 28.0;
                                if (y as f32) >= row_y - 2.0 && (y as f32) <= row_y + 24.0 {
                                    ctx.page_state.toggle_task(i);
                                    update_window_surface();
                                    break;
                                }
                            }
                        }
                        PageId::Shelf => {
                            // Shelf cards at y0 + 38 + i * 38
                            for i in 0..ctx.page_state.shelf_items.len().min(2) {
                                let card_y = y0 + 38.0 + (i as f32) * 38.0;
                                if (y as f32) >= card_y && (y as f32) <= card_y + 34.0 {
                                    let _ = std::process::Command::new("explorer").arg(r"D:\Downloads").spawn();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            LRESULT(0)
        }

        WM_TIMER => {
            let timer_id = wparam.0;
            if timer_id == TIMER_ANIMATION {
                if let Some(ctx) = (*ctx_ptr).as_mut() {
                    let dt = 0.016; // 60 FPS step
                    let w_moving = ctx.width_spring.update(dt);
                    let h_moving = ctx.height_spring.update(dt);

                    // Dynamic equalizer bounce while playing
                    if ctx.page_state.is_playing {
                        for bar in &mut ctx.page_state.eq_bars {
                            *bar = (*bar + 0.12) % 1.0;
                        }
                    }

                    update_window_surface();

                    // Zero-Idle: If both springs have settled, kill animation timer
                    if !w_moving && !h_moving {
                        ctx.is_animating = false;
                        let _ = KillTimer(hwnd, TIMER_ANIMATION);
                    }
                }
            } else if timer_id == TIMER_MASCOT_BLINK {
                if let Some(ctx) = (*ctx_ptr).as_mut() {
                    let need_redraw = ctx.mascot.update_blink(0.1);

                    // Second-ticking for pomodoro
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
            let _ = KillTimer(hwnd, TIMER_ANIMATION);
            let _ = KillTimer(hwnd, TIMER_MASCOT_BLINK);
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
