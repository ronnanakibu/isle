//! LRC (synced lyrics) parsing and lookup.
//!
//! Lyrics arrive as a flat list of `[mm:ss.xx] text` lines. We parse them
//! into sorted millisecond offsets and resolve the active line for the current
//! playback position with a binary search.

use std::sync::Mutex;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Networking::WinInet::{
    InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
    INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_RELOAD, INTERNET_FLAG_SECURE,
};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::window::WM_LYRICS_LOADED;

static FETCHED: Mutex<Option<(String, String, Lyrics)>> = Mutex::new(None);

/// Spawn a background thread to fetch synced lyrics from LRCLIB and notify `hwnd`.
pub fn fetch_async(hwnd: HWND, title: String, artist: String) {
    if title.is_empty() {
        return;
    }
    let hwnd_raw = hwnd.0 as isize;
    std::thread::Builder::new()
        .name("isle-lyrics".to_string())
        .spawn(move || {
            if let Some(lrc) = fetch_lrclib(&title, &artist) {
                let parsed = Lyrics::parse(&lrc);
                if parsed.is_synced {
                    if let Ok(mut g) = FETCHED.lock() {
                        *g = Some((title, artist, parsed));
                    }
                    unsafe {
                        let _ = PostMessageW(
                            HWND(hwnd_raw as *mut _),
                            WM_LYRICS_LOADED,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                }
            }
        })
        .ok();
}

/// Take the most recently fetched lyrics, if any.
pub fn take_fetched() -> Option<(String, String, Lyrics)> {
    FETCHED.lock().ok().and_then(|mut g| g.take())
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

fn fetch_lrclib(title: &str, artist: &str) -> Option<String> {
    let url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}",
        url_encode(title),
        url_encode(artist)
    );
    let wide_url: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let agent = InternetOpenW(w!("Isle/0.1"), 0, PCWSTR::null(), PCWSTR::null(), 0);
        if agent.is_null() {
            return None;
        }
        let flags = INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE | INTERNET_FLAG_SECURE;
        let req = InternetOpenUrlW(agent, PCWSTR(wide_url.as_ptr()), None, flags, 0);
        if req.is_null() {
            let _ = InternetCloseHandle(agent);
            return None;
        }

        let mut body = Vec::new();
        let mut buf = [0u8; 4096];
        let mut read = 0u32;
        while InternetReadFile(req, buf.as_mut_ptr() as *mut _, buf.len() as u32, &mut read).is_ok()
            && read > 0
        {
            body.extend_from_slice(&buf[..read as usize]);
            if body.len() > 512 * 1024 {
                break;
            }
        }

        let _ = InternetCloseHandle(req);
        let _ = InternetCloseHandle(agent);

        let json = String::from_utf8(body).ok()?;
        extract_json_string(&json, "syncedLyrics")
    }
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
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
                '/' => out.push('/'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}

/// A single synced lyric line.
#[derive(Clone, Debug)]
pub struct LyricLine {
    /// Offset from the start of the track, in milliseconds.
    pub time_ms: u32,
    pub text: String,
}

/// A parsed LRC document.
#[derive(Clone, Debug, Default)]
pub struct Lyrics {
    /// Lines sorted ascending by `time_ms`.
    pub lines: Vec<LyricLine>,
    /// Track duration in milliseconds, when the server reported one.
    pub duration_ms: Option<u32>,
    /// Whether the document contains any timed lines at all.
    pub is_synced: bool,
}

impl Lyrics {
    /// Parse raw LRC text.
    pub fn parse(raw: &str) -> Self {
        let mut lines: Vec<LyricLine> = Vec::new();
        let mut duration_ms = None;

        for raw_line in raw.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            // Metadata tags: [ti:...], [ar:...], [al:...], [length:...], [offset:...]
            if line.starts_with('[') {
                if let Some(close) = line.find(']') {
                    let inner = &line[1..close];
                    if let Some((key, value)) = inner.split_once(':') {
                        let key = key.trim().to_ascii_lowercase();
                        if key == "length" || key == "duration" {
                            if let Some(ms) = parse_clock(value.trim()) {
                                duration_ms = Some(ms);
                                continue;
                            }
                        }
                        if !key.chars().all(|c| c.is_ascii_alphabetic()) || key.len() > 12 {
                            // Not a known metadata tag — fall through and treat
                            // it as a timestamp line.
                        } else {
                            continue;
                        }
                    }
                }
            }

            // Timestamp line: one or more `[mm:ss.xx]` tags followed by text.
            let mut timestamps = Vec::new();
            let mut rest = line;
            while rest.starts_with('[') {
                let close = match rest.find(']') {
                    Some(c) => c,
                    None => break,
                };
                let tag = &rest[1..close];
                match parse_timestamp(tag) {
                    Some(ms) => timestamps.push(ms),
                    None => break,
                }
                rest = &rest[close + 1..];
            }

            if timestamps.is_empty() {
                continue;
            }

            let text = rest.trim().to_string();
            for time_ms in timestamps {
                lines.push(LyricLine { time_ms, text: text.clone() });
            }
        }

        lines.sort_by_key(|l| l.time_ms);
        // Deduplicate identical timestamps, keeping the last.
        lines.dedup_by(|a, b| a.time_ms == b.time_ms);

        let is_synced = !lines.is_empty();
        Self { lines, duration_ms, is_synced }
    }

    /// Index of the line active at `position_ms`, if any.
    ///
    /// Returns `None` before the first line starts.
    pub fn active_index(&self, position_ms: u32) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }
        // Rightmost line whose time is <= position.
        let idx = self
            .lines
            .partition_point(|l| l.time_ms <= position_ms);
        if idx == 0 {
            None
        } else {
            Some(idx - 1)
        }
    }

    /// The active lyric text at `position_ms`.
    pub fn active_text(&self, position_ms: u32) -> Option<&str> {
        self.active_index(position_ms).map(|i| self.lines[i].text.as_str())
    }

    /// The upcoming line after `position_ms`, used for the "next line"
    /// preview in the card.
    pub fn next_text(&self, position_ms: u32) -> Option<&str> {
        let idx = self.active_index(position_ms)?;
        self.lines.get(idx + 1).map(|l| l.text.as_str())
    }

    /// Progress (0..=1) through the active line, for a subtle sweep animation.
    #[allow(dead_code)]
    pub fn line_progress(&self, position_ms: u32) -> f32 {
        let idx = match self.active_index(position_ms) {
            Some(i) => i,
            None => return 0.0,
        };
        let start = self.lines[idx].time_ms;
        let end = self.lines.get(idx + 1).map(|l| l.time_ms).unwrap_or(start + 4000);
        if end <= start {
            return 1.0;
        }
        ((position_ms.saturating_sub(start)) as f32 / (end - start) as f32).clamp(0.0, 1.0)
    }
}

/// Parse a fractional-seconds string into milliseconds.
///
/// The fraction is right-padded to hundredths, the LRC convention: `.5` is
/// 500 ms, `.45` is 450 ms, `.678` is 678 ms.
fn parse_fraction(frac: &str) -> u32 {
    let mut value = 0u32;
    let mut digits = 0u32;
    for ch in frac.chars() {
        if !ch.is_ascii_digit() {
            break;
        }
        value = value * 10 + (ch as u32 - '0' as u32);
        digits += 1;
        if digits >= 3 {
            break;
        }
    }
    if digits == 0 {
        return 0;
    }
    let divisor = 10u32.pow(digits);
    value * 1000 / divisor
}

/// Parse a timestamp tag body, e.g. `01:23.45` or `01:23`.
fn parse_timestamp(tag: &str) -> Option<u32> {
    let tag = tag.trim();
    // Split off fractional seconds.
    let (main, frac) = match tag.split_once('.') {
        Some((m, f)) => (m, f),
        None => match tag.split_once(':') {
            Some((m, f)) => (m, f),
            None => return None,
        },
    };
    let (mm, ss) = main.split_once(':')?;
    let mm: u32 = mm.trim().parse().ok()?;
    let ss: u32 = ss.trim().parse().ok()?;
    if ss > 59 {
        return None;
    }
    Some(mm * 60_000 + ss * 1000 + parse_fraction(frac))
}

/// Parse a clock-style duration, e.g. `3:21` or `03:21.5` -> milliseconds.
fn parse_clock(value: &str) -> Option<u32> {
    let (mm, ss) = value.split_once(':')?;
    let mm: u32 = mm.trim().parse().ok()?;
    let (ss, frac) = match ss.trim().split_once('.') {
        Some((s, f)) => (s, f),
        None => (ss.trim(), ""),
    };
    let ss: u32 = ss.parse().ok()?;
    Some(mm * 60_000 + ss * 1000 + parse_fraction(frac))
}

/// Format a millisecond duration as `m:ss`.
pub fn format_duration(ms: u32) -> String {
    let total_secs = ms / 1000;
    format!("{}:{:02}", total_secs / 60, total_secs % 60)
}

/// Format a duration as `h:mm:ss` when long enough, else `m:ss`.
#[allow(dead_code)]
pub fn format_clock(ms: u32) -> String {
    let total_secs = ms / 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_lrc() {
        let raw = "[ti:Song]\n[ar:Artist]\n[00:12.00]First line\n[00:15.50]Second line\n";
        let l = Lyrics::parse(raw);
        assert_eq!(l.lines.len(), 2);
        assert_eq!(l.lines[0].text, "First line");
        assert_eq!(l.lines[0].time_ms, 12_000);
        assert_eq!(l.lines[1].time_ms, 15_500);
    }

    #[test]
    fn parses_enhanced_timestamp_format() {
        let l = Lyrics::parse("[00:12.34]Hello\n[00:15.678]World\n");
        assert_eq!(l.lines[0].time_ms, 12_340);
        assert_eq!(l.lines[1].time_ms, 15_678);
    }

    #[test]
    fn handles_multiple_timestamps_per_line() {
        let l = Lyrics::parse("[00:10.00][01:10.00]Repeated\n");
        assert_eq!(l.lines.len(), 2);
        assert_eq!(l.lines[1].time_ms, 70_000);
    }

    #[test]
    fn binary_search_resolves_active_line() {
        let l = Lyrics::parse("[00:00.00]Zero\n[00:10.00]Ten\n[00:20.00]Twenty\n");
        assert_eq!(l.active_text(5_000), Some("Zero"));
        assert_eq!(l.active_text(10_000), Some("Ten"));
        assert_eq!(l.active_text(25_000), Some("Twenty"));
        assert_eq!(l.active_text(0), Some("Zero"));
    }

    #[test]
    fn before_first_line_is_none() {
        let l = Lyrics::parse("[00:10.00]Ten\n");
        assert_eq!(l.active_text(1_000), None);
    }

    #[test]
    fn metadata_tags_are_skipped() {
        let l = Lyrics::parse("[ti:Title]\n[length:3:21]\n[00:01.00]Real\n");
        assert_eq!(l.lines.len(), 1);
        assert_eq!(l.duration_ms, Some(201_000));
    }

    #[test]
    fn unsorted_input_is_sorted() {
        let l = Lyrics::parse("[00:30.00]Late\n[00:10.00]Early\n");
        assert_eq!(l.lines[0].text, "Early");
    }

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(65_000), "1:05");
        assert_eq!(format_clock(3_725_000), "1:02:05");
    }
}
