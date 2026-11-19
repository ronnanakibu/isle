//! Interactive activity states: Focus (Pomodoro), Hydration (Water),
//! Quick Reminders (Tasks), and Quick File Shelf (Files).

use std::path::PathBuf;

use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::render::Rgba;
use crate::spring::Spring;
use crate::theme;

// ---------------------------------------------------------------------------
// Focus / Pomodoro
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusPhase {
    Work,
    ShortBreak,
    LongBreak,
}

impl FocusPhase {
    pub fn duration_secs(self) -> u32 {
        match self {
            FocusPhase::Work => 25 * 60,
            FocusPhase::ShortBreak => 5 * 60,
            FocusPhase::LongBreak => 15 * 60,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FocusPhase::Work => "FOCUS",
            FocusPhase::ShortBreak => "SHORT BREAK",
            FocusPhase::LongBreak => "LONG BREAK",
        }
    }

    pub fn color(self) -> Rgba {
        match self {
            FocusPhase::Work => theme::ACCENT,
            FocusPhase::ShortBreak => theme::SUCCESS,
            FocusPhase::LongBreak => theme::WARNING,
        }
    }
}

pub struct FocusState {
    pub phase: FocusPhase,
    pub remaining_secs: u32,
    pub total_secs: u32,
    pub running: bool,
    pub completed_sessions: u32,
}

impl FocusState {
    pub fn new() -> Self {
        let total = FocusPhase::Work.duration_secs();
        Self {
            phase: FocusPhase::Work,
            remaining_secs: total,
            total_secs: total,
            running: false,
            completed_sessions: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
    }

    /// Reset back to a fresh 25:00 Work session.
    pub fn reset(&mut self) {
        self.phase = FocusPhase::Work;
        self.total_secs = FocusPhase::Work.duration_secs();
        self.remaining_secs = self.total_secs;
        self.running = false;
    }

    /// Advance to the next phase in the Pomodoro cycle and pause.
    pub fn advance_phase(&mut self) {
        self.phase = match self.phase {
            FocusPhase::Work => {
                self.completed_sessions += 1;
                if self.completed_sessions % 4 == 0 {
                    FocusPhase::LongBreak
                } else {
                    FocusPhase::ShortBreak
                }
            }
            FocusPhase::ShortBreak | FocusPhase::LongBreak => FocusPhase::Work,
        };
        self.total_secs = self.phase.duration_secs();
        self.remaining_secs = self.total_secs;
        self.running = false;
    }

    /// Advance the countdown by 1 second. Returns `true` if the phase completed.
    pub fn tick_one_second(&mut self) -> bool {
        if !self.running {
            return false;
        }
        if self.remaining_secs > 1 {
            self.remaining_secs -= 1;
            false
        } else {
            self.advance_phase();
            true
        }
    }

    pub fn progress(&self) -> f32 {
        if self.total_secs == 0 {
            return 0.0;
        }
        (1.0 - self.remaining_secs as f32 / self.total_secs as f32).clamp(0.0, 1.0)
    }

    pub fn formatted_time(&self) -> String {
        let m = self.remaining_secs / 60;
        let s = self.remaining_secs % 60;
        format!("{:02}:{:02}", m, s)
    }
}

// ---------------------------------------------------------------------------
// Hydration (Water Tracker)
// ---------------------------------------------------------------------------

pub struct WaterState {
    pub date: String,
    pub intake_ml: u32,
    pub goal_ml: u32,
    pub history: Vec<u32>,
    pub bounce: Spring,
}

impl WaterState {
    pub fn load() -> Self {
        let today = today_date();
        let mut state = Self {
            date: today.clone(),
            intake_ml: 0,
            goal_ml: 2000,
            history: Vec::new(),
            bounce: Spring::new(0.0, theme::motion::MICRO_OMEGA, theme::motion::MICRO_ZETA),
        };
        if let Some(path) = app_data_path("water.json") {
            if let Ok(raw) = std::fs::read_to_string(path) {
                state.apply_json(&raw, &today);
            }
        }
        state
    }

    fn apply_json(&mut self, raw: &str, today: &str) {
        let saved_date = extract_json_str(raw, "date").unwrap_or_default();
        if let Some(goal) = extract_json_u32(raw, "goal_ml") {
            if goal >= 250 {
                self.goal_ml = goal;
            }
        }
        if saved_date == today {
            if let Some(intake) = extract_json_u32(raw, "intake_ml") {
                self.intake_ml = intake;
            }
        } else {
            self.date = today.to_string();
            self.intake_ml = 0;
            self.history.clear();
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"date\":\"{}\",\"intake_ml\":{},\"goal_ml\":{}}}",
            escape_json(&self.date),
            self.intake_ml,
            self.goal_ml
        )
    }

    pub fn save(&self) {
        if let Some(path) = app_data_path("water.json") {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, self.to_json());
        }
    }

    pub fn add(&mut self, ml: u32) {
        let today = today_date();
        if self.date != today {
            self.date = today;
            self.intake_ml = 0;
            self.history.clear();
        }
        self.intake_ml = self.intake_ml.saturating_add(ml);
        self.history.push(ml);
        self.bounce.value = 0.0;
        self.bounce.target = 0.0;
        self.bounce.velocity = 28.0;
        self.save();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(last) = self.history.pop() {
            self.intake_ml = self.intake_ml.saturating_sub(last);
            self.save();
            true
        } else if self.intake_ml > 0 {
            self.intake_ml = self.intake_ml.saturating_sub(250);
            self.save();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.intake_ml = 0;
        self.history.clear();
        self.save();
    }

    pub fn progress(&self) -> f32 {
        if self.goal_ml == 0 {
            return 0.0;
        }
        (self.intake_ml as f32 / self.goal_ml as f32).clamp(0.0, 1.0)
    }

    pub fn formatted_total(&self) -> String {
        format!("{} / {} ml", format_commas(self.intake_ml), format_commas(self.goal_ml))
    }
}

// ---------------------------------------------------------------------------
// Quick Reminders / Tasks
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TaskItem {
    pub title: String,
    pub completed: bool,
    pub color: Rgba,
}

pub struct TasksState {
    pub items: Vec<TaskItem>,
}

impl TasksState {
    pub fn default_items() -> Vec<TaskItem> {
        vec![
            TaskItem {
                title: "Finish presentation".to_string(),
                completed: false,
                color: theme::ACCENT,
            },
            TaskItem {
                title: "Review PR".to_string(),
                completed: true,
                color: theme::SUCCESS,
            },
            TaskItem {
                title: "Hydrate & stretch".to_string(),
                completed: false,
                color: theme::WARNING,
            },
            TaskItem {
                title: "Ship Isle release".to_string(),
                completed: false,
                color: Rgba::rgb(192, 132, 252),
            },
        ]
    }

    pub fn load() -> Self {
        if let Some(path) = app_data_path("tasks.json") {
            if let Ok(raw) = std::fs::read_to_string(path) {
                let parsed = parse_tasks_json(&raw);
                if !parsed.is_empty() {
                    return Self { items: parsed };
                }
            }
        }
        Self { items: Self::default_items() }
    }

    pub fn save(&self) {
        if let Some(path) = app_data_path("tasks.json") {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, serialize_tasks_json(&self.items));
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.completed = !item.completed;
            self.save();
        }
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|t| t.completed).count()
    }
}

// ---------------------------------------------------------------------------
// Quick File Shelf
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ShelfFile {
    pub name: String,
    pub path: String,
}

pub struct FilesState {
    pub items: Vec<ShelfFile>,
}

impl FilesState {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_path(&mut self, path: String) {
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();
        self.items.retain(|f| f.path != path);
        self.items.insert(0, ShelfFile { name, path });
        self.items.truncate(4);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn today_date() -> String {
    let st = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay)
}

fn app_data_path(filename: &str) -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(base).join("Isle").join(filename))
}

pub fn format_commas(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut chars = rest[1..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}

fn extract_json_u32(json: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\":");
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\":");
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn serialize_tasks_json(items: &[TaskItem]) -> String {
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"title\":\"{}\",\"completed\":{},\"r\":{},\"g\":{},\"b\":{}}}",
            escape_json(&item.title),
            item.completed,
            item.color.r,
            item.color.g,
            item.color.b
        ));
    }
    out.push(']');
    out
}

fn parse_tasks_json(raw: &str) -> Vec<TaskItem> {
    let mut out = Vec::new();
    let mut rest = raw;
    while let Some(obj_start) = rest.find('{') {
        let after = &rest[obj_start..];
        let Some(obj_end) = after.find('}') else {
            break;
        };
        let obj = &after[..=obj_end];
        if let Some(title) = extract_json_str(obj, "title") {
            let completed = extract_json_bool(obj, "completed").unwrap_or(false);
            let r = extract_json_u32(obj, "r").unwrap_or(94).min(255) as u8;
            let g = extract_json_u32(obj, "g").unwrap_or(200).min(255) as u8;
            let b = extract_json_u32(obj, "b").unwrap_or(255).min(255) as u8;
            out.push(TaskItem {
                title,
                completed,
                color: Rgba::rgb(r, g, b),
            });
        }
        rest = &after[obj_end + 1..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_cycle_transitions_correctly() {
        let mut f = FocusState::new();
        assert_eq!(f.phase, FocusPhase::Work);
        assert_eq!(f.formatted_time(), "25:00");
        f.running = true;
        f.remaining_secs = 1;
        assert!(f.tick_one_second());
        assert_eq!(f.phase, FocusPhase::ShortBreak);
        assert_eq!(f.formatted_time(), "05:00");
        assert!(!f.running);

        // 4th completed work session triggers LongBreak.
        f.completed_sessions = 3;
        f.phase = FocusPhase::Work;
        f.advance_phase();
        assert_eq!(f.phase, FocusPhase::LongBreak);
        assert_eq!(f.formatted_time(), "15:00");

        f.reset();
        assert_eq!(f.phase, FocusPhase::Work);
        assert_eq!(f.formatted_time(), "25:00");
    }

    #[test]
    fn water_json_roundtrip_and_rollover() {
        let mut w = WaterState {
            date: "2026-11-18".to_string(),
            intake_ml: 1250,
            goal_ml: 2000,
            history: vec![250],
            bounce: Spring::new(0.0, 26.0, 0.85),
        };
        assert_eq!(w.formatted_total(), "1,250 / 2,000 ml");
        let json = w.to_json();

        let mut same_day = WaterState {
            date: "2026-11-18".to_string(),
            intake_ml: 0,
            goal_ml: 2000,
            history: Vec::new(),
            bounce: Spring::new(0.0, 26.0, 0.85),
        };
        same_day.apply_json(&json, "2026-11-18");
        assert_eq!(same_day.intake_ml, 1250);

        w.apply_json(&json, "2026-11-19");
        assert_eq!(w.intake_ml, 0);
        assert_eq!(w.date, "2026-11-19");
    }

    #[test]
    fn tasks_json_roundtrip() {
        let items = TasksState::default_items();
        let json = serialize_tasks_json(&items);
        let parsed = parse_tasks_json(&json);
        assert_eq!(parsed.len(), items.len());
        assert_eq!(parsed[0].title, "Finish presentation");
        assert!(!parsed[0].completed);
        assert!(parsed[1].completed);
    }
}
