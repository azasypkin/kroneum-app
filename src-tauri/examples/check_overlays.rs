//! Run the real overlay constructor on the AppKit main thread without showing it.
// This example compiles only the native renderer's dependencies, not the engine.
#[allow(dead_code)]
#[path = "../src/clock.rs"]
mod clock;
#[allow(dead_code)]
#[path = "../src/dimming.rs"]
mod dimming;
#[allow(dead_code)]
#[path = "../src/screen_fade.rs"]
mod screen_fade;

#[cfg(all(target_os = "macos", debug_assertions))]
fn main() {
    let mut context = tauri::generate_context!();
    context.config_mut().identifier = "app.kroneum.overlay-check".into();
    context.config_mut().app.windows.clear();
    let app = tauri::Builder::default()
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let count = screen_fade::validate_hidden_overlays().map_err(std::io::Error::other)?;
            println!("Native overlay check passed on {count} displays. No overlay was shown.");
            app.handle().exit(0);
            Ok(())
        })
        .build(context)
        .expect("The native overlay check could not start");
    app.run(|_, _| {});
}

#[cfg(not(all(target_os = "macos", debug_assertions)))]
fn main() {
    eprintln!("Use a macOS debug build for the nonvisual overlay check.");
}
