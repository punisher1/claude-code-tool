use crate::models::Provider;
use lazy_static::lazy_static;
use std::collections::HashMap;

/// 结构化的提供商定义
struct ProviderDef {
    name: &'static str,
    description: &'static str,
    base_url: &'static str,
    model: &'static str,
    haiku_model: &'static str,
    sonnet_model: &'static str,
    opus_model: &'static str,
    env: Option<HashMap<&'static str, crate::models::EnvValue>>,
}

lazy_static! {
    pub static ref BUILTIN_PROVIDERS: HashMap<String, Provider> = {
        let mut providers = HashMap::new();

        // 定义所有需要环境变量的提供商配置
        let provider_configs = vec![
            ProviderDef {
                name: "deepseek",
                description: "DeepSeek API",
                base_url: "https://api.deepseek.com/anthropic",
                model: "deepseek-v4-pro",
                haiku_model: "deepseek-v4-flash",
                sonnet_model: "deepseek-v4-pro",
                opus_model: "deepseek-v4-pro",
                env: Some({
                    let mut env = HashMap::new();
                    env.insert(
                        "CLAUDE_CODE_EFFORT_LEVEL",
                        crate::models::EnvValue::String("max".to_string()),
                    );
                    env
                }),
            },
            ProviderDef {
                name: "kimi-coding",
                description: "Kimi Coding",
                base_url: "https://api.kimi.com/coding",
                model: "kimi-for-coding",
                haiku_model: "kimi-for-coding",
                sonnet_model: "kimi-for-coding",
                opus_model: "kimi-for-coding",
                env: None,
            },
            ProviderDef {
                name: "zhipu",
                description: "Zhipu GLM Coding",
                base_url: "https://open.bigmodel.cn/api/anthropic",
                model: "glm-5.1",
                haiku_model: "glm-4.5-air",
                sonnet_model: "glm-5.1",
                opus_model: "glm-5.1",
                env: None,
            },
            ProviderDef {
                name: "xiaomi-mimo",
                description: "Xiaomi Mimo Coding",
                base_url: "https://api.xiaomimimo.com/anthropic",
                model: "mimo-v2.5-pro",
                haiku_model: "mimo-v2.5-pro",
                sonnet_model: "mimo-v2.5-pro",
                opus_model: "mimo-v2.5-pro",
                env: None,
            },
            ProviderDef {
                name: "minimaxi-m2",
                description: "Minimax M2 Coding",
                base_url: "https://api.minimaxi.com/anthropic",
                model: "MiniMax-M2.7",
                haiku_model: "MiniMax-M2.7",
                sonnet_model: "MiniMax-M2.7",
                opus_model: "MiniMax-M2.7",
                env: None,
            },
        ];

        // 添加 Claude Code (无环境变量)
        providers.insert("claude-code".to_string(), Provider {
            description: Some("Anthropic Claude Code".to_string()),
            env: None,
        });

        // 添加 Claude Console
        providers.insert("claude-console".to_string(), Provider {
            description: Some("Anthropic Claude Console".to_string()),
            env: Some({
                let mut env_map = HashMap::new();
                env_map.insert(
                    "ANTHROPIC_BASE_URL".to_string(),
                    crate::models::EnvValue::String("https://api.anthropic.com".to_string())
                );
                env_map
            })
        });

        // 使用循环处理所有提供商
        for config in provider_configs {
            let mut env_map = HashMap::new();

            // 添加通用的环境变量
            env_map.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                crate::models::EnvValue::String(config.base_url.to_string())
            );
            env_map.insert(
                "ANTHROPIC_MODEL".to_string(),
                crate::models::EnvValue::String(config.model.to_string())
            );
            env_map.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                crate::models::EnvValue::String(config.haiku_model.to_string())
            );
            env_map.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                crate::models::EnvValue::String(config.sonnet_model.to_string())
            );
            env_map.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                crate::models::EnvValue::String(config.opus_model.to_string())
            );
            env_map.insert(
                "API_TIMEOUT_MS".to_string(),
                crate::models::EnvValue::Int(3000000)
            );
            env_map.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                crate::models::EnvValue::Int(1)
            );

            // 如果有额外的环境变量，添加它们
            if let Some(extra_env) = &config.env {
                for (key, value) in extra_env {
                    env_map.insert(key.to_string(), value.clone());
                }
            }

            providers.insert(config.name.to_string(), Provider {
                description: Some(config.description.to_string()),
                env: Some(env_map),
            });
        }

        providers
    };
}

pub struct ProviderStore;

impl ProviderStore {
    /// 获取内置 provider 的克隆
    pub fn get_builtin_providers() -> HashMap<String, Provider> {
        BUILTIN_PROVIDERS.clone()
    }

    /// 合并 providers，按优先级：内置 → providers.toml → config.toml providers
    /// 后面的会覆盖前面的同名 provider
    pub fn get_merged_providers(
        config_providers: &HashMap<String, Provider>,
    ) -> HashMap<String, Provider> {
        // 加载 providers.toml 中的 provider
        let file_providers = crate::config_manager::ConfigManager::load_providers()
            .unwrap_or_else(|_| HashMap::new());

        Self::get_merged_providers_with_override(config_providers, &file_providers)
    }

    /// 合并 providers（带 providers.toml 覆盖），用于测试
    pub fn get_merged_providers_with_override(
        config_providers: &HashMap<String, Provider>,
        file_providers: &HashMap<String, Provider>,
    ) -> HashMap<String, Provider> {
        // 1. 从内置 provider 开始
        let mut merged = BUILTIN_PROVIDERS.clone();

        // 2. 应用 providers.toml 覆盖
        for (name, provider) in file_providers {
            merged.insert(name.clone(), provider.clone());
        }

        // 3. 应用 config.toml providers 覆盖
        for (name, provider) in config_providers {
            merged.insert(name.clone(), provider.clone());
        }

        merged
    }

    /// 判断 provider 是否为内置 provider
    pub fn is_builtin_provider(name: &str) -> bool {
        BUILTIN_PROVIDERS.contains_key(name)
    }

    /// 判断 provider 来源类型
    /// 返回: "内置", "providers.toml", "config.toml"
    pub fn get_provider_source(
        name: &str,
        file_providers: &HashMap<String, Provider>,
        config_providers: &HashMap<String, Provider>,
    ) -> &'static str {
        if config_providers.contains_key(name) {
            "config.toml"
        } else if file_providers.contains_key(name) {
            "providers.toml"
        } else if BUILTIN_PROVIDERS.contains_key(name) {
            "内置"
        } else {
            "未知"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EnvValue;

    #[test]
    fn test_deepseek_builtin_models_follow_current_api_docs() {
        let providers = ProviderStore::get_builtin_providers();
        let deepseek = providers.get("deepseek").unwrap();
        let env = deepseek.env.as_ref().unwrap();

        assert_eq!(
            env.get("ANTHROPIC_MODEL"),
            Some(&EnvValue::String("deepseek-v4-pro".to_string()))
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
            Some(&EnvValue::String("deepseek-v4-flash".to_string()))
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL"),
            Some(&EnvValue::String("deepseek-v4-pro".to_string()))
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            Some(&EnvValue::String("deepseek-v4-pro".to_string()))
        );
        assert_eq!(
            env.get("CLAUDE_CODE_EFFORT_LEVEL"),
            Some(&EnvValue::String("max".to_string()))
        );
    }

    #[test]
    fn test_provider_merge() {
        let mut config_providers = HashMap::new();

        // Add a custom provider
        let mut custom_env = HashMap::new();
        custom_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            EnvValue::String("https://custom.api.com".to_string()),
        );
        let custom_provider = Provider {
            description: Some("Custom Provider".to_string()),
            env: Some(custom_env),
        };
        config_providers.insert("custom".to_string(), custom_provider.clone());

        let merged =
            ProviderStore::get_merged_providers_with_override(&config_providers, &HashMap::new());

        // Verify built-in providers exist
        assert!(merged.contains_key("claude-code"));
        assert!(merged.contains_key("kimi-coding"));
        assert!(merged.contains_key("zhipu"));

        // Verify custom provider exists
        assert!(merged.contains_key("custom"));
        if let Some(EnvValue::String(url)) = merged
            .get("custom")
            .unwrap()
            .env
            .as_ref()
            .unwrap()
            .get("ANTHROPIC_BASE_URL")
        {
            assert_eq!(url, "https://custom.api.com");
        } else {
            panic!("Expected string value for ANTHROPIC_BASE_URL");
        }
    }

    #[test]
    fn test_can_override_builtin_provider_with_file() {
        // 测试 providers.toml 可以覆盖内置 provider
        let mut file_providers = HashMap::new();

        let mut override_env = HashMap::new();
        override_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            EnvValue::String("https://new.deepseek.com".to_string()),
        );
        let override_provider = Provider {
            description: Some("Updated DeepSeek".to_string()),
            env: Some(override_env),
        };
        file_providers.insert("deepseek".to_string(), override_provider.clone());

        let merged =
            ProviderStore::get_merged_providers_with_override(&HashMap::new(), &file_providers);

        // 验证覆盖生效
        assert!(merged.contains_key("deepseek"));
        let merged_provider = merged.get("deepseek").unwrap();
        assert_eq!(
            merged_provider.description,
            Some("Updated DeepSeek".to_string())
        );
    }

    #[test]
    fn test_override_priority() {
        // 测试优先级：config.toml > providers.toml > 内置

        // providers.toml 覆盖
        let mut file_providers = HashMap::new();
        file_providers.insert(
            "deepseek".to_string(),
            Provider {
                description: Some("File Override".to_string()),
                env: None,
            },
        );

        // config.toml 覆盖
        let mut config_providers = HashMap::new();
        config_providers.insert(
            "deepseek".to_string(),
            Provider {
                description: Some("Config Override".to_string()),
                env: None,
            },
        );

        let merged =
            ProviderStore::get_merged_providers_with_override(&config_providers, &file_providers);

        // config.toml 优先级最高
        assert_eq!(
            merged.get("deepseek").unwrap().description,
            Some("Config Override".to_string())
        );

        // 只有 file_providers 时
        let merged_file_only =
            ProviderStore::get_merged_providers_with_override(&HashMap::new(), &file_providers);
        assert_eq!(
            merged_file_only.get("deepseek").unwrap().description,
            Some("File Override".to_string())
        );
    }

    #[test]
    fn test_builtin_provider_check() {
        assert!(ProviderStore::is_builtin_provider("claude-code"));
        assert!(ProviderStore::is_builtin_provider("deepseek"));
        assert!(ProviderStore::is_builtin_provider("kimi-coding"));
        assert!(ProviderStore::is_builtin_provider("zhipu"));
        assert!(!ProviderStore::is_builtin_provider("custom"));
        assert!(!ProviderStore::is_builtin_provider("unknown"));
    }

    #[test]
    fn test_get_provider_source() {
        let mut file_providers = HashMap::new();
        file_providers.insert(
            "deepseek".to_string(),
            Provider {
                description: Some("File".to_string()),
                env: None,
            },
        );

        let mut config_providers = HashMap::new();
        config_providers.insert(
            "kimi-coding".to_string(),
            Provider {
                description: Some("Config".to_string()),
                env: None,
            },
        );

        // 内置 provider
        assert_eq!(
            ProviderStore::get_provider_source("claude-code", &file_providers, &config_providers),
            "内置"
        );

        // providers.toml 覆盖的内置 provider
        assert_eq!(
            ProviderStore::get_provider_source("deepseek", &file_providers, &config_providers),
            "providers.toml"
        );

        // config.toml 覆盖的内置 provider
        assert_eq!(
            ProviderStore::get_provider_source("kimi-coding", &file_providers, &config_providers),
            "config.toml"
        );

        // 未知 provider
        assert_eq!(
            ProviderStore::get_provider_source("unknown", &file_providers, &config_providers),
            "未知"
        );
    }
}
