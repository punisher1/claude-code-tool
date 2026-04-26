use std::process::Command;

/// 基本测试：验证 provider list 命令能够成功运行
#[test]
fn test_provider_list_runs() {
    let output = Command::new("cargo")
        .args(["run", "--", "provider", "list"])
        .output()
        .expect("Failed to execute command");

    // 验证命令成功执行
    assert!(
        output.status.success(),
        "provider list command should succeed"
    );

    // 验证输出不为空
    assert!(!output.stdout.is_empty() || !output.stderr.is_empty());
}

/// 测试 version 命令
#[test]
fn test_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cct"), "version should contain 'cct'");
}

/// 测试 help 命令
#[test]
fn test_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Claude"), "help should contain 'Claude'");
}

/// 测试 provider ls 命令（应该与 provider list 等效）
#[test]
fn test_provider_ls() {
    let output = Command::new("cargo")
        .args(["run", "--", "provider", "ls"])
        .output()
        .expect("Failed to execute command");

    // 验证命令成功执行
    assert!(
        output.status.success(),
        "provider ls command should succeed"
    );

    // 验证输出与 provider list 相同（包含内置提供商）
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("deepseek") || stdout.contains("kimi-coding") || stdout.contains("zhipu"),
        "provider ls should display at least one built-in provider"
    );
}
