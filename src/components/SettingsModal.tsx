import React from 'react';
import { motion, AnimatePresence, Reorder } from 'framer-motion';
import {
  AlertTriangle,
  BellOff,
  Camera,
  GripVertical,
  Laptop,
  MonitorOff,
  Moon,
  Sparkles,
  Sun,
  X,
} from 'lucide-react';
import type { PresenceLevel, ShibaSettings, TabConfigItem, ThemeMode } from '../types';

interface SettingsModalProps {
  open: boolean;
  settings: ShibaSettings;
  tabConfig: TabConfigItem[];
  onClose: () => void;
  onUpdateSettings: (patch: Partial<ShibaSettings>) => void;
  onReorderTabs: (next: TabConfigItem[]) => void;
  onToggleTabEnabled: (key: TabConfigItem['key']) => void;
}

const THEME_OPTIONS: Array<{
  id: ThemeMode;
  label: string;
  desc: string;
  icon: typeof Moon;
}> = [
  { id: 'system', label: 'System Auto', desc: 'Sync with OS', icon: Laptop },
  { id: 'dark', label: 'OLED Dark', desc: 'Deep obsidian glass', icon: Moon },
  { id: 'light', label: 'Pearl Light', desc: 'Frosted white glass', icon: Sun },
  { id: 'liquid', label: 'Liquid Glass', desc: 'Mica real-time blur', icon: Sparkles },
];

const PRESENCE_OPTIONS: Array<{
  id: PresenceLevel;
  label: string;
  desc: string;
}> = [
  { id: 'playful', label: 'Playful', desc: 'Full ear wiggles & reactions' },
  { id: 'polite', label: 'Polite', desc: 'Reacts only on hover or click' },
  { id: 'stealth', label: 'Stealth', desc: 'Minimalist clock only' },
];

export const SettingsModal: React.FC<SettingsModalProps> = ({
  open,
  settings,
  tabConfig,
  onClose,
  onUpdateSettings,
  onReorderTabs,
  onToggleTabEnabled,
}) => {
  const enabledCount = tabConfig.filter((t) => t.enabled).length;

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4"
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.94, y: 12 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.94, y: 12 }}
            transition={{ type: 'spring', stiffness: 380, damping: 30 }}
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-[460px] max-h-[84vh] rounded-[28px] isle-glass border border-white/15 overflow-hidden flex flex-col text-white shadow-2xl"
          >
            {/* Header */}
            <div className="px-5 py-3.5 border-b border-white/10 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-amber-300" />
                <h2 className="text-[14px] font-semibold tracking-tight">
                  Isle & Shiba Settings
                </h2>
              </div>
              <button
                type="button"
                onClick={onClose}
                className="w-7 h-7 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center text-white/70 hover:text-white cursor-pointer transition-colors"
                aria-label="Close settings"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Scrollable Body */}
            <div className="p-5 overflow-y-auto isle-scroll space-y-5">
              {/* Instant DND Toggle */}
              <div className="flex items-center justify-between p-3 rounded-2xl bg-white/[0.05] border border-white/[0.08]">
                <div className="flex items-center gap-2.5">
                  <div className="w-8 h-8 rounded-xl bg-indigo-500/20 border border-indigo-400/30 flex items-center justify-center text-indigo-300">
                    <BellOff className="w-4 h-4" />
                  </div>
                  <div>
                    <p className="text-[12.5px] font-semibold">Do Not Disturb</p>
                    <p className="text-[10.5px] text-white/50">
                      Silence all Shiba animations & nudges instantly
                    </p>
                  </div>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={settings.dnd}
                  onClick={() => onUpdateSettings({ dnd: !settings.dnd })}
                  className={`w-11 h-6 rounded-full p-0.5 transition-colors cursor-pointer ${
                    settings.dnd ? 'bg-indigo-400' : 'bg-white/15'
                  }`}
                >
                  <motion.div
                    className="w-5 h-5 rounded-full bg-white shadow"
                    animate={{ x: settings.dnd ? 20 : 0 }}
                    transition={{ type: 'spring', stiffness: 500, damping: 30 }}
                  />
                </button>
              </div>

              {/* 4-Theme Architecture System */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <p className="text-[11px] font-semibold uppercase tracking-wider text-white/50">
                    Theme Architecture
                  </p>
                  <span className="text-[10px] text-white/40">4 Styles</span>
                </div>
                <div className="grid grid-cols-2 gap-2">
                  {THEME_OPTIONS.map((opt) => {
                    const active = (settings.theme ?? 'dark') === opt.id;
                    const Icon = opt.icon;
                    return (
                      <button
                        key={opt.id}
                        type="button"
                        onClick={() => onUpdateSettings({ theme: opt.id })}
                        className={`p-2.5 rounded-2xl border text-left transition-all cursor-pointer ${
                          active
                            ? 'bg-sky-400/20 border-sky-400/50 text-white shadow-sm'
                            : 'bg-white/[0.04] hover:bg-white/[0.08] border-white/[0.08] text-white/70'
                        }`}
                      >
                        <div className="flex items-center gap-1.5 mb-1">
                          <Icon className={`w-3.5 h-3.5 ${active ? 'text-sky-300' : 'text-white/60'}`} />
                          <p className="text-[12px] font-semibold">{opt.label}</p>
                        </div>
                        <p className="text-[9.5px] text-white/50 leading-snug">
                          {opt.desc}
                        </p>
                      </button>
                    );
                  })}
                </div>
                {/* Performance Advisory Badge for Liquid Glass (Mica) */}
                {(settings.theme === 'liquid') && (
                  <motion.div
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="mt-2 p-2.5 rounded-xl bg-amber-500/10 border border-amber-400/25 flex items-start gap-2"
                  >
                    <AlertTriangle className="w-3.5 h-3.5 text-amber-300 shrink-0 mt-0.5" />
                    <p className="text-[10px] text-amber-200/90 leading-relaxed">
                      <strong>Performance Advisory:</strong> Liquid Glass may slightly increase CPU, GPU, and RAM consumption due to multi-pass real-time backdrop blur filtering.
                    </p>
                  </motion.div>
                )}
              </div>

              {/* Shiba Presence Level */}
              <div>
                <p className="text-[11px] font-semibold uppercase tracking-wider text-white/50 mb-2">
                  Shiba Presence Level
                </p>
                <div className="grid grid-cols-3 gap-2">
                  {PRESENCE_OPTIONS.map((opt) => {
                    const active = settings.presenceLevel === opt.id;
                    return (
                      <button
                        key={opt.id}
                        type="button"
                        onClick={() => onUpdateSettings({ presenceLevel: opt.id })}
                        className={`p-2.5 rounded-2xl border text-left transition-all cursor-pointer ${
                          active
                            ? 'bg-amber-400/18 border-amber-400/45 text-white'
                            : 'bg-white/[0.04] hover:bg-white/[0.08] border-white/[0.08] text-white/70'
                        }`}
                      >
                        <p className="text-[12px] font-semibold">{opt.label}</p>
                        <p className="text-[9.5px] text-white/50 mt-0.5 leading-snug">
                          {opt.desc}
                        </p>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Deep Work & Fullscreen Auto-Hide */}
              <div className="space-y-2.5">
                <p className="text-[11px] font-semibold uppercase tracking-wider text-white/50">
                  Attention & Etiquette
                </p>

                <div className="p-3 rounded-2xl bg-white/[0.04] border border-white/[0.08] space-y-2.5">
                  <label className="flex items-center justify-between cursor-pointer">
                    <span className="flex items-center gap-2 text-[12px] font-medium">
                      <Moon className="w-3.5 h-3.5 text-sky-300" />
                      Deep Work Zen Mode
                    </span>
                    <input
                      type="checkbox"
                      checked={settings.deepWorkEnabled}
                      onChange={(e) =>
                        onUpdateSettings({ deepWorkEnabled: e.target.checked })
                      }
                      className="accent-sky-400 w-4 h-4 cursor-pointer"
                    />
                  </label>

                  <div className="flex items-center gap-2 text-[11px] text-white/65 pl-5">
                    <span>Hours:</span>
                    <input
                      type="time"
                      value={settings.deepWorkStart}
                      onChange={(e) =>
                        onUpdateSettings({ deepWorkStart: e.target.value })
                      }
                      className="bg-white/10 border border-white/15 rounded-lg px-2 py-0.5 text-white text-[11px]"
                    />
                    <span>to</span>
                    <input
                      type="time"
                      value={settings.deepWorkEnd}
                      onChange={(e) =>
                        onUpdateSettings({ deepWorkEnd: e.target.value })
                      }
                      className="bg-white/10 border border-white/15 rounded-lg px-2 py-0.5 text-white text-[11px]"
                    />
                  </div>

                  <label className="flex items-center justify-between cursor-pointer pl-5 text-[11.5px] text-white/75">
                    <span>Auto-enable Zen during Pomodoro</span>
                    <input
                      type="checkbox"
                      checked={settings.linkDeepWorkToPomodoro}
                      onChange={(e) =>
                        onUpdateSettings({ linkDeepWorkToPomodoro: e.target.checked })
                      }
                      className="accent-sky-400 w-3.5 h-3.5 cursor-pointer"
                    />
                  </label>
                </div>

                <label className="flex items-center justify-between p-3 rounded-2xl bg-white/[0.04] border border-white/[0.08] cursor-pointer">
                  <span className="flex items-center gap-2 text-[12px] font-medium">
                    <MonitorOff className="w-3.5 h-3.5 text-emerald-300" />
                    Fullscreen Auto-Hide (Games / Cinema)
                  </span>
                  <input
                    type="checkbox"
                    checked={settings.fullscreenAutoHide}
                    onChange={(e) =>
                      onUpdateSettings({ fullscreenAutoHide: e.target.checked })
                    }
                    className="accent-emerald-400 w-4 h-4 cursor-pointer"
                  />
                </label>

                {/* Live Lyric Ticker in Notch Toggle */}
                <label className="flex items-center justify-between p-3 rounded-2xl bg-white/[0.04] border border-white/[0.08] cursor-pointer">
                  <span className="flex items-center gap-2 text-[12px] font-medium">
                    <Sparkles className="w-3.5 h-3.5 text-sky-300" />
                    Live Lyric Ticker in Notch
                  </span>
                  <input
                    type="checkbox"
                    checked={settings.liveLyricTicker}
                    onChange={(e) =>
                      onUpdateSettings({ liveLyricTicker: e.target.checked })
                    }
                    className="accent-sky-400 w-4 h-4 cursor-pointer"
                  />
                </label>

                {/* Section 2: Screenshot PrtSc Behavior Toggle */}
                <div className="p-3 rounded-2xl bg-white/[0.04] border border-white/[0.08] space-y-1.5">
                  <label className="flex items-center justify-between cursor-pointer">
                    <span className="flex items-center gap-2 text-[12px] font-medium">
                      <Camera className="w-3.5 h-3.5 text-amber-300" />
                      Auto-Hide Isle during Screenshots (PrtSc)
                    </span>
                    <input
                      type="checkbox"
                      checked={settings.autoHideOnScreenshot}
                      onChange={(e) =>
                        onUpdateSettings({ autoHideOnScreenshot: e.target.checked })
                      }
                      className="accent-amber-400 w-4 h-4 cursor-pointer"
                    />
                  </label>
                  <p className="text-[10px] text-white/50 pl-5.5 leading-relaxed">
                    {settings.autoHideOnScreenshot
                      ? 'Option A (Hide — Default): Isle becomes 100% transparent during capture so it never obstructs content behind it.'
                      : 'Option B (Stay Visible): Isle remains visible on top during capture (for showcasing and sharing).'}
                  </p>
                </div>
              </div>

              {/* Tab Manager (Toggle + Drag to Reorder) */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <p className="text-[11px] font-semibold uppercase tracking-wider text-white/50">
                    Tab Manager (Drag to Reorder)
                  </p>
                  <span className="text-[10px] text-white/40">
                    {enabledCount} active
                  </span>
                </div>

                <Reorder.Group
                  axis="y"
                  values={tabConfig}
                  onReorder={onReorderTabs}
                  className="space-y-1.5"
                >
                  {tabConfig.map((tab) => {
                    const disableUncheck = tab.enabled && enabledCount <= 1;
                    return (
                      <Reorder.Item
                        key={tab.key}
                        value={tab}
                        className="flex items-center justify-between px-3 py-2 rounded-xl bg-white/[0.05] border border-white/[0.08] cursor-grab active:cursor-grabbing"
                      >
                        <div className="flex items-center gap-2.5">
                          <GripVertical className="w-3.5 h-3.5 text-white/35" />
                          <span
                            className="w-2 h-2 rounded-full"
                            style={{ backgroundColor: tab.accent }}
                          />
                          <span className="text-[12px] font-medium text-white/90">
                            {tab.label}
                          </span>
                        </div>

                        <input
                          type="checkbox"
                          checked={tab.enabled}
                          disabled={disableUncheck}
                          onChange={() => onToggleTabEnabled(tab.key)}
                          className="accent-sky-400 w-4 h-4 cursor-pointer disabled:opacity-40"
                          aria-label={`Toggle ${tab.label} tab`}
                        />
                      </Reorder.Item>
                    );
                  })}
                </Reorder.Group>
              </div>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
