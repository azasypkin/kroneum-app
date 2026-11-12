// Canonical vector geometry, redrawn from the supplied eye / focus / cycle design.
// Every app, UI, and menu-bar asset derives from these paths. No embedded bitmaps.
export const ribbon =
  'M4 52C20 49 29 18 51 9C79-2 100 19 124 46C104 31 88 22 67 28C43 34 25 57 4 52Z';
export const spark =
  'M64 34C65.6 43.2 68.5 46.4 78 48C68.5 49.6 65.6 52.8 64 62C62.4 52.8 59.5 49.6 50 48C59.5 46.4 62.4 43.2 64 34Z';
const root = (viewBox, content, attributes = '') =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}" ${attributes}>${content}</svg>\n`;

const gradients = `
  <linearGradient id="ribbon" x1="18" y1="29" x2="117" y2="45" gradientUnits="userSpaceOnUse">
    <stop stop-color="#76b67a"/>
    <stop offset=".22" stop-color="#e4f6b2"/>
    <stop offset=".48" stop-color="#c6e99c"/>
    <stop offset=".72" stop-color="#91c887"/>
    <stop offset="1" stop-color="#285d48"/>
  </linearGradient>
  <linearGradient id="ribbon-edge" x1="40" y1="9" x2="102" y2="44" gradientUnits="userSpaceOnUse">
    <stop stop-color="#efffda" stop-opacity=".9"/>
    <stop offset=".5" stop-color="#d7f0b0" stop-opacity=".35"/>
    <stop offset="1" stop-color="#8dca8b" stop-opacity="0"/>
  </linearGradient>
  <radialGradient id="spark" cx=".44" cy=".36" r=".7">
    <stop stop-color="#ffffed"/><stop offset="1" stop-color="#ddf3ae"/>
  </radialGradient>`;

const coloredMark = `
  <g fill="url(#ribbon)">
    <path d="${ribbon}"/>
    <path d="${ribbon}" transform="rotate(180 64 48)"/>
  </g>
  <g fill="none" stroke="url(#ribbon-edge)" stroke-width=".8" stroke-linecap="round">
    <path d="M5 52C21 48 30 18 51 9C79-2 100 19 123 46"/>
    <path d="M5 52C21 48 30 18 51 9C79-2 100 19 123 46" transform="rotate(180 64 48)"/>
  </g>
  <path d="${spark}" fill="url(#spark)"/>`;

export function markSvg(appearance = 'dark') {
  // The unframed mark needs darker pigment on a light interface; app tiles keep
  // the reference's pale green ribbons against their dark emerald background.
  const palette =
    appearance === 'light'
      ? gradients
          .replaceAll('#76b67a', '#2c7951')
          .replaceAll('#e4f6b2', '#518c3c')
          .replaceAll('#c6e99c', '#568c4b')
          .replaceAll('#91c887', '#367650')
          .replaceAll('#285d48', '#163e30')
          .replaceAll('#ffffed', '#31573b')
          .replaceAll('#ddf3ae', '#284a34')
      : gradients;
  return root(
    '0 0 128 96',
    `<defs>${palette}</defs>${coloredMark}`,
    'width="128" height="96"',
  );
}

export function templateSvg() {
  // Strong silhouette, generous aperture, and no gradients at menu-bar scale.
  return root(
    '0 0 128 96',
    `<g fill="#000"><path d="${ribbon}"/><path d="${ribbon}" transform="rotate(180 64 48)"/><path d="${spark}"/></g>`,
    'width="48" height="36"',
  );
}

export function outlineSvg() {
  // UI-sized outline variant, retaining the central four-point focus spark.
  return root(
    '0 0 24 24',
    '<g id="eye-focus"><path d="M2 12S5.7 5.5 12 5.5 22 12 22 12s-3.7 6.5-10 6.5S2 12 2 12Z" fill="none" stroke="currentColor" stroke-width="1.45" stroke-linejoin="round"/><path d="M12 7.8c.5 2.7 1.5 3.7 4.2 4.2-2.7.5-3.7 1.5-4.2 4.2-.5-2.7-1.5-3.7-4.2-4.2 2.7-.5 3.7-1.5 4.2-4.2Z" fill="currentColor"/></g>',
    'width="24" height="24"',
  );
}

export function appIconSvg() {
  return root(
    '0 0 160 160',
    `
    <defs>${gradients}
      <linearGradient id="tile" x1="17" y1="10" x2="143" y2="155" gradientUnits="userSpaceOnUse">
        <stop stop-color="#294d3d"/><stop offset=".45" stop-color="#142923"/><stop offset=".75" stop-color="#101e1c"/><stop offset="1" stop-color="#253d2b"/>
      </linearGradient>
      <radialGradient id="tile-glow" cx=".13" cy=".03" r="1">
        <stop stop-color="#c2f0ac" stop-opacity=".4"/><stop offset=".43" stop-color="#70b388" stop-opacity=".06"/><stop offset="1" stop-color="#13312b" stop-opacity="0"/>
      </radialGradient>
      <linearGradient id="tile-edge" x1="30" y1="8" x2="128" y2="155" gradientUnits="userSpaceOnUse">
        <stop stop-color="#b8e4ae"/><stop offset=".35" stop-color="#7ba487" stop-opacity=".35"/><stop offset=".7" stop-color="#355b4c" stop-opacity=".45"/><stop offset="1" stop-color="#b0c794" stop-opacity=".7"/>
      </linearGradient>
    </defs>
    <rect x="8" y="8" width="144" height="144" rx="36" fill="url(#tile)"/>
    <rect x="8" y="8" width="144" height="144" rx="36" fill="url(#tile-glow)"/>
    <rect x="8.5" y="8.5" width="143" height="143" rx="35.5" fill="none" stroke="url(#tile-edge)" stroke-width="1"/>
    <rect x="10" y="10" width="140" height="140" rx="34" fill="none" stroke="#f1ffe2" stroke-opacity=".035"/>
    <g transform="translate(16 33)">${coloredMark}</g>`,
    'width="160" height="160"',
  );
}
