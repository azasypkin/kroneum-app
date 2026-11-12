# Brand assets

The mark combines an eye aperture, a four-point focus spark, and two curved ribbons that suggest a cycle.
The vector paths are a redraw of the supplied design reference. The reference image is not an exported logo file.

## Source and variants

`scripts/brand-artwork.mjs` owns the paths and gradients. The generator produces these local assets:

| Asset                         | Use                                                         |
| ----------------------------- | ----------------------------------------------------------- |
| `public/brand/app-icon.svg`   | The emerald app tile, header logo, and browser preview icon |
| `public/brand/mark.svg`       | The pale green mark on dark surfaces                        |
| `public/brand/mark-light.svg` | A darker pigment variant on light surfaces                  |
| `public/brand/template.svg`   | The monochrome menu-bar silhouette                          |
| `public/brand/outline.svg`    | The eye-break button's outline and focus spark              |
| `src-tauri/icons/icon.png`    | The 1024-pixel app master                                   |
| `src-tauri/icons/icon.icns`   | Native app icon sizes for macOS                             |
| `src-tauri/icons/tray.png`    | The 48-by-36-pixel menu-bar template for Retina displays    |
| `src-tauri/icons/tray@1x.png` | The 24-by-18-pixel menu-bar template for standard displays  |

The header and countdown use vector images, not resized PNG files. The light/dark mark changes through a native HTML `picture` source.
The outline uses same-origin SVG `use` and inherits the control's color. Decorative artwork has empty alternative text or `aria-hidden`.
Explicit dimensions reserve space before each image loads. Artwork ignores pointer events, so the header retains its drag behavior.

The menu-bar template uses an opaque black silhouette with transparent surroundings. macOS supplies its light or dark display color.
Gradients stay in the app and interface artwork. The template retains a clear aperture and spark at 18 points high.

## Regeneration

1. Change the canonical paths or palette in `scripts/brand-artwork.mjs`.
2. Run `npm run assets`.
3. Run `npm test`.
4. Run `npm run test:ui`.
5. Start the development server.
6. Run `npm run screenshots`.

The generator uses resvg as a development dependency. It rasterizes every native size directly from SVG, without enlarged intermediate bitmaps.
The runtime has no new image library or external image requests. The macOS `iconutil` command creates the final ICNS file.
Generated SVG files remain unchanged by Prettier, so regeneration stays deterministic.

The screenshot script creates light/dark interface captures and `docs/images/brand.png` for visual review.

## Native review

1. Open a freshly built app.
2. Read the menu-bar mark on light and dark wallpapers.
3. Repeat this check on standard and Retina displays.
4. Read the app icon in Finder at small and large sizes.
5. Make sure that the aperture and focus spark remain distinct.

An already installed app can retain an old icon in the Finder cache. Replace the old bundle before this review.
If the icon remains stale, quit the app and reopen the new bundle. A system cache refresh is outside the build process.
