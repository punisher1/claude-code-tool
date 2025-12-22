use crate::commands::Command;
use crate::config_manager::ConfigManager;
use crate::models::{AppConfig, ConfigInstance};
use crate::provider_store::ProviderStore;
use anyhow::{Result, anyhow};
use console::style;
use tabled::{Table, Tabled, settings::{Style, Color}, settings::object::Rows};

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

#[derive(Tabled)]
struct ConfigRow {
    #[tabled(rename = "别名")]
    alias: String,
    #[tabled(rename = "提供商")]
    provider: String,
    #[tabled(rename = "API密钥")]
    api_key: String,
}

pub struct ListCommand;

impl Command for ListCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        if config.configs.is_empty() {
            println!("{}", style("暂无配置").dim());
            return Ok(());
        }

        // 收集所有配置并按别名排序
        let mut rows = Vec::new();
        let mut current_row_idx = None;

        // 先收集并排序
        let mut sorted_configs: Vec<_> = config.configs.iter().collect();
        sorted_configs.sort_by(|a, b| a.0.cmp(b.0));

        for (index, (alias, config_instance)) in sorted_configs.iter().enumerate() {
            rows.push(ConfigRow {
                alias: alias.to_string(),
                provider: config_instance.provider.clone(),
                api_key: config_instance.api_key.clone(),
            });

            // 记录当前激活的配置行（+1 因为第1行是表头）
            if config.current.as_ref() == Some(*alias) {
                current_row_idx = Some(index + 1);
            }
        }

        // 构建表格字符串
        let table_str = if let Some(row_idx) = current_row_idx {
            // 有当前激活配置，需要高亮
            use tabled::settings::style::BorderColor;
            Table::new(&rows)
                .with(Style::modern())
                .modify(
                    Rows::new(row_idx..row_idx+1),
                    BorderColor::filled(Color::FG_GREEN)
                )
                .to_string()
        } else {
            // 没有当前激活配置
            Table::new(&rows)
                .with(Style::modern())
                .to_string()
        };

        println!("{}", table_str);

        Ok(())
    }
}
