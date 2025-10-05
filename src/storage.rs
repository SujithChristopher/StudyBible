use crate::types::AppSettings;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Storage manager for persisting app settings across platforms
pub struct SettingsStorage {
    config_path: PathBuf,
}

impl SettingsStorage {
    /// Creates a new SettingsStorage instance
    ///
    /// Platform-specific paths:
    /// - Windows: C:\Users\<user>\AppData\Roaming\StudyBible\settings.json
    /// - macOS: ~/Library/Application Support/StudyBible/settings.json
    /// - Linux: ~/.config/StudyBible/settings.json
    /// - Android: App-specific internal storage (no permissions needed)
    pub fn new() -> Result<Self, String> {
        let proj_dirs = ProjectDirs::from("com", "studybible", "StudyBible")
            .ok_or("Could not determine config directory")?;

        // For Android, prefer data_dir over config_dir (internal app storage)
        let config_dir = if cfg!(target_os = "android") {
            println!("🤖 Android detected: using data_dir for settings");
            proj_dirs.data_dir()
        } else {
            proj_dirs.config_dir()
        };

        println!("📁 Config directory: {:?}", config_dir);

        // Create config directory if it doesn't exist
        match fs::create_dir_all(config_dir) {
            Ok(_) => println!("✓ Config directory created/verified"),
            Err(e) => {
                eprintln!("❌ Failed to create config directory: {}", e);
                return Err(format!("Failed to create config directory: {}", e));
            }
        }

        let config_path = config_dir.join("settings.json");
        println!("💾 Settings will be stored at: {:?}", config_path);

        Ok(Self { config_path })
    }

    /// Load settings from disk
    /// Returns default settings if file doesn't exist or is corrupted
    pub fn load(&self) -> AppSettings {
        match fs::read_to_string(&self.config_path) {
            Ok(contents) => {
                // Try to parse the JSON
                match serde_json::from_str::<AppSettings>(&contents) {
                    Ok(settings) => {
                        println!("✓ Loaded settings from: {:?}", self.config_path);
                        settings
                    }
                    Err(e) => {
                        eprintln!("⚠ Failed to parse settings file: {}. Using defaults.", e);
                        AppSettings::default()
                    }
                }
            }
            Err(_) => {
                // File doesn't exist yet, use defaults
                println!("ℹ No settings file found. Using defaults.");
                AppSettings::default()
            }
        }
    }

    /// Save settings to disk
    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        println!("💾 Attempting to save settings...");

        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| {
                eprintln!("❌ Serialization failed: {}", e);
                format!("Failed to serialize settings: {}", e)
            })?;

        println!("📝 Serialized settings ({} bytes)", json.len());

        match fs::write(&self.config_path, &json) {
            Ok(_) => {
                println!("✓ Successfully saved settings to: {:?}", self.config_path);

                // Verify the write
                if let Ok(contents) = fs::read_to_string(&self.config_path) {
                    println!("✓ Verified: file contains {} bytes", contents.len());
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to write settings file: {} (path: {:?})", e, self.config_path);
                Err(format!("Failed to write settings file: {}", e))
            }
        }
    }

    /// Get the path where settings are stored
    pub fn get_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Delete the settings file (reset to defaults)
    pub fn delete(&self) -> Result<(), String> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| format!("Failed to delete settings file: {}", e))?;
            println!("✓ Deleted settings file");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_roundtrip() {
        let storage = SettingsStorage::new().unwrap();

        let mut settings = AppSettings::default();
        settings.font_size = 20.0;

        // Save
        storage.save(&settings).unwrap();

        // Load
        let loaded = storage.load();
        assert_eq!(loaded.font_size, 20.0);

        // Cleanup
        storage.delete().unwrap();
    }
}
