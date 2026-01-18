# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## 项目概述

`Open Switch` 是一个 Tauri 桌面应用，用于管理 OpenCode 的 Provider 和 Model 配置。

## 常用命令

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 前端预览
npm run dev
npm run build
```

## 架构概览

### 模块结构

```
src/                     # 前端代码 (Vue 3 + TypeScript)
├── main.ts              # Vue 入口
├── App.vue              # 根组件
├── components/          # Vue 组件
├── views/               # 页面视图
├── stores/              # Pinia 状态管理
├── router/              # Vue Router
├── config/              # 配置预设
├── types/               # TypeScript 类型
├── i18n/                # 国际化
└── styles/              # 样式文件

src-tauri/               # 后端代码 (Rust + Tauri)
├── src/
│   ├── main.rs          # Tauri 入口
│   ├── lib.rs           # 库入口
│   ├── error.rs         # 错误类型定义
│   ├── commands/        # Tauri 命令
│   └── config/          # 配置管理
└── tauri.conf.json      # Tauri 配置
```

### 核心数据流

1. **Tauri Commands** (`src-tauri/src/commands/`) 是前后端通信的桥梁
   - 前端通过 `@tauri-apps/api` 调用后端命令
   - 后端处理配置读写和系统操作

2. **配置层级**
   - 全局配置: `~/.opencode/opencode.json`
   - 项目配置: `./.opencode/opencode.json` (当前目录)
   - 项目配置优先于全局配置

3. **前端状态管理** (`src/stores/`)
   - 使用 Pinia 管理应用状态
   - `providers.ts` 管理 Provider 和 Model 数据

### 关键类型

- `OpenCodeConfig`: Provider 和 Model 的配置集合
- `OpenCodeProvider`: 单个 Provider 配置 (name, options, models)
- `McpServer`: MCP 服务器配置

## 开发注意事项

- 前端使用 Vue 3 Composition API
- 样式使用 Tailwind CSS
- 配置文件使用 JSON 格式，Rust 端用 serde 序列化
- 异步操作使用 tokio 运行时
