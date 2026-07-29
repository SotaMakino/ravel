use serde::Deserialize;
use std::path::Path;

const CONFIG_FILE: &str = "ravel.json";

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub base: String,
    pub port: u16,
}

impl Config {
    /// Load `ravel.json` from the current working directory.
    pub fn load() -> Self {
        Self::load_from(Path::new("."))
    }

    /// Load `ravel.json` from `dir`. Taking the directory explicitly keeps
    /// this testable without mutating the process-wide working directory.
    pub fn load_from(dir: &Path) -> Self {
        let path = dir.join(CONFIG_FILE);
        if !path.exists() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", CONFIG_FILE, e);
                return Self::default();
            }
        };
        match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", CONFIG_FILE, e);
                Self::default()
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base: String::new(),
            port: 3000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_load_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(dir.path());
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_load_valid() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("ravel.json"),
            r#"{"base": "/my-repo", "port": 8080}"#,
        )
        .unwrap();
        let config = Config::load_from(dir.path());
        assert_eq!(config.base, "/my-repo");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_load_partial() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("ravel.json"), r#"{"base": "/app"}"#).unwrap();
        let config = Config::load_from(dir.path());
        assert_eq!(config.base, "/app");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_load_port_only() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("ravel.json"), r#"{"port": 5000}"#).unwrap();
        let config = Config::load_from(dir.path());
        assert_eq!(config.base, "");
        assert_eq!(config.port, 5000);
    }

    #[test]
    fn test_config_load_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("ravel.json"), r#"{invalid}"#).unwrap();
        let config = Config::load_from(dir.path());
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_config_load_empty_object() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("ravel.json"), "{}").unwrap();
        let config = Config::load_from(dir.path());
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
    }
}