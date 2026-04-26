use crate::claude_adapter::ClaudeAdapter;
use crate::commands::Command;
use crate::config_manager::ConfigManager;
use crate::models::AppConfig;
use crate::models::EnvValue;
use crate::provider_store::ProviderStore;
use crate::utils::backup_file;
use anyhow::{Result, anyhow};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use std::collections::HashMap;

pub struct UseCommand {
    pub alias: Option<String>,
}

pub struct ResetCommand;

const PROVIDER_MANAGED_ENV_KEYS: &[&str] = &[
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_AUTH_TOKEN",
    "API_TIMEOUT_MS",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    "CLAUDE_CODE_EFFORT_LEVEL",
];

impl Command for UseCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        let alias = match self.alias {
            Some(alias) => alias,
            None => {
                // Interactive selection mode
                // Prepare items for selection (with None option)
                let mut items = vec!["none [Clear all provider settings]".to_string()];
                let mut keys: Vec<_> = config.configs.keys().cloned().collect();
                keys.sort();
                items.extend(keys);

                // Find current selection index (None is at index 0)
                let current_index = match config.current.as_ref() {
                    Some(current) => {
                        items
                            .iter()
                            .position(|item| item == current)
                            // .map(|i| i + 1) // +1 because None is at index 0
                            .unwrap_or(0)
                    }
                    None => 0,
                };

                // Show selection dialog
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a configuration to use")
                    .items(&items)
                    .default(current_index)
                    .interact();

                match selection {
                    Ok(index) => {
                        if index == 0 {
                            // User selected "None" - clear all settings
                            return self.clear_settings(config);
                        } else {
                            items[index].clone()
                        }
                    }
                    Err(e) => return Err(anyhow!("Failed to read selection: {}", e)),
                }
            }
        };

        // Check if the configuration exists
        let config_instance = config
            .configs
            .get(&alias)
            .ok_or_else(|| anyhow!("Configuration '{}' not found", alias))?;

        // Get merged providers
        let merged_providers = ProviderStore::get_merged_providers(&config.providers);

        // Get the provider
        let provider = merged_providers
            .get(&config_instance.provider)
            .ok_or_else(|| {
                anyhow!(
                    "Provider '{}' not found for configuration '{}'",
                    config_instance.provider,
                    alias
                )
            })?;

        // Merge environment variables
        // Priority: config env > provider env > generated from api_key
        let mut env_vars = HashMap::new();

        // Start with provider's env
        if let Some(provider_env) = &provider.env {
            env_vars.extend(provider_env.clone());
        }

        // Override with config's env if any
        if let Some(config_env) = &config_instance.env {
            env_vars.extend(config_env.clone());
        }

        // Add API key if provided
        if let Some(api_key) = &config_instance.api_key {
            env_vars.insert(
                "ANTHROPIC_AUTH_TOKEN".to_string(),
                EnvValue::String(api_key.clone()),
            );
        }

        // Update Claude settings (with backup)
        let settings_path = ClaudeAdapter::get_settings_path()?;
        backup_file(&settings_path, "settings", 5)?;
        ClaudeAdapter::remove_env_keys(&settings_path, PROVIDER_MANAGED_ENV_KEYS)?;
        ClaudeAdapter::update_settings(&settings_path, env_vars)?;

        // Update current configuration
        config.current = Some(alias.clone());

        // Save config
        let manager = ConfigManager::new()?;
        manager.save_config(config)?;

        println!(
            "{} Now using configuration: {}",
            style("✓").green(),
            style(&alias).bold()
        );
        println!("  Provider: {}", style(&config_instance.provider).cyan());

        if let Some(env) = &provider.env {
            if let Some(base_url) = env.get("ANTHROPIC_BASE_URL") {
                println!("  Base URL: {}", style(base_url).dim());
            }
            if let Some(model) = env.get("ANTHROPIC_MODEL") {
                println!("  Model: {}", style(model).cyan());
            }
        }

        Ok(())
    }
}

impl UseCommand {
    /// Clear all provider settings from Claude settings
    fn clear_settings(self, config: &mut AppConfig) -> Result<()> {
        // Remove only provider-managed env vars from Claude settings.
        let settings_path = ClaudeAdapter::get_settings_path()?;
        backup_file(&settings_path, "settings", 5)?;
        ClaudeAdapter::remove_env_keys(&settings_path, PROVIDER_MANAGED_ENV_KEYS)?;

        // Clear current configuration
        config.current = None;

        // Save config
        let manager = ConfigManager::new()?;
        manager.save_config(config)?;

        println!("{} Cleared all provider settings", style("✓").green());

        Ok(())
    }
}

impl Command for ResetCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        let cmd = UseCommand { alias: None };
        cmd.clear_settings(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;
    use crate::models::{AppConfig, ConfigInstance};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static HOME_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_env_var_merging() {
        let mut provider_env = HashMap::new();
        provider_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://provider.com".to_string(),
        );
        provider_env.insert("ANTHROPIC_MODEL".to_string(), "provider-model".to_string());

        let mut config_env = HashMap::new();
        config_env.insert("ANTHROPIC_MODEL".to_string(), "config-model".to_string());
        config_env.insert("CUSTOM_VAR".to_string(), "custom-value".to_string());

        let mut merged = HashMap::new();
        merged.extend(provider_env); // Start with provider env
        merged.extend(config_env); // Override with config env
        merged.insert("ANTHROPIC_API_KEY".to_string(), "test-key".to_string()); // Add API key

        assert_eq!(
            merged.get("ANTHROPIC_BASE_URL"),
            Some(&"https://provider.com".to_string())
        );
        assert_eq!(
            merged.get("ANTHROPIC_MODEL"),
            Some(&"config-model".to_string())
        ); // Config overrides provider
        assert_eq!(merged.get("CUSTOM_VAR"), Some(&"custom-value".to_string()));
        assert_eq!(
            merged.get("ANTHROPIC_API_KEY"),
            Some(&"test-key".to_string())
        );
    }

    #[test]
    fn test_use_reapplies_active_config_from_providers_toml() -> anyhow::Result<()> {
        let _guard = HOME_ENV_LOCK.lock().unwrap();
        let temp_dir = TempDir::new()?;
        let cct_dir = temp_dir.path().join(".cct");
        let claude_dir = temp_dir.path().join(".claude");
        fs::create_dir_all(&cct_dir)?;
        fs::create_dir_all(&claude_dir)?;

        fs::write(
            cct_dir.join("providers.toml"),
            r#"
[deepseek]
description = "DeepSeek API"

[deepseek.env]
ANTHROPIC_BASE_URL = "https://api.deepseek.com/anthropic"
ANTHROPIC_MODEL = "file-model"
"#,
        )?;
        fs::write(
            claude_dir.join("settings.json"),
            r#"{
                "env": {
                    "ANTHROPIC_MODEL": "old-model",
                    "ANTHROPIC_DEFAULT_OPUS_MODEL": "stale-opus",
                    "CLAUDE_CODE_EFFORT_LEVEL": "stale-effort",
                    "CUSTOM_VAR": "keep-me"
                }
            }"#,
        )?;

        let old_test_home = std::env::var_os("CCT_TEST_HOME");
        unsafe {
            std::env::set_var("CCT_TEST_HOME", temp_dir.path());
        }

        let mut configs = HashMap::new();
        configs.insert(
            "deepseek".to_string(),
            ConfigInstance {
                provider: "deepseek".to_string(),
                api_key: None,
                env: None,
            },
        );
        let mut config = AppConfig {
            providers: HashMap::new(),
            configs,
            current: Some("deepseek".to_string()),
        };

        let result = UseCommand {
            alias: Some("deepseek".to_string()),
        }
        .execute(&mut config);

        unsafe {
            match old_test_home {
                Some(value) => std::env::set_var("CCT_TEST_HOME", value),
                None => std::env::remove_var("CCT_TEST_HOME"),
            }
        }

        result?;

        let settings: Value =
            serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json"))?)?;
        assert_eq!(settings["env"]["ANTHROPIC_MODEL"], "file-model");
        assert_eq!(settings["env"]["CUSTOM_VAR"], "keep-me");
        assert!(
            settings["env"]
                .get("ANTHROPIC_DEFAULT_OPUS_MODEL")
                .is_none()
        );
        assert!(settings["env"].get("CLAUDE_CODE_EFFORT_LEVEL").is_none());

        Ok(())
    }

    #[test]
    fn test_reset_only_removes_provider_managed_env_vars() -> anyhow::Result<()> {
        let _guard = HOME_ENV_LOCK.lock().unwrap();
        let temp_dir = TempDir::new()?;
        let claude_dir = temp_dir.path().join(".claude");
        fs::create_dir_all(&claude_dir)?;

        fs::write(
            claude_dir.join("settings.json"),
            r#"{
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.deepseek.com/anthropic",
                    "ANTHROPIC_MODEL": "deepseek-v4-pro",
                    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "deepseek-v4-flash",
                    "ANTHROPIC_DEFAULT_SONNET_MODEL": "deepseek-v4-pro",
                    "ANTHROPIC_DEFAULT_OPUS_MODEL": "deepseek-v4-pro",
                    "API_TIMEOUT_MS": 3000000,
                    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": 1,
                    "CLAUDE_CODE_EFFORT_LEVEL": "max",
                    "CUSTOM_VAR": "keep-me",
                    "ANTHROPIC_AUTH_TOKEN": "keep-token"
                },
                "theme": "dark"
            }"#,
        )?;

        let old_test_home = std::env::var_os("CCT_TEST_HOME");
        unsafe {
            std::env::set_var("CCT_TEST_HOME", temp_dir.path());
        }

        let mut config = AppConfig {
            providers: HashMap::new(),
            configs: HashMap::new(),
            current: Some("deepseek".to_string()),
        };

        let result = ResetCommand.execute(&mut config);

        unsafe {
            match old_test_home {
                Some(value) => std::env::set_var("CCT_TEST_HOME", value),
                None => std::env::remove_var("CCT_TEST_HOME"),
            }
        }

        result?;

        let settings: Value =
            serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json"))?)?;
        let env = settings["env"].as_object().unwrap();

        assert!(!env.contains_key("ANTHROPIC_BASE_URL"));
        assert!(!env.contains_key("ANTHROPIC_MODEL"));
        assert!(!env.contains_key("ANTHROPIC_DEFAULT_HAIKU_MODEL"));
        assert!(!env.contains_key("ANTHROPIC_DEFAULT_SONNET_MODEL"));
        assert!(!env.contains_key("ANTHROPIC_DEFAULT_OPUS_MODEL"));
        assert!(!env.contains_key("API_TIMEOUT_MS"));
        assert!(!env.contains_key("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"));
        assert!(!env.contains_key("CLAUDE_CODE_EFFORT_LEVEL"));
        assert_eq!(
            env.get("CUSTOM_VAR").and_then(|value| value.as_str()),
            Some("keep-me")
        );
        assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
        assert_eq!(settings["theme"].as_str(), Some("dark"));
        assert_eq!(config.current, None);

        Ok(())
    }
}
