# Ai Switch 架构设计

> 版本: v1.4.0 | 更新: 2026-01-29

## 概述

Ai Switch 是一个用于管理 OpenCode 配置的桌面应用工具，支持多 Provider、MCP 服务器、技能 (Skills)、规则 (Rule) 的统一管理，以及配置的备份与恢复。

## 技术栈

### 前端 (Tauri + Vue)
| 类型 | 技术 | 版本 |
|------|------|------|
| 桌面框架 | Tauri | 2.0 |
| 前端框架 | Vue + TypeScript | 3.4 |
| 状态管理 | Pinia | 2.1 |
| 路由 | Vue Router | 4.2 |
| 样式 | Tailwind CSS | 3.4 |
| 国际化 | Vue I18n | 9.14 |
| 构建工具 | Vite | 5.0 |

### 后端 (Rust)
| 类型 | 技术 | 说明 |
|------|------|------|
| 异步运行时 | Tokio | 异步 IO |
| HTTP 客户端 | Reqwest | API 调用 |
| 序列化 | Serde + serde_json | JSON 处理 |
| 错误处理 | thiserror | 类型化错误 |

## 目录结构

### 前端结构 (src/)
```
src/
├── main.ts                  # Vue 入口
├── App.vue                  # 根组件
├── vite-env.d.ts            # Vite 类型声明
│
├── assets/                  # 静态资源
│   └── iconfont/            # 图标字体
│       └── iconfont.js
│
├── components/              # 公共组件
│   ├── SvgIcon.vue          # SVG 图标组件
│   ├── ApplyDialog.vue      # 配置应用对话框
│   ├── ConfirmDialog.vue    # 确认对话框
│   ├── DeepLinkDialog.vue   # 深链接配置对话框
│   ├── DeployedProvidersDialog.vue  # 已部署服务商对话框
│   ├── DetailPanel.vue      # 详情面板
│   ├── FetchModelsDialog.vue  # 模型获取对话框
│   ├── LanguageSwitch.vue   # 语言切换器
│   ├── ModelDialog.vue      # 模型编辑对话框
│   ├── ModelList.vue        # 模型列表
│   ├── ModelTypeSelector.vue  # 模型类型选择器
│   ├── ProviderDialog.vue   # Provider 编辑对话框
│   ├── ProviderList.vue     # Provider 列表
│   └── UpdateDialog.vue     # 更新提示对话框
│
├── views/                   # 页面视图
│   ├── ProvidersView.vue    # Provider 管理页
│   ├── McpView.vue          # MCP 服务器管理页
│   ├── SkillView.vue        # 技能管理页
│   ├── OhMyView.vue         # Oh My 规则管理页
│   ├── BackupView.vue       # 备份恢复页
│   └── StatusView.vue       # 状态信息页
│
├── stores/                  # Pinia 状态库
│   └── providers.ts         # Provider/Model 状态
│
├── router/                  # 路由配置
│   └── index.ts
│
├── config/                  # 前端配置
│   ├── providerPresets.ts   # Provider 预设配置
│   └── modelTypes.ts        # 模型类型配置
│
├── types/                   # TypeScript 类型
│   └── index.ts
│
├── i18n/                    # 国际化
│   ├── index.ts
│   └── locales/
│       ├── en.ts            # 英文
│       └── zh-CN.ts         # 简体中文
│
└── styles/                  # 样式文件
    └── main.css             # 全局样式 + Tailwind
```

### 后端结构 (src-tauri/src/)
```
src-tauri/src/
├── main.rs                  # Tauri 入口
├── lib.rs                   # 库入口，注册所有命令
├── error.rs                 # 全局错误处理
│
├── commands/                # Tauri 命令 (IPC 接口)
│   ├── mod.rs               # 模块导出
│   ├── provider.rs          # Provider 相关命令
│   ├── model.rs             # Model 相关命令
│   ├── mcp.rs               # MCP 服务器命令
│   ├── skills.rs            # Skills 管理命令
│   ├── rule.rs              # Rule 管理命令
│   ├── ohmy.rs              # Oh My 规则命令
│   ├── deeplink.rs          # 深链接处理命令
│   ├── settings.rs          # 设置相关命令
│   ├── backup.rs            # 备份导入导出命令
│   └── status.rs            # 状态信息命令
│
├── config/                  # 配置管理模块
│   ├── mod.rs               # 模块导出 + ConfigError
│   ├── manager.rs           # 核心配置管理器 ConfigManager
│   ├── opencode_manager.rs  # OpenCode 配置管理
│   ├── mcp_manager.rs       # MCP 配置管理
│   ├── models.rs            # 数据结构定义
│   └── detector.rs          # 配置检测器
│
└── assets/                  # 内置资源
    └── rules/               # 内置规则模板
        └── riper-5-cn.md
```

## 核心模块详解

### ConfigManager (配置管理器)
文件: `src-tauri/src/config/manager.rs`

主管理器，协调各子管理器：
```rust
pub struct ConfigManager {
    global_config_file: PathBuf,           // ~/.Ai Switch/config.json
    opencode_manager: OpenCodeConfigManager,
    mcp_manager: McpConfigManager,
}
```

**职责**:
- 读写全局配置 `~/.Ai Switch/config.json`
- 提供 `opencode()` 和 `mcp()` 访问子管理器
- 管理配置应用 (项目/全局)

### OpenCodeConfigManager
文件: `src-tauri/src/config/opencode_manager.rs`

**职责**:
- 管理 Provider 和 Model 配置
- 同步配置到 OpenCode 全局/项目
- 配置文件: `~/.Ai Switch/opencode.json`

### McpConfigManager
文件: `src-tauri/src/config/mcp_manager.rs`

**职责**:
- 管理 MCP 服务器配置
- 支持本地 (Local) 和远程 (Remote) 服务器
- 同步到 OpenCode 的 mcpServers 配置

## Tauri 命令 (IPC 接口)

所有命令在 `lib.rs` 中注册，前端通过 `invoke()` 调用。

### Provider 命令
| 命令 | 说明 |
|------|------|
| `get_providers` | 获取所有 Provider 列表 |
| `get_provider` | 获取单个 Provider 详情 |
| `add_provider` | 添加 Provider |
| `update_provider` | 更新 Provider |
| `delete_provider` | 删除 Provider |
| `toggle_provider` | 切换启用状态 |
| `check_provider_applied` | 检查是否已应用 |
| `apply_config` | 应用配置到全局/项目 |

### Model 命令
| 命令 | 说明 |
|------|------|
| `get_models` | 获取 Provider 下的模型列表 |
| `add_model` | 添加模型 |
| `delete_model` | 删除模型 |
| `fetch_site_models` | 从站点获取可用模型 |
| `add_models_batch` | 批量添加模型 |

### MCP 命令
| 命令 | 说明 |
|------|------|
| `get_mcp_servers` | 获取所有 MCP 服务器 |
| `get_mcp_server` | 获取单个 MCP 服务器详情 |
| `add_mcp_server` | 添加 MCP 服务器 |
| `update_mcp_server` | 更新 MCP 服务器 |
| `delete_mcp_server` | 删除 MCP 服务器 |
| `toggle_mcp_server` | 切换启用状态 |
| `sync_mcp_config` | 同步到 OpenCode |
| `get_recommended_mcp_servers` | 获取推荐 MCP 服务器 |
| `check_mcp_server_health` | 检查服务器健康状态 |

### skills 命令
| 命令 | 说明 |
|------|------|
| `get_installed_skills` | 获取已安装技能 |
| `get_recommended_skills` | 获取推荐技能 |
| `install_skills` | 安装技能 |
| `delete_skills` | 删除技能 |
| `read_skills_content` | 读取技能内容 |

### Rule 命令
| 命令 | 说明 |
|------|------|
| `get_installed_rules` | 获取已安装规则 |
| `get_recommended_rules` | 获取推荐规则 |
| `install_rule` | 安装规则 |
| `delete_rule` | 删除规则 |
| `read_rule_content` | 读取规则内容 |
| `save_rule_content` | 保存规则内容 |
| `toggle_rule_enabled` | 切换规则启用状态 |

### 备份命令
| 命令 | 说明 |
|------|------|
| `create_backup` | 创建备份数据 |
| `export_backup` | 导出备份到文件 |
| `preview_backup` | 预览备份文件 |
| `import_backup` | 导入备份 |

### 状态命令
| 命令 | 说明 |
|------|------|
| `get_status` | 获取应用状态 |
| `get_version` | 获取版本信息 |
| `get_local_ip` | 获取本地 IP |

## 前端状态管理

### providers store
文件: `src/stores/providers.ts`

使用 Pinia 管理 Provider 和 Model 状态：

```typescript
interface State {
  providers: ProviderItem[]      // Provider 列表
  selectedProvider: string | null // 当前选中
  models: ModelItem[]            // 当前 Provider 的 Model
  selectedModel: string | null   // 当前选中的 Model
  loading: boolean
  error: string | null
}
```

**主要方法**:
- `loadProviders()` - 加载 Provider 列表
- `loadModels()` - 加载 Model 列表
- `addProvider()` / `updateProvider()` / `deleteProvider()`
- `addModel()` / `deleteModel()`
- `fetchSiteModels()` - 从站点拉取模型
- `applyConfig()` - 应用配置

## 路由结构

| 路径 | 视图 | 说明 |
|------|------|------|
| `/` | ProvidersView | Provider 和 Model 管理 (默认页) |
| `/mcp` | McpView | MCP 服务器管理 |
| `/skills` | SkillView | Skills 技能管理 |
| `/ohmy` | OhMyView | Oh My 规则管理 |
| `/backup` | BackupView | 备份与恢复 |
| `/status` | StatusView | 状态信息 |

## 配置文件说明

### 配置层级
1. **应用配置**: `~/.Ai Switch/` - Ai Switch 内部配置
2. **项目配置**: `./.opencode/opencode.json` - 作用于当前项目
3. **全局配置**: `~/.config/opencode/` - 作用于所有项目

项目配置优先于全局配置。

### Ai Switch 内部配置结构
```
~/.Ai Switch/
├── config.json          # 全局配置 (激活状态等)
├── opencode.json        # Provider/Model 配置存储
└── mcp/                 # MCP 服务器配置
    ├── server1.json
    └── server2.json
```

### OpenCode 配置格式
```json
{
  "provider": {
    "provider_name": {
      "apiKey": "your-api-key",
      "baseURL": "https://api.example.com"
    }
  },
  "models": {
    "model_name": {
      "provider": "provider_name",
      "model": "model-id"
    }
  },
  "mcpServers": {
    "server_name": {
      "command": "npx",
      "args": ["-y", "@example/mcp-server"]
    }
  }
}
```

### 备份文件格式
```json
{
  "version": "1.0.0",
  "created_at": "2025-01-20T00:00:00Z",
  "app_name": "Ai Switch",
  "providers": [...],
  "mcp_servers": [...],
  "rules": [...],
  "skills": [...]
}
```

## 开发指南

### 环境要求
- Node.js >= 18
- Rust >= 1.70
- pnpm / npm / yarn

### 开发命令
```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建发布版
npm run tauri build
```

### 添加新的 Tauri 命令
1. 在 `src-tauri/src/commands/` 中创建或编辑模块
2. 在 `mod.rs` 中导出新命令
3. 在 `lib.rs` 的 `invoke_handler` 中注册命令
4. 前端使用 `invoke('command_name', { params })` 调用

### 添加新页面
1. 在 `src/views/` 中创建 Vue 组件
2. 在 `src/router/index.ts` 中添加路由
3. 在 `src/i18n/locales/` 中添加翻译

### 添加新的配置管理功能
1. 在 `src-tauri/src/config/models.rs` 中定义数据结构
2. 在相应的 manager 中实现管理方法
3. 在 commands 中暴露 Tauri 命令
4. 前端在 store 或组件中调用
