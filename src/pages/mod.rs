//! Page Modules for Apple Dynamic Island Live Activities — Consolidated 7-Tab Suite

pub mod media_tab;
pub mod pomodoro_tab;
pub mod hydration_tab;
pub mod tasks_tab;
pub mod shelf_tab;
pub mod hardware_tab;
pub mod zen_tab;

pub use media_tab::MediaTabState;
pub use pomodoro_tab::PomodoroTabState;
pub use hydration_tab::HydrationTabState;
pub use tasks_tab::{TasksTabState, TaskItem};
pub use shelf_tab::{ShelfTabState, ShelfFileItem, ClipboardSnippet};
pub use hardware_tab::HardwareTabState;
pub use zen_tab::{ZenTabState, BreathPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Media = 0,
    Pomodoro = 1,
    Hydration = 2,
    Tasks = 3,
    Shelf = 4,
    Hardware = 5,
    Zen = 6,
}

impl PageId {
    // Aliases for compatibility
    pub const Music: PageId = PageId::Media;
    pub const Water: PageId = PageId::Hydration;
    pub const Todo: PageId = PageId::Tasks;

    pub const ALL: [PageId; 7] = [
        PageId::Media,
        PageId::Pomodoro,
        PageId::Hydration,
        PageId::Tasks,
        PageId::Shelf,
        PageId::Hardware,
        PageId::Zen,
    ];

    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            PageId::Media => "Media & Lyrics",
            PageId::Pomodoro => "Pomodoro Timer",
            PageId::Hydration => "Hydration",
            PageId::Tasks => "Tasks & Reminders",
            PageId::Shelf => "Quick Shelf & Clipboard",
            PageId::Hardware => "Hardware Telemetry",
            PageId::Zen => "Zen Breathing",
        }
    }

    pub fn from_index(i: usize) -> PageId {
        match i % 7 {
            0 => PageId::Media,
            1 => PageId::Pomodoro,
            2 => PageId::Hydration,
            3 => PageId::Tasks,
            4 => PageId::Shelf,
            5 => PageId::Hardware,
            _ => PageId::Zen,
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> PageId {
        let idx = (*self as usize + 1) % 7;
        Self::from_index(idx)
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> PageId {
        let idx = (*self as usize + 6) % 7;
        Self::from_index(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    None,
    PageDot(usize),
    // Media
    MediaPrev,
    MediaPlayPause,
    MediaNext,
    Scrubber,
    // Pomodoro
    PomoMode(usize),
    PomoStart,
    PomoReset,
    // Water
    WaterAdd250,
    WaterAdd500,
    WaterReset,
    // Tasks
    TaskItem(usize),
    // Shelf
    ShelfOpen(usize),
    // Hardware
    HardwarePlan(usize),
    // Zen
    ZenToggle,
}

pub struct PageState {
    pub current_page: PageId,
    // Legacy fields for backward compatibility with existing renderers
    pub is_playing: bool,
    pub track_title: String,
    pub track_artist: String,
    pub current_lyric: String,
    pub track_progress: f32, // 0.0 to 1.0
    pub eq_bars: [f32; 5],
    pub pomo_seconds_left: u32,
    pub pomo_running: bool,
    pub pomo_mode: usize,
    pub water_taken_ml: u32,
    pub water_goal_ml: u32,
    pub tasks: Vec<(String, bool)>,
    pub shelf_items: Vec<(String, String)>,

    // Specialized tab states
    pub media_tab: MediaTabState,
    pub pomo_tab: PomodoroTabState,
    pub water_tab: HydrationTabState,
    pub tasks_tab: TasksTabState,
    pub shelf_tab: ShelfTabState,
    pub hw_tab: HardwareTabState,
    pub zen_tab: ZenTabState,
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            current_page: PageId::Media,
            is_playing: false,
            track_title: "Nothing Playing".to_string(),
            track_artist: "Open Spotify, YouTube, or Apple Music".to_string(),
            current_lyric: "Live activity ready".to_string(),
            track_progress: 0.0,
            eq_bars: [0.0; 5],
            pomo_seconds_left: 25 * 60,
            pomo_running: false,
            pomo_mode: 0,
            water_taken_ml: 850,
            water_goal_ml: 1500,
            tasks: vec![
                ("Refine Dynamic Island squircle curves".to_string(), true),
                ("Apple spring physics damping".to_string(), true),
                ("Live WinRT SMTC audio & LRCLIB integration".to_string(), true),
            ],
            shelf_items: vec![
                ("Design_Review.fig".to_string(), "14.2 MB".to_string()),
                ("Prototype_v2.mov".to_string(), "48.6 MB".to_string()),
            ],
            media_tab: MediaTabState::default(),
            pomo_tab: PomodoroTabState::default(),
            water_tab: HydrationTabState::default(),
            tasks_tab: TasksTabState::default(),
            shelf_tab: ShelfTabState::default(),
            hw_tab: HardwareTabState::default(),
            zen_tab: ZenTabState::default(),
        }
    }
}

impl PageState {
    pub fn toggle_pomo(&mut self) {
        self.pomo_running = !self.pomo_running;
        self.pomo_tab.toggle();
    }

    pub fn set_pomo_mode(&mut self, mode: usize) {
        self.pomo_mode = mode;
        self.pomo_tab.set_mode(mode);
        self.pomo_seconds_left = self.pomo_tab.seconds_left;
        self.pomo_running = false;
    }

    pub fn reset_pomo(&mut self) {
        self.set_pomo_mode(self.pomo_mode);
    }

    pub fn add_water(&mut self, ml: u32) {
        self.water_taken_ml = (self.water_taken_ml + ml).min(self.water_goal_ml * 2);
        self.water_tab.add(ml);
    }

    pub fn reset_water(&mut self) {
        self.water_taken_ml = 0;
        self.water_tab.reset();
    }

    pub fn toggle_task(&mut self, idx: usize) {
        if idx < self.tasks.len() {
            self.tasks[idx].1 = !self.tasks[idx].1;
        }
        self.tasks_tab.toggle(idx);
    }
}
