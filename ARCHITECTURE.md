# Open Switch 架构设计

## 概述
Open Switch 是一个用于管理 OpenCode 配置的桌面应用工具。

## 技术栈

### GUI (Tauri + Vue)
- **桌面框架**: Tauri 2.0
- **前端框架**: Vue 3.4 + TypeScript
- **状态管理**: Pinia
- **路由**: Vue Router 4
- **样式**: Tailwind CSS 3.4
- **国际化**: Vue I18n

### 后端 (Rust)
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **序列化**: Serde + serde_json

## 目录结构

```
src/
├── main.ts              # Vue 入口
├── App.vue              # Vue 根组件
├── components/          # Vue 组件
│   ├── ApplyDialog.vue
│   ├── ConfirmDialog.vue
│   ├── DetailPanel.vue
│   ├── FetchModelsDialog.vue
│   ├── LanguageSwitch.vue
│   ├── ModelDialog.vue
│   ├── ModelList.vue
│   ├── ProviderDialog.vue
│   └── ProviderList.vue
├── views/               # Vue 页面
│   ├── BackupView.vue
│   ├── McpView.vue
│   ├── ProvidersView.vue
│   └── StatusView.vue
├── stores/              # Pinia 状态
│   └── providers.ts
├── router/              # Vue Router
│   └── index.ts
├── config/              # 配置预设
│   └── providerPresets.ts
├── types/               # TypeScript 类型
│   └── index.ts
├── i18n/                # 国际化
│   └── locales/
└── styles/              # 样式文件
    └── main.css

src-tauri/
├── src/
│   ├── main.rs          # Tauri 入口
│   ├── lib.rs           # 库入口
│   ├── error.rs         # 错误处理
│   ├── commands/        # Tauri 命令
│   └── config/          # 配置管理
└── tauri.conf.json      # Tauri 配置
```

## 核心模块

### ConfigManager (Rust)
统一的配置管理入口，负责：
- 读写 OpenCode 配置文件
- 管理 Provider 和 Model
- 支持项目级和全局配置

### MCP 管理
独立的 MCP (Model Context Protocol) 服务器配置管理，支持：
- 添加/编辑/删除 MCP 服务器
- 配置应用到项目或全局

## 配置文件

### 配置层级
1. **项目配置**: `./.opencode/opencode.json`
2. **全局配置**: `~/.opencode/opencode.json`

项目配置优先于全局配置。

### 配置格式
```json
{
  "providers": {
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

## 构建

```bash
npm install
npm run tauri build
```
