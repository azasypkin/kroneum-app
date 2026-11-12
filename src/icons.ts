const paths = {
  work: '<rect x="4" y="4" width="16" height="12" rx="2"/><path d="M8 20h8m-4-4v4"/>',
  settings:
    '<path d="m9 3-.5 2-2 .9-1.9-.6-2 3.4 1.4 1.5v2.3L2.6 14l2 3.4 1.9-.6 2 .9L9 20h4l.5-2.3 2-.9 1.9.6 2-3.4-1.4-1.5v-2.3l1.4-1.5-2-3.4-1.9.6-2-.9L13 3Z"/><circle cx="11" cy="11.5" r="3"/>',
  close: '<path d="m6 6 12 12M18 6 6 18"/>',
  arrow: '<path d="M5 12h14m-5-5 5 5-5 5"/>',
  back: '<path d="m14 6-6 6 6 6"/>',
  sound:
    '<path d="m11 4-6 5H2v6h3l6 5V4Zm4 4a6 6 0 0 1 0 8m3-11a10 10 0 0 1 0 14"/>',
  leaf: '<path d="M20 3C8 2 2 8 5 15c3 7 14 4 15-12Z"/><path d="m4 21 11-12"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
};

export function icon(name: keyof typeof paths | 'eye', className = ''): string {
  if (name === 'eye') {
    return `<svg class="icon ${className}" viewBox="0 0 24 24" aria-hidden="true"><use href="/brand/outline.svg#eye-focus"/></svg>`;
  }
  return `<svg class="icon ${className}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths[name]}</svg>`;
}

export function logo(variant: 'app' | 'mark'): string {
  const file = variant === 'app' ? 'app-icon' : 'mark';
  const size = variant === 'app' ? [40, 40] : [44, 33];
  if (variant === 'mark') {
    return `<picture class="brand-picture"><source media="(prefers-color-scheme: dark)" srcset="/brand/mark.svg"/><img class="brand-art brand-art-mark" src="/brand/mark-light.svg" width="44" height="30" alt="" draggable="false" /></picture>`;
  }
  return `<img class="brand-art brand-art-${variant}" src="/brand/${file}.svg" width="${size[0]}" height="${size[1]}" alt="" draggable="false" />`;
}
