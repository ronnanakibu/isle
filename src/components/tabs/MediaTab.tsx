import React, { useEffect, useMemo, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  AlignLeft,
  Pause,
  Play,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
} from 'lucide-react';
import type { LyricsState, Track } from '../../types';
import { formatTime, getActiveLyricIndex } from '../../services/lyrics';
import { Equalizer } from '../Equalizer';

interface MediaTabProps {
  track: Track;
  playing: boolean;
  positionSec: number;
  volume: number;
  lyrics: LyricsState;
  showLyrics: boolean;
  onToggleShowLyrics: () => void;
  onTogglePlay: () => void;
  onPrevTrack: () => void;
  onNextTrack: () => void;
  onSeek: (sec: number) => void;
  onVolumeChange: (vol: number) => void;
  onScrollTrapChange?: (trapped: boolean) => void;
}

export const MediaTab: React.FC<MediaTabProps> = ({
  track,
  playing,
  positionSec,
  volume,
  lyrics,
  showLyrics,
  onToggleShowLyrics,
  onTogglePlay,
  onPrevTrack,
  onNextTrack,
  onSeek,
  onVolumeChange,
  onScrollTrapChange,
}) => {
  const lineRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const positionMs = Math.round(positionSec * 1000);
  const activeIndex = useMemo(
    () => getActiveLyricIndex(lyrics.lines, positionMs),
    [lyrics.lines, positionMs]
  );

  // Smoothly center the active lyric line
  useEffect(() => {
    if (!showLyrics || activeIndex < 0) return;
    const activeEl = lineRefs.current[activeIndex];
    if (activeEl) {
      activeEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, [activeIndex, track.id, showLyrics]);

  const progressPct =
    track.durationSec > 0
      ? Math.min(100, Math.max(0, (positionSec / track.durationSec) * 100))
      : 0;

  return (
    <div className="w-full h-full px-4 pb-3 flex flex-col justify-between select-none">
      {/* Main Container: Full width or Side-by-Side when Lyrics Active */}
      <div className="flex-1 flex items-stretch gap-3 min-h-0">
        {/* Playback Controls Column (Zero harsh vertical dividing line) */}
        <motion.div
          layout
          transition={{ type: 'spring', stiffness: 380, damping: 30 }}
          className={`flex flex-col justify-between ${
            showLyrics ? 'w-1/2 pr-2' : 'w-full'
          }`}
        >
          {/* Top Row: Album Art + Track Info + Actions */}
          <div className="flex items-center gap-3">
            {/* Vinyl Album Art */}
            <motion.div
              className="relative w-[50px] h-[50px] rounded-2xl shrink-0 overflow-hidden shadow-md border border-white/15"
              style={{
                background: `linear-gradient(135deg, ${track.coverGradient[0]}, ${track.coverGradient[1]}, ${track.coverGradient[2]})`,
              }}
              animate={{ scale: playing ? 1 : 0.94 }}
              transition={{ type: 'spring', stiffness: 360, damping: 26 }}
            >
              <div
                className="absolute inset-0 opacity-30"
                style={{
                  background:
                    'radial-gradient(circle at 28% 24%, rgba(255,255,255,0.45), transparent 55%)',
                }}
              />
              <div
                className={`absolute inset-2 rounded-full border border-white/20 flex items-center justify-center ${
                  playing ? 'animate-isle-spin' : ''
                }`}
              >
                <div className="w-2.5 h-2.5 rounded-full bg-black/70 border border-white/30" />
              </div>
            </motion.div>

            {/* Title & Artist */}
            <div className="min-w-0 flex-1">
              <h3 className="text-[13px] font-semibold text-white truncate leading-tight">
                {track.title}
              </h3>
              <p className="text-[11px] text-white/60 truncate mt-0.5">
                {track.artist} · <span className="text-white/40">{track.album}</span>
              </p>
            </div>

            {/* EQ & Lyrics Toggle Button */}
            <div
              className="flex items-center gap-1.5 shrink-0"
              onPointerDown={(e) => e.stopPropagation()}
            >
              <Equalizer
                playing={playing}
                bpm={track.bpm}
                color={track.accentColor}
                size="sm"
              />

              {/* Apple-style Lyrics Reveal Toggle */}
              <motion.button
                type="button"
                whileHover={{ scale: 1.08 }}
                whileTap={{ scale: 0.88 }}
                onClick={onToggleShowLyrics}
                className={`w-7 h-7 rounded-full flex items-center justify-center border transition-all cursor-pointer ${
                  showLyrics
                    ? 'bg-white/20 border-white/30 text-white'
                    : 'bg-white/[0.06] border-white/10 text-white/60 hover:text-white hover:bg-white/12'
                }`}
                title={showLyrics ? 'Hide Lyrics Pane' : 'Reveal Synced Lyrics'}
                aria-label="Toggle synced lyrics pane"
              >
                <AlignLeft className="w-3.5 h-3.5" />
              </motion.button>
            </div>
          </div>

          {/* Fluid Transport Row (Clean, unconstrained, centered) */}
          <div
            className="flex items-center justify-center gap-4 my-1"
            onPointerDown={(e) => e.stopPropagation()}
          >
            <motion.button
              type="button"
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.88 }}
              onClick={onPrevTrack}
              className="w-7 h-7 rounded-full flex items-center justify-center text-white/75 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
              aria-label="Previous track"
            >
              <SkipBack className="w-3.5 h-3.5 fill-current" />
            </motion.button>

            <motion.button
              type="button"
              whileHover={{ scale: 1.08 }}
              whileTap={{ scale: 0.86 }}
              onClick={onTogglePlay}
              className="w-9 h-9 rounded-full flex items-center justify-center bg-white text-black shadow-sm hover:bg-white/90 transition-colors cursor-pointer"
              aria-label={playing ? 'Pause' : 'Play'}
            >
              {playing ? (
                <Pause className="w-4 h-4 fill-current" />
              ) : (
                <Play className="w-4 h-4 fill-current ml-0.5" />
              )}
            </motion.button>

            <motion.button
              type="button"
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.88 }}
              onClick={onNextTrack}
              className="w-7 h-7 rounded-full flex items-center justify-center text-white/75 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
              aria-label="Next track"
            >
              <SkipForward className="w-3.5 h-3.5 fill-current" />
            </motion.button>
          </div>

          {/* Scrubber & Mini Volume Slider */}
          <div
            className="flex items-center gap-2"
            onPointerDown={(e) => e.stopPropagation()}
          >
            <span className="text-[10px] font-medium text-white/55 tabular-nums w-7">
              {formatTime(positionSec)}
            </span>

            <div className="relative flex-1 h-3.5 flex items-center group">
              <div className="w-full h-1 rounded-full bg-white/15 overflow-hidden">
                <div
                  className="h-full rounded-full transition-[width] duration-100"
                  style={{
                    width: `${progressPct}%`,
                    backgroundColor: track.accentColor,
                  }}
                />
              </div>
              <input
                type="range"
                min={0}
                max={track.durationSec || 100}
                step={0.5}
                value={positionSec}
                onChange={(e) => onSeek(Number(e.target.value))}
                className="isle-range absolute inset-0 w-full h-full opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
                aria-label="Seek playback position"
              />
            </div>

            <span className="text-[10px] font-medium text-white/45 tabular-nums w-7 text-right">
              {formatTime(track.durationSec)}
            </span>

            <div className="w-px h-3 bg-white/12 mx-0.5" />

            {/* Mini Volume Slider */}
            <div className="flex items-center gap-1.5 w-[62px] shrink-0">
              <button
                type="button"
                onClick={() => onVolumeChange(volume > 0 ? 0 : 0.45)}
                className="text-white/60 hover:text-white transition-colors cursor-pointer"
                aria-label={volume === 0 ? 'Unmute' : 'Mute'}
              >
                {volume === 0 ? (
                  <VolumeX className="w-3 h-3 text-rose-400" />
                ) : (
                  <Volume2 className="w-3 h-3" />
                )}
              </button>
              <div className="relative flex-1 h-3 flex items-center">
                <div className="w-full h-1 rounded-full bg-white/15 overflow-hidden">
                  <div
                    className="h-full rounded-full bg-white/80"
                    style={{ width: `${Math.round(volume * 100)}%` }}
                  />
                </div>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.02}
                  value={volume}
                  onChange={(e) => onVolumeChange(Number(e.target.value))}
                  className="isle-range absolute inset-0 w-full h-full opacity-0 hover:opacity-100 cursor-pointer"
                  aria-label="Volume"
                />
              </div>
            </div>
          </div>
        </motion.div>

        {/* Right Half: Floating Glass Synced Lyrics Pane (Section 4 Spec) */}
        <AnimatePresence>
          {showLyrics && (
            <motion.div
              key="lyrics-pane"
              initial={{ opacity: 0, x: 20, width: 0 }}
              animate={{ opacity: 1, x: 0, width: '50%' }}
              exit={{ opacity: 0, x: 20, width: 0 }}
              transition={{ type: 'spring', stiffness: 400, damping: 32 }}
              onPointerDown={(e) => e.stopPropagation()}
              onMouseEnter={() => onScrollTrapChange?.(true)}
              onMouseLeave={() => onScrollTrapChange?.(false)}
              onWheel={(e) => {
                // Scroll Trap: isolate lyrics scrolling vertically, prevent tab flipping!
                e.stopPropagation();
              }}
              className="relative h-full overflow-hidden rounded-2xl bg-white/[0.03] backdrop-blur-md p-3 border border-white/5 flex flex-col"
            >
              <div
                className="flex-1 overflow-y-auto isle-scroll py-4 space-y-2 text-center"
                style={{
                  maskImage:
                    'linear-gradient(to bottom, transparent 0%, black 15%, black 85%, transparent 100%)',
                  WebkitMaskImage:
                    'linear-gradient(to bottom, transparent 0%, black 15%, black 85%, transparent 100%)',
                }}
              >
                {lyrics.lines.length === 0 ? (
                  <p className="text-[11px] text-white/45 py-8">
                    No synced lyrics available
                  </p>
                ) : (
                  lyrics.lines.map((line, idx) => {
                    const isActive = idx === activeIndex;
                    const isPast = activeIndex >= 0 && idx < activeIndex;
                    const displayText = line.text || '• • •';
                    return (
                      <button
                        key={`${line.timeMs}-${idx}`}
                        ref={(el) => {
                          lineRefs.current[idx] = el;
                        }}
                        type="button"
                        onClick={() => onSeek(line.timeMs / 1000)}
                        className={`block w-full text-center transition-all duration-200 cursor-pointer rounded px-1.5 leading-snug ${
                          isActive
                            ? 'text-[12px] font-bold text-white scale-[1.02]'
                            : isPast
                              ? 'text-[10.5px] font-normal text-white/35 hover:text-white/60'
                              : 'text-[10.5px] font-normal text-white/45 hover:text-white/75'
                        }`}
                      >
                        {displayText}
                      </button>
                    );
                  })
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};
