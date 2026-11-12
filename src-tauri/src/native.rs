//! Main-thread AppKit audio, menu-bar attention, and native panel placement.

#[cfg(target_os = "macos")]
mod platform {
    use crate::engine::Sound;
    use objc2::{rc::Retained, AnyThread, MainThreadMarker};
    use objc2_app_kit::{NSFont, NSSound, NSWindow, NSWorkspace};
    use objc2_foundation::{NSData, NSNumber, NSPoint, NSRect, NSSize, NSString};
    use objc2_quartz_core::{CABasicAnimation, CAMediaTiming, CAMediaTimingFunction};
    use std::cell::{Cell, RefCell};
    use tauri::Manager;

    struct Alarm {
        sound: Retained<NSSound>,
        kind: Sound,
    }

    thread_local! {
        static ALARM: RefCell<Option<Alarm>> = const { RefCell::new(None) };
        static PREVIEW: RefCell<Option<Retained<NSSound>>> = const { RefCell::new(None) };
        static PULSING: Cell<bool> = const { Cell::new(false) };
    }

    fn create(kind: Sound) -> Result<Retained<NSSound>, String> {
        let sound = match kind {
            Sound::Chime => {
                let data = NSData::with_bytes(include_bytes!("../sounds/chime.wav"));
                NSSound::initWithData(NSSound::alloc(), &data)
            }
            Sound::Glass | Sound::Ping => {
                let name = if kind == Sound::Glass {
                    "Glass"
                } else {
                    "Ping"
                };
                let path = NSString::from_str(&format!("/System/Library/Sounds/{name}.aiff"));
                NSSound::initWithContentsOfFile_byReference(NSSound::alloc(), &path, false)
            }
        };
        sound.ok_or_else(|| "macOS could not load the reminder sound.".into())
    }

    pub fn configure_tray(tray: &tauri::tray::TrayIcon) -> Result<(), String> {
        tray.with_inner_tray_icon(|inner| {
            let mtm = MainThreadMarker::new().ok_or("Tray setup requires the main thread.")?;
            let button = inner
                .ns_status_item()
                .and_then(|item| item.button(mtm))
                .ok_or("The macOS status button is unavailable.")?;
            // Regular system weight is zero. Fixed-width digits prevent title jitter.
            let font = NSFont::monospacedDigitSystemFontOfSize_weight(12.0, 0.0);
            button.setFont(Some(&font));
            Ok(())
        })
        .map_err(|e| e.to_string())?
    }

    fn idle_animation() -> Retained<CABasicAnimation> {
        let animation =
            CABasicAnimation::animationWithKeyPath(Some(&NSString::from_str("opacity")));
        let full = NSNumber::numberWithFloat(1.0);
        let dimmed = NSNumber::numberWithFloat(0.5);
        // SAFETY: The opacity key path accepts NSNumber values in the 0..=1 range.
        unsafe {
            animation.setFromValue(Some(&full));
            animation.setToValue(Some(&dimmed));
        }
        animation.setDuration(1.5);
        animation.setAutoreverses(true);
        animation.setRepeatCount(f32::INFINITY);
        let easing = CAMediaTimingFunction::functionWithName(&NSString::from_str("easeInEaseOut"));
        animation.setTimingFunction(Some(&easing));
        animation
    }

    pub fn sync_idle_pulse(
        tray: &tauri::tray::TrayIcon,
        phase: &'static str,
    ) -> Result<(), String> {
        tray.with_inner_tray_icon(move |inner| {
            let mtm = MainThreadMarker::new().ok_or("Idle attention requires the main thread.")?;
            let reduce_motion =
                NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion();
            let active = crate::attention::should_pulse(phase, reduce_motion);
            if PULSING.with(|state| state.get()) == active {
                return Ok(());
            }
            let button = inner
                .ns_status_item()
                .and_then(|item| item.button(mtm))
                .ok_or("The macOS status button is unavailable.")?;
            button.setWantsLayer(true);
            let layer = button
                .layer()
                .ok_or("The macOS status layer is unavailable.")?;
            let key = NSString::from_str("kroneum-idle-pulse");
            if active {
                // Core Animation handles every frame. No JavaScript or extra Rust timer.
                layer.addAnimation_forKey(&idle_animation(), Some(&key));
            } else {
                layer.removeAnimationForKey(&key);
                layer.setOpacity(1.0);
            }
            PULSING.with(|state| state.set(active));
            Ok(())
        })
        .map_err(|e| e.to_string())?
    }

    pub fn position_panel(
        window: &tauri::WebviewWindow,
        tray: &tauri::tray::TrayIcon,
    ) -> Result<(), String> {
        let window = window.clone();
        tray.with_inner_tray_icon(move |inner| {
            let mtm = MainThreadMarker::new().ok_or("Panel placement requires the main thread.")?;
            let status_window = inner
                .ns_status_item()
                .and_then(|item| item.button(mtm))
                .and_then(|button| button.window())
                .ok_or("The macOS status window is unavailable.")?;
            let screen = status_window
                .screen()
                .ok_or("The tray screen is unavailable.")?;
            let ptr = window
                .ns_window()
                .map_err(|e| e.to_string())?
                .cast::<NSWindow>();
            // SAFETY: Tauri supplies the NSWindow pointer. The cloned WebviewWindow
            // keeps it alive for this closure, which runs on the AppKit main thread.
            let panel = unsafe { ptr.as_ref() }.ok_or("The macOS panel is unavailable.")?;
            let configured = window
                .app_handle()
                .config()
                .app
                .windows
                .iter()
                .find(|w| w.label == window.label());
            let frame = panel.frame();
            let desired = configured.map_or((frame.size.width, frame.size.height), |w| {
                (w.width, w.height)
            });
            let rect = |r: NSRect| crate::placement::Rect {
                x: r.origin.x,
                y: r.origin.y,
                width: r.size.width,
                height: r.size.height,
            };
            // Use native points end-to-end. tray-icon's exported Rect is scaled
            // using the *tray's* backing scale, not the webview's backing scale.
            let result = crate::placement::below_tray(
                rect(status_window.frame()),
                rect(screen.visibleFrame()),
                desired,
            );
            panel.setFrame_display(
                NSRect::new(
                    NSPoint::new(result.x, result.y),
                    NSSize::new(result.width, result.height),
                ),
                false,
            );
            Ok(())
        })
        .map_err(|e| e.to_string())?
    }

    pub fn alarm(active: bool, kind: Sound, volume: f32) -> Result<(), String> {
        ALARM.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot
                .as_ref()
                .is_some_and(|alarm| !active || alarm.kind != kind)
            {
                if let Some(old) = slot.take() {
                    old.sound.stop();
                }
            }
            if !active {
                return Ok(());
            }
            if slot.is_none() {
                stop_preview();
                let sound = create(kind)?;
                sound.setVolume(volume);
                sound.setLoops(true);
                if !sound.play() {
                    return Err(
                        "macOS could not play the reminder sound. Check the audio output.".into(),
                    );
                }
                *slot = Some(Alarm { sound, kind });
            } else if let Some(alarm) = slot.as_ref() {
                alarm.sound.setVolume(volume);
                // Audio output changes can interrupt an NSSound loop.
                if !alarm.sound.isPlaying() && !alarm.sound.play() {
                    return Err("The reminder sound stopped. Check the audio output.".into());
                }
            }
            Ok(())
        })
    }

    pub fn preview(kind: Sound, volume: f32) -> Result<(), String> {
        if ALARM.with(|cell| cell.borrow().is_some()) {
            return Err("Respond to the current reminder before you try a sound.".into());
        }
        stop_preview();
        let sound = create(kind)?;
        sound.setVolume(volume);
        sound.setLoops(false);
        if !sound.play() {
            return Err("macOS could not play the sound. Check the audio output.".into());
        }
        PREVIEW.with(|cell| *cell.borrow_mut() = Some(sound));
        Ok(())
    }

    pub fn stop_preview() {
        PREVIEW.with(|cell| {
            if let Some(sound) = cell.borrow_mut().take() {
                sound.stop();
            }
        });
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn bundled_and_system_sounds_load_without_playback() {
            for kind in [Sound::Chime, Sound::Glass, Sound::Ping] {
                let sound = create(kind).expect("native reminder sound must load");
                assert!(sound.duration() > 0.0);
                assert!(!sound.isPlaying());
                if kind == Sound::Chime {
                    assert!((sound.duration() - 4.0).abs() < 0.01);
                }
            }
        }

        #[test]
        fn idle_pulse_is_a_slow_reversing_native_animation() {
            let animation = idle_animation();
            assert_eq!(animation.keyPath().unwrap().to_string(), "opacity");
            assert_eq!(animation.duration(), 1.5);
            assert!(animation.autoreverses());
            assert!(animation.repeatCount().is_infinite());
        }

        #[test]
        fn menu_bar_artwork_is_a_retina_monochrome_template() {
            let image =
                tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png")).unwrap();
            assert_eq!((image.width(), image.height()), (48, 36));
            assert!(image
                .rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[..3] == [0, 0, 0]));
            assert!(image
                .rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] == 255));
            assert_eq!(image.rgba()[3], 0);
        }

        #[test]
        fn bundled_app_artwork_has_a_full_resolution_master() {
            let image =
                tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap();
            assert_eq!((image.width(), image.height()), (1024, 1024));
            assert_eq!(image.rgba()[3], 0);
        }
    }
}

#[cfg(target_os = "macos")]
pub use platform::*;

#[cfg(not(target_os = "macos"))]
pub fn alarm(_: bool, _: crate::engine::Sound, _: f32) -> Result<(), String> {
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn preview(_: crate::engine::Sound, _: f32) -> Result<(), String> {
    Err("Native audio requires macOS.".into())
}
#[cfg(not(target_os = "macos"))]
pub fn stop_preview() {}

#[cfg(not(target_os = "macos"))]
pub fn configure_tray(_: &tauri::tray::TrayIcon) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn position_panel(
    window: &tauri::WebviewWindow,
    _: &tauri::tray::TrayIcon,
) -> Result<(), String> {
    window.center().map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn sync_idle_pulse(_: &tauri::tray::TrayIcon, _: &'static str) -> Result<(), String> {
    Ok(())
}
