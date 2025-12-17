# claude-code-tool 项目上下文

## 项目概览
`claude-code-tool` (cct) 是一个计划中的命令行界面 (CLI) 应用程序，旨在管理和切换 `claudecode` 工具的不同 API 提供商。通过修改 `claudecode` 配置，它旨在让用户能够轻松地在官方 Anthropic API 和兼容的第三方提供商（如 DeepSeek、Moonshot/Kimi、智谱 GLM 等）之间进行切换。

**当前状态:** 设计与文档阶段。尚未实现任何源代码。

## 架构与设计

### 核心概念
该工具充当用户定义/内置 API 提供商定义与 `claudecode` 设置文件之间的桥梁。

### 数据源
1.  **自定义提供商（高优先级）：** 用户定义的提供商，存储在 `~/claude-code-tool/config.toml` 中。
2.  **内置提供商（低优先级）：** 硬编码的默认值（例如，`claudecode`、`kimi-k2`、`deepseek`、`zhipu-glm`）。

### 文件系统
*   **配置：** `~/claude-code-tool/config.toml` (存储自定义提供商和用户 API 密钥配置)。
*   **目标：** `~/.claude/settings.json` (`claudecode` 的原生配置文件)。`cct` 修改此文件中的 `env` 字段。

## 计划中的功能 (CLI)

*   **`cct provider list`：** 列出可用的提供商（内置 + 自定义）。
*   **`cct provider add`：** 添加或更新自定义提供商模板。
*   **`cct provider rm`：** 删除自定义提供商。
*   **`cct add`：** 创建一个新的配置实例（别名），将提供商链接到 API 密钥。
*   **`cct use`：** 激活特定配置，更新 `~/.claude/settings.json`。

## 开发路线图
1.  **数据层：** 实现用于 TOML 读写操作的 `ConfigManager`。
2.  **提供商层：** 实现 `ProviderRegistry` 以处理内置和自定义提供商之间的合并逻辑。
3.  **CLI 层：** 实现命令行界面和用户交互逻辑。

## 关键文档
*   `docs/1-需求文档.md`：包含工具的详细需求、数据结构定义和逻辑。