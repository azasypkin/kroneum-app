import { readFileSync } from 'node:fs';
import { Resvg } from '@resvg/resvg-js';
import { describe, expect, it } from 'vitest';
import { icon, logo } from '../../src/icons';

const asset = (name: string) =>
  readFileSync(
    new URL(`../../public/brand/${name}.svg`, import.meta.url),
    'utf8',
  );
const raster = (name: string, width: number) =>
  new Resvg(asset(name), {
    fitTo: { mode: 'width', value: width },
  }).render();
const png = (path: string) =>
  readFileSync(new URL(`../../src-tauri/icons/${path}`, import.meta.url));

describe('canonical vector branding', () => {
  it.each(['app-icon', 'mark', 'mark-light', 'template', 'outline'])(
    '%s uses vector geometry without bitmaps, filters, fonts, or remote resources',
    (name) => {
      const svg = asset(name);
      expect(svg).toContain('viewBox=');
      expect(svg).not.toMatch(/<(image|text|filter)\b|data:|https:\/\//);
      expect(() => raster(name, 128)).not.toThrow();
    },
  );

  it('uses the same ribbon and focus spark in the app, UI, and tray', () => {
    const paths = (name: string) =>
      [...asset(name).matchAll(/<path d="([^"]+)"/g)].map((match) => match[1]);
    const templatePaths = paths('template');
    expect(templatePaths).toHaveLength(3);
    for (const name of ['app-icon', 'mark', 'mark-light']) {
      for (const path of templatePaths) expect(paths(name)).toContain(path);
    }
  });

  it('retains a transparent eye aperture and an opaque central focus spark at tray size', () => {
    const rendered = raster('template', 48);
    expect([rendered.width, rendered.height]).toEqual([48, 36]);
    const alpha = (x: number, y: number) =>
      rendered.pixels[(y * rendered.width + x) * 4 + 3];
    expect(alpha(24, 18)).toBe(255);
    expect(alpha(16, 18)).toBe(0);
    expect(alpha(0, 0)).toBe(0);
    let hasAntialiasing = false;
    for (let i = 0; i < rendered.pixels.length; i += 4) {
      expect(rendered.pixels[i]).toBe(0);
      expect(rendered.pixels[i + 1]).toBe(0);
      expect(rendered.pixels[i + 2]).toBe(0);
      const a = rendered.pixels[i + 3]!;
      if (a > 0 && a < 255) hasAntialiasing = true;
    }
    expect(hasAntialiasing).toBe(true);
    expect(png('tray.png').equals(rendered.asPng())).toBe(true);
    expect(png('tray@1x.png').equals(raster('template', 24).asPng())).toBe(
      true,
    );
  });

  it.each([16, 32, 128, 256, 512])(
    'generates exact %s-point app icons at both macOS pixel densities',
    (size) => {
      for (const scale of [1, 2]) {
        const filename = `app.iconset/icon_${size}x${size}${scale === 2 ? '@2x' : ''}.png`;
        expect(
          png(filename).equals(raster('app-icon', size * scale).asPng()),
        ).toBe(true);
      }
    },
  );

  it('exports a 1024-pixel icon with transparent corners', () => {
    const rendered = raster('app-icon', 1024);
    expect([rendered.width, rendered.height]).toEqual([1024, 1024]);
    expect(rendered.pixels[3]).toBe(0);
    expect(png('icon.png').equals(rendered.asPng())).toBe(true);
  });

  it('uses a same-origin outline and decorative, non-draggable logo images', () => {
    expect(icon('eye')).toContain('/brand/outline.svg#eye-focus');
    expect(asset('outline')).toContain('id="eye-focus"');
    for (const variant of ['app', 'mark'] as const) {
      expect(logo(variant)).toContain('alt=""');
      expect(logo(variant)).toContain('draggable="false"');
    }
    expect(logo('mark')).toContain('prefers-color-scheme: dark');
  });
});
