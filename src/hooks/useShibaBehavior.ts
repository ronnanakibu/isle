import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { HudOverlay, ShibaSettings, ShibaState } from '../types';

interface UseShibaBehaviorParams {
  settings: ShibaSettings;
  simulatedFullscreen: boolean;
  pomodoroRunning: boolean;
  musicPlaying: boolean;
  batteryPct: number;
  charging: boolean;
  lastWaterLoggedAt: number;
  zenBreathingActive: boolean;
  moodOverride: ShibaState | null;
  onTriggerNudgeHud: (hud: HudOverlay, durationMs: number) => void;
}

function isWithinSchedule(startHHMM: string, endHHMM: string): boolean {
  const now = new Date();
  const curMins = now.getHours() * 60 + now.getMinutes();
  const [sh, sm] = startHHMM.split(':').map(Number);
  const [eh, em] = endHHMM.split(':').map(Number);
  if (!Number.isFinite(sh) || !Number.isFinite(sm) || !Number.isFinite(eh) || !Number.isFinite(em)) {
    return false;
  }
  const startMins = sh * 60 + sm;
  const endMins = eh * 60 + em;
  if (startMins <= endMins) {
    return curMins >= startMins && curMins <= endMins;
  }
  return curMins >= startMins || curMins <= endMins;
}

export function useShibaBehavior({
  settings,
  simulatedFullscreen,
  pomodoroRunning,
  musicPlaying,
  batteryPct,
  charging,
  lastWaterLoggedAt,
  zenBreathingActive,
  moodOverride,
  onTriggerNudgeHud,
}: UseShibaBehaviorParams) {
  const [realFullscreen, setRealFullscreen] = useState(false);
  const [eyeRestActive, setEyeRestActive] = useState(false);
  const [eyeRestRemainingSec, setEyeRestRemainingSec] = useState(0);
  const [happyBurst, setHappyBurst] = useState(false);

  const typingStartRef = useRef<number | null>(null);
  const lastKeystrokeRef = useRef<number>(0);
  const lastHydrationNudgeRef = useRef<number>(0);
  const happyTimerRef = useRef<number | null>(null);

  // 1.1 Fullscreen Auto-Hide listener
  useEffect(() => {
    const onFsChange = () => {
      setRealFullscreen(Boolean(document.fullscreenElement));
    };
    document.addEventListener('fullscreenchange', onFsChange);
    return () => document.removeEventListener('fullscreenchange', onFsChange);
  }, []);

  const isHiddenFullscreen =
    settings.fullscreenAutoHide && (realFullscreen || simulatedFullscreen);

  // 1.2 Deep Work & Focus Hours Mode
  const isDeepWorkActive = useMemo(() => {
    if (settings.deepWorkEnabled) return true;
    if (settings.linkDeepWorkToPomodoro && pomodoroRunning) return true;
    if (
      settings.deepWorkStart &&
      settings.deepWorkEnd &&
      settings.deepWorkEnabled &&
      isWithinSchedule(settings.deepWorkStart, settings.deepWorkEnd)
    ) {
      return true;
    }
    return false;
  }, [
    settings.deepWorkEnabled,
    settings.deepWorkStart,
    settings.deepWorkEnd,
    settings.linkDeepWorkToPomodoro,
    pomodoroRunning,
  ]);

  const triggerHappyBurst = useCallback(
    (durationMs = 2000) => {
      if (settings.dnd || isDeepWorkActive || settings.presenceLevel === 'stealth') return;
      setHappyBurst(true);
      if (happyTimerRef.current !== null) window.clearTimeout(happyTimerRef.current);
      happyTimerRef.current = window.setTimeout(() => setHappyBurst(false), durationMs);
    },
    [settings.dnd, isDeepWorkActive, settings.presenceLevel]
  );

  // 1.4 Smart Productive Nudge: Hydration (2 hours without water)
  const triggerHydrationNudge = useCallback(() => {
    if (settings.dnd || isDeepWorkActive || isHiddenFullscreen || settings.presenceLevel === 'stealth') {
      return;
    }
    onTriggerNudgeHud(
      {
        kind: 'nudge',
        nudgeType: 'hydration',
        message: 'Hydration check · Take a sip',
      },
      3000
    );
  }, [settings.dnd, isDeepWorkActive, isHiddenFullscreen, settings.presenceLevel, onTriggerNudgeHud]);

  useEffect(() => {
    const id = window.setInterval(() => {
      const now = Date.now();
      const twoHoursMs = 2 * 60 * 60 * 1000;
      if (
        now - lastWaterLoggedAt >= twoHoursMs &&
        now - lastHydrationNudgeRef.current >= twoHoursMs
      ) {
        lastHydrationNudgeRef.current = now;
        triggerHydrationNudge();
      }
    }, 60_000);
    return () => window.clearInterval(id);
  }, [lastWaterLoggedAt, triggerHydrationNudge]);

  // 1.4 Smart Productive Nudge: 20-20-20 Eye Rest after 50m continuous typing
  const triggerEyeRestNudge = useCallback(() => {
    if (settings.dnd || isHiddenFullscreen || settings.presenceLevel === 'stealth') return;
    setEyeRestActive(true);
    setEyeRestRemainingSec(20);
    onTriggerNudgeHud(
      {
        kind: 'nudge',
        nudgeType: 'eyeRest',
        message: '20-20-20 Eye Rest · Look 20ft away',
        remainingSec: 20,
      },
      20_000
    );
  }, [settings.dnd, isHiddenFullscreen, settings.presenceLevel, onTriggerNudgeHud]);

  useEffect(() => {
    const onKeyDown = () => {
      const now = Date.now();
      if (typingStartRef.current === null || now - lastKeystrokeRef.current > 120_000) {
        typingStartRef.current = now;
      }
      lastKeystrokeRef.current = now;
      if (now - typingStartRef.current >= 50 * 60 * 1000) {
        typingStartRef.current = now;
        triggerEyeRestNudge();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [triggerEyeRestNudge]);

  useEffect(() => {
    if (!eyeRestActive) return;
    const id = window.setInterval(() => {
      setEyeRestRemainingSec((prev) => {
        if (prev <= 1) {
          setEyeRestActive(false);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => window.clearInterval(id);
  }, [eyeRestActive]);

  // Resolve final Shiba visual state
  const shibaState: ShibaState = useMemo(() => {
    if (moodOverride) return moodOverride;
    if (zenBreathingActive || eyeRestActive || isDeepWorkActive || settings.dnd) return 'zen';
    if (happyBurst) return 'happy';
    if (charging) return 'charging';
    if (batteryPct < 20) return 'lowBattery';
    if (musicPlaying) return 'music';
    return 'normal';
  }, [
    moodOverride,
    zenBreathingActive,
    eyeRestActive,
    isDeepWorkActive,
    settings.dnd,
    happyBurst,
    charging,
    batteryPct,
    musicPlaying,
  ]);

  return {
    shibaState,
    isHiddenFullscreen,
    isDeepWorkActive,
    eyeRestActive,
    eyeRestRemainingSec,
    triggerHappyBurst,
    triggerHydrationNudge,
    triggerEyeRestNudge,
  };
}
