use tauri::Manager;
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::menu::{MenuBuilder, MenuItemBuilder, CheckMenuItemBuilder, PredefinedMenuItem};
use tauri::Emitter;

// ---------------------------------------------------------------------------
// Tray Menu State — holds CheckMenuItem handles for bidirectional sync
// ---------------------------------------------------------------------------
struct TrayMenuState {
  item_click: tauri::menu::CheckMenuItem<tauri::Wry>,
  item_hover: tauri::menu::CheckMenuItem<tauri::Wry>,
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
  let handle = app.handle();

  let item_settings = MenuItemBuilder::with_id("open_settings", "Open Settings...").build(handle)?;
  let item_click = CheckMenuItemBuilder::with_id("trigger_click", "Click to Expand").checked(true).build(handle)?;
  let item_hover = CheckMenuItemBuilder::with_id("trigger_hover", "Hover to Expand").checked(false).build(handle)?;
  let item_toggle = MenuItemBuilder::with_id("toggle_island", "Toggle Island").build(handle)?;
  let item_exit = MenuItemBuilder::with_id("exit_app", "Exit Isle").build(handle)?;
  let sep1 = PredefinedMenuItem::separator(handle)?;
  let sep2 = PredefinedMenuItem::separator(handle)?;
  let sep3 = PredefinedMenuItem::separator(handle)?;

  let menu = MenuBuilder::new(handle)
    .items(&[&item_settings, &sep1, &item_click, &item_hover, &sep2, &item_toggle, &sep3, &item_exit])
    .build()?;

  // Store check items in managed state for bidirectional sync
  app.manage(TrayMenuState {
    item_click: item_click.clone(),
    item_hover: item_hover.clone(),
  });

  let icon = app.default_window_icon().unwrap().clone();

  TrayIconBuilder::new()
    .icon(icon)
    .tooltip("Isle — Living Dynamic Island")
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
        let app = tray.app_handle();
        let _ = app.emit("tray-open-settings", ());
      }
    })
    .on_menu_event(|app, event| {
      match event.id.as_ref() {
        "open_settings" => { let _ = app.emit("tray-open-settings", ()); },
        "trigger_click" => {
          let _ = app.emit("tray-change-trigger", "click");
          if let Some(state) = app.try_state::<TrayMenuState>() {
            let _ = state.item_click.set_checked(true);
            let _ = state.item_hover.set_checked(false);
          }
        },
        "trigger_hover" => {
          let _ = app.emit("tray-change-trigger", "hover");
          if let Some(state) = app.try_state::<TrayMenuState>() {
            let _ = state.item_click.set_checked(false);
            let _ = state.item_hover.set_checked(true);
          }
        },
        "toggle_island" => { let _ = app.emit("tray-toggle-island", ()); },
        "exit_app" => { std::process::exit(0); },
        _ => {}
      }
    })
    .build(handle)?;

  Ok(())
}

// ---------------------------------------------------------------------------
// Bidirectional Tray Sync Command
// ---------------------------------------------------------------------------
#[tauri::command]
fn sync_expand_trigger_tray(app: tauri::AppHandle, mode: String) {
  if let Some(state) = app.try_state::<TrayMenuState>() {
    match mode.as_str() {
      "click" => {
        let _ = state.item_click.set_checked(true);
        let _ = state.item_hover.set_checked(false);
      }
      "hover" => {
        let _ = state.item_click.set_checked(false);
        let _ = state.item_hover.set_checked(true);
      }
      _ => {}
    }
  }
}

// ---------------------------------------------------------------------------
// Real Battery Status via GetSystemPowerStatus
// ---------------------------------------------------------------------------
#[derive(serde::Serialize, Clone)]
pub struct RealBatteryStatus {
  pub percentage: u8,
  pub charging: bool,
}

#[tauri::command]
fn get_real_battery() -> RealBatteryStatus {
  #[cfg(target_os = "windows")]
  {
    #[repr(C)]
    struct SYSTEM_POWER_STATUS {
      ac_line_status: u8,
      battery_flag: u8,
      battery_life_percent: u8,
      system_status_flag: u8,
      battery_life_time: u32,
      battery_full_life_time: u32,
    }
    unsafe extern "system" {
      fn GetSystemPowerStatus(status: *mut SYSTEM_POWER_STATUS) -> i32;
    }
    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    if unsafe { GetSystemPowerStatus(&mut status) } != 0 {
      let pct = if status.battery_life_percent <= 100 { status.battery_life_percent } else { 100 };
      return RealBatteryStatus {
        percentage: pct,
        charging: status.ac_line_status == 1,
      };
    }
  }
  RealBatteryStatus { percentage: 100, charging: true }
}

// ---------------------------------------------------------------------------
// Real Hardware Telemetry — GlobalMemoryStatusEx + GetSystemTimes delta CPU
// ---------------------------------------------------------------------------
#[derive(serde::Serialize, Clone)]
pub struct RealHardwareTelemetry {
  pub ram_percent: u8,
  pub ram_used_gb: f32,
  pub ram_total_gb: f32,
  pub cpu_percent: u8,
}

// Store previous GetSystemTimes ticks for delta CPU calculation
struct CpuTimeState {
  prev_idle: u64,
  prev_kernel: u64,
  prev_user: u64,
}

static CPU_STATE: std::sync::Mutex<Option<CpuTimeState>> = std::sync::Mutex::new(None);

#[tauri::command]
fn get_real_hardware() -> RealHardwareTelemetry {
  #[cfg(target_os = "windows")]
  {
    #[repr(C)]
    struct MEMORYSTATUSEX {
      dw_length: u32,
      dw_memory_load: u32,
      ull_total_phys: u64,
      ull_avail_phys: u64,
      ull_total_page_file: u64,
      ull_avail_page_file: u64,
      ull_total_virtual: u64,
      ull_avail_virtual: u64,
      ull_avail_extended_virtual: u64,
    }
    #[repr(C)]
    struct FILETIME {
      dw_low_date_time: u32,
      dw_high_date_time: u32,
    }
    unsafe extern "system" {
      fn GlobalMemoryStatusEx(stat: *mut MEMORYSTATUSEX) -> i32;
      fn GetSystemTimes(idle: *mut FILETIME, kernel: *mut FILETIME, user: *mut FILETIME) -> i32;
    }

    fn filetime_to_u64(ft: &FILETIME) -> u64 {
      ((ft.dw_high_date_time as u64) << 32) | (ft.dw_low_date_time as u64)
    }

    // RAM
    let mut mem: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem.dw_length = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    let (ram_percent, ram_used_gb, ram_total_gb) = if unsafe { GlobalMemoryStatusEx(&mut mem) } != 0 {
      let total_gb = mem.ull_total_phys as f32 / (1024.0 * 1024.0 * 1024.0);
      let avail_gb = mem.ull_avail_phys as f32 / (1024.0 * 1024.0 * 1024.0);
      let used_gb = total_gb - avail_gb;
      (mem.dw_memory_load as u8, used_gb, total_gb)
    } else {
      (50, 8.0, 16.0)
    };

    // CPU — delta from previous sample
    let mut idle_ft: FILETIME = unsafe { std::mem::zeroed() };
    let mut kernel_ft: FILETIME = unsafe { std::mem::zeroed() };
    let mut user_ft: FILETIME = unsafe { std::mem::zeroed() };

    let cpu_percent = if unsafe { GetSystemTimes(&mut idle_ft, &mut kernel_ft, &mut user_ft) } != 0 {
      let idle = filetime_to_u64(&idle_ft);
      let kernel = filetime_to_u64(&kernel_ft);
      let user = filetime_to_u64(&user_ft);

      let mut guard = CPU_STATE.lock().unwrap();
      let pct = if let Some(ref prev) = *guard {
        let d_idle = idle.wrapping_sub(prev.prev_idle);
        let d_kernel = kernel.wrapping_sub(prev.prev_kernel);
        let d_user = user.wrapping_sub(prev.prev_user);
        // kernel includes idle time
        let d_sys = d_kernel + d_user;
        if d_sys == 0 {
          0u8
        } else {
          let busy = d_sys.saturating_sub(d_idle);
          ((busy as f64 / d_sys as f64) * 100.0).round().min(100.0) as u8
        }
      } else {
        0u8
      };
      *guard = Some(CpuTimeState { prev_idle: idle, prev_kernel: kernel, prev_user: user });
      pct
    } else {
      10
    };

    return RealHardwareTelemetry { ram_percent, ram_used_gb, ram_total_gb, cpu_percent };
  }
  #[allow(unreachable_code)]
  RealHardwareTelemetry { ram_percent: 50, ram_used_gb: 8.0, ram_total_gb: 16.0, cpu_percent: 10 }
}

// ---------------------------------------------------------------------------
// Real Media Status via WinRT SMTC
// ---------------------------------------------------------------------------
#[derive(serde::Serialize, Clone)]
pub struct RealMediaStatus {
  pub has_session: bool,
  pub title: String,
  pub artist: String,
  pub album: String,
  pub is_playing: bool,
  pub position_sec: f64,
  pub duration_sec: f64,
}

#[tauri::command]
fn get_real_media() -> RealMediaStatus {
  // WinRT SMTC is initialized on a dedicated background thread (started in run())
  // This command returns the latest cached media state from the shared state
  if let Ok(guard) = MEDIA_STATE.lock() {
    if let Some(ref s) = *guard {
      return s.clone();
    }
  }
  RealMediaStatus {
    has_session: false,
    title: String::new(),
    artist: String::new(),
    album: String::new(),
    is_playing: false,
    position_sec: 0.0,
    duration_sec: 0.0,
  }
}

static MEDIA_STATE: std::sync::Mutex<Option<RealMediaStatus>> = std::sync::Mutex::new(None);

#[tauri::command]
fn media_control(action: String) {
  // Send control to the background SMTC thread via a channel
  if let Ok(guard) = MEDIA_CMD_TX.lock() {
    if let Some(ref tx) = *guard {
      let _ = tx.send(action);
    }
  }
}

static MEDIA_CMD_TX: std::sync::Mutex<Option<std::sync::mpsc::Sender<String>>> = std::sync::Mutex::new(None);

fn start_smtc_thread(app_handle: tauri::AppHandle) {
  let (tx, rx) = std::sync::mpsc::channel::<String>();
  {
    let mut guard = MEDIA_CMD_TX.lock().unwrap();
    *guard = Some(tx);
  }

  std::thread::spawn(move || {
    #[cfg(target_os = "windows")]
    {
      use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
      };
      use windows::Foundation::TimeSpan;

      // Initialize WinRT on this thread
      let _ = unsafe { windows::Win32::System::WinRT::RoInitialize(
        windows::Win32::System::WinRT::RO_INIT_MULTITHREADED
      ) };

      fn read_session(manager: &GlobalSystemMediaTransportControlsSessionManager) -> Option<RealMediaStatus> {
        let session = manager.GetCurrentSession().ok()?;
        let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
        let pb = session.GetPlaybackInfo().ok()?;
        let title = props.Title().unwrap_or_default().to_string();
        let artist = props.Artist().unwrap_or_default().to_string();
        let album = props.AlbumTitle().unwrap_or_default().to_string();
        let is_playing = pb.PlaybackStatus().ok()?
          == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
        let tl = session.GetTimelineProperties().ok()?;
        let pos_ticks = tl.Position().ok().map(|t: TimeSpan| t.Duration).unwrap_or(0);
        let dur_ticks = tl.EndTime().ok().map(|t: TimeSpan| t.Duration).unwrap_or(0);
        Some(RealMediaStatus {
          has_session: true,
          title,
          artist,
          album,
          is_playing,
          position_sec: pos_ticks as f64 / 10_000_000.0,
          duration_sec: dur_ticks as f64 / 10_000_000.0,
        })
      }

      let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|op| op.get())
      {
        Ok(m) => m,
        Err(_) => return,
      };

      loop {
        // Check for media control commands (non-blocking)
        while let Ok(cmd) = rx.try_recv() {
          if let Ok(session) = manager.GetCurrentSession() {
            match cmd.as_str() {
              "play_pause" => { let _ = session.TryTogglePlayPauseAsync(); },
              "next" => { let _ = session.TrySkipNextAsync(); },
              "prev" => { let _ = session.TrySkipPreviousAsync(); },
              _ => {}
            }
          }
        }

        // Poll current session
        let status = read_session(&manager).unwrap_or(RealMediaStatus {
          has_session: false,
          title: String::new(),
          artist: String::new(),
          album: String::new(),
          is_playing: false,
          position_sec: 0.0,
          duration_sec: 0.0,
        });

        {
          let mut guard = MEDIA_STATE.lock().unwrap();
          let changed = guard.as_ref().map(|prev| {
            prev.has_session != status.has_session
              || prev.title != status.title
              || prev.artist != status.artist
              || prev.is_playing != status.is_playing
          }).unwrap_or(true);

          if changed {
            *guard = Some(status.clone());
            let _ = app_handle.emit("smtc-media-changed", status);
          } else {
            *guard = Some(status);
          }
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));
      }
    }
  });
}

// ---------------------------------------------------------------------------
// Real Clipboard via Win32 AddClipboardFormatListener
// ---------------------------------------------------------------------------
static CLIPBOARD_TEXT: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[tauri::command]
fn get_real_clipboard() -> Option<String> {
  CLIPBOARD_TEXT.lock().ok()?.clone()
}

fn start_clipboard_thread(app_handle: tauri::AppHandle) {
  std::thread::spawn(move || {
    #[cfg(target_os = "windows")]
    {
      use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT, HGLOBAL};
      use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
        RegisterClassExW, WNDCLASSEXW, WS_EX_NOACTIVATE, HWND_MESSAGE,
        MSG, WM_DESTROY,
      };
      use windows::Win32::System::DataExchange::{
        AddClipboardFormatListener, OpenClipboard, GetClipboardData,
        CloseClipboard,
      };
      use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
      use windows::core::{PCWSTR, w};

      const WM_CLIPBOARDUPDATE: u32 = 0x031D;
      const CF_UNICODETEXT: u32 = 13;

      static APP_HANDLE_PTR: std::sync::atomic::AtomicPtr<tauri::AppHandle> =
        std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

      let handle_box = Box::new(app_handle);
      APP_HANDLE_PTR.store(Box::into_raw(handle_box), std::sync::atomic::Ordering::SeqCst);

      unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        if msg == WM_CLIPBOARDUPDATE {
          unsafe {
            if OpenClipboard(hwnd).is_ok() {
              let h = GetClipboardData(CF_UNICODETEXT);
              if let Ok(h) = h {
                if !h.0.is_null() {
                  let ptr = GlobalLock(HGLOBAL(h.0)) as *const u16;
                  if !ptr.is_null() {
                    let mut len = 0usize;
                    while *ptr.add(len) != 0 { len += 1; }
                    let slice = std::slice::from_raw_parts(ptr, len);
                    let text = String::from_utf16_lossy(slice);
                    let _ = GlobalUnlock(HGLOBAL(h.0));

                    if !text.is_empty() {
                      if let Ok(mut guard) = CLIPBOARD_TEXT.lock() {
                        *guard = Some(text.clone());
                      }
                      let raw = APP_HANDLE_PTR.load(std::sync::atomic::Ordering::SeqCst);
                      if !raw.is_null() {
                        let _ = (*raw).emit("clipboard-changed", text);
                      }
                    }
                  }
                }
                let _ = CloseClipboard();
              } else {
                let _ = CloseClipboard();
              }
            }
          }
          return LRESULT(0);
        }
        if msg == WM_DESTROY {
          return LRESULT(0);
        }
        unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
      }

      let class_name: Vec<u16> = "IsleClipboardWnd\0".encode_utf16().collect();
      let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wnd_proc),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
      };
      let _ = unsafe { RegisterClassExW(&wc) };

      let hwnd = unsafe {
        CreateWindowExW(
          WS_EX_NOACTIVATE,
          PCWSTR(class_name.as_ptr()),
          w!(""),
          Default::default(),
          0, 0, 0, 0,
          HWND_MESSAGE,
          None,
          None,
          None,
        )
      };

      if hwnd.is_err() { return; }
      let hwnd = hwnd.unwrap();

      let _ = unsafe { AddClipboardFormatListener(hwnd) };

      let mut msg: MSG = unsafe { std::mem::zeroed() };
      loop {
        let ret = unsafe { GetMessageW(&mut msg, hwnd, 0, 0) };
        if ret.0 <= 0 { break; }
        unsafe { DispatchMessageW(&msg) };
      }
    }
  });
}

// ---------------------------------------------------------------------------
// Real Microphone Mute via Windows Core Audio IAudioEndpointVolume
// ---------------------------------------------------------------------------
#[tauri::command]
fn get_real_mic_mute() -> bool {
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::Media::Audio::{
      IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eCommunications,
    };
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    unsafe {
      let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
      let enumerator: IMMDeviceEnumerator = match CoCreateInstance(
        &MMDeviceEnumerator,
        None,
        CLSCTX_ALL,
      ) {
        Ok(e) => e,
        Err(_) => return false,
      };
      let device = match enumerator.GetDefaultAudioEndpoint(eCapture, eCommunications) {
        Ok(d) => d,
        Err(_) => return false,
      };
      let volume: IAudioEndpointVolume = match device.Activate(CLSCTX_ALL, None) {
        Ok(v) => v,
        Err(_) => return false,
      };
      return match volume.GetMute() {
        Ok(b) => b.as_bool(),
        Err(_) => false,
      };
    }
  }
  #[allow(unreachable_code)]
  false
}

#[tauri::command]
fn toggle_real_mic_mute() -> bool {
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::Media::Audio::{
      IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eCommunications,
    };
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    unsafe {
      let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
      let enumerator: IMMDeviceEnumerator = match CoCreateInstance(
        &MMDeviceEnumerator,
        None,
        CLSCTX_ALL,
      ) {
        Ok(e) => e,
        Err(_) => return false,
      };
      let device = match enumerator.GetDefaultAudioEndpoint(eCapture, eCommunications) {
        Ok(d) => d,
        Err(_) => return false,
      };
      let volume: IAudioEndpointVolume = match device.Activate(CLSCTX_ALL, None) {
        Ok(v) => v,
        Err(_) => return false,
      };
      let current = match volume.GetMute() {
        Ok(b) => b.as_bool(),
        Err(_) => return false,
      };
      let next = !current;
      let _ = volume.SetMute(next, std::ptr::null());
      return next;
    }
  }
  #[allow(unreachable_code)]
  false
}

// ---------------------------------------------------------------------------
// Window Sizing
// ---------------------------------------------------------------------------
#[tauri::command]
fn set_island_size(window: tauri::WebviewWindow, width: u32, height: u32) {
  if let Ok(Some(monitor)) = window.current_monitor() {
    let screen_w = monitor.size().width;
    let x = ((screen_w as i32) - (width as i32)) / 2;
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
      width,
      height,
    }));
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
      x,
      y: 0,
    }));
  }
}

// ---------------------------------------------------------------------------
// App Entry Point
// ---------------------------------------------------------------------------
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      set_island_size,
      sync_expand_trigger_tray,
      get_real_battery,
      get_real_hardware,
      get_real_media,
      media_control,
      get_real_clipboard,
      get_real_mic_mute,
      toggle_real_mic_mute,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        let _ = app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        );
      }

      // Setup system tray
      setup_tray(app)?;

      // Start background threads
      let handle = app.handle().clone();
      start_smtc_thread(handle.clone());
      start_clipboard_thread(handle.clone());

      if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "windows")]
        {
          use std::ffi::c_void;
          if let Ok(hwnd) = window.hwnd() {
            let hwnd = hwnd.0 as *mut c_void;
            unsafe extern "system" {
              fn LoadLibraryA(lpLibFileName: *const u8) -> *mut c_void;
              fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
            }
            type DwmSetWindowAttributeFn = unsafe extern "system" fn(*mut c_void, u32, *const c_void, u32) -> i32;
            unsafe {
              let lib = LoadLibraryA(b"dwmapi.dll\0".as_ptr());
              if !lib.is_null() {
                let proc = GetProcAddress(lib, b"DwmSetWindowAttribute\0".as_ptr());
                if !proc.is_null() {
                  let set_attr: DwmSetWindowAttributeFn = std::mem::transmute(proc);
                  // DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_DONOTROUND = 1
                  let corner_pref: u32 = 1;
                  set_attr(hwnd, 33, &corner_pref as *const _ as *const c_void, 4);
                  // DWMWA_BORDER_COLOR = 34, DWMWA_COLOR_NONE = 0xFFFFFFFE
                  let border_color: u32 = 0xFFFFFFFE;
                  set_attr(hwnd, 34, &border_color as *const _ as *const c_void, 4);
                }
              }
            }
          }
        }

        if let Ok(Some(monitor)) = window.current_monitor() {
          let screen_w = monitor.size().width;
          let window_w: u32 = 280;
          let window_h: u32 = 44;
          let x = ((screen_w as i32) - (window_w as i32)) / 2;
          let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: window_w,
            height: window_h,
          }));
          let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y: 0 }));
        }
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
