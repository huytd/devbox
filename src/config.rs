use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBoxConfig {
    pub container_name: String,
    pub volume_name: String,
    pub absolute_path: String,
    pub backend: BackendType,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendType {
    Docker,
    Lima,
}

impl DevBoxConfig {
    pub fn new(path: &str, backend: BackendType) -> Self {
        let hash = Self::compute_hash(path);
        Self {
            container_name: format!("devbox-{}", hash),
            volume_name: format!("devbox-data-{}", hash),
            absolute_path: path.to_string(),
            backend,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let config_path = path.join(".devbox").join("config.json");
        let content = fs::read_to_string(&config_path)
            .context(format!("Failed to read config at {:?}", config_path))?;
        serde_json::from_str(&content)
            .context("Failed to parse config JSON")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let devbox_dir = path.join(".devbox");
        fs::create_dir_all(&devbox_dir)?;
        let config_path = devbox_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn exists(path: &Path) -> bool {
        path.join(".devbox").join("config.json").exists()
    }

    fn compute_hash(path: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..8].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_new() {
        let config = DevBoxConfig::new("/tmp/test", BackendType::Docker);
        assert!(config.container_name.starts_with("devbox-"));
        assert!(config.volume_name.starts_with("devbox-data-"));
        assert_eq!(config.backend, BackendType::Docker);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let config = DevBoxConfig::new(dir.path().to_str().unwrap(), BackendType::Docker);
        config.save(dir.path()).unwrap();

        let loaded = DevBoxConfig::load(dir.path()).unwrap();
        assert_eq!(loaded.container_name, config.container_name);
        assert_eq!(loaded.absolute_path, config.absolute_path);
    }

    #[test]
    fn test_config_exists() {
        let dir = tempdir().unwrap();
        assert!(!DevBoxConfig::exists(dir.path()));

        let config = DevBoxConfig::new(dir.path().to_str().unwrap(), BackendType::Docker);
        config.save(dir.path()).unwrap();
        assert!(DevBoxConfig::exists(dir.path()));
    }

    #[test]
    fn test_hash_consistency() {
        let hash1 = DevBoxConfig::compute_hash("/tmp/test");
        let hash2 = DevBoxConfig::compute_hash("/tmp/test");
        assert_eq!(hash1, hash2);

        let hash3 = DevBoxConfig::compute_hash("/tmp/different");
        assert_ne!(hash1, hash3);
    }
}