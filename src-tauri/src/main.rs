// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  std::panic::set_hook(Box::new(|info| {
    let msg = format!("Isle Panic: {:?}", info);
    let _ = std::fs::write("D:/CE/isle_runtime.log", msg);
  }));
  app_lib::run();
}
