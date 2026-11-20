//! Media & LRCLIB Synced Lyrics Tab
use crate::media::MediaSnapshot;

#[derive(Debug, Clone)]
pub struct MediaTabState {
    pub is_playing: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub progress: f32, // 0.0 to 1.0
    pub current_lyric: String,
    pub eq_bars: [f32; 5],
}

impl Default for MediaTabState {
    fn default() -> Self {
        Self {
            is_playing: false,
            title: "No Media Playing".to_string(),
            artist: "Open Spotify, YouTube, or Apple Music".to_string(),
            album: String::new(),
            progress: 0.0,
            current_lyric: "Live activity ready".to_string(),
            eq_bars: [0.0; 5],
        }
    }
}

impl MediaTabState {
    pub fn sync_from_smtc(&mut self, snap: &MediaSnapshot) {
        if snap.has_session {
            self.is_playing = snap.playing;
            self.title = snap.title.clone();
            self.artist = snap.artist.clone();
            self.album = snap.album.clone();
            let pos = snap.position_ms_now();
            self.progress = if snap.duration_ms > 0 {
                (pos as f32 / snap.duration_ms as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if let Some(lyric) = snap.current_lyric() {
                self.current_lyric = lyric.to_string();
            } else if !snap.title.is_empty() {
                self.current_lyric = format!("{} · {}", snap.title, snap.artist);
            }
        } else {
            self.is_playing = false;
            self.title = "No Media Playing".to_string();
            self.artist = "Serene Idle".to_string();
            self.current_lyric = "Waiting for media playback...".to_string();
            self.progress = 0.0;
        }
    }
}
