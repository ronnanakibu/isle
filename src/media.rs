//! SMTC Bridge: WinRT Media Control & Live Album Artwork
//! Runs on a background worker thread. UI thread receives WM_MEDIA_CHANGED.
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use windows::{
    core::{Interface, Result},
    Foundation::{EventRegistrationToken, TypedEventHandler},
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

#[derive(Clone, Default)]
pub struct MediaSnapshot {
    pub has_session: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source: String,
    pub playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub fetched_at: Option<Instant>,
    pub art: Option<Arc<ArtBitmap>>,
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

fn exec(mgr: &Manager, m: Msg) {
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
        let snap = match &session {
            Some(s) => read(s, &prev, &mut art_key, &sub_id).unwrap_or(prev),
            None => MediaSnapshot::default(),
        };
        *state.lock().unwrap_or_else(|e| e.into_inner()) = snap;
        unsafe {
            let _ = PostMessageW(HWND(raw as *mut _), WM_MEDIA_CHANGED, WPARAM(0), LPARAM(0));
        }

        match rx.recv() {
            Ok(m) => exec(&mgr, m),
            Err(_) => return Ok(()),
        }
        std::thread::sleep(Duration::from_millis(50));
        while let Ok(m) = rx.try_recv() {
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
