# 自动推断服务商类型测试用例

## 测试场景

### 场景 1: Claude 服务商 - 通过名称识别

**输入**:
- Provider 名称: `Claude Official`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"claude"`
- 行为: 点击导入后直接导入,无需手动选择

**验证方法**:
1. 在 OpenCode 配置中添加名为 "Claude Official" 的服务商
2. 打开 Open Switch 的"已部署服务商"对话框
3. 确认该服务商显示推断类型提示
4. 点击导入按钮,验证是否直接导入

---

### 场景 2: Anthropic 服务商

**输入**:
- Provider 名称: `anthropic-api`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"claude"`
- 行为: 直接导入为 Claude 类型

---

### 场景 3: GPT 服务商 - 通过名称识别

**输入**:
- Provider 名称: `GPT-4 Turbo`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"codex"`
- 行为: 直接导入为 Codex 类型

---

### 场景 4: OpenAI 服务商

**输入**:
- Provider 名称: `OpenAI Official`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"codex"`
- 行为: 直接导入为 Codex 类型

---

### 场景 5: Code 服务商

**输入**:
- Provider 名称: `Code Completion API`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"codex"`
- 行为: 直接导入为 Codex 类型

---

### 场景 6: Gemini 服务商 - 通过名称识别

**输入**:
- Provider 名称: `Gemini Pro`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"gemini"`
- 行为: 直接导入为 Gemini 类型

---

### 场景 7: Google 服务商

**输入**:
- Provider 名称: `Google AI Studio`
- 模型列表: 任意

**预期输出**:
- `inferred_model_type`: `"gemini"`
- 行为: 直接导入为 Gemini 类型

---

### 场景 8: 通过模型列表推断 - Claude

**输入**:
- Provider 名称: `CustomAPI-1`
- 模型列表:
  ```json
  {
    "claude-4.5-sonnet": {...},
    "claude-4-opus": {...},
    "claude-3.5-haiku": {...}
  }
  ```

**预期输出**:
- `inferred_model_type`: `"claude"`
- 推断依据: 所有模型都包含 "claude"

---

### 场景 9: 通过模型列表推断 - Codex

**输入**:
- Provider 名称: `CustomAPI-2`
- 模型列表:
  ```json
  {
    "gpt-5.2-codex": {...},
    "gpt-5.1": {...},
    "code-davinci": {...}
  }
  ```

**预期输出**:
- `inferred_model_type`: `"codex"`
- 推断依据: 模型包含 "gpt" 和 "code"

---

### 场景 10: 通过模型列表推断 - Gemini

**输入**:
- Provider 名称: `CustomAPI-3`
- 模型列表:
  ```json
  {
    "gemini-3-pro": {...},
    "gemini-2.5-pro": {...}
  }
  ```

**预期输出**:
- `inferred_model_type`: `"gemini"`
- 推断依据: 所有模型都包含 "gemini"

---

### 场景 11: 混合模型 - Claude 占优

**输入**:
- Provider 名称: `MixedAPI`
- 模型列表:
  ```json
  {
    "claude-4.5-sonnet": {...},
    "claude-4-opus": {...},
    "gpt-4": {...}
  }
  ```

**预期输出**:
- `inferred_model_type`: `"claude"`
- 推断依据: Claude 模型数量 (2) > GPT 模型数量 (1)

---

### 场景 12: 无法推断 - 自定义模型

**输入**:
- Provider 名称: `CustomProvider`
- 模型列表:
  ```json
  {
    "model-alpha": {...},
    "model-beta": {...},
    "model-gamma": {...}
  }
  ```

**预期输出**:
- `inferred_model_type`: `null`
- 行为: 点击导入后弹出手动选择对话框

---

### 场景 13: 大小写不敏感

**输入**:
- Provider 名称: `CLAUDE` / `Claude` / `claude`

**预期输出**:
- 所有情况都应识别为 `"claude"`
- 验证大小写不敏感功能

---

### 场景 14: 已有 model_type 的服务商

**输入**:
- Provider 名称: `ExistingProvider`
- 已有 `model_type`: `"claude"`
- 模型列表: 包含 GPT 模型

**预期输出**:
- `inferred_model_type`: `"claude"` (使用已有的,不重新推断)
- 行为: 直接使用已有的类型

---

## 自动化测试建议

### 单元测试 (Rust)

在 `src-tauri/src/commands/provider.rs` 中添加:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::config::OpenCodeModelInfo;

    #[test]
    fn test_infer_by_name_claude() {
        let models = HashMap::new();
        assert_eq!(infer_model_type("Claude API", &models), Some("claude".to_string()));
        assert_eq!(infer_model_type("anthropic-test", &models), Some("claude".to_string()));
    }

    #[test]
    fn test_infer_by_name_codex() {
        let models = HashMap::new();
        assert_eq!(infer_model_type("GPT-4 Service", &models), Some("codex".to_string()));
        assert_eq!(infer_model_type("OpenAI API", &models), Some("codex".to_string()));
        assert_eq!(infer_model_type("Code Service", &models), Some("codex".to_string()));
    }

    #[test]
    fn test_infer_by_name_gemini() {
        let models = HashMap::new();
        assert_eq!(infer_model_type("Gemini Pro", &models), Some("gemini".to_string()));
        assert_eq!(infer_model_type("Google AI", &models), Some("gemini".to_string()));
    }

    #[test]
    fn test_infer_by_models() {
        let mut models = HashMap::new();
        models.insert("claude-4.5-sonnet".to_string(), OpenCodeModelInfo::default());
        models.insert("claude-4-opus".to_string(), OpenCodeModelInfo::default());
        
        assert_eq!(infer_model_type("CustomAPI", &models), Some("claude".to_string()));
    }

    #[test]
    fn test_cannot_infer() {
        let mut models = HashMap::new();
        models.insert("model-a".to_string(), OpenCodeModelInfo::default());
        models.insert("model-b".to_string(), OpenCodeModelInfo::default());
        
        assert_eq!(infer_model_type("CustomProvider", &models), None);
    }
}
```

### 集成测试步骤

1. **准备测试环境**
   - 备份 `~/.config/opencode/opencode.json`
   - 创建测试用的服务商配置

2. **执行测试**
   - 启动 Open Switch 应用
   - 打开"已部署服务商"对话框
   - 验证每个测试场景

3. **验证点**
   - 检查 API 返回的 `inferred_model_type` 值
   - 验证 UI 是否正确显示推断结果
   - 测试导入按钮的行为(直接导入 vs 弹出选择框)
   - 确认导入后的服务商 `model_type` 正确

4. **清理**
   - 恢复原始配置
   - 删除测试数据

---

## 边界情况测试

### 边界 1: 空模型列表
- Provider 名称: `Test`
- 模型列表: `{}`
- 预期: 无法推断

### 边界 2: 特殊字符
- Provider 名称: `Claude-API-v2`
- 预期: 识别为 `"claude"`

### 边界 3: 多关键词冲突
- Provider 名称: `Claude-GPT-Proxy`
- 预期: 识别为 `"claude"` (优先匹配第一个)

### 边界 4: 部分匹配
- Provider 名称: `myclaude`
- 预期: 识别为 `"claude"`
