# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

claude-code-tool (cct) 是一个 Rust CLI 应用程序，用于管理和切换 claudecode 工具的不同 API 提供商。

### 核心功能
- **多提供商支持**：内置支持 Anthropic、DeepSeek、Kimi/Moonshot、智谱 GLM 等 API 提供商
- **自定义提供商**：可添加和管理自定义 API 提供商配置
- **配置管理**：创建和管理多个 API 配置实例（含 API 密钥和环境变量）
- **一键切换**：快速激活不同的 API 配置并更新 Claude Code 设置
- **环境变量类型支持**：支持 String、Int、Bool 三种类型的环境变量值

## 开发命令

### 构建和运行
```bash
# 构建项目
cargo build

# 构建发布版本
cargo build --release

# 运行测试
cargo test

# 运行特定测试
cargo test test_name

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 架构设计

### 核心模块结构
- `src/models.rs` - 数据模型定义
  - `EnvValue` 枚举：支持 String、Int、Bool 三种类型的环境变量值
  - `Provider` 结构体：提供商配置（描述和环境变量）
  - `ConfigInstance` 结构体：配置实例（提供商名称、API 密钥、自定义环境变量）
  - `AppConfig` 结构体：主配置（提供商列表、配置实例、当前激活配置）
- `src/config_manager.rs` - TOML 配置文件读写管理
  - 处理配置文件的加载、保存、备份
  - 路径：`~/.cct/config.toml`
- `src/provider_store.rs` - 提供商管理
  - 内置提供商（deepseek、kimi-coding、zhipu 等）
  - 自定义提供商加载和合并逻辑
  - 提供商优先级：自定义 > 内置
- `src/claude_adapter.rs` - Claude settings.json 适配器
  - 读取和修改 `~/.claude/settings.json`
  - 合并环境变量（配置 env > 提供商 env > API 密钥）
- `src/commands/` - CLI 命令模块
  - `mod.rs` - 命令路由
  - `provider.rs` - 提供商管理命令（list/add/rm）
  - `config.rs` - 配置管理命令（list/add/rm/use/current）
  - `switch.rs` - 交互式配置切换
- `src/utils.rs` - 工具函数
  - 提供商名称验证、环境变量合并等

### 文件系统操作
1. **配置文件**: `~/.cct/config.toml` (Windows: `%USERPROFILE%\.cct\config.toml`)
   - 存储用户自定义的提供商配置
   - 存储所有配置实例（含 API 密钥和环境变量）
   - 存储当前激活的配置名称
2. **目标文件**: `~/.claude/settings.json` (Windows: `%USERPROFILE%\.claude\settings.json`)
   - Claude Code 的主配置文件
   - cct 修改其中的 `env` 字段以切换 API 提供商

### 数据流
1. 用户执行 CLI 命令（如 `cct use <config>`）
2. CLI 解析命令并调用对应模块（commands/config.rs）
3. ConfigManager 从 TOML 文件加载配置
4. ProviderStore 合并内置提供商和自定义提供商
5. 根据配置实例获取对应提供商的环境变量
6. 合并三层环境变量（优先级：配置 env > 提供商 env > API 密钥）
7. ClaudeAdapter 读取当前 settings.json，更新 env 字段并写回
8. 完成切换，用户可重启 Claude Code 使用新配置

## 开发注意事项

### 测试要求
- 每个核心模块都需要编写单元测试（当前覆盖率：models, config_manager, provider_store, claude_adapter, utils, commands）
- 使用 tempfile 进行文件系统操作的测试，避免污染真实文件系统
- 确保测试覆盖关键逻辑路径：配置序列化/反序列化、提供商合并、环境变量优先级
- **每开发完一个子命令，需要进行集成测试**，验证端到端流程
- 所有测试通过后执行 `cargo build` 确保代码编译正常

### 错误处理
- 使用 `anyhow::Result` 进行错误封装，提供上下文信息
- 使用 `thiserror` 定义自定义错误类型（当前未定义，可后续扩展）
- 提供清晰的错误信息给用户，特别是文件操作和配置解析错误
- 关键操作（如更新 settings.json）前进行备份，失败时恢复原文件

### 提供商管理规则
- 内置提供商（claude-code, deepseek, kimi-coding, zhipu）不可删除
- 自定义提供商可以覆盖同名内置提供商（优先级：自定义 > 内置）
- 删除提供商前需要检查是否有配置实例引用，防止孤立的配置
- 提供商的环境变量会与应用配置实例的环境变量合并

### 环境变量优先级规则
三层环境变量合并机制：
1. 配置实例中的 env（最高优先级，用户自定义）
2. 提供商定义的 env（内置或自定义提供商的默认环境变量）
3. API 密钥（作为 ANTHROPIC_API_KEY 环境变量，最低优先级）

### 代码规范
- 使用 `cargo fmt` 保持代码格式化，确保一致性
- 使用 `cargo clippy` 检查潜在问题并修复建议
- 公共函数和结构体添加适当文档注释（///）
- 复杂逻辑添加行内注释说明设计意图
- 每次改完代码都必须执行 `cargo build` 保证没有error
- git的commit和push操作必须在我明确说明的时候才执行

### 序列化/反序列化注意事项
- `EnvValue` 枚举需要自定义 Serialize/Deserialize 实现，支持 String/Int/Bool 三种类型
- TOML 配置文件的路径使用 `-` 而非 `_`（如 `claude-code-tool`）
- JSON 输出到 settings.json 时，Int 类型会转换为 Number，确保 Claude Code 正确识别