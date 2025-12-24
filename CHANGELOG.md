# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Documentation
- Added Chinese localization for README (`README_zh-CN.md`).
- Added badges for Release, Platform, Downloads, and License to README.
- Updated Built-in Providers list in README to include Xiaomi Mimo and Minimax M2.

## [0.1.3]

### Added
- `start` command to launch Claude Code with specific configurations (`cct start <alias>`).
- Support for launching with arguments (e.g., `cct start <alias> -- -p "hello"`).
- Support for `xiaomi-mimo` and `minimaxi-m2` as built-in providers.
- Enhanced environment variable support in configuration (String, Int, Bool types).

### Changed
- Improved configuration management and provider switching logic.
- Optimized backup and restore mechanism for settings.

## [0.1.2]

### Added
- `list` (or `ls`) command to display user configurations.
- Table view for configuration list.

## [0.1.0]

### Added
- Initial release.
- Core functionality: `provider list`, `provider add`, `provider rm`.
- `add`, `use`, `current` commands for configuration management.
- Multi-provider support (Anthropic, DeepSeek, Kimi, Zhipu).
