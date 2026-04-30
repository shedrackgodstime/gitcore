use crate::models::GityConfig;
use std::fs;
use std::io;
use std::path::PathBuf;

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("gity")
        .join("config.json")
}

pub fn load_config() -> GityConfig {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        GityConfig::default()
    }
}

pub fn save_config(config: &GityConfig) -> io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;

    #[cfg(unix)]
    {
        use std::process::Command;
        let _ = Command::new("chmod")
            .args(["600", path.to_str().unwrap()])
            .output();
    }

    Ok(())
}
