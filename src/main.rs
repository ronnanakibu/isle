//! Isle — Living Dynamic Island for Windows
//! Clean, modular native architecture matching Nusa design

mod theme;
mod spring;
mod mascot;
mod pages;
mod render;
mod window;
mod settings;
mod tray;
mod media;
mod gesture;

use windows::core::w;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::Threading::CreateMutexW;

fn main() -> windows::core::Result<()> {
    // Single instance mutex
    unsafe {
        let _mutex = CreateMutexW(None, true, w!("Local\\Isle.SingleInstance"));
        if GetLastError().0 == 183 {
            // ERROR_ALREADY_EXISTS (183)
            println!("Isle is already running! (If you need to restart, run: taskkill /F /IM isle.exe)");
            return Ok(());
        }

        println!("Starting Isle Dynamic Island at the top of your screen...");
        println!("- Intro: Springs down smoothly into the compact pill.");
        println!("- Hover: Move your mouse to the top center to expand.");
        println!("- Pages: Click the bottom dots to switch between Music, Pomodoro, Water, Reminders, and Shelf.");
        window::run_window()?;
    }

    Ok(())
}
