# Development

## Automated checks

From the project root, run:

```sh
npm run check
npm test
npm run test:rust
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

For browser checks, install the browser engines first:

```sh
npx playwright install chromium webkit
npm run test:ui
```

The browser suite covers layout, keyboard controls, settings, and accessibility in light and dark appearance.
AppKit sound, physical sleep, login behavior, and native window interaction require the [macOS procedure](macos-testing.md).
The [verification record](verification.md) separates automated results from native checks that remain.

## Browser preview

```sh
npm run dev
```

Open `http://127.0.0.1:1420`.
The preview simulates the interface without native sound, saved settings, a menu-bar timer, or screen overlays.
Automatic window hiding leaves the preview page visible. The footer identifies this mode.

The `preview` query parameter selects a visual state: `work`, `break`, `alert`, `idle-alert`, or `break-alert`.
The screenshot script also captures snooze through its normal controls:

```sh
npm run screenshots
```

## Code and assets

- `src-tauri/src/engine.rs` owns deterministic timer transitions.
- `src-tauri/src/lib.rs` connects commands, the clock, the menu bar, and window behavior.
- `clock.rs`, `native.rs`, and `screen_fade.rs` contain native platform code.
- `dimming.rs` owns the fading policy. `placement.rs` owns panel geometry.
- `persistence.rs` reads and writes settings.
- `src/` contains the typed interface and separate preview simulator.
- `tests/` contains presentation, asset, policy, and browser checks.

The [architecture guide](architecture.md) describes the state transitions and module boundaries.
The [brand asset guide](branding.md) describes the vector artwork and native icon generation.
The [release guide](releasing.md) covers universal builds, signing, and notarization.

Use Conventional Commits for commit messages, such as `fix(timer): preserve the snoozed reminder`.
