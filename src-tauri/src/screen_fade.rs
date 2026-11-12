//! Native, click-through overlays. Hardware brightness and gamma remain unchanged.
#[cfg(target_os = "macos")]
mod platform {
    use crate::{clock, dimming::DimPlan};
    use objc2::{rc::Retained, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSBackingStoreType, NSColor, NSFloatingWindowLevel, NSModalPanelWindowLevel,
        NSPopUpMenuWindowLevel, NSScreen, NSView, NSWindow, NSWindowAnimationBehavior,
        NSWindowCollectionBehavior, NSWindowStyleMask,
    };
    use objc2_foundation::{NSNumber, NSPoint, NSRect, NSString};
    use objc2_quartz_core::{
        CABasicAnimation, CACurrentMediaTime, CALayer, CAMediaTiming, CAMediaTimingFunction,
        CATransaction,
    };
    use std::cell::RefCell;

    struct Overlay {
        window: Retained<NSWindow>,
        layer: Retained<CALayer>,
    }
    impl Drop for Overlay {
        fn drop(&mut self) {
            self.window.orderOut(None);
        }
    }

    #[derive(Default)]
    struct State {
        overlays: Vec<Overlay>,
        frames: Vec<NSRect>,
        plan: Option<DimPlan>,
        last_sync: u64,
    }

    thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

    fn panel_level(window: &tauri::WebviewWindow, active: bool) -> Result<(), String> {
        let ptr = window
            .ns_window()
            .map_err(|e| e.to_string())?
            .cast::<NSWindow>();
        // SAFETY: Tauri owns this NSWindow. The caller holds the WebviewWindow
        // and accesses its pointer only on the AppKit main thread.
        let panel = unsafe { ptr.as_ref() }.ok_or("The macOS panel is unavailable.")?;
        panel.setLevel(if active {
            NSPopUpMenuWindowLevel
        } else {
            NSFloatingWindowLevel
        });
        Ok(())
    }

    fn create_overlay(frame: NSRect, mtm: MainThreadMarker) -> Result<Overlay, String> {
        // SAFETY: The frame comes from NSScreen. All enum values are valid, the
        // allocation is fresh, and the window is created on the main thread.
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                frame,
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        window.setOpaque(false);
        window.setBackgroundColor(Some(&NSColor::clearColor()));
        window.setHasShadow(false);
        window.setIgnoresMouseEvents(true);
        window.setHidesOnDeactivate(false);
        // Menu-bar menus and Kroneum's panel remain above these windows.
        window.setLevel(NSModalPanelWindowLevel);
        window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
        window.setAnimationBehavior(NSWindowAnimationBehavior::None);
        let view = NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), frame.size),
        );
        view.setWantsLayer(true);
        let layer = view
            .layer()
            .ok_or("The screen overlay layer is unavailable.")?;
        layer.setBackgroundColor(Some(&NSColor::blackColor().CGColor()));
        layer.setOpacity(0.0);
        window.setContentView(Some(&view));
        Ok(Overlay { window, layer })
    }

    fn opacity_animation(from: f32, to: f32, seconds: f64) -> Retained<CABasicAnimation> {
        let animation =
            CABasicAnimation::animationWithKeyPath(Some(&NSString::from_str("opacity")));
        let from = NSNumber::numberWithFloat(from);
        let to = NSNumber::numberWithFloat(to);
        // SAFETY: NSNumber is the required value type for the opacity key path.
        unsafe {
            animation.setFromValue(Some(&from));
            animation.setToValue(Some(&to));
        }
        animation.setDuration(seconds);
        let timing = CAMediaTimingFunction::functionWithName(&NSString::from_str("linear"));
        animation.setTimingFunction(Some(&timing));
        animation
    }

    fn restore_animation(after_seconds: f64) -> Retained<CABasicAnimation> {
        let restore = opacity_animation(0.0, 0.0, 0.001);
        restore.setBeginTime(CACurrentMediaTime() + after_seconds);
        restore.setRemovedOnCompletion(false);
        restore.setFillMode(&NSString::from_str("forwards"));
        restore
    }

    fn apply_plan(overlay: &Overlay, plan: DimPlan) {
        // Window creation can take time. Never project a deadline from stale time.
        let now = clock::now_ms();
        if plan.clear_at_ms.is_some_and(|deadline| now >= deadline) {
            overlay.window.orderOut(None);
            return;
        }
        // Show at the current (initially zero) opacity before starting animation.
        overlay.window.orderFrontRegardless();
        let fade_key = NSString::from_str("kroneum-screen-fade");
        let restore_key = NSString::from_str("kroneum-screen-restore");
        overlay.layer.removeAnimationForKey(&fade_key);
        overlay.layer.removeAnimationForKey(&restore_key);
        let remaining = plan.fade_remaining_ms(now);
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        overlay.layer.setOpacity(plan.max_opacity());
        if remaining > 0 {
            overlay.layer.addAnimation_forKey(
                &opacity_animation(
                    plan.opacity_at(now),
                    plan.max_opacity(),
                    remaining as f64 / 1_000.0,
                ),
                Some(&fade_key),
            );
        }
        if let Some(deadline) = plan.clear_at_ms {
            // The compositor restores opacity exactly at break completion even
            // if the application main thread is briefly busy at that instant.
            let restore = restore_animation(deadline.saturating_sub(now) as f64 / 1_000.0);
            overlay
                .layer
                .addAnimation_forKey(&restore, Some(&restore_key));
        }
        CATransaction::commit();
    }

    pub fn sync(window: &tauri::WebviewWindow, plan: Option<DimPlan>) -> Result<(), String> {
        let mtm = MainThreadMarker::new().ok_or("Screen fading requires the main thread.")?;
        let now = clock::now_ms();
        let plan = plan.filter(|p| p.clear_at_ms.is_none_or(|deadline| now < deadline));
        STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            let Some(plan) = plan else {
                if state.plan.is_some() {
                    state.overlays.clear();
                    state.frames.clear();
                    state.plan = None;
                    panel_level(window, false)?;
                }
                return Ok(());
            };
            let frames: Vec<_> = NSScreen::screens(mtm)
                .iter()
                .map(|screen| screen.frame())
                .collect();
            if frames.is_empty() {
                return Err("No screen is available for dimming.".into());
            }
            let rebuild = frames != state.frames;
            let resync =
                state.plan != Some(plan) || rebuild || now.saturating_sub(state.last_sync) > 2_000;
            state.last_sync = now;
            if !resync {
                return Ok(());
            }
            panel_level(window, true)?;
            if rebuild {
                let overlays = frames
                    .iter()
                    .map(|frame| create_overlay(*frame, mtm))
                    .collect::<Result<Vec<_>, _>>()?;
                state.overlays = overlays;
                state.frames = frames;
            }
            for overlay in &state.overlays {
                apply_plan(overlay, plan);
            }
            state.plan = Some(plan);
            Ok(())
        })
    }

    pub fn clear(window: &tauri::WebviewWindow) {
        STATE.with(|cell| *cell.borrow_mut() = State::default());
        let _ = panel_level(window, false);
    }

    /// Nonvisual development check. Created overlay windows never enter the screen.
    #[cfg(debug_assertions)]
    #[allow(dead_code)] // Called by the standalone main-thread check example.
    pub fn validate_hidden_overlays() -> Result<usize, String> {
        let mtm = MainThreadMarker::new().ok_or("Overlay checks require the main thread.")?;
        let screens = NSScreen::screens(mtm);
        for screen in screens.iter() {
            let overlay = create_overlay(screen.frame(), mtm)?;
            let layer = overlay
                .window
                .contentView()
                .and_then(|view| view.layer())
                .ok_or("The content view has no layer.")?;
            if overlay.window.isVisible()
                || !overlay.window.ignoresMouseEvents()
                || overlay.layer.opacity() != 0.0
                || overlay.window.level() != NSModalPanelWindowLevel
                || !std::ptr::eq::<CALayer>(&*layer, &*overlay.layer)
            {
                return Err("An overlay failed the hidden, click-through, or layer check.".into());
            }
        }
        Ok(screens.len())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn native_opacity_animation_is_finite_and_has_no_repeat() {
            let animation = opacity_animation(0.0, 0.85, 30.0);
            assert_eq!(animation.keyPath().unwrap().to_string(), "opacity");
            assert_eq!(animation.duration(), 30.0);
            assert_eq!(animation.repeatCount(), 0.0);
            assert!(!animation.autoreverses());
        }

        #[test]
        fn native_restore_is_an_instant_persistent_reset_not_a_fade_back() {
            let before = CACurrentMediaTime();
            let animation = restore_animation(10.0);
            assert_eq!(animation.duration(), 0.001);
            assert!(animation.beginTime() >= before + 10.0);
            assert!(!animation.isRemovedOnCompletion());
            assert_eq!(animation.fillMode().to_string(), "forwards");
        }
    }
}

#[cfg(target_os = "macos")]
pub use platform::*;
#[cfg(not(target_os = "macos"))]
pub fn sync(_: &tauri::WebviewWindow, _: Option<crate::dimming::DimPlan>) -> Result<(), String> {
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn clear(_: &tauri::WebviewWindow) {}
