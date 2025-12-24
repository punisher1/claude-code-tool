# claude-code-tool (cct)

[English](README.md)

Claude Code API 切换工具 - 一个 CLI 应用程序，用于管理和切换 claudecode 工具的不同 API 提供商。

## 功能

- **多提供商支持**：轻松切换 Anthropic 官方 API 和第三方兼容提供商（DeepSeek、Kimi/Moonshot、智谱 GLM 等）
- **自定义提供商**：添加和管理自定义 API 提供商
- **配置管理**：创建和管理多个 API 配置实例
- **一键切换**：快速激活不同的 API 配置
- **启动运行**：直接启动 Claude Code 并设置指定配置的环境变量

## 安装

### 从 GitHub Release 下载（推荐）
访问 [Releases 页面](https://github.com/punisher1/claude-code-tool/releases) 下载适合您平台的预编译二进制文件：

- **Linux x86_64**: `cct-Linux-x86_64.tar.gz`
- **macOS x86_64**: `cct-Darwin-x86_64.tar.gz`
- **macOS Apple Silicon**: `cct-Darwin-aarch64.tar.gz`
- **Windows x86_64**: `cct-Windows-x86_64.zip`

下载后解压并将二进制文件添加到系统 PATH。

### 从源码构建

确保已安装 Rust 工具链（1.70+），然后运行：

```bash
# 克隆仓库
git clone https://github.com/punisher1/claude-code-tool.git
cd claude-code-tool

# 构建发布版本
cargo build --release
```

可执行文件将生成在 `target/release/cct` (Windows 上为 `cct.exe`)

## 使用方法

### 列出所有提供商
```bash
cct provider list
```

### 添加自定义提供商
```bash
cct provider add [name]
# 或使用交互模式
cct provider add
```

### 删除自定义提供商
```bash
cct provider rm <name>
```

### 添加 API 配置
```bash
cct add -p <provider> -a <api_key> <alias>
# 例如
cct add -p deepseek -a sk-xxx my-deepseek
```

### 使用配置
```bash
cct use <alias>
# 例如
cct use my-deepseek
```

### 启动 Claude Code
```bash
# 使用指定配置启动 Claude Code
cct start <alias>
# 例如
cct start my-deepseek

# 启动并传递参数给 claude
cct start my-deepseek -- -p "hello claude"

# 启动并设置代理
cct start my-deepseek --proxy "http://127.0.0.1:11225"
```

## 文件位置

- 配置文件：`~/.cct/config.toml` (macOS/Linux) 或 `%USERPROFILE%\.cct\config.toml` (Windows)
- Claude 设置：`~/.claude/settings.json` (macOS/Linux) 或 `%USERPROFILE%\.claude\settings.json` (Windows)

## 支持的提供商

### 内置提供商
- **claude-code** - Anthropic Claude Code (官方版)
- **deepseek** - DeepSeek API
- **kimi-coding** - Kimi for Coding (Moonshot API)
- **zhipu** - 智谱 GLM API

### 自定义提供商
支持添加任何兼容 Anthropic API 格式的第三方提供商。

## 技术特性

### 环境变量类型支持
配置文件中的环境变量支持三种类型：
- **字符串 (String)**: 适用于 URL、API 密钥等文本值
- **整数 (Int)**: 适用于数值类型的配置，如超时时间、标志位等
- **布尔值 (Bool)**: 适用于 true/false 配置选项

示例配置：
```toml
[providers.deepseek]
description = "DeepSeek API"
env.ANTHROPIC_BASE_URL = "https://api.deepseek.com"
env.ANTHROPIC_MODEL = "deepseek-chat"
env.API_TIMEOUT_MS = 3000000  # 整数类型
env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = 1  # 整数标志位
```

## 开发

### 运行测试
```bash
cargo test
```

### 开发构建
```bash
cargo build
```

### 发布构建
```bash
cargo build --release
```

### 创建 Release

项目使用 GitHub Actions 自动化 release 流程。创建新版本：

1. **创建并推送标签**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **GitHub Actions 自动执行**:
   - 为 Linux、macOS、Windows 构建二进制文件
   - 创建 GitHub Release
   - 自动生成 Release Notes
   - 上传所有平台的二进制文件和 checksum

3. **在 GitHub Release 页面查看结果**:
   - 自动生成的 Release Notes（基于 commit 分类）
   - 所有平台的预编译二进制文件
   - 每个二进制文件的 SHA256 checksum

release workflow 配置在 `.github/workflows/release.yml`。

### 代码格式化
```bash
cargo fmt
```

### 代码检查
```bash
cargo clippy
```

## 许可证

MIT License
