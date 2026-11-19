import React, { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence, useMotionValue, useSpring } from 'framer-motion';
import type { CliAgentStatus, PresenceLevel, ShibaState } from '../types';

interface ShibaMascotProps {
  state: ShibaState;
  presenceLevel?: PresenceLevel;
  isIslandHovered?: boolean;
  deepWorkZen?: boolean;
  dnd?: boolean;
  micMuted?: boolean;
  agentState?: CliAgentStatus;
  size?: 'sm' | 'md' | 'lg';
  onPet?: () => void;
  onToggleMicMute?: () => void;
}

const MAX_PUPIL_TRAVEL = 1.35;
const GAZE_STRENGTH = 0.014;

export const ShibaMascot: React.FC<ShibaMascotProps> = ({
  state,
  presenceLevel = 'polite',
  isIslandHovered = false,
  deepWorkZen = false,
  dnd = false,
  micMuted = false,
  agentState = 'idle',
  size = 'sm',
  onPet,
  onToggleMicMute,
}) => {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const [isBlinking, setIsBlinking] = useState(false);
  const [petted, setPetted] = useState(false);

  const targetX = useMotionValue(0);
  const targetY = useMotionValue(0);
  const pupilX = useSpring(targetX, { stiffness: 320, damping: 24, mass: 0.5 });
  const pupilY = useSpring(targetY, { stiffness: 320, damping: 24, mass: 0.5 });

  // Determine whether animations are allowed under Attention & Etiquette Engine
  const allowMotion =
    !dnd &&
    !deepWorkZen &&
    presenceLevel !== 'stealth' &&
    (presenceLevel === 'playful' || isIslandHovered || petted || agentState !== 'idle');

  // Derive effective emotional state based on priorities:
  // petted > agent error > agent complete > lowBattery > charging > music > zen > normal
  const effectiveState: ShibaState = petted
    ? 'happy'
    : agentState === 'error'
      ? 'concerned'
      : agentState === 'complete'
        ? 'happy'
        : deepWorkZen
          ? 'zen'
          : state;

  const isDroopy = effectiveState === 'lowBattery' || effectiveState === 'concerned';
  const isVibing = (effectiveState === 'music' || agentState === 'complete') && allowMotion;
  const isCharging = effectiveState === 'charging';
  const isHappy = effectiveState === 'happy';
  const isZen = effectiveState === 'zen';
  const isThinking = agentState === 'thinking';

  // Cursor gaze tracking for normal dot eyes
  useEffect(() => {
    if (
      effectiveState !== 'normal' ||
      deepWorkZen ||
      dnd ||
      presenceLevel === 'stealth' ||
      isThinking
    ) {
      targetX.set(0);
      targetY.set(0);
      return;
    }

    const handleMouseMove = (e: MouseEvent) => {
      if (presenceLevel === 'polite' && !isIslandHovered) {
        targetX.set(0);
        targetY.set(0);
        return;
      }
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

    window.addEventListener('mousemove', handleMouseMove, { passive: true });
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, [effectiveState, deepWorkZen, dnd, presenceLevel, isIslandHovered, isThinking, targetX, targetY]);

  // Procedural blinking every 3.2-5.2s when awake
  useEffect(() => {
    if (
      effectiveState !== 'normal' ||
      deepWorkZen ||
      dnd ||
      presenceLevel === 'stealth' ||
      isThinking
    ) {
      setIsBlinking(false);
      return;
    }
    let t1: number;
    let t2: number;
    const schedule = () => {
      t1 = window.setTimeout(() => {
        setIsBlinking(true);
        t2 = window.setTimeout(() => {
          setIsBlinking(false);
          schedule();
        }, 130);
      }, 3200 + Math.random() * 2000);
    };
    schedule();
    return () => {
      window.clearTimeout(t1);
      window.clearTimeout(t2);
    };
  }, [effectiveState, deepWorkZen, dnd, presenceLevel, isThinking]);

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (dnd) return;
    if (micMuted && onToggleMicMute) {
      onToggleMicMute();
      return;
    }
    setPetted(true);
    window.setTimeout(() => setPetted(false), 1600);
    onPet?.();
  };

  const dims =
    size === 'lg'
      ? { w: 68, h: 56 }
      : size === 'md'
        ? { w: 42, h: 34 }
        : { w: 34, h: 28 };

  const furColor = deepWorkZen ? '#9CA3AF' : '#E89B48';
  const creamColor = deepWorkZen ? '#E5E7EB' : '#FFF3DC';
  const innerEarColor = deepWorkZen ? '#D1D5DB' : '#FDE4C0';

  return (
    <motion.div
      onClick={handleClick}
      className="relative inline-flex items-center justify-center shrink-0 cursor-pointer select-none"
      animate={
        isThinking
          ? { y: [0, -1, 0] }
          : isVibing
            ? { y: [0, -1.5, 0] }
            : isHappy
              ? { scale: [1, 1.08, 1], y: [0, -2, 0] }
              : { y: 0, scale: 1 }
      }
      transition={
        isThinking
          ? { duration: 1.4, repeat: Infinity, ease: 'easeInOut' }
          : isVibing
            ? { duration: 0.65, repeat: Infinity, ease: 'easeInOut' }
            : isHappy
              ? { duration: 0.45 }
              : { duration: 0.3 }
      }
      title={
        micMuted
          ? 'Microphone is muted! Click to unmute'
          : isThinking
            ? 'Claude Code is thinking / executing tools...'
            : deepWorkZen
              ? 'Shiba in Deep Work Zen Mode'
              : 'Click to pet Shiba'
      }
      aria-label={`Shiba mascot (${effectiveState}, mic: ${micMuted ? 'muted' : 'live'})`}
    >
      {/* Floating Music Notes when Music Vibing */}
      <AnimatePresence>
        {isVibing && !isThinking && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="pointer-events-none absolute -top-1.5 inset-x-0 flex justify-between px-0.5 text-[8px] text-sky-300 font-bold"
          >
            <motion.span
              animate={{ y: [0, -3, 0], opacity: [0.6, 1, 0.6] }}
              transition={{ duration: 1.1, repeat: Infinity }}
            >
              ♪
            </motion.span>
            <motion.span
              animate={{ y: [-1, -4, -1], opacity: [0.5, 1, 0.5] }}
              transition={{ duration: 1.3, repeat: Infinity, delay: 0.3 }}
            >
              ♫
            </motion.span>
          </motion.div>
        )}
      </AnimatePresence>

      <svg
        ref={svgRef}
        width={dims.w}
        height={dims.h}
        viewBox="0 0 52 42"
        fill="none"
        className="overflow-visible"
      >
        <defs>
          <radialGradient id="shiba-blush" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#FB7185" stopOpacity="0.75" />
            <stop offset="100%" stopColor="#FB7185" stopOpacity="0" />
          </radialGradient>
          <linearGradient id="hud-visor-grad" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#06B6D4" stopOpacity="0.85" />
            <stop offset="50%" stopColor="#38BDF8" stopOpacity="0.95" />
            <stop offset="100%" stopColor="#06B6D4" stopOpacity="0.85" />
          </linearGradient>
        </defs>

        {/* Left Ear */}
        <motion.g
          style={{ transformOrigin: '15px 15px' }}
          animate={
            isDroopy
              ? { rotate: -48, x: -2, y: 4 }
              : isVibing || isHappy
                ? { rotate: [-8, 6, -8], y: -1 }
                : isCharging
                  ? { rotate: 2, y: -1.2, scaleY: 1.06 }
                  : { rotate: 0, x: 0, y: 0, scaleY: 1 }
          }
          transition={
            isVibing || isHappy
              ? { duration: 0.55, repeat: isVibing ? Infinity : 2, ease: 'easeInOut' }
              : { type: 'spring', stiffness: 340, damping: 22 }
          }
        >
          <path
            d="M 10 18 L 11.5 4.5 Q 12.5 2 15 4.2 L 21 13.5 Z"
            fill={furColor}
            stroke="#0A0B0F"
            strokeWidth="1.6"
            strokeLinejoin="round"
          />
          <path d="M 12.5 15.5 L 13.4 7.2 L 18.2 13.2 Z" fill={innerEarColor} />
        </motion.g>

        {/* Right Ear */}
        <motion.g
          style={{ transformOrigin: '37px 15px' }}
          animate={
            isDroopy
              ? { rotate: 48, x: 2, y: 4 }
              : isVibing || isHappy
                ? { rotate: [8, -6, 8], y: -1 }
                : isCharging
                  ? { rotate: -2, y: -1.2, scaleY: 1.06 }
                  : { rotate: 0, x: 0, y: 0, scaleY: 1 }
          }
          transition={
            isVibing || isHappy
              ? { duration: 0.55, repeat: isVibing ? Infinity : 2, ease: 'easeInOut' }
              : { type: 'spring', stiffness: 340, damping: 22 }
          }
        >
          <path
            d="M 42 18 L 40.5 4.5 Q 39.5 2 37 4.2 L 31 13.5 Z"
            fill={furColor}
            stroke="#0A0B0F"
            strokeWidth="1.6"
            strokeLinejoin="round"
          />
          <path d="M 39.5 15.5 L 38.6 7.2 L 33.8 13.2 Z" fill={innerEarColor} />
        </motion.g>

        {/* Head Base */}
        <ellipse
          cx="26"
          cy="24.5"
          rx="15.5"
          ry="12.5"
          fill={furColor}
          stroke="#0A0B0F"
          strokeWidth="1.6"
        />

        {/* Classic Shiba Cream Cheeks & Muzzle */}
        <path
          d="M 11.2 25.5 C 11.2 21.5, 16.5 20.5, 21.5 23.2 C 23.8 24.4, 24.8 25.8, 26 25.8 C 27.2 25.8, 28.2 24.4, 30.5 23.2 C 35.5 20.5, 40.8 21.5, 40.8 25.5 C 40.8 32.5, 34.5 36.6, 26 36.6 C 17.5 36.6, 11.2 32.5, 11.2 25.5 Z"
          fill={creamColor}
        />

        {/* Cream Eyebrow Dots */}
        <motion.ellipse
          cx="19.5"
          cy="18.2"
          rx="2.1"
          ry="1.2"
          fill={creamColor}
          animate={isDroopy ? { rotate: -22, y: 0.6 } : { rotate: -8, y: 0 }}
          style={{ transformOrigin: '19.5px 18.2px' }}
        />
        <motion.ellipse
          cx="32.5"
          cy="18.2"
          rx="2.1"
          ry="1.2"
          fill={creamColor}
          animate={isDroopy ? { rotate: 22, y: 0.6 } : { rotate: 8, y: 0 }}
          style={{ transformOrigin: '32.5px 18.2px' }}
        />

        {/* Rosy Blushing Cheeks (Happy / Petted / Complete) */}
        {(isHappy || agentState === 'complete') && (
          <g>
            <ellipse cx="15.5" cy="27.5" rx="4.2" ry="2.6" fill="url(#shiba-blush)" />
            <ellipse cx="36.5" cy="27.5" rx="4.2" ry="2.6" fill="url(#shiba-blush)" />
          </g>
        )}

        {/* Eyes by State */}
        {isZen ? (
          /* Zen / Deep Work: Peaceful closed eyes (- ᴥ -) */
          <g stroke="#1A1614" strokeWidth="1.8" strokeLinecap="round">
            <path d="M 17 23.2 Q 19.5 24.4 22 23.2" />
            <path d="M 30 23.2 Q 32.5 24.4 35 23.2" />
          </g>
        ) : isThinking ? (
          /* Programmer Visor / Thinking Eyes */
          <g>
            <circle cx="19.5" cy="23" r="1.8" fill="#0284C7" />
            <circle cx="32.5" cy="23" r="1.8" fill="#0284C7" />
          </g>
        ) : isVibing || effectiveState === 'music' ? (
          /* Music Vibing: Happy crescent eyes (^ ᴥ ^) */
          <g stroke="#1A1614" strokeWidth="1.9" strokeLinecap="round" fill="none">
            <path d="M 17 24 Q 19.5 20.5 22 24" />
            <path d="M 30 24 Q 32.5 20.5 35 24" />
          </g>
        ) : isCharging ? (
          /* Charging: Sparkling 5-point star eyes (★ ᴥ ★) */
          <g fill="#1A1614">
            <path d="M 19.5 19.8 L 20.4 21.8 L 22.6 22.1 L 21 23.6 L 21.4 25.8 L 19.5 24.7 L 17.6 25.8 L 18 23.6 L 16.4 22.1 L 18.6 21.8 Z" />
            <path d="M 32.5 19.8 L 33.4 21.8 L 35.6 22.1 L 34 23.6 L 34.4 25.8 L 32.5 24.7 L 30.6 25.8 L 31 23.6 L 29.4 22.1 L 31.6 21.8 Z" />
          </g>
        ) : isHappy ? (
          /* Happy / Petted: Left dot eye + Right winking eye */
          <g>
            <circle cx="19.5" cy="23" r="2.1" fill="#1A1614" />
            <circle cx="18.8" cy="22.2" r="0.7" fill="#FFFFFF" />
            <path
              d="M 30 21.5 L 34.2 23.2 L 30 24.8"
              stroke="#1A1614"
              strokeWidth="1.9"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
            />
          </g>
        ) : isDroopy ? (
          /* Low Battery / Error: Sad puppy eyes with glossy catchlights (( ɵ̥̥ ︵ ɵ̥̥ )) */
          <g>
            <circle cx="19.5" cy="23.4" r="2.35" fill="#1A1614" />
            <circle cx="18.7" cy="22.5" r="0.95" fill="#FFFFFF" />
            <circle cx="20.3" cy="24.2" r="0.45" fill="#FFFFFF" />
            <circle cx="32.5" cy="23.4" r="2.35" fill="#1A1614" />
            <circle cx="31.7" cy="22.5" r="0.95" fill="#FFFFFF" />
            <circle cx="33.3" cy="24.2" r="0.45" fill="#FFFFFF" />
          </g>
        ) : (
          /* Normal / Idle: Dot eyes with cursor gaze tracking & procedural blink */
          <g>
            {[19.5, 32.5].map((cx) => (
              <motion.g
                key={cx}
                style={{ transformOrigin: `${cx}px 23px` }}
                animate={{ scaleY: isBlinking ? 0.1 : 1 }}
                transition={{ duration: 0.08 }}
              >
                <motion.g style={{ x: pupilX, y: pupilY }}>
                  <circle cx={cx} cy={23} r="2.1" fill="#1A1614" />
                  <circle cx={cx - 0.65} cy={22.2} r="0.68" fill="#FFFFFF" />
                </motion.g>
              </motion.g>
            ))}
          </g>
        )}

        {/* Nose */}
        <ellipse cx="26" cy="27.2" rx="2.1" ry="1.45" fill="#1A1614" />

        {/* Mouth & Tongue / Mic Mute Tape Cross */}
        {micMuted ? (
          /* Section 5.4: Shiba Mic-Mute Visual State (Tape Cross · 🗙 ·) */
          <g style={{ transformOrigin: '26px 30.5px' }}>
            {/* White cross-tape strips */}
            <line
              x1="22.5"
              y1="28"
              x2="29.5"
              y2="33"
              stroke="#FFFFFF"
              strokeWidth="2.4"
              strokeLinecap="round"
            />
            <line
              x1="22.5"
              y1="28"
              x2="29.5"
              y2="33"
              stroke="#D1D5DB"
              strokeWidth="0.8"
              strokeLinecap="round"
            />
            <line
              x1="22.5"
              y1="33"
              x2="29.5"
              y2="28"
              stroke="#FFFFFF"
              strokeWidth="2.4"
              strokeLinecap="round"
            />
            <line
              x1="22.5"
              y1="33"
              x2="29.5"
              y2="28"
              stroke="#D1D5DB"
              strokeWidth="0.8"
              strokeLinecap="round"
            />
          </g>
        ) : isDroopy ? (
          /* Sad puppy frown (( ɵ̥̥ ︵ ɵ̥̥ )) */
          <path
            d="M 23.2 31.4 Q 26 29.2 28.8 31.4"
            stroke="#1A1614"
            strokeWidth="1.5"
            strokeLinecap="round"
            fill="none"
          />
        ) : (
          <g>
            {/* Cute pink tongue when Happy / Complete */}
            {(isHappy || agentState === 'complete') && (
              <motion.path
                initial={{ scaleY: 0.3, opacity: 0 }}
                animate={{ scaleY: 1, opacity: 1 }}
                d="M 24.2 30.2 L 27.8 30.2 L 27.8 32.8 A 1.8 1.8 0 0 1 24.2 32.8 Z"
                fill="#FB7185"
                stroke="#1A1614"
                strokeWidth="0.9"
              />
            )}
            {/* Classic Shiba ω snout */}
            <path
              d="M 22.2 29.6 Q 24.1 31.6 26 29.4 Q 27.9 31.6 29.8 29.6"
              stroke="#1A1614"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
            />
          </g>
        )}

        {/* Section 1.1: Cute Programmer Glasses / HUD Visor when Thinking */}
        {isThinking && (
          <motion.g
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0 }}
            transition={{ type: 'spring', stiffness: 420, damping: 25 }}
          >
            {/* Left Frame */}
            <rect
              x="14"
              y="18.5"
              width="10.5"
              height="8.5"
              rx="2.5"
              fill="rgba(6, 182, 212, 0.22)"
              stroke="#22D3EE"
              strokeWidth="1.4"
            />
            {/* Right Frame */}
            <rect
              x="27.5"
              y="18.5"
              width="10.5"
              height="8.5"
              rx="2.5"
              fill="rgba(6, 182, 212, 0.22)"
              stroke="#22D3EE"
              strokeWidth="1.4"
            />
            {/* Visor Bridge */}
            <line
              x1="24.5"
              y1="21.5"
              x2="27.5"
              y2="21.5"
              stroke="#22D3EE"
              strokeWidth="1.5"
            />
            {/* Temple arms */}
            <line
              x1="14"
              y1="21"
              x2="11.5"
              y2="20"
              stroke="#22D3EE"
              strokeWidth="1.2"
            />
            <line
              x1="38"
              y1="21"
              x2="40.5"
              y2="20"
              stroke="#22D3EE"
              strokeWidth="1.2"
            />
            {/* Subtle glow reflection line */}
            <motion.line
              x1="16"
              y1="20"
              x2="22"
              y2="20"
              stroke="#FFFFFF"
              strokeWidth="1"
              strokeLinecap="round"
              opacity="0.8"
              animate={{ opacity: [0.4, 0.9, 0.4] }}
              transition={{ duration: 1.6, repeat: Infinity }}
            />
          </motion.g>
        )}

        {/* Charging Green Power Bolt Badge */}
        {isCharging && (
          <motion.path
            initial={{ scale: 0.6, opacity: 0 }}
            animate={{ scale: [1, 1.15, 1], opacity: 1 }}
            transition={{ duration: 1.2, repeat: Infinity }}
            d="M 46.5 18 L 43.2 23.5 L 45.8 23.5 L 44.5 28.5 L 48.8 22.2 L 46 22.2 Z"
            fill="#4ADE80"
            style={{ filter: 'drop-shadow(0 0 4px rgba(74,222,128,0.8))' }}
          />
        )}

        {/* Low Battery Warning Icon */}
        {isDroopy && !micMuted && (
          <g transform="translate(43, 21)">
            <rect
              x="0"
              y="0"
              width="7"
              height="4.2"
              rx="1"
              stroke="#F87171"
              strokeWidth="0.9"
              fill="rgba(248,113,113,0.15)"
            />
            <rect x="1" y="1" width="1.6" height="2.2" rx="0.4" fill="#F87171" />
            <rect x="7.2" y="1.2" width="0.9" height="1.8" rx="0.3" fill="#F87171" />
          </g>
        )}
      </svg>
    </motion.div>
  );
};
