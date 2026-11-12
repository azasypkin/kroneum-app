# Architecture and behavior

## Ownership

`Engine` owns the timer state. Its methods receive monotonic milliseconds from the host.
The engine has no AppKit calls, file access, threads, or browser dependencies.

The Tauri host owns one engine behind a mutex. Commands serialize changes through this mutex.
The host schedules periodic main-thread updates once per second. State changes and nearby timer deadlines wake the clock early.
Each update reads the current state, updates native controls, and manages audio and overlays.

The host emits snapshots only while the window is visible or a new alarm requires the window.
The interface requests a fresh snapshot at startup and on focus. Browser suspension cannot stop a production timer.

AppKit audio objects stay in main-thread storage. Each alarm uses one looping `NSSound` instance.
The generated soft chime includes silence between repeats. Glass and Ping use local macOS sound files.

Idle attention uses one native Core Animation opacity animation on the status button.
It fades between full and half opacity over a three-second cycle. Core Animation handles the frames without another application timer.
The host starts the pulse only in the idle state. Running, paused, snoozed, and alarm states retain a static menu-bar item.
The host reads the macOS Reduce Motion preference during its existing update. A preference change stops or starts the pulse.

## Screen fading

`Dimmer` tracks an overdue break and an active break independently of alarm acceptance. Both fading modes are off by default.
After-work mode fades when work finishes. Always-break mode also fades every manually started break, without a previous work completion.
The configured maximum is 1% through 100%, with an 85% default. Fade duration defaults to 30 seconds.
Either mode can keep an existing fade active. Changing triggers does not restart its clock.
Break completion clears both causes and the dimming plan. Starting work also clears both causes, so settings saves cannot rearm dimming during work.
Stopping a break clears its active-break cause. An overdue-break cause can keep fading active under after-work mode.
Pause retains the active break and its fade. Snooze and dismissal retain an overdue-break fade.
Manual restoration suppresses the current fade. A new work completion or explicitly started always-dim break can create another fade.
Only enabling a trigger that matches a current cause can override that suppression. Unrelated settings changes leave restoration intact.

`screen_fade.rs` owns one borderless, click-through AppKit overlay per display. Each overlay has a black Core Animation layer.
The menu bar and the Kroneum panel remain above the overlays. These windows do not change display brightness or gamma.
Quit clears all overlay windows. Process termination removes them through normal macOS window ownership.

The native fade uses elapsed time from the engine's monotonic plan. The compositor animates opacity without application frame timers.
A scheduled native opacity reset restores screens at the running break's deadline, even if the application thread is briefly busy.
The clock also wakes at the exact deadline to clear the engine plan and remove the overlays.
Pause cancels that reset deadline. Resume supplies a new deadline without restarting the fade.
After sleep, a late work completion starts a fresh fade rather than an abrupt jump to full dimming.

Display changes recreate overlays using native screen frames. Existing fade durations remain fixed after settings changes.
Saving a new maximum updates the current plan's limit while preserving its start time. The native renderer uses that plan's limit, not a constant.
Native overlay errors restore the screens and suppress the current fade. The interface displays the error.

## State transitions

| State             | Trigger                       | Result                                         |
| ----------------- | ----------------------------- | ---------------------------------------------- |
| Idle              | Idle deadline                 | Persistent idle alarm                          |
| Any               | Start work or eye break       | Running timer, current alarm cleared           |
| Running           | Timer deadline                | Persistent finished alarm                      |
| Running           | Pause before deadline         | Paused timer with exact remaining milliseconds |
| Paused            | Resume                        | Running timer with a new deadline              |
| Running or paused | Stop                          | Idle with a new idle deadline                  |
| Finished alarm    | Accept, manual cycle          | Idle with a new idle deadline                  |
| Finished alarm    | Accept, automatic cycle       | Opposite timer starts                          |
| Any alarm         | Snooze                        | Snoozed original alarm                         |
| Snoozed           | Snooze deadline or end snooze | Original alarm returns                         |
| Idle alarm        | Accept                        | Command error, alarm remains                   |

An eye break increments the session count at its first completion. Snooze does not increment this count.
Timer selection replaces the current timer. Preset selection also restarts a matching active timer.

Pause is an explicit reminder suspension. Paused time does not count toward the idle reminder.
The idle clock measures time without a timer, not user input or system inactivity.

Automatic cycling requires acceptance at each timer boundary. It does not start a break before the user responds.
The primary completion action always starts the next timer. The separate acceptance action obeys the cycle setting.

## Clock and sleep

macOS uses `mach_continuous_time` with the system timebase. This monotonic clock includes system sleep.
A change to the wall clock does not change a deadline. The engine calculates remaining time from deadlines, not tick counts.

The scheduler controls display updates and alarm delivery, not elapsed time.
It wakes at deadlines. Operating-system and main-thread scheduling delays can still delay alarm delivery.
Sleep can delay delivery until wake. A late update creates one alarm, not a chain of missed cycles.

The non-macOS clock exists for development builds only. Native sound and installable bundles target macOS.

## Settings and storage

The host checks durations and volume before storage. Each duration must contain at least one second.
Work permits up to four hours. Break, snooze, and idle durations permit up to one hour.
Volume permits 5% through 100%. System mute still prevents audible output.

The versioned settings file contains one settings object. Storage writes a temporary file, flushes it, then renames it over the destination.
The host changes in-memory settings only after storage succeeds. An unreadable or unsupported file remains intact.
The app displays a warning and uses defaults until an explicit save replaces that file.

Active timer and snooze deadlines keep their original durations after settings changes.
The idle threshold uses the latest settings against the existing idle start time. A shorter threshold can trigger an immediate idle alarm.

Launch at login uses the Tauri autostart plugin with a macOS LaunchAgent. Its state stays separate from the settings file.
The host reads the LaunchAgent state once at startup and updates its cached state after an explicit change.

## Interface and security

The interface uses TypeScript and native HTML controls. CSS supports Safari 15, system appearance, and reduced motion.
The countdown has `role="timer"` with live announcements off. Timer states, errors, and save results use polite status regions.
The interface changes status text only when its content changes, so each tick does not repeat an announcement.
An alarm changes the visible content and moves focus to the primary response.
Completion actions keep separate hit areas and hover backgrounds. Short panels scroll rather than overlap the action controls.
The main scroll area mirrors reserved scrollbar space on both sides, so classic scrollbars do not shift the content off center.
Overlay scrollbars retain the same center line without reserved space. Older WebViews use their native scrollbar behavior.
Timer captions wrap within the lower interior of the circle, rather than its full diameter. Long captions remain complete and unclipped.
Balanced wrapping is a progressive enhancement. Older WKWebView versions use ordinary wrapping within the same width constraint.
The header, countdown, and eye-break control share vector brand artwork. Native app and tray icons derive from the same geometry.
For asset regeneration or icon variants, read [branding.md](branding.md).

Tauri capabilities grant event subscription and native header dragging to the local window. App commands handle timer actions and constrained settings.
The interface has no filesystem, shell, notification, or external URL permissions. The content policy restricts assets to local sources.
Development permits only the local Vite server and IPC endpoints.

`preview.ts` loads only outside Tauri or for the default-settings form helper. Its scheduler is a browser-only simulator.
Production time never uses JavaScript intervals. Preview fixtures never alter native behavior.

## Window lifecycle

The app uses accessory activation and `LSUIElement`. It has no normal Dock icon.
The native status button uses system typography with fixed-width digits to prevent countdown width changes.
Left-click toggles the controls near the menu-bar item. Right-click opens the native action menu.
Placement reads the native status window and its screen's visible frame in AppKit points.
The panel fits inside that frame, including on displays with different backing scales or negative origins.
Placement uses native window frames rather than the tray library's exported pixel coordinates.
Small visible frames reduce the panel size. The interface can scroll to keep all controls available.
The header uses Tauri's native drag region. Its settings and hide buttons remain outside that region.
Dragging moves the open panel. Reopening the panel or a new alarm positions it near the menu-bar item again.
The window stays open after an alarm until the user responds or explicitly hides it.
Focus loss hides the window outside an alarm. Hiding leaves the engine and sound active.

The optional `hideOnTimerStart` setting is off by default, including for older settings files.
Successful actions report whether they started a timer. The native host uses this effect after the state and native controls update.
Work and break selection, presets, shortcuts, tray actions, and cycle acceptance share this path.
Resume is not a new timer start. Settings saves, failed actions, and periodic clock updates have no start effect.
Before hiding, the host makes sure that the latest state is still running. A delayed effect cannot hide a new completion alarm.
Auto-hide does not change the engine, fade, or deadline. Completion alarms still show the window.
The browser preview exercises the same hide-request policy, but its hide method keeps the browser page visible.

The single-instance plugin opens the existing controls instead of creating duplicate timers.
Login starts include `--background`. Other starts display the controls.
An app restart begins idle rather than restoring a potentially stale alarm or timer.
