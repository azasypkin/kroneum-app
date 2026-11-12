// Keep this boundary in sync with the serialized types in engine.rs and lib.rs.
export type Timer = 'work' | 'break';
export type Sound = 'chime' | 'glass' | 'ping';
export type Action =
  | 'startWork'
  | 'startBreak'
  | 'pause'
  | 'resume'
  | 'stop'
  | 'acknowledge'
  | 'snooze'
  | 'cancelSnooze'
  | 'restoreScreens';

export interface Settings {
  workSeconds: number;
  breakSeconds: number;
  snoozeSeconds: number;
  idleSeconds: number;
  volume: number;
  sound: Sound;
  autoContinue: boolean;
  dimAfterWork: boolean;
  fadeSeconds: number;
  maxDimPercent: number;
  alwaysDimBreak: boolean;
  hideOnTimerStart: boolean;
}

export interface DimPlan {
  startedAtMs: number;
  fadeSeconds: number;
  maxDimPercent: number;
  clearAtMs: number | null;
}

export type Alert = { kind: 'finished'; timer: Timer } | { kind: 'idle' };

export interface Snapshot {
  phase: 'idle' | 'running' | 'paused' | 'alert' | 'snoozed';
  timer: Timer | null;
  remainingSeconds: number;
  totalSeconds: number;
  alert: Alert | null;
  nextTimer: Timer;
  completedBreaks: number;
  settings: Settings;
  launchAtLogin: boolean;
  warning: string | null;
  dimming: DimPlan | null;
}
