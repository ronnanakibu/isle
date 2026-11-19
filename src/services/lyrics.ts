import type { LyricLine } from '../types';

export interface LrclibRecord {
  id?: number;
  trackName?: string;
  artistName?: string;
  albumName?: string;
  duration?: number;
  instrumental?: boolean;
  plainLyrics?: string | null;
  syncedLyrics?: string | null;
}

export interface LyricsFetchResult {
  lines: LyricLine[];
  trackName?: string;
  artistName?: string;
  duration?: number;
  instrumental?: boolean;
}

/**
 * Parse fractional seconds from an LRC timestamp tag.
 * e.g. `.5` -> 500ms, `.45` -> 450ms, `.678` -> 678ms.
 */
function parseFraction(frac: string): number {
  let value = 0;
  let digits = 0;
  for (const ch of frac) {
    if (ch < '0' || ch > '9') break;
    value = value * 10 + Number(ch);
    digits += 1;
    if (digits >= 3) break;
  }
  if (digits === 0) return 0;
  return Math.round((value * 1000) / Math.pow(10, digits));
}

/**
 * Parse a timestamp tag body like `01:23.45` or `01:23` into milliseconds.
 */
function parseTimestamp(tag: string): number | null {
  const trimmed = tag.trim();
  const dotIdx = trimmed.indexOf('.');
  let main = trimmed;
  let frac = '';

  if (dotIdx !== -1) {
    main = trimmed.slice(0, dotIdx);
    frac = trimmed.slice(dotIdx + 1);
  } else {
    const parts = trimmed.split(':');
    if (parts.length === 3) {
      main = `${parts[0]}:${parts[1]}`;
      frac = parts[2];
    }
  }

  const colonIdx = main.indexOf(':');
  if (colonIdx === -1) return null;

  const mm = Number(main.slice(0, colonIdx).trim());
  const ss = Number(main.slice(colonIdx + 1).trim());
  if (!Number.isFinite(mm) || !Number.isFinite(ss) || mm < 0 || ss < 0 || ss > 59) {
    return null;
  }

  return mm * 60_000 + ss * 1000 + parseFraction(frac);
}

/**
 * Parse raw LRC text into sorted, deduplicated LyricLine array.
 */
export function parseLrc(raw: string): LyricLine[] {
  const lines: LyricLine[] = [];

  for (const rawLine of raw.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) continue;

    const timestamps: number[] = [];
    let rest = line;

    while (rest.startsWith('[')) {
      const close = rest.indexOf(']');
      if (close === -1) break;
      const tag = rest.slice(1, close);
      const ms = parseTimestamp(tag);
      if (ms === null) break;
      timestamps.push(ms);
      rest = rest.slice(close + 1);
    }

    if (timestamps.length === 0) continue;

    const text = rest.trim();
    for (const timeMs of timestamps) {
      lines.push({ timeMs, text });
    }
  }

  lines.sort((a, b) => a.timeMs - b.timeMs);

  // Deduplicate identical timestamps, preserving the latest text
  const deduped: LyricLine[] = [];
  for (const item of lines) {
    const prev = deduped[deduped.length - 1];
    if (prev && prev.timeMs === item.timeMs) {
      deduped[deduped.length - 1] = item;
    } else {
      deduped.push(item);
    }
  }

  return deduped;
}

/**
 * Binary search for active lyric line index at `positionMs`.
 * Returns -1 if before the first lyric timestamp.
 */
export function getActiveLyricIndex(lines: LyricLine[], positionMs: number): number {
  if (lines.length === 0) return -1;
  let lo = 0;
  let hi = lines.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (lines[mid].timeMs <= positionMs) {
      lo = mid + 1;
    } else {
      hi = mid;
    }
  }
  return lo - 1;
}

/**
 * Format total seconds as `mm:ss` (e.g. `03:42`).
 */
export function formatTime(totalSeconds: number): string {
  const clamped = Math.max(0, Math.floor(totalSeconds));
  const mins = Math.floor(clamped / 60);
  const secs = clamped % 60;
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

const LRCLIB_HEADERS: HeadersInit = {
  'User-Agent': 'Isle/1.0 (+https://github.com/isle)',
};

/**
 * Live LRCLIB client fetching synced lyrics with exact match or search fallback.
 */
export async function fetchLrclibLyrics(
  title: string,
  artist: string,
  durationSec?: number,
  signal?: AbortSignal
): Promise<LyricsFetchResult | null> {
  const cleanTitle = title.trim();
  const cleanArtist = artist.trim();
  if (!cleanTitle) return null;

  try {
    // 1. Exact match endpoint with duration if available
    if (cleanArtist) {
      let getUrl = `https://lrclib.net/api/get?track_name=${encodeURIComponent(
        cleanTitle
      )}&artist_name=${encodeURIComponent(cleanArtist)}`;
      if (durationSec && durationSec > 0) {
        getUrl += `&duration=${Math.round(durationSec)}`;
      }

      const res = await fetch(getUrl, {
        headers: LRCLIB_HEADERS,
        signal,
      });

      if (res.ok) {
        const data = (await res.json()) as LrclibRecord;
        if (data.syncedLyrics) {
          const parsed = parseLrc(data.syncedLyrics);
          if (parsed.length > 0) {
            return {
              lines: parsed,
              trackName: data.trackName,
              artistName: data.artistName,
              duration: data.duration,
              instrumental: data.instrumental,
            };
          }
        }
      }
    }

    // 2. Fallback search endpoint
    const query = cleanArtist ? `${cleanTitle} ${cleanArtist}` : cleanTitle;
    const searchUrl = `https://lrclib.net/api/search?q=${encodeURIComponent(query)}`;
    const searchRes = await fetch(searchUrl, {
      headers: LRCLIB_HEADERS,
      signal,
    });

    if (!searchRes.ok) return null;

    const results = (await searchRes.json()) as LrclibRecord[];
    const match = results.find((r) => Boolean(r.syncedLyrics));
    if (!match?.syncedLyrics) return null;

    const parsed = parseLrc(match.syncedLyrics);
    if (parsed.length === 0) return null;

    return {
      lines: parsed,
      trackName: match.trackName,
      artistName: match.artistName,
      duration: match.duration,
      instrumental: match.instrumental,
    };
  } catch {
    return null;
  }
}

/**
 * Manual search query against LRCLIB for the search modal / omnibar.
 */
export async function searchLrclibManual(
  query: string,
  signal?: AbortSignal
): Promise<LrclibRecord[]> {
  const clean = query.trim();
  if (!clean) return [];

  try {
    const searchUrl = `https://lrclib.net/api/search?q=${encodeURIComponent(clean)}`;
    const res = await fetch(searchUrl, {
      headers: LRCLIB_HEADERS,
      signal,
    });
    if (!res.ok) return [];
    return (await res.json()) as LrclibRecord[];
  } catch {
    return [];
  }
}
