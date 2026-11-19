import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Pause, Play, RotateCcw } from 'lucide-react';
import { ShibaMascot } from '../ShibaMascot';

interface ZenTabProps {
  active: boolean;
  onToggleActive: (next: boolean) => void;
}

const PHASES = [
  { label: 'Inhale', prompt: 'Breathe in slowly...', scale: 1.12, color: '#2DD4BF' },
  { label: 'Hold', prompt: 'Hold gently...', scale: 1.12, color: '#5EC8FF' },
  { label: 'Exhale', prompt: 'Release tension...', scale: 0.72, color: '#818CF8' },
  { label: 'Hold', prompt: 'Rest in stillness...', scale: 0.72, color: '#34D399' },
] as const;

const TOTAL_SESSION_SEC = 60;

export const ZenTab: React.FC<ZenTabProps> = ({ active, onToggleActive }) => {
  const [elapsedSec, setElapsedSec] = useState(0);

  useEffect(() => {
    if (!active) return;
    const id = window.setInterval(() => {
      setElapsedSec((prev) => {
        if (prev + 1 >= TOTAL_SESSION_SEC) {
          onToggleActive(false);
          return 0;
        }
        return prev + 1;
      });
    }, 1000);
    return () => window.clearInterval(id);
  }, [active, onToggleActive]);

  const phaseIdx = Math.floor((elapsedSec % 16) / 4);
  const phaseSecLeft = 4 - (elapsedSec % 4);
  const currentPhase = PHASES[phaseIdx];
  const remainingTotal = TOTAL_SESSION_SEC - elapsedSec;

  const handleReset = () => {
    onToggleActive(false);
    setElapsedSec(0);
  };

  return (
    <div className="w-full h-full px-5 pb-3 flex items-center justify-between gap-4 select-none">
      {/* Left: Soft Ambient Breathing Bloom (Zero Radioactive Glows, 0.15 Opacity) */}
      <div className="relative w-[132px] h-[132px] shrink-0 flex items-center justify-center">
        <motion.div
          className="absolute w-28 h-28 rounded-full blur-2xl opacity-15"
          animate={{
            scale: active ? currentPhase.scale * 1.15 : 0.85,
            backgroundColor: currentPhase.color,
          }}
          transition={{ duration: 3.8, ease: 'easeInOut' }}
        />

        <motion.div
          className="relative w-24 h-24 rounded-full border flex flex-col items-center justify-center bg-black/40 backdrop-blur-md"
          animate={{
            scale: active ? currentPhase.scale : 0.9,
            borderColor: active ? `${currentPhase.color}88` : 'rgba(255,255,255,0.15)',
          }}
          transition={{ duration: 3.8, ease: 'easeInOut' }}
        >
          <span className="text-[13px] font-semibold text-white">
            {active ? currentPhase.label : 'Box 4-4-4-4'}
          </span>
          <span
            className="text-[18px] font-bold tabular-nums mt-0.5"
            style={{ color: currentPhase.color }}
          >
            {active ? `${phaseSecLeft}s` : '1:00'}
          </span>
        </motion.div>
      </div>

      {/* Right: Meditating Shiba + Prompt + Controls */}
      <div
        className="flex-1 flex flex-col justify-between h-[132px] py-1"
        onPointerDown={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between gap-2">
          <div>
            <p className="text-[12.5px] font-semibold text-white">
              Zen Box Breathing
            </p>
            <p className="text-[11px] text-teal-200/75 mt-0.5">
              {active ? currentPhase.prompt : '1-minute guided stress relief'}
            </p>
          </div>

          <div className="px-2 py-1 rounded-2xl bg-white/[0.05] border border-white/[0.08]">
            <ShibaMascot state="zen" size="md" />
          </div>
        </div>

        <div className="flex items-center justify-between text-[10.5px] text-white/50 tabular-nums">
          <span>Session remaining</span>
          <span className="text-white/80 font-semibold">{remainingTotal}s</span>
        </div>

        <div className="flex items-center gap-2">
          <motion.button
            type="button"
            whileHover={{ scale: 1.03 }}
            whileTap={{ scale: 0.94 }}
            onClick={() => onToggleActive(!active)}
            className="flex-1 h-9 rounded-xl font-semibold text-[12px] flex items-center justify-center gap-1.5 text-slate-950 cursor-pointer shadow-sm transition-colors"
            style={{
              backgroundColor: currentPhase.color,
            }}
          >
            {active ? (
              <>
                <Pause className="w-3.5 h-3.5 fill-current" />
                Pause
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5 fill-current" />
                Begin 1m Zen
              </>
            )}
          </motion.button>

          <motion.button
            type="button"
            whileHover={{ scale: 1.06 }}
            whileTap={{ scale: 0.9 }}
            onClick={handleReset}
            className="w-9 h-9 rounded-xl bg-white/[0.08] hover:bg-white/[0.14] border border-white/10 flex items-center justify-center text-white/75 hover:text-white cursor-pointer"
            title="Reset breathing session"
            aria-label="Reset breathing session"
          >
            <RotateCcw className="w-3.5 h-3.5" />
          </motion.button>
        </div>
      </div>
    </div>
  );
};
