use crate::claude_adapter::ClaudeAdapter;
use crate::commands::Command;
use crate::config_manager::ConfigManager;
use crate::models::AppConfig;
use crate::provider_store::ProviderStore;
use anyhow::{Result, anyhow};
use console::style;
use dialoguer::{Select, theme::ColorfulTheme};
use std::collections::HashMap;
use crate::models::EnvValue;


pub struct UseCommand {
    pub alias: Option<String>,
}

impl Command for UseCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        let alias = match self.alias {
            Some(alias) => alias,
            None => {
                // Interactive selection mode
                if config.configs.is_empty() {
                    return Err(anyhow!("No configurations found. Add a configuration first with 'cct add'."));
                }

                // Prepare items for selection
                let items: Vec<_> = config.configs
                    .keys()
                    .cloned()
                    .collect();

                if items.is_empty() {
                    return Err(anyhow!("No configurations available"));
                }

                // Find current selection index
                let current_index = config.current.as_ref()
                    .and_then(|current| items.iter().position(|item| item == current))
                    .unwrap_or(0);

                // Show selection dialog
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a configuration to use")
                    .items(&items)
                    .default(current_index)
                    .interact();

                match selection {
                    Ok(index) => items[index].clone(),
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

        // Ensure API key is set (use ANTHROPIC_AUTH_TOKEN instead of ANTHROPIC_API_KEY)
        env_vars.insert("ANTHROPIC_AUTH_TOKEN".to_string(), EnvValue::String(config_instance.api_key.clone()));

        // Update Claude settings
        let settings_path = ClaudeAdapter::get_settings_path()?;
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
