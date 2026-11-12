import { describe, expect, it } from 'vitest';
import { durationLabel, formatTime, view } from '../../src/presentation';
import { defaultSettings } from '../../src/preview';
import type { Snapshot } from '../../src/types';

function snapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
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
    ...overrides,
  };
}

describe('countdown formatting', () => {
  it.each([
    [1200, '20:00'],
    [30, '00:30'],
    [0, '00:00'],
    [-1, '00:00'],
    [61, '01:01'],
    [14400, '240:00'],
  ])('formats %s as %s', (input, output) => {
    expect(formatTime(input as number)).toBe(output);
  });
  it.each([
    [30, '30 sec'],
    [300, '5 min'],
    [61, '1 min 1 sec'],
  ])('describes %s', (input, output) => {
    expect(durationLabel(input as number)).toBe(output);
  });
});

describe('timer presentation', () => {
  it('offers the work timer first', () => {
    expect(view(snapshot()).action).toBe('startWork');
  });
  it('shows break instructions without a live announcement each second', () => {
    const model = view(
      snapshot({
        phase: 'running',
        timer: 'break',
        remainingSeconds: 15,
        totalSeconds: 30,
      }),
    );
    expect(model.progress).toBe(0.5);
    expect(model.description).toContain('Blink');
  });
  it('offers resume while paused', () => {
    expect(view(snapshot({ phase: 'paused', timer: 'work' })).action).toBe(
      'resume',
    );
  });
  it('offers an eye break after work ends', () => {
    const model = view(
      snapshot({
        phase: 'alert',
        alert: { kind: 'finished', timer: 'work' },
        nextTimer: 'break',
      }),
    );
    expect(model.action).toBe('startBreak');
    expect(model.secondaryAction).toBe('snooze');
  });
  it('offers work after a break ends', () => {
    expect(
      view(
        snapshot({
          phase: 'alert',
          alert: { kind: 'finished', timer: 'break' },
          nextTimer: 'work',
        }),
      ).action,
    ).toBe('startWork');
  });
  it('never offers dismiss as the primary idle-reminder action', () => {
    expect(
      view(snapshot({ phase: 'alert', alert: { kind: 'idle' } })).action,
    ).toBe('startWork');
  });
  it('shows the configured snooze duration', () => {
    const model = view(
      snapshot({
        phase: 'alert',
        alert: { kind: 'idle' },
        settings: { ...defaultSettings, snoozeSeconds: 600 },
      }),
    );
    expect(model.secondary).toBe('Snooze for 10 min');
  });
});
