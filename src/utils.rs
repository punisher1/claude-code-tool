use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::Path;

/// 备份指定文件，按照 prefix_YYYYMMDDHHMMSS.ext 格式命名
/// 最多保留 max_backups 个备份文件
pub fn backup_file(file_path: &Path, prefix: &str, max_backups: usize) -> Result<()> {
    if !file_path.exists() {
        return Ok(()); // 文件不存在时无需备份
    }

    let parent_dir = file_path
        .parent()
        .context("Failed to get parent directory")?;

    // 获取文件扩展名
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // 生成备份文件名：prefix_YYYYMMDDHHMMSS.ext
    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let backup_name = if extension.is_empty() {
        format!("{}_{}", prefix, timestamp)
    } else {
        format!("{}_{}.{}", prefix, timestamp, extension)
    };
    let backup_path = parent_dir.join(&backup_name);

    // 复制文件到备份
    fs::copy(file_path, &backup_path)
        .with_context(|| format!("Failed to create backup: {:?}", backup_path))?;

    // 清理旧的备份文件，只保留 max_backups 个
    cleanup_old_backups(parent_dir, prefix, extension, max_backups)?;

    Ok(())
}

/// 清理旧的备份文件，只保留最新的 max_backups 个
fn cleanup_old_backups(
    dir: &Path,
    prefix: &str,
    extension: &str,
    max_backups: usize,
) -> Result<()> {
    // 收集匹配的备份文件
    let pattern = format!("{}_", prefix);

    let mut backups: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {:?}", dir))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            // 匹配 prefix_YYYYMMDDHHMMSS.ext 格式
            if !file_name.starts_with(&pattern) {
                return false;
            }

            // 检查扩展名
            if extension.is_empty() {
                !file_name.contains('.')
            } else {
                file_name.ends_with(&format!(".{}", extension))
            }
        })
        .collect();

    // 如果备份数量未超过限制，无需清理
    if backups.len() <= max_backups {
        return Ok(());
    }

    // 按文件名排序（因为时间戳在文件名中，所以字母序等于时间序）
    backups.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

    // 删除超出限制的旧备份
    for backup in backups.iter().skip(max_backups) {
        fs::remove_file(backup.path())
            .with_context(|| format!("Failed to remove old backup: {:?}", backup.path()))?;
    }

    Ok(())
}

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
    use tempfile::TempDir;

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

    #[test]
    fn test_backup_file_creates_backup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("config.toml");

        // 创建原始文件
        fs::write(&file_path, "original content")?;

        // 执行备份
        backup_file(&file_path, "config", 5)?;

        // 验证备份文件存在
        let entries: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("config_") && name.ends_with(".toml")
            })
            .collect();

        assert_eq!(entries.len(), 1);

        // 验证备份内容
        let backup_content = fs::read_to_string(entries[0].path())?;
        assert_eq!(backup_content, "original content");

        Ok(())
    }

    #[test]
    fn test_backup_file_nonexistent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("nonexistent.toml");

        // 备份不存在的文件应该成功（无操作）
        let result = backup_file(&file_path, "config", 5);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_backup_cleanup_old_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("settings.json");

        // 创建原始文件
        fs::write(&file_path, "content")?;

        // 手动创建 12 个旧备份文件（模拟不同时间戳）
        for i in 0..12 {
            let backup_name = format!("settings_202401{:02}120000.json", i + 1);
            fs::write(temp_dir.path().join(&backup_name), "backup content")?;
        }

        // 执行一次备份，触发清理
        backup_file(&file_path, "settings", 5)?;

        // 验证只保留 5 个备份（max_backups=5）
        let entries: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("settings_") && name.ends_with(".json")
            })
            .collect();

        assert_eq!(entries.len(), 5);

        Ok(())
    }
}