# OpenCode 配置管理重构说明

## 📋 概述

本次重构将 Open Switch 工具的配置管理从**内部存储**改为**直接管理 OpenCode 官方配置文件**，实现了与 OpenCode CLI 的完全同步。

---

## 🔄 架构变更

### 旧架构（问题）
```
工具内部配置：~/.Open Switch/opencode.json （主存储）
                    ↓ 手动同步
OpenCode配置：~/.config/opencode/opencode.json （仅同步，不读）
```

**问题**：
- 工具和 OpenCode 配置不同步
- 用户在 OpenCode 中配置的 provider 不会显示在工具中
- 需要手动同步操作，容易出错

### 新架构（正确）
```
OpenCode配置：~/.config/opencode/opencode.json （主存储，直接读写）
                    ↓ 自动备份
工具备份：~/.opencode/opencode.json （仅备份）
```

**优势**：
- ✅ 工具和 OpenCode 完全同步
- ✅ 所有修改直接作用于 OpenCode 配置
- ✅ 自动备份到 `~/.opencode/opencode.json`
- ✅ 支持删除、编辑 provider 和 model

---

## 📁 配置文件路径

### 主配置文件
```
~/.config/opencode/opencode.json
```
- **用途**：OpenCode 官方主配置文件
- **权限**：工具直接读写
- **作用**：所有 provider、model、主题等配置的唯一真实来源

### 备份文件
```
~/.opencode/opencode.json
```
- **用途**：自动备份
- **更新时机**：每次写入主配置时自动同步
- **作用**：作为配置历史和恢复点

### 废弃文件
```
~/.config/opencode/package.json
~/.Open Switch/opencode.json
```
- **状态**：已废弃，不再使用
- **处理**：代码中保留兼容性，但不再写入

---

## 🛠️ 代码修改

### 核心文件
```
src-tauri/src/config/opencode_manager.rs
```

### 主要修改点

#### 1. 结构体简化
```rust
pub struct OpenCodeConfigManager {
    // OpenCode 官方主配置文件
    opencode_config_json: PathBuf,  // ~/.config/opencode/opencode.json
    
    // 备份路径
    home_dir: PathBuf,              // ~/.opencode/
    home_json: PathBuf,             // ~/.opencode/opencode.json
    
    // 废弃字段（保留兼容）
    config_json_alt: PathBuf,       // ~/.config/opencode/package.json
}
```

#### 2. 读取配置（read_config）
```rust
pub fn read_config(&self) -> Result<OpenCodeConfig, String> {
    let config_path = &self.opencode_config_json;  // 直接读取官方配置
    // ...
}
```

#### 3. 写入配置（write_config）
```rust
pub fn write_config(&self, config: &OpenCodeConfig) -> Result<(), String> {
    // 1. 写入官方配置
    fs::write(&self.opencode_config_json, content)?;
    
    // 2. 自动备份到 ~/.opencode/
    self.backup_to_home()?;
    
    Ok(())
}
```

#### 4. 自动备份（backup_to_home）
```rust
fn backup_to_home(&self) -> Result<(), String> {
    if !self.home_dir.exists() {
        fs::create_dir_all(&self.home_dir)?;
    }
    fs::copy(&self.opencode_config_json, &self.home_json)?;
    Ok(())
}
```

---

## 🚀 功能支持

### 已实现功能

#### Provider 管理
- ✅ **读取所有 provider**：`get_all_providers()`
- ✅ **添加 provider**：`add_provider()`
- ✅ **修改 provider 元数据**：`update_provider_metadata()`
- ✅ **删除 provider**：`delete_provider()`
- ✅ **启用/禁用 provider**：`toggle_provider()`

#### Model 管理
- ✅ **读取 provider 的所有 model**：`get_models()`
- ✅ **添加 model**：`add_model()`
- ✅ **修改 model**：`update_model()`
- ✅ **删除 model**：`delete_model()`

#### 自动功能
- ✅ **自动备份**：每次写入时自动备份到 `~/.opencode/opencode.json`
- ✅ **自动创建目录**：确保 `~/.config/opencode/` 目录存在
- ✅ **配置同步**：所有修改立即反映在 OpenCode CLI 中

---

## 📊 当前配置验证

### 已配置的 i7 relay provider

| Provider Key | 名称 | 协议 | URL | 模型数 |
|-------------|------|------|-----|--------|
| `i7 Claude` | i7 Claude | `@ai-sdk/anthropic` | `https://i7dc.com/api/v1` | 5 |
| `i7 Gemini` | i7 Gemini | `@ai-sdk/anthropic` | `https://i7dc.com/api/v1` | 3 |
| `i7 Relay` | i7 Relay | `@ai-sdk/anthropic` | `https://i7dc.com/api/v1` | 4 |
| `i7 code` | i7 code | `@ai-sdk/anthropic` | `https://i7dc.com/api/v1` | 4 |

### 统一配置规范
- **协议**：全部使用 `@ai-sdk/anthropic`
- **URL**：全部使用 `https://i7dc.com/api/v1`
- **API Key**：各自独立的 i7-relay 密钥

---

## ✅ 测试验证

### 1. 配置文件验证
```powershell
# 检查配置文件内容
Get-Content "C:\Users\Administrator\.config\opencode\opencode.json" | ConvertFrom-Json | Select-Object -ExpandProperty provider

# 验证 provider 数量
$cfg = Get-Content "C:\Users\Administrator\.config\opencode\opencode.json" -Raw | ConvertFrom-Json
($cfg.provider.PSObject.Properties | Measure-Object).Count  # 应该是 4
```

### 2. 工具 UI 验证
1. 启动 Open Switch 工具
2. 导航到 Provider 管理页面
3. 验证显示 4 个 i7 relay provider
4. 测试编辑、删除功能

### 3. OpenCode CLI 验证
```bash
# 列出所有 provider
opencode provider list

# 测试使用 i7 Claude provider
opencode --provider "i7 Claude" --model "claude-4.5-sonnet" "Hello"
```

---

## 🔍 故障排查

### 工具报错：“解析全局配置失败: expected value at line 1 column 1”

**原因**：配置文件开头包含 UTF-8 BOM（Byte Order Mark）。PowerShell 的 `ConvertTo-Json | Set-Content` 默认会添加 BOM。

**解决方法 1：手动移除 BOM**：
```powershell
# 读取配置
$configPath = "$env:USERPROFILE\.config\opencode\opencode.json"
$config = Get-Content $configPath -Raw | ConvertFrom-Json

# 重新保存为无 BOM 的 UTF-8
$json = $config | ConvertTo-Json -Depth 100
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($configPath, $json, $utf8NoBom)

Write-Output "✅ 已移除 BOM"
```

**解决方法 2：升级工具**：
工具现已自动处理 BOM，重新构建并启动工具即可。

### 工具中看不到 provider？

**检查配置文件路径**：
```powershell
Test-Path "C:\Users\Administrator\.config\opencode\opencode.json"
```

**检查配置文件格式**：
```powershell
$cfg = Get-Content "C:\Users\Administrator\.config\opencode\opencode.json" -Raw | ConvertFrom-Json
$cfg.provider.PSObject.Properties.Name
```

**检查是否有 BOM**：
```powershell
$bytes = [System.IO.File]::ReadAllBytes("$env:USERPROFILE\.config\opencode\opencode.json")
if ($bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
    Write-Output "⚠️ 发现 UTF-8 BOM，需要移除"
} else {
    Write-Output "✅ 无 BOM，文件正常"
}
```

### OpenCode CLI 报错？

**检查 provider 绑定**：
```
Error: Forbidden: no provider group bound to this API key
```
→ 需要在 i7 relay 后台绑定 API key 到对应的 provider group

**检查协议配置**：
```
Error: 模型不存在
```
→ 确认 provider 使用的协议（`@ai-sdk/anthropic`）和 URL（`https://i7dc.com/api/v1`）正确

---

## 📝 未来改进

### 短期（已完成）
- ✅ 直接读写 OpenCode 官方配置
- ✅ 自动备份机制
- ✅ Provider 和 Model 完整管理

### 中期（计划中）
- [ ] 配置文件冲突检测和合并
- [ ] 多账户支持（profile 管理）
- [ ] 配置导入/导出功能
- [ ] 配置历史和版本管理

### 长期（规划中）
- [ ] 云端配置同步
- [ ] 团队配置共享
- [ ] 配置模板市场

---

## 📞 联系和支持

如有问题或建议，请提交 Issue 或 PR。

**版本**：1.3.1  
**重构日期**：2026-01-23  
**维护者**：Open Switch Team
