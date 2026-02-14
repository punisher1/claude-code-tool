use crate::commands::Command;
use crate::config_manager::ConfigManager;
use crate::models::{AppConfig, Provider};
use crate::provider_store::ProviderStore;
use crate::utils::validate_provider_name;
use anyhow::Result;
use console::style;
use dialoguer::Input;
use std::collections::HashMap;
use tabled::settings::Style;
use tabled::{Table, Tabled};

pub enum ProviderCommand {
    List,
    Add { name: Option<String> },
    Remove { name: String },
    Init,
}

#[derive(Tabled)]
struct ProviderRow {
    #[tabled(rename = "名称")]
    name: String,
    #[tabled(rename = "类型")]
    provider_type: String,
    #[tabled(rename = "描述")]
    description: String,
    #[tabled(rename = "基础URL")]
    base_url: String,
    #[tabled(rename = "默认模型")]
    model: String,
    #[tabled(rename = "Haiku模型")]
    haiku_model: String,
    #[tabled(rename = "Sonnet模型")]
    sonnet_model: String,
    #[tabled(rename = "Opus模型")]
    opus_model: String,
}

impl Command for ProviderCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        match self {
            ProviderCommand::List => list_providers(config),
            ProviderCommand::Add { name } => add_provider(config, name),
            ProviderCommand::Remove { name } => remove_provider(config, name),
            ProviderCommand::Init => init_providers(),
        }
    }
}

fn list_providers(config: &AppConfig) -> Result<()> {
    // 加载 providers.toml
    let file_providers = ConfigManager::load_providers().unwrap_or_else(|_| HashMap::new());

    let merged = ProviderStore::get_merged_providers_with_override(&config.providers, &file_providers);

    let mut rows = Vec::new();

    for (name, provider) in &merged {
        let provider_type = ProviderStore::get_provider_source(name, &file_providers, &config.providers).to_string();

        let description = provider.description.clone().unwrap_or_else(|| "-".to_string());

        let base_url = if let Some(env) = &provider.env {
            env.get("ANTHROPIC_BASE_URL")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string())
        } else {
            "".to_string()
        };

        let model = if let Some(env) = &provider.env {
            env.get("ANTHROPIC_MODEL")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string())
        } else {
            "".to_string()
        };

        let haiku_model = if let Some(env) = &provider.env {
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string())
        } else {
            "".to_string()
        };

        let sonnet_model = if let Some(env) = &provider.env {
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string())
        } else {
            "".to_string()
        };

        let opus_model = if let Some(env) = &provider.env {
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string())
        } else {
            "".to_string()
        };

        rows.push(ProviderRow {
            name: name.clone(),
            provider_type,
            description,
            base_url,
            model,
            haiku_model,
            sonnet_model,
            opus_model,
        });
    }

    if rows.is_empty() {
        println!("{}", style("暂无提供商配置").dim());
        return Ok(());
    }

    let table = Table::new(rows).with(Style::modern()).to_string();

    println!("{}", table);

    Ok(())
}

fn add_provider(config: &mut AppConfig, name: Option<String>) -> Result<()> {
    let name = match name {
        Some(n) => n,
        None => match Input::<String>::new().with_prompt("Provider name").interact_text() {
            Ok(n) => n,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read provider name: {}", e));
            }
        },
    };

    validate_provider_name(&name)?;

    // 检查是否为内置 provider，提示用户将创建覆盖配置
    if ProviderStore::is_builtin_provider(&name) {
        println!(
            "{} 内置 provider '{}' 将被覆盖。",
            style("!").yellow(),
            name
        );
    }

    if config.providers.contains_key(&name) {
        println!(
            "{} Provider '{}' already exists. Updating...",
            style("!").yellow(),
            name
        );
    }

    // Get description
    let description: Option<String> = match Input::<String>::new()
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact_text()
    {
        Ok(input) if input.trim().is_empty() => None,
        Ok(input) => Some(input),
        Err(e) => {
            eprintln!(
                "Warning: Failed to read input interactively ({}), using empty description",
                e
            );
            None
        }
    };

    // Get base URL
    let base_url: String = match Input::new()
        .with_prompt("Base URL")
        .default("https://api.example.com".into())
        .interact_text()
    {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Warning: Failed to read input interactively ({}), using default", e);
            "".to_string()
        }
    };

    // Get model
    let model: String = match Input::<String>::new().with_prompt("Default Model").interact_text() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Failed to read input interactively ({}), using default", e);
            "".to_string()
        }
    };

    // Get Haiku model
    let haiku_model: String = match Input::<String>::new()
        .with_prompt("Default Haiku Model")
        .default(model.to_string())
        .interact_text()
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Failed to read input interactively ({}), using default", e);
            "".to_string()
        }
    };

    // Get Sonnet model
    let sonnet_model: String = match Input::<String>::new()
        .with_prompt("Default Sonnet Model")
        .default(model.to_string())
        .interact_text()
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Failed to read input interactively ({}), using default", e);
            "".to_string()
        }
    };

    // Get Opus model
    let opus_model: String = match Input::<String>::new()
        .with_prompt("Default Opus Model")
        .default(model.to_string())
        .interact_text()
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Failed to read input interactively ({}), using default", e);
            "".to_string()
        }
    };

    // Create env map
    let mut env = HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        crate::models::EnvValue::String(base_url),
    );
    env.insert("ANTHROPIC_MODEL".to_string(), crate::models::EnvValue::String(model));
    env.insert(
        "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
        crate::models::EnvValue::String(haiku_model),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
        crate::models::EnvValue::String(sonnet_model),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
        crate::models::EnvValue::String(opus_model),
    );

    let provider = Provider {
        description,
        env: Some(env),
    };

    config.providers.insert(name.clone(), provider);

    // Save config
    let manager = ConfigManager::new()?;
    manager.save_config(config)?;

    println!("{} Provider '{}' added successfully!", style("✓").green(), name);

    Ok(())
}

fn remove_provider(config: &mut AppConfig, name: String) -> Result<()> {
    // 检查是否在 config.toml 中
    if config.providers.contains_key(&name) {
        // 检查是否有配置使用此 provider
        let mut used_by = Vec::new();
        for (config_name, config_instance) in &config.configs {
            if config_instance.provider == name {
                used_by.push(config_name.clone());
            }
        }

        if !used_by.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot remove provider '{}' because it's being used by configurations: {}",
                name,
                used_by.join(", ")
            ));
        }

        config.providers.remove(&name);

        let manager = ConfigManager::new()?;
        manager.save_config(config)?;

        println!("{} Provider '{}' removed from config.toml successfully!", style("✓").green(), name);
        return Ok(());
    }

    // 检查是否在 providers.toml 中
    let file_providers = ConfigManager::load_providers().unwrap_or_else(|_| HashMap::new());
    if file_providers.contains_key(&name) {
        // 从 providers.toml 中移除
        let mut updated = file_providers.clone();
        updated.remove(&name);
        ConfigManager::save_providers(&updated)?;

        println!("{} Provider '{}' removed from providers.toml successfully!", style("✓").green(), name);
        return Ok(());
    }

    // 检查是否为内置 provider（不能删除，只能通过覆盖）
    if ProviderStore::is_builtin_provider(&name) {
        return Err(anyhow::anyhow!(
            "Cannot remove built-in provider '{}'. You can override it by adding a custom provider with the same name.",
            name
        ));
    }

    Err(anyhow::anyhow!("Provider '{}' not found", name))
}

/// 导出内置 provider 到 ~/.cct/providers.toml
fn init_providers() -> Result<()> {
    let providers_path = ConfigManager::get_providers_path()?;

    // 检查文件是否已存在
    if providers_path.exists() {
        println!(
            "{} providers.toml 已存在于: {:?}",
            style("!").yellow(),
            providers_path
        );
        println!("如果继续，将覆盖现有文件。");
    }

    // 获取内置 providers
    let builtin_providers = ProviderStore::get_builtin_providers();

    // 保存到文件
    ConfigManager::save_providers(&builtin_providers)?;

    println!(
        "{} 内置 providers 已导出到: {:?}",
        style("✓").green(),
        providers_path
    );
    println!("共导出 {} 个 provider 配置。", builtin_providers.len());

    Ok(())
}
