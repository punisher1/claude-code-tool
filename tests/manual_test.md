# 手动测试计划

由于网络问题无法下载依赖，我们将手动验证代码逻辑。

## 代码结构验证

1. **模块结构** ✓
   - models.rs - 数据模型定义
   - config_manager.rs - 配置管理
   - provider_store.rs - 提供商管理
   - claude_adapter.rs - Claude 设置适配
   - utils.rs - 工具函数
   - commands/ - 命令实现
   - main.rs - CLI 入口

2. **关键功能验证**

### 数据模型 (models.rs)
- [x] Provider 结构体包含 base_url 和 models
- [x] ConfigInstance 包含 provider, api_key, env
- [x] AppConfig 包含 providers, configs, current
- [x] 实现了 Default trait
- [x] 包含 TOML 序列化测试

### 配置管理 (config_manager.rs)
- [x] ConfigManager 结构体
- [x] get_config_path() 返回 ~/claude-code-tool/config.toml
- [x] load_config() 处理文件不存在的情况
- [x] save_config() 创建目录并写入文件
- [x] 包含完整的测试用例

### Provider 仓库 (provider_store.rs)
- [x] BUILTIN_PROVIDERS 包含 claude, deepseek, kimi, zhipu
- [x] get_merged_providers() 合并内置和自定义提供商
- [x] 自定义提供商可以覆盖内置提供商
- [x] is_builtin_provider() 检查是否为内置提供商
- [x] 包含完整的测试用例

### Claude 适配器 (claude_adapter.rs)
- [x] update_settings() 保留其他字段
- [x] 处理文件不存在的情况
- [x] 创建目录结构
- [x] 包含完整的测试用例

### 工具函数 (utils.rs)
- [x] generate_env_vars() 为不同提供商生成正确的环境变量
- [x] validate_provider_name() 验证提供商名称格式
- [x] 包含完整的测试用例

### 命令实现
- [x] provider list - 列出所有提供商
- [x] provider add - 添加自定义提供商
- [x] provider rm - 删除自定义提供商
- [x] add - 添加配置实例
- [x] use - 使用配置实例

## 集成测试计划

一旦网络问题解决，需要运行：
1. `cargo test` - 运行所有单元测试
2. `cargo build` - 构建项目
3. 手动集成测试流程