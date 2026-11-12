# Native macOS checks

Automated browser checks cannot prove AppKit audio, menu-bar placement, launch at login, or behavior during sleep.
Record the macOS version, CPU architecture, app version, and build type with each native test run.

## Prepare

1. Run `npm run app:dev` or open a local app bundle.
2. Open settings.
3. Set work to 10 seconds.
4. Set the eye break to 5 seconds.
5. Set snooze to 8 seconds.
6. Set the idle delay to 12 seconds.
7. Save settings.

## Countdown and alarms

Before timer selection, make sure that the idle menu-bar item pulses gently over a three-second cycle.
After timer selection, make sure that it returns to full, steady opacity.

1. Start the work timer.
2. Hide the controls.
3. Make sure that the menu-bar countdown continues.
4. At completion, make sure that the controls appear and the sound repeats.
5. Wait through at least three sound repeats.
6. Hide the controls.
7. Make sure that the sound continues.
8. Open the controls.
9. Select **Start eye break**.
10. Make sure that the sound stops immediately.
11. At break completion, select **Back to work**.

The completion criterion is one uninterrupted cycle with both completion alarms and no countdown freeze while hidden.

## Snooze and idle

1. At a finished alarm, select snooze.
2. Make sure that the sound stops.
3. At snooze completion, make sure that the same alarm returns.
4. Make sure that snooze does not add another completed break.
5. Accept a finished timer without a new timer.
6. At the idle deadline, make sure that an idle alarm appears.
7. Make sure that the idle alarm has no dismiss action.
8. Snooze the idle alarm.
9. At snooze completion, make sure that the idle alarm returns.
10. Select either timer.
11. Make sure that the idle alarm stops.

## Pause and settings

1. Start a timer.
2. Pause it.
3. Wait longer than the idle delay.
4. Make sure that no reminder sounds.
5. Resume the timer.
6. Make sure that it retains its remaining time.
7. During a running timer, change its duration.
8. Make sure that its current deadline does not change.
9. Start that timer again.
10. Make sure that it uses the new duration.
11. Enable **Continue the cycle**.
12. At a finished alarm, select **Accept & start next timer**.
13. Make sure that the opposite timer starts.

## Automatic window hiding

1. Enable **Hide window after starting a timer**.
2. Save settings.
3. Make sure that saving leaves the controls open.
4. Start the work timer.
5. Make sure that the controls hide and the menu-bar countdown continues.
6. At completion, make sure that the reminder reopens the controls.
7. Select **Start eye break**.
8. Make sure that the controls hide again.
9. At break completion, select **Back to work**.
10. Make sure that the controls hide and screen fading clears as before.
11. Repeat timer selection through each preset, keyboard shortcut, and native tray action.
12. Enable **Continue the cycle**.
13. At completion, select **Accept & start next timer**.
14. Make sure that the next timer starts and the controls hide.
15. Reopen the controls from the menu bar.
16. Pause and resume the timer.
17. Make sure that the controls stay open.
18. Disable automatic window hiding.
19. Save settings.
20. Make sure that later timer selections keep the controls open.

## Sleep and system clock

1. Start a short timer.
2. Put the Mac to sleep before completion.
3. Wake the Mac after the deadline.
4. Make sure that one finished alarm appears.
5. Make sure that the app does not create several missed cycles.

If a wall-clock change is safe on the test Mac, change the wall clock during a timer.
The countdown must retain its original monotonic deadline. Restore the system clock after this check.

## Audio

1. Try each sound from settings.
2. Make sure that the volume control affects playback.
3. Make sure that an alarm stops on acceptance, timer selection, and snooze.
4. During an alarm, change the audio output.
5. Make sure that playback continues or a visible audio error appears.
6. Quit the app.
7. Make sure that all audio stops.

System mute and a disconnected output can prevent sound. A visible alarm must remain available in these conditions.

## Idle attention

1. Leave the app idle before its alarm deadline.
2. Make sure that the menu-bar item pulses without repeated flashes.
3. Start a timer.
4. Make sure that the pulse stops.
5. Stop the timer.
6. Make sure that the pulse returns during the idle wait.
7. Enable macOS Reduce Motion.
8. Make sure that the menu-bar item remains steady.
9. Restore the previous Reduce Motion setting.

Pause, snooze, and an active alarm must also stop the pulse.

## Screen fading

For a nonvisual constructor check:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --example check_overlays
```

This check creates hidden, transparent overlays on the main thread. It checks their click-through flags, layers, and window level.
It does not display an overlay or change the visible screen.

For the visible behavior checks:

1. Set work to 10 seconds and the eye break to 5 seconds.
2. Make sure that screen fading is off initially.
3. Enable **Dim screens after work**.
4. Set the fade duration to 10 seconds.
5. Save settings.
6. Start work.
7. At work completion, make sure that every display starts to dim gradually.
8. Make sure that Kroneum and the menu bar remain usable above the overlays.
9. Snooze the work alarm.
10. Make sure that fading continues without restarting.
11. Start the eye break.
12. Make sure that the screens remain dimmed during the break.
13. At break completion, make sure that all screens restore without acceptance or a fade-back animation.
14. Repeat the cycle with a different fade duration.
15. During fading, select **Restore screens** from the panel or native menu.
16. Make sure that the timer retains its state and the screens restore immediately.
17. Repeat this check with `Escape` and `⌘⇧R` while Kroneum has focus.
18. During another fade, disable the feature and save settings.
19. Make sure that all screens restore.
20. During another fade, quit Kroneum.
21. Make sure that no dimming remains.

For display changes, connect or disconnect a display during a fade. The overlay frames must match the remaining displays.
For full-screen apps and Spaces, make sure that the overlays remain click-through and the native menu remains accessible.
These checks change the visible screen temporarily. They do not change hardware brightness or gamma.

During fading, start work instead of completing the eye break. All screens must restore immediately.
Save settings without changes during that work timer. Fading must stay off until the new work timer finishes.
Repeat this check from a snoozed work alarm and an unfinished eye break.

For always-dim mode and the maximum level:

1. Disable **Dim screens after work**.
2. Enable **Always dim during eye breaks**.
3. Set **Maximum dimming** to 40%.
4. Save settings.
5. Start an eye break without a work timer.
6. Make sure that fading starts and clears at break completion.
7. Repeat with maximum values of 1%, 50%, and 100%.
8. At 100%, make sure that Kroneum and the menu bar remain accessible.
9. During a fade, change the maximum and save settings.
10. Make sure that the new limit applies without restarting the break timer.
11. Stop a manual break.
12. Make sure that its fade clears.
13. Enable both modes.
14. During a running break, disable only after-work mode.
15. Make sure that always-break mode retains the fade.
16. Disable always-break mode too.
17. Make sure that all screens restore.

## Menu bar and multiple displays

For logo changes, complete the native review in [branding.md](branding.md).

1. Open controls from the menu-bar item.
2. Make sure that the panel remains within the display edges.
3. Repeat this check on each connected display.
4. Repeat this check with different display scale factors.
5. Right-click the menu-bar item.
6. Use each enabled native action.
7. Make sure that snooze and acceptance are disabled outside an applicable alarm.
8. Open a second instance.
9. Make sure that the existing app opens without another menu-bar item.

For panel dragging:

1. Drag the header by its name, icon, or empty space.
2. Make sure that the panel moves on the current display.
3. Drag the panel across displays with different scale factors.
4. Make sure that the settings and hide buttons still respond to clicks.
5. Hide the panel.
6. Open it from the menu bar on another display.
7. Make sure that the whole panel fits inside that display's visible frame.

This check covers the placement regression reported on November 11, 2026.
The earlier implementation mixed the tray's scaled pixel coordinates with the panel's scale factor.

For a read-only native geometry check, run `swift scripts/check-panel.swift <Kroneum process ID>` with the panel visible.
The script compares that process's panel bounds with the visible screen frames. It does not read other window titles or change windows.

macOS can hide menu-bar items behind a camera notch or when the bar is full.
The app cannot override menu-bar space limits.

## Login and accessibility

1. Install the app in Applications.
2. Enable launch at login.
3. Sign out of macOS.
4. Sign in to macOS.
5. Make sure that Kroneum starts in the menu bar without its controls open.
6. Disable launch at login.
7. Make sure that a later login does not start Kroneum.
8. Use Tab, Shift-Tab, Enter, Escape, and the documented shortcuts.
9. Use VoiceOver to read settings and respond to an alarm.
10. Make sure that VoiceOver does not announce every countdown second.
11. Change the system appearance while the app is open.
12. Make sure that the interface and menu-bar icon retain readable contrast.

## Restore

1. Open settings.
2. Select **Use default settings**.
3. Save settings.
4. Quit the app.
