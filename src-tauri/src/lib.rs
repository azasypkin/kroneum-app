mod attention;
mod clock;
pub mod dimming;
pub mod engine;
mod native;
mod persistence;
mod placement;
mod screen_fade;

use engine::{Action, ActionEffect, Alert, Engine, Settings, Snapshot, Sound, Timer};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;

struct Runtime {
    engine: Mutex<Engine>,
    settings_path: PathBuf,
    warning: Mutex<Option<String>>,
    last_snapshot: Mutex<Option<AppSnapshot>>,
    quitting: AtomicBool,
    launch_at_login: AtomicBool,
    clock_thread: Mutex<Option<std::thread::Thread>>,
}

struct MenuControls {
    work: MenuItem<tauri::Wry>,
    eye_break: MenuItem<tauri::Wry>,
    snooze: MenuItem<tauri::Wry>,
    acknowledge: MenuItem<tauri::Wry>,
    pause: MenuItem<tauri::Wry>,
    restore: MenuItem<tauri::Wry>,
}

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSnapshot {
    #[serde(flatten)]
    timer: Snapshot,
    launch_at_login: bool,
    warning: Option<String>,
}

fn snapshot(app: &AppHandle) -> Result<AppSnapshot, String> {
    let runtime = app.state::<Runtime>();
    let mut engine = runtime.engine.lock().map_err(|e| e.to_string())?;
    let now = clock::now_ms();
    engine.tick(now);
    let warning = runtime.warning.lock().map_err(|e| e.to_string())?.clone();
    Ok(AppSnapshot {
        timer: engine.snapshot(now),
        launch_at_login: runtime.launch_at_login.load(Ordering::Relaxed),
        warning,
    })
}

fn show(app: &AppHandle) {
    position_below_tray(app);
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.show().and_then(|()| window.set_focus()) {
            eprintln!("Show window: {e}");
        }
        publish(app);
    }
}

fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.hide() {
            eprintln!("Hide window: {e}");
        }
    }
}

fn tray_title(s: &Snapshot) -> String {
    let time = format!(
        "{:02}:{:02}",
        s.remaining_seconds / 60,
        s.remaining_seconds % 60
    );
    match s.phase {
        "running" => time,
        "paused" => format!("Ⅱ {time}"),
        "snoozed" => format!("z {time}"),
        "alert" => "00:00 !".into(),
        _ => "—".into(),
    }
}

// Called on the main thread. Read current state here, not a stale worker snapshot.
fn publish(app: &AppHandle) {
    let Ok(mut current) = snapshot(app) else {
        return;
    };
    if let Err(e) = native::alarm(
        current.timer.phase == "alert",
        current.timer.settings.sound,
        current.timer.settings.volume,
    ) {
        current.warning = Some(e);
    }
    let runtime = app.state::<Runtime>();
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = screen_fade::sync(&window, current.timer.dimming) {
            screen_fade::clear(&window);
            let _ = runtime
                .engine
                .lock()
                .map(|mut engine| engine.act(Action::RestoreScreens, clock::now_ms()));
            current.timer.dimming = None;
            let warning = format!("Screen fading stopped: {e}");
            if let Ok(mut stored) = runtime.warning.lock() {
                *stored = Some(warning.clone());
            }
            current.warning = Some(warning);
        }
    }
    let mut last = runtime
        .last_snapshot
        .lock()
        .expect("snapshot lock poisoned");
    let new_alert =
        current.timer.phase == "alert" && last.as_ref().is_none_or(|s| s.timer.phase != "alert");
    if let Some(controls) = app.try_state::<MenuControls>() {
        if last.as_ref().is_none_or(|s| {
            s.timer.phase != current.timer.phase
                || s.timer.settings != current.timer.settings
                || s.timer.dimming != current.timer.dimming
        }) {
            let settings = &current.timer.settings;
            let _ = controls
                .restore
                .set_enabled(current.timer.dimming.is_some());
            let _ = controls.work.set_text(format!(
                "Start work timer ({})",
                menu_duration(settings.work_seconds)
            ));
            let _ = controls.eye_break.set_text(format!(
                "Start eye break ({})",
                menu_duration(settings.break_seconds)
            ));
            let _ = controls.snooze.set_text(format!(
                "Snooze for {}",
                menu_duration(settings.snooze_seconds)
            ));
            let _ = controls.snooze.set_enabled(current.timer.phase == "alert");
            let _ = controls.acknowledge.set_enabled(
                current.timer.phase == "alert"
                    && matches!(current.timer.alert, Some(Alert::Finished { .. })),
            );
            let _ = controls.pause.set_text(if current.timer.phase == "paused" {
                "Resume timer"
            } else {
                "Pause timer"
            });
            let _ = controls
                .pause
                .set_enabled(matches!(current.timer.phase, "running" | "paused"));
        }
    }
    if let Some(tray) = app.tray_by_id("kroneum") {
        if let Err(e) = native::sync_idle_pulse(&tray, current.timer.phase) {
            eprintln!("Idle pulse: {e}");
        }
        if last
            .as_ref()
            .is_none_or(|s| tray_title(&s.timer) != tray_title(&current.timer))
        {
            if let Err(e) = tray.set_title(Some(tray_title(&current.timer))) {
                eprintln!("Tray title: {e}");
            }
        }
        let label = match (
            current.timer.phase,
            current.timer.timer,
            current.timer.alert,
        ) {
            ("alert", _, Some(Alert::Idle)) => "Pick a timer — Kroneum is waiting",
            ("alert", _, _) => "Timer finished — respond or snooze",
            ("running", Some(Timer::EyeBreak), _) => "Eye break — look far away and blink",
            ("running", _, _) => "Work timer — Kroneum",
            ("paused", _, _) => "Paused — reminders are suspended",
            ("snoozed", _, _) => "Reminder snoozed — Kroneum",
            _ => "Kroneum — ready when you are",
        };
        if last.as_ref().is_none_or(|s| {
            s.timer.phase != current.timer.phase || s.timer.timer != current.timer.timer
        }) {
            if let Err(e) = tray.set_tooltip(Some(label)) {
                eprintln!("Tray tooltip: {e}");
            }
        }
    }
    let visible = app
        .get_webview_window("main")
        .is_some_and(|w| w.is_visible().unwrap_or(false));
    if (visible || new_alert) && last.as_ref() != Some(&current) {
        if let Err(e) = app.emit("timer-state", &current) {
            eprintln!("Timer event: {e}");
        }
    }
    *last = Some(current);
    drop(last);
    if new_alert {
        position_below_tray(app);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show().and_then(|()| window.set_focus());
        }
    }
}

fn schedule_publish(app: &AppHandle) {
    schedule_action_publish(app, ActionEffect::default());
}

fn should_hide_after_action(effect: ActionEffect, current: &Snapshot) -> bool {
    effect.timer_started && current.settings.hide_on_timer_start && current.phase == "running"
}

fn schedule_action_publish(app: &AppHandle, effect: ActionEffect) {
    let handle = app.clone();
    if let Err(e) = app.run_on_main_thread(move || {
        publish(&handle);
        if !effect.timer_started {
            return;
        }
        let should_hide = handle.state::<Runtime>().engine.lock().is_ok_and(|engine| {
            should_hide_after_action(effect, &engine.snapshot(clock::now_ms()))
        });
        if should_hide {
            hide(&handle);
        }
    }) {
        eprintln!("Main-thread update: {e}");
    }
}

fn apply(app: &AppHandle, action: Action) -> Result<(), String> {
    let result = app
        .state::<Runtime>()
        .engine
        .lock()
        .map_err(|e| e.to_string())?
        .act(action, clock::now_ms());
    schedule_action_publish(app, result.as_ref().copied().unwrap_or_default());
    wake_clock(app);
    result.map(|_| ())
}

#[cfg(test)]
mod window_policy_tests {
    use super::*;

    #[test]
    fn auto_hide_is_off_by_default_and_requires_a_successful_start() {
        let mut engine = Engine::new(Settings::default(), 0);
        let effect = engine.act(Action::StartWork, 0).unwrap();
        assert!(!should_hide_after_action(effect, &engine.snapshot(0)));
        engine.settings.hide_on_timer_start = true;
        assert!(should_hide_after_action(effect, &engine.snapshot(0)));
        assert!(!should_hide_after_action(
            ActionEffect::default(),
            &engine.snapshot(0)
        ));
    }

    #[test]
    fn delayed_start_effect_cannot_hide_a_completion_reminder_or_a_paused_timer() {
        let mut engine = Engine::new(
            Settings {
                hide_on_timer_start: true,
                break_seconds: 1,
                ..Settings::default()
            },
            0,
        );
        let start = engine.act(Action::StartBreak, 0).unwrap();
        engine.act(Action::Pause, 100).unwrap();
        assert!(!should_hide_after_action(start, &engine.snapshot(100)));
        engine.act(Action::Resume, 200).unwrap();
        engine.tick(1_100);
        assert_eq!(engine.snapshot(1_100).phase, "alert");
        assert!(!should_hide_after_action(start, &engine.snapshot(1_100)));
    }
}

#[tauri::command]
fn get_state(app: AppHandle) -> Result<AppSnapshot, String> {
    snapshot(&app)
}

#[tauri::command]
fn timer_action(app: AppHandle, action: Action) -> Result<AppSnapshot, String> {
    apply(&app, action)?;
    snapshot(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<AppSnapshot, String> {
    settings.validate()?;
    let runtime = app.state::<Runtime>();
    let mut engine = runtime.engine.lock().map_err(|e| e.to_string())?;
    persistence::save(&runtime.settings_path, &settings)?;
    engine.update_settings(settings, clock::now_ms());
    *runtime.warning.lock().map_err(|e| e.to_string())? = None;
    drop(engine);
    schedule_publish(&app);
    wake_clock(&app);
    snapshot(&app)
}

fn wake_clock(app: &AppHandle) {
    if let Ok(thread) = app.state::<Runtime>().clock_thread.lock() {
        if let Some(thread) = thread.as_ref() {
            thread.unpark();
        }
    }
}

#[tauri::command]
fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<AppSnapshot, String> {
    if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|e| e.to_string())?;
    app.state::<Runtime>()
        .launch_at_login
        .store(enabled, Ordering::Relaxed);
    snapshot(&app)
}

fn menu_duration(seconds: u32) -> String {
    if seconds.is_multiple_of(60) {
        format!("{} min", seconds / 60)
    } else {
        format!("{seconds} sec")
    }
}

#[tauri::command]
async fn preview_sound(app: AppHandle, sound: Sound, volume: f32) -> Result<(), String> {
    if !volume.is_finite() || !(0.05..=1.0).contains(&volume) {
        return Err("Volume must be between 5% and 100%.".into());
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = sender.send(native::preview(sound, volume));
    })
    .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || receiver.recv().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    hide(&app);
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

fn position_below_tray(app: &AppHandle) {
    if let (Some(window), Some(tray)) = (app.get_webview_window("main"), app.tray_by_id("kroneum"))
    {
        if let Err(e) = native::position_panel(&window, &tray) {
            eprintln!("Panel placement: {e}");
        }
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(tauri_plugin_autostart::Builder::new().macos_launcher(tauri_plugin_autostart::MacosLauncher::LaunchAgent).args(["--background"]).build())
        .invoke_handler(tauri::generate_handler![get_state, timer_action, save_settings, set_launch_at_login, preview_sound, hide_window, quit_app])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let path = app.path().app_config_dir()?.join("settings.json");
            let (settings, warning) = match persistence::load(&path) {
                Ok(settings) => (settings, None),
                Err(e) => (Settings::default(), Some(format!("The saved settings could not be read. Defaults are active. Your file is unchanged. {e}"))),
            };
            app.manage(Runtime { engine: Mutex::new(Engine::new(settings, clock::now_ms())), settings_path: path,
                warning: Mutex::new(warning), last_snapshot: Mutex::new(None), quitting: AtomicBool::new(false),
                launch_at_login: AtomicBool::new(app.autolaunch().is_enabled().unwrap_or(false)), clock_thread: Mutex::new(None) });
            let work = MenuItem::with_id(app, "work", "Start work timer", true, None::<&str>)?;
            let eye_break = MenuItem::with_id(app, "break", "Start eye break", true, None::<&str>)?;
            let open = MenuItem::with_id(app, "open", "Open Kroneum", true, None::<&str>)?;
            let snooze = MenuItem::with_id(app, "snooze", "Snooze reminder", true, None::<&str>)?;
            let acknowledge = MenuItem::with_id(app, "acknowledge", "Accept timer", false, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause timer", false, None::<&str>)?;
            let restore = MenuItem::with_id(app, "restore", "Restore screens", false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Kroneum", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let bottom_separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&open, &separator, &work, &eye_break, &pause, &acknowledge, &snooze, &restore, &bottom_separator, &quit])?;
            app.manage(MenuControls { work, eye_break, snooze, acknowledge, pause, restore });
            let tray = TrayIconBuilder::with_id("kroneum")
                .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?)
                .icon_as_template(true).title("—").tooltip("Kroneum — eye break timer")
                .menu(&menu).show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show(app), "work" => { let _ = apply(app, Action::StartWork); },
                    "break" => { let _ = apply(app, Action::StartBreak); },
                    "acknowledge" => { let _ = apply(app, Action::Acknowledge); },
                    "restore" => {
                        if let Some(window) = app.get_webview_window("main") { screen_fade::clear(&window); }
                        let _ = apply(app, Action::RestoreScreens);
                    },
                    "pause" => {
                        let paused = app.state::<Runtime>().engine.lock().is_ok_and(|e| e.snapshot(clock::now_ms()).phase == "paused");
                        let _ = apply(app, if paused { Action::Resume } else { Action::Pause });
                    },
                    "snooze" => { let _ = apply(app, Action::Snooze); }, "quit" => app.exit(0), _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if app.get_webview_window("main").is_some_and(|w| w.is_visible().unwrap_or(false)) { hide(app); }
                        else { show(app); }
                    }
                }).build(app)?;
            native::configure_tray(&tray)?;
            publish(app.handle());
            if !std::env::args().any(|a| a == "--background") { show(app.handle()); }
            let handle = app.handle().clone();
            let worker = std::thread::Builder::new().name("kroneum-clock".into()).spawn(move || {
                loop {
                    if handle.state::<Runtime>().quitting.load(Ordering::Relaxed) { break; }
                    let delay = {
                        let runtime = handle.state::<Runtime>();
                        let mut engine = runtime.engine.lock().expect("engine lock poisoned");
                        let now = clock::now_ms();
                        engine.tick(now);
                        engine.next_wake_ms(now)
                    };
                    schedule_publish(&handle);
                    std::thread::park_timeout(Duration::from_millis(delay));
                }
            })?;
            *app.state::<Runtime>().clock_thread.lock().expect("clock lock poisoned") = Some(worker.thread().clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => { api.prevent_close(); hide(window.app_handle()); },
            tauri::WindowEvent::Focused(false) => {
                let app = window.app_handle();
                if app.try_state::<Runtime>().is_some_and(|r| r.engine.lock().is_ok_and(|e| e.snapshot(clock::now_ms()).phase != "alert")) { hide(app); }
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("Kroneum could not start");
    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event {
            if let Some(window) = app.get_webview_window("main") {
                screen_fade::clear(&window);
            }
            app.state::<Runtime>()
                .quitting
                .store(true, Ordering::Relaxed);
            wake_clock(app);
            let _ = native::alarm(false, Sound::Chime, 0.45);
            native::stop_preview();
        }
    });
}
