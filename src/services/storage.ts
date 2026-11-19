import {
  DEFAULT_CLIPBOARD_ITEMS,
  DEFAULT_SHIBA_SETTINGS,
  DEFAULT_TAB_CONFIG,
  DEFAULT_TASKS,
  STORAGE_KEYS,
} from '../constants';
import type {
  ClipboardItem,
  ShelfFile,
  ShibaSettings,
  TabConfigItem,
  TaskItem,
  WaterState,
} from '../types';

export function getTodayKey(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function getYesterdayKey(): string {
  const d = new Date();
  d.setDate(d.getDate() - 1);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

// ---------------------------------------------------------------------------
// Debounced LocalStorage Write Engine (Zero-Jitter & Battery Optimization)
// ---------------------------------------------------------------------------
const writeDebounceTimers = new Map<string, number>();

function debouncedSetItem(key: string, value: string, delayMs = 250): void {
  const existing = writeDebounceTimers.get(key);
  if (existing !== undefined) {
    window.clearTimeout(existing);
  }
  writeDebounceTimers.set(
    key,
    window.setTimeout(() => {
      try {
        localStorage.setItem(key, value);
      } catch {
        // Ignore quota/storage errors
      }
      writeDebounceTimers.delete(key);
    }, delayMs)
  );
}

// ---------------------------------------------------------------------------
// Hydration Store with Midnight Rollover & Streak Tracking
// ---------------------------------------------------------------------------

export const HydrationStorage = {
  load(): WaterState {
    const today = getTodayKey();
    const yesterday = getYesterdayKey();

    try {
      const raw = localStorage.getItem(STORAGE_KEYS.water);
      if (raw) {
        const parsed = JSON.parse(raw) as Partial<WaterState> & { lastArchivedDate?: string };

        // Check if date changed (midnight rollover)
        if (parsed.date === today && typeof parsed.intakeMl === 'number') {
          return {
            date: today,
            intakeMl: Math.max(0, parsed.intakeMl),
            goalMl: typeof parsed.goalMl === 'number' && parsed.goalMl >= 250 ? parsed.goalMl : 2000,
            history: Array.isArray(parsed.history) ? parsed.history : [],
            streakDays: typeof parsed.streakDays === 'number' ? parsed.streakDays : 1,
          };
        }

        // Rollover: If parsed.date was yesterday and goal was met, increment streak!
        let nextStreak = 0;
        const prevGoal = parsed.goalMl ?? 2000;
        const prevIntake = parsed.intakeMl ?? 0;
        const metYesterday = parsed.date === yesterday && prevIntake >= prevGoal;

        if (metYesterday) {
          nextStreak = (typeof parsed.streakDays === 'number' ? parsed.streakDays : 0) + 1;
        } else if (parsed.date === yesterday) {
          nextStreak = 0; // reset streak if didn't meet goal yesterday
        }

        // Archive yesterday's data in history log
        try {
          const archiveKey = `isle_water_archive_${parsed.date}`;
          localStorage.setItem(archiveKey, JSON.stringify({
            date: parsed.date,
            intakeMl: prevIntake,
            goalMl: prevGoal,
          }));
        } catch {
          // Ignore
        }

        const freshState: WaterState = {
          date: today,
          intakeMl: 0,
          goalMl: prevGoal,
          history: [],
          streakDays: Math.max(nextStreak, 0),
        };
        this.save(freshState);
        return freshState;
      }
    } catch {
      // Ignore parse errors
    }

    // Default initial seed
    const defaultState: WaterState = {
      date: today,
      intakeMl: 1250,
      goalMl: 2000,
      history: [250, 500, 500],
      streakDays: 3,
    };
    this.save(defaultState);
    return defaultState;
  },

  save(state: WaterState): void {
    debouncedSetItem(STORAGE_KEYS.water, JSON.stringify(state));
  },
};

// ---------------------------------------------------------------------------
// Tasks Store with Priority Categories, Timestamps & Clear Completed
// ---------------------------------------------------------------------------

export const TasksStorage = {
  load(): TaskItem[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.tasks);
      if (raw) {
        const parsed = JSON.parse(raw) as TaskItem[];
        if (Array.isArray(parsed)) return parsed;
      }
    } catch {
      // Ignore
    }
    return DEFAULT_TASKS;
  },

  save(tasks: TaskItem[]): void {
    debouncedSetItem(STORAGE_KEYS.tasks, JSON.stringify(tasks));
  },

  clearCompleted(tasks: TaskItem[]): TaskItem[] {
    const remaining = tasks.filter((t) => !t.completed);
    this.save(remaining);
    return remaining;
  },
};

// ---------------------------------------------------------------------------
// Shelf Store: Staging Area for Drag & Drop Files
// ---------------------------------------------------------------------------

export const ShelfStorage = {
  load(): ShelfFile[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.shelf);
      if (raw) {
        const parsed = JSON.parse(raw) as ShelfFile[];
        if (Array.isArray(parsed)) return parsed;
      }
    } catch {
      // Ignore
    }
    return [];
  },

  save(files: ShelfFile[]): void {
    const safeFiles = files.slice(0, 12).map((f) => ({
      ...f,
      dataUrl: f.dataUrl && f.dataUrl.length > 500_000 ? undefined : f.dataUrl,
    }));
    debouncedSetItem(STORAGE_KEYS.shelf, JSON.stringify(safeFiles));
  },
};

// ---------------------------------------------------------------------------
// Clipboard Store
// ---------------------------------------------------------------------------

export const ClipboardStorage = {
  load(): ClipboardItem[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.clipboard);
      if (raw) {
        const parsed = JSON.parse(raw) as ClipboardItem[];
        if (Array.isArray(parsed) && parsed.length > 0) return parsed;
      }
    } catch {
      // Ignore
    }
    return DEFAULT_CLIPBOARD_ITEMS;
  },

  save(items: ClipboardItem[]): void {
    debouncedSetItem(STORAGE_KEYS.clipboard, JSON.stringify(items.slice(0, 30)));
  },
};

// ---------------------------------------------------------------------------
// Settings & Tabs Store
// ---------------------------------------------------------------------------

export const SettingsStorage = {
  loadSettings(): ShibaSettings {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.settings);
      if (raw) {
        return { ...DEFAULT_SHIBA_SETTINGS, ...(JSON.parse(raw) as Partial<ShibaSettings>) };
      }
    } catch {
      // Ignore
    }
    return DEFAULT_SHIBA_SETTINGS;
  },

  saveSettings(settings: ShibaSettings): void {
    debouncedSetItem(STORAGE_KEYS.settings, JSON.stringify(settings));
  },

  loadTabConfig(): TabConfigItem[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS.tabs);
      if (raw) {
        const parsed = JSON.parse(raw) as TabConfigItem[];
        if (Array.isArray(parsed) && parsed.length === DEFAULT_TAB_CONFIG.length) {
          return parsed;
        }
      }
    } catch {
      // Ignore
    }
    return DEFAULT_TAB_CONFIG;
  },

  saveTabConfig(config: TabConfigItem[]): void {
    debouncedSetItem(STORAGE_KEYS.tabs, JSON.stringify(config));
  },
};
