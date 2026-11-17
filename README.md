# 🏝️ Isle — Living Dynamic Island for Windows

A native, ultra-lightweight (**~350 KB**), zero-bloat Apple Dynamic Island for Windows built in pure **Rust**. Features 60 FPS harmonic oscillator spring physics, continuous squircle blending, specular rim lighting, interactive Live Activities, and native Windows Media Key integration.

---

## ✨ Features

- **Apple Dynamic Island Aesthetics**: Continuous squircle curvature, OLED true black (`#000000`), and a specular 1px white glass rim (`rgba(255, 255, 255, 0.14)`).
- **Harmonic Spring Physics**: Damped harmonic oscillator ($\omega = 16.0\text{ rad/s}$, $\zeta = 0.72$ damping ratio) providing Apple's signature fluid rubber-band bounce.
- **Zero-Idle Efficiency**: Consumes **0.0% CPU** and only **~12 MB RAM** when static. Timers activate strictly during dynamic transitions.
- **Click-Through Transparency**: Non-visible canvas pixels (`alpha = 0`) pass mouse clicks directly to underlying Windows apps.
- **Living Notch Mascot**: Interactive animated companion with procedural pupil gaze tracking that follows the mouse cursor and blinks naturally.
- **5 Live Activity Pages**:
  - 🎵 **Music Player**: High-resolution album art with vinyl grooves, 5-band dynamic audio equalizer, scrubber bar, and **real Windows Media Key integration** (`VK_MEDIA_PLAY_PAUSE`, `VK_MEDIA_NEXT_TRACK`, `VK_MEDIA_PREV_TRACK`).
  - ⏱️ **Pomodoro Focus Timer**: Circular progress countdown ring, interval mode selector pills (`25m Focus`, `5m Break`, `15m Rest`), live second-by-second ticking, and Start/Pause/Reset buttons.
  - 💧 **Hydration Tracker**: Visual tumbler with animated liquid fill level, percentage counter, and one-click quick-log buttons (`+250 ml`, `+500 ml`, `Reset`).
  - 📝 **Reminders Checklist**: Apple Reminders style checklist with interactive circular checkboxes that toggle task states on click.
  - 📁 **File Shelf**: AirDrop style file cards with quick-open shortcuts to pinned files and downloads.

---

## 📐 Dimensions & States

| State | Width | Height | Corner Radius | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Compact Pill** | `205 pt` | `36 pt` | `18 pt` | Unobtrusive top-docked pill with mini art and equalizer |
| **Expanded Card**| `370 pt` | `160 pt`| `42 pt` | Full interactive Live Activity card with bottom page dots |
| **Canvas** | `740 px` | `220 px`| N/A | Transparent 32-bit premultiplied ARGB DIB Section |

---

## 🛠️ Architecture

```
isle/
├── src/
│   ├── main.rs       # Single-instance guard (Local\Isle.SingleInstance mutex)
│   ├── theme.rs      # Apple design tokens, colors & dimensions
│   ├── spring.rs     # Damped harmonic oscillator physics & smoothstep
│   ├── mascot.rs     # Procedural living notch mascot engine
│   ├── render.rs     # Anti-aliased squircle rasterizer & crisp GDI typography
│   ├── window.rs     # Win32 Layered Window (ULW_ALPHA), hit testing & media controls
│   └── pages/
│       └── mod.rs    # Live Activity models for all 5 pages
├── changelogs/
│   └── v1.0.0.md     # Release notes
├── Cargo.toml
└── .gitignore
```

---

## 🚀 Building & Running

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (edition 2021)
- Windows 10 / 11

### Run Development Build
```bash
cargo run
```

### Build Optimized Release Binary
```bash
cargo build --release
```
The compiled standalone executable will be located at `target/release/isle.exe` (**~350 KB**).

---

## 📄 License
MIT License.
