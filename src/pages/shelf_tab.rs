//! Quick Shelf & Real Clipboard History Tab

#[derive(Debug, Clone)]
pub struct ShelfFileItem {
    pub name: String,
    pub size_str: String,
}

#[derive(Debug, Clone)]
pub struct ClipboardSnippet {
    pub text: String,
    pub timestamp_sec: u64,
}

#[derive(Debug, Clone)]
pub struct ShelfTabState {
    pub files: Vec<ShelfFileItem>,
    pub clipboard_history: Vec<ClipboardSnippet>,
}

impl Default for ShelfTabState {
    fn default() -> Self {
        Self {
            files: vec![
                ShelfFileItem {
                    name: "Design_Review.fig".to_string(),
                    size_str: "14.2 MB".to_string(),
                },
                ShelfFileItem {
                    name: "Prototype_v2.mov".to_string(),
                    size_str: "48.6 MB".to_string(),
                },
            ],
            clipboard_history: Vec::new(),
        }
    }
}

impl ShelfTabState {
    pub fn add_file(&mut self, name: String, size_str: String) {
        self.files.insert(0, ShelfFileItem { name, size_str });
        if self.files.len() > 12 {
            self.files.truncate(12);
        }
    }

    pub fn remove_file(&mut self, idx: usize) {
        if idx < self.files.len() {
            self.files.remove(idx);
        }
    }

    pub fn push_clipboard(&mut self, text: String) {
        if text.trim().is_empty() {
            return;
        }
        // Deduplicate recent
        if let Some(first) = self.clipboard_history.first() {
            if first.text == text {
                return;
            }
        }
        self.clipboard_history.insert(
            0,
            ClipboardSnippet {
                text,
                timestamp_sec: 0,
            },
        );
        if self.clipboard_history.len() > 25 {
            self.clipboard_history.truncate(25);
        }
    }
}
