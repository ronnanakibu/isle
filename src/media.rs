//! SMTC Bridge: WinRT Media Control, Live Album Artwork, & LRCLIB Synced Lyrics
//! Runs on a background worker thread. UI thread receives WM_MEDIA_CHANGED.
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use windows::{
    core::{Interface, Result, HSTRING},
    Foundation::{EventRegistrationToken, TypedEventHandler, Uri},
    Graphics::Imaging::{
        BitmapAlphaMode, BitmapDecoder, BitmapInterpolationMode, BitmapPixelFormat,
        BitmapTransform, ColorManagementMode, ExifOrientationMode,
    },
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionMediaProperties as Props,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as PStatus,
    },
    Storage::Streams::IRandomAccessStream,
    Web::Http::HttpClient,
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
        UI::WindowsAndMessaging::{PostMessageW, WM_APP},
    },
};

pub const WM_MEDIA_CHANGED: u32 = WM_APP + 2;
const ART_PX: u32 = 64;

pub struct ArtBitmap {
    pub size: u32,
    pub bgra_premul: Vec<u8>, // size * size * 4
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LyricLine {
    pub time_ms: u64,
    pub text: String,
}

#[derive(Clone, Default)]
pub struct MediaSnapshot {
    pub has_session: bool,
    pub title: String,
    pub artist: String,
    #[allow(dead_code)]
    pub album: String,
    pub source: String,
    pub playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub fetched_at: Option<Instant>,
    pub art: Option<Arc<ArtBitmap>>,
    pub lyrics: Option<Arc<Vec<LyricLine>>>,
}

impl MediaSnapshot {
    pub fn position_ms_now(&self) -> u64 {
        if !self.playing {
            return self.position_ms;
        }
        let el = self.fetched_at.map_or(0, |t| t.elapsed().as_millis() as u64);
        let p = self.position_ms + el;
        if self.duration_ms > 0 {
            p.min(self.duration_ms)
        } else {
            p
        }
    }

    /// Return the active lyric line at the current playback position
    pub fn current_lyric(&self) -> Option<&str> {
        let lines = self.lyrics.as_ref()?;
        let pos = self.position_ms_now();
        let mut candidate = None;
        for line in lines.iter() {
            if line.time_ms <= pos {
                candidate = Some(line.text.as_str());
            } else {
                break;
            }
        }
        candidate
    }
}

#[derive(Clone, Copy)]
pub enum MediaCmd {
    PlayPause,
    Next,
    Prev,
}

enum Msg {
    Refresh,
    Cmd(MediaCmd),
    LyricsReady {
        key: String,
        lyrics: Arc<Vec<LyricLine>>,
    },
}

pub struct MediaHook {
    state: Arc<Mutex<MediaSnapshot>>,
    tx: mpsc::Sender<Msg>,
}

impl MediaHook {
    pub fn start(hwnd: HWND) -> Self {
        let state = Arc::new(Mutex::new(MediaSnapshot::default()));
        let (tx, rx) = mpsc::channel();
        let (st, tx2, raw) = (state.clone(), tx.clone(), hwnd.0 as isize);
        let _ = std::thread::Builder::new()
            .name("isle-smtc".into())
            .spawn(move || {
                let _ = worker(raw, st, tx2, rx);
            });
        Self { state, tx }
    }

    pub fn snapshot(&self) -> MediaSnapshot {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn send(&self, c: MediaCmd) {
        let _ = self.tx.send(Msg::Cmd(c));
    }
}

fn exec(mgr: &Manager, m: &Msg) {
    if let Msg::Cmd(c) = m {
        if let Ok(s) = mgr.GetCurrentSession() {
            let _ = match c {
                MediaCmd::PlayPause => s.TryTogglePlayPauseAsync(),
                MediaCmd::Next => s.TrySkipNextAsync(),
                MediaCmd::Prev => s.TrySkipPreviousAsync(),
            };
        }
    }
}

fn worker(
    raw: isize,
    state: Arc<Mutex<MediaSnapshot>>,
    tx: mpsc::Sender<Msg>,
    rx: mpsc::Receiver<Msg>,
) -> Result<()> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }
    let mgr = Manager::RequestAsync()?.get()?;

    let t = tx.clone();
    mgr.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
        let _ = t.send(Msg::Refresh);
        Ok(())
    }))?;

    let mut sub: Option<(Session, EventRegistrationToken, EventRegistrationToken)> = None;
    let mut sub_id = String::new();
    let mut art_key = String::new();
    let mut lyrics_key = String::new();
    let mut lyrics_cache: HashMap<String, Arc<Vec<LyricLine>>> = HashMap::new();

    loop {
        let session = mgr.GetCurrentSession().ok();

        let id = session
            .as_ref()
            .and_then(|s| s.SourceAppUserModelId().ok())
            .map(|h| h.to_string())
            .unwrap_or_default();

        if id != sub_id {
            if let Some((old, a, b)) = sub.take() {
                let _ = old.RemoveMediaPropertiesChanged(a);
                let _ = old.RemovePlaybackInfoChanged(b);
            }
            if let Some(s) = &session {
                let (t1, t2) = (tx.clone(), tx.clone());
                let a = s.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
                    let _ = t1.send(Msg::Refresh);
                    Ok(())
                }))?;
                let b = s.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
                    let _ = t2.send(Msg::Refresh);
                    Ok(())
                }))?;
                sub = Some((s.clone(), a, b));
            }
            sub_id = id;
        }

        let prev = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let mut snap = match &session {
            Some(s) => read(s, &prev, &mut art_key, &sub_id).unwrap_or(prev),
            None => MediaSnapshot::default(),
        };

        // Handle synced lyrics
        if snap.has_session && !snap.title.is_empty() {
            let key = format!("{}::{}", snap.title, snap.artist);
            if let Some(cached) = lyrics_cache.get(&key) {
                snap.lyrics = Some(cached.clone());
            } else if key != lyrics_key {
                lyrics_key = key.clone();
                let tx_fetch = tx.clone();
                let title = snap.title.clone();
                let artist = snap.artist.clone();
                let duration_sec = snap.duration_ms / 1000;
                let fetch_key = key.clone();

                std::thread::spawn(move || {
                    let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
                    if let Some(lyrics) = fetch_lrclib(&title, &artist, duration_sec) {
                        let _ = tx_fetch.send(Msg::LyricsReady {
                            key: fetch_key,
                            lyrics: Arc::new(lyrics),
                        });
                    }
                });
            }
        }

        *state.lock().unwrap_or_else(|e| e.into_inner()) = snap;
        unsafe {
            let _ = PostMessageW(HWND(raw as *mut _), WM_MEDIA_CHANGED, WPARAM(0), LPARAM(0));
        }

        match rx.recv() {
            Ok(ref m) => {
                if let Msg::LyricsReady { key, lyrics } = m {
                    lyrics_cache.insert(key.clone(), lyrics.clone());
                    let mut lock = state.lock().unwrap_or_else(|e| e.into_inner());
                    if format!("{}::{}", lock.title, lock.artist) == *key {
                        lock.lyrics = Some(lyrics.clone());
                        unsafe {
                            let _ = PostMessageW(HWND(raw as *mut _), WM_MEDIA_CHANGED, WPARAM(0), LPARAM(0));
                        }
                    }
                }
                exec(&mgr, m);
            }
            Err(_) => return Ok(()),
        }
        std::thread::sleep(Duration::from_millis(50));
        while let Ok(ref m) = rx.try_recv() {
            if let Msg::LyricsReady { key, lyrics } = m {
                lyrics_cache.insert(key.clone(), lyrics.clone());
                let mut lock = state.lock().unwrap_or_else(|e| e.into_inner());
                if format!("{}::{}", lock.title, lock.artist) == *key {
                    lock.lyrics = Some(lyrics.clone());
                    unsafe {
                        let _ = PostMessageW(HWND(raw as *mut _), WM_MEDIA_CHANGED, WPARAM(0), LPARAM(0));
                    }
                }
            }
            exec(&mgr, m);
        }
    }
}

fn read(s: &Session, prev: &MediaSnapshot, art_key: &mut String, source: &str) -> Result<MediaSnapshot> {
    let props = s.TryGetMediaPropertiesAsync()?.get()?;
    let title = props.Title()?.to_string();
    let artist = props.Artist()?.to_string();
    let album = props.AlbumTitle()?.to_string();
    let playing = s.GetPlaybackInfo()?.PlaybackStatus()? == PStatus::Playing;

    let ms = |d: i64| (d.max(0) / 10_000) as u64;
    let (position_ms, duration_ms) = match s.GetTimelineProperties() {
        Ok(t) => {
            let start = t.StartTime()?.Duration;
            (
                ms(t.Position()?.Duration - start),
                ms(t.EndTime()?.Duration - start),
            )
        }
        Err(_) => (0, 0),
    };

    let key = format!("{title}\u{1}{artist}\u{1}{album}");
    let art = if key == *art_key {
        prev.art.clone()
    } else {
        load_art(&props).ok().map(|a| {
            *art_key = key.clone();
            a
        })
    };

    Ok(MediaSnapshot {
        has_session: true,
        title,
        artist,
        album,
        source: source.to_string(),
        playing,
        position_ms,
        duration_ms,
        fetched_at: Some(Instant::now()),
        art,
        lyrics: prev.lyrics.clone(),
    })
}

fn load_art(props: &Props) -> Result<Arc<ArtBitmap>> {
    let stream: IRandomAccessStream = props.Thumbnail()?.OpenReadAsync()?.get()?.cast()?;
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
    let tf = BitmapTransform::new()?;
    tf.SetScaledWidth(ART_PX)?;
    tf.SetScaledHeight(ART_PX)?;
    tf.SetInterpolationMode(BitmapInterpolationMode::Fant)?;
    let px = decoder
        .GetPixelDataTransformedAsync(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
            &tf,
            ExifOrientationMode::IgnoreExifOrientation,
            ColorManagementMode::DoNotColorManage,
        )?
        .get()?;
    Ok(Arc::new(ArtBitmap {
        size: ART_PX,
        bgra_premul: px.DetachPixelData()?.to_vec(),
    }))
}

// ---------------------------------------------------------------------------
// LRCLIB Synced Lyrics Client & Parser
// ---------------------------------------------------------------------------

fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":", field);
    let start = json.find(&pattern)? + pattern.len();
    let rem = json[start..].trim_start();
    if rem.starts_with('"') {
        let mut unescaped = String::new();
        let mut chars = rem[1..].chars();
        while let Some(c) = chars.next() {
            if c == '"' {
                break;
            } else if c == '\\' {
                match chars.next()? {
                    'n' => unescaped.push('\n'),
                    'r' => unescaped.push('\r'),
                    't' => unescaped.push('\t'),
                    '\\' => unescaped.push('\\'),
                    '"' => unescaped.push('"'),
                    other => unescaped.push(other),
                }
            } else {
                unescaped.push(c);
            }
        }
        Some(unescaped)
    } else {
        None
    }
}

pub fn parse_lrc(lrc: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for raw in lrc.lines() {
        let trimmed = raw.trim();
        if let Some(bracket_end) = trimmed.find(']') {
            if trimmed.starts_with('[') {
                let tag = &trimmed[1..bracket_end];
                let text = trimmed[bracket_end + 1..].trim().to_string();
                let parts: Vec<&str> = tag.split(|c| c == ':' || c == '.').collect();
                if parts.len() >= 2 {
                    if let (Ok(m), Ok(s)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                        let ms_fraction = if parts.len() >= 3 {
                            if let Ok(frac) = parts[2].parse::<u64>() {
                                if parts[2].len() == 2 {
                                    frac * 10
                                } else {
                                    frac
                                }
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let time_ms = m * 60_000 + s * 1_000 + ms_fraction;
                        lines.push(LyricLine { time_ms, text });
                    }
                }
            }
        }
    }
    lines.sort_by_key(|l| l.time_ms);
    lines
}

fn fetch_lrclib(title: &str, artist: &str, duration_sec: u64) -> Option<Vec<LyricLine>> {
    let enc_title = url_encode(title);
    let enc_artist = url_encode(artist);
    let url_str = if duration_sec > 0 {
        format!(
            "https://lrclib.net/api/get?track_name={}&artist_name={}&duration={}",
            enc_title, enc_artist, duration_sec
        )
    } else {
        format!(
            "https://lrclib.net/api/get?track_name={}&artist_name={}",
            enc_title, enc_artist
        )
    };

    let client = HttpClient::new().ok()?;
    let uri = Uri::CreateUri(&HSTRING::from(url_str)).ok()?;
    let resp = client.GetStringAsync(&uri).ok()?.get().ok()?.to_string();

    if let Some(synced) = extract_json_field(&resp, "syncedLyrics") {
        let parsed = parse_lrc(&synced);
        if !parsed.is_empty() {
            return Some(parsed);
        }
    }

    if let Some(plain) = extract_json_field(&resp, "plainLyrics") {
        let lines: Vec<LyricLine> = plain
            .lines()
            .enumerate()
            .map(|(i, l)| LyricLine {
                time_ms: (i as u64) * 3500,
                text: l.trim().to_string(),
            })
            .collect();
        if !lines.is_empty() {
            return Some(lines);
        }
    }

    None
}
