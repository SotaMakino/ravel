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
    pub fn load() -> Self {
        let path = Path::new(CONFIG_FILE);
        if !path.exists() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(path) {
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
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = Config::load();
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_config_load_valid() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("ravel.json");
        fs::write(&config_path, r#"{"base": "/my-repo", "port": 8080}"#).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = Config::load();
        assert_eq!(config.base, "/my-repo");
        assert_eq!(config.port, 8080);
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_config_load_partial() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("ravel.json");
        fs::write(&config_path, r#"{"base": "/app"}"#).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = Config::load();
        assert_eq!(config.base, "/app");
        assert_eq!(config.port, 3000);
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_config_load_port_only() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("ravel.json");
        fs::write(&config_path, r#"{"port": 5000}"#).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = Config::load();
        assert_eq!(config.base, "");
        assert_eq!(config.port, 5000);
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_config_load_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("ravel.json");
        fs::write(&config_path, r#"{invalid}"#).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = Config::load();
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_config_load_empty_object() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("ravel.json");
        fs::write(&config_path, "{}").unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = Config::load();
        assert_eq!(config.base, "");
        assert_eq!(config.port, 3000);
        std::env::set_current_dir(original).unwrap();
    }
}