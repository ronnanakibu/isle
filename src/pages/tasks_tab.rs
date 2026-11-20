//! Tasks & Reminders Tab

#[derive(Debug, Clone)]
pub struct TaskItem {
    pub id: usize,
    pub title: String,
    pub completed: bool,
    pub color_tag: u32, // ARGB accent dot
}

#[derive(Debug, Clone)]
pub struct TasksTabState {
    pub items: Vec<TaskItem>,
    pub next_id: usize,
}

impl Default for TasksTabState {
    fn default() -> Self {
        Self {
            items: vec![
                TaskItem {
                    id: 1,
                    title: "Refine Dynamic Island squircle curves".to_string(),
                    completed: true,
                    color_tag: 0xFFC084FC,
                },
                TaskItem {
                    id: 2,
                    title: "Live WinRT SMTC audio & LRCLIB lyrics".to_string(),
                    completed: true,
                    color_tag: 0xFF38BDF8,
                },
                TaskItem {
                    id: 3,
                    title: "ThinkPad F4 mic mute tape cross ( · 🗙 · )".to_string(),
                    completed: false,
                    color_tag: 0xFFFBBF24,
                },
            ],
            next_id: 4,
        }
    }
}

impl TasksTabState {
    pub fn toggle(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.items[idx].completed = !self.items[idx].completed;
        }
    }

    pub fn add(&mut self, title: String, color_tag: u32) {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(TaskItem {
            id,
            title,
            completed: false,
            color_tag,
        });
    }

    pub fn clear_completed(&mut self) {
        self.items.retain(|t| !t.completed);
    }
}
