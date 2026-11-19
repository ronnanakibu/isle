import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  ArrowDownToLine,
  Battery,
  BatteryCharging,
  BatteryWarning,
  CalendarClock,
  Camera,
  CheckCircle2,
  Droplets,
  Eye,
  Headphones,
  Heart,
  Layers,
  Maximize2,
  Mic,
  MicOff,
  Monitor,
  Moon,
  MousePointerClick,
  Music2,
  PanelTop,
  Printer,
  Settings,
  Smartphone,
  Sparkles,
  Sun,
  Volume2,
  Wifi,
  XCircle,
} from 'lucide-react';
import type {
  CliAgentActivity,
  CliAgentStatus,
  ExpandTrigger,
  IslandVisualState,
  MountMode,
  PeripheralType,
  SensorAccessState,
  SensorType,
  ShibaState,
  WallpaperId,
} from '../types';
import { CliAgentLogo } from './CliAgentLogo';

interface DesktopCanvasProps {
  wallpaper: WallpaperId;
  mountMode: MountMode;
  expandTrigger: ExpandTrigger;
  visualState: IslandVisualState;
  expanded: boolean;
  simulatedFullscreen: boolean;
  deepWorkActive: boolean;
  moodOverride: ShibaState | null;
  charging: boolean;
  batteryPct: number;
  micMuted: boolean;
  cliAgent: CliAgentActivity;
  sensorAccess?: SensorAccessState | null;
  autoHideOnScreenshot?: boolean;
  showDevSimulator?: boolean;
  onChangeWallpaper: (w: WallpaperId) => void;
  onChangeMountMode: (m: MountMode) => void;
  onChangeExpandTrigger: (t: ExpandTrigger) => void;
  onTriggerStatePreset: (preset: IslandVisualState) => void;
  onToggleSimulatedFullscreen: () => void;
  onOpenSettings: () => void;
  // Shiba & HUD Simulator triggers
  onSetMoodOverride: (mood: ShibaState | null) => void;
  onToggleLowBattery: () => void;
  onToggleCharging: () => void;
  onTriggerBatteryFull: () => void;
  onPetShiba: () => void;
  onToggleDeepWork: () => void;
  onTriggerVolumeHud: (overscroll?: 'min' | 'max') => void;
  onTriggerBrightnessHud: (overscroll?: 'min' | 'max') => void;
  onTriggerCapsLock: () => void;
  onToggleMicMute: () => void;
  onTriggerDownloadRadar: () => void;
  onTriggerCalendarRadar: () => void;
  onTriggerHydrationNudge: () => void;
  onTriggerEyeRestNudge: () => void;
  // Section 1: CLI Agent Simulator Triggers
  onSetCliAgentStatus: (status: CliAgentStatus, agentName?: string) => void;
  // Section 1.3: Dynamic Sensor Privacy Simulator
  onTriggerSensorAccess?: (type: SensorType, app?: string) => void;
  onClearSensorAccess?: () => void;
  // Section 2: Screenshot PrtSc
  onToggleAutoHideScreenshot?: () => void;
  onTriggerPeripheral: (type: PeripheralType) => void;
  onTriggerScreenshot: () => void;
}

export const DesktopCanvas: React.FC<DesktopCanvasProps> = ({
  wallpaper,
  mountMode,
  expandTrigger,
  visualState,
  expanded,
  simulatedFullscreen,
  deepWorkActive,
  moodOverride,
  charging,
  batteryPct,
  micMuted,
  cliAgent,
  sensorAccess,
  autoHideOnScreenshot = true,
  showDevSimulator = false,
  onChangeWallpaper,
  onChangeMountMode,
  onChangeExpandTrigger,
  onTriggerStatePreset,
  onToggleSimulatedFullscreen,
  onOpenSettings,
  onSetMoodOverride,
  onToggleLowBattery,
  onToggleCharging,
  onTriggerBatteryFull,
  onPetShiba,
  onToggleDeepWork,
  onTriggerVolumeHud,
  onTriggerBrightnessHud,
  onTriggerCapsLock,
  onToggleMicMute,
  onTriggerDownloadRadar,
  onTriggerCalendarRadar,
  onTriggerHydrationNudge,
  onTriggerEyeRestNudge,
  onSetCliAgentStatus,
  onTriggerSensorAccess,
  onClearSensorAccess,
  onToggleAutoHideScreenshot,
  onTriggerPeripheral,
  onTriggerScreenshot,
}) => {
  const currentState: IslandVisualState = expanded ? 'expanded' : visualState;

  return (
    <div className="relative w-full h-full overflow-hidden">
      {/* Wallpaper Layers */}
      <AnimatePresence mode="wait">
        {wallpaper === 'sonoma' && (
          <motion.div
            key="sonoma"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.45 }}
            className="absolute inset-0"
            style={{
              background:
                'radial-gradient(120% 90% at 50% 15%, #134E5E 0%, #0F2942 42%, #1B1438 75%, #0B0D17 100%)',
            }}
          >
            <div
              className="absolute inset-0 opacity-85"
              style={{
                background:
                  'radial-gradient(65% 45% at 72% 38%, rgba(245, 158, 11, 0.42) 0%, rgba(234, 88, 12, 0.18) 45%, transparent 75%), radial-gradient(70% 55% at 24% 62%, rgba(20, 184, 166, 0.45) 0%, rgba(14, 116, 144, 0.2) 50%, transparent 80%)',
              }}
            />
            <svg
              viewBox="0 0 1440 900"
              preserveAspectRatio="xMidYMid slice"
              className="absolute inset-0 w-full h-full opacity-75"
            >
              <defs>
                <linearGradient id="sonoma-hill-1" x1="0" y1="0" x2="1" y2="1">
                  <stop offset="0%" stopColor="#0D9488" stopOpacity="0.55" />
                  <stop offset="55%" stopColor="#0369A1" stopOpacity="0.45" />
                  <stop offset="100%" stopColor="#1E1B4B" stopOpacity="0.85" />
                </linearGradient>
                <linearGradient id="sonoma-hill-2" x1="0" y1="0" x2="1" y2="0.6">
                  <stop offset="0%" stopColor="#F59E0B" stopOpacity="0.38" />
                  <stop offset="50%" stopColor="#EA580C" stopOpacity="0.28" />
                  <stop offset="100%" stopColor="#312E81" stopOpacity="0.75" />
                </linearGradient>
              </defs>
              <path
                d="M0 520 C 320 410, 620 610, 980 470 C 1180 390, 1340 430, 1440 480 L 1440 900 L 0 900 Z"
                fill="url(#sonoma-hill-2)"
              />
              <path
                d="M0 610 C 280 540, 540 690, 880 590 C 1140 515, 1310 580, 1440 550 L 1440 900 L 0 900 Z"
                fill="url(#sonoma-hill-1)"
              />
              <path
                d="M0 720 C 380 660, 760 790, 1120 690 C 1280 645, 1380 670, 1440 700 L 1440 900 L 0 900 Z"
                fill="rgba(8, 15, 32, 0.72)"
              />
            </svg>
          </motion.div>
        )}

        {wallpaper === 'win11' && (
          <motion.div
            key="win11"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.45 }}
            className="absolute inset-0"
            style={{
              background:
                'radial-gradient(100% 100% at 50% 50%, #0B2559 0%, #061333 55%, #030817 100%)',
            }}
          >
            <div className="absolute inset-0 flex items-center justify-center">
              <svg
                viewBox="0 0 1000 700"
                className="w-[92vw] max-w-[1100px] h-auto opacity-95"
              >
                <defs>
                  <radialGradient id="bloom-petal-1" cx="35%" cy="35%" r="70%">
                    <stop offset="0%" stopColor="#7DD3FC" stopOpacity="0.95" />
                    <stop offset="55%" stopColor="#2563EB" stopOpacity="0.75" />
                    <stop offset="100%" stopColor="#1E3A8A" stopOpacity="0.2" />
                  </radialGradient>
                  <radialGradient id="bloom-petal-2" cx="65%" cy="35%" r="70%">
                    <stop offset="0%" stopColor="#60A5FA" stopOpacity="0.9" />
                    <stop offset="55%" stopColor="#3B82F6" stopOpacity="0.7" />
                    <stop offset="100%" stopColor="#1D4ED8" stopOpacity="0.15" />
                  </radialGradient>
                  <radialGradient id="bloom-petal-3" cx="50%" cy="65%" r="70%">
                    <stop offset="0%" stopColor="#38BDF8" stopOpacity="0.85" />
                    <stop offset="60%" stopColor="#4F46E5" stopOpacity="0.65" />
                    <stop offset="100%" stopColor="#1E1B4B" stopOpacity="0.2" />
                  </radialGradient>
                </defs>
                <g transform="translate(500, 350)">
                  <path
                    d="M 0 0 C -140 -40, -240 -180, -120 -250 C -10 -310, 90 -170, 0 0 Z"
                    fill="url(#bloom-petal-1)"
                  />
                  <path
                    d="M 0 0 C 40 -150, 210 -230, 260 -100 C 300 10, 140 90, 0 0 Z"
                    fill="url(#bloom-petal-2)"
                  />
                  <path
                    d="M 0 0 C 160 30, 230 190, 100 250 C -20 300, -90 150, 0 0 Z"
                    fill="url(#bloom-petal-3)"
                  />
                  <path
                    d="M 0 0 C -40 160, -210 220, -250 90 C -290 -30, -130 -90, 0 0 Z"
                    fill="url(#bloom-petal-2)"
                    opacity="0.85"
                  />
                  <circle cx="0" cy="0" r="42" fill="rgba(147, 197, 253, 0.28)" />
                </g>
              </svg>
            </div>
          </motion.div>
        )}

        {wallpaper === 'transparent' && (
          <motion.div
            key="transparent"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.25 }}
            className="absolute inset-0 bg-transparent"
          />
        )}

        {wallpaper === 'oled' && (
          <motion.div
            key="oled"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.35 }}
            className="absolute inset-0 bg-black"
          />
        )}
      </AnimatePresence>

      {/* Subtle Desktop Menu Bar Context */}
      {wallpaper !== 'oled' && wallpaper !== 'transparent' && !simulatedFullscreen && (
        <div className="pointer-events-none fixed top-0 inset-x-0 h-8 px-5 flex items-center justify-between text-[11.5px] font-medium text-white/70 z-10">
          <div className="flex items-center gap-3">
            <span className="font-semibold text-white/90 tracking-tight">Isle</span>
            <span className="hidden sm:inline text-white/50">Dynamic Island & Agent HUD</span>
          </div>
          <div className="flex items-center gap-3">
            {micMuted && (
              <span className="inline-flex items-center gap-1 text-rose-300 text-[10.5px] font-semibold bg-rose-500/20 px-2 py-0.5 rounded-full border border-rose-400/30">
                <MicOff className="w-3 h-3" />
                Mic Muted [F4]
              </span>
            )}
            <Wifi className="w-3.5 h-3.5 text-white/75" />
            <span className="hidden sm:inline text-white/60 tabular-nums">
              {batteryPct}% {charging ? '⚡' : ''}
            </span>
          </div>
        </div>
      )}

      {/* Developer Simulator Bar & Center Stage Hint (Hidden by default; toggle with Ctrl + Shift + D) */}
      <AnimatePresence>
        {showDevSimulator && (
          <>
            {/* Center Stage Hint */}
            <motion.div
              key="dev-center-hint"
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 12 }}
              transition={{ duration: 0.18 }}
              className="pointer-events-none fixed inset-x-0 bottom-44 flex flex-col items-center justify-center px-4 text-center z-10"
            >
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-black/45 backdrop-blur-md border border-white/10 text-[11px] text-white/70">
                <Sparkles className="w-3 h-3 text-amber-300" />
                <span>
                  {simulatedFullscreen ? (
                    <strong className="text-emerald-300 font-medium">
                      Fullscreen Auto-Hide Active — Island tucked away
                    </strong>
                  ) : (
                    <>
                      Dev Simulator (<kbd className="px-1 py-0.5 rounded bg-white/10 text-white font-mono text-[9.5px]">Ctrl+Shift+D</kbd> to toggle)
                      {' · '}
                      State: <strong className="text-white font-medium capitalize">{currentState}</strong>
                      {' · '}
                      <kbd className="px-1.5 py-0.5 rounded bg-white/10 text-white font-mono text-[10px]">PrtScn</kbd> Auto-Capture
                      {' · '}
                      <kbd className="px-1.5 py-0.5 rounded bg-white/10 text-white font-mono text-[10px]">F4</kbd> Mic Mute
                    </>
                  )}
                </span>
              </div>
            </motion.div>

            {/* Developer Simulator Bar (3-Row Glass Dock) */}
            <motion.div
              key="dev-simulator-dock"
              initial={{ opacity: 0, y: 35, x: '-50%' }}
              animate={{ opacity: 1, y: 0, x: '-50%' }}
              exit={{ opacity: 0, y: 35, x: '-50%' }}
              transition={{ type: 'spring', stiffness: 380, damping: 28 }}
              data-isle-dock="true"
              className="fixed bottom-3 left-1/2 z-30 max-w-[96vw]"
            >
              <div className="flex flex-col gap-1.5 px-3 py-2 rounded-2xl bg-zinc-950/85 backdrop-blur-2xl border border-white/12 shadow-[0_16px_40px_rgba(0,0,0,0.7)]">
                {/* Row 1: Environment, Mount, Trigger, Morph States, Fullscreen, Settings */}
                <div className="flex items-center gap-2 overflow-x-auto">
                  {/* Wallpaper */}
                  <div className="flex items-center gap-1 pr-2 border-r border-white/10 shrink-0">
                    <Monitor className="w-3.5 h-3.5 text-white/45 mr-0.5" />
                    {(
                      [
                        { id: 'transparent', label: 'Transparent' },
                        { id: 'sonoma', label: 'Sonoma' },
                        { id: 'win11', label: 'Win 11' },
                        { id: 'oled', label: 'OLED #000' },
                      ] as Array<{ id: WallpaperId; label: string }>
                    ).map((item) => (
                      <button
                        key={item.id}
                        type="button"
                        onClick={() => onChangeWallpaper(item.id)}
                        className={`px-2 py-1 rounded-lg text-[10.5px] font-medium whitespace-nowrap transition-colors cursor-pointer ${
                          wallpaper === item.id
                            ? 'bg-white/16 text-white border border-white/15'
                            : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
                        }`}
                      >
                        {item.label}
                      </button>
                    ))}
                  </div>

            {/* Mount Mode (Section 2.1: Notch vs Floating) */}
            <div className="flex items-center gap-1 pr-2 border-r border-white/10 shrink-0">
              <PanelTop className="w-3.5 h-3.5 text-white/45 mr-0.5" />
              {(
                [
                  { id: 'notch', label: 'Notch (Flush y=0)' },
                  { id: 'floating', label: 'Floating' },
                ] as Array<{ id: MountMode; label: string }>
              ).map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onChangeMountMode(item.id)}
                  className={`px-2 py-1 rounded-lg text-[10.5px] font-medium whitespace-nowrap transition-colors cursor-pointer ${
                    mountMode === item.id
                      ? 'bg-sky-400/20 text-sky-200 border border-sky-400/35'
                      : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </div>

            {/* Expand Trigger */}
            <div className="flex items-center gap-1 pr-2 border-r border-white/10 shrink-0">
              <MousePointerClick className="w-3.5 h-3.5 text-white/45 mr-0.5" />
              {(
                [
                  { id: 'click', label: 'Click' },
                  { id: 'hover', label: 'Hover' },
                ] as Array<{ id: ExpandTrigger; label: string }>
              ).map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onChangeExpandTrigger(item.id)}
                  className={`px-2 py-1 rounded-lg text-[10.5px] font-medium whitespace-nowrap transition-colors cursor-pointer ${
                    expandTrigger === item.id
                      ? 'bg-emerald-400/20 text-emerald-200 border border-emerald-400/35'
                      : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </div>

            {/* Morph States */}
            <div className="flex items-center gap-1 pr-2 border-r border-white/10 shrink-0">
              <Layers className="w-3.5 h-3.5 text-white/45 mr-0.5" />
              {(
                [
                  { id: 'idle', label: 'Idle' },
                  { id: 'media', label: 'Media' },
                  { id: 'split', label: 'Split' },
                  { id: 'expanded', label: 'Expand' },
                ] as Array<{ id: IslandVisualState; label: string }>
              ).map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onTriggerStatePreset(item.id)}
                  className={`px-2 py-1 rounded-lg text-[10.5px] font-medium whitespace-nowrap transition-colors cursor-pointer ${
                    currentState === item.id
                      ? 'bg-violet-400/25 text-violet-200 border border-violet-400/40'
                      : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </div>

            {/* Fullscreen & Settings */}
            <div className="flex items-center gap-1 shrink-0">
              <button
                type="button"
                onClick={onToggleSimulatedFullscreen}
                className={`px-2 py-1 rounded-lg text-[10.5px] font-medium inline-flex items-center gap-1 whitespace-nowrap transition-colors cursor-pointer ${
                  simulatedFullscreen
                    ? 'bg-rose-500/25 text-rose-200 border border-rose-400/45'
                    : 'text-white/60 hover:text-white hover:bg-white/[0.06]'
                }`}
              >
                <Maximize2 className="w-3 h-3" />
                Fullscreen
              </button>

              <button
                type="button"
                onClick={onOpenSettings}
                className="px-2 py-1 rounded-lg text-[10.5px] font-medium inline-flex items-center gap-1 text-white/70 hover:text-white hover:bg-white/[0.08] whitespace-nowrap transition-colors cursor-pointer"
              >
                <Settings className="w-3 h-3" />
                Settings
              </button>
            </div>
          </div>

          {/* Row 2: Section 1 — Authentic CLI Agent & Dynamic Privacy Sensor Simulator */}
          <div className="flex items-center gap-1.5 pt-1 border-t border-white/[0.08] overflow-x-auto">
            <span className="text-[9.5px] font-bold uppercase tracking-wider text-white/70 pr-1 shrink-0 inline-flex items-center gap-1">
              <CliAgentLogo agentName={cliAgent.agentName} className="w-3.5 h-3.5" />
              CLI Agent:
            </span>

            {/* Agent Selectors (Claude Code / Codex / Generic Terminal) */}
            {(['Claude Code', 'Codex', 'Terminal'] as const).map((name) => {
              const active = cliAgent.agentName === name;
              return (
                <button
                  key={name}
                  type="button"
                  onClick={() => onSetCliAgentStatus(cliAgent.status === 'idle' ? 'thinking' : cliAgent.status, name)}
                  className={`px-2 py-0.5 rounded-lg text-[10px] font-semibold whitespace-nowrap cursor-pointer transition-colors ${
                    active
                      ? 'bg-white/20 text-white border border-white/30'
                      : 'text-white/50 hover:text-white hover:bg-white/[0.06]'
                  }`}
                >
                  {name}
                </button>
              );
            })}

            <div className="w-px h-3.5 bg-white/10 mx-0.5 shrink-0" />

            <button
              type="button"
              onClick={() => onSetCliAgentStatus('idle')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium whitespace-nowrap cursor-pointer ${
                cliAgent.status === 'idle'
                  ? 'bg-white/20 text-white border border-white/25'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              Standby
            </button>

            <button
              type="button"
              onClick={() => onSetCliAgentStatus('thinking')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                cliAgent.status === 'thinking'
                  ? 'bg-white/20 text-white border border-white/30'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              Thinking
            </button>

            <button
              type="button"
              onClick={() => onSetCliAgentStatus('approval')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                cliAgent.status === 'approval'
                  ? 'bg-amber-400/20 text-amber-200 border border-amber-400/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              Approval
            </button>

            <button
              type="button"
              onClick={() => onSetCliAgentStatus('complete')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                cliAgent.status === 'complete'
                  ? 'bg-[#34C759]/20 text-[#34C759] border border-[#34C759]/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              <CheckCircle2 className="w-3 h-3 text-[#34C759]" />
              Passed
            </button>

            <button
              type="button"
              onClick={() => onSetCliAgentStatus('error')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                cliAgent.status === 'error'
                  ? 'bg-[#FF3B30]/20 text-[#FF3B30] border border-[#FF3B30]/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              <XCircle className="w-3 h-3 text-[#FF3B30]" />
              Failed
            </button>

            <div className="w-px h-3.5 bg-white/10 mx-0.5 shrink-0" />

            {/* Section 1.3: Dynamic Privacy Sensor Access Simulator */}
            <span className="text-[9.5px] font-bold uppercase tracking-wider text-emerald-300 pr-1 shrink-0">
              Privacy:
            </span>

            <button
              type="button"
              onClick={() => onTriggerSensorAccess?.('mic', 'Zoom')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                sensorAccess?.active && sensorAccess.type === 'mic'
                  ? 'bg-[#FF9500]/25 text-[#FF9500] border border-[#FF9500]/40'
                  : 'text-amber-200/80 hover:text-amber-100 hover:bg-amber-400/15'
              }`}
            >
              <Mic className="w-3 h-3 text-[#FF9500]" />
              Mic (Zoom)
            </button>

            <button
              type="button"
              onClick={() => onTriggerSensorAccess?.('camera', 'Teams')}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                sensorAccess?.active && sensorAccess.type === 'camera'
                  ? 'bg-[#34C759]/25 text-[#34C759] border border-[#34C759]/40'
                  : 'text-emerald-200/80 hover:text-emerald-100 hover:bg-emerald-400/15'
              }`}
            >
              <Camera className="w-3 h-3 text-[#34C759]" />
              Cam (Teams)
            </button>

            {sensorAccess?.active && (
              <button
                type="button"
                onClick={onClearSensorAccess}
                className="px-1.5 py-0.5 rounded-lg text-[9.5px] font-medium text-white/50 hover:text-white hover:bg-white/10 whitespace-nowrap cursor-pointer"
              >
                Clear
              </button>
            )}

            <div className="w-px h-3.5 bg-white/10 mx-0.5 shrink-0" />

            {/* Section 2: Screenshot PrtSc & Auto-Hide Toggle */}
            <button
              type="button"
              onClick={onTriggerScreenshot}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-sky-200/90 hover:text-sky-100 hover:bg-sky-400/15 whitespace-nowrap cursor-pointer"
              title="Trigger PrtSc capture"
            >
              <Camera className="w-3 h-3" />
              PrtSc Snip
            </button>

            <button
              type="button"
              onClick={onToggleAutoHideScreenshot}
              className={`px-2 py-0.5 rounded-lg text-[9.5px] font-semibold whitespace-nowrap cursor-pointer transition-colors ${
                autoHideOnScreenshot
                  ? 'bg-amber-400/20 text-amber-200 border border-amber-400/35'
                  : 'bg-white/10 text-white/60 hover:text-white'
              }`}
              title="Toggle Auto-Hide during screenshots (Option A: Hide, Option B: Visible)"
            >
              PrtSc Hide: {autoHideOnScreenshot ? 'ON' : 'OFF'}
            </button>

            <div className="w-px h-3.5 bg-white/10 mx-0.5 shrink-0" />

            <button
              type="button"
              onClick={() => onTriggerPeripheral('headphones')}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-sky-200/80 hover:text-sky-100 hover:bg-sky-400/15 whitespace-nowrap cursor-pointer"
            >
              <Headphones className="w-3 h-3" />
              AirPods
            </button>

            <button
              type="button"
              onClick={() => onTriggerPeripheral('smartphone')}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-violet-200/80 hover:text-violet-100 hover:bg-violet-400/15 whitespace-nowrap cursor-pointer"
            >
              <Smartphone className="w-3 h-3" />
              Pixel 9
            </button>

            <button
              type="button"
              onClick={() => onTriggerPeripheral('printer')}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-emerald-200/80 hover:text-emerald-100 hover:bg-emerald-400/15 whitespace-nowrap cursor-pointer"
            >
              <Printer className="w-3 h-3" />
              Printer
            </button>
          </div>

          {/* Row 3: Shiba Moods, ThinkPad F4 Mic Mute, & Battery Engine */}
          <div className="flex items-center gap-1.5 pt-1 border-t border-white/[0.08] overflow-x-auto">
            <span className="text-[9.5px] font-semibold uppercase tracking-wider text-amber-300/80 pr-1 shrink-0">
              Shiba & Battery:
            </span>

            <button
              type="button"
              onClick={() => onSetMoodOverride(null)}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium whitespace-nowrap cursor-pointer ${
                moodOverride === null && !deepWorkActive && !charging && batteryPct >= 20
                  ? 'bg-amber-400/20 text-amber-200 border border-amber-400/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              Auto
            </button>

            {/* Section 1.4: Authentic Battery Charging Sweep Animation */}
            <button
              type="button"
              onClick={onToggleCharging}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                charging
                  ? 'bg-[#34C759]/20 text-[#34C759] border border-[#34C759]/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
              title="Plug / Unplug charger with authentic Apple battery sweep fill animation"
            >
              <BatteryCharging className="w-3 h-3" />
              {charging ? 'Unplug' : 'Plug ⚡'}
            </button>

            <button
              type="button"
              onClick={onTriggerBatteryFull}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                batteryPct === 100
                  ? 'bg-amber-400/25 text-amber-200 border border-amber-400/40'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
              title="Battery 100% full"
            >
              <Battery className="w-3 h-3 text-amber-300" />
              100% Full
            </button>

            <button
              type="button"
              onClick={onToggleLowBattery}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                batteryPct < 20
                  ? 'bg-[#FF3B30]/20 text-[#FF3B30] border border-[#FF3B30]/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
              title="Low battery <20% triggers droopy Shiba (ɵ̥̥ ︵ ɵ̥̥)"
            >
              <BatteryWarning className="w-3 h-3 text-[#FF3B30]" />
              Low Bat
            </button>

            {/* Section 3: ThinkPad Hardware F4 Mic Mute Tape Cross */}
            <button
              type="button"
              onClick={onToggleMicMute}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                micMuted
                  ? 'bg-[#FF3B30]/20 text-[#FF3B30] border border-[#FF3B30]/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
              title="ThinkPad F4 Mic Mute (renders tape cross on Shiba mouth)"
            >
              {micMuted ? <MicOff className="w-3 h-3 text-rose-300" /> : <Mic className="w-3 h-3" />}
              F4 Mute {micMuted ? 'ON' : 'OFF'}
            </button>

            <button
              type="button"
              onClick={onPetShiba}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-pink-200/80 hover:text-pink-100 hover:bg-pink-400/15 whitespace-nowrap cursor-pointer"
            >
              <Heart className="w-3 h-3" />
              Pet
            </button>

            <button
              type="button"
              onClick={() =>
                onSetMoodOverride(moodOverride === 'music' ? null : 'music')
              }
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                moodOverride === 'music'
                  ? 'bg-sky-400/20 text-sky-200 border border-sky-400/35'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              <Music2 className="w-3 h-3" />
              Vibing
            </button>

            <button
              type="button"
              onClick={onToggleDeepWork}
              className={`px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 whitespace-nowrap cursor-pointer ${
                deepWorkActive
                  ? 'bg-indigo-400/25 text-indigo-200 border border-indigo-400/40'
                  : 'text-white/55 hover:text-white hover:bg-white/[0.06]'
              }`}
            >
              <Moon className="w-3 h-3" />
              Deep Work
            </button>

            <div className="w-px h-3.5 bg-white/10 mx-0.5 shrink-0" />

            <button
              type="button"
              onClick={() => onTriggerVolumeHud()}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              <Volume2 className="w-3 h-3 text-sky-300" />
              Vol
            </button>

            <button
              type="button"
              onClick={() => onTriggerVolumeHud('max')}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-sky-200 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
              title="Test Apple boundary wall bounce physics at 100% volume"
            >
              Vol Bounce
            </button>

            <button
              type="button"
              onClick={() => onTriggerBrightnessHud()}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              <Sun className="w-3 h-3 text-amber-300" />
              Bright
            </button>

            <button
              type="button"
              onClick={() => onTriggerBrightnessHud('max')}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-amber-200 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
              title="Test Apple boundary wall bounce physics at 100% brightness"
            >
              Bright Bounce
            </button>

            <button
              type="button"
              onClick={onTriggerCapsLock}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              ⇪ Caps
            </button>

            <button
              type="button"
              onClick={onTriggerDownloadRadar}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              <ArrowDownToLine className="w-3 h-3 text-sky-300" />
              Download
            </button>

            <button
              type="button"
              onClick={onTriggerCalendarRadar}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              <CalendarClock className="w-3 h-3 text-amber-300" />
              Calendar
            </button>

            <button
              type="button"
              onClick={onTriggerHydrationNudge}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              <Droplets className="w-3 h-3 text-sky-300" />
              Hydrate
            </button>

            <button
              type="button"
              onClick={onTriggerEyeRestNudge}
              className="px-2 py-0.5 rounded-lg text-[10px] font-medium inline-flex items-center gap-1 text-white/65 hover:text-white hover:bg-white/[0.08] whitespace-nowrap cursor-pointer"
            >
              <Eye className="w-3 h-3 text-teal-300" />
              Eye Rest
            </button>
          </div>
        </div>
      </motion.div>
    </>
  )}
</AnimatePresence>
    </div>
  );
};
