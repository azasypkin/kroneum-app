import type { Snapshot, Timer } from './types';

export function formatTime(seconds: number): string {
  const safe = Math.max(0, Math.ceil(seconds));
  return `${Math.floor(safe / 60)
    .toString()
    .padStart(2, '0')}:${(safe % 60).toString().padStart(2, '0')}`;
}

export function durationLabel(seconds: number): string {
  if (seconds < 60) return `${seconds} sec`;
  if (seconds % 60 === 0) return `${seconds / 60} min`;
  return `${Math.floor(seconds / 60)} min ${seconds % 60} sec`;
}

export function startAction(timer: Timer) {
  return timer === 'work' ? ('startWork' as const) : ('startBreak' as const);
}

export function view(s: Snapshot) {
  const onBreak = s.timer === 'break';
  const nextLabel =
    s.nextTimer === 'break' ? 'Start eye break' : 'Back to work';
  const base = {
    status: 'Ready when you are',
    heading: 'A calmer rhythm.',
    description: 'Focus for a while. Then give your eyes a little room.',
    time: formatTime(s.settings.workSeconds),
    caption: 'YOUR WORK TIMER',
    primary: 'Start work timer',
    action: 'startWork' as import('./types').Action,
    secondary: 'Take an eye break',
    secondaryAction: 'startBreak' as import('./types').Action,
    progress: 1,
    kind: 'work',
  };
  if (s.phase === 'running' || s.phase === 'paused') {
    return {
      ...base,
      status:
        s.phase === 'paused'
          ? 'Paused · reminders suspended'
          : onBreak
            ? 'A moment for your eyes'
            : 'Time to focus',
      heading: onBreak ? 'Look beyond the screen.' : 'One thing at a time.',
      description: onBreak
        ? 'Look at something far away. Blink slowly and gently.'
        : 'Your next eye break is on its way. Settle into your work.',
      time: formatTime(s.remainingSeconds),
      caption: onBreak ? 'LOOK FAR AWAY & BLINK' : 'UNTIL YOUR EYE BREAK',
      primary: s.phase === 'paused' ? 'Resume timer' : 'Pause timer',
      action: s.phase === 'paused' ? ('resume' as const) : ('pause' as const),
      secondary: 'Stop timer',
      secondaryAction: 'stop' as const,
      progress:
        s.totalSeconds > 0
          ? Math.min(1, s.remainingSeconds / s.totalSeconds)
          : 0,
      kind: onBreak ? 'break' : 'work',
    };
  }
  if (s.phase === 'snoozed') {
    return {
      ...base,
      status: 'A little later',
      heading: 'A moment of breathing room.',
      description:
        'The sound is off. Your reminder will return when snooze ends.',
      time: formatTime(s.remainingSeconds),
      caption: 'UNTIL THE REMINDER RETURNS',
      primary: nextLabel,
      action: startAction(s.nextTimer),
      secondary: 'End snooze now',
      secondaryAction: 'cancelSnooze' as const,
      progress:
        s.totalSeconds > 0
          ? Math.min(1, s.remainingSeconds / s.totalSeconds)
          : 0,
      kind: 'snooze',
    };
  }
  if (s.phase === 'alert') {
    const idle = s.alert?.kind === 'idle';
    const workFinished =
      s.alert?.kind === 'finished' && s.alert.timer === 'work';
    return {
      ...base,
      status: idle ? 'Let’s pick a timer' : 'Your timer is complete',
      heading: idle
        ? 'Find your rhythm again.'
        : workFinished
          ? 'Your eyes deserve a break.'
          : 'A fresh start.',
      description: idle
        ? 'No timer is active. Choose work or an eye break to stop the reminder.'
        : workFinished
          ? 'Look far away and blink. Start the eye break when you are ready.'
          : 'Your eye break is complete. Return to work when you are ready.',
      time: '00:00',
      caption: idle
        ? 'A FRIENDLY NUDGE'
        : workFinished
          ? 'TIME TO LOOK FAR AWAY'
          : 'EYE BREAK COMPLETE',
      primary: idle ? 'Start work timer' : nextLabel,
      action: idle ? ('startWork' as const) : startAction(s.nextTimer),
      secondary: `Snooze for ${durationLabel(s.settings.snoozeSeconds)}`,
      secondaryAction: 'snooze' as const,
      progress: 0,
      kind: 'alert',
    };
  }
  return base;
}
