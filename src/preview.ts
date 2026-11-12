import type { Bridge } from './bridge';
import type { Action, Settings, Snapshot, Timer } from './types';

export const defaultSettings: Settings = {
  workSeconds: 1200,
  breakSeconds: 30,
  snoozeSeconds: 300,
  idleSeconds: 300,
  volume: 0.45,
  sound: 'chime',
  autoContinue: false,
  dimAfterWork: false,
  fadeSeconds: 30,
  maxDimPercent: 85,
  alwaysDimBreak: false,
  hideOnTimerStart: false,
};

export function createPreviewBridge(): Bridge {
  let s: Snapshot = {
    phase: 'idle',
    timer: null,
    remainingSeconds: 300,
    totalSeconds: 300,
    alert: null,
    nextTimer: 'work',
    completedBreaks: 0,
    settings: { ...defaultSettings },
    launchAtLogin: false,
    warning: null,
    dimming: null,
  };
  let deadline = Date.now() + 300_000;
  let listener: ((s: Snapshot) => void) | undefined;
  let breakDue = false;
  let breakActive = false;
  let dimSuppressed = false;
  const syncDimming = () => {
    const wanted =
      (s.settings.dimAfterWork && breakDue) ||
      (s.settings.alwaysDimBreak && breakActive);
    if (!wanted) s.dimming = null;
    else if (!dimSuppressed && !s.dimming) {
      s.dimming = {
        startedAtMs: Date.now(),
        fadeSeconds: s.settings.fadeSeconds,
        maxDimPercent: s.settings.maxDimPercent,
        clearAtMs: null,
      };
    }
    if (s.dimming) {
      s.dimming.maxDimPercent = s.settings.maxDimPercent;
      s.dimming.clearAtMs =
        s.phase === 'running' && s.timer === 'break' ? deadline : null;
    }
  };
  const result = () => structuredClone(s);
  const emit = () => listener?.(result());
  const start = (timer: Timer) => {
    if (timer === 'work') {
      breakDue = false;
      breakActive = false;
      dimSuppressed = false;
      s.dimming = null;
    } else {
      breakActive = true;
      if (s.settings.alwaysDimBreak) dimSuppressed = false;
    }
    s.phase = 'running';
    s.timer = timer;
    s.alert = null;
    s.nextTimer = timer === 'work' ? 'break' : 'work';
    s.totalSeconds =
      timer === 'work' ? s.settings.workSeconds : s.settings.breakSeconds;
    s.remainingSeconds = s.totalSeconds;
    deadline = Date.now() + s.totalSeconds * 1000;
    syncDimming();
  };
  const idle = () => {
    s.phase = 'idle';
    s.timer = null;
    s.alert = null;
    s.totalSeconds = s.settings.idleSeconds;
    s.remainingSeconds = s.totalSeconds;
    deadline = Date.now() + s.totalSeconds * 1000;
  };
  const tick = () => {
    if (s.phase === 'running' || s.phase === 'snoozed' || s.phase === 'idle') {
      s.remainingSeconds = Math.max(
        0,
        Math.ceil((deadline - Date.now()) / 1000),
      );
      if (s.remainingSeconds === 0) {
        if (s.phase === 'running') {
          s.alert = { kind: 'finished', timer: s.timer! };
          if (s.timer === 'break') {
            s.completedBreaks++;
            breakDue = false;
            breakActive = false;
            s.dimming = null;
          } else {
            breakDue = true;
            dimSuppressed = false;
          }
        } else if (s.phase === 'idle') {
          s.alert = { kind: 'idle' };
        }
        s.phase = 'alert';
        s.timer = null;
        s.totalSeconds = 0;
        syncDimming();
      }
    }
  };
  // Deterministic visual fixtures for browser tests and design review.
  const fixture = new URLSearchParams(location.search).get('preview');
  if (fixture === 'work' || fixture === 'break') start(fixture);
  if (
    fixture === 'alert' ||
    fixture === 'idle-alert' ||
    fixture === 'break-alert'
  ) {
    s.phase = 'alert';
    s.remainingSeconds = 0;
    s.totalSeconds = 0;
    s.alert =
      fixture === 'idle-alert'
        ? { kind: 'idle' }
        : {
            kind: 'finished',
            timer: fixture === 'break-alert' ? 'break' : 'work',
          };
    s.nextTimer = fixture === 'alert' ? 'break' : 'work';
    if (fixture === 'alert') breakDue = true;
  }
  const bridge: Bridge = {
    native: false,
    state: async () => {
      tick();
      return result();
    },
    act: async (action: Action) => {
      tick();
      let timerStarted = false;
      switch (action) {
        case 'startWork':
          start('work');
          timerStarted = true;
          break;
        case 'startBreak':
          start('break');
          timerStarted = true;
          break;
        case 'pause':
          if (s.phase !== 'running') throw Error('No timer is running.');
          s.phase = 'paused';
          break;
        case 'resume':
          if (s.phase !== 'paused') throw Error('No timer is paused.');
          s.phase = 'running';
          deadline = Date.now() + s.remainingSeconds * 1000;
          break;
        case 'stop':
          if (s.phase !== 'running' && s.phase !== 'paused')
            throw Error('No timer is active.');
          breakActive = false;
          idle();
          break;
        case 'snooze':
          if (s.phase !== 'alert')
            throw Error('There is no reminder to snooze.');
          s.phase = 'snoozed';
          s.totalSeconds = s.settings.snoozeSeconds;
          s.remainingSeconds = s.totalSeconds;
          deadline = Date.now() + s.totalSeconds * 1000;
          break;
        case 'cancelSnooze':
          if (s.phase !== 'snoozed') throw Error('No reminder is snoozed.');
          s.phase = 'alert';
          s.remainingSeconds = 0;
          break;
        case 'acknowledge':
          if (s.phase !== 'alert' || s.alert?.kind !== 'finished')
            throw Error('Pick a timer or snooze the idle reminder.');
          if (s.settings.autoContinue) {
            start(s.nextTimer);
            timerStarted = true;
          } else idle();
          break;
        case 'restoreScreens':
          s.dimming = null;
          dimSuppressed = true;
          break;
      }
      syncDimming();
      emit();
      // The browser preview keeps its window visible, but exposes the same hide
      // request through Bridge.hide so tests can inspect the effect.
      if (timerStarted && s.settings.hideOnTimerStart) await bridge.hide();
      return result();
    },
    save: async (settings) => {
      if (
        (settings.dimAfterWork && !s.settings.dimAfterWork && breakDue) ||
        (settings.alwaysDimBreak && !s.settings.alwaysDimBreak && breakActive)
      )
        dimSuppressed = false;
      s.settings = { ...settings };
      syncDimming();
      emit();
      return result();
    },
    login: async () => {
      throw Error('Launch at login is available in the macOS app.');
    },
    preview: async () => {
      throw Error('Native sound is available in the macOS app.');
    },
    hide: async () => {},
    quit: async () => {},
    subscribe: async (handler) => {
      listener = handler;
      const interval = window.setInterval(() => {
        tick();
        emit();
      }, 1000);
      return () => {
        window.clearInterval(interval);
        listener = undefined;
      };
    },
  };
  return bridge;
}
