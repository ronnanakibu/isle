import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { audioEngine } from '../utils/audioEngine';

interface EqualizerProps {
  playing: boolean;
  bpm: number;
  color?: string;
  size?: 'sm' | 'md';
}

export const Equalizer: React.FC<EqualizerProps> = ({
  playing,
  bpm,
  color = '#5EC8FF',
  size = 'sm',
}) => {
  const [bands, setBands] = useState<[number, number, number]>([0.35, 0.7, 0.45]);

  useEffect(() => {
    if (!playing) {
      setBands([0.22, 0.22, 0.22]);
      return;
    }

    let rafId = 0;
    let lastUpdate = 0;

    const tick = (now: number) => {
      if (now - lastUpdate >= 75) {
        lastUpdate = now;
        setBands(audioEngine.sampleBands(now / 1000, bpm));
      }
      rafId = window.requestAnimationFrame(tick);
    };

    rafId = window.requestAnimationFrame(tick);
    return () => window.cancelAnimationFrame(rafId);
  }, [playing, bpm]);

  const maxH = size === 'sm' ? 13 : 16;
  const barW = size === 'sm' ? 2.5 : 3;

  return (
    <div
      className="flex items-center justify-center gap-[2.5px]"
      style={{ height: maxH }}
      aria-label={playing ? 'Audio playing' : 'Audio paused'}
    >
      {bands.map((level, idx) => (
        <motion.span
          key={idx}
          className="rounded-full origin-center"
          style={{
            width: barW,
            height: maxH,
            backgroundColor: playing ? color : 'rgba(255,255,255,0.35)',
          }}
          animate={{
            scaleY: playing ? level : 0.24,
            opacity: playing ? 0.95 : 0.5,
          }}
          transition={{
            type: 'spring',
            stiffness: 420,
            damping: 24,
            mass: 0.4,
          }}
        />
      ))}
    </div>
  );
};
