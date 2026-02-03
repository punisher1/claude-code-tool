use crate::commands::Command;
use crate::models::{AppConfig, EnvValue};
use crate::provider_store::ProviderStore;
use anyhow::{Result, anyhow};
use console::style;
use std::collections::HashMap;
use std::process::{Command as ProcessCommand, Stdio};

pub struct RunCommand {
    pub alias: String,
    pub proxy: Option<String>,
    /// 透传给 claude 的参数
    pub claude_args: Vec<String>,
}

impl Command for RunCommand {
    fn execute(self, config: &mut AppConfig) -> Result<()> {
        // 检查是否设置了 current（settings.json 中有 env 配置）
        if config.current.is_some() {
            eprintln!(
                "{} settings.json 中已设置提供商配置 (current: {})",
                style("⚠").yellow(),
                style(config.current.as_ref().unwrap()).yellow()
            );
            eprintln!(
                "  settings.json 中的 env 优先级高于终端环境变量，"
            );
            eprintln!(
                "  请先运行 {} 清除 settings.json 中的配置",
                style("cct reset").cyan()
            );
            return Err(anyhow!("请先运行 'cct reset' 清除 settings.json 中的配置"));
        }

        // 检查配置是否存在
        let config_instance = config
            .configs
            .get(&self.alias)
            .ok_or_else(|| anyhow!("配置 '{}' 不存在", self.alias))?;

        // 获取合并后的提供商
        let merged_providers = ProviderStore::get_merged_providers(&config.providers)
            .map_err(|e| anyhow!("{}", e))?;

        // 获取提供商
        let provider = merged_providers
            .get(&config_instance.provider)
            .ok_or_else(|| {
                anyhow!(
                    "配置 '{}' 引用的提供商 '{}' 不存在",
                    self.alias,
                    config_instance.provider
                )
            })?;

        // 合并环境变量
        // 优先级: config env > provider env > api_key
        let mut env_vars: HashMap<String, EnvValue> = HashMap::new();

        // 从提供商获取环境变量
        if let Some(provider_env) = &provider.env {
            env_vars.extend(provider_env.clone());
        }

        // 从配置实例获取环境变量（覆盖提供商的）
        if let Some(config_env) = &config_instance.env {
            env_vars.extend(config_env.clone());
        }

        // 添加 API 密钥
        if let Some(api_key) = &config_instance.api_key {
            env_vars.insert(
                "ANTHROPIC_AUTH_TOKEN".to_string(),
                EnvValue::String(api_key.clone()),
            );
        }

        // 添加代理设置
        if let Some(proxy_url) = &self.proxy {
            env_vars.insert(
                "HTTP_PROXY".to_string(),
                EnvValue::String(proxy_url.clone()),
            );
            env_vars.insert(
                "HTTPS_PROXY".to_string(),
                EnvValue::String(proxy_url.clone()),
            );
        }

        // 输出启动信息
        println!(
            "{} 正在启动 Claude Code...",
            style("▶").cyan()
        );
        println!("  配置: {}", style(&self.alias).bold());
        println!("  提供商: {}", style(&config_instance.provider).cyan());

        if let Some(base_url) = env_vars.get("ANTHROPIC_BASE_URL") {
            println!("  Base URL: {}", style(base_url).dim());
        }
        if let Some(model) = env_vars.get("ANTHROPIC_MODEL") {
            println!("  Model: {}", style(model).cyan());
        }
        if !self.claude_args.is_empty() {
            println!("  参数: {}", style(self.claude_args.join(" ")).dim());
        }
        println!();

        // 构建 claude 命令
        let mut cmd = ProcessCommand::new("claude");

        // 添加透传的参数
        cmd.args(&self.claude_args);

        // 设置环境变量
        for (key, value) in &env_vars {
            cmd.env(key, value.to_string());
        }

        // 继承 stdin/stdout/stderr，让 claude 可以交互
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // 执行命令
        let status = cmd.status().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow!("找不到 'claude' 命令，请确保已安装 Claude Code CLI")
            } else {
                anyhow!("启动 Claude Code 失败: {}", e)
            }
        })?;

        // 返回 claude 的退出状态
        if status.success() {
            Ok(())
        } else {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConfigInstance, Provider};

    #[test]
    fn test_run_command_env_merging() {
        // 测试环境变量合并逻辑
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

        let provider = Provider {
            description: Some("Test Provider".to_string()),
            env: Some(provider_env),
        };

        let config_instance = ConfigInstance {
            provider: "test".to_string(),
            api_key: Some("test-key".to_string()),
            env: Some(config_env),
        };

        // 模拟合并逻辑
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

        // 验证合并结果
        assert_eq!(
            env_vars.get("ANTHROPIC_BASE_URL"),
            Some(&EnvValue::String("https://provider.com".to_string()))
        );
        // config 覆盖 provider
        assert_eq!(
            env_vars.get("ANTHROPIC_MODEL"),
            Some(&EnvValue::String("config-model".to_string()))
        );
        assert_eq!(
            env_vars.get("CUSTOM_VAR"),
            Some(&EnvValue::String("custom-value".to_string()))
        );
        assert_eq!(
            env_vars.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&EnvValue::String("test-key".to_string()))
        );
    }
}
