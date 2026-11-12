// Reproducible artwork from one vector source, rasterized at each native size.
import { mkdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { Resvg } from '@resvg/resvg-js';
import {
  appIconSvg,
  markSvg,
  outlineSvg,
  templateSvg,
} from './brand-artwork.mjs';

const root = new URL('../src-tauri/', import.meta.url);
const path = (name) => new URL(name, root);
mkdirSync(path('icons'), { recursive: true });
mkdirSync(path('sounds'), { recursive: true });
const brand = new URL('../public/brand/', import.meta.url);
mkdirSync(brand, { recursive: true });
const artwork = appIconSvg();
const template = templateSvg();
for (const [file, svg] of [
  ['app-icon.svg', artwork],
  ['mark.svg', markSvg()],
  ['mark-light.svg', markSvg('light')],
  ['template.svg', template],
  ['outline.svg', outlineSvg()],
])
  writeFileSync(new URL(file, brand), svg);
const rasterize = (svg, width) =>
  new Resvg(svg, { fitTo: { mode: 'width', value: width } }).render().asPng();
// macOS displays the template at 24×18 points. Supply exact 2× and 1× variants.
writeFileSync(path('icons/tray.png'), rasterize(template, 48));
writeFileSync(path('icons/tray@1x.png'), rasterize(template, 24));
writeFileSync(path('icons/icon.png'), rasterize(artwork, 1024));
if (process.platform === 'darwin') {
  mkdirSync(path('icons/app.iconset'), { recursive: true });
  for (const size of [16, 32, 128, 256, 512])
    for (const scale of [1, 2]) {
      writeFileSync(
        path(
          `icons/app.iconset/icon_${size}x${size}${scale === 2 ? '@2x' : ''}.png`,
        ),
        rasterize(artwork, size * scale),
      );
    }
  execFileSync('iconutil', [
    '-c',
    'icns',
    new URL('icons/app.iconset', root).pathname,
    '-o',
    new URL('icons/icon.icns', root).pathname,
  ]);
}

const rate = 22050;
const duration = 4;
const samples = rate * duration;
const wav = Buffer.alloc(44 + samples * 2);
wav.write('RIFF');
wav.writeUInt32LE(wav.length - 8, 4);
wav.write('WAVEfmt ', 8);
wav.writeUInt32LE(16, 16);
wav.writeUInt16LE(1, 20);
wav.writeUInt16LE(1, 22);
wav.writeUInt32LE(rate, 24);
wav.writeUInt32LE(rate * 2, 28);
wav.writeUInt16LE(2, 32);
wav.writeUInt16LE(16, 34);
wav.write('data', 36);
wav.writeUInt32LE(samples * 2, 40);
for (let i = 0; i < samples; i++) {
  const time = i / rate;
  let signal = 0;
  for (const [start, frequency, gain] of [
    [0, 659.25, 0.23],
    [0.18, 880, 0.18],
    [0.36, 1318.51, 0.09],
  ]) {
    const t = time - start;
    if (t >= 0 && t < 2.2) {
      const envelope =
        Math.min(1, t / 0.025) *
        Math.exp(-t * 3) *
        Math.min(1, (2.2 - t) / 0.1);
      signal +=
        gain *
        envelope *
        (Math.sin(2 * Math.PI * frequency * t) +
          0.15 * Math.sin(2 * Math.PI * frequency * 2 * t));
    }
  }
  wav.writeInt16LE(
    Math.round(Math.max(-1, Math.min(1, signal)) * 32767),
    44 + i * 2,
  );
}
writeFileSync(path('sounds/chime.wav'), wav);
console.log(
  'Generated app icons, menu-bar template, and a four-second chime loop.',
);
