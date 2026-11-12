import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';

it('allows native header dragging only from the local main window', () => {
  const capability = JSON.parse(
    readFileSync(
      new URL('../../src-tauri/capabilities/main.json', import.meta.url),
      'utf8',
    ),
  );
  expect(capability.windows).toEqual(['main']);
  expect(capability.permissions).toContain('core:window:allow-start-dragging');
  expect(capability.remote).toBeUndefined();
});
