import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPreviewBridge, defaultSettings } from '../../src/preview';

beforeEach(() => {
  vi.stubGlobal('location', new URL('http://127.0.0.1:1420/'));
});
afterEach(() => {
  vi.unstubAllGlobals();
});

describe('preview parity for native auto-hide effects', () => {
  it('keeps auto-hide off by default', async () => {
    const bridge = createPreviewBridge();
    const hide = vi.spyOn(bridge, 'hide');
    await bridge.act('startWork');
    await bridge.act('startBreak');
    expect(hide).not.toHaveBeenCalled();
    expect((await bridge.state()).settings.hideOnTimerStart).toBe(false);
  });

  it.each(['startWork', 'startBreak'] as const)(
    'requests one hide after %s succeeds and leaves the timer running',
    async (action) => {
      const bridge = createPreviewBridge();
      const hide = vi.spyOn(bridge, 'hide');
      await bridge.save({ ...defaultSettings, hideOnTimerStart: true });
      expect(hide).not.toHaveBeenCalled();
      const state = await bridge.act(action);
      expect(hide).toHaveBeenCalledTimes(1);
      expect(state.phase).toBe('running');
      expect(state.remainingSeconds).toBeGreaterThan(0);
      await bridge.state();
      expect(hide).toHaveBeenCalledTimes(1);
    },
  );

  it('does not hide on pause, resume, stop, restoration, or a failed action', async () => {
    const bridge = createPreviewBridge();
    await bridge.save({ ...defaultSettings, hideOnTimerStart: true });
    await bridge.act('startWork');
    const hide = vi.spyOn(bridge, 'hide');
    await bridge.act('pause');
    await bridge.act('resume');
    await bridge.act('restoreScreens');
    await bridge.act('stop');
    await expect(bridge.act('resume')).rejects.toThrow();
    expect(hide).not.toHaveBeenCalled();
  });

  it.each([false, true])(
    'hides after acceptance only if acceptance starts the cycle (%s)',
    async (autoContinue) => {
      vi.stubGlobal(
        'location',
        new URL('http://127.0.0.1:1420/?preview=alert'),
      );
      const bridge = createPreviewBridge();
      const hide = vi.spyOn(bridge, 'hide');
      await bridge.save({
        ...defaultSettings,
        hideOnTimerStart: true,
        autoContinue,
      });
      await bridge.act('acknowledge');
      expect(hide).toHaveBeenCalledTimes(autoContinue ? 1 : 0);
    },
  );

  it('keeps snooze visible and honors a later disabled setting', async () => {
    vi.stubGlobal('location', new URL('http://127.0.0.1:1420/?preview=alert'));
    const bridge = createPreviewBridge();
    const hide = vi.spyOn(bridge, 'hide');
    await bridge.save({ ...defaultSettings, hideOnTimerStart: true });
    await bridge.act('snooze');
    expect(hide).not.toHaveBeenCalled();
    await bridge.save(defaultSettings);
    await bridge.act('startBreak');
    expect(hide).not.toHaveBeenCalled();
  });
});
