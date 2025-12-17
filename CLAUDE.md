# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

claude-code-tool (cct) 是一个 Rust CLI 应用程序，用于管理和切换 claudecode 工具的不同 API 提供商。项目当前处于设计阶段，尚未实现源代码。

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
- `src/models.rs` - 数据模型定义（Provider, ConfigInstance, AppConfig）
- `src/config_manager.rs` - TOML 配置文件读写管理
- `src/provider_store.rs` - 提供商管理（内置 + 自定义）
- `src/claude_adapter.rs` - Claude settings.json 修改适配器
- `src/commands/` - CLI 命令实现模块
- `src/utils.rs` - 工具函数

### 文件系统操作
1. **配置文件**: `~/claude-code-tool/config.toml` - 存储自定义提供商和配置实例
2. **目标文件**: `~/.claude/settings.json` - 修改 claudecode 的 env 配置

### 数据流
1. 用户通过 CLI 命令与系统交互
2. ProviderStore 合并内置和自定义提供商
3. ConfigManager 处理配置文件的读写
4. ClaudeAdapter 更新 settings.json 文件

## 开发注意事项

### 测试要求
- 每个核心模块都需要编写单元测试
- 使用 tempfile 进行文件系统操作的测试
- 确保测试覆盖关键逻辑路径
- **每开发完一个子命令，需要进行集成测试**，验证端到端流程

### 错误处理
- 使用 anyhow 进行错误封装
- 使用 thiserror 定义自定义错误类型
- 提供清晰的错误信息给用户

### 提供商管理规则
- 内置提供商不可删除
- 自定义提供商可以覆盖同名内置提供商
- 删除提供商前需要检查是否有配置实例引用