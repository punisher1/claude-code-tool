use crate::commands::Command;
use crate::config_manager::ConfigManager;
use crate::models::{AppConfig, ConfigInstance};
use crate::provider_store::ProviderStore;
use anyhow::{Result, anyhow};
use console::style;

pub struct AddCommand {
    pub alias: String,
    pub provider: String,
    pub api_key: String,
}

impl Command for AddCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        // Check if alias already exists
        if config.configs.contains_key(&self.alias) {
            return Err(anyhow!("Configuration '{}' already exists", self.alias));
        }

        // Get merged providers to validate
        let merged_providers = ProviderStore::get_merged_providers(&config.providers).map_err(|e| anyhow!("{}", e))?;

        // Check if provider exists
        if !merged_providers.contains_key(&self.provider) {
            return Err(anyhow!("Provider '{}' not found", self.provider));
        }

        // Create new config instance
        let config_instance = ConfigInstance {
            provider: self.provider.clone(),
            api_key: self.api_key.clone(),
            env: None,
        };

        // Add to config
        config.configs.insert(self.alias.clone(), config_instance);

        // Save config
        let manager = ConfigManager::new()?;
        manager.save_config(config)?;

        println!(
            "{} Configuration '{}' added successfully!",
            style("✓").green(),
            self.alias
        );

        Ok(())
    }
}
