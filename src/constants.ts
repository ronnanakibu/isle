import type {
  FocusPhase,
  PowerPlan,
  ShibaSettings,
  TabConfigItem,
  TabKey,
  TaskItem,
  TaskPriorityColor,
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
  expandTrigger: 'isle_expand_trigger',
} as const;
