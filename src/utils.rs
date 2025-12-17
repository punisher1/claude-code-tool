use anyhow::Result;

pub fn validate_provider_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Provider name cannot be empty"));
    }

    if name.contains(' ') {
        return Err(anyhow::anyhow!("Provider name cannot contain spaces"));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow::anyhow!(
            "Provider name can only contain letters, numbers, hyphens, and underscores"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_provider_name() {
        assert!(validate_provider_name("valid-name").is_ok());
        assert!(validate_provider_name("valid_name").is_ok());
        assert!(validate_provider_name("valid123").is_ok());

        assert!(validate_provider_name("").is_err());
        assert!(validate_provider_name("invalid name").is_err());
        assert!(validate_provider_name("invalid@name").is_err());
        assert!(validate_provider_name("invalid.name").is_err());
    }
}