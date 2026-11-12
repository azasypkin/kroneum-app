import './styles.css';
import { createBridge, type Bridge } from './bridge';
import { icon, logo } from './icons';
import { durationLabel, view } from './presentation';
import type { Action, Settings, Snapshot, Sound } from './types';

const root = document.querySelector<HTMLDivElement>('#app')!;
root.innerHTML = `
  <div class="shell">
    <header class="app-header" data-tauri-drag-region="deep">
      <div class="brand"><span class="brand-mark">${logo('app')}</span><div><span class="brand-name">Kroneum</span><span class="brand-note">A little care, regularly.</span></div></div>
      <div class="header-actions" data-tauri-drag-region="false">
        <button class="icon-button" id="settings-open" aria-label="Open settings">${icon('settings')}</button>
        <button class="icon-button" id="hide" aria-label="Hide window">${icon('close')}</button>
      </div>
    </header>
    <main>
      <div id="timer-view">
        <div class="status-line" role="status"><span class="status-dot" aria-hidden="true"></span><span id="phase-label">Connecting to your timer…</span></div>
        <h1 id="heading">A calmer rhythm.</h1>
        <p class="intro" id="description">Focus for a while. Then give your eyes a little room.</p>
        <div class="timer-face">
          <svg class="timer-ring" viewBox="0 0 220 220" aria-hidden="true">
            <circle class="ring-track" cx="110" cy="110" r="98"/>
            <circle class="ring-value" id="ring" cx="110" cy="110" r="98" pathLength="1"/>
          </svg>
          <div class="face-content"><span class="face-eye">${logo('mark')}</span><span id="time" role="timer" aria-live="off">20:00</span><span id="caption">YOUR WORK TIMER</span></div>
        </div>
        <div class="preset-grid" aria-label="Start a timer">
          <button class="preset" id="work-preset" data-action="startWork">${icon('work')}<span class="preset-copy"><span class="preset-title">Work</span> <span id="work-duration">20 min</span></span><span class="preset-key" aria-hidden="true">⌘1</span></button>
          <button class="preset" id="break-preset" data-action="startBreak">${icon('eye')}<span class="preset-copy"><span class="preset-title">Eye break</span> <span id="break-duration">30 sec</span></span><span class="preset-key" aria-hidden="true">⌘2</span></button>
        </div>
        <div class="timer-actions">
          <button class="primary" id="primary" disabled><span id="primary-label">Start work timer</span>${icon('arrow')}</button>
          <button class="text-button" id="secondary" disabled>Take an eye break</button>
          <button class="text-button dismiss-button" id="acknowledge" data-action="acknowledge" hidden>Accept & dismiss</button>
        </div>
        <aside class="care-note">${icon('leaf')}<p id="care-note">Look far away. Blink gently.<br />Small breaks make a good habit.</p></aside>
      </div>
      <div id="settings-view" hidden>
        <button class="text-button back-button" id="settings-back">${icon('back')}Back to timer</button>
        <h1 tabindex="-1" id="settings-heading">Your rhythm.</h1>
        <p class="intro">Make these little pauses work for you.</p>
        <form id="settings-form">
          <fieldset class="settings-group"><legend>Timers</legend>
            <div class="setting-row"><label for="work-seconds">Work timer <span class="field-hint">1–14,400 seconds</span></label><div class="number-control"><input id="work-seconds" name="workSeconds" type="number" min="1" max="14400" step="1" required /><span>sec</span></div></div>
            <div class="setting-row"><label for="break-seconds">Eye break <span class="field-hint">1–3,600 seconds</span></label><div class="number-control"><input id="break-seconds" name="breakSeconds" type="number" min="1" max="3600" step="1" required /><span>sec</span></div></div>
          </fieldset>
          <fieldset class="settings-group"><legend>Reminders</legend>
            <div class="setting-row"><label for="snooze-seconds">Snooze for <span class="field-hint">1–3,600 seconds</span></label><div class="number-control"><input id="snooze-seconds" name="snoozeSeconds" type="number" min="1" max="3600" step="1" required /><span>sec</span></div></div>
            <div class="setting-row"><label for="idle-seconds">Nudge after idle <span class="field-hint">No timer active · 1–3,600 sec</span></label><div class="number-control"><input id="idle-seconds" name="idleSeconds" type="number" min="1" max="3600" step="1" required /><span>sec</span></div></div>
            <div class="setting-row"><label for="sound">Reminder sound</label><select id="sound" name="sound"><option value="chime">Soft chime</option><option value="glass">Glass</option><option value="ping">Ping</option></select></div>
            <div class="volume-row"><label for="volume">Volume <output id="volume-value" for="volume">45%</output></label><div class="volume-controls"><input id="volume" name="volume" type="range" min="5" max="100" step="1" /><button type="button" class="icon-button" id="preview-sound" aria-label="Try reminder sound">${icon('sound')}</button></div></div>
          </fieldset>
          <fieldset class="settings-group"><legend>Screen fading</legend>
            <div class="setting-row"><label for="dim-after-work">Dim screens after work <span class="field-hint">Start fading when work finishes.</span></label><input id="dim-after-work" name="dimAfterWork" type="checkbox" role="switch" aria-describedby="dimming-hint" /></div>
            <div class="setting-row"><label for="always-dim-break">Always dim during eye breaks <span class="field-hint">Includes manually started breaks.</span></label><input id="always-dim-break" name="alwaysDimBreak" type="checkbox" role="switch" aria-describedby="dimming-hint" /></div>
            <p class="settings-note dimming-hint" id="dimming-hint">Start Work or use Restore screens to clear dimming. At 100%, desktop content is black but Kroneum and the menu bar remain visible.</p>
            <div class="setting-row"><label for="fade-seconds">Fade duration <span class="field-hint">To the selected limit · 1–600 sec</span></label><div class="number-control"><input id="fade-seconds" name="fadeSeconds" type="number" min="1" max="600" step="1" required disabled aria-describedby="dimming-hint" /><span>sec</span></div></div>
            <div class="volume-row"><div class="range-label"><label for="max-dim-percent">Maximum dimming</label><output id="max-dim-value" for="max-dim-percent" aria-hidden="true">85%</output></div><div class="volume-controls"><input id="max-dim-percent" name="maxDimPercent" type="range" min="1" max="100" step="1" value="85" aria-valuetext="85% dimming" aria-describedby="dimming-hint" disabled /></div></div>
          </fieldset>
          <fieldset class="settings-group"><legend>A little more automatic</legend>
            <div class="setting-row"><label for="auto-continue">Continue the cycle <span class="field-hint">Accept starts the next timer.</span></label><input id="auto-continue" name="autoContinue" type="checkbox" role="switch" /></div>
            <div class="setting-row"><label for="hide-on-timer-start">Hide window after starting a timer <span class="field-hint" id="auto-hide-hint">Keep the menu-bar timer. Reopen for reminders.</span></label><input id="hide-on-timer-start" name="hideOnTimerStart" type="checkbox" role="switch" aria-describedby="auto-hide-hint" /></div>
            <div class="setting-row"><label for="launch-login">Launch at login <span class="field-hint" id="login-hint">Start quietly in the menu bar.</span></label><input id="launch-login" type="checkbox" role="switch" aria-describedby="login-hint" /></div>
          </fieldset>
          <p class="settings-note">Timer changes apply to the next start. A paused timer suspends reminders.</p>
          <button class="primary" type="submit" id="save-settings"><span>Save settings</span>${icon('check')}</button>
          <button class="text-button" type="button" id="reset-settings">Use default settings</button>
        </form>
      </div>
    </main>
    <div class="message-area"><button class="text-button restore-button" id="restore-screens" data-action="restoreScreens" hidden>Restore screens</button><p class="message" id="message" role="status" hidden></p><p class="warning" id="warning" role="status" hidden></p></div>
    <footer><span id="footer-note">Made for your eyes.</span><button class="footer-button" id="quit">Quit</button></footer>
  </div>
`;

function el<T extends HTMLElement = HTMLElement>(id: string): T {
  return document.getElementById(id) as T;
}

let bridge: Bridge;
let state: Snapshot | undefined;
let busy = false;
let settingsOpen = false;
let previousPhase = '';
let pendingAlertFocus = false;
let unlisten: (() => void) | undefined;

function message(text: string, error = false) {
  const node = el('message');
  node.textContent = text;
  node.classList.toggle('error', error);
  node.hidden = !text;
}

function render(s: Snapshot) {
  state = s;
  const model = view(s);
  root.dataset.phase = s.phase;
  root.dataset.kind = model.kind;
  if (el('phase-label').textContent !== model.status) {
    el('phase-label').textContent = model.status;
  }
  el('heading').textContent = model.heading;
  el('description').textContent = model.description;
  el('time').textContent = model.time;
  if (el('caption').textContent !== model.caption) {
    el('caption').textContent = model.caption;
  }
  el('ring').style.strokeDashoffset = String(1 - model.progress);
  el('primary-label').textContent = model.primary;
  el('primary').dataset.action = model.action;
  el('secondary').textContent = model.secondary;
  el('secondary').dataset.action = model.secondaryAction;
  el('work-duration').textContent = durationLabel(s.settings.workSeconds);
  el('break-duration').textContent = durationLabel(s.settings.breakSeconds);
  const completed = s.completedBreaks;
  el('footer-note').textContent = bridge.native
    ? completed === 0
      ? 'Made for your eyes.'
      : `${completed} eye ${completed === 1 ? 'break' : 'breaks'} this session`
    : 'UI preview · no native alarms';
  el('work-preset').classList.toggle('selected', s.timer === 'work');
  el('break-preset').classList.toggle('selected', s.timer === 'break');
  el('restore-screens').hidden = !s.dimming;
  el('acknowledge').hidden =
    s.phase !== 'alert' || s.alert?.kind !== 'finished';
  el('acknowledge').textContent = s.settings.autoContinue
    ? 'Accept & start next timer'
    : 'Accept & dismiss';
  el('care-note').textContent =
    s.phase === 'alert'
      ? 'The sound repeats until you respond or snooze.'
      : s.phase === 'paused'
        ? 'Take your time. Reminders are paused too.'
        : s.phase === 'idle'
          ? `Look far away. Blink gently. Idle reminder in ${durationLabel(s.remainingSeconds)}.`
          : 'Look far away. Blink gently. Small breaks make a good habit.';
  if (el('warning').textContent !== (s.warning ?? '')) {
    el('warning').textContent = s.warning;
  }
  el('warning').hidden = !s.warning;
  for (const node of document.querySelectorAll<HTMLButtonElement>(
    '[data-action]',
  ))
    node.disabled = busy;
  if (previousPhase !== s.phase && s.phase === 'alert') {
    closeSettings(false);
    if (!busy) el('primary').focus();
    else pendingAlertFocus = true;
  }
  previousPhase = s.phase;
}

async function run(task: () => Promise<void>) {
  if (busy) return;
  busy = true;
  message('');
  if (state) render(state);
  try {
    await task();
  } catch (error) {
    message(String(error instanceof Error ? error.message : error), true);
  } finally {
    busy = false;
    if (state) render(state);
    if (pendingAlertFocus) {
      el('primary').focus();
      pendingAlertFocus = false;
    }
  }
}

function fillSettings(s: Settings) {
  for (const key of [
    'workSeconds',
    'breakSeconds',
    'snoozeSeconds',
    'idleSeconds',
    'fadeSeconds',
  ] as const) {
    const input = document.querySelector<HTMLInputElement>(`[name="${key}"]`)!;
    input.value = String(s[key]);
  }
  el<HTMLSelectElement>('sound').value = s.sound;
  el<HTMLInputElement>('volume').value = String(Math.round(s.volume * 100));
  el('volume-value').textContent = `${Math.round(s.volume * 100)}%`;
  el<HTMLInputElement>('auto-continue').checked = s.autoContinue;
  el<HTMLInputElement>('hide-on-timer-start').checked = s.hideOnTimerStart;
  el<HTMLInputElement>('dim-after-work').checked = s.dimAfterWork;
  el<HTMLInputElement>('always-dim-break').checked = s.alwaysDimBreak;
  el<HTMLInputElement>('max-dim-percent').value = String(s.maxDimPercent);
  syncFadeControls();
}

function syncFadeControls() {
  const enabled =
    el<HTMLInputElement>('dim-after-work').checked ||
    el<HTMLInputElement>('always-dim-break').checked;
  el<HTMLInputElement>('fade-seconds').disabled = !enabled;
  const maximum = el<HTMLInputElement>('max-dim-percent');
  maximum.disabled = !enabled;
  el('max-dim-value').textContent = `${maximum.value}%`;
  maximum.setAttribute('aria-valuetext', `${maximum.value}% dimming`);
}

function openSettings() {
  if (!state || busy) return;
  fillSettings(state.settings);
  el<HTMLInputElement>('launch-login').checked = state.launchAtLogin;
  el<HTMLInputElement>('launch-login').disabled = !bridge.native;
  el('login-hint').textContent = bridge.native
    ? 'Start quietly in the menu bar.'
    : 'Available in the macOS app.';
  el('dimming-hint').textContent = bridge.native
    ? 'Start Work or use Restore screens to clear dimming. At 100%, desktop content is black but Kroneum and the menu bar remain visible.'
    : 'Settings preview only: native screen overlays are not shown here. Both fading modes restore screens when the eye break finishes.';
  settingsOpen = true;
  el('timer-view').hidden = true;
  el('settings-view').hidden = false;
  el('settings-heading').focus();
  message('');
}

function closeSettings(focus = true) {
  settingsOpen = false;
  el('timer-view').hidden = false;
  el('settings-view').hidden = true;
  if (focus) el('settings-open').focus();
}

root.addEventListener('click', (event) => {
  const button = (event.target as Element).closest<HTMLButtonElement>(
    'button[data-action]',
  );
  if (button?.dataset.action && bridge) {
    void run(async () => {
      render(await bridge.act(button.dataset.action as Action));
    });
  }
});

el('settings-open').addEventListener('click', () =>
  settingsOpen ? closeSettings() : openSettings(),
);
el('settings-back').addEventListener('click', () => closeSettings());
el('hide').addEventListener('click', () => {
  if (bridge) void run(() => bridge.hide());
});
el('quit').addEventListener('click', () => {
  if (bridge) void run(() => bridge.quit());
});
el('volume').addEventListener('input', () => {
  el('volume-value').textContent = `${el<HTMLInputElement>('volume').value}%`;
});
el('dim-after-work').addEventListener('change', syncFadeControls);
el('always-dim-break').addEventListener('change', syncFadeControls);
el('max-dim-percent').addEventListener('input', syncFadeControls);
el('preview-sound').addEventListener('click', () => {
  void run(() =>
    bridge.preview(
      el<HTMLSelectElement>('sound').value as Sound,
      Number(el<HTMLInputElement>('volume').value) / 100,
    ),
  );
});
el('launch-login').addEventListener('change', () => {
  const input = el<HTMLInputElement>('launch-login');
  void run(async () => {
    try {
      render(await bridge.login(input.checked));
    } catch (e) {
      input.checked = state?.launchAtLogin ?? false;
      throw e;
    }
  });
});
el('reset-settings').addEventListener('click', async () => {
  const { defaultSettings } = await import('./preview');
  fillSettings(defaultSettings);
  message('Defaults are ready. Save settings to apply them.');
});
el('settings-form').addEventListener('submit', (event) => {
  event.preventDefault();
  const form = el<HTMLFormElement>('settings-form');
  if (!form.reportValidity()) return;
  const data = new FormData(form);
  const fadeValue = Number(el<HTMLInputElement>('fade-seconds').value);
  const settings: Settings = {
    workSeconds: Number(data.get('workSeconds')),
    breakSeconds: Number(data.get('breakSeconds')),
    snoozeSeconds: Number(data.get('snoozeSeconds')),
    idleSeconds: Number(data.get('idleSeconds')),
    volume: Number(data.get('volume')) / 100,
    sound: data.get('sound') as Sound,
    autoContinue: data.has('autoContinue'),
    hideOnTimerStart: data.has('hideOnTimerStart'),
    dimAfterWork: data.has('dimAfterWork'),
    alwaysDimBreak: data.has('alwaysDimBreak'),
    maxDimPercent: Number(el<HTMLInputElement>('max-dim-percent').value),
    fadeSeconds:
      data.has('dimAfterWork') ||
      data.has('alwaysDimBreak') ||
      (Number.isInteger(fadeValue) && fadeValue >= 1 && fadeValue <= 600)
        ? fadeValue
        : (state?.settings.fadeSeconds ?? 30),
  };
  void run(async () => {
    render(await bridge.save(settings));
    message('Settings saved.');
  });
});

document.addEventListener('keydown', (event) => {
  if (!bridge || busy || event.repeat) return;
  if (event.key === 'Escape') {
    event.preventDefault();
    if (state?.dimming)
      void run(async () => {
        render(await bridge.act('restoreScreens'));
      });
    else if (settingsOpen) closeSettings();
    else void bridge.hide().catch((e) => message(String(e), true));
  }
  if (!event.metaKey || event.altKey || event.ctrlKey) return;
  if (event.shiftKey && event.key.toLowerCase() === 'r') {
    event.preventDefault();
    void run(async () => {
      render(await bridge.act('restoreScreens'));
    });
    return;
  }
  const action =
    event.key === '1'
      ? 'startWork'
      : event.key === '2'
        ? 'startBreak'
        : undefined;
  if (action) {
    event.preventDefault();
    void run(async () => {
      render(await bridge.act(action));
    });
  }
  if (event.key === ',') {
    event.preventDefault();
    openSettings();
  }
  if (event.key.toLowerCase() === 'q') {
    event.preventDefault();
    void bridge.quit().catch((e) => message(String(e), true));
  }
});

// Subscribe first, then fetch, so a deadline cannot pass in the initialization gap.
async function initialize() {
  try {
    bridge = await createBridge();
    root.dataset.native = String(bridge.native);
    unlisten = await bridge.subscribe(render);
    render(await bridge.state());
  } catch (error) {
    message(
      `The timer could not connect. Restart Kroneum. ${String(error)}`,
      true,
    );
  }
}
window.addEventListener('focus', () => {
  if (bridge)
    void bridge
      .state()
      .then(render)
      .catch((e) => message(String(e), true));
});
window.addEventListener('pagehide', () => unlisten?.(), { once: true });
void initialize();
