import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  AnimatePresence,
  motion,
} from 'framer-motion';
import {
  Activity,
  AlertTriangle,
  ArrowDownToLine,
  Battery,
  BatteryCharging,
  BatteryWarning,
  CalendarClock,
  Camera,
  Check,
  CheckSquare,
  Droplets,
  Eye,
  Folder,
  Headphones,
  MapPin,
  Mic,
  MicOff,
  Moon,
  Music2,
  Printer,
  Settings,
  Smartphone,
  Smile,
  Sun,
  Timer,
  Video,
  Volume2,
  VolumeX,
  Zap,
} from 'lucide-react';
import {
  FOCUS_PHASE_CONFIG,
  ISLAND_METRICS,
  ISLAND_SPRING,
  POWER_PLAN_META,
} from '../constants';
import { getActiveLyricIndex } from '../services/lyrics';
import type {
  BatteryActionEffect,
  CalendarRadarState,
  CliAgentActivity,
  ClipboardItem,
  DownloadRadarState,
  ExpandTrigger,
  FocusPhase,
  FocusState,
  HudOverlay,
  IslandVisualState,
  LyricsState,
  MountMode,
  PowerPlan,
  ScrcpyState,
  ScreenshotRecord,
  SensorAccessState,
  ShelfFile,
  ShibaSettings,
  ShibaState,
  TabConfigItem,
  TabKey,
  TaskItem,
  TaskPriorityColor,
  Track,
  WallpaperId,
  WaterState,
} from '../types';
import { audioEngine } from '../utils/audioEngine';
import { CliAgentLogo } from './CliAgentLogo';
import { Equalizer } from './Equalizer';
import { ShibaMascot } from './ShibaMascot';
import { FocusTab } from './tabs/FocusTab';
import { MediaTab } from './tabs/MediaTab';
import { MonitorTab } from './tabs/MonitorTab';
import { ShelfTab } from './tabs/ShelfTab';
import { TasksTab } from './tabs/TasksTab';
import { WaterTab } from './tabs/WaterTab';
import { ZenTab } from './tabs/ZenTab';

interface DynamicIslandProps {
  expanded: boolean;
  activeTabKey: TabKey;
  enabledTabs: TabConfigItem[];
  mountMode: MountMode;
  expandTrigger: ExpandTrigger;
  wallpaper: WallpaperId;
  visualState: IslandVisualState;
  // Shiba & Etiquette
  shibaState: ShibaState;
  shibaSettings: ShibaSettings;
  isHiddenFullscreen: boolean;
  screenshotHiding?: boolean;
  isDeepWorkActive: boolean;
  onPetShiba: () => void;
  onToggleDnd: () => void;
  onOpenSettings: () => void;
  // System HUD & Radars
  hudOverlay: HudOverlay | null;
  systemVolume: number;
  micActive: boolean;
  micMuted: boolean;
  batteryPct: number;
  charging: boolean;
  batteryEffect?: BatteryActionEffect;
  powerPlan: PowerPlan;
  downloadRadar: DownloadRadarState;
  calendarRadar: CalendarRadarState;
  sensorAccess?: SensorAccessState | null;
  onWheelVolume: (deltaY: number) => void;
  onToggleMicMute: () => void;
  onCyclePowerPlan: () => void;
  onOpenDownloadedFile: () => void;
  onJoinCalendarMeet: () => void;
  // CLI Coding Agent Monitor (Zero AI-Slop)
  cliAgent: CliAgentActivity;
  onApproveAgent?: () => void;
  onRejectAgent?: () => void;
  onFocusTerminal?: () => void;
  // Media & Lyrics
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
  // Focus
  focus: FocusState;
  focusCelebrating: boolean;
  onToggleFocus: () => void;
  onResetFocus: () => void;
  onSelectFocusPhase: (phase: FocusPhase) => void;
  onCompleteFocusNow: () => void;
  // Water
  water: WaterState;
  onAddWater: (ml: number) => void;
  onUndoWater: () => void;
  onResetWater: () => void;
  // Tasks
  tasks: TaskItem[];
  onToggleTask: (id: string) => void;
  onAddTask: (title: string, color: TaskPriorityColor) => void;
  onDeleteTask: (id: string) => void;
  onClearTasksCompleted?: () => void;
  // Shelf & Scrcpy & Screenshots
  shelfFiles: ShelfFile[];
  scrcpy: ScrcpyState;
  screenshots?: ScreenshotRecord[];
  onAddShelfFiles: (files: FileList | ShelfFile[]) => void;
  onRemoveShelfFile: (id: string) => void;
  onClearShelfFiles: () => void;
  onToggleMirror: () => void;
  onToggleRecord: () => void;
  onToggleWirelessAdb: () => void;
  onCopyClipboard: () => void;
  onOpenScreenshot?: (record: ScreenshotRecord) => void;
  onCopyScreenshotPath?: (record: ScreenshotRecord) => void;
  // Clipboard Items
  clipboardItems: ClipboardItem[];
  onToggleClipboardPin: (id: string) => void;
  onCopyClipboardItem: (item: ClipboardItem) => void;
  // Zen Breathing
  zenActive: boolean;
  onToggleZenActive: (next: boolean) => void;
  // Island navigation
  onExpand: (tabKey?: TabKey) => void;
  onCollapse: () => void;
  onSelectTab: (tabKey: TabKey) => void;
}

function formatClock(date: Date): string {
  const hh = String(date.getHours()).padStart(2, '0');
  const mm = String(date.getMinutes()).padStart(2, '0');
  return `${hh}:${mm}`;
}

export function TabIcon({ tabKey, className }: { tabKey: TabKey; className?: string }) {
  switch (tabKey) {
    case 'media':
      return <Music2 className={className} />;
    case 'focus':
      return <Timer className={className} />;
    case 'water':
      return <Droplets className={className} />;
    case 'tasks':
      return <CheckSquare className={className} />;
    case 'shelf':
      return <Folder className={className} />;
    case 'monitor':
      return <Activity className={className} />;
    case 'zen':
      return <Smile className={className} />;
  }
}

export const DynamicIsland: React.FC<DynamicIslandProps> = ({
  expanded,
  activeTabKey,
  enabledTabs,
  mountMode,
  expandTrigger,
  wallpaper,
  visualState,
  shibaState,
  shibaSettings,
  isHiddenFullscreen,
  screenshotHiding = false,
  isDeepWorkActive,
  onPetShiba,
  onToggleDnd,
  onOpenSettings,
  hudOverlay,
  systemVolume: _systemVolume,
  micActive: _micActive,
  micMuted,
  batteryPct,
  charging,
  batteryEffect: _batteryEffect,
  powerPlan,
  downloadRadar,
  calendarRadar,
  sensorAccess,
  onWheelVolume,
  onToggleMicMute,
  onCyclePowerPlan,
  onOpenDownloadedFile,
  onJoinCalendarMeet,
  cliAgent,
  onApproveAgent,
  onRejectAgent,
  onFocusTerminal,
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
  focus,
  focusCelebrating,
  onToggleFocus,
  onResetFocus,
  onSelectFocusPhase,
  onCompleteFocusNow,
  water,
  onAddWater,
  onUndoWater,
  onResetWater,
  tasks,
  onToggleTask,
  onAddTask,
  onDeleteTask,
  onClearTasksCompleted,
  shelfFiles,
  scrcpy,
  screenshots = [],
  onAddShelfFiles,
  onRemoveShelfFile,
  onClearShelfFiles,
  onToggleMirror,
  onToggleRecord,
  onToggleWirelessAdb,
  onCopyClipboard,
  onOpenScreenshot,
  onCopyScreenshotPath,
  clipboardItems,
  onToggleClipboardPin,
  onCopyClipboardItem,
  zenActive,
  onToggleZenActive,
  onExpand,
  onCollapse,
  onSelectTab,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const hoverLeaveTimer = useRef<number | null>(null);
  const lastWheelTimeRef = useRef(0);

  const [now, setNow] = useState(() => new Date());
  const [isHovered, setIsHovered] = useState(false);
  const [dockHovered, setDockHovered] = useState(false);
  const [slideDirection, setSlideDirection] = useState<1 | -1>(1);
  const [scrollTrapped, setScrollTrapped] = useState(false);

  const activeIndex = Math.max(
    0,
    enabledTabs.findIndex((t) => t.key === activeTabKey)
  );
  const currentTabMeta = enabledTabs[activeIndex] ?? enabledTabs[0];

  // Clock tick
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 10000);
    return () => window.clearInterval(id);
  }, []);

  // Click-outside & Escape key collapse
  useEffect(() => {
    if (!expanded) return;
    const handlePointerDown = (e: PointerEvent) => {
      const target = e.target as HTMLElement | null;
      if (!target) return;
      if (containerRef.current?.contains(target)) return;
      if (target.closest('[data-isle-dock="true"]')) return;
      if (target.closest('[data-isle-modal="true"]')) return;
      audioEngine.playPop();
      onCollapse();
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        audioEngine.playPop();
        onCollapse();
      }
    };
    window.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [expanded, onCollapse]);

  // Hover triggers
  const handleMouseEnter = () => {
    setIsHovered(true);
    if (hoverLeaveTimer.current !== null) {
      window.clearTimeout(hoverLeaveTimer.current);
      hoverLeaveTimer.current = null;
    }
    if (expandTrigger === 'hover' && !expanded) {
      audioEngine.playPop();
      onExpand();
    }
  };

  const handleMouseLeave = () => {
    setIsHovered(false);
    if (expandTrigger === 'hover' && expanded) {
      hoverLeaveTimer.current = window.setTimeout(() => {
        audioEngine.playPop();
        onCollapse();
      }, 260);
    }
  };

  // Drag over auto-expand to Shelf
  const handleDragEnter = (e: React.DragEvent) => {
    if (e.dataTransfer.types.includes('Files')) {
      e.preventDefault();
      audioEngine.playPop();
      onExpand('shelf');
    }
  };

  // Scroll-Wheel Isolation
  const handleWheel = (e: React.WheelEvent) => {
    if (!expanded) {
      onWheelVolume(e.deltaY);
      return;
    }

    if (scrollTrapped) {
      return;
    }

    const nowTime = Date.now();
    if (nowTime - lastWheelTimeRef.current < 200) return;

    if (e.deltaY > 30) {
      lastWheelTimeRef.current = nowTime;
      const nextIdx = (activeIndex + 1) % enabledTabs.length;
      setSlideDirection(1);
      audioEngine.playClick();
      onSelectTab(enabledTabs[nextIdx].key);
    } else if (e.deltaY < -30) {
      lastWheelTimeRef.current = nowTime;
      const prevIdx = (activeIndex - 1 + enabledTabs.length) % enabledTabs.length;
      setSlideDirection(-1);
      audioEngine.playClick();
      onSelectTab(enabledTabs[prevIdx].key);
    }
  };

  // Compact Mode Priority:
  // HUD > Agent Approval > Agent Toast > Agent Thinking > Download > Calendar > Media > Split > Idle
  const compactMode:
    | 'hud'
    | 'agentApproval'
    | 'agentToast'
    | 'agentThinking'
    | 'download'
    | 'calendar'
    | 'split'
    | 'media'
    | 'idle' = hudOverlay
    ? 'hud'
    : cliAgent.status === 'approval'
      ? 'agentApproval'
      : cliAgent.status === 'complete' || cliAgent.status === 'error'
        ? 'agentToast'
        : cliAgent.status === 'thinking'
          ? 'agentThinking'
          : downloadRadar.active
            ? 'download'
            : calendarRadar.active
              ? 'calendar'
              : visualState === 'split'
                ? 'split'
                : visualState === 'media'
                  ? 'media'
                  : 'idle';

  const isSplit = compactMode === 'split' && !expanded;
  const isStealth = shibaSettings.presenceLevel === 'stealth';

  // Active lyric index and dynamic ticker calculation
  const activeLyricIdx = useMemo(() => {
    return getActiveLyricIndex(lyrics.lines, Math.round(positionSec * 1000));
  }, [lyrics.lines, positionSec]);

  const activeLyricLine = lyrics.lines[activeLyricIdx];
  const nextLyricLine = lyrics.lines[activeLyricIdx + 1];

  const isInstrumentalBreak = useMemo(() => {
    if (!playing) return true;
    if (!lyrics.lines || lyrics.lines.length === 0) return true;
    const posMs = Math.round(positionSec * 1000);

    if (activeLyricIdx < 0) {
      const firstLineTime = lyrics.lines[0]?.timeMs ?? 0;
      return firstLineTime - posMs > 3500;
    }

    const text = activeLyricLine?.text?.trim() ?? '';
    if (!text || text.startsWith('♪') || text.startsWith('[') || text === '• • •') {
      return true;
    }

    if (nextLyricLine) {
      const timeUntilNext = (nextLyricLine.timeMs - posMs) / 1000;
      const timeSinceCurrent = (posMs - activeLyricLine.timeMs) / 1000;
      if (timeUntilNext > 3.5 && timeSinceCurrent > 2.5) {
        return true;
      }
    }

    return false;
  }, [playing, lyrics.lines, positionSec, activeLyricIdx, activeLyricLine, nextLyricLine]);

  // System color-scheme listener for 'system' theme
  const [systemIsDark, setSystemIsDark] = useState(() => {
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return true;
  });

  useEffect(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return;
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => setSystemIsDark(e.matches);
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  const selectedTheme = shibaSettings.theme ?? 'dark';
  const effectiveTheme =
    selectedTheme === 'system' ? (systemIsDark ? 'dark' : 'light') : selectedTheme;

  // Breathing width: clamp(210px, textWidth + 84px, 480px)
  // During instrumental breaks (> 3.5s) or inactive lyrics, contracts back to minimal resting width (~200px)
  const breathingPillWidth = useMemo(() => {
    if (isInstrumentalBreak || !activeLyricLine?.text || shibaSettings.liveLyricTicker === false) {
      return 200;
    }
    const charLen = activeLyricLine.text.length;
    const approxTextWidth = Math.min(380, Math.max(80, charLen * 7.5));
    return Math.min(480, Math.max(210, Math.round(approxTextWidth + 84)));
  }, [isInstrumentalBreak, activeLyricLine, shibaSettings.liveLyricTicker]);

  // Main island width & geometry metrics (Apple HIG Standards)
  const mainMetrics = expanded
    ? ISLAND_METRICS.expanded
    : compactMode === 'hud'
      ? hudOverlay?.kind === 'batteryNotice' && (hudOverlay.effect === 'chargingPulse' || hudOverlay.effect === 'chargingSweep')
        ? ISLAND_METRICS.batterySweep
        : hudOverlay?.kind === 'capsLock'
          ? ISLAND_METRICS.capsLockToast
          : hudOverlay?.kind === 'sensorToast'
            ? ISLAND_METRICS.sensorToast
            : hudOverlay?.kind === 'nudge'
              ? { width: 286, height: 36, radius: 18 }
              : hudOverlay?.kind === 'peripheral'
                ? ISLAND_METRICS.peripheralToast
                : hudOverlay?.kind === 'screenshot'
                  ? { width: 330, height: 36, radius: 18 }
                  : hudOverlay?.kind === 'batteryNotice'
                    ? { width: 290, height: 36, radius: 18 }
                    : ISLAND_METRICS.hud
      : compactMode === 'agentApproval'
        ? ISLAND_METRICS.agentApproval
        : compactMode === 'agentToast'
          ? ISLAND_METRICS.agentToast
          : compactMode === 'agentThinking'
            ? ISLAND_METRICS.agentThinking
            : compactMode === 'download'
              ? ISLAND_METRICS.radarDownload
              : compactMode === 'calendar'
                ? ISLAND_METRICS.radarCalendar
                : compactMode === 'media'
                  ? { width: breathingPillWidth, height: 36, radius: 18 }
                  : isSplit
                    ? playing
                      ? { width: Math.min(320, breathingPillWidth), height: 36, radius: 18 }
                      : ISLAND_METRICS.splitMainIdle
                    : isStealth
                      ? ISLAND_METRICS.stealthIdle
                      : ISLAND_METRICS.idle;

  // Notch Mode Bezel Flush Silhouette
  // When mountMode === 'notch', container top edge is strictly flush ($y = 0, topRadius = 0).
  // Bottom corners curve downwards with smooth bottom corners (rounded-b-[22px]).
  // Zero outer concave wings.
  const topOffset = mountMode === 'notch' ? 0 : 12;
  const topRadius = mountMode === 'notch' ? 0 : mainMetrics.radius;
  const bottomRadius = mountMode === 'notch' ? 22 : mainMetrics.radius;
  const glassClass =
    effectiveTheme === 'liquid'
      ? 'isle-glass-liquid'
      : effectiveTheme === 'light'
        ? 'isle-glass-light'
        : wallpaper === 'oled'
          ? 'isle-glass-oled'
          : 'isle-glass';

  // Focus badge ring math for Split Island
  const focusCfg = FOCUS_PHASE_CONFIG[focus.phase];
  const focusProgress =
    focus.totalSec > 0 ? Math.min(1, Math.max(0, 1 - focus.remainingSec / focus.totalSec)) : 0;
  const badgeRadius = 12;
  const badgeCirc = 2 * Math.PI * badgeRadius;
  const badgeOffset = badgeCirc * (1 - focusProgress);

  const renderTabContent = (key: TabKey) => {
    switch (key) {
      case 'media':
        return (
          <MediaTab
            track={track}
            playing={playing}
            positionSec={positionSec}
            volume={volume}
            lyrics={lyrics}
            showLyrics={showLyrics}
            onToggleShowLyrics={onToggleShowLyrics}
            onTogglePlay={onTogglePlay}
            onPrevTrack={onPrevTrack}
            onNextTrack={onNextTrack}
            onSeek={onSeek}
            onVolumeChange={onVolumeChange}
            onScrollTrapChange={setScrollTrapped}
          />
        );
      case 'focus':
        return (
          <FocusTab
            focus={focus}
            celebrating={focusCelebrating}
            onToggle={onToggleFocus}
            onReset={onResetFocus}
            onSelectPhase={onSelectFocusPhase}
            onCompleteNow={onCompleteFocusNow}
          />
        );
      case 'water':
        return (
          <WaterTab
            water={water}
            onAddWater={onAddWater}
            onUndo={onUndoWater}
            onReset={onResetWater}
          />
        );
      case 'tasks':
        return (
          <TasksTab
            tasks={tasks}
            onToggleTask={onToggleTask}
            onAddTask={onAddTask}
            onDeleteTask={onDeleteTask}
            onClearCompleted={onClearTasksCompleted}
            onScrollTrapChange={setScrollTrapped}
          />
        );
      case 'shelf':
        return (
          <ShelfTab
            files={shelfFiles}
            scrcpy={scrcpy}
            clipboardItems={clipboardItems}
            screenshots={screenshots}
            onAddFiles={onAddShelfFiles}
            onRemoveFile={onRemoveShelfFile}
            onClearFiles={onClearShelfFiles}
            onToggleMirror={onToggleMirror}
            onToggleRecord={onToggleRecord}
            onToggleWirelessAdb={onToggleWirelessAdb}
            onCopyClipboard={onCopyClipboard}
            onToggleClipboardPin={onToggleClipboardPin}
            onCopyClipboardItem={onCopyClipboardItem}
            onOpenScreenshot={onOpenScreenshot}
            onCopyScreenshotPath={onCopyScreenshotPath}
            onScrollTrapChange={setScrollTrapped}
          />
        );
      case 'monitor':
        return <MonitorTab />;
      case 'zen':
        return <ZenTab active={zenActive} onToggleActive={onToggleZenActive} />;
    }
  };

  return (
    <motion.div
      ref={containerRef}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onDragEnter={handleDragEnter}
      onWheel={handleWheel}
      className="fixed left-1/2 -translate-x-1/2 z-40 flex items-start justify-center"
      animate={{
        top: topOffset,
        y: isHiddenFullscreen || screenshotHiding ? -120 : 0,
        opacity: isHiddenFullscreen || screenshotHiding ? 0 : 1,
      }}
      style={{ pointerEvents: isHiddenFullscreen || screenshotHiding ? 'none' : 'auto' }}
      transition={ISLAND_SPRING}
    >
      <div className="relative flex items-start">
        {/* Main Island Glass Body — Zero AI-Slop (No radioactive neon glows) */}
        <motion.div
          layout
          onClick={() => {
            if (!expanded) {
              audioEngine.playPop();
              onExpand();
            }
          }}
          animate={{
            width: mainMetrics.width,
            height: mainMetrics.height,
            borderTopLeftRadius: topRadius,
            borderTopRightRadius: topRadius,
            borderBottomLeftRadius: bottomRadius,
            borderBottomRightRadius: bottomRadius,
            boxShadow: '0 12px 36px rgba(0, 0, 0, 0.65)',
          }}
          transition={{ type: 'spring', stiffness: 320, damping: 28 }}
          className={`relative overflow-hidden cursor-pointer border border-white/10 ${glassClass}`}
          data-theme={effectiveTheme}
          style={{ willChange: 'width, height, border-radius' }}
        >
          <AnimatePresence mode="wait">
            {!expanded ? (
              /* Compact Pill State */
              <motion.div
                key="compact-pill"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.16 }}
                className="w-full h-full px-2.5 flex items-center justify-between gap-2 overflow-hidden"
              >
                {/* 1. HUD OVERLAYS */}
                {compactMode === 'hud' && hudOverlay ? (
                  hudOverlay.kind === 'volume' ? (
                    /* Section 1.6: Volume Slider with Over-Scroll Wall Bounce Physics */
                    <div className="w-full flex items-center gap-2">
                      {hudOverlay.muted || hudOverlay.value === 0 ? (
                        <VolumeX className="w-3.5 h-3.5 text-[#FF3B30] shrink-0" />
                      ) : (
                        <Volume2 className="w-3.5 h-3.5 text-white/90 shrink-0" />
                      )}
                      <div className="flex-1 h-1.5 rounded-full bg-white/20 overflow-hidden relative">
                        <motion.div
                          className="h-full bg-white rounded-full"
                          animate={{
                            width: `${hudOverlay.value}%`,
                            scaleX:
                              hudOverlay.overscroll === 'max'
                                ? [1, 1.1, 1]
                                : hudOverlay.overscroll === 'min'
                                  ? [1, 0.9, 1]
                                  : 1,
                            originX: hudOverlay.overscroll === 'max' ? 1 : 0,
                          }}
                          transition={{ type: 'spring', stiffness: 500, damping: 20 }}
                        />
                      </div>
                      <span className="text-[11px] font-semibold text-white tabular-nums w-8 text-right">
                        {hudOverlay.value}%
                      </span>
                    </div>
                  ) : hudOverlay.kind === 'brightness' ? (
                    /* Section 1.6: Brightness Slider with Over-Scroll Wall Bounce Physics */
                    <div className="w-full flex items-center gap-2">
                      <Sun className="w-3.5 h-3.5 text-[#FF9500] shrink-0" />
                      <div className="flex-1 h-1.5 rounded-full bg-white/20 overflow-hidden relative">
                        <motion.div
                          className="h-full bg-[#FF9500] rounded-full"
                          animate={{
                            width: `${hudOverlay.value}%`,
                            scaleX:
                              hudOverlay.overscroll === 'max'
                                ? [1, 1.1, 1]
                                : hudOverlay.overscroll === 'min'
                                  ? [1, 0.9, 1]
                                  : 1,
                            originX: hudOverlay.overscroll === 'max' ? 1 : 0,
                          }}
                          transition={{ type: 'spring', stiffness: 500, damping: 20 }}
                        />
                      </div>
                      <span className="text-[11px] font-semibold text-white tabular-nums w-8 text-right">
                        {hudOverlay.value}%
                      </span>
                    </div>
                  ) : hudOverlay.kind === 'capsLock' ? (
                    /* Section 1.5: Apple-Grade Transient Caps Lock Toast (No Crude Nested Boxes) */
                    <div className="w-full flex items-center justify-center gap-2">
                      {hudOverlay.enabled ? (
                        <motion.div
                          initial={{ y: 8, opacity: 0 }}
                          animate={{ y: 0, opacity: 1 }}
                          exit={{ y: -8, opacity: 0 }}
                          transition={{ type: 'spring', stiffness: 420, damping: 28 }}
                          className="flex items-center gap-2"
                        >
                          <span className="text-[14px] font-bold text-white leading-none">⇧</span>
                          <span className="text-[11.5px] font-semibold text-white tracking-tight">
                            Caps Lock
                          </span>
                          <span className="w-1.5 h-1.5 rounded-full bg-[#34C759]" />
                        </motion.div>
                      ) : (
                        <motion.div
                          initial={{ opacity: 1, y: 0 }}
                          animate={{ opacity: 0.75, y: 0 }}
                          exit={{ opacity: 0, y: -8 }}
                          transition={{ duration: 0.2 }}
                          className="flex items-center gap-2"
                        >
                          <div className="relative w-4 h-4 flex items-center justify-center">
                            <span className="text-[14px] font-bold text-white/50 leading-none">⇧</span>
                            <svg className="absolute inset-0 w-4 h-4 pointer-events-none" viewBox="0 0 16 16">
                              <motion.line
                                x1="2"
                                y1="14"
                                x2="14"
                                y2="2"
                                stroke="#FF3B30"
                                strokeWidth="2"
                                strokeLinecap="round"
                                initial={{ pathLength: 0 }}
                                animate={{ pathLength: 1 }}
                                transition={{ duration: 0.2, ease: 'easeOut' }}
                              />
                            </svg>
                          </div>
                          <span className="text-[11.5px] font-semibold text-white/60 tracking-tight">
                            Caps Lock Off
                          </span>
                        </motion.div>
                      )}
                    </div>
                  ) : hudOverlay.kind === 'sensorToast' ? (
                    /* Section 1.3: Dynamic Privacy Toast (Temporary 2.5s) */
                    <div className="w-full flex items-center justify-center gap-2">
                      {hudOverlay.sensor === 'camera' ? (
                        <>
                          <Camera className="w-3.5 h-3.5 text-[#34C759] shrink-0" />
                          <span className="text-[11.5px] font-semibold text-white">
                            Camera Active · {hudOverlay.app}
                          </span>
                        </>
                      ) : hudOverlay.sensor === 'mic' ? (
                        <>
                          <Mic className="w-3.5 h-3.5 text-[#FF9500] shrink-0" />
                          <span className="text-[11.5px] font-semibold text-white">
                            Mic in use · {hudOverlay.app}
                          </span>
                        </>
                      ) : (
                        <>
                          <MapPin className="w-3.5 h-3.5 text-[#007AFF] shrink-0" />
                          <span className="text-[11.5px] font-semibold text-white">
                            Location Accessed · {hudOverlay.app}
                          </span>
                        </>
                      )}
                    </div>
                  ) : hudOverlay.kind === 'batteryNotice' ? (
                    /* Section 1.4: Authentic Apple Battery Sweep Fill (0% -> Current %) */
                    hudOverlay.effect === 'chargingPulse' || hudOverlay.effect === 'chargingSweep' ? (
                      <div className="w-full h-full flex items-center justify-between px-2 gap-3">
                        <div className="flex items-center gap-1.5">
                          <Zap className="w-3.5 h-3.5 text-[#34C759] fill-[#34C759] shrink-0" />
                          <span className="text-[11.5px] font-semibold text-white tracking-tight">
                            Charging
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="text-[12px] font-bold text-white tabular-nums">
                            {hudOverlay.pct ?? batteryPct}%
                          </span>
                          {/* Apple Battery Outline Glyph with terminal nipple */}
                          <div className="relative w-8 h-4 rounded-[4px] border-[1.5px] border-white/70 p-[1.5px] flex items-center shrink-0">
                            <div className="absolute -right-[3.5px] top-1/2 -translate-y-1/2 w-[2px] h-[5px] rounded-r-[1px] bg-white/70" />
                            <motion.div
                              className="h-full rounded-[2px] bg-[#34C759]"
                              initial={{ width: '0%' }}
                              animate={{ width: `${hudOverlay.pct ?? batteryPct}%` }}
                              transition={{ duration: 1.2, ease: [0.16, 1, 0.3, 1] }}
                            />
                          </div>
                        </div>
                      </div>
                    ) : (
                      /* Clean Unplugged / Status Indicator */
                      <div className="w-full flex items-center justify-center gap-2">
                        {hudOverlay.effect === 'lowBatteryToast' ? (
                          <BatteryWarning className="w-3.5 h-3.5 text-[#FF3B30] shrink-0" />
                        ) : (
                          <Battery className="w-3.5 h-3.5 text-white/70 shrink-0" />
                        )}
                        <span className="text-[11px] font-semibold text-white">
                          {hudOverlay.message}
                        </span>
                      </div>
                    )
                  ) : hudOverlay.kind === 'mic' ? (
                    /* ThinkPad F4 Hardware Mic Mute Status Toast */
                    <div className="w-full flex items-center justify-center gap-2">
                      {hudOverlay.muted ? (
                        <>
                          <MicOff className="w-3.5 h-3.5 text-[#FF3B30] shrink-0" />
                          <span className="text-[11.5px] font-semibold text-rose-300">
                            Mic Muted · F4
                          </span>
                        </>
                      ) : (
                        <>
                          <Mic className="w-3.5 h-3.5 text-[#34C759] shrink-0" />
                          <span className="text-[11.5px] font-semibold text-emerald-300">
                            Microphone Live
                          </span>
                        </>
                      )}
                    </div>
                  ) : hudOverlay.kind === 'powerPlan' ? (
                    <div className="w-full flex items-center justify-center gap-2">
                      <Zap
                        className="w-3.5 h-3.5"
                        style={{ color: POWER_PLAN_META[hudOverlay.plan].color }}
                      />
                      <span className="text-[11px] font-semibold text-white">
                        {POWER_PLAN_META[hudOverlay.plan].label}
                      </span>
                    </div>
                  ) : hudOverlay.kind === 'peripheral' ? (
                    <div className="w-full flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        {hudOverlay.device.type === 'headphones' ? (
                          <Headphones className="w-3.5 h-3.5 text-sky-400 shrink-0" />
                        ) : hudOverlay.device.type === 'smartphone' ? (
                          <Smartphone className="w-3.5 h-3.5 text-violet-400 shrink-0" />
                        ) : (
                          <Printer className="w-3.5 h-3.5 text-[#34C759] shrink-0" />
                        )}
                        <span className="text-[11px] font-semibold text-white truncate">
                          {hudOverlay.device.name}
                        </span>
                      </div>
                      <span className="px-2 py-0.5 rounded-full bg-white/10 text-white/90 border border-white/10 text-[10px] font-bold shrink-0">
                        {hudOverlay.device.detail}
                      </span>
                    </div>
                  ) : hudOverlay.kind === 'screenshot' ? (
                    <div className="w-full flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <Camera className="w-3.5 h-3.5 text-sky-400 shrink-0" />
                        <span className="text-[11px] font-medium text-white truncate">
                          {hudOverlay.record.fileName}
                        </span>
                      </div>
                      <span className="text-[10px] font-semibold text-sky-300 px-2 py-0.5 rounded-full bg-sky-400/20 shrink-0">
                        Staged to Shelf
                      </span>
                    </div>
                  ) : (
                    /* Productive Nudge */
                    <div className="w-full flex items-center justify-between gap-2">
                      <ShibaMascot
                        state={hudOverlay.nudgeType === 'eyeRest' ? 'zen' : 'happy'}
                        presenceLevel={shibaSettings.presenceLevel}
                        isIslandHovered={isHovered}
                        micMuted={micMuted}
                        size="sm"
                      />
                      <span className="text-[11px] font-medium text-white/90 truncate flex-1">
                        {hudOverlay.message}
                      </span>
                      {hudOverlay.nudgeType === 'hydration' ? (
                        <Droplets className="w-3.5 h-3.5 text-sky-400 shrink-0" />
                      ) : (
                        <Eye className="w-3.5 h-3.5 text-teal-300 shrink-0" />
                      )}
                    </div>
                  )
                ) : compactMode === 'agentApproval' ? (
                  /* Section 1.2: Claude Code Approval (Clean Apple HIG, No Robot Emojis) */
                  <div
                    className="w-full flex items-center justify-between gap-2 px-1"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="flex items-center gap-1.5 min-w-0">
                      <AlertTriangle className="w-3.5 h-3.5 text-[#FF9500] shrink-0" />
                      <div className="min-w-0">
                        <p className="text-[11px] font-bold text-white truncate leading-none">
                          {cliAgent.agentName} Approval
                        </p>
                        <p className="text-[9.5px] text-white/75 truncate leading-none mt-0.5">
                          {cliAgent.question || cliAgent.headline}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center gap-1.5 shrink-0">
                      <button
                        type="button"
                        onClick={onApproveAgent}
                        className="px-2.5 py-0.5 rounded-full bg-[#34C759] hover:bg-[#34C759]/90 text-black text-[10px] font-bold cursor-pointer transition-colors shadow-sm"
                      >
                        Approve
                      </button>
                      <button
                        type="button"
                        onClick={onRejectAgent}
                        className="px-2.5 py-0.5 rounded-full bg-[#FF3B30]/20 hover:bg-[#FF3B30]/30 border border-[#FF3B30]/30 text-[#FF3B30] text-[10px] font-semibold cursor-pointer transition-colors"
                      >
                        Reject
                      </button>
                      <button
                        type="button"
                        onClick={onFocusTerminal}
                        className="px-2 py-0.5 rounded-full bg-white/10 hover:bg-white/20 text-white/70 hover:text-white text-[9.5px] cursor-pointer"
                        title="Focus active terminal"
                      >
                        Terminal
                      </button>
                    </div>
                  </div>
                ) : compactMode === 'agentToast' ? (
                  /* Section 1.2: Task Complete / Error Toast */
                  <div
                    className="w-full flex items-center justify-between gap-2 px-1"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <ShibaMascot
                        state={cliAgent.status === 'complete' ? 'happy' : 'concerned'}
                        presenceLevel={shibaSettings.presenceLevel}
                        isIslandHovered={isHovered}
                        agentState={cliAgent.status}
                        micMuted={micMuted}
                        size="sm"
                      />
                      <div className="min-w-0">
                        <p
                          className={`text-[11px] font-semibold truncate leading-tight flex items-center gap-1.5 ${
                            cliAgent.status === 'complete'
                              ? 'text-[#34C759]'
                              : 'text-[#FF3B30]'
                          }`}
                        >
                          {cliAgent.status === 'complete' ? (
                            <Check className="w-3 h-3 text-[#34C759] shrink-0" />
                          ) : (
                            <AlertTriangle className="w-3 h-3 text-[#FF3B30] shrink-0" />
                          )}
                          <span>
                            {cliAgent.agentName}: {cliAgent.headline}
                          </span>
                        </p>
                        {cliAgent.detail && (
                          <p className="text-[9px] text-white/60 truncate leading-none mt-0.5">
                            {cliAgent.detail}
                          </p>
                        )}
                      </div>
                    </div>
                    <button
                      type="button"
                      onClick={onFocusTerminal}
                      className="px-2 py-0.5 rounded-full bg-white/10 hover:bg-white/20 text-[10px] text-white font-medium cursor-pointer shrink-0"
                    >
                      View
                    </button>
                  </div>
                ) : compactMode === 'agentThinking' ? (
                  /* Section 1.2: Agent Thinking (Clean Anthropic Spark Logo, Zero Emojis) */
                  <div className="w-full flex items-center justify-between gap-2 px-1">
                    <ShibaMascot
                      state="normal"
                      presenceLevel={shibaSettings.presenceLevel}
                      isIslandHovered={isHovered}
                      agentState="thinking"
                      micMuted={micMuted}
                      size="sm"
                      onPet={onPetShiba}
                    />

                    <div className="flex-1 min-w-0 px-1">
                      <div className="flex items-center gap-1.5">
                        <CliAgentLogo agentName={cliAgent.agentName} className="w-3.5 h-3.5 shrink-0" />
                        <span className="text-[11.5px] font-medium text-white/95 truncate">
                          {cliAgent.agentName}: {cliAgent.headline}
                        </span>
                      </div>
                    </div>

                    <span className="text-[9px] font-mono text-white/50 px-1.5 py-0.5 rounded bg-white/5 border border-white/10 shrink-0">
                      CLI
                    </span>
                  </div>
                ) : compactMode === 'download' ? (
                  /* Download Progress Pill */
                  <div
                    className="w-full flex items-center justify-between gap-2"
                    onClick={(e) => e.stopPropagation()}
                  >
                    {downloadRadar.completed ? (
                      <>
                        <div className="flex items-center gap-2 min-w-0">
                          <span className="w-5 h-5 rounded-full bg-[#34C759]/20 border border-[#34C759]/40 flex items-center justify-center text-[#34C759] shrink-0">
                            <Check className="w-3 h-3" />
                          </span>
                          <span className="text-[11px] font-medium text-white truncate">
                            {downloadRadar.filename}
                          </span>
                        </div>
                        <button
                          type="button"
                          onClick={onOpenDownloadedFile}
                          className="px-2.5 py-0.5 rounded-full bg-[#34C759] text-black text-[10px] font-bold hover:bg-[#34C759]/90 cursor-pointer shrink-0"
                        >
                          Open File
                        </button>
                      </>
                    ) : (
                      <>
                        <ArrowDownToLine className="w-3.5 h-3.5 text-sky-400 animate-bounce shrink-0" />
                        <span className="text-[10.5px] font-medium text-white/90 truncate">
                          {downloadRadar.filename}
                        </span>
                        <span className="text-[10px] text-sky-300 font-semibold tabular-nums shrink-0">
                          {Math.round(downloadRadar.progress)}% · {downloadRadar.speedMbps} MB/s
                        </span>
                        <div
                          className="pointer-events-none absolute bottom-0 left-0 h-[2px] bg-sky-400 transition-all duration-150"
                          style={{ width: `${downloadRadar.progress}%` }}
                        />
                      </>
                    )}
                  </div>
                ) : compactMode === 'calendar' ? (
                  /* Calendar Alert Pill */
                  <div
                    className="w-full flex items-center justify-between gap-2"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <CalendarClock className="w-3.5 h-3.5 text-[#FF9500] shrink-0" />
                      <span className="text-[11px] font-medium text-white truncate">
                        Next: {calendarRadar.title} in {calendarRadar.startsInMin}m
                      </span>
                    </div>
                    <button
                      type="button"
                      onClick={onJoinCalendarMeet}
                      className={`px-2.5 py-0.5 rounded-full text-[10px] font-semibold flex items-center gap-1 shrink-0 cursor-pointer transition-colors ${
                        calendarRadar.joined
                          ? 'bg-[#34C759]/20 text-[#34C759] border border-[#34C759]/40'
                          : 'bg-[#34C759] text-black hover:bg-[#34C759]/90'
                      }`}
                    >
                      <Video className="w-2.5 h-2.5" />
                      {calendarRadar.joined ? 'Joined' : 'Join'}
                    </button>
                  </div>
                ) : compactMode === 'media' || (isSplit && playing) ? (
                  /* Dynamic Breathing Lyric Pill OR Compact Marquee */
                  !isInstrumentalBreak && activeLyricLine?.text && shibaSettings.liveLyricTicker !== false ? (
                    /* Living Lyric Ticker */
                    <>
                      <ShibaMascot
                        state="music"
                        presenceLevel={shibaSettings.presenceLevel}
                        isIslandHovered={isHovered}
                        micMuted={micMuted}
                        size="sm"
                        onPet={onPetShiba}
                      />

                      <div className="flex-1 min-w-0 px-2 text-center overflow-hidden">
                        <AnimatePresence mode="wait">
                          <motion.p
                            key={`${activeLyricLine.timeMs}-${activeLyricLine.text}`}
                            initial={{ y: 6, opacity: 0 }}
                            animate={{ y: 0, opacity: 1 }}
                            exit={{ y: -6, opacity: 0 }}
                            transition={{ type: 'spring', stiffness: 360, damping: 26 }}
                            className="text-[11.5px] font-semibold text-white tracking-tight truncate leading-none"
                          >
                            {activeLyricLine.text}
                          </motion.p>
                        </AnimatePresence>
                      </div>

                      <Equalizer
                        playing={playing}
                        bpm={track.bpm}
                        color={track.accentColor}
                        size="sm"
                      />
                    </>
                  ) : (
                    /* Standard Media Marquee */
                    <>
                      <div
                        className="w-[22px] h-[22px] rounded-full shrink-0 overflow-hidden border border-white/20 shadow-sm flex items-center justify-center"
                        style={{
                          background: `linear-gradient(135deg, ${track.coverGradient[0]}, ${track.coverGradient[1]}, ${track.coverGradient[2]})`,
                        }}
                      >
                        <div
                          className={`w-full h-full flex items-center justify-center ${
                            playing ? 'animate-isle-spin' : ''
                          }`}
                        >
                          <div className="w-1.5 h-1.5 rounded-full bg-black/70 border border-white/40" />
                        </div>
                      </div>

                      <div
                        className="flex-1 min-w-0 overflow-hidden"
                        style={{
                          maskImage:
                            'linear-gradient(to right, transparent 0%, black 10%, black 90%, transparent 100%)',
                          WebkitMaskImage:
                            'linear-gradient(to right, transparent 0%, black 10%, black 90%, transparent 100%)',
                        }}
                      >
                        <span className="animate-isle-marquee text-[11.5px] font-medium text-white/95 tracking-tight">
                          {track.title} · {track.artist}
                        </span>
                      </div>

                      <Equalizer
                        playing={playing}
                        bpm={track.bpm}
                        color={track.accentColor}
                        size="sm"
                      />
                    </>
                  )
                ) : (
                  /* Idle Compact Pill: Shiba + Clock + Dynamic Privacy Micro-Dot + Battery */
                  <>
                    {!isStealth && (
                      <ShibaMascot
                        state={shibaState}
                        presenceLevel={shibaSettings.presenceLevel}
                        isIslandHovered={isHovered}
                        deepWorkZen={isDeepWorkActive}
                        dnd={shibaSettings.dnd}
                        micMuted={micMuted}
                        size="sm"
                        onPet={onPetShiba}
                        onToggleMicMute={onToggleMicMute}
                      />
                    )}

                    <span className="text-[12px] font-semibold text-white/90 tabular-nums tracking-tight">
                      {formatClock(now)}
                    </span>

                    <div
                      className="flex items-center gap-2 shrink-0"
                      onClick={(e) => e.stopPropagation()}
                    >
                      {/* Section 1.3: Dynamic Sensor Privacy Micro-Dot (Camera: Green, Mic: Amber) */}
                      <AnimatePresence>
                        {sensorAccess?.active && (
                          <motion.span
                            key={`sensor-dot-${sensorAccess.type}`}
                            initial={{ scale: 0, opacity: 0, y: -4 }}
                            animate={{ scale: 1, opacity: 1, y: 0 }}
                            exit={{ scale: 0, opacity: 0, y: -8 }}
                            transition={{ type: 'spring', stiffness: 450, damping: 25 }}
                            className={`w-2 h-2 rounded-full shrink-0 ${
                              sensorAccess.type === 'camera'
                                ? 'bg-[#34C759]'
                                : sensorAccess.type === 'mic'
                                  ? 'bg-[#FF9500]'
                                  : 'bg-[#007AFF]'
                            }`}
                            title={`${sensorAccess.type.toUpperCase()} in use by ${sensorAccess.app}`}
                          />
                        )}
                      </AnimatePresence>

                      {/* Clickable Battery Indicator */}
                      <button
                        type="button"
                        onClick={onCyclePowerPlan}
                        className="flex items-center gap-1 px-1 py-0.5 rounded-full hover:bg-white/10 transition-colors cursor-pointer"
                        title={`Battery ${batteryPct}% · Plan: ${POWER_PLAN_META[powerPlan].label}`}
                        aria-label="Switch power plan"
                      >
                        {charging ? (
                          <BatteryCharging className="w-3.5 h-3.5 text-[#34C759]" />
                        ) : batteryPct < 20 ? (
                          <BatteryWarning className="w-3.5 h-3.5 text-[#FF3B30]" />
                        ) : (
                          <Battery
                            className="w-3.5 h-3.5"
                            style={{ color: POWER_PLAN_META[powerPlan].color }}
                          />
                        )}
                        <span className="text-[10.5px] font-medium text-white/75 tabular-nums">
                          {batteryPct}%
                        </span>
                      </button>
                    </div>
                  </>
                )}
              </motion.div>
            ) : (
              /* Expanded Island Card */
              <motion.div
                key="expanded-card"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.18 }}
                className="relative z-10 w-full h-full flex flex-col justify-between"
              >
                {/* Clean Apple Header:
                    Left: Shiba Mascot
                    Center: Clean Active Section pill with Lucide Icon (Zero neon dots)
                    Right: Focus DND Moon toggle
                */}
                <div className="h-[38px] px-4 pt-1.5 flex items-center justify-between shrink-0">
                  {/* Left: Mascot */}
                  <div className="flex items-center gap-2">
                    {!isStealth && (
                      <ShibaMascot
                        state={shibaState}
                        presenceLevel={shibaSettings.presenceLevel}
                        isIslandHovered={isHovered}
                        deepWorkZen={isDeepWorkActive}
                        dnd={shibaSettings.dnd}
                        micMuted={micMuted}
                        size="sm"
                        onPet={onPetShiba}
                        onToggleMicMute={onToggleMicMute}
                      />
                    )}
                  </div>

                  {/* Center: Clean Active Section Pill */}
                  <div className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-white/[0.06] border border-white/[0.08] shadow-sm">
                    <TabIcon
                      tabKey={activeTabKey}
                      className="w-3.5 h-3.5 text-white/80"
                    />
                    <span className="text-[11.5px] font-semibold text-white/95">
                      {currentTabMeta?.label}
                    </span>
                  </div>

                  {/* Right: Focus DND Moon Button */}
                  <div
                    className="flex items-center gap-1.5 shrink-0"
                    onPointerDown={(e) => e.stopPropagation()}
                  >
                    <motion.button
                      type="button"
                      whileTap={{ scale: 0.88 }}
                      onClick={onToggleDnd}
                      className={`w-6 h-6 rounded-full border flex items-center justify-center cursor-pointer transition-colors ${
                        shibaSettings.dnd
                          ? 'bg-indigo-400/25 border-indigo-400/45 text-indigo-200'
                          : 'bg-white/[0.07] hover:bg-white/[0.15] border-white/10 text-white/60 hover:text-white'
                      }`}
                      title={shibaSettings.dnd ? 'Focus DND Active' : 'Do Not Disturb'}
                    >
                      <Moon className="w-3 h-3" />
                    </motion.button>
                  </div>
                </div>

                {/* Center Stage: Active Tab with Spring Sliding Transitions */}
                <div className="relative flex-1 w-full overflow-hidden px-1">
                  <AnimatePresence mode="wait" custom={slideDirection}>
                    <motion.div
                      key={activeTabKey}
                      custom={slideDirection}
                      initial={{ x: slideDirection > 0 ? 28 : -28, opacity: 0 }}
                      animate={{ x: 0, opacity: 1 }}
                      exit={{ x: slideDirection > 0 ? -28 : 28, opacity: 0 }}
                      transition={{ type: 'spring', stiffness: 420, damping: 32 }}
                      className="w-full h-full"
                    >
                      {renderTabContent(activeTabKey)}
                    </motion.div>
                  </AnimatePresence>
                </div>

                {/* Bottom Dock: Minimalist White Dots <-> Hover Expanded Frosted Glass Icon Dock */}
                <div
                  onMouseEnter={() => setDockHovered(true)}
                  onMouseLeave={() => setDockHovered(false)}
                  onPointerDown={(e) => e.stopPropagation()}
                  className="relative w-full h-[34px] flex items-center justify-center pb-1.5 z-30"
                >
                  <AnimatePresence mode="wait">
                    {!dockHovered ? (
                      /* Default State: Minimalist White Dots */
                      <motion.div
                        key="minimalist-dots"
                        initial={{ opacity: 0, scale: 0.85 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.85 }}
                        transition={{ duration: 0.14 }}
                        className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-black/35 backdrop-blur-md border border-white/[0.08]"
                      >
                        {enabledTabs.map((tab) => {
                          const isActive = tab.key === activeTabKey;
                          return (
                            <motion.button
                              key={tab.key}
                              type="button"
                              onClick={() => {
                                audioEngine.playClick();
                                onSelectTab(tab.key);
                              }}
                              className="cursor-pointer rounded-full"
                              animate={{
                                width: isActive ? 18 : 5,
                                height: 5,
                                backgroundColor: isActive ? '#FFFFFF' : 'rgba(255,255,255,0.3)',
                              }}
                              transition={{ type: 'spring', stiffness: 420, damping: 28 }}
                              title={tab.label}
                            />
                          );
                        })}
                      </motion.div>
                    ) : (
                      /* Hover State: Frosted Glass Icon Dock for Consolidated 7 Tabs */
                      <motion.div
                        key="expanded-dock"
                        initial={{ opacity: 0, y: 10, scale: 0.92 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: 8, scale: 0.92 }}
                        transition={{ type: 'spring', stiffness: 440, damping: 28 }}
                        className="flex items-center gap-1 px-2.5 py-1 rounded-full bg-zinc-950/92 backdrop-blur-2xl border border-white/20 shadow-xl"
                      >
                        {enabledTabs.map((tab) => {
                          const isActive = tab.key === activeTabKey;
                          return (
                            <motion.button
                              key={tab.key}
                              type="button"
                              whileHover={{ scale: 1.15 }}
                              whileTap={{ scale: 0.88 }}
                              onClick={() => {
                                audioEngine.playClick();
                                onSelectTab(tab.key);
                              }}
                              className={`relative group p-1.5 rounded-full transition-colors cursor-pointer ${
                                isActive
                                  ? 'bg-white/20 text-white'
                                  : 'text-white/50 hover:text-white hover:bg-white/10'
                              }`}
                              title={tab.label}
                            >
                              <TabIcon tabKey={tab.key} className="w-3.5 h-3.5" />
                              <span className="pointer-events-none absolute -top-7 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-opacity px-2 py-0.5 rounded-md bg-black/90 border border-white/15 text-[9.5px] font-semibold text-white whitespace-nowrap shadow-lg z-40">
                                {tab.label}
                              </span>
                            </motion.button>
                          );
                        })}

                        {/* Settings Button in Dock */}
                        <div className="w-px h-3.5 bg-white/15 mx-0.5" />
                        <motion.button
                          type="button"
                          whileHover={{ scale: 1.15 }}
                          whileTap={{ scale: 0.88 }}
                          onClick={() => {
                            audioEngine.playClick();
                            onOpenSettings();
                          }}
                          className="relative group p-1.5 rounded-full text-white/50 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
                          title="Settings & Tab Reordering"
                        >
                          <Settings className="w-3.5 h-3.5" />
                          <span className="pointer-events-none absolute -top-7 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-opacity px-2 py-0.5 rounded-md bg-black/90 border border-white/15 text-[9.5px] font-semibold text-white whitespace-nowrap shadow-lg z-40">
                            Settings
                          </span>
                        </motion.button>
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </motion.div>

        {/* Split Island Circular Focus Badge (36x36) */}
        <AnimatePresence>
          {isSplit && (
            <motion.button
              key="split-focus-badge"
              type="button"
              onClick={() => {
                audioEngine.playPop();
                onExpand('focus');
              }}
              initial={{ opacity: 0, scale: 0.4, width: 0, marginLeft: 0 }}
              animate={{
                opacity: 1,
                scale: 1,
                width: ISLAND_METRICS.splitBadge.width,
                marginLeft: ISLAND_METRICS.splitGap,
              }}
              exit={{ opacity: 0, scale: 0.4, width: 0, marginLeft: 0 }}
              transition={ISLAND_SPRING}
              style={{
                height: ISLAND_METRICS.splitBadge.height,
                borderTopLeftRadius: mountMode === 'notch' ? 0 : 18,
                borderTopRightRadius: mountMode === 'notch' ? 0 : 18,
                borderBottomLeftRadius: 18,
                borderBottomRightRadius: 18,
              }}
              className={`relative overflow-hidden shrink-0 flex items-center justify-center cursor-pointer border border-white/10 ${glassClass}`}
              title="Focus timer running — click to open Focus tab"
              aria-label="Focus timer badge"
            >
              <svg className="w-7 h-7 -rotate-90" viewBox="0 0 30 30">
                <circle
                  cx="15"
                  cy="15"
                  r={badgeRadius}
                  fill="none"
                  stroke="rgba(255,255,255,0.14)"
                  strokeWidth="2.5"
                />
                <motion.circle
                  cx="15"
                  cy="15"
                  r={badgeRadius}
                  fill="none"
                  stroke={focusCfg.color}
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeDasharray={badgeCirc}
                  animate={{ strokeDashoffset: badgeOffset }}
                  transition={{ duration: 0.4 }}
                />
              </svg>
              <span className="absolute text-[8.5px] font-bold text-white tabular-nums">
                {Math.ceil(focus.remainingSec / 60)}m
              </span>
            </motion.button>
          )}
        </AnimatePresence>
      </div>
    </motion.div>
  );
};
