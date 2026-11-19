import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DesktopCanvas } from './components/DesktopCanvas';
import { DynamicIsland } from './components/DynamicIsland';
import { SettingsModal } from './components/SettingsModal';
import {
  FOCUS_PHASE_CONFIG,
  PRESET_TRACKS,
} from './constants';
import { useShibaBehavior } from './hooks/useShibaBehavior';
import { fetchLrclibLyrics, parseLrc } from './services/lyrics';
import {
  ClipboardStorage,
  getTodayKey,
  HydrationStorage,
  SettingsStorage,
  ShelfStorage,
  TasksStorage,
} from './services/storage';
import type {
  BatteryActionEffect,
  CalendarRadarState,
  CliAgentActivity,
  CliAgentStatus,
  ClipboardItem,
  DownloadRadarState,
  ExpandTrigger,
  FocusPhase,
  FocusState,
  HudOverlay,
  IslandVisualState,
  LyricsState,
  MountMode,
  PeripheralType,
  PowerPlan,
  ScrcpyState,
  ScreenshotRecord,
  SensorAccessState,
  SensorType,
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
} from './types';
import { audioEngine } from './utils/audioEngine';

function inferFileKind(name: string): ShelfFile['kind'] {
  const lower = name.toLowerCase();
  if (/\.(png|jpe?g|webp|gif|svg|fig)$/.test(lower)) return 'image';
  if (/\.(ts|tsx|js|jsx|rs|py|json|css|html)$/.test(lower)) return 'code';
  if (/\.(zip|tar|gz|7z|rar)$/.test(lower)) return 'archive';
  if (/\.(pdf|md|txt|docx?)$/.test(lower)) return 'doc';
  return 'other';
}

export function App() {
  // Desktop simulation controls
  const [wallpaper, setWallpaper] = useState<WallpaperId>('transparent');
  const [mountMode, setMountMode] = useState<MountMode>('notch');
  const [expandTrigger, setExpandTrigger] = useState<ExpandTrigger>('click');
  const [expanded, setExpanded] = useState(false);
  const [simulatedFullscreen, setSimulatedFullscreen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [showDevSimulator, setShowDevSimulator] = useState(false);

  // Tab configuration & active tab (Consolidated 7 Tabs)
  const [tabConfig, setTabConfig] = useState<TabConfigItem[]>(() =>
    SettingsStorage.loadTabConfig()
  );
  const enabledTabs = useMemo(() => tabConfig.filter((t) => t.enabled), [tabConfig]);
  const [activeTabKey, setActiveTabKey] = useState<TabKey>('media');

  useEffect(() => {
    SettingsStorage.saveTabConfig(tabConfig);
    if (!enabledTabs.some((t) => t.key === activeTabKey) && enabledTabs[0]) {
      setActiveTabKey(enabledTabs[0].key);
    }
  }, [tabConfig, enabledTabs, activeTabKey]);

  // Shiba Settings
  const [shibaSettings, setShibaSettings] = useState<ShibaSettings>(() =>
    SettingsStorage.loadSettings()
  );
  useEffect(() => {
    SettingsStorage.saveSettings(shibaSettings);
  }, [shibaSettings]);

  const updateShibaSettings = useCallback((patch: Partial<ShibaSettings>) => {
    setShibaSettings((prev) => ({ ...prev, ...patch }));
  }, []);

  // System Hardware & HUD State
  const [systemVolume, setSystemVolume] = useState(72);
  const [, setBrightness] = useState(80);
  const [, setCapsLockOn] = useState(false);
  const [micActive] = useState(true);
  const [micMuted, setMicMuted] = useState(false);
  const [batteryPct, setBatteryPct] = useState(84);
  const [charging, setCharging] = useState(false);
  const [batteryEffect, setBatteryEffect] = useState<BatteryActionEffect>(null);
  const [powerPlan, setPowerPlan] = useState<PowerPlan>('balanced');
  const [moodOverride, setMoodOverride] = useState<ShibaState | null>(null);
  const [hudOverlay, setHudOverlay] = useState<HudOverlay | null>(null);
  const hudTimerRef = useRef<number | null>(null);

  const showHudOverlay = useCallback((overlay: HudOverlay, durationMs = 1500) => {
    setHudOverlay(overlay);
    if (hudTimerRef.current !== null) window.clearTimeout(hudTimerRef.current);
    hudTimerRef.current = window.setTimeout(() => {
      setHudOverlay(null);
    }, durationMs);
  }, []);

  // Section 1: Claude Code / CLI Coding Agent Live Monitor State
  const [cliAgent, setCliAgent] = useState<CliAgentActivity>({
    status: 'idle',
    agentName: 'Claude Code',
    headline: 'Standby',
    timestamp: Date.now(),
  });

  // Section 5.1: Screenshot Auto-Capture Records
  const [screenshots, setScreenshots] = useState<ScreenshotRecord[]>([]);
  const [screenshotHiding, setScreenshotHiding] = useState(false);

  // Section 1.3: Dynamic Sensor Privacy State (Android 15 / iOS Parity)
  const [sensorAccess, setSensorAccess] = useState<SensorAccessState | null>(null);
  const sensorTimerRef = useRef<number | null>(null);

  const handleTriggerSensorAccess = useCallback(
    (type: SensorType, app = 'Zoom') => {
      audioEngine.playPop();
      setSensorAccess({
        active: true,
        type,
        app,
        startTime: Date.now(),
      });
      showHudOverlay({ kind: 'sensorToast', sensor: type, app }, 2500);

      if (sensorTimerRef.current !== null) window.clearTimeout(sensorTimerRef.current);
      sensorTimerRef.current = window.setTimeout(() => {
        setSensorAccess(null);
      }, 12000);
    },
    [showHudOverlay]
  );

  const handleClearSensorAccess = useCallback(() => {
    audioEngine.playClick();
    setSensorAccess(null);
    if (sensorTimerRef.current !== null) window.clearTimeout(sensorTimerRef.current);
  }, []);

  // Section 2.3: Lyrics Pane Reveal State
  const [showLyrics, setShowLyrics] = useState(false);

  // Trigger Screenshot Auto-Capture Hook (Section 2 PrtSc Auto-Hide Toggle Support)
  const captureScreenshot = useCallback(() => {
    const executeCapture = () => {
      audioEngine.playPop();
      const now = Date.now();
      const dateStr = new Date().toISOString().slice(0, 10);
      const snipSeq = String(Math.floor(Math.random() * 900) + 100);
      const fileName = `Screenshot_${dateStr}_${snipSeq}.png`;
      const filePath = `C:/Users/Ronn/Pictures/Screenshots/${fileName}`;

      const record: ScreenshotRecord = {
        id: `snip_${now}`,
        type: 'screenshot',
        filePath,
        fileName,
        timestamp: now,
      };

      setScreenshots((prev) => [record, ...prev].slice(0, 8));
      showHudOverlay({ kind: 'screenshot', record }, 2500);
    };

    if (shibaSettings.autoHideOnScreenshot) {
      setScreenshotHiding(true);
      window.setTimeout(() => {
        setScreenshotHiding(false);
        executeCapture();
      }, 300);
    } else {
      executeCapture();
    }
  }, [shibaSettings.autoHideOnScreenshot, showHudOverlay]);

  // Section 5.4: ThinkPad F4 Mic Mute Handler
  const handleToggleMicMute = useCallback(() => {
    audioEngine.playClick();
    setMicMuted((prev) => {
      const next = !prev;
      showHudOverlay({ kind: 'mic', muted: next }, 1600);
      return next;
    });
  }, [showHudOverlay]);

  // Real Hardware hotkeys: CapsLock, ThinkPad F4 Mic Mute, PrintScreen
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'CapsLock') {
        const next = e.getModifierState('CapsLock');
        setCapsLockOn(next);
        showHudOverlay({ kind: 'capsLock', enabled: next }, 1600);
      } else if (e.key === 'F4') {
        e.preventDefault();
        handleToggleMicMute();
      } else if (e.key === 'PrintScreen') {
        e.preventDefault();
        captureScreenshot();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [showHudOverlay, handleToggleMicMute, captureScreenshot]);

  // Media & LRCLIB state
  const [tracks] = useState<Track[]>(PRESET_TRACKS);
  const [trackIndex, setTrackIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [positionSec, setPositionSec] = useState(PRESET_TRACKS[0].initialPositionSec);
  const [volume, setVolume] = useState(0.45);
  const localAudioElRef = useRef<HTMLAudioElement | null>(null);

  const currentTrack = tracks[trackIndex] ?? tracks[0];

  const [lyrics, setLyrics] = useState<LyricsState>(() => ({
    lines: parseLrc(PRESET_TRACKS[0].fallbackLrc),
    source: 'loading',
    synced: true,
    trackKey: `${PRESET_TRACKS[0].title}::${PRESET_TRACKS[0].artist}`,
  }));
  const lyricsCacheRef = useRef<Map<string, LyricsState>>(new Map());

  // Fetch live LRCLIB synced lyrics
  useEffect(() => {
    const key = `${currentTrack.title}::${currentTrack.artist}`;
    const cached = lyricsCacheRef.current.get(key);
    if (cached) {
      setLyrics(cached);
      return;
    }

    const controller = new AbortController();
    const fallbackLines = parseLrc(currentTrack.fallbackLrc);

    setLyrics({
      lines: fallbackLines,
      source: 'loading',
      synced: fallbackLines.length > 0,
      trackKey: key,
    });

    void fetchLrclibLyrics(
      currentTrack.title,
      currentTrack.artist,
      currentTrack.durationSec,
      controller.signal
    ).then((result) => {
      if (controller.signal.aborted) return;
      const nextState: LyricsState =
        result && result.lines.length > 0
          ? {
              lines: result.lines,
              source: 'lrclib',
              synced: true,
              trackKey: key,
            }
          : {
              lines: fallbackLines,
              source: 'fallback',
              synced: fallbackLines.length > 0,
              trackKey: key,
            };
      lyricsCacheRef.current.set(key, nextState);
      setLyrics(nextState);
    });

    return () => controller.abort();
  }, [currentTrack.id, currentTrack.title, currentTrack.artist, currentTrack.durationSec, currentTrack.fallbackLrc]);

  // Audio engine playback synchronization
  useEffect(() => {
    if (localAudioElRef.current) {
      if (playing) {
        void localAudioElRef.current.play().catch(() => {});
      } else {
        localAudioElRef.current.pause();
      }
      return;
    }

    if (playing) {
      audioEngine.startAmbient(currentTrack.bpm, currentTrack.synthChord, volume);
    } else {
      audioEngine.stopAmbient();
    }
    return () => {
      audioEngine.stopAmbient();
    };
  }, [playing, currentTrack.id, currentTrack.bpm, currentTrack.synthChord, volume]);

  useEffect(() => {
    audioEngine.setVolume(volume);
  }, [volume]);

  // Playback timer progression
  useEffect(() => {
    if (!playing || localAudioElRef.current) return;
    const id = window.setInterval(() => {
      setPositionSec((prev) => {
        const next = prev + 0.2;
        if (next >= currentTrack.durationSec) {
          setTrackIndex((idx) => (idx + 1) % tracks.length);
          return 0;
        }
        return next;
      });
    }, 200);
    return () => window.clearInterval(id);
  }, [playing, currentTrack.durationSec, tracks.length]);

  const handleSelectTrack = useCallback(
    (nextIdx: number) => {
      if (localAudioElRef.current) {
        localAudioElRef.current.pause();
        localAudioElRef.current = null;
      }
      const normalized = (nextIdx + tracks.length) % tracks.length;
      setTrackIndex(normalized);
      setPositionSec(tracks[normalized].initialPositionSec);
    },
    [tracks]
  );

  const handleSeek = useCallback((sec: number) => {
    setPositionSec(sec);
    if (localAudioElRef.current) {
      localAudioElRef.current.currentTime = sec;
    }
  }, []);

  // Focus / Pomodoro state
  const [focus, setFocus] = useState<FocusState>({
    phase: 'work',
    remainingSec: FOCUS_PHASE_CONFIG.work.durationSec,
    totalSec: FOCUS_PHASE_CONFIG.work.durationSec,
    running: false,
    completedSessions: 0,
  });
  const [focusCelebrating, setFocusCelebrating] = useState(false);

  // Water Tracker state
  const [water, setWater] = useState<WaterState>(() => HydrationStorage.load());
  const [lastWaterLoggedAt, setLastWaterLoggedAt] = useState(() => Date.now());

  useEffect(() => {
    HydrationStorage.save(water);
  }, [water]);

  // Zen Breathing state
  const [zenActive, setZenActive] = useState(false);

  // Shiba Attention & Etiquette Engine
  const {
    shibaState,
    isHiddenFullscreen,
    isDeepWorkActive,
    triggerHappyBurst,
    triggerHydrationNudge,
    triggerEyeRestNudge,
  } = useShibaBehavior({
    settings: shibaSettings,
    simulatedFullscreen,
    pomodoroRunning: focus.running,
    musicPlaying: playing,
    batteryPct,
    charging,
    lastWaterLoggedAt,
    zenBreathingActive: zenActive,
    moodOverride,
    onTriggerNudgeHud: showHudOverlay,
  });

  const completeFocusPhase = useCallback(() => {
    audioEngine.playSingingBowl();
    setFocusCelebrating(true);
    triggerHappyBurst(2600);
    window.setTimeout(() => setFocusCelebrating(false), 1400);

    setFocus((prev) => {
      const nextCompleted =
        prev.phase === 'work' ? prev.completedSessions + 1 : prev.completedSessions;
      const nextPhase: FocusPhase =
        prev.phase === 'work'
          ? nextCompleted % 4 === 0
            ? 'longBreak'
            : 'shortBreak'
          : 'work';
      const nextTotal = FOCUS_PHASE_CONFIG[nextPhase].durationSec;
      return {
        phase: nextPhase,
        remainingSec: nextTotal,
        totalSec: nextTotal,
        running: false,
        completedSessions: nextCompleted,
      };
    });
  }, [triggerHappyBurst]);

  useEffect(() => {
    if (!focus.running) return;
    const id = window.setInterval(() => {
      setFocus((prev) => {
        if (!prev.running) return prev;
        if (prev.remainingSec <= 1) {
          window.setTimeout(() => completeFocusPhase(), 0);
          return { ...prev, remainingSec: 0, running: false };
        }
        return { ...prev, remainingSec: prev.remainingSec - 1 };
      });
    }, 1000);
    return () => window.clearInterval(id);
  }, [focus.running, completeFocusPhase]);

  const handleAddWater = useCallback(
    (ml: number) => {
      audioEngine.playWaterDroplet();
      setLastWaterLoggedAt(Date.now());
      const today = getTodayKey();
      setWater((prev) => {
        const base = prev.date === today ? prev : { ...prev, date: today, intakeMl: 0, history: [] };
        const nextIntake = base.intakeMl + ml;
        if (base.intakeMl < base.goalMl && nextIntake >= base.goalMl) {
          triggerHappyBurst(2400);
        }
        return {
          ...base,
          intakeMl: nextIntake,
          history: [...base.history, ml],
        };
      });
    },
    [triggerHappyBurst]
  );

  const handleUndoWater = useCallback(() => {
    audioEngine.playClick();
    setWater((prev) => {
      if (prev.history.length > 0) {
        const last = prev.history[prev.history.length - 1];
        return {
          ...prev,
          intakeMl: Math.max(0, prev.intakeMl - last),
          history: prev.history.slice(0, -1),
        };
      }
      return { ...prev, intakeMl: Math.max(0, prev.intakeMl - 250) };
    });
  }, []);

  const handleResetWater = useCallback(() => {
    audioEngine.playClick();
    setWater((prev) => ({ ...prev, date: getTodayKey(), intakeMl: 0, history: [] }));
  }, []);

  // Tasks state
  const [tasks, setTasks] = useState<TaskItem[]>(() => TasksStorage.load());
  useEffect(() => {
    TasksStorage.save(tasks);
  }, [tasks]);

  const handleToggleTask = useCallback(
    (id: string) => {
      audioEngine.playClick();
      setTasks((prev) =>
        prev.map((t) => {
          if (t.id === id) {
            if (!t.completed) triggerHappyBurst(1400);
            return { ...t, completed: !t.completed };
          }
          return t;
        })
      );
    },
    [triggerHappyBurst]
  );

  const handleAddTask = useCallback((title: string, color: TaskPriorityColor) => {
    audioEngine.playClick();
    setTasks((prev) => [
      {
        id: `task-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        title,
        completed: false,
        color,
        createdAt: Date.now(),
      },
      ...prev,
    ]);
  }, []);

  const handleDeleteTask = useCallback((id: string) => {
    audioEngine.playClick();
    setTasks((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const handleClearTasksCompleted = useCallback(() => {
    audioEngine.playClick();
    setTasks((prev) => TasksStorage.clearCompleted(prev));
  }, []);

  // Shelf & Scrcpy state
  const [shelfFiles, setShelfFiles] = useState<ShelfFile[]>(() => ShelfStorage.load());
  const [scrcpy, setScrcpy] = useState<ScrcpyState>({
    deviceName: 'Pixel 9 Pro',
    batteryPct: 84,
    connected: true,
    mirroring: false,
    recording: false,
    recordingSec: 0,
    wirelessAdb: false,
    adbAddress: '192.168.1.42:5555',
    toast: null,
  });

  useEffect(() => {
    ShelfStorage.save(shelfFiles);
  }, [shelfFiles]);

  useEffect(() => {
    if (!scrcpy.recording) return;
    const id = window.setInterval(() => {
      setScrcpy((prev) => ({ ...prev, recordingSec: prev.recordingSec + 1 }));
    }, 1000);
    return () => window.clearInterval(id);
  }, [scrcpy.recording]);

  const showScrcpyToast = useCallback((msg: string) => {
    setScrcpy((prev) => ({ ...prev, toast: msg }));
    window.setTimeout(() => {
      setScrcpy((prev) => (prev.toast === msg ? { ...prev, toast: null } : prev));
    }, 1800);
  }, []);

  const handleAddShelfFiles = useCallback((incoming: FileList | ShelfFile[]) => {
    audioEngine.playPop();
    const items: ShelfFile[] = Array.isArray(incoming)
      ? incoming
      : Array.from(incoming).map((f) => ({
          id: `file-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
          name: f.name,
          sizeBytes: f.size,
          kind: inferFileKind(f.name),
          addedAt: Date.now(),
        }));
    setShelfFiles((prev) => [...items, ...prev].slice(0, 8));
  }, []);

  // Clipboard History Hub state
  const [clipboardItems, setClipboardItems] = useState<ClipboardItem[]>(() =>
    ClipboardStorage.load()
  );
  useEffect(() => {
    ClipboardStorage.save(clipboardItems);
  }, [clipboardItems]);

  const handleToggleClipboardPin = useCallback((id: string) => {
    audioEngine.playClick();
    setClipboardItems((prev) =>
      prev.map((item) => (item.id === id ? { ...item, pinned: !item.pinned } : item))
    );
  }, []);

  const handleCopyClipboardItem = useCallback((item: ClipboardItem) => {
    audioEngine.playClick();
    try {
      void navigator.clipboard?.writeText(item.content);
    } catch {
      // Ignore
    }
  }, []);

  // Section 5.1: Screenshot Actions
  const handleOpenScreenshot = useCallback((record: ScreenshotRecord) => {
    audioEngine.playPop();
    showHudOverlay(
      { kind: 'screenshot', record },
      2000
    );
  }, [showHudOverlay]);

  const handleCopyScreenshotPath = useCallback((record: ScreenshotRecord) => {
    audioEngine.playClick();
    try {
      void navigator.clipboard?.writeText(record.filePath);
    } catch {
      // Ignore
    }
    showHudOverlay(
      { kind: 'screenshot', record: { ...record, fileName: 'Path Copied!' } },
      1400
    );
  }, [showHudOverlay]);

  // Live Download & Calendar Radars
  const [downloadRadar, setDownloadRadar] = useState<DownloadRadarState>({
    active: false,
    filename: 'downloading-update.iso',
    progress: 0,
    speedMbps: 18.4,
    etaSec: 14,
    completed: false,
  });

  useEffect(() => {
    if (!downloadRadar.active || downloadRadar.completed) return;
    const id = window.setInterval(() => {
      setDownloadRadar((prev) => {
        if (!prev.active || prev.completed) return prev;
        const nextProg = Math.min(100, prev.progress + 8);
        const done = nextProg >= 100;
        if (done) {
          audioEngine.playSingingBowl();
          triggerHappyBurst(2200);
        }
        return {
          ...prev,
          progress: nextProg,
          speedMbps: Number((16.5 + Math.random() * 4.2).toFixed(1)),
          etaSec: Math.max(0, Math.ceil(((100 - nextProg) / 100) * 14)),
          completed: done,
        };
      });
    }, 320);
    return () => window.clearInterval(id);
  }, [downloadRadar.active, downloadRadar.completed, triggerHappyBurst]);

  const [calendarRadar, setCalendarRadar] = useState<CalendarRadarState>({
    active: false,
    title: 'Sprint Review',
    startsInMin: 8,
    meetUrl: 'https://meet.google.com',
    joined: false,
  });

  // System HUD Handlers with Over-Scroll "Wall Bounce" Physics (Section 1.6)
  const handleWheelVolume = useCallback(
    (deltaY: number) => {
      const step = deltaY < 0 ? 4 : -4;
      setSystemVolume((prev) => {
        const overscroll = prev >= 100 && step > 0 ? 'max' : prev <= 0 && step < 0 ? 'min' : null;
        const next = Math.min(100, Math.max(0, prev + step));
        setVolume(next / 100);
        if (overscroll) {
          audioEngine.playPop();
        }
        showHudOverlay({ kind: 'volume', value: next, muted: next === 0, overscroll }, 1400);
        return next;
      });
    },
    [showHudOverlay]
  );

  const handleTriggerVolumeHud = useCallback(
    (overscroll?: 'min' | 'max') => {
      setExpanded(false);
      const targetVal = overscroll === 'max' ? 100 : overscroll === 'min' ? 0 : systemVolume;
      if (overscroll) {
        setSystemVolume(targetVal);
        setVolume(targetVal / 100);
        audioEngine.playPop();
      }
      showHudOverlay({ kind: 'volume', value: targetVal, muted: targetVal === 0, overscroll }, 1400);
    },
    [systemVolume, showHudOverlay]
  );

  const handleTriggerBrightnessHud = useCallback(
    (overscroll?: 'min' | 'max') => {
      setExpanded(false);
      setBrightness((prev) => {
        const target = overscroll === 'max' ? 100 : overscroll === 'min' ? 0 : (prev >= 95 ? 45 : prev + 15);
        if (overscroll) {
          audioEngine.playPop();
        }
        showHudOverlay({ kind: 'brightness', value: target, overscroll }, 1400);
        return target;
      });
    },
    [showHudOverlay]
  );

  const handleTriggerCapsLock = useCallback(() => {
    setExpanded(false);
    setCapsLockOn((prev) => {
      const next = !prev;
      showHudOverlay({ kind: 'capsLock', enabled: next }, 1600);
      return next;
    });
  }, [showHudOverlay]);

  const handleCyclePowerPlan = useCallback(() => {
    audioEngine.playClick();
    const order: PowerPlan[] = ['eco', 'balanced', 'performance'];
    setPowerPlan((prev) => {
      const next = order[(order.indexOf(prev) + 1) % order.length];
      showHudOverlay({ kind: 'powerPlan', plan: next }, 1500);
      return next;
    });
  }, [showHudOverlay]);

  // Section 1.4: Authentic Battery Charging Sweep Animation (Nusa & Apple HIG)
  const handleToggleCharging = useCallback(() => {
    setMoodOverride(null);
    setCharging((c) => {
      const next = !c;
      if (next) {
        audioEngine.playPop();
        triggerHappyBurst(1800);
        setBatteryEffect('chargingSweep');
        showHudOverlay({ kind: 'batteryNotice', effect: 'chargingSweep', message: 'Charging', pct: batteryPct }, 2800);
        window.setTimeout(() => setBatteryEffect(null), 2800);
      } else {
        audioEngine.playClick();
        setBatteryEffect('unplugged');
        showHudOverlay({ kind: 'batteryNotice', effect: 'unplugged', message: `Power Unplugged · ${batteryPct}%`, pct: batteryPct }, 1800);
        window.setTimeout(() => setBatteryEffect(null), 1800);
      }
      return next;
    });
  }, [batteryPct, showHudOverlay, triggerHappyBurst]);

  const handleTriggerBatteryFull = useCallback(() => {
    setMoodOverride(null);
    setBatteryPct(100);
    setBatteryEffect('fullGlow');
    showHudOverlay({ kind: 'batteryNotice', effect: 'fullGlow', message: 'Battery 100% Full' }, 2200);
    window.setTimeout(() => setBatteryEffect(null), 2400);
  }, [showHudOverlay]);

  const handleToggleLowBattery = useCallback(() => {
    setMoodOverride(null);
    setCharging(false);
    setBatteryPct((p) => {
      const next = p < 20 ? 84 : 14;
      if (next < 20) {
        setBatteryEffect('lowBatteryToast');
        showHudOverlay({ kind: 'batteryNotice', effect: 'lowBatteryToast', message: 'Low Battery (14%) — Connect Charger' }, 2400);
        window.setTimeout(() => setBatteryEffect(null), 2400);
      }
      return next;
    });
  }, [showHudOverlay]);

  // Section 5.2: Peripherals Detection Simulator
  const handleTriggerPeripheral = useCallback((type: PeripheralType) => {
    audioEngine.playPop();
    if (type === 'headphones') {
      showHudOverlay({
        kind: 'peripheral',
        device: {
          type: 'headphones',
          name: 'AirPods Pro',
          detail: 'L: 85% R: 90%',
          connected: true,
          batteryPct: 88,
        },
      }, 2500);
    } else if (type === 'smartphone') {
      showHudOverlay({
        kind: 'peripheral',
        device: {
          type: 'smartphone',
          name: 'Pixel 9 Pro',
          detail: '84% Connected',
          connected: true,
          batteryPct: 84,
        },
      }, 2500);
    } else {
      showHudOverlay({
        kind: 'peripheral',
        device: {
          type: 'printer',
          name: 'HP LaserJet Pro',
          detail: 'Ready (Wireless)',
          connected: true,
        },
      }, 2500);
    }
  }, [showHudOverlay]);

  // Section 1: Claude Code / CLI Agent Simulator Handlers (Authentic Logos, Zero AI Slop)
  const handleSetCliAgentStatus = useCallback(
    (status: CliAgentStatus, agentName?: string) => {
      setExpanded(false);
      const name = agentName ?? cliAgent.agentName;
      if (status === 'idle') {
        setCliAgent({
          status: 'idle',
          agentName: name,
          headline: 'Standby',
          timestamp: Date.now(),
        });
      } else if (status === 'thinking') {
        audioEngine.playPop();
        setCliAgent({
          status: 'thinking',
          agentName: name,
          headline: 'Editing window.rs...',
          detail: 'cargo check running',
          timestamp: Date.now(),
        });
      } else if (status === 'approval') {
        audioEngine.playSingingBowl();
        setCliAgent({
          status: 'approval',
          agentName: name,
          headline: 'Approval Required',
          question: "Run 'cargo test'?",
          command: 'cargo test',
          timestamp: Date.now(),
        });
      } else if (status === 'complete') {
        audioEngine.playSingingBowl();
        triggerHappyBurst(2400);
        setCliAgent({
          status: 'complete',
          agentName: name,
          headline: 'Build Passed',
          detail: '8 files modified · 0 errors',
          timestamp: Date.now(),
        });
        window.setTimeout(() => {
          setCliAgent((prev) => (prev.status === 'complete' ? { ...prev, status: 'idle' } : prev));
        }, 4000);
      } else if (status === 'error') {
        audioEngine.playClick();
        setCliAgent({
          status: 'error',
          agentName: name,
          headline: 'Test Failed',
          detail: 'Exit 1 in window_test.rs:42',
          timestamp: Date.now(),
        });
        window.setTimeout(() => {
          setCliAgent((prev) => (prev.status === 'error' ? { ...prev, status: 'idle' } : prev));
        }, 4000);
      }
    },
    [cliAgent.agentName, triggerHappyBurst]
  );

  const handleApproveAgent = useCallback(() => {
    audioEngine.playClick();
    setCliAgent({
      status: 'thinking',
      agentName: 'Claude Code',
      headline: 'Running: cargo test...',
      detail: 'Executing in terminal',
      timestamp: Date.now(),
    });
    window.setTimeout(() => {
      audioEngine.playSingingBowl();
      triggerHappyBurst(2200);
      setCliAgent({
        status: 'complete',
        agentName: 'Claude Code',
        headline: 'Tests Passed (14 ok)',
        detail: 'All assertions verified',
        timestamp: Date.now(),
      });
      window.setTimeout(() => {
        setCliAgent((prev) => (prev.status === 'complete' ? { ...prev, status: 'idle' } : prev));
      }, 3500);
    }, 1800);
  }, [triggerHappyBurst]);

  const handleRejectAgent = useCallback(() => {
    audioEngine.playClick();
    setCliAgent({
      status: 'idle',
      agentName: 'Claude Code',
      headline: 'Approval Rejected',
      timestamp: Date.now(),
    });
  }, []);

  const handleFocusTerminal = useCallback(() => {
    audioEngine.playClick();
    showHudOverlay({
      kind: 'powerPlan',
      plan: 'performance',
    }, 1200);
  }, [showHudOverlay]);

  const handleTriggerDownloadRadar = useCallback(() => {
    setExpanded(false);
    setHudOverlay(null);
    if (hudTimerRef.current !== null) window.clearTimeout(hudTimerRef.current);
    setCalendarRadar((prev) => ({ ...prev, active: false }));
    setDownloadRadar({
      active: true,
      filename: 'downloading-update.iso',
      progress: 24,
      speedMbps: 18.4,
      etaSec: 11,
      completed: false,
    });
  }, []);

  const handleOpenDownloadedFile = useCallback(() => {
    audioEngine.playPop();
    setDownloadRadar((prev) => ({ ...prev, active: false }));
    setExpanded(true);
    setActiveTabKey('shelf');
  }, []);

  const handleTriggerCalendarRadar = useCallback(() => {
    setExpanded(false);
    setHudOverlay(null);
    if (hudTimerRef.current !== null) window.clearTimeout(hudTimerRef.current);
    setDownloadRadar((prev) => ({ ...prev, active: false }));
    setCalendarRadar((prev) => ({
      ...prev,
      active: !prev.active,
      joined: false,
    }));
  }, []);

  const handleJoinCalendarMeet = useCallback(() => {
    audioEngine.playSingingBowl();
    setCalendarRadar((prev) => ({ ...prev, joined: true }));
    window.setTimeout(() => {
      setCalendarRadar((prev) => ({ ...prev, active: false }));
    }, 1600);
  }, []);

  // Derived Island Visual State
  const visualState: IslandVisualState = useMemo(() => {
    if (expanded) return 'expanded';
    if (focus.running) return 'split';
    if (playing) return 'media';
    return 'idle';
  }, [expanded, focus.running, playing]);

  const handleExpand = useCallback(
    (tabKey?: TabKey) => {
      audioEngine.playPop();
      if (tabKey && enabledTabs.some((t) => t.key === tabKey)) {
        setActiveTabKey(tabKey);
      }
      setExpanded(true);
    },
    [enabledTabs]
  );

  const handleCollapse = useCallback(() => {
    audioEngine.playPop();
    setExpanded(false);
  }, []);

  const handleTriggerStatePreset = useCallback((preset: IslandVisualState) => {
    setHudOverlay(null);
    setDownloadRadar((prev) => ({ ...prev, active: false }));
    setCalendarRadar((prev) => ({ ...prev, active: false }));
    if (preset === 'idle') {
      setExpanded(false);
      setPlaying(false);
      setFocus((prev) => ({ ...prev, running: false }));
    } else if (preset === 'media') {
      setExpanded(false);
      setPlaying(true);
      setFocus((prev) => ({ ...prev, running: false }));
    } else if (preset === 'split') {
      setExpanded(false);
      setPlaying(true);
      setFocus((prev) => ({ ...prev, running: true }));
    } else if (preset === 'expanded') {
      setExpanded(true);
    }
  }, []);

  const handleToggleTabEnabled = useCallback((key: TabKey) => {
    audioEngine.playClick();
    setTabConfig((prev) => {
      const enabledCount = prev.filter((t) => t.enabled).length;
      return prev.map((t) => {
        if (t.key !== key) return t;
        if (t.enabled && enabledCount <= 1) return t;
        return { ...t, enabled: !t.enabled };
      });
    });
  }, []);

  // Keyboard Shortcuts: Space, ArrowLeft, ArrowRight, Escape, Digits 1-7
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isInput =
        target?.tagName === 'INPUT' ||
        target?.tagName === 'TEXTAREA' ||
        target?.isContentEditable;

      if (isInput) return;

      // Developer shortcut: Ctrl + Shift + D toggles the simulator on/off
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'D' || e.key === 'd')) {
        e.preventDefault();
        audioEngine.playPop();
        setShowDevSimulator((prev) => !prev);
        return;
      }

      if (e.code === 'Space') {
        e.preventDefault();
        audioEngine.playClick();
        setPlaying((p) => !p);
        return;
      }

      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        audioEngine.playClick();
        handleSeek(Math.max(0, positionSec - 5));
        return;
      }
      if (e.key === 'ArrowRight') {
        e.preventDefault();
        audioEngine.playClick();
        handleSeek(Math.min(currentTrack.durationSec, positionSec + 5));
        return;
      }

      if (e.key === 'Escape') {
        if (settingsOpen) {
          setSettingsOpen(false);
          return;
        }
        if (expanded) {
          audioEngine.playPop();
          setExpanded(false);
          return;
        }
      }

      // Quick jump to 7 tabs via 1-7
      if (/^[1-7]$/.test(e.key)) {
        const num = parseInt(e.key, 10);
        const targetIdx = num - 1;
        if (targetIdx < enabledTabs.length) {
          e.preventDefault();
          audioEngine.playClick();
          setActiveTabKey(enabledTabs[targetIdx].key);
          if (!expanded) {
            setExpanded(true);
          }
        }
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
    currentTrack.durationSec,
    enabledTabs,
    expanded,
    handleSeek,
    positionSec,
    settingsOpen,
  ]);

  return (
    <main className="relative w-screen h-screen overflow-hidden">
      <DesktopCanvas
        wallpaper={wallpaper}
        mountMode={mountMode}
        expandTrigger={expandTrigger}
        visualState={visualState}
        expanded={expanded}
        simulatedFullscreen={simulatedFullscreen}
        deepWorkActive={isDeepWorkActive}
        moodOverride={moodOverride}
        charging={charging}
        batteryPct={batteryPct}
        micMuted={micMuted}
        cliAgent={cliAgent}
        onChangeWallpaper={setWallpaper}
        onChangeMountMode={setMountMode}
        onChangeExpandTrigger={setExpandTrigger}
        onTriggerStatePreset={handleTriggerStatePreset}
        onToggleSimulatedFullscreen={() => setSimulatedFullscreen((v) => !v)}
        onOpenSettings={() => setSettingsOpen(true)}
        onSetMoodOverride={setMoodOverride}
        onToggleLowBattery={handleToggleLowBattery}
        onToggleCharging={handleToggleCharging}
        onTriggerBatteryFull={handleTriggerBatteryFull}
        onPetShiba={() => {
          setMoodOverride(null);
          triggerHappyBurst(1800);
        }}
        onToggleDeepWork={() =>
          updateShibaSettings({ deepWorkEnabled: !shibaSettings.deepWorkEnabled })
        }
        onTriggerVolumeHud={handleTriggerVolumeHud}
        onTriggerBrightnessHud={handleTriggerBrightnessHud}
        onTriggerCapsLock={handleTriggerCapsLock}
        onToggleMicMute={handleToggleMicMute}
        onTriggerDownloadRadar={handleTriggerDownloadRadar}
        onTriggerCalendarRadar={handleTriggerCalendarRadar}
        onTriggerHydrationNudge={() => {
          setExpanded(false);
          triggerHydrationNudge();
        }}
        onTriggerEyeRestNudge={() => {
          setExpanded(false);
          triggerEyeRestNudge();
        }}
        onSetCliAgentStatus={handleSetCliAgentStatus}
        onTriggerPeripheral={handleTriggerPeripheral}
        onTriggerScreenshot={captureScreenshot}
        sensorAccess={sensorAccess}
        autoHideOnScreenshot={shibaSettings.autoHideOnScreenshot}
        showDevSimulator={showDevSimulator}
        onTriggerSensorAccess={handleTriggerSensorAccess}
        onClearSensorAccess={handleClearSensorAccess}
        onToggleAutoHideScreenshot={() =>
          updateShibaSettings({ autoHideOnScreenshot: !shibaSettings.autoHideOnScreenshot })
        }
      />

      <DynamicIsland
        expanded={expanded}
        activeTabKey={activeTabKey}
        enabledTabs={enabledTabs}
        mountMode={mountMode}
        expandTrigger={expandTrigger}
        wallpaper={wallpaper}
        visualState={visualState}
        shibaState={shibaState}
        shibaSettings={shibaSettings}
        isHiddenFullscreen={isHiddenFullscreen}
        screenshotHiding={screenshotHiding}
        isDeepWorkActive={isDeepWorkActive}
        onPetShiba={() => triggerHappyBurst(1600)}
        onToggleDnd={() => updateShibaSettings({ dnd: !shibaSettings.dnd })}
        onOpenSettings={() => setSettingsOpen(true)}
        hudOverlay={hudOverlay}
        systemVolume={systemVolume}
        micActive={micActive}
        micMuted={micMuted}
        batteryPct={batteryPct}
        charging={charging}
        batteryEffect={batteryEffect}
        powerPlan={powerPlan}
        downloadRadar={downloadRadar}
        calendarRadar={calendarRadar}
        sensorAccess={sensorAccess}
        onWheelVolume={handleWheelVolume}
        onToggleMicMute={handleToggleMicMute}
        onCyclePowerPlan={handleCyclePowerPlan}
        onOpenDownloadedFile={handleOpenDownloadedFile}
        onJoinCalendarMeet={handleJoinCalendarMeet}
        cliAgent={cliAgent}
        onApproveAgent={handleApproveAgent}
        onRejectAgent={handleRejectAgent}
        onFocusTerminal={handleFocusTerminal}
        track={currentTrack}
        playing={playing}
        positionSec={positionSec}
        volume={volume}
        lyrics={lyrics}
        showLyrics={showLyrics}
        onToggleShowLyrics={() => setShowLyrics((v) => !v)}
        onTogglePlay={() => {
          audioEngine.playClick();
          setPlaying((p) => !p);
        }}
        onPrevTrack={() => handleSelectTrack(trackIndex - 1)}
        onNextTrack={() => handleSelectTrack(trackIndex + 1)}
        onSeek={handleSeek}
        onVolumeChange={(vol) => {
          setVolume(vol);
          setSystemVolume(Math.round(vol * 100));
        }}
        focus={focus}
        focusCelebrating={focusCelebrating}
        onToggleFocus={() => {
          audioEngine.playClick();
          setFocus((prev) => ({ ...prev, running: !prev.running }));
        }}
        onResetFocus={() => {
          audioEngine.playClick();
          setFocus((prev) => {
            const total = FOCUS_PHASE_CONFIG[prev.phase].durationSec;
            return { ...prev, remainingSec: total, totalSec: total, running: false };
          });
        }}
        onSelectFocusPhase={(phase) => {
          audioEngine.playClick();
          const total = FOCUS_PHASE_CONFIG[phase].durationSec;
          setFocus((prev) => ({
            ...prev,
            phase,
            remainingSec: total,
            totalSec: total,
            running: false,
          }));
        }}
        onCompleteFocusNow={completeFocusPhase}
        water={water}
        onAddWater={handleAddWater}
        onUndoWater={handleUndoWater}
        onResetWater={handleResetWater}
        tasks={tasks}
        onToggleTask={handleToggleTask}
        onAddTask={handleAddTask}
        onDeleteTask={handleDeleteTask}
        onClearTasksCompleted={handleClearTasksCompleted}
        shelfFiles={shelfFiles}
        scrcpy={scrcpy}
        screenshots={screenshots}
        onAddShelfFiles={handleAddShelfFiles}
        onRemoveShelfFile={(id) => {
          audioEngine.playClick();
          setShelfFiles((prev) => prev.filter((f) => f.id !== id));
        }}
        onClearShelfFiles={() => {
          audioEngine.playClick();
          setShelfFiles([]);
          setScreenshots([]);
        }}
        onToggleMirror={() =>
          setScrcpy((prev) => {
            const next = !prev.mirroring;
            showScrcpyToast(next ? 'Scrcpy Live' : 'Mirror Off');
            return { ...prev, mirroring: next };
          })
        }
        onToggleRecord={() =>
          setScrcpy((prev) => {
            const next = !prev.recording;
            if (!next) showScrcpyToast('Saved MP4');
            return { ...prev, recording: next, recordingSec: next ? 0 : prev.recordingSec };
          })
        }
        onToggleWirelessAdb={() =>
          setScrcpy((prev) => {
            const next = !prev.wirelessAdb;
            showScrcpyToast(next ? prev.adbAddress : 'USB Mode');
            return { ...prev, wirelessAdb: next };
          })
        }
        onCopyClipboard={() => {
          try {
            void navigator.clipboard?.writeText('adb connect 192.168.1.42:5555');
          } catch {
            // Ignore
          }
          showScrcpyToast('Copied!');
        }}
        onOpenScreenshot={handleOpenScreenshot}
        onCopyScreenshotPath={handleCopyScreenshotPath}
        clipboardItems={clipboardItems}
        onToggleClipboardPin={handleToggleClipboardPin}
        onCopyClipboardItem={handleCopyClipboardItem}
        zenActive={zenActive}
        onToggleZenActive={setZenActive}
        onExpand={handleExpand}
        onCollapse={handleCollapse}
        onSelectTab={setActiveTabKey}
      />

      <div data-isle-modal="true">
        <SettingsModal
          open={settingsOpen}
          settings={shibaSettings}
          tabConfig={tabConfig}
          onClose={() => setSettingsOpen(false)}
          onUpdateSettings={updateShibaSettings}
          onReorderTabs={setTabConfig}
          onToggleTabEnabled={handleToggleTabEnabled}
        />
      </div>
    </main>
  );
}

export default App;
