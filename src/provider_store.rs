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
                model: "deepseek-chat",
                haiku_model: "deepseek-chat",
                sonnet_model: "deepseek-chat",
                opus_model: "deepseek-chat",
                env: None,
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
                model: "glm-4.7",
                haiku_model: "glm-4.5-air",
                sonnet_model: "glm-4.7",
                opus_model: "glm-4.7",
                env: None,
            },
            ProviderDef {
                name: "xiaomi-mimo",
                description: "Xiaomi Mimo Coding",
                base_url: "https://api.xiaomimimo.com/anthropic",
                model: "mimo-v2-flash",
                haiku_model: "mimo-v2-flash",
                sonnet_model: "mimo-v2-flash",
                opus_model: "mimo-v2-flash",
                env: None,
            },
            ProviderDef {
                name: "minimaxi-m2",
                description: "Minimax M2 Coding",
                base_url: "https://api.minimaxi.com/anthropic",
                model: "MiniMax-M2.1",
                haiku_model: "MiniMax-M2.1",
                sonnet_model: "MiniMax-M2.1",
                opus_model: "MiniMax-M2.1",
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
    pub fn get_merged_providers(
        custom_providers: &HashMap<String, Provider>,
    ) -> Result<HashMap<String, Provider>, String> {
        // First check if any custom provider conflicts with built-in providers
        for name in custom_providers.keys() {
            if BUILTIN_PROVIDERS.contains_key(name) {
                return Err(format!(
                    "Custom provider '{}' cannot have the same name as a built-in provider. \
                     Please choose a different name for your custom provider.",
                    name
                ));
            }
        }

        let mut merged = BUILTIN_PROVIDERS.clone();

        // Merge custom providers (they don't conflict with built-in ones)
        for (name, provider) in custom_providers {
            merged.insert(name.clone(), provider.clone());
        }

        Ok(merged)
    }

    pub fn is_builtin_provider(name: &str) -> bool {
        BUILTIN_PROVIDERS.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EnvValue;

    #[test]
    fn test_provider_merge() {
        let mut custom_providers = HashMap::new();

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
        custom_providers.insert("custom".to_string(), custom_provider.clone());

        let merged = ProviderStore::get_merged_providers(&custom_providers).unwrap();

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
    fn test_cannot_override_builtin_provider() {
        let mut custom_providers = HashMap::new();

        // Try to override a built-in provider
        let mut override_env = HashMap::new();
        override_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            EnvValue::String("https://new.deepseek.com".to_string()),
        );
        let override_provider = Provider {
            description: Some("Updated DeepSeek".to_string()),
            env: Some(override_env),
        };
        custom_providers.insert("deepseek".to_string(), override_provider.clone());

        // This should return an error
        let result = ProviderStore::get_merged_providers(&custom_providers);
        assert!(result.is_err());

        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Custom provider 'deepseek' cannot have the same name as a built-in provider"));
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
}
