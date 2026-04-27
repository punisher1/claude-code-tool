use crate::commands::Command;
use crate::models::{AppConfig, EnvValue};
use crate::provider_store::ProviderStore;
use anyhow::{Result, anyhow};
use console::style;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const SKIP_PERMISSIONS_ARG: &str = "--allow-dangerously-skip-permissions";

pub struct RunCommand {
    pub alias: Option<String>,
    pub proxy: Option<String>,
    pub claude_args: Vec<String>,
    pub dangerously_skip_permissions: bool,
}

impl Command for RunCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        let prepared = prepare_run(&self, config)?;
        let claude_args = claude_args_for_process(self);

        println!("{} Starting Claude Code...", style(">").cyan());
        println!("  Config: {}", style(&prepared.alias).bold());
        println!("  Provider: {}", style(&prepared.provider).cyan());

        if let Some(base_url) = prepared.env_vars.get("ANTHROPIC_BASE_URL") {
            println!("  Base URL: {}", style(base_url).dim());
        }
        if let Some(model) = prepared.env_vars.get("ANTHROPIC_MODEL") {
            println!("  Model: {}", style(model).cyan());
        }
        if !claude_args.is_empty() {
            println!("  Args: {}", style(claude_args.join(" ")).dim());
        }
        println!();

        let settings_path = write_session_settings_file(&prepared.env_vars)?;
        let mut cmd = ProcessCommand::new("claude");

        cmd.arg("--settings").arg(&settings_path);
        cmd.args(&claude_args);

        for (key, value) in &prepared.env_vars {
            cmd.env(key, value.to_string());
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = match cmd.status() {
            Ok(status) => {
                cleanup_session_settings_file(&settings_path);
                status
            }
            Err(e) => {
                cleanup_session_settings_file(&settings_path);
                return Err(if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow!("Cannot find 'claude' command. Please install Claude Code CLI.")
                } else {
                    anyhow!("Failed to start Claude Code: {}", e)
                });
            }
        };

        if status.success() {
            Ok(())
        } else {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}

fn claude_args_for_process(mut command: RunCommand) -> Vec<String> {
    if command.dangerously_skip_permissions
        && !command.claude_args.iter().any(|arg| arg == SKIP_PERMISSIONS_ARG)
    {
        command.claude_args.push(SKIP_PERMISSIONS_ARG.to_string());
    }
    command.claude_args
}

#[derive(Debug, PartialEq)]
struct PreparedRun {
    alias: String,
    provider: String,
    env_vars: HashMap<String, EnvValue>,
}

fn prepare_run(command: &RunCommand, config: &AppConfig) -> Result<PreparedRun> {
    let alias = command
        .alias
        .as_deref()
        .or(config.current.as_deref())
        .ok_or_else(|| {
            anyhow!("No configuration specified and no current configuration set. Run 'cct use <alias>' or pass an alias to 'cct run <alias>'.")
        })?;

    let config_instance = config
        .configs
        .get(alias)
        .ok_or_else(|| anyhow!("Config '{}' does not exist", alias))?;

    let merged_providers = ProviderStore::get_merged_providers(&config.providers);
    let provider = merged_providers
        .get(&config_instance.provider)
        .ok_or_else(|| {
            anyhow!(
                "Config '{}' references missing provider '{}'",
                alias,
                config_instance.provider
            )
        })?;

    let mut env_vars: HashMap<String, EnvValue> = HashMap::new();

    if let Some(provider_env) = &provider.env {
        env_vars.extend(provider_env.clone());
    }

    if let Some(config_env) = &config_instance.env {
        env_vars.extend(config_env.clone());
    }

    if let Some(api_key) = &config_instance.api_key {
        env_vars.insert(
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            EnvValue::String(api_key.clone()),
        );
    }

    if let Some(proxy_url) = &command.proxy {
        env_vars.insert(
            "HTTP_PROXY".to_string(),
            EnvValue::String(proxy_url.clone()),
        );
        env_vars.insert(
            "HTTPS_PROXY".to_string(),
            EnvValue::String(proxy_url.clone()),
        );
    }

    Ok(PreparedRun {
        alias: alias.to_string(),
        provider: config_instance.provider.clone(),
        env_vars,
    })
}

fn session_settings_json(env_vars: &HashMap<String, EnvValue>) -> serde_json::Value {
    let env: serde_json::Map<String, serde_json::Value> = env_vars
        .iter()
        .map(|(key, value)| (key.clone(), value.to_json_value()))
        .collect();

    serde_json::json!({ "env": env })
}

fn write_session_settings_file(env_vars: &HashMap<String, EnvValue>) -> Result<PathBuf> {
    let mut path = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    path.push(format!(
        "cct-session-settings-{}-{}.json",
        std::process::id(),
        timestamp
    ));

    let settings = serde_json::to_vec_pretty(&session_settings_json(env_vars))?;
    fs::write(&path, settings)?;

    Ok(path)
}

fn cleanup_session_settings_file(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        eprintln!(
            "{} Failed to delete temporary settings file {}: {}",
            style("!").yellow(),
            path.display(),
            error
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConfigInstance, Provider};
    use serde_json::json;

    #[test]
    fn test_run_command_env_merging() {
        let mut provider_env = HashMap::new();
        provider_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            EnvValue::String("https://provider.com".to_string()),
        );
        provider_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("provider-model".to_string()),
        );

        let mut config_env = HashMap::new();
        config_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("config-model".to_string()),
        );
        config_env.insert(
            "CUSTOM_VAR".to_string(),
            EnvValue::String("custom-value".to_string()),
        );

        let mut providers = HashMap::new();
        providers.insert(
            "test".to_string(),
            Provider {
                description: Some("Test Provider".to_string()),
                env: Some(provider_env),
            },
        );

        let mut configs = HashMap::new();
        configs.insert(
            "session".to_string(),
            ConfigInstance {
                provider: "test".to_string(),
                api_key: Some("test-key".to_string()),
                env: Some(config_env),
            },
        );

        let config = AppConfig {
            providers,
            configs,
            current: None,
        };
        let command = RunCommand {
            alias: Some("session".to_string()),
            proxy: None,
            claude_args: vec![],
            dangerously_skip_permissions: false,
        };
        let prepared = prepare_run(&command, &config).expect("run preparation should succeed");

        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_BASE_URL"),
            Some(&EnvValue::String("https://provider.com".to_string()))
        );
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_MODEL"),
            Some(&EnvValue::String("config-model".to_string()))
        );
        assert_eq!(
            prepared.env_vars.get("CUSTOM_VAR"),
            Some(&EnvValue::String("custom-value".to_string()))
        );
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&EnvValue::String("test-key".to_string()))
        );
    }

    #[test]
    fn test_prepare_run_allows_current_config() {
        let mut provider_env = HashMap::new();
        provider_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            EnvValue::String("https://provider.com".to_string()),
        );

        let mut providers = HashMap::new();
        providers.insert(
            "test-provider".to_string(),
            Provider {
                description: Some("Test Provider".to_string()),
                env: Some(provider_env),
            },
        );

        let mut configs = HashMap::new();
        configs.insert(
            "session".to_string(),
            ConfigInstance {
                provider: "test-provider".to_string(),
                api_key: Some("test-key".to_string()),
                env: None,
            },
        );

        let config = AppConfig {
            providers,
            configs,
            current: Some("global".to_string()),
        };

        let command = RunCommand {
            alias: Some("session".to_string()),
            proxy: None,
            claude_args: vec![],
            dangerously_skip_permissions: false,
        };

        let prepared = prepare_run(&command, &config).expect("run preparation should succeed");

        assert_eq!(prepared.provider, "test-provider");
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&EnvValue::String("test-key".to_string()))
        );
    }

    #[test]
    fn test_prepare_run_uses_requested_alias_when_current_has_same_provider() {
        let mut provider_env = HashMap::new();
        provider_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("provider-model".to_string()),
        );

        let mut target_env = HashMap::new();
        target_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("target-model".to_string()),
        );

        let mut current_env = HashMap::new();
        current_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("current-model".to_string()),
        );

        let mut providers = HashMap::new();
        providers.insert(
            "deepseek".to_string(),
            Provider {
                description: Some("DeepSeek".to_string()),
                env: Some(provider_env),
            },
        );

        let mut configs = HashMap::new();
        configs.insert(
            "target".to_string(),
            ConfigInstance {
                provider: "deepseek".to_string(),
                api_key: Some("target-key".to_string()),
                env: Some(target_env),
            },
        );
        configs.insert(
            "current".to_string(),
            ConfigInstance {
                provider: "deepseek".to_string(),
                api_key: Some("current-key".to_string()),
                env: Some(current_env),
            },
        );

        let config = AppConfig {
            providers,
            configs,
            current: Some("current".to_string()),
        };

        let command = RunCommand {
            alias: Some("target".to_string()),
            proxy: None,
            claude_args: vec![],
            dangerously_skip_permissions: false,
        };

        let prepared = prepare_run(&command, &config).expect("run preparation should succeed");

        assert_eq!(prepared.provider, "deepseek");
        assert_eq!(prepared.alias, "target");
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_MODEL"),
            Some(&EnvValue::String("target-model".to_string()))
        );
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&EnvValue::String("target-key".to_string()))
        );
    }

    #[test]
    fn test_prepare_run_defaults_to_current_config_when_alias_is_omitted() {
        let mut provider_env = HashMap::new();
        provider_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("provider-model".to_string()),
        );

        let mut current_env = HashMap::new();
        current_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("current-model".to_string()),
        );

        let mut providers = HashMap::new();
        providers.insert(
            "deepseek".to_string(),
            Provider {
                description: Some("DeepSeek".to_string()),
                env: Some(provider_env),
            },
        );

        let mut configs = HashMap::new();
        configs.insert(
            "current".to_string(),
            ConfigInstance {
                provider: "deepseek".to_string(),
                api_key: Some("current-key".to_string()),
                env: Some(current_env),
            },
        );

        let config = AppConfig {
            providers,
            configs,
            current: Some("current".to_string()),
        };

        let command = RunCommand {
            alias: None,
            proxy: None,
            claude_args: vec![],
            dangerously_skip_permissions: false,
        };

        let prepared = prepare_run(&command, &config).expect("run preparation should succeed");

        assert_eq!(prepared.alias, "current");
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_MODEL"),
            Some(&EnvValue::String("current-model".to_string()))
        );
        assert_eq!(
            prepared.env_vars.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&EnvValue::String("current-key".to_string()))
        );
    }

    #[test]
    fn test_prepare_run_requires_alias_or_current_config() {
        let config = AppConfig {
            providers: HashMap::new(),
            configs: HashMap::new(),
            current: None,
        };

        let command = RunCommand {
            alias: None,
            proxy: None,
            claude_args: vec![],
            dangerously_skip_permissions: false,
        };

        let error = prepare_run(&command, &config).expect_err("run preparation should fail");

        assert!(
            error
                .to_string()
                .contains("No configuration specified and no current configuration set")
        );
    }

    #[test]
    fn test_session_settings_json_includes_typed_env_and_auth_token() {
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            EnvValue::String("test-key".to_string()),
        );
        env_vars.insert("MAX_THINKING_TOKENS".to_string(), EnvValue::Int(1024));
        env_vars.insert("CLAUDE_CODE_USE_BEDROCK".to_string(), EnvValue::Bool(true));

        let settings = session_settings_json(&env_vars);

        assert_eq!(
            settings,
            json!({
                "env": {
                    "ANTHROPIC_AUTH_TOKEN": "test-key",
                    "MAX_THINKING_TOKENS": 1024,
                    "CLAUDE_CODE_USE_BEDROCK": true
                }
            })
        );
    }

    #[test]
    fn test_runx_adds_skip_permissions_arg() {
        let command = RunCommand {
            alias: Some("session".to_string()),
            proxy: None,
            claude_args: vec!["--print".to_string()],
            dangerously_skip_permissions: true,
        };

        assert_eq!(
            claude_args_for_process(command),
            vec![
                "--print".to_string(),
                "--allow-dangerously-skip-permissions".to_string()
            ]
        );
    }

    #[test]
    fn test_runx_does_not_duplicate_skip_permissions_arg() {
        let command = RunCommand {
            alias: Some("session".to_string()),
            proxy: None,
            claude_args: vec![
                "--allow-dangerously-skip-permissions".to_string(),
                "--print".to_string(),
            ],
            dangerously_skip_permissions: true,
        };

        assert_eq!(
            claude_args_for_process(command),
            vec![
                "--allow-dangerously-skip-permissions".to_string(),
                "--print".to_string()
            ]
        );
    }
}
