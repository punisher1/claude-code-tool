use crate::models::AppConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        Ok(Self { config_path })
    }

    #[cfg(test)]
    pub fn new_with_path(path: PathBuf) -> Self {
        Self { config_path: path }
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home_dir.join(".cct").join("config.toml"))
    }

    pub fn load_config() -> Result<AppConfig> {
        let config_path = Self::get_config_path()?;
        Self::load_config_from_path(&config_path)
    }

    pub fn load_config_from_path(path: &Path) -> Result<AppConfig> {
        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {:?}", path))?;

        Ok(config)
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        let content =
            toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        fs::write(&self.config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", self.config_path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_save_cycle() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");

        let manager = ConfigManager::new_with_path(config_path.clone());

        // Test loading non-existent file returns default
        let config = ConfigManager::load_config_from_path(&config_path)?;
        assert_eq!(config.providers.len(), 0);
        assert_eq!(config.configs.len(), 0);
        assert_eq!(config.current, None);

        // Save a config
        let mut providers = std::collections::HashMap::new();
        let env = std::collections::HashMap::new();
        let provider = crate::models::Provider {
            description: Some("Test Provider".to_string()),
            env: Some(env),
        };
        providers.insert("test".to_string(), provider);

        let mut configs = std::collections::HashMap::new();
        let config_instance = crate::models::ConfigInstance {
            provider: "test".to_string(),
            api_key: Some("key123".to_string()),
            env: None,
        };
        configs.insert("my-config".to_string(), config_instance);

        let app_config = AppConfig {
            providers,
            configs,
            current: Some("my-config".to_string()),
        };

        manager.save_config(&app_config)?;

        // Load it back and verify
        let loaded_config = ConfigManager::load_config_from_path(&config_path)?;
        assert_eq!(loaded_config, app_config);
        assert_eq!(loaded_config.current, Some("my-config".to_string()));

        Ok(())
    }
}
