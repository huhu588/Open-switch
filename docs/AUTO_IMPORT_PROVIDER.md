# 自动推断服务商类型功能

## 功能概述

在导入已部署的 OpenCode 服务商时,系统现在能够自动推断服务商的类型（Claude / Codex / Gemini），无需用户手动选择。

## 推断规则

### 1. 基于服务商名称推断（优先级最高）

系统会检查服务商名称（不区分大小写）中的关键词：

- **Claude**: 名称包含 `claude` 或 `anthropic` → 识别为 `claude` 类型
- **Codex**: 名称包含 `gpt`、`openai`、`code` 或 `codex` → 识别为 `codex` 类型  
- **Gemini**: 名称包含 `gemini` 或 `google` → 识别为 `gemini` 类型

### 2. 基于模型列表推断（备用方案）

如果无法从服务商名称推断,系统会分析该服务商下的所有模型ID：

- 统计包含 `claude` 的模型数量
- 统计包含 `gpt` 或 `code` 的模型数量
- 统计包含 `gemini` 的模型数量

选择占比最高的类型作为推断结果。

## 使用示例

### 示例 1: 根据名称自动识别

```
服务商名称: "Claude API"
推断结果: claude (自动识别)
操作: 点击导入按钮后直接导入,无需选择类型
```

### 示例 2: 根据名称自动识别 (Code)

```
服务商名称: "GPT-4 Service"
推断结果: codex (自动识别)
操作: 点击导入按钮后直接导入,无需选择类型
```

### 示例 3: 根据模型推断

```
服务商名称: "MyAPI"
模型列表: 
  - claude-4.5-sonnet
  - claude-4-opus
  - claude-3.5-haiku
推断结果: claude (根据模型名称推断)
操作: 点击导入按钮后直接导入,无需选择类型
```

### 示例 4: 无法推断

```
服务商名称: "CustomProvider"
模型列表: 
  - model-a
  - model-b
推断结果: null (无法推断)
操作: 点击导入按钮后弹出手动选择对话框
```

## 技术实现

### 后端 (Rust)

**文件**: `src-tauri/src/commands/provider.rs`

新增函数:
```rust
fn infer_model_type(
    provider_name: &str, 
    models: &HashMap<String, OpenCodeModelInfo>
) -> Option<String>
```

修改结构体:
```rust
pub struct DeployedProviderItem {
    pub name: String,
    pub base_url: String,
    pub model_count: usize,
    pub source: String,
    pub inferred_model_type: Option<String>, // 新增字段
}
```

修改命令:
- `get_deployed_providers`: 在返回列表时自动推断每个服务商的类型

### 前端 (Vue)

**文件**: `src/components/DeployedProvidersDialog.vue`

修改函数:
```typescript
function startImport(provider: DeployedProviderItem) {
  importingProvider.value = provider
  // 如果能够推断出 model_type，直接导入
  if (provider.inferred_model_type) {
    importProvider(provider.inferred_model_type)
  } else {
    // 否则显示手动选择对话框
    showModelTypeDialog.value = true
  }
}
```

**文件**: `src/stores/providers.ts`

更新接口:
```typescript
export interface DeployedProviderItem {
  name: string
  base_url: string
  model_count: number
  source: string
  inferred_model_type?: string // 新增字段
}
```

## 用户体验改进

### 之前
1. 用户点击"导入"按钮
2. 总是弹出手动选择对话框
3. 用户选择 Claude / Codex / Gemini
4. 完成导入

### 现在  
1. 用户点击"导入"按钮
2. **如果可以自动识别**: 直接完成导入 ✨
3. **如果无法识别**: 弹出手动选择对话框
4. 完成导入

大多数情况下可以跳过手动选择步骤,提升用户体验。

## 兼容性

- 向后兼容: 无法自动识别时仍然提供手动选择功能
- 现有数据: 已导入的服务商不受影响
- OpenCode 配置: 不修改 OpenCode 原始配置文件格式
