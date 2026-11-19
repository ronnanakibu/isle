import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { CheckCircle2, Pause, Play, RotateCcw } from 'lucide-react';
import { FOCUS_PHASE_CONFIG } from '../../constants';
import type { FocusPhase, FocusState } from '../../types';
import { formatTime } from '../../services/lyrics';

interface FocusTabProps {
  focus: FocusState;
  celebrating: boolean;
  onToggle: () => void;
  onReset: () => void;
  onSelectPhase: (phase: FocusPhase) => void;
  onCompleteNow: () => void;
}

const CONFETTI_PARTICLES = [
  { x: -52, y: -38, color: '#5EC8FF', delay: 0 },
  { x: 48, y: -42, color: '#4ADE80', delay: 0.03 },
  { x: -64, y: 12, color: '#FBBF24', delay: 0.05 },
  { x: 62, y: 16, color: '#C084FC', delay: 0.02 },
  { x: -28, y: -54, color: '#FB7185', delay: 0.04 },
  { x: 26, y: -52, color: '#38BDF8', delay: 0.06 },
  { x: -38, y: 42, color: '#4ADE80', delay: 0.03 },
  { x: 40, y: 40, color: '#FBBF24', delay: 0.05 },
];

export const FocusTab: React.FC<FocusTabProps> = ({
  focus,
  celebrating,
  onToggle,
  onReset,
  onSelectPhase,
  onCompleteNow,
}) => {
  const cfg = FOCUS_PHASE_CONFIG[focus.phase];
  const progress =
    focus.totalSec > 0 ? Math.min(1, Math.max(0, 1 - focus.remainingSec / focus.totalSec)) : 0;

  const radius = 46;
  const circumference = 2 * Math.PI * radius;
  const dashOffset = circumference * (1 - progress);

  return (
    <div className="relative w-full h-full px-5 pb-3 flex items-center justify-between gap-4 select-none">
      {/* Confetti burst on completion */}
      <AnimatePresence>
        {celebrating && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center overflow-hidden z-20">
            {CONFETTI_PARTICLES.map((p, idx) => (
              <motion.span
                key={idx}
                className="absolute w-2 h-2 rounded-full"
                style={{ backgroundColor: p.color, boxShadow: `0 0 8px ${p.color}` }}
                initial={{ x: 0, y: 0, scale: 0.4, opacity: 1 }}
                animate={{ x: p.x * 1.6, y: p.y * 1.4, scale: [0.8, 1.2, 0], opacity: [1, 1, 0] }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.85, delay: p.delay, ease: 'easeOut' }}
              />
            ))}
          </div>
        )}
      </AnimatePresence>

      {/* Left: Centered SVG Circular Countdown Ring */}
      <div className="relative w-[116px] h-[116px] shrink-0 flex items-center justify-center">
        <svg className="w-full h-full -rotate-90" viewBox="0 0 112 112">
          {/* Track ring */}
          <circle
            cx="56"
            cy="56"
            r={radius}
            fill="none"
            stroke="rgba(255,255,255,0.1)"
            strokeWidth="7"
          />
          {/* Progress ring */}
          <motion.circle
            cx="56"
            cy="56"
            r={radius}
            fill="none"
            stroke={cfg.color}
            strokeWidth="7"
            strokeLinecap="round"
            strokeDasharray={circumference}
            animate={{ strokeDashoffset: dashOffset }}
            transition={{ duration: 0.45, ease: 'easeOut' }}
            style={{
              filter: `drop-shadow(0 0 8px ${cfg.color}66)`,
            }}
          />
        </svg>

        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-[22px] font-semibold tracking-tight text-white tabular-nums leading-none">
            {formatTime(focus.remainingSec)}
          </span>
          <span
            className="text-[9.5px] font-semibold uppercase tracking-wider mt-1"
            style={{ color: cfg.color }}
          >
            {focus.running ? 'Running' : 'Paused'}
          </span>
        </div>
      </div>

      {/* Right: Phase Selector Pills + Controls */}
      <div
        className="flex-1 flex flex-col justify-between h-[124px] py-0.5"
        onPointerDown={(e) => e.stopPropagation()}
      >
        <div>
          <div className="flex items-center justify-between mb-1.5">
            <span className="text-[11px] font-medium text-white/60">{cfg.label}</span>
            <span className="text-[10px] font-medium text-white/45 tabular-nums">
              {focus.completedSessions} done
            </span>
          </div>

          {/* Quick Phase Selector Pills */}
          <div className="grid grid-cols-3 gap-1 p-1 rounded-xl bg-white/[0.06] border border-white/[0.08]">
            {(['work', 'shortBreak', 'longBreak'] as FocusPhase[]).map((phase) => {
              const active = focus.phase === phase;
              const pCfg = FOCUS_PHASE_CONFIG[phase];
              return (
                <button
                  key={phase}
                  type="button"
                  onClick={() => onSelectPhase(phase)}
                  className={`relative py-1 rounded-lg text-[10.5px] font-medium transition-colors cursor-pointer ${
                    active ? 'text-white' : 'text-white/50 hover:text-white/80'
                  }`}
                >
                  {active && (
                    <motion.span
                      layoutId="focus-phase-pill"
                      className="absolute inset-0 rounded-lg bg-white/14 border border-white/15"
                      transition={{ type: 'spring', stiffness: 420, damping: 30 }}
                    />
                  )}
                  <span className="relative z-10">{pCfg.shortLabel}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-2">
          <motion.button
            type="button"
            whileHover={{ scale: 1.03 }}
            whileTap={{ scale: 0.94 }}
            onClick={onToggle}
            className="flex-1 h-9 rounded-xl font-semibold text-[12px] flex items-center justify-center gap-1.5 text-black shadow-sm cursor-pointer transition-colors"
            style={{
              backgroundColor: cfg.color,
            }}
          >
            {focus.running ? (
              <>
                <Pause className="w-3.5 h-3.5 fill-current" />
                Pause
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5 fill-current" />
                Start
              </>
            )}
          </motion.button>

          <motion.button
            type="button"
            whileHover={{ scale: 1.06 }}
            whileTap={{ scale: 0.9 }}
            onClick={onReset}
            className="w-9 h-9 rounded-xl bg-white/[0.08] hover:bg-white/[0.14] border border-white/10 flex items-center justify-center text-white/75 hover:text-white cursor-pointer"
            title="Reset timer"
            aria-label="Reset timer"
          >
            <RotateCcw className="w-3.5 h-3.5" />
          </motion.button>

          <motion.button
            type="button"
            whileHover={{ scale: 1.06 }}
            whileTap={{ scale: 0.9 }}
            onClick={onCompleteNow}
            className="w-9 h-9 rounded-xl bg-white/[0.08] hover:bg-emerald-400/20 border border-white/10 hover:border-emerald-400/40 flex items-center justify-center text-white/75 hover:text-emerald-300 cursor-pointer"
            title="Complete session & test chime"
            aria-label="Complete session"
          >
            <CheckCircle2 className="w-3.5 h-3.5" />
          </motion.button>
        </div>
      </div>
    </div>
  );
};
