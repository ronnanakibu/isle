import React, { useEffect, useRef, useState } from 'react';
import { motion, useMotionValue, useSpring } from 'framer-motion';
import type { MascotMood } from '../types';

interface MascotProps {
  mood: MascotMood;
  size?: 'sm' | 'md';
  accentColor?: string;
  onPoke?: () => void;
}

const MAX_PUPIL_TRAVEL = 2.4;
const GAZE_STRENGTH = 0.022;

export const Mascot: React.FC<MascotProps> = ({
  mood,
  size = 'sm',
  accentColor = '#5EC8FF',
  onPoke,
}) => {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const [isBlinking, setIsBlinking] = useState(false);
  const [isSleepy, setIsSleepy] = useState(false);
  const [pokedHappy, setPokedHappy] = useState(false);

  const targetX = useMotionValue(0);
  const targetY = useMotionValue(0);
  const pupilX = useSpring(targetX, { stiffness: 320, damping: 24, mass: 0.5 });
  const pupilY = useSpring(targetY, { stiffness: 320, damping: 24, mass: 0.5 });

  // Track cursor across the viewport with leashed eye-socket clamping
  useEffect(() => {
    let idleTimer: number | null = null;

    const resetIdle = () => {
      setIsSleepy(false);
      if (idleTimer !== null) window.clearTimeout(idleTimer);
      idleTimer = window.setTimeout(() => {
        setIsSleepy(true);
        targetX.set(0);
        targetY.set(0.4);
      }, 12000);
    };

    const handleMouseMove = (e: MouseEvent) => {
      resetIdle();
      const el = svgRef.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const dx = e.clientX - cx;
      const dy = e.clientY - cy;
      const dist = Math.hypot(dx, dy);
      if (dist > 0.5) {
        const travel = Math.min(dist * GAZE_STRENGTH, MAX_PUPIL_TRAVEL);
        targetX.set((dx / dist) * travel);
        targetY.set((dy / dist) * travel);
      }
    };

    const handleMouseLeave = () => {
      targetX.set(0);
      targetY.set(0);
    };

    resetIdle();
    window.addEventListener('mousemove', handleMouseMove, { passive: true });
    document.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      if (idleTimer !== null) window.clearTimeout(idleTimer);
      window.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [targetX, targetY]);

  // Procedural blinking every 3-5 seconds (with occasional double-blink)
  useEffect(() => {
    let timeoutId: number;
    let innerId: number;

    const scheduleBlink = () => {
      const delay = 3000 + Math.random() * 2000;
      timeoutId = window.setTimeout(() => {
        setIsBlinking(true);
        innerId = window.setTimeout(() => {
          setIsBlinking(false);
          // 18% chance of a quick double-blink
          if (Math.random() < 0.18) {
            innerId = window.setTimeout(() => {
              setIsBlinking(true);
              innerId = window.setTimeout(() => {
                setIsBlinking(false);
                scheduleBlink();
              }, 110);
            }, 130);
          } else {
            scheduleBlink();
          }
        }, 135);
      }, delay);
    };

    scheduleBlink();
    return () => {
      window.clearTimeout(timeoutId);
      window.clearTimeout(innerId);
    };
  }, []);

  const effectiveMood: MascotMood = pokedHappy
    ? 'happy'
    : mood === 'happy' || mood === 'grooving'
      ? mood
      : isSleepy
        ? 'sleepy'
        : mood;

  const width = size === 'sm' ? 28 : 32;
  const height = size === 'sm' ? 16 : 18;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setPokedHappy(true);
    window.setTimeout(() => setPokedHappy(false), 1100);
    onPoke?.();
  };

  const eyeCenters = [10, 22];

  return (
    <motion.svg
      ref={svgRef}
      width={width}
      height={height}
      viewBox="0 0 32 18"
      fill="none"
      onClick={handleClick}
      className="cursor-pointer shrink-0 overflow-visible"
      animate={
        effectiveMood === 'grooving'
          ? { y: [0, -0.9, 0] }
          : effectiveMood === 'happy'
            ? { scale: [1, 1.08, 1] }
            : { y: 0, scale: 1 }
      }
      transition={
        effectiveMood === 'grooving'
          ? { duration: 0.9, repeat: Infinity, ease: 'easeInOut' }
          : { duration: 0.35 }
      }
      aria-label="Isle living mascot"
    >
      {effectiveMood === 'happy' ? (
        // Cheerful crescent eyes (^ ^)
        <g>
          {eyeCenters.map((cx) => (
            <motion.path
              key={cx}
              d={`M ${cx - 3.6} 10.2 Q ${cx} 5.2 ${cx + 3.6} 10.2`}
              stroke={accentColor}
              strokeWidth="2.1"
              strokeLinecap="round"
              fill="none"
              initial={{ pathLength: 0.4, opacity: 0.6 }}
              animate={{ pathLength: 1, opacity: 1 }}
              transition={{ duration: 0.25 }}
              style={{ filter: `drop-shadow(0 0 4px ${accentColor}88)` }}
            />
          ))}
        </g>
      ) : (
        // Glassy dome eyes with leashed pupils & specular highlights
        <g>
          {eyeCenters.map((cx) => {
            const aperture = isBlinking ? 0.08 : effectiveMood === 'sleepy' ? 0.48 : 1;
            return (
              <motion.g
                key={cx}
                style={{ transformOrigin: `${cx}px 9px` }}
                animate={{ scaleY: aperture }}
                transition={{
                  duration: isBlinking ? 0.075 : 0.14,
                  ease: 'easeOut',
                }}
              >
                {/* Eye sclera dome */}
                <circle
                  cx={cx}
                  cy={9}
                  r={4.8}
                  fill="#EEF2F8"
                  stroke={effectiveMood === 'grooving' ? `${accentColor}55` : 'rgba(255,255,255,0.15)'}
                  strokeWidth="0.5"
                />
                {/* Tracking pupil + specular highlights */}
                <motion.g style={{ x: pupilX, y: pupilY }}>
                  <circle cx={cx} cy={9} r={2.25} fill="#0A0B0E" />
                  {/* Upper-left primary specular highlight */}
                  <circle cx={cx - 0.9} cy={7.9} r={0.85} fill="rgba(255,255,255,0.92)" />
                  {/* Lower-right subtle bounce highlight */}
                  <circle cx={cx + 0.85} cy={9.95} r={0.42} fill="rgba(255,255,255,0.42)" />
                </motion.g>
              </motion.g>
            );
          })}
        </g>
      )}
    </motion.svg>
  );
};
