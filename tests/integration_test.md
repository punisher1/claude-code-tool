# 集成测试计划

## 测试场景 1: 基本流程

### 步骤 1: 列出内置提供商
```bash
cct provider list
```
**预期输出:**
- 显示 claude (Built-in)
- 显示 deepseek (Built-in)
- 显示 kimi (Built-in)
- 显示 zhipu (Built-in)

### 步骤 2: 添加自定义提供商
```bash
cct provider add my-provider
# 输入 Base URL: https://my-api.com
# 输入 Models:
# gpt-4=gpt-4-turbo
# gpt-3=gpt-3.5-turbo
# (空行结束)
```
**预期结果:**
- 成功添加提供商
- 配置文件更新

### 步骤 3: 验证提供商已添加
```bash
cct provider list
```
**预期输出:**
- my-provider 显示为 Custom 类型

### 步骤 4: 添加配置实例
```bash
cct add -p my-provider -k my-api-key-123 my-config
```
**预期结果:**
- 成功添加配置
- 配置文件包含新配置

### 步骤 5: 使用配置
```bash
cct use my-config
```
**预期结果:**
- 更新 ~/.claude/settings.json
- 设置正确的环境变量
- 显示成功消息

## 测试场景 2: 错误处理

### 测试 1: 删除正在使用的提供商
```bash
cct provider rm my-provider
```
**预期结果:**
- 错误: 不能删除正在使用的提供商

### 测试 2: 使用不存在的配置
```bash
cct use non-existent
```
**预期结果:**
- 错误: 配置不存在

### 测试 3: 添加重复的别名
```bash
cct add -p deepseek -k key123 my-config
```
**预期结果:**
- 错误: 别名已存在

## 测试场景 3: 覆盖内置提供商

### 步骤 1: 覆盖 deepseek 提供商
```bash
cct provider add deepseek
# 输入新的 Base URL: https://custom-deepseek.com
```

### 步骤 2: 验证覆盖
```bash
cct provider list
```
**预期结果:**
- deepseek 显示为 Custom 类型
- Base URL 已更新

## 测试场景 4: 配置文件持久化

### 步骤 1: 重启后验证配置
1. 添加多个配置
2. 退出程序
3. 重新运行程序
4. 验证所有配置仍然存在

### 步骤 2: 验证 Claude 设置
```bash
cat ~/.claude/settings.json
```
**预期内容:**
```json
{
  "env": {
    "ANTHROPIC_API_KEY": "my-api-key-123",
    "ANTHROPIC_BASE_URL": "https://my-api.com"
  }
}
```

## 性能测试

### 大规模提供商测试
- 添加 50 个自定义提供商
- 验证列表性能

### 大规模配置测试
- 添加 100 个配置实例
- 验证切换性能