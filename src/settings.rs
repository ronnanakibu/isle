//! Isle Configuration & Monitor Management
use std::mem::{size_of, zeroed};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
        System::{
            LibraryLoader::GetModuleFileNameW,
            Registry::*,
        },
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Notch,
    Floating,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MonitorChoice {
    Primary,
    Secondary,
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub mode: Mode,
    pub monitor: MonitorChoice,
    pub autostart: bool,
}

const APP_KEY: PCWSTR = w!("Software\\Isle");
const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

fn open_app_key() -> Option<HKEY> {
    let mut h = HKEY::default();
    let r = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            APP_KEY,
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_READ | KEY_WRITE,
            None,
            &mut h,
            None,
        )
    };
    r.is_ok().then_some(h)
}

fn get_dword(h: HKEY, name: PCWSTR) -> Option<u32> {
    let (mut v, mut sz) = (0u32, 4u32);
    let r = unsafe {
        RegQueryValueExW(
            h,
            name,
            None,
            None,
            Some(&mut v as *mut u32 as *mut u8),
            Some(&mut sz),
        )
    };
    r.is_ok().then_some(v)
}

fn set_dword(h: HKEY, name: PCWSTR, v: u32) {
    unsafe {
        let _ = RegSetValueExW(h, name, 0, REG_DWORD, Some(&v.to_ne_bytes()));
    }
}

fn run_value_exists(name: PCWSTR) -> bool {
    unsafe {
        let mut h = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_READ, &mut h).is_ok() {
            let exists = RegQueryValueExW(h, name, None, None, None, None).is_ok();
            let _ = RegCloseKey(h);
            return exists;
        }
        false
    }
}

impl Settings {
    pub fn load() -> Self {
        let mut s = Settings {
            mode: Mode::Notch,
            monitor: MonitorChoice::Primary,
            autostart: false,
        };
        if let Some(h) = open_app_key() {
            if get_dword(h, w!("Mode")) == Some(1) {
                s.mode = Mode::Floating;
            }
            if get_dword(h, w!("Monitor")) == Some(1) {
                s.monitor = MonitorChoice::Secondary;
            }
            unsafe {
                let _ = RegCloseKey(h);
            }
        }
        s.autostart = run_value_exists(w!("Isle"));
        s
    }

    pub fn save(&self) {
        if let Some(h) = open_app_key() {
            set_dword(h, w!("Mode"), (self.mode == Mode::Floating) as u32);
            set_dword(h, w!("Monitor"), (self.monitor == MonitorChoice::Secondary) as u32);
            unsafe {
                let _ = RegCloseKey(h);
            }
        }
    }

    pub fn set_autostart(&mut self, on: bool) {
        unsafe {
            let mut h = HKEY::default();
            if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_WRITE, &mut h).is_ok() {
                if on {
                    let mut buf = [0u16; 520];
                    let n = GetModuleFileNameW(None, &mut buf) as usize;
                    let mut wide: Vec<u16> = Vec::with_capacity(n + 3);
                    wide.push(b'"' as u16);
                    wide.extend_from_slice(&buf[..n]);
                    wide.push(b'"' as u16);
                    wide.push(0);
                    let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
                    let _ = RegSetValueExW(h, w!("Isle"), 0, REG_SZ, Some(bytes));
                } else {
                    let _ = RegDeleteValueW(h, w!("Isle"));
                }
                let _ = RegCloseKey(h);
            }
        }
        self.autostart = on;
    }

    pub fn monitor_count() -> usize {
        monitors().len()
    }

    pub fn anchor(&self, win_w: i32, float_gap: i32) -> (i32, i32) {
        let list = monitors();
        let rc = match self.monitor {
            MonitorChoice::Secondary => list.iter().find(|m| !m.1).or_else(|| list.first()),
            MonitorChoice::Primary => list.iter().find(|m| m.1).or_else(|| list.first()),
        }
        .map(|m| m.0)
        .unwrap_or(RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        });

        let x = rc.left + ((rc.right - rc.left) - win_w) / 2;
        let y = rc.top + if self.mode == Mode::Floating { float_gap } else { 0 };
        (x, y)
    }
}

fn monitors() -> Vec<(RECT, bool)> {
    unsafe extern "system" fn cb(h: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
        let v = &mut *(lp.0 as *mut Vec<(RECT, bool)>);
        let mut mi: MONITORINFO = zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(h, &mut mi).as_bool() {
            v.push((mi.rcMonitor, (mi.dwFlags & 1) != 0));
        }
        BOOL(1)
    }
    let mut v: Vec<(RECT, bool)> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(cb), LPARAM(&mut v as *mut _ as isize));
    }
    v
}
