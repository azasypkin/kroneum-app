use crate::engine::Settings;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Deserialize, Serialize)]
struct SavedSettings {
    version: u32,
    settings: Settings,
}

pub fn load(path: &Path) -> Result<Settings, String> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() > 65_536 {
        return Err("The settings file is too large.".into());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let saved: SavedSettings = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if saved.version != 1 {
        return Err("The settings file has an unsupported version.".into());
    }
    saved.settings.validate()?;
    Ok(saved.settings)
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    settings.validate()?;
    let parent = path
        .parent()
        .ok_or("The settings path has no parent directory.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&SavedSettings {
        version: 1,
        settings: settings.clone(),
    })
    .map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    let mut file = fs::File::create(&temp).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temp, path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_settings_use_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load(&dir.path().join("settings.json")).unwrap(),
            Settings::default()
        );
    }

    #[test]
    fn settings_round_trip_without_a_partial_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/settings.json");
        let settings = Settings {
            break_seconds: 45,
            max_dim_percent: 55,
            always_dim_break: true,
            hide_on_timer_start: true,
            ..Settings::default()
        };
        save(&path, &settings).unwrap();
        assert_eq!(load(&path).unwrap(), settings);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn corrupt_settings_remain_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "not json").unwrap();
        assert!(load(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "not json");
    }

    #[test]
    fn invalid_save_leaves_the_previous_settings_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        save(&path, &Settings::default()).unwrap();
        assert!(save(
            &path,
            &Settings {
                break_seconds: 0,
                ..Settings::default()
            }
        )
        .is_err());
        assert_eq!(load(&path).unwrap(), Settings::default());
    }
}
