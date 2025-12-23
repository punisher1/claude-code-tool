use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom enum that can be either a String, Bool, or Int
#[derive(Debug, Clone, PartialEq)]
pub enum EnvValue {
    String(String),
    Bool(bool),
    Int(i64),
}

impl std::fmt::Display for EnvValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvValue::String(s) => write!(f, "{}", s),
            EnvValue::Bool(b) => write!(f, "{}", b),
            EnvValue::Int(i) => write!(f, "{}", i),
        }
    }
}

impl Serialize for EnvValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EnvValue::String(s) => serializer.serialize_str(s),
            EnvValue::Bool(b) => serializer.serialize_bool(*b),
            EnvValue::Int(i) => serializer.serialize_i64(*i),
        }
    }
}

impl<'de> Deserialize<'de> for EnvValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use std::fmt;

        struct EnvValueVisitor;

        impl<'de> serde::de::Visitor<'de> for EnvValueVisitor {
            type Value = EnvValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string, boolean, or integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<EnvValue, E>
            where
                E: Error,
            {
                Ok(EnvValue::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<EnvValue, E>
            where
                E: Error,
            {
                Ok(EnvValue::String(value))
            }

            fn visit_bool<E>(self, value: bool) -> Result<EnvValue, E>
            where
                E: Error,
            {
                Ok(EnvValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<EnvValue, E>
            where
                E: Error,
            {
                Ok(EnvValue::Int(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<EnvValue, E>
            where
                E: Error,
            {
                Ok(EnvValue::Int(value as i64))
            }
        }

        deserializer.deserialize_any(EnvValueVisitor)
    }
}

impl EnvValue {

    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            EnvValue::String(s) => serde_json::Value::String(s.clone()),
            EnvValue::Bool(b) => serde_json::Value::Bool(*b),
            EnvValue::Int(i) => serde_json::Value::Number((*i).into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Provider {
    pub description: Option<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, EnvValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigInstance {
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, EnvValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    pub providers: HashMap<String, Provider>,
    pub configs: HashMap<String, ConfigInstance>,
    pub current: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let mut env: HashMap<String, EnvValue> = HashMap::new();
        env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            EnvValue::String("https://api.anthropic.com".to_string()),
        );
        env.insert(
            "ANTHROPIC_MODEL".to_string(),
            EnvValue::String("claude-3-sonnet-20240229".to_string()),
        );

        let provider = Provider {
            description: Some("Anthropic Claude".to_string()),
            env: Some(env),
        };

        let mut providers = HashMap::new();
        providers.insert("anthropic".to_string(), provider.clone());

        let config_instance = ConfigInstance {
            provider: "anthropic".to_string(),
            api_key: Some("test-key".to_string()),
            env: None,
        };

        let mut configs = HashMap::new();
        configs.insert("my-config".to_string(), config_instance);

        let app_config = AppConfig {
            providers,
            configs,
            current: Some("my-config".to_string()),
        };

        // Test TOML serialization
        let toml_str = toml::to_string(&app_config).unwrap();
        let deserialized: AppConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(app_config, deserialized);
        assert_eq!(deserialized.current, Some("my-config".to_string()));
        assert!(deserialized.providers.contains_key("anthropic"));
    }

    #[test]
    fn test_env_value_int_serialization() {
        // Test integer serialization
        let int_value = EnvValue::Int(1);
        assert_eq!(int_value.to_json_value(), serde_json::Value::Number(1.into()));

        // Test that integer is properly serialized in TOML
        let mut env: HashMap<String, EnvValue> = HashMap::new();
        env.insert("TEST_INT".to_string(), EnvValue::Int(42));
        env.insert("TEST_STRING".to_string(), EnvValue::String("hello".to_string()));
        env.insert("TEST_BOOL".to_string(), EnvValue::Bool(true));

        let toml_str = toml::to_string(&env).unwrap();
        println!("TOML output: {}", toml_str);

        // Verify the TOML contains integer value (not string)
        assert!(toml_str.contains("TEST_INT = 42")); // Should be raw number, not "42"
        assert!(toml_str.contains("TEST_STRING = \"hello\"")); // String should be quoted
        assert!(toml_str.contains("TEST_BOOL = true")); // Bool should be true/false
    }
}
