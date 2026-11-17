//! Page Modules for Apple Dynamic Island Live Activities

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Music = 0,
    Pomodoro = 1,
    Water = 2,
    Todo = 3,
    Shelf = 4,
}

impl PageId {
    pub const ALL: [PageId; 5] = [
        PageId::Music,
        PageId::Pomodoro,
        PageId::Water,
        PageId::Todo,
        PageId::Shelf,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            PageId::Music => "Now Playing",
            PageId::Pomodoro => "Pomodoro Timer",
            PageId::Water => "Hydration",
            PageId::Todo => "Reminders",
            PageId::Shelf => "Quick Shelf",
        }
    }

    pub fn from_index(i: usize) -> PageId {
        match i % 5 {
            0 => PageId::Music,
            1 => PageId::Pomodoro,
            2 => PageId::Water,
            3 => PageId::Todo,
            _ => PageId::Shelf,
        }
    }

    pub fn next(&self) -> PageId {
        match self {
            PageId::Music => PageId::Pomodoro,
            PageId::Pomodoro => PageId::Water,
            PageId::Water => PageId::Todo,
            PageId::Todo => PageId::Shelf,
            PageId::Shelf => PageId::Music,
        }
    }

    pub fn prev(&self) -> PageId {
        match self {
            PageId::Music => PageId::Shelf,
            PageId::Pomodoro => PageId::Music,
            PageId::Water => PageId::Pomodoro,
            PageId::Todo => PageId::Water,
            PageId::Shelf => PageId::Todo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    None,
    PageDot(usize),
    // Music
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
}

pub struct PageState {
    pub current_page: PageId,
    // Music
    pub is_playing: bool,
    pub track_title: String,
    pub track_artist: String,
    pub current_lyric: String,
    pub track_progress: f32, // 0.0 to 1.0
    pub eq_bars: [f32; 5],
    // Pomodoro
    pub pomo_seconds_left: u32,
    pub pomo_running: bool,
    pub pomo_mode: usize, // 0=25m, 1=5m, 2=15m
    // Water
    pub water_taken_ml: u32,
    pub water_goal_ml: u32,
    // Tasks
    pub tasks: Vec<(String, bool)>,
    // Shelf
    pub shelf_items: Vec<(String, String)>, // (name, size)
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            current_page: PageId::Music,
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
                ("Refine dynamic island curves".to_string(), true),
                ("Apple spring physics damping".to_string(), true),
                ("Live WinRT SMTC audio integration".to_string(), true),
            ],
            shelf_items: vec![
                ("Design_Review.fig".to_string(), "14.2 MB".to_string()),
                ("Prototype_v2.mov".to_string(), "48.6 MB".to_string()),
            ],
        }
    }
}

impl PageState {
    pub fn toggle_pomo(&mut self) {
        self.pomo_running = !self.pomo_running;
    }

    pub fn set_pomo_mode(&mut self, mode: usize) {
        self.pomo_mode = mode;
        self.pomo_seconds_left = match mode {
            0 => 25 * 60,
            1 => 5 * 60,
            2 => 15 * 60,
            _ => 25 * 60,
        };
        self.pomo_running = false;
    }

    pub fn reset_pomo(&mut self) {
        self.set_pomo_mode(self.pomo_mode);
    }

    pub fn add_water(&mut self, ml: u32) {
        self.water_taken_ml = (self.water_taken_ml + ml).min(self.water_goal_ml * 2);
    }

    pub fn reset_water(&mut self) {
        self.water_taken_ml = 0;
    }

    pub fn toggle_task(&mut self, idx: usize) {
        if idx < self.tasks.len() {
            self.tasks[idx].1 = !self.tasks[idx].1;
        }
    }
}
