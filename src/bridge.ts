import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Action, Settings, Snapshot, Sound } from './types';

export interface Bridge {
  native: boolean;
  state(): Promise<Snapshot>;
  act(action: Action): Promise<Snapshot>;
  save(settings: Settings): Promise<Snapshot>;
  login(enabled: boolean): Promise<Snapshot>;
  preview(sound: Sound, volume: number): Promise<void>;
  hide(): Promise<void>;
  quit(): Promise<void>;
  subscribe(handler: (s: Snapshot) => void): Promise<UnlistenFn>;
}

export async function createBridge(): Promise<Bridge> {
  if (!isTauri()) {
    // Preview is an explicit UI-only simulator, never the production timer.
    const { createPreviewBridge } = await import('./preview');
    return createPreviewBridge();
  }
  return {
    native: true,
    state: () => invoke('get_state'),
    act: (action) => invoke('timer_action', { action }),
    save: (settings) => invoke('save_settings', { settings }),
    login: (enabled) => invoke('set_launch_at_login', { enabled }),
    preview: (sound, volume) => invoke('preview_sound', { sound, volume }),
    hide: () => invoke('hide_window'),
    quit: () => invoke('quit_app'),
    subscribe: (handler) =>
      listen<Snapshot>('timer-state', (event) => handler(event.payload)),
  };
}
