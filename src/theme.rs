//! Apple Dynamic Island Design Tokens & Constants
//! Recreated from Apple Human Interface Guidelines & iOS 17/18 Live Activity specifications

pub const CANVAS_WIDTH: i32 = 740;
pub const CANVAS_HEIGHT: i32 = 220;

// 1. Idle Pill State (Apple standard: ~125 x 36 pt, r=18)
pub const IDLE_WIDTH: f32 = 125.0;
pub const IDLE_HEIGHT: f32 = 36.0;
pub const IDLE_RADIUS: f32 = 18.0;

// 2. Compact Playing State (Apple standard: ~205 x 36 pt, r=18)
pub const COMPACT_WIDTH: f32 = 205.0;
pub const COMPACT_HEIGHT: f32 = 36.0;
pub const COMPACT_RADIUS: f32 = 18.0;

// 3. Expanded Card State (Apple standard: ~370 x 160 pt, r=42)
pub const EXPANDED_WIDTH: f32 = 370.0;
pub const EXPANDED_HEIGHT: f32 = 160.0;
pub const EXPANDED_RADIUS: f32 = 42.0;

// Apple Color Palette (premultiplied 0xAARRGGBB)
pub const COLOR_TRANSPARENT: u32 = 0x00000000;
pub const COLOR_ISLAND_BLACK: u32 = 0xFF000000; // Deep OLED true black
pub const COLOR_ISLAND_SURFACE: u32 = 0xFA0C0D10; // Dark glass surface (#0C0D10, 98% opacity)
pub const COLOR_ISLAND_BORDER: u32 = 0x24FFFFFF; // Apple 1px specular border highlight (14% opacity)
pub const COLOR_ISLAND_SHADOW: u32 = 0x40000000; // Ambient shadow

// Typography Colors
pub const COLOR_TEXT_WHITE: u32 = 0xFFFFFFFF; // Pure white title
pub const COLOR_TEXT_SECONDARY: u32 = 0xFFA1A1AA; // iOS secondary label (#A1A1AA)
pub const COLOR_TEXT_TERTIARY: u32 = 0xFF71717A; // iOS tertiary label (#71717A)

// Apple Accent Colors
pub const APPLE_CYAN: u32 = 0xFF38BDF8; // Electric cyan
pub const APPLE_GREEN: u32 = 0xFF34C759; // iOS System Green
pub const APPLE_ORANGE: u32 = 0xFFFF9500; // iOS System Orange (Music visualizer / Timer)
pub const APPLE_RED: u32 = 0xFFFF3B30; // iOS System Red
pub const APPLE_PURPLE: u32 = 0xFFAF52DE; // iOS System Purple
pub const APPLE_BLUE: u32 = 0xFF007AFF; // iOS System Blue

// Interactive Elements
pub const COLOR_TRACK_BG: u32 = 0x33FFFFFF; // 20% white scrubber track
pub const COLOR_CONTROL_BG: u32 = 0x1EFFFFFF; // Circle button glass
pub const COLOR_CONTROL_HOVER: u32 = 0x3AFFFFFF;
