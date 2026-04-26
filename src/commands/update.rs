use crate::commands::Command;
use crate::models::AppConfig;
use anyhow::{Context, Result, anyhow};
use console::style;
use serde_json::Value;
use std::fs;
#[cfg(not(windows))]
use std::fs::OpenOptions;
#[cfg(not(windows))]
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

const REPO: &str = "punisher1/claude-code-tool";
const GITHUB_API_LATEST: &str =
    "https://api.github.com/repos/punisher1/claude-code-tool/releases/latest";

pub struct UpdateCommand;

#[derive(Debug, Clone, PartialEq)]
enum ArchiveKind {
    TarGz,
    Zip,
}

#[derive(Debug, Clone, PartialEq)]
struct PlatformAsset {
    artifact_name: String,
    binary_name: &'static str,
    archive_kind: ArchiveKind,
}

impl Command for UpdateCommand {
    fn execute(self, _config: &mut AppConfig) -> Result<()> {
        let asset = detect_platform_asset(std::env::consts::OS, std::env::consts::ARCH)?;
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let install_dir = default_install_dir(&home_dir);

        println!("{} Checking latest cct release...", style(">").cyan());
        let release_json = download_text(GITHUB_API_LATEST)?;
        let version = parse_release_tag(&release_json)?;
        let download_url = format!(
            "https://github.com/{}/releases/download/{}/{}",
            REPO, version, asset.artifact_name
        );

        println!(
            "{} Downloading {} ({})...",
            style(">").cyan(),
            asset.artifact_name,
            version
        );

        fs::create_dir_all(&install_dir)
            .with_context(|| format!("Failed to create install directory: {:?}", install_dir))?;

        let temp_dir = TempDir::new("cct-update")?;
        let archive_path = temp_dir.path().join(&asset.artifact_name);
        download_file(&download_url, &archive_path)?;

        println!(
            "{} Installing to {}...",
            style(">").cyan(),
            install_dir.display()
        );
        let binary_path = extract_archive(&archive_path, temp_dir.path(), &asset)?;
        install_binary(&binary_path, &install_dir, asset.binary_name)?;

        ensure_path_or_warn(&install_dir, &home_dir);

        println!(
            "{} Updated cct to {} at {}",
            style("✓").green(),
            version,
            install_dir.join(asset.binary_name).display()
        );

        Ok(())
    }
}

fn detect_platform_asset(os: &str, arch: &str) -> Result<PlatformAsset> {
    let arch_suffix = match arch {
        "x86_64" | "amd64" => "x86_64",
        "aarch64" | "arm64" => "aarch64",
        other => return Err(anyhow!("Unsupported architecture: {}", other)),
    };

    match os {
        "windows" | "Windows" => {
            if arch_suffix != "x86_64" {
                return Err(anyhow!(
                    "Windows {} is not currently supported",
                    arch_suffix
                ));
            }
            Ok(PlatformAsset {
                artifact_name: "cct-Windows-x86_64.zip".to_string(),
                binary_name: "cct.exe",
                archive_kind: ArchiveKind::Zip,
            })
        }
        "linux" | "Linux" => {
            if arch_suffix != "x86_64" {
                return Err(anyhow!("Linux {} is not currently supported", arch_suffix));
            }
            Ok(PlatformAsset {
                artifact_name: "cct-Linux-x86_64.tar.gz".to_string(),
                binary_name: "cct",
                archive_kind: ArchiveKind::TarGz,
            })
        }
        "macos" | "Darwin" => Ok(PlatformAsset {
            artifact_name: format!("cct-Darwin-{}.tar.gz", arch_suffix),
            binary_name: "cct",
            archive_kind: ArchiveKind::TarGz,
        }),
        other => Err(anyhow!("Unsupported operating system: {}", other)),
    }
}

fn default_install_dir(home: &Path) -> PathBuf {
    home.join(".local").join("bin")
}

fn path_contains_dir(path: &str, dir: &Path) -> bool {
    let separator = if path.contains(';') { ';' } else { ':' };
    let wanted = normalize_path_for_compare(dir);

    path.split(separator).any(|part| {
        let trimmed = part.trim().trim_matches('"');
        if trimmed.is_empty() {
            return false;
        }
        normalize_path_for_compare(Path::new(trimmed)) == wanted
    })
}

#[cfg(any(not(windows), test))]
fn shell_profile_path(home: &Path, shell: Option<&str>) -> PathBuf {
    match shell
        .and_then(|value| Path::new(value).file_name())
        .and_then(|value| value.to_str())
    {
        Some("zsh") => home.join(".zshrc"),
        Some("bash") => home.join(".bashrc"),
        _ => home.join(".profile"),
    }
}

fn normalize_path_for_compare(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        text.to_ascii_lowercase()
    } else {
        text
    }
}

fn parse_release_tag(json: &str) -> Result<String> {
    let parsed: Value =
        serde_json::from_str(json).context("Failed to parse GitHub release JSON")?;
    parsed
        .get("tag_name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("GitHub release JSON does not contain tag_name"))
}

fn download_text(url: &str) -> Result<String> {
    let curl = ProcessCommand::new("curl")
        .args(["-fsSL", "-H", "User-Agent: cct-updater", url])
        .output();
    if let Ok(output) = curl {
        if output.status.success() {
            return String::from_utf8(output.stdout).context("GitHub response was not UTF-8");
        }
    }

    let wget = ProcessCommand::new("wget")
        .args(["-qO-", "--header=User-Agent: cct-updater", url])
        .output();
    if let Ok(output) = wget {
        if output.status.success() {
            return String::from_utf8(output.stdout).context("GitHub response was not UTF-8");
        }
    }

    #[cfg(windows)]
    {
        let script = "$ProgressPreference='SilentlyContinue'; \
            (Invoke-WebRequest -UseBasicParsing -Headers @{'User-Agent'='cct-updater'} -Uri $args[0]).Content";
        let powershell = ProcessCommand::new("powershell")
            .args(["-NoProfile", "-Command", script, url])
            .output();
        if let Ok(output) = powershell {
            if output.status.success() {
                return String::from_utf8(output.stdout).context("GitHub response was not UTF-8");
            }
        }
    }

    Err(anyhow!("Failed to download {}", url))
}

fn download_file(url: &str, output_path: &Path) -> Result<()> {
    let output = output_path.to_string_lossy().to_string();

    let curl = ProcessCommand::new("curl")
        .args(["-fSL", "-H", "User-Agent: cct-updater", url, "-o", &output])
        .status();
    if matches!(curl, Ok(status) if status.success()) {
        return Ok(());
    }

    let wget = ProcessCommand::new("wget")
        .args([
            "--quiet",
            "--header=User-Agent: cct-updater",
            "-O",
            &output,
            url,
        ])
        .status();
    if matches!(wget, Ok(status) if status.success()) {
        return Ok(());
    }

    #[cfg(windows)]
    {
        let script = "$ProgressPreference='SilentlyContinue'; \
            Invoke-WebRequest -UseBasicParsing -Headers @{'User-Agent'='cct-updater'} \
            -Uri $args[0] -OutFile $args[1]";
        let powershell = ProcessCommand::new("powershell")
            .args(["-NoProfile", "-Command", script, url, &output])
            .status();
        if matches!(powershell, Ok(status) if status.success()) {
            return Ok(());
        }
    }

    Err(anyhow!("Failed to download {}", url))
}

fn extract_archive(archive_path: &Path, temp_dir: &Path, asset: &PlatformAsset) -> Result<PathBuf> {
    match asset.archive_kind {
        ArchiveKind::TarGz => {
            let status = ProcessCommand::new("tar")
                .arg("-xzf")
                .arg(archive_path)
                .arg("-C")
                .arg(temp_dir)
                .status()
                .context("Failed to run tar")?;
            if !status.success() {
                return Err(anyhow!("Failed to extract {}", archive_path.display()));
            }
        }
        ArchiveKind::Zip => {
            let archive = archive_path.to_string_lossy().to_string();
            let dest = temp_dir.to_string_lossy().to_string();
            let script = "Expand-Archive -Force -Path $args[0] -DestinationPath $args[1]";
            let status = ProcessCommand::new("powershell")
                .args(["-NoProfile", "-Command", script, &archive, &dest])
                .status()
                .context("Failed to run PowerShell Expand-Archive")?;
            if !status.success() {
                return Err(anyhow!("Failed to extract {}", archive_path.display()));
            }
        }
    }

    let binary_path = temp_dir.join(asset.binary_name);
    if !binary_path.exists() {
        return Err(anyhow!(
            "Binary not found in archive: {}",
            binary_path.display()
        ));
    }

    Ok(binary_path)
}

fn install_binary(binary_path: &Path, install_dir: &Path, binary_name: &str) -> Result<()> {
    let dest_path = install_dir.join(binary_name);
    fs::copy(binary_path, &dest_path)
        .with_context(|| format!("Failed to install binary to {}", dest_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&dest_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&dest_path, permissions)?;
    }

    Ok(())
}

fn ensure_path_or_warn(install_dir: &Path, home_dir: &Path) {
    let current_path = std::env::var("PATH").unwrap_or_default();
    if path_contains_dir(&current_path, install_dir) {
        println!(
            "{} {} is already in PATH",
            style("✓").green(),
            install_dir.display()
        );
        return;
    }

    if try_persist_path(install_dir, home_dir).is_ok() {
        println!(
            "{} Added {} to user PATH. Open a new terminal for it to take effect.",
            style("✓").green(),
            install_dir.display()
        );
    } else {
        println!(
            "{} {} is not in PATH. Add it manually to run cct from any terminal.",
            style("!").yellow(),
            install_dir.display()
        );
    }
}

#[cfg(windows)]
fn try_persist_path(install_dir: &Path, _home_dir: &Path) -> Result<()> {
    let install_dir = install_dir.to_string_lossy().to_string();
    let user_path_script = "[Environment]::GetEnvironmentVariable('Path', 'User')";
    let output = ProcessCommand::new("powershell")
        .args(["-NoProfile", "-Command", user_path_script])
        .output()
        .context("Failed to read user Path")?;
    if !output.status.success() {
        return Err(anyhow!("Failed to read user Path"));
    }

    let user_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path_contains_dir(&user_path, Path::new(&install_dir)) {
        return Ok(());
    }

    let new_path = if user_path.is_empty() {
        install_dir
    } else {
        format!("{};{}", install_dir, user_path)
    };
    let set_path_script = "[Environment]::SetEnvironmentVariable('Path', $args[0], 'User')";
    let status = ProcessCommand::new("powershell")
        .args(["-NoProfile", "-Command", set_path_script, &new_path])
        .status()
        .context("Failed to update user Path")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Failed to update user Path"))
    }
}

#[cfg(not(windows))]
fn try_persist_path(install_dir: &Path, home_dir: &Path) -> Result<()> {
    let shell = std::env::var("SHELL").ok();
    let profile_path = shell_profile_path(home_dir, shell.as_deref());
    let marker = format!("{}", install_dir.display());

    if let Ok(existing) = fs::read_to_string(&profile_path) {
        if existing.contains(&marker) {
            return Ok(());
        }
    }

    let mut profile = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)
        .with_context(|| format!("Failed to open shell profile: {}", profile_path.display()))?;

    writeln!(
        profile,
        "\n# Added by cct updater\nexport PATH=\"{}:$PATH\"",
        install_dir.display()
    )
    .with_context(|| format!("Failed to update shell profile: {}", profile_path.display()))?;

    Ok(())
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System clock is before UNIX_EPOCH")?
            .as_millis();
        let path = std::env::temp_dir().join(format!("{}-{}", prefix, timestamp));
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create temp directory: {}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform_asset_names() {
        assert_eq!(
            detect_platform_asset("windows", "x86_64").unwrap(),
            PlatformAsset {
                artifact_name: "cct-Windows-x86_64.zip".to_string(),
                binary_name: "cct.exe",
                archive_kind: ArchiveKind::Zip,
            }
        );
        assert_eq!(
            detect_platform_asset("linux", "x86_64").unwrap(),
            PlatformAsset {
                artifact_name: "cct-Linux-x86_64.tar.gz".to_string(),
                binary_name: "cct",
                archive_kind: ArchiveKind::TarGz,
            }
        );
        assert_eq!(
            detect_platform_asset("macos", "aarch64").unwrap(),
            PlatformAsset {
                artifact_name: "cct-Darwin-aarch64.tar.gz".to_string(),
                binary_name: "cct",
                archive_kind: ArchiveKind::TarGz,
            }
        );
    }

    #[test]
    fn test_default_install_dir_uses_local_bin() {
        assert_eq!(
            default_install_dir(Path::new("/home/alice")),
            PathBuf::from("/home/alice/.local/bin")
        );
    }

    #[test]
    fn test_path_contains_dir_uses_path_segments() {
        assert!(path_contains_dir(
            "/usr/bin:/home/alice/.local/bin:/bin",
            Path::new("/home/alice/.local/bin")
        ));
        assert!(!path_contains_dir(
            "/usr/bin:/home/alice/.local/bin-extra:/bin",
            Path::new("/home/alice/.local/bin")
        ));
    }

    #[test]
    fn test_shell_profile_path_prefers_shell_rc() {
        assert_eq!(
            shell_profile_path(Path::new("/home/alice"), Some("/bin/zsh")),
            PathBuf::from("/home/alice/.zshrc")
        );
        assert_eq!(
            shell_profile_path(Path::new("/home/alice"), Some("/usr/bin/bash")),
            PathBuf::from("/home/alice/.bashrc")
        );
        assert_eq!(
            shell_profile_path(Path::new("/home/alice"), Some("/bin/fish")),
            PathBuf::from("/home/alice/.profile")
        );
    }
}
