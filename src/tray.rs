//! System Tray Icon & Modal Settings Menu
use crate::settings::{Mode, MonitorChoice, Settings, TriggerMode};
use std::mem::{size_of, zeroed};
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, POINT, WPARAM},
        UI::{
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
};

pub const WM_TRAY: u32 = WM_APP + 1;
const ID_NOTCH: usize = 1001;
const ID_FLOAT: usize = 1002;
const ID_MON_PRIMARY: usize = 1003;
const ID_MON_SECONDARY: usize = 1004;
const ID_STARTUP: usize = 1005;
const ID_QUIT: usize = 1006;
const ID_TRIGGER_CLICK: usize = 1007;
const ID_TRIGGER_HOVER: usize = 1008;

pub enum TrayCmd {
    Mode(Mode),
    Monitor(MonitorChoice),
    Trigger(TriggerMode),
    ToggleStartup,
    Quit,
}

fn nid(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut n: NOTIFYICONDATAW = unsafe { zeroed() };
    n.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    n.hWnd = hwnd;
    n.uID = 1;
    n
}

pub fn add_tray_icon(hwnd: HWND) {
    let mut n = nid(hwnd);
    n.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    n.uCallbackMessage = WM_TRAY;
    n.hIcon = unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() };
    for (d, c) in n.szTip.iter_mut().zip("Isle — Dynamic Island".encode_utf16()) {
        *d = c;
    }
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &n);
    }
}

pub fn remove_tray_icon(hwnd: HWND) {
    let n = nid(hwnd);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &n);
    }
}

pub fn show_tray_menu(hwnd: HWND, s: &Settings, monitor_count: usize) -> Option<TrayCmd> {
    unsafe {
        let menu = CreatePopupMenu().ok()?;
        let chk = |on: bool| if on { MF_STRING | MF_CHECKED } else { MF_STRING };

        let _ = AppendMenuW(menu, chk(s.mode == Mode::Notch), ID_NOTCH, w!("Notch Mode (Docked)"));
        let _ = AppendMenuW(menu, chk(s.mode == Mode::Floating), ID_FLOAT, w!("Floating Pill Mode (10px)"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);

        let _ = AppendMenuW(menu, chk(s.trigger == TriggerMode::Click), ID_TRIGGER_CLICK, w!("Click to Expand"));
        let _ = AppendMenuW(menu, chk(s.trigger == TriggerMode::Hover), ID_TRIGGER_HOVER, w!("Hover to Expand"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);

        let _ = AppendMenuW(menu, chk(s.monitor == MonitorChoice::Primary), ID_MON_PRIMARY, w!("Primary Display"));
        let sec = chk(s.monitor == MonitorChoice::Secondary)
            | if monitor_count < 2 { MF_GRAYED } else { MENU_ITEM_FLAGS(0) };
        let _ = AppendMenuW(menu, sec, ID_MON_SECONDARY, w!("Secondary Display"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);

        let _ = AppendMenuW(menu, chk(s.autostart), ID_STARTUP, w!("Run on Windows Startup"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, ID_QUIT, w!("Quit Isle"));

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);

        let id = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_RETURNCMD,
            pt.x,
            pt.y,
            0,
            hwnd,
            None,
        )
        .0 as usize;

        let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);

        match id {
            ID_NOTCH => Some(TrayCmd::Mode(Mode::Notch)),
            ID_FLOAT => Some(TrayCmd::Mode(Mode::Floating)),
            ID_MON_PRIMARY => Some(TrayCmd::Monitor(MonitorChoice::Primary)),
            ID_MON_SECONDARY => Some(TrayCmd::Monitor(MonitorChoice::Secondary)),
            ID_STARTUP => Some(TrayCmd::ToggleStartup),
            ID_TRIGGER_CLICK => Some(TrayCmd::Trigger(TriggerMode::Click)),
            ID_TRIGGER_HOVER => Some(TrayCmd::Trigger(TriggerMode::Hover)),
            ID_QUIT => Some(TrayCmd::Quit),
            _ => None,
        }
    }
}
