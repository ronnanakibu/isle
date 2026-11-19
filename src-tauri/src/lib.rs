use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        let _ = app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        );
      }

      if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.current_monitor() {
          let screen_w = monitor.size().width;
          let window_w = window.outer_size().map(|s| s.width).unwrap_or(780);
          let x = ((screen_w as i32) - (window_w as i32)) / 2;
          let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y: 0 }));
        }
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
