use tauri::Manager;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![set_island_size])
    .setup(|app| {
      if cfg!(debug_assertions) {
        let _ = app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        );
      }

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
