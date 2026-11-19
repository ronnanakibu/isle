import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Droplets, RotateCcw, Undo2 } from 'lucide-react';
import type { WaterState } from '../../types';

interface WaterTabProps {
  water: WaterState;
  onAddWater: (ml: number) => void;
  onUndo: () => void;
  onReset: () => void;
}

const SPLASH_DROPS = [
  { x: -18, y: -24, delay: 0 },
  { x: 0, y: -30, delay: 0.03 },
  { x: 18, y: -22, delay: 0.05 },
  { x: -10, y: -16, delay: 0.02 },
  { x: 12, y: -18, delay: 0.04 },
];

export const WaterTab: React.FC<WaterTabProps> = ({
  water,
  onAddWater,
  onUndo,
  onReset,
}) => {
  const [splashKey, setSplashKey] = useState(0);

  const pct = water.goalMl > 0 ? Math.floor((water.intakeMl / water.goalMl) * 100) : 0;
  const clampedFill = Math.min(100, Math.max(0, pct));
  const goalMet = pct >= 100;

  const handleAdd = (ml: number) => {
    setSplashKey((k) => k + 1);
    onAddWater(ml);
  };

  // Wave vertical offset inside 120px tall SVG vessel
  const waveY = 116 - (clampedFill / 100) * 108;

  return (
    <div className="w-full h-full px-5 pb-3 flex items-center justify-between gap-4 select-none">
      {/* Left: Animated Water Wave Vessel */}
      <div className="relative w-[104px] h-[126px] rounded-[24px] bg-white/[0.05] border border-white/15 overflow-hidden shrink-0 shadow-inner">
        <svg
          viewBox="0 0 104 126"
          className="absolute inset-0 w-full h-full"
          preserveAspectRatio="none"
        >
          <defs>
            <linearGradient id="isle-water-grad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={goalMet ? '#4ADE80' : '#38BDF8'} stopOpacity="0.92" />
              <stop offset="100%" stopColor={goalMet ? '#059669' : '#0284C7'} stopOpacity="0.98" />
            </linearGradient>
          </defs>

          {/* Back wave */}
          <motion.g
            animate={{ y: waveY }}
            transition={{ type: 'spring', stiffness: 180, damping: 22 }}
          >
            <motion.path
              d="M -104 6 Q -78 0 -52 6 T 0 6 T 52 6 T 104 6 T 156 6 T 208 6 L 208 140 L -104 140 Z"
              fill={goalMet ? 'rgba(74, 222, 128, 0.35)' : 'rgba(56, 189, 248, 0.35)'}
              animate={{ x: [0, -104] }}
              transition={{ duration: 4.2, repeat: Infinity, ease: 'linear' }}
            />
            {/* Front wave */}
            <motion.path
              d="M -104 8 Q -78 14 -52 8 T 0 8 T 52 8 T 104 8 T 156 8 T 208 8 L 208 140 L -104 140 Z"
              fill="url(#isle-water-grad)"
              animate={{ x: [-104, 0] }}
              transition={{ duration: 3.1, repeat: Infinity, ease: 'linear' }}
            />
          </motion.g>
        </svg>

        {/* Glass highlight */}
        <div
          className="pointer-events-none absolute inset-0 rounded-[24px]"
          style={{
            background:
              'linear-gradient(115deg, rgba(255,255,255,0.22) 0%, transparent 38%, transparent 82%, rgba(255,255,255,0.08) 100%)',
          }}
        />

        {/* Percentage Overlay */}
        <div className="relative z-10 w-full h-full flex flex-col items-center justify-center">
          <span className="text-[22px] font-bold text-white tabular-nums drop-shadow-[0_2px_8px_rgba(0,0,0,0.55)]">
            {pct}%
          </span>
          <span className="text-[9.5px] font-semibold uppercase tracking-wider text-white/80 drop-shadow">
            {goalMet ? 'Goal Met' : 'Hydrated'}
          </span>
        </div>
      </div>

      {/* Right: Stats & Quick-Add Controls */}
      <div
        className="flex-1 flex flex-col justify-between h-[124px] py-0.5"
        onPointerDown={(e) => e.stopPropagation()}
      >
        <div>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5">
              <Droplets className="w-3.5 h-3.5 text-sky-400" />
              <span className="text-[11.5px] font-medium text-white/65">Daily Hydration</span>
            </div>
            {typeof water.streakDays === 'number' && water.streakDays > 0 && (
              <span className="px-2 py-0.5 rounded-full bg-amber-400/20 border border-amber-400/35 text-[9.5px] font-semibold text-amber-200 flex items-center gap-1">
                🔥 {water.streakDays}d streak
              </span>
            )}
          </div>
          <div className="mt-1 flex items-baseline gap-1.5">
            <span className="text-[18px] font-semibold text-white tabular-nums">
              {water.intakeMl.toLocaleString('en-US')} ml
            </span>
            <span className="text-[11.5px] text-white/45 tabular-nums">
              / {water.goalMl.toLocaleString('en-US')} ml · {pct}%
            </span>
          </div>
        </div>

        {/* Quick-Add Buttons with Splash Micro-Animation */}
        <div className="relative flex items-center gap-2">
          <AnimatePresence>
            {splashKey > 0 && (
              <div
                key={splashKey}
                className="pointer-events-none absolute left-12 top-0 z-20"
              >
                {SPLASH_DROPS.map((drop, idx) => (
                  <motion.span
                    key={idx}
                    className="absolute w-1.5 h-1.5 rounded-full bg-sky-300"
                    initial={{ x: 0, y: 0, scale: 0.6, opacity: 1 }}
                    animate={{ x: drop.x, y: drop.y, scale: [1, 0.4], opacity: [1, 0] }}
                    transition={{ duration: 0.48, delay: drop.delay, ease: 'easeOut' }}
                  />
                ))}
              </div>
            )}
          </AnimatePresence>

          <motion.button
            type="button"
            whileHover={{ scale: 1.03 }}
            whileTap={{ scale: 0.92 }}
            onClick={() => handleAdd(250)}
            className="flex-1 h-9 rounded-xl bg-sky-400/20 hover:bg-sky-400/30 border border-sky-400/35 text-sky-200 text-[11.5px] font-semibold flex items-center justify-center gap-1 cursor-pointer transition-colors"
          >
            +250 ml
            <span className="text-[10px] font-normal text-sky-200/70">Glass</span>
          </motion.button>

          <motion.button
            type="button"
            whileHover={{ scale: 1.03 }}
            whileTap={{ scale: 0.92 }}
            onClick={() => handleAdd(500)}
            className="flex-1 h-9 rounded-xl bg-sky-500 hover:bg-sky-400 text-slate-950 text-[11.5px] font-semibold flex items-center justify-center gap-1 shadow-sm cursor-pointer transition-colors"
          >
            +500 ml
            <span className="text-[10px] font-medium text-slate-900/75">Bottle</span>
          </motion.button>
        </div>

        {/* Undo & Reset */}
        <div className="flex items-center justify-between text-[10.5px] text-white/50">
          <button
            type="button"
            onClick={onUndo}
            disabled={water.intakeMl === 0}
            className="inline-flex items-center gap-1 hover:text-white disabled:opacity-35 disabled:pointer-events-none transition-colors cursor-pointer"
          >
            <Undo2 className="w-3 h-3" />
            Undo last
          </button>
          <button
            type="button"
            onClick={onReset}
            disabled={water.intakeMl === 0}
            className="inline-flex items-center gap-1 hover:text-rose-300 disabled:opacity-35 disabled:pointer-events-none transition-colors cursor-pointer"
          >
            <RotateCcw className="w-3 h-3" />
            Reset today
          </button>
        </div>
      </div>
    </div>
  );
};
