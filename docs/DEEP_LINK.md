# Ai Switch 深链接配置指南

> 版本: v1.4.0 | 协议: `aiswitch://`

通过深链接让用户一键配置 Provider 到 Ai Switch 应用。

## URL 格式

```
aiswitch://add-provider?name=<名称>&api_key=<密钥>&base_url=<地址>&model_type=<协议>&models=<模型列表>&description=<描述>
```

## 参数说明

| 参数 | 必需 | 说明 |
|------|:----:|------|
| `name` | ✅ | 服务商名称 |
| `api_key` | ✅ | API Key |
| `base_url` | ✅ | API Base URL |
| `model_type` | ✅ | 协议类型：`openai`、`claude`、`gemini`、`codex`、`azure` |
| `models` | ❌ | 模型列表，逗号分隔 |
| `description` | ❌ | 描述 |

> **注意**：参数值需进行 URL 编码（如 `https://` → `https%3A%2F%2F`）

## 示例
## URL 编码

所有参数值必须进行 URL 编码：

| 字符 | 编码 |
|------|------|
| 空格 | `%20` |
| `/` | `%2F` |
| `:` | `%3A` |
| `?` | `%3F` |
| `&` | `%26` |
| `=` | `%3D` |

### 编码示例

原始值：`https://api.example.com/v1`

编码后：`https%3A%2F%2Fapi.example.com%2Fv1`

**基础配置：**
```
aiswitch://add-provider?name=MyAPI&api_key=sk-123&base_url=https%3A%2F%2Fapi.example.com%2Fv1&model_type=openai
```

**带模型列表：**
```
aiswitch://add-provider?name=i7%20Claude&api_key=i7-relay-8888&base_url=https%3A%2F%2Fi7dc.com%2Fapi&model_type=claude&models=claude-4.5-opus,claude-4.5-sonnet
```

## 网页集成

### HTML 按钮

```html
<a href="aiswitch://add-provider?name=MyAPI&api_key=YOUR_KEY&base_url=https%3A%2F%2Fapi.example.com%2Fv1&model_type=openai" 
   style="display:inline-block;padding:12px 24px;background:linear-gradient(135deg,#f59e0b,#d97706);color:white;text-decoration:none;border-radius:8px;font-weight:bold;">
  🚀 一键配置到 Ai Switch
</a>
```

### JavaScript

```javascript
function generateAiSwitchLink(config) {
  const params = new URLSearchParams({
    name: config.name,
    api_key: config.apiKey,
    base_url: config.baseUrl,
    model_type: config.modelType,
  });
  if (config.models?.length) params.set('models', config.models.join(','));
  if (config.description) params.set('description', config.description);
  return `aiswitch://add-provider?${params.toString()}`;
}
```

## 常见问题

**Q: 点击没反应？**
- 确保已安装 Ai Switch
- Windows/Linux 需运行一次应用以注册协议
- 检查浏览器是否阻止协议跳转

**Q: URL 长度限制？**
- 建议保持在 2000 字符以内

## 技术支持

如有问题，请加 QQ 2019588810 或发邮件联系。
