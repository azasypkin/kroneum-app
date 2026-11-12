# User guide

## Timers and reminders

The default work timer lasts 20 minutes. The default eye break lasts 30 seconds.
Settings use seconds for exact durations. Changes apply to the next timer start, not an active deadline.

At completion, the sound repeats until you select a timer, accept the reminder, or snooze.
Selecting either preset starts that timer and stops the current alarm. Selecting the active preset restarts it.
Snooze delays the same reminder. It does not restart a timer or count another completed break.

After five minutes without an active timer, Kroneum sounds an idle reminder. Settings can change this delay.
The menu-bar item pulses during the idle wait, unless macOS Reduce Motion is enabled.
An idle reminder requires a timer choice or snooze. It has no dismiss action.

Pause suspends the timer and idle reminders. Resume retains the remaining time.
**Continue the cycle** makes acceptance start the opposite timer. Each completion still requires a response.

## Window and menu bar

Left-click the menu-bar item to open or hide the controls. Right-click it for native timer actions.
Drag the header to move the panel. Reopening it positions the panel near the menu-bar item again.
Closing the controls leaves the timer and reminders active. **Quit** stops the app and restores dimmed screens.

**Hide window after starting a timer** is off by default. It applies to timer buttons, presets, shortcuts, and native tray actions.
**Accept & start next timer** also hides the controls under this setting. Completion reminders reopen them.
Pause, resume, snooze, and settings saves do not automatically hide the controls.

## Screen dimming

Both dimming modes are off by default:

- **Dim screens after work** starts fading at work completion and continues through the eye break.
- **Always dim during eye breaks** also fades manually started breaks, without a previous work timer.

The modes work independently. The default fade takes 30 seconds to reach 85% dimming.
**Fade duration** permits 1 to 600 seconds. **Maximum dimming** permits 1% to 100%.
At 100%, desktop content is black. Kroneum and the menu bar remain accessible above the overlays.

Break completion restores the screens without acceptance or a fade-back animation. Starting Work also restores them immediately.
Stopping a manual break restores its fade, unless after-work mode still requires a break.
Snooze and dismissal leave an overdue-break fade active. Pause retains a break fade and postpones its restoration deadline.

**Restore screens** clears dimming without changing the timer or accepting its alarm.
Manual restoration suppresses the current fade. A later work completion or new break can start another fade under the enabled mode.
Disabling both modes or quitting restores all screens.

Fade duration changes apply to the next fade. Maximum-level changes apply on save, including during an active fade.
The overlays ignore mouse clicks. They do not change hardware brightness or gamma, and process termination removes them.

## Shortcuts

These shortcuts require focus in Kroneum:

| Shortcut | Action                                                   |
| -------- | -------------------------------------------------------- |
| `⌘1`     | Start or restart Work                                    |
| `⌘2`     | Start or restart the eye break                           |
| `⌘,`     | Open settings                                            |
| `Escape` | Restore screens during dimming, otherwise return or hide |
| `⌘⇧R`    | Restore screens without changing the timer               |
| `⌘Q`     | Quit                                                     |

## Privacy and limits

Settings stay in `~/Library/Application Support/app.kroneum.desktop/settings.json`.
The app saves settings, not active timers. Each restart begins idle. The break count covers the current session only.

The clock includes system sleep and ignores wall-clock changes. After wake, an expired timer produces one reminder.
The app does not wake a sleeping Mac. Sound requires an audible output and sufficient system volume.

Kroneum uses a native menu-bar item, not an Apple Clock timer or a Control Center extension.
It is not a medical device or a treatment for an eye condition. For persistent discomfort, consult an eye-care professional.
