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
                        items.iter().position(|item| item == current)
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

        // Check if already using this configuration
        if config.current.as_ref() == Some(&alias) {
            println!(
                "{} Configuration '{}' is already active",
                style("!").yellow(),
                style(&alias).yellow()
            );
            return Ok(());
        }

        // Get merged providers
        let merged_providers = ProviderStore::get_merged_providers(&config.providers).map_err(|e| anyhow!("{}", e))?;

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
            env_vars.insert("ANTHROPIC_AUTH_TOKEN".to_string(), EnvValue::String(api_key.clone()));
        }

        // Update Claude settings (with backup)
        let settings_path = ClaudeAdapter::get_settings_path()?;
        backup_file(&settings_path, "settings", 5)?;
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
        // Check if already cleared
        if config.current.is_none() {
            println!(
                "{} Already cleared all provider settings",
                style("!").yellow()
            );
            return Ok(());
        }

        // Update Claude settings with empty env (with backup)
        let settings_path = ClaudeAdapter::get_settings_path()?;
        backup_file(&settings_path, "settings", 5)?;
        let empty_env = HashMap::new();
        ClaudeAdapter::update_settings(&settings_path, empty_env)?;

        // Clear current configuration
        config.current = None;

        // Save config
        let manager = ConfigManager::new()?;
        manager.save_config(config)?;

        println!(
            "{} Cleared all provider settings",
            style("✓").green()
        );

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
    use std::collections::HashMap;

    #[test]
    fn test_env_var_merging() {
        let mut provider_env = HashMap::new();
        provider_env.insert("ANTHROPIC_BASE_URL".to_string(), "https://provider.com".to_string());
        provider_env.insert("ANTHROPIC_MODEL".to_string(), "provider-model".to_string());

        let mut config_env = HashMap::new();
        config_env.insert("ANTHROPIC_MODEL".to_string(), "config-model".to_string());
        config_env.insert("CUSTOM_VAR".to_string(), "custom-value".to_string());

        let mut merged = HashMap::new();
        merged.extend(provider_env); // Start with provider env
        merged.extend(config_env); // Override with config env
        merged.insert("ANTHROPIC_API_KEY".to_string(), "test-key".to_string()); // Add API key

        assert_eq!(merged.get("ANTHROPIC_BASE_URL"), Some(&"https://provider.com".to_string()));
        assert_eq!(merged.get("ANTHROPIC_MODEL"), Some(&"config-model".to_string())); // Config overrides provider
        assert_eq!(merged.get("CUSTOM_VAR"), Some(&"custom-value".to_string()));
        assert_eq!(merged.get("ANTHROPIC_API_KEY"), Some(&"test-key".to_string()));
    }
}
