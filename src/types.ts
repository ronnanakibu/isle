export type WallpaperId = 'transparent' | 'sonoma' | 'win11' | 'oled';

export type ThemeMode = 'system' | 'dark' | 'light' | 'liquid';

export type MountMode = 'notch' | 'floating';

export type ExpandTrigger = 'click' | 'hover';

export type IslandVisualState = 'idle' | 'media' | 'split' | 'expanded';

// 7 Consolidated Tabs (Consolidated Suite)
export type TabKey =
  | 'media'
  | 'focus'
  | 'water'
  | 'tasks'
  | 'shelf'
  | 'monitor'
  | 'zen';

export type ShibaState =
  | 'normal'
  | 'lowBattery'
  | 'charging'
  | 'music'
  | 'happy'
  | 'zen'
  | 'thinking'
  | 'concerned';

export type MascotMood = 'watching' | 'resting' | 'sleepy' | 'happy' | 'grooving';

export type PresenceLevel = 'playful' | 'polite' | 'stealth';

export type PowerPlan = 'eco' | 'balanced' | 'performance';

// CLI Coding Agent Live Monitor State
export type CliAgentStatus = 'idle' | 'thinking' | 'approval' | 'complete' | 'error';

export interface CliAgentActivity {
  status: CliAgentStatus;
  agentName: string; // e.g. "Claude Code"
  headline: string; // e.g. "Editing window.rs..." or "Run 'cargo test'?"
  detail?: string; // e.g. "8 files modified" or "Exit 1"
  question?: string; // e.g. "Run 'cargo test'?"
  command?: string; // e.g. "cargo test"
  timestamp: number;
}

// Peripheral & Wireless Devices
export type PeripheralType = 'headphones' | 'smartphone' | 'printer';

export interface PeripheralDevice {
  type: PeripheralType;
  name: string;
  detail: string;
  connected: boolean;
  batteryPct?: number;
  batteryDetail?: { left: number; right: number };
}

// Screenshot JSON Metadata Record (Zero Physical Duplication)
export interface ScreenshotRecord {
  id: string;
  type: 'screenshot';
  filePath: string;
  fileName: string;
  timestamp: number;
  previewGradient?: [string, string];
}

// Sensor Privacy Types (Android 15 / iOS Dynamic Sensor Access)
export type SensorType = 'mic' | 'camera' | 'location';

export interface SensorAccessState {
  active: boolean;
  type: SensorType;
  app: string;
  startTime: number;
}

// Nusa Battery Visual Action Effects
export type BatteryActionEffect =
  | 'chargingPulse'
  | 'unplugged'
  | 'fullGlow'
  | 'lowBatteryToast'
  | 'chargingSweep'
  | null;

export type HudOverlay =
  | { kind: 'volume'; value: number; muted: boolean; overscroll?: 'min' | 'max' | null }
  | { kind: 'brightness'; value: number; overscroll?: 'min' | 'max' | null }
  | { kind: 'capsLock'; enabled: boolean; isNumLock?: boolean }
  | { kind: 'mic'; muted: boolean }
  | { kind: 'powerPlan'; plan: PowerPlan }
  | { kind: 'nudge'; nudgeType: 'hydration' | 'eyeRest'; message: string; remainingSec?: number }
  | { kind: 'peripheral'; device: PeripheralDevice }
  | { kind: 'screenshot'; record: ScreenshotRecord }
  | { kind: 'batteryNotice'; effect: BatteryActionEffect; message: string; pct?: number }
  | { kind: 'sensorToast'; sensor: SensorType; app: string };

export interface DownloadRadarState {
  active: boolean;
  filename: string;
  progress: number; // 0..100
  speedMbps: number;
  etaSec: number;
  completed: boolean;
}

export interface CalendarRadarState {
  active: boolean;
  title: string;
  startsInMin: number;
  meetUrl: string;
  joined: boolean;
}

export interface LyricLine {
  timeMs: number;
  text: string;
}

export interface LyricsState {
  lines: LyricLine[];
  source: 'lrclib' | 'fallback' | 'loading';
  synced: boolean;
  trackKey: string;
}

export interface Track {
  id: string;
  title: string;
  artist: string;
  album: string;
  durationSec: number;
  initialPositionSec: number;
  bpm: number;
  accentColor: string;
  coverGradient: [string, string, string];
  synthChord: number[];
  fallbackLrc: string;
}

export type FocusPhase = 'work' | 'shortBreak' | 'longBreak';

export interface FocusState {
  phase: FocusPhase;
  remainingSec: number;
  totalSec: number;
  running: boolean;
  completedSessions: number;
}

export interface WaterState {
  date: string;
  intakeMl: number;
  goalMl: number;
  history: number[];
  streakDays: number;
}

export type TaskPriorityColor = 'cyan' | 'emerald' | 'amber' | 'violet' | 'rose';

export interface TaskItem {
  id: string;
  title: string;
  completed: boolean;
  color: TaskPriorityColor;
  createdAt: number;
}

export interface ShelfFile {
  id: string;
  name: string;
  sizeBytes: number;
  kind: 'image' | 'doc' | 'code' | 'archive' | 'other';
  addedAt: number;
  dataUrl?: string;
  filePath?: string;
  isScreenshot?: boolean;
}

export interface ScrcpyState {
  deviceName: string;
  batteryPct: number;
  connected: boolean;
  mirroring: boolean;
  recording: boolean;
  recordingSec: number;
  wirelessAdb: boolean;
  adbAddress: string;
  toast: string | null;
}

export type ClipboardBadge = 'CODE' | 'URL' | 'HEX' | 'TEXT';

export interface ClipboardItem {
  id: string;
  content: string;
  badge: ClipboardBadge;
  pinned: boolean;
  copiedAt: string;
}

export interface TabConfigItem {
  key: TabKey;
  label: string;
  accent: string;
  enabled: boolean;
}

export interface ShibaSettings {
  theme: ThemeMode;
  presenceLevel: PresenceLevel;
  deepWorkEnabled: boolean;
  deepWorkStart: string; // HH:mm
  deepWorkEnd: string; // HH:mm
  linkDeepWorkToPomodoro: boolean;
  fullscreenAutoHide: boolean;
  dnd: boolean;
  liveLyricTicker: boolean;
  autoHideOnScreenshot: boolean;
}
