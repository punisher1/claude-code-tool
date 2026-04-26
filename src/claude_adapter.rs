use crate::models::EnvValue;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, serde_json::Value>>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

pub struct ClaudeAdapter;

impl ClaudeAdapter {
    fn home_dir() -> Result<PathBuf> {
        #[cfg(test)]
        if let Some(home_dir) = std::env::var_os("CCT_TEST_HOME") {
            return Ok(PathBuf::from(home_dir));
        }

        dirs::home_dir().context("Failed to get home directory")
    }

    pub fn get_settings_path() -> Result<PathBuf> {
        let home_dir = Self::home_dir()?;
        Ok(home_dir.join(".claude").join("settings.json"))
    }

    pub fn update_settings(path: &Path, env_map: HashMap<String, EnvValue>) -> Result<()> {
        // Read existing settings or create default
        let mut settings = if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read settings file: {:?}", path))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse settings JSON: {:?}", path))?
        } else {
            ClaudeSettings {
                env: None,
                other: HashMap::new(),
            }
        };

        // Convert EnvValue to serde_json::Value
        let json_env: HashMap<String, serde_json::Value> = env_map
            .into_iter()
            .map(|(k, v)| (k, v.to_json_value()))
            .collect();

        // Merge env field instead of replacing unrelated user settings.
        let mut merged_env = settings.env.unwrap_or_default();
        merged_env.extend(json_env);
        settings.env = Some(merged_env);

        // Write back to file
        let content = serde_json::to_string_pretty(&settings)
            .context("Failed to serialize settings to JSON")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create settings directory: {:?}", parent))?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write settings file: {:?}", path))?;

        Ok(())
    }

    pub fn remove_env_keys(path: &Path, keys: &[&str]) -> Result<()> {
        let mut settings = if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read settings file: {:?}", path))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse settings JSON: {:?}", path))?
        } else {
            ClaudeSettings {
                env: None,
                other: HashMap::new(),
            }
        };

        if let Some(env) = settings.env.as_mut() {
            for key in keys {
                env.remove(*key);
            }
        }

        let content = serde_json::to_string_pretty(&settings)
            .context("Failed to serialize settings to JSON")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create settings directory: {:?}", parent))?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write settings file: {:?}", path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EnvValue;
    use tempfile::TempDir;

    #[test]
    fn test_update_settings_preserve_fields() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let settings_path = temp_dir.path().join("settings.json");

        // Create initial settings with extra fields
        let initial_settings = serde_json::json!({
            "theme": "dark",
            "font_size": 14,
            "env": {
                "ANTHROPIC_API_KEY": "old-key"
            }
        });

        fs::write(&settings_path, initial_settings.to_string())?;

        // Update with new env vars
        let mut new_env = HashMap::new();
        new_env.insert(
            "ANTHROPIC_API_KEY".to_string(),
            EnvValue::String("new-key-123".to_string()),
        );
        new_env.insert(
            "CUSTOM_VAR".to_string(),
            EnvValue::String("custom-value".to_string()),
        );

        ClaudeAdapter::update_settings(&settings_path, new_env)?;

        // Read and verify
        let content = fs::read_to_string(&settings_path)?;
        let parsed: serde_json::Value = serde_json::from_str(&content)?;

        // Verify env was updated
        assert_eq!(
            parsed["env"]["ANTHROPIC_API_KEY"].as_str(),
            Some("new-key-123")
        );
        assert_eq!(parsed["env"]["CUSTOM_VAR"].as_str(), Some("custom-value"));

        // Verify other fields were preserved
        assert_eq!(parsed["theme"].as_str(), Some("dark"));
        assert_eq!(parsed["font_size"].as_i64(), Some(14));

        Ok(())
    }
}
