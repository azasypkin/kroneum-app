# Kroneum

Kroneum is a macOS menu-bar app for work timers and eye breaks. Rust owns time, reminders, audio, and saved settings.
TypeScript displays snapshots and sends commands. The browser preview is a UI simulator, not a production timer.

## Read by change

- For timer transitions, sleep behavior, or command changes, read [docs/architecture.md](docs/architecture.md).
- For AppKit, tray placement, or window changes, read [docs/macos-testing.md](docs/macos-testing.md).
- For logo or generated asset changes, read [docs/branding.md](docs/branding.md).
- For packaging or signing changes, read [docs/releasing.md](docs/releasing.md).

## Invariants

- Keep the production timer in `engine.rs`. Supply time through the host clock.
- Keep alarms active until a timer starts, the user accepts a finished timer, or the user snoozes.
- Require a timer choice or snooze for an idle alarm.
- Preserve the original alarm through snooze. Count a completed break once.
- Preserve active deadlines after settings changes. Apply new durations to the next start.
- Keep AppKit calls on the main thread. Use `run_on_main_thread` at thread boundaries.
- Save settings before you change the in-memory settings. Preserve unreadable files until an explicit save.
- Keep the Rust serialization and `src/types.ts` consistent.
- Keep screen fading opt-in and click-through. Keep the panel and menu bar accessible above the overlays.
- Restore screens at eye-break completion or Work start. Preserve manual restoration and restoration on quit.
- Hide the window only after a successful timer start. Keep a later completion alarm visible.

## Interface and performance

- Target WKWebView on macOS 12 or later. Provide CSS fallbacks for features absent from Safari 15.
- Use system fonts, semantic controls, visible focus, and the system color scheme.
- Keep countdown updates outside live regions. Announce state changes and errors, not each second.
- Respect reduced motion. Keep the idle interface static.
- Keep native actions usable through the tray menu without the webview.
- Keep runtime assets local. Use narrowly scoped Tauri capabilities for new system access.
- Use one periodic clock wake per second, with early wakes for state changes and timer deadlines.
- Update the tray title only when its text changes. Run fade frames through Core Animation rather than application timers.
- Keep JavaScript countdown scheduling in the preview module only.

## Documentation

Update the linked behavior document with each observable behavior change. Use consistent names and short sentences without em dashes.
Keep detailed platform checks in the native test document rather than this file.

## Completion

1. Add a regression test for each changed transition or boundary.
2. Run the applicable scripts from `package.json`.
3. For Rust changes, run `cargo fmt` and `cargo clippy` with the manifest in `src-tauri/Cargo.toml`.
4. Record native check results or untested behavior in `docs/verification.md`.
5. Report passed, failed, and skipped checks.

The change is complete when relevant automated checks pass and each affected native behavior has a recorded result or limitation.
Use Conventional Commits for commit messages.
