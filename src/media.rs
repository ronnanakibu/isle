//! Live media integration via the System Media Transport Controls.
//!
//! SMTC is a WinRT API, and `HWND` is not `Send`, so every WinRT call lives
//! on a dedicated worker thread (`isle-smtc`). That thread watches for session
//! changes, snapshots the current track, and posts a message to the UI thread
//! when something changed. The UI thread only ever reads a plain struct
//! behind a mutex — it never touches a COM object.
//!
//! Album art is pulled from the session thumbnail and decoded through WIC
//! straight into **premultiplied BGRA**, which is exactly the layout the
//! canvas expects, so no conversion is needed at blit time.
//!
//! The worker is purely event-driven: it sleeps on a channel and only wakes
//! when SMTC raises an event or the UI asks for a refresh. Nothing polls.

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use windows::Win32::System::SystemInformation::GetTickCount64;

use windows::core::Interface;
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::Media::Control::{
    CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus, MediaPropertiesChangedEventArgs,
    PlaybackInfoChangedEventArgs, TimelinePropertiesChangedEventArgs,
};
use windows::Storage::Streams::{DataReader, IRandomAccessStream};
use windows::Win32::Graphics::Imaging::{
    IWICBitmapFrameDecode, IWICBitmapScaler, IWICFormatConverter, IWICImagingFactory,
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone,
    WICBitmapInterpolationModeFant, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::{
    CoCreateInstance as CoCreate, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::SHCreateMemStream;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::window::WM_MEDIA_CHANGED;

/// The edge length of a decoded album-art image.
const ART_SIZE: u32 = 256;

/// A decoded album-art image: premultiplied BGRA, row-major.
#[derive(Clone)]
pub struct Art {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

/// Everything the UI needs to know about the current media session.
#[derive(Clone, Default)]
pub struct Snapshot {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Whether the session is currently playing.
    pub playing: bool,
    /// Playback position in milliseconds, as of the snapshot.
    pub position_ms: u32,
    /// Track duration in milliseconds.
    pub duration_ms: u32,
    /// Decoded album art, when the session supplied one.
    pub art: Option<Art>,
    /// System tick count when this snapshot was taken.
    pub taken_at: u64,
}

impl Snapshot {
    fn is_empty(&self) -> bool {
        self.title.is_empty() && self.artist.is_empty()
    }

    pub fn position_now(&self) -> u32 {
        if !self.playing || self.duration_ms == 0 {
            return self.position_ms;
        }
        let now = unsafe { GetTickCount64() };
        let elapsed = now.saturating_sub(self.taken_at) as u32;
        (self.position_ms + elapsed).min(self.duration_ms)
    }

    pub fn progress(&self) -> f32 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        self.position_now() as f32 / self.duration_ms as f32
    }
}

/// The latest snapshot, shared between the worker and the UI thread.
static SNAPSHOT: Mutex<Option<Snapshot>> = Mutex::new(None);

/// The window that receives change notifications.
static WINDOW: AtomicIsize = AtomicIsize::new(0);

/// The live session, so the UI thread can drive transport controls.
static SESSION: Mutex<Option<GlobalSystemMediaTransportControlsSession>> = Mutex::new(None);

/// Commands sent from the UI thread to the SMTC worker.
enum Command {
    /// Re-read the current session.
    Refresh,
}

static SENDER: Mutex<Option<Sender<Command>>> = Mutex::new(None);

/// Start the SMTC worker for the given window.
pub fn init(hwnd: HWND) {
    WINDOW.store(hwnd.0 as isize, Ordering::SeqCst);

    let (tx, rx) = channel::<Command>();
    if let Ok(mut slot) = SENDER.lock() {
        *slot = Some(tx.clone());
    }

    let _ = std::thread::Builder::new()
        .name("isle-smtc".to_string())
        .spawn(move || worker_main(rx));
}

/// The current snapshot, if any.
pub fn snapshot() -> Option<Snapshot> {
    SNAPSHOT.lock().ok().and_then(|s| s.clone())
}

/// The current track title.
#[allow(dead_code)]
pub fn current_title() -> String {
    snapshot().map(|s| s.title).unwrap_or_default()
}

/// The current track artist.
#[allow(dead_code)]
pub fn current_artist() -> String {
    snapshot().map(|s| s.artist).unwrap_or_default()
}

/// Ask the worker to re-read the session.
pub fn refresh() {
    if let Ok(slot) = SENDER.lock() {
        if let Some(tx) = slot.as_ref() {
            let _ = tx.send(Command::Refresh);
        }
    }
}

/// Toggle play/pause.
pub fn toggle_play_pause() {
    with_session(|session| {
        let _ = session.TryTogglePlayPauseAsync().and_then(|op| op.get());
    });
    refresh();
}

/// Skip to the next track.
pub fn next() {
    with_session(|session| {
        let _ = session.TrySkipNextAsync().and_then(|op| op.get());
    });
    refresh();
}

/// Go back to the previous track.
pub fn previous() {
    with_session(|session| {
        let _ = session.TrySkipPreviousAsync().and_then(|op| op.get());
    });
    refresh();
}

fn with_session(f: impl FnOnce(&GlobalSystemMediaTransportControlsSession)) {
    if let Ok(guard) = SESSION.lock() {
        if let Some(session) = guard.as_ref() {
            f(session);
        }
    }
}

/// Notify the UI thread that the media state changed.
fn post_change() {
    let hwnd = WINDOW.load(Ordering::SeqCst);
    if hwnd == 0 {
        return;
    }
    unsafe {
        let _ = PostMessageW(
            HWND(hwnd as *mut _),
            WM_MEDIA_CHANGED,
            WPARAM(0),
            LPARAM(0),
        );
    }
}

// ---------------------------------------------------------------------------
// Worker
// ---------------------------------------------------------------------------

/// Event subscriptions for the session currently being watched.
struct Bindings {
    /// Identifies the session the tokens belong to.
    session_id: String,
    props: EventRegistrationToken,
    playback: EventRegistrationToken,
    timeline: EventRegistrationToken,
}

/// The worker's body. It owns the SMTC manager and keeps the event
/// subscriptions alive for the life of the process.
fn worker_main(rx: Receiver<Command>) {
    unsafe {
        // The worker needs its own COM apartment.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|op| op.get())
        {
            Ok(m) => m,
            Err(_) => {
                CoUninitialize();
                return;
            }
        };

        let tx = SENDER
            .lock()
            .ok()
            .and_then(|s| s.clone())
            .unwrap_or_else(|| channel().0);

        // The session can change at any time; when it does we re-bind to the
        // new session's events.
        let manager_for_event = manager.clone();
        let tx_for_event = tx.clone();
        let current_token = manager.CurrentSessionChanged(
            &TypedEventHandler::<
                GlobalSystemMediaTransportControlsSessionManager,
                CurrentSessionChangedEventArgs,
            >::new(
                move |_sender, _args| {
                    let _ = tx_for_event.send(Command::Refresh);
                    let _ = &manager_for_event;
                    Ok(())
                },
            ),
        );
        let _ = current_token;

        let mut bindings: Option<Bindings> = None;

        while let Ok(command) = rx.recv() {
            match command {
                Command::Refresh => {
                    // Re-bind if the active session changed.
                    if let Ok(session) = manager.GetCurrentSession() {
                        let id = session
                            .SourceAppUserModelId()
                            .map(|s| s.to_string())
                            .unwrap_or_default();

                        let needs_bind = match &bindings {
                            Some(b) => b.session_id != id,
                            None => true,
                        };

                        if needs_bind {
                            if let Some(old) = bindings.take() {
                                if let Ok(session) = manager.GetCurrentSession() {
                                    let _ = session.RemoveMediaPropertiesChanged(old.props);
                                    let _ = session.RemovePlaybackInfoChanged(old.playback);
                                    let _ = session.RemoveTimelinePropertiesChanged(old.timeline);
                                }
                            }

                            let tx1 = tx.clone();
                            let tx2 = tx.clone();
                            let tx3 = tx.clone();
                            let props = session.MediaPropertiesChanged(
                                &TypedEventHandler::<
                                    GlobalSystemMediaTransportControlsSession,
                                    MediaPropertiesChangedEventArgs,
                                >::new(move |_s, _a| {
                                    let _ = tx1.send(Command::Refresh);
                                    Ok(())
                                }),
                            );
                            let playback = session.PlaybackInfoChanged(
                                &TypedEventHandler::<
                                    GlobalSystemMediaTransportControlsSession,
                                    PlaybackInfoChangedEventArgs,
                                >::new(move |_s, _a| {
                                    let _ = tx2.send(Command::Refresh);
                                    Ok(())
                                }),
                            );
                            let timeline = session.TimelinePropertiesChanged(
                                &TypedEventHandler::<
                                    GlobalSystemMediaTransportControlsSession,
                                    TimelinePropertiesChangedEventArgs,
                                >::new(move |_s, _a| {
                                    let _ = tx3.send(Command::Refresh);
                                    Ok(())
                                }),
                            );

                            bindings = Some(Bindings {
                                session_id: id,
                                props: props.unwrap_or_default(),
                                playback: playback.unwrap_or_default(),
                                timeline: timeline.unwrap_or_default(),
                            });
                        }

                        take_snapshot(&session);
                    } else {
                        clear_snapshot();
                    }
                    publish();
                }
            }
        }

        CoUninitialize();
    }
}

/// Read the current session into the shared state.
unsafe fn take_snapshot(session: &GlobalSystemMediaTransportControlsSession) {
    let props = session
        .TryGetMediaPropertiesAsync()
        .and_then(|op| op.get())
        .ok();

    let mut snap = Snapshot::default();
    if let Some(p) = &props {
        snap.title = p.Title().map(|s| s.to_string()).unwrap_or_default();
        snap.artist = p.Artist().map(|s| s.to_string()).unwrap_or_default();
        snap.album = p.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();
        if let Ok(thumb) = p.Thumbnail() {
            snap.art = decode_thumbnail(&thumb).ok();
        }
    }

    if let Ok(info) = session.GetPlaybackInfo() {
        snap.playing = info
            .PlaybackStatus()
            .map(|s| s == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
            .unwrap_or(false);
    }

    if let Ok(timeline) = session.GetTimelineProperties() {
        // TimeSpans are in 100-nanosecond ticks.
        let end = timeline.EndTime().map(|t| t.Duration).unwrap_or(0);
        snap.duration_ms = (end / 10_000).max(0) as u32;
        let pos = timeline.Position().map(|t| t.Duration).unwrap_or(0);
        snap.position_ms = (pos / 10_000).max(0) as u32;
    }

    snap.taken_at = GetTickCount64();
    if let Ok(mut g) = SNAPSHOT.lock() {
        *g = if snap.is_empty() { None } else { Some(snap) };
    }
    if let Ok(mut g) = SESSION.lock() {
        *g = Some(session.clone());
    }
}

/// Forget the current session.
fn clear_snapshot() {
    if let Ok(mut g) = SNAPSHOT.lock() {
        *g = None;
    }
    if let Ok(mut g) = SESSION.lock() {
        *g = None;
    }
}

/// Tell the UI thread that the media state changed.
fn publish() {
    post_change();
}

// ---------------------------------------------------------------------------
// Album art
// ---------------------------------------------------------------------------

/// Decode a session thumbnail into premultiplied BGRA.
unsafe fn decode_thumbnail(
    thumb: &windows::Storage::Streams::IRandomAccessStreamReference,
) -> Result<Art, String> {
    let stream = thumb
        .OpenReadAsync()
        .and_then(|op| op.get())
        .map_err(|e| format!("thumbnail unavailable: {e}"))?;

    // Read the encoded bytes out of the WinRT stream.
    let random: IRandomAccessStream = stream.cast().map_err(|e| e.to_string())?;
    let size = random.Size().map_err(|e| e.to_string())? as usize;
    if size == 0 || size > 32 * 1024 * 1024 {
        return Err("thumbnail has an implausible size".into());
    }
    let input = random.GetInputStreamAt(0).map_err(|e| e.to_string())?;
    let reader = DataReader::CreateDataReader(&input).map_err(|e| e.to_string())?;
    reader
        .LoadAsync(size as u32)
        .and_then(|op| op.get())
        .map_err(|e| e.to_string())?;
    let mut bytes =
        vec![0u8; reader.UnconsumedBufferLength().map_err(|e| e.to_string())? as usize];
    reader.ReadBytes(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Err("thumbnail was empty".into());
    }

    // Decode with WIC directly into premultiplied BGRA.
    let factory: IWICImagingFactory = CoCreate(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
        .map_err(|e| e.to_string())?;

    let mem = SHCreateMemStream(Some(&bytes)).ok_or("could not wrap the thumbnail in a stream")?;
    let decoder = factory
        .CreateDecoderFromStream(&mem, std::ptr::null(), WICDecodeMetadataCacheOnDemand)
        .map_err(|e| e.to_string())?;
    let frame: IWICBitmapFrameDecode = decoder.GetFrame(0).map_err(|e| e.to_string())?;

    // Scale down first: album art is routinely 1000+ pixels wide, and we only
    // ever draw it at 58 or 128 pixels.
    let scaler: IWICBitmapScaler = factory.CreateBitmapScaler().map_err(|e| e.to_string())?;
    scaler
        .Initialize(&frame, ART_SIZE, ART_SIZE, WICBitmapInterpolationModeFant)
        .map_err(|e| e.to_string())?;

    let converter: IWICFormatConverter =
        factory.CreateFormatConverter().map_err(|e| e.to_string())?;
    converter
        .Initialize(
            &scaler,
            &GUID_WICPixelFormat32bppPBGRA,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeCustom,
        )
        .map_err(|e| e.to_string())?;

    let stride = ART_SIZE * 4;
    let mut data = vec![0u8; (stride as usize) * (ART_SIZE as usize)];
    converter
        .CopyPixels(std::ptr::null(), stride, &mut data)
        .map_err(|e| e.to_string())?;

    Ok(Art {
        width: ART_SIZE as usize,
        height: ART_SIZE as usize,
        data,
    })
}
