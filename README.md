# claude-code-tool (cct)

[中文文档](README_zh-CN.md)

[![GitHub Release](https://img.shields.io/github/v/release/punisher1/claude-code-tool)](https://github.com/punisher1/claude-code-tool/releases)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-blue)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/punisher1/claude-code-tool/total)](https://github.com/punisher1/claude-code-tool/releases)
[![License](https://img.shields.io/github/license/punisher1/claude-code-tool)](LICENSE)

Claude Code API Switching Tool - A CLI application for managing and switching between different API providers for the claudecode tool.

## Features

- **Multi-provider Support**: Easily switch between Anthropic official API and third-party compatible providers (DeepSeek, Kimi/Moonshot, Zhipu GLM, etc.)
- **Custom Providers**: Add and manage custom API providers
- **Configuration Management**: Create and manage multiple API configuration instances
- **One-click Switching**: Quickly activate different API configurations
- **Launch & Run**: Directly launch Claude Code with environment variables set for a specific configuration

## Installation

### Download from GitHub Release (Recommended)
Visit the [Releases Page](https://github.com/punisher1/claude-code-tool/releases) to download pre-compiled binaries for your platform:

- **Linux x86_64**: `cct-Linux-x86_64.tar.gz`
- **macOS x86_64**: `cct-Darwin-x86_64.tar.gz`
- **macOS Apple Silicon**: `cct-Darwin-aarch64.tar.gz`
- **Windows x86_64**: `cct-Windows-x86_64.zip`

After downloading, unzip and add the binary to your system PATH.

### Build from Source

Ensure you have the Rust toolchain (1.70+) installed, then run:

```bash
# Clone the repository
git clone https://github.com/punisher1/claude-code-tool.git
cd claude-code-tool

# Build release version
cargo build --release
```

The executable will be generated at `target/release/cct` (or `cct.exe` on Windows).

## Usage

### List all providers
```bash
cct provider list
```

### Add a custom provider
```bash
cct provider add [name]
# Or use interactive mode
cct provider add
```

### Remove a custom provider
```bash
cct provider rm <name>
```

### Add an API configuration
```bash
cct add -p <provider> -a <api_key> <alias>
# Example
cct add -p deepseek -a sk-xxx my-deepseek
```

### Use a configuration
```bash
cct use <alias>
# Example
cct use my-deepseek
```

### Start Claude Code
```bash
# Start Claude Code with a specific configuration
cct start <alias>
# Example
cct start my-deepseek

# Start and pass arguments to claude
cct start my-deepseek -- -p "hello claude"

# Start and set proxy
cct start my-deepseek --proxy "http://127.0.0.1:11225"
```

## File Locations

- Config File: `~/.cct/config.toml` (macOS/Linux) or `%USERPROFILE%\.cct\config.toml` (Windows)
- Claude Settings: `~/.claude/settings.json` (macOS/Linux) or `%USERPROFILE%\.claude\settings.json` (Windows)

## Supported Providers

### Built-in Providers
- **claude-code** - Anthropic Claude Code (Official)
- **deepseek** - DeepSeek API
- **kimi-coding** - Kimi for Coding (Moonshot API)
- **zhipu** - Zhipu GLM API

### Custom Providers
Supports adding any third-party provider compatible with the Anthropic API format.

## Technical Features

### Environment Variable Type Support
Environment variables in the configuration file support three types:
- **String**: For text values like URLs, API keys
- **Int**: For numeric configurations like timeouts, flags
- **Bool**: For true/false configuration options

Example configuration:
```toml
[providers.deepseek]
description = "DeepSeek API"
env.ANTHROPIC_BASE_URL = "https://api.deepseek.com"
env.ANTHROPIC_MODEL = "deepseek-chat"
env.API_TIMEOUT_MS = 3000000  # Integer type
env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = 1  # Integer flag
```

## Development

### Run Tests
```bash
cargo test
```

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Create Release

The project uses GitHub Actions to automate the release process. To create a new version:

1. **Create and push a tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **GitHub Actions execution**:
   - Builds binaries for Linux, macOS, and Windows
   - Creates a GitHub Release
   - Automatically generates Release Notes
   - Uploads binaries and checksums for all platforms

3. **Check results on the GitHub Release page**:
   - Auto-generated Release Notes (based on commit categories)
   - Pre-compiled binaries for all platforms
   - SHA256 checksum for each binary

The release workflow is configured in `.github/workflows/release.yml`.

### Code Formatting
```bash
cargo fmt
```

### Code Linting
```bash
cargo clippy
```

## License

MIT License