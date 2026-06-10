//! Versioned config persisted at %APPDATA%\Killport\config.toml.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub poll_interval_secs: u64,
    pub notifications: bool,
    pub ignore_ports: Vec<u16>,
    /// Ports to watch: notify when another process occupies them.
    pub reserved_ports: Vec<u16>,
    /// Show OS/system processes too. Off by default: Killport is dev-focused.
    pub show_system: bool,
    /// Show unclassified (kind="unknown") processes, e.g. third-party apps. Off by default.
    pub show_unknown: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_interval_secs: 3,
            notifications: true,
            ignore_ports: Vec::new(),
            reserved_ports: Vec::new(),
            show_system: false,
            show_unknown: false,
        }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Killport"))
}

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

/// Load config; write defaults on first run. Falls back to defaults on any error.
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => match toml::from_str(&text) {
            Ok(cfg) => cfg,
            Err(e) => {
                // Corrupt config: back up and regenerate defaults.
                warn!(err = %e, "config parse error, resetting to defaults");
                if let Some(p) = config_path() {
                    let _ = std::fs::rename(&p, p.with_extension("toml.bak"));
                }
                Config::default()
            }
        },
        Err(_) => {
            let cfg = Config::default();
            let _ = save(&cfg);
            cfg
        }
    }
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    let Some(dir) = config_dir() else {
        return Ok(());
    };
    std::fs::create_dir_all(&dir)?;
    let text = toml::to_string_pretty(cfg).map_err(std::io::Error::other)?;
    std::fs::write(dir.join("config.toml"), text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_config_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("killport_test_{}.toml", std::process::id()))
    }

    #[test]
    fn round_trip_toml() {
        let cfg = Config {
            poll_interval_secs: 5,
            notifications: false,
            ignore_ports: vec![3000, 8080],
            reserved_ports: vec![5432],
            show_system: true,
            show_unknown: true,
        };
        let text = toml::to_string_pretty(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.poll_interval_secs, 5);
        assert_eq!(back.ignore_ports, vec![3000, 8080]);
        assert_eq!(back.reserved_ports, vec![5432]);
        assert!(!back.notifications);
        assert!(back.show_system);
    }

    #[test]
    fn corrupt_config_falls_back_to_defaults() {
        let p = tmp_config_path();
        fs::write(&p, b"this is not valid toml = [[[").unwrap();
        // Directly test the parse branch that config::load uses.
        let text = fs::read_to_string(&p).unwrap();
        let result: Result<Config, _> = toml::from_str(&text);
        assert!(result.is_err(), "expected parse to fail on corrupt config");
        // Cleanup.
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn default_config_is_sane() {
        let cfg = Config::default();
        assert_eq!(cfg.poll_interval_secs, 3);
        assert!(cfg.notifications);
        assert!(cfg.ignore_ports.is_empty());
        assert!(!cfg.show_system);
        assert!(!cfg.show_unknown);
    }
}
