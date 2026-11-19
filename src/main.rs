//! Isle — a native Dynamic Island / Living Notch for Windows.
//!
//! The program is a single layered, click-through overlay window that draws
//! itself with a software rasterizer and animates with a spring solver. It
//! contains no UI framework, no browser runtime, and no background thread
//! that isn't asleep.

mod activities;
mod lyrics;
mod mascot;
mod media;
mod pager;
mod render;
mod spring;
mod text;
mod theme;
mod ui;
mod window;

fn main() {
    if let Err(e) = window::run() {
        eprintln!("isle: {e}");
        std::process::exit(1);
    }
}
