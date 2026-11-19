import type {
  ClipboardItem,
  FocusPhase,
  PowerPlan,
  ShibaSettings,
  TabConfigItem,
  TabKey,
  TaskItem,
  TaskPriorityColor,
  Track,
} from './types';

// ---------------------------------------------------------------------------
// Apple HIG Spring Physics (60 / 120 FPS Zero Jitter)
// ---------------------------------------------------------------------------

export const ISLAND_SPRING = {
  type: 'spring' as const,
  stiffness: 380,
  damping: 30,
  mass: 0.8,
};

export const PAGER_SPRING = {
  type: 'spring' as const,
  stiffness: 360,
  damping: 32,
  mass: 0.85,
};

export const MICRO_SPRING = {
  type: 'spring' as const,
  stiffness: 480,
  damping: 28,
  mass: 0.6,
};

// ---------------------------------------------------------------------------
// Island Geometry Metrics
// ---------------------------------------------------------------------------

export const ISLAND_METRICS = {
  idle: { width: 164, height: 36, radius: 18 },
  stealthIdle: { width: 112, height: 36, radius: 18 },
  media: { width: 232, height: 36, radius: 18 },
  hud: { width: 256, height: 36, radius: 18 },
  batterySweep: { width: 290, height: 36, radius: 18 },
  capsLockToast: { width: 184, height: 36, radius: 18 },
  sensorToast: { width: 236, height: 36, radius: 18 },
  radarDownload: { width: 336, height: 36, radius: 18 },
  radarCalendar: { width: 328, height: 36, radius: 18 },
  agentThinking: { width: 348, height: 36, radius: 18 },
  agentApproval: { width: 458, height: 40, radius: 20 },
  agentToast: { width: 364, height: 36, radius: 18 },
  peripheralToast: { width: 318, height: 36, radius: 18 },
  splitMainMedia: { width: 180, height: 36, radius: 18 },
  splitMainIdle: { width: 152, height: 36, radius: 18 },
  splitBadge: { width: 36, height: 36, radius: 18 },
  splitGap: 8,
  expanded: { width: 468, height: 242, radius: 28 },
} as const;

// ---------------------------------------------------------------------------
// Theme Tokens
// ---------------------------------------------------------------------------

export const THEME_TOKENS = {
  accent: '#5EC8FF',
  success: '#4ADE80',
  warning: '#FBBF24',
  danger: '#F87171',
  violet: '#C084FC',
  textPrimary: '#F5F7FA',
  textSecondary: '#9AA2AE',
  textTertiary: '#606874',
} as const;

export const TASK_COLOR_MAP: Record<TaskPriorityColor, { dot: string; ring: string; label: string }> = {
  cyan: { dot: '#5EC8FF', ring: 'rgba(94, 200, 255, 0.35)', label: 'Work' },
  emerald: { dot: '#4ADE80', ring: 'rgba(74, 222, 128, 0.35)', label: 'Health' },
  amber: { dot: '#FBBF24', ring: 'rgba(251, 191, 36, 0.35)', label: 'Focus' },
  violet: { dot: '#C084FC', ring: 'rgba(192, 132, 252, 0.35)', label: 'Ship' },
  rose: { dot: '#FB7185', ring: 'rgba(251, 113, 133, 0.35)', label: 'Urgent' },
};

// ---------------------------------------------------------------------------
// Consolidated 7-Tab Suite Default Order
// ---------------------------------------------------------------------------

export const DEFAULT_TAB_CONFIG: TabConfigItem[] = [
  { key: 'media', label: 'Media', accent: '#5EC8FF', enabled: true },
  { key: 'focus', label: 'Focus', accent: '#FBBF24', enabled: true },
  { key: 'water', label: 'Hydration', accent: '#38BDF8', enabled: true },
  { key: 'tasks', label: 'Tasks', accent: '#4ADE80', enabled: true },
  { key: 'shelf', label: 'Shelf & Clipboard', accent: '#C084FC', enabled: true },
  { key: 'monitor', label: 'Hardware', accent: '#34D399', enabled: true },
  { key: 'zen', label: 'Zen Breathing', accent: '#2DD4BF', enabled: true },
];

export const TAB_META_MAP: Record<TabKey, { label: string; accent: string }> = {
  media: { label: 'Media', accent: '#5EC8FF' },
  focus: { label: 'Focus', accent: '#FBBF24' },
  water: { label: 'Hydration', accent: '#38BDF8' },
  tasks: { label: 'Tasks', accent: '#4ADE80' },
  shelf: { label: 'Shelf & Clipboard', accent: '#C084FC' },
  monitor: { label: 'Hardware', accent: '#34D399' },
  zen: { label: 'Zen Breathing', accent: '#2DD4BF' },
};

// ---------------------------------------------------------------------------
// Shiba Default Settings & Power Plans
// ---------------------------------------------------------------------------

export const DEFAULT_SHIBA_SETTINGS: ShibaSettings = {
  theme: 'dark',
  presenceLevel: 'polite',
  deepWorkEnabled: false,
  deepWorkStart: '09:00',
  deepWorkEnd: '17:00',
  linkDeepWorkToPomodoro: true,
  fullscreenAutoHide: true,
  dnd: false,
  liveLyricTicker: true,
  autoHideOnScreenshot: true,
};

export const POWER_PLAN_META: Record<
  PowerPlan,
  { label: string; shortLabel: string; color: string }
> = {
  eco: { label: 'Eco / Battery Saver', shortLabel: 'Eco', color: '#4ADE80' },
  balanced: { label: 'Balanced', shortLabel: 'Balanced', color: '#5EC8FF' },
  performance: { label: 'High Performance', shortLabel: 'Perf', color: '#FBBF24' },
};

// ---------------------------------------------------------------------------
// Focus / Pomodoro Config
// ---------------------------------------------------------------------------

export const FOCUS_PHASE_CONFIG: Record<
  FocusPhase,
  { label: string; shortLabel: string; durationSec: number; color: string }
> = {
  work: {
    label: 'Focus Session',
    shortLabel: 'Focus',
    durationSec: 25 * 60,
    color: '#5EC8FF',
  },
  shortBreak: {
    label: 'Short Break',
    shortLabel: 'Short',
    durationSec: 5 * 60,
    color: '#4ADE80',
  },
  longBreak: {
    label: 'Long Break',
    shortLabel: 'Long',
    durationSec: 15 * 60,
    color: '#FBBF24',
  },
};

// ---------------------------------------------------------------------------
// Clipboard Seed Items
// ---------------------------------------------------------------------------

export const DEFAULT_CLIPBOARD_ITEMS: ClipboardItem[] = [
  {
    id: 'clip-1',
    content: 'const spring = { type: "spring", stiffness: 380, damping: 30 };',
    badge: 'CODE',
    pinned: true,
    copiedAt: '2m ago',
  },
  {
    id: 'clip-2',
    content: 'https://lrclib.net/api/get?track_name=Midnight+City',
    badge: 'URL',
    pinned: false,
    copiedAt: '6m ago',
  },
  {
    id: 'clip-3',
    content: '#5EC8FF',
    badge: 'HEX',
    pinned: false,
    copiedAt: '11m ago',
  },
  {
    id: 'clip-4',
    content: 'adb connect 192.168.1.42:5555',
    badge: 'CODE',
    pinned: false,
    copiedAt: '18m ago',
  },
  {
    id: 'clip-5',
    content: 'Sprint Review — Claude Code HUD & Nusa Parity demo at 15:00',
    badge: 'TEXT',
    pinned: false,
    copiedAt: '24m ago',
  },
  {
    id: 'clip-6',
    content: '#E89B48',
    badge: 'HEX',
    pinned: false,
    copiedAt: '31m ago',
  },
  {
    id: 'clip-7',
    content: 'cargo test --lib -- --nocapture',
    badge: 'CODE',
    pinned: false,
    copiedAt: '45m ago',
  },
  {
    id: 'clip-8',
    content: 'backdrop-filter: blur(32px) saturate(180%);',
    badge: 'CODE',
    pinned: false,
    copiedAt: '1h ago',
  },
];

// ---------------------------------------------------------------------------
// Curated Tracks (Live LRCLIB + Offline Fallback Cues)
// ---------------------------------------------------------------------------

export const PRESET_TRACKS: Track[] = [
  {
    id: 'midnight-city',
    title: 'Midnight City',
    artist: 'M83',
    album: "Hurry Up, We're Dreaming",
    durationSec: 243,
    initialPositionSec: 38,
    bpm: 105,
    accentColor: '#A855F7',
    coverGradient: ['#311042', '#7E22CE', '#EC4899'],
    synthChord: [261.63, 329.63, 392.0, 493.88],
    fallbackLrc: `[00:00.00] ♪ Synthwave Intro — M83
[00:18.00] ♪ Analog arpeggio rising
[00:37.68] Waiting in a car
[00:42.37] Waiting for a ride in the dark
[00:46.88] The night city grows
[00:51.33] Look at the horizon glow
[00:56.78] ♪ Instrumental shimmer
[01:14.23] Waiting in a car
[01:18.80] Waiting for a ride in the dark
[01:23.40] Drinking in the lounge
[01:28.10] Following the neon signs`,
  },
  {
    id: 'blinding-lights',
    title: 'Blinding Lights',
    artist: 'The Weeknd',
    album: 'After Hours',
    durationSec: 200,
    initialPositionSec: 28,
    bpm: 171,
    accentColor: '#F43F5E',
    coverGradient: ['#450A0A', '#DC2626', '#F59E0B'],
    synthChord: [293.66, 349.23, 440.0, 523.25],
    fallbackLrc: `[00:00.00] ♪ Retro Synth Pulse — The Weeknd
[00:14.00] ♪ Drum machine kicks in
[00:27.50] I've been tryna call
[00:30.20] I've been on my own for long enough
[00:33.10] Maybe you can show me how to love, maybe
[00:38.80] I'm going through withdrawals
[00:41.50] You don't even have to do too much`,
  },
  {
    id: 'get-lucky',
    title: 'Get Lucky',
    artist: 'Daft Punk',
    album: 'Random Access Memories',
    durationSec: 248,
    initialPositionSec: 32,
    bpm: 116,
    accentColor: '#FBBF24',
    coverGradient: ['#1E1B4B', '#B45309', '#FBBF24'],
    synthChord: [246.94, 293.66, 369.99, 440.0],
    fallbackLrc: `[00:00.00] ♪ Funk Guitar Groove — Daft Punk
[00:16.00] ♪ Vocoder shimmer
[00:31.50] Like the legend of the phoenix
[00:35.80] All ends with beginnings
[00:39.90] What keeps the planet spinning
[00:44.10] The force from the beginning`,
  },
  {
    id: 'as-it-was',
    title: 'As It Was',
    artist: 'Harry Styles',
    album: "Harry's House",
    durationSec: 167,
    initialPositionSec: 16,
    bpm: 174,
    accentColor: '#38BDF8',
    coverGradient: ['#0C4A6E', '#0284C7', '#FB7185'],
    synthChord: [220.0, 277.18, 329.63, 415.3],
    fallbackLrc: `[00:00.00] ♪ Bright Synth Riff — Harry Styles
[00:15.20] Holdin' me back
[00:18.40] Gravity's holdin' me back
[00:21.60] I want you to hold out the palm of your hand
[00:25.10] Why don't we leave it at that?`,
  },
];

// ---------------------------------------------------------------------------
// Default Tasks & Storage Keys
// ---------------------------------------------------------------------------

export const DEFAULT_TASKS: TaskItem[] = [
  {
    id: 'task-1',
    title: 'Refactor Isle Notch & eliminate AI slop',
    completed: true,
    color: 'violet',
    createdAt: 1,
  },
  {
    id: 'task-2',
    title: 'Integrate Claude Code Live Monitor HUD',
    completed: true,
    color: 'cyan',
    createdAt: 2,
  },
  {
    id: 'task-3',
    title: 'Isolate lyrics wheel scroll trap',
    completed: false,
    color: 'emerald',
    createdAt: 3,
  },
  {
    id: 'task-4',
    title: 'Verify ThinkPad F4 mic mute tape cross',
    completed: false,
    color: 'amber',
    createdAt: 4,
  },
];

export const STORAGE_KEYS = {
  water: 'isle_water_v2',
  tasks: 'isle_tasks_v2',
  shelf: 'isle_shelf_v2',
  clipboard: 'isle_clipboard_v2',
  settings: 'isle_shiba_settings_v2',
  tabs: 'isle_tab_config_v2',
} as const;
