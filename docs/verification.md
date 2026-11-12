# Verification record

This record covers version 0.1.0, updated on November 12, 2026.
The build host uses macOS 26.7.3 on Apple Silicon, Rust 1.99.1, and Node.js 24.21.0.

## Automated checks

| Check                                                         | Result                                    |
| ------------------------------------------------------------- | ----------------------------------------- |
| TypeScript and Prettier                                       | Passed                                    |
| Presentation, branding, and window capability tests           | 38 passed                                 |
| Rust engine, storage, placement, attention, and native checks | 64 passed                                 |
| WebKit and Chromium interface checks                          | 124 passed                                |
| Light/dark accessibility checks                               | Passed within the interface suite         |
| Alarm focus and layout fit                                    | Passed within the interface suite         |
| Rust formatting                                               | Passed                                    |
| Clippy with warnings treated as errors                        | Passed                                    |
| Apple Silicon app and disk image build                        | Passed                                    |
| Packaged executable startup                                   | Passed, visible launch and geometry check |

The Rust sound test loads all three sounds without playback. It does not prove audible output or alarm volume.
The rebuilt executable opened its panel with no stdout or stderr errors.
The native geometry check passed for a 392-by-684-point panel on one screen of a two-display Mac.
This check inspected panel bounds, not mouse dragging or animation frames.
The sleep test supplies a late clock value to the engine. It does not suspend the physical Mac.
Placement tests cover panel edge calculations, not physical displays with different scale factors.

The placement correction uses the native tray window and its screen's visible frame, entirely in AppKit points.
New regression tests cover secondary and negative screen origins, short displays, and Dock boundaries.
The header has a native drag region with its controls excluded. A capability test covers the required drag permission.
Browser tests cover the capitalized name, header controls, and the higher countdown position.
The idle pulse tests cover state selection, Reduce Motion, and native animation timing.
They do not measure the rendered menu-bar pulse.

Screen fade tests cover opt-in settings, legacy settings, fade limits, pending breaks, snooze, pause, manual restoration, and break deadlines.
Native animation tests cover the finite fade and persistent deadline reset. Browser checks cover the new settings and restoration controls.
The nonvisual native overlay check passed on both displays. The windows remained hidden and transparent, with click-through flags and native layers intact.
The rendered fade, full-screen Spaces behavior, and visible restoration still need the interactive procedure.
Work-restart regression checks cover restoration from a snoozed alarm and an unfinished eye break.
They also cover the old break deadline and settings saves, so neither can rearm a cleared fade during work.
Maximum-level and always-break checks cover all trigger combinations, manual breaks, live settings changes, keyboard slider controls, and saved settings.
Legacy settings retain the 85% limit and leave the new always-break mode off.

Branding checks cover shared vector geometry, transparent icon corners, the tray aperture and spark, and deterministic native pixel sizes.
Browser checks cover light/dark artwork at standard and Retina densities, decorative image semantics, and the eye-break outline.
The regenerated screenshots received a visual review. Physical Finder caching and menu-bar readability remain native review items.
Auto-hide checks cover saved settings, timer-start effects, cycle acceptance, and the guard against hiding a later completion alarm.
Preview checks observe the hide request without hiding the browser page. Actual native hide/reopen behavior requires the interactive procedure.
Completion-action geometry checks cover 340-, 357-, and 392-pixel panels in both browser engines, including Snooze hover and cycle acceptance.
The checks cover the visible button gaps, acceptance target height, and absence of a button hit area in each gap.
Updated light/dark completion screenshots received a visual review.
Caption regression tests first reproduced text crossing the ring in snooze and completion states.
The corrected tests cover every caption at narrow and standard widths, with balanced and ordinary wrapping in both browser engines.
They check the actual text rectangles against the circle, preserve the font size, and exclude clipping as a workaround.
Horizontal alignment checks first reproduced a 7.5-pixel offset from a one-sided scrollbar gutter.
The corrected checks cover work, snooze, completion, and scrollable settings at narrow and standard widths in both browser engines.
Regenerated screenshots show equal outer spacing. GitHub rendering preserves the centered README screenshot table.

## Native checks that remain

These checks require an interactive test on the target Mac:

- Audible playback, repeated sound, immediate sound stop, and audio output changes.
- Menu-bar countdown, right-click actions, and panel placement on each physical display.
- Native header dragging and the idle pulse, including live Reduce Motion changes.
- Automatic window hiding and reopening after timer completion.
- Visible screen fading, restoration, and display changes during fading.
- Physical sleep and wake.
- Launch at login after an actual sign-out and sign-in.
- VoiceOver navigation and system appearance changes in WKWebView.
- Gatekeeper acceptance after signing and notarization.

The procedures are in [macos-testing.md](macos-testing.md). The signing procedure is in [releasing.md](releasing.md).
The local package has no Developer ID signature or notarization ticket.

## Agent-document review

`AGENTS.md` received a review with the writing-for-agents guidance.
The review covered scoped context pointers, grouped invariants, completion criteria, positive instructions, and duplicate instructions.
The documentation also received a plain-English review for sentence length and consistent names.
