use crate::models::Provider;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref BUILTIN_PROVIDERS: HashMap<String, Provider> = {
        let mut providers = HashMap::new();

        // Anthropic Claude Code
        providers.insert("claude-code".to_string(), Provider {
            description:Some("Anthropic Claude Code".to_string()),
            env: None,
        });

        // DeepSeek coding
        let mut deepseek_env = HashMap::new();
        deepseek_env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            crate::models::EnvValue::String("https://api.deepseek.com/anthropic".to_string()));
        deepseek_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            crate::models::EnvValue::String("deepseek-chat".to_string()));
        deepseek_env.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            crate::models::EnvValue::String("deepseek-chat".to_string()));
        deepseek_env.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            crate::models::EnvValue::String("deepseek-chat".to_string()));
        deepseek_env.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            crate::models::EnvValue::String("deepseek-chat".to_string()));
        deepseek_env.insert(
            "API_TIMEOUT_MS".to_string(),
            crate::models::EnvValue::Int(3000000));
        deepseek_env.insert(
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            crate::models::EnvValue::Int(1));

        providers.insert("deepseek".to_string(), Provider {
            description: Some("DeepSeek API".to_string()),
            env: Some(deepseek_env),
        });

        // Kimi coding
        let mut kimi_env = HashMap::new();
        kimi_env.insert("ANTHROPIC_BASE_URL".to_string(),
            crate::models::EnvValue::String("https://api.kimi.com/coding".to_string()));
        kimi_env.insert("ANTHROPIC_MODEL".to_string(),
            crate::models::EnvValue::String("kimi-for-coding".to_string()));
        kimi_env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            crate::models::EnvValue::String("kimi-for-coding".to_string()));
        kimi_env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            crate::models::EnvValue::String("kimi-for-coding".to_string()));
        kimi_env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            crate::models::EnvValue::String("kimi-for-coding".to_string()));
        kimi_env.insert("API_TIMEOUT_MS".to_string(),
            crate::models::EnvValue::Int(3000000));
        kimi_env.insert("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            crate::models::EnvValue::Int(1));

        providers.insert("kimi-coding".to_string(), Provider {
            description: Some("Kimi Coding".to_string()),
            env: Some(kimi_env),
        });

        // Zhipu GLM coding
        let mut zhipu_env = HashMap::new();
        zhipu_env.insert("ANTHROPIC_BASE_URL".to_string(),
            crate::models::EnvValue::String("https://open.bigmodel.cn/api/anthropic".to_string()));
        zhipu_env.insert("ANTHROPIC_MODEL".to_string(),
            crate::models::EnvValue::String("glm-4.6".to_string()));
        zhipu_env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            crate::models::EnvValue::String("glm-4.5-air".to_string()));
        zhipu_env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            crate::models::EnvValue::String("glm-4.6".to_string()));
        zhipu_env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            crate::models::EnvValue::String("glm-4.6".to_string()));
        zhipu_env.insert("API_TIMEOUT_MS".to_string(),
            crate::models::EnvValue::Int(3000000));
        zhipu_env.insert("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            crate::models::EnvValue::Int(1));

        providers.insert("zhipu".to_string(), Provider {
            description: Some("Zhipu GLM Coding".to_string()),
            env: Some(zhipu_env),
        });

        // xiaomi mimo coding
        let mut zhipu_env = HashMap::new();
        zhipu_env.insert("ANTHROPIC_BASE_URL".to_string(),
            crate::models::EnvValue::String("https://api.xiaomimimo.com/anthropic".to_string()));
        zhipu_env.insert("ANTHROPIC_MODEL".to_string(),
            crate::models::EnvValue::String("mimo-v2-flash".to_string()));
        zhipu_env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            crate::models::EnvValue::String("mimo-v2-flash".to_string()));
        zhipu_env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            crate::models::EnvValue::String("mimo-v2-flash".to_string()));
        zhipu_env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            crate::models::EnvValue::String("mimo-v2-flash".to_string()));
        zhipu_env.insert("API_TIMEOUT_MS".to_string(),
            crate::models::EnvValue::Int(3000000));
        zhipu_env.insert("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            crate::models::EnvValue::Int(1));

        providers.insert("xiaomi-mimo".to_string(), Provider {
            description: Some("Xiaomi Mimo Coding".to_string()),
            env: Some(zhipu_env),
        });

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
