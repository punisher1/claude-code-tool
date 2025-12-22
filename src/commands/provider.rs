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
        }
    }
}

fn list_providers(config: &AppConfig) -> Result<()> {
    let merged = ProviderStore::get_merged_providers(&config.providers).map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut rows = Vec::new();

    for (name, provider) in &merged {
        let provider_type = if ProviderStore::is_builtin_provider(name) {
            "内置"
        } else {
            "自定义"
        }
        .to_string();

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

    // Check if it's a built-in provider
    if ProviderStore::is_builtin_provider(&name) {
        return Err(anyhow::anyhow!(
            "Cannot add provider '{}' because it is a built-in provider. \
             Please choose a different name for your custom provider.",
            name
        ));
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
    // Check if it's a built-in provider
    if ProviderStore::is_builtin_provider(&name) {
        return Err(anyhow::anyhow!("Cannot remove built-in provider '{}'", name));
    }

    // Check if any configs are using this provider
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

    // Remove the provider
    if config.providers.remove(&name).is_none() {
        return Err(anyhow::anyhow!("Provider '{}' not found", name));
    }

    // Save config
    let manager = ConfigManager::new()?;
    manager.save_config(config)?;

    println!("{} Provider '{}' removed successfully!", style("✓").green(), name);

    Ok(())
}
