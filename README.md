# ai switch

> 版本: v0.16.3 | 更新: 2026-03-19

一款**通用的 AI IDE 账号管理工具**，目前支持 **Antigravity**、**Codex**、**GitHub Copilot**、**Windsurf**、**Kiro**、**Cursor**、**Gemini Cli**、**CodeBuddy**、**CodeBuddy CN**、**WorkBuddy**、**Qoder** 和 **Trae**，并支持多账号多实例并行运行。

> 本工具旨在帮助用户高效管理多个 AI IDE 账号，支持一键切换、配额监控、自动唤醒与多开实例并行运行，助您充分利用不同账号的资源。

**功能**：一键切号 · 多账号管理 · 多开实例 · 配额监控 · 唤醒任务 · 设备指纹 · 插件联动 · API 网关 · 本地代理 · Sub2api 中转 · GitHub Copilot 管理 · Windsurf 管理 · Kiro 管理 · Cursor 管理 · Gemini Cli 管理 · CodeBuddy 管理 · CodeBuddy CN 管理 · WorkBuddy 管理 · Qoder 管理 · Trae 管理

**语言**：支持 17 种语言

🇺🇸 English · 🇨🇳 简体中文 · 繁體中文 · 🇯🇵 日本語 · 🇩🇪 Deutsch · 🇪🇸 Español · 🇫🇷 Français · 🇮🇹 Italiano · 🇰🇷 한국어 · 🇧🇷 Português · 🇷🇺 Русский · 🇹🇷 Türkçe · 🇵🇱 Polski · 🇨🇿 Čeština · 🇸🇦 العربية · 🇻🇳 Tiếng Việt

---
---
项目参考
1. https://github.com/farion1231/cc-switch — Claude / Codex / Gemini 等 Key 配置
2. https://github.com/jlcodes99/cockpit-tools — 多账号切换（本项目主体）
3. https://github.com/qxcnm/Codex-Manager — Codex 反代（已集成至 Gateway 模块）
4. https://github.com/Wei-Shaw/sub2api — Claude / OpenAI / Gemini / Antigravity 中转（已集成至 Sub2api 模块）
5.https://github.com/libaxuan/cursor2api-go    cursor官网逆向出免费的模型
6.（ cursor：  https://github.com/ibrahim317/cursor-chat-transfer）（https://github.com/lohasle/AI-Conversation-Viewer）
获取ai（例如cursor，trae等）本地对话记录，可以迁移至另一台电脑   
7.（https://github.com/junhoyeo/tokscale）（https://github.com/ramo-dev/tokwatch）可以获取ai本地token使用量


## 安全性与隐私（简明版）

下面是最关心的几个问题，尽量用直白语言说明：

- **这是本地桌面工具**：不需要单独注册平台账号，也不依赖项目自建云端来存你的账号列表。
- **数据主要保存在本机**：
 

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

这将启动：
- Vite 开发服务器（前端热重载）
- Tauri 应用窗口
- Rust 代码自动重新编译（修改后）

### 构建产物

```bash
npm run tauri build
```

构建产物将输出到 `src-tauri/target/release/bundle/` 目录。

### 类型检查

```bash
npm run typecheck
```

### 版本同步

在发布前，确保所有配置文件中的版本号一致：

```bash
npm run sync-version
```

---

## 项目结构

```
Open Switch/
├── src/                              # React 前端源码
│   ├── components/                  # 可复用组件
│   ├── pages/                       # 页面组件
│   │   ├── DashboardPage.tsx        # 仪表盘
│   │   ├── AccountsPage.tsx         # Antigravity 账号
│   │   ├── Codex/Copilot/...        # 各平台账号页
│   │   ├── GatewayDashboardPage.tsx # Gateway 仪表盘
│   │   ├── GatewayAccountPoolPage   # Gateway 账号池
│   │   ├── GatewayApiKeysPage.tsx   # API Key 管理
│   │   ├── GatewayRequestLogPage    # 请求日志
│   │   └── Sub2apiPage.tsx          # Sub2api 管理
│   ├── stores/                      # Zustand 状态管理
│   │   ├── useGatewayStore.ts       # Gateway 状态
│   │   ├── useSub2apiStore.ts       # Sub2api 状态
│   │   └── use*AccountStore.ts      # 各平台账号状态
│   ├── services/                    # API 服务层
│   ├── types/                       # TypeScript 类型定义
│   ├── hooks/                       # React Hooks
│   ├── utils/                       # 工具函数
│   ├── i18n/                        # 国际化配置
│   └── App.tsx                      # 应用入口
│
├── src-tauri/                        # Rust 后端源码
│   ├── src/
│   │   ├── commands/               # Tauri 命令（IPC 接口）
│   │   │   ├── gateway.rs          # Gateway 命令
│   │   │   ├── subprocess.rs       # Sub2api 子进程命令
│   │   │   ├── opencode/           # OpenCode 配置命令
│   │   │   └── ...                 # 各平台账号/实例命令
│   │   ├── modules/                # 业务模块
│   │   │   ├── gateway/            # API 网关
│   │   │   │   ├── account_pool.rs          # 账号池与负载均衡
│   │   │   │   ├── account_pool_bridge.rs   # 多平台账号桥接
│   │   │   │   ├── api_key.rs               # API Key 管理
│   │   │   │   ├── proxy.rs                 # 反代代理逻辑
│   │   │   │   ├── protocol_adapter.rs      # 协议适配
│   │   │   │   ├── router.rs                # 路由分发
│   │   │   │   ├── db.rs                    # SQLite 存储
│   │   │   │   └── server.rs                # HTTP 服务器
│   │   │   ├── subprocess/         # 子进程管理
│   │   │   │   ├── sub2api.rs               # Sub2api 启停
│   │   │   │   └── sub2api_sync.rs          # 账号同步到 Sub2api
│   │   │   ├── proxy/              # 本地透明代理
│   │   │   │   ├── server.rs                # 代理服务器
│   │   │   │   ├── handlers.rs              # 请求处理
│   │   │   │   └── usage/                   # 用量统计
│   │   │   ├── opencode_config/    # OpenCode 配置管理
│   │   │   └── ...                 # 账号/OAuth/设备指纹等
│   │   ├── lib.rs                  # 库入口（命令注册）
│   │   └── main.rs                 # 应用入口
│   ├── binaries/                   # 外部二进制（sub2api）
│   └── Cargo.toml                  # Rust 依赖配置
│
├── docs/                            # 项目文档
│   ├── images/                     # 文档图片
│   ├── DONATE.md                   # 赞助文档
│   └── release-process.md          # 发布流程
│
├── package.json                     # Node.js 依赖配置
├── tsconfig.json                    # TypeScript 配置
├── vite.config.ts                  # Vite 配置
└── README.md                       # 项目说明（本文件）
```

---


## 致谢

- Antigravity 账号切号逻辑参考：[Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager)
1. https://github.com/farion1231/cc-switch — Claude / Codex / Gemini 等 Key 配置
2. https://github.com/jlcodes99/cockpit-tools — 多账号切换（本项目主体）
3. https://github.com/qxcnm/Codex-Manager — Codex 反代（已集成至 Gateway 模块）
4. https://github.com/Wei-Shaw/sub2api — Claude / OpenAI / Gemini / Antigravity 中转（已集成至 Sub2api 模块）
5.https://github.com/libaxuan/cursor2api-go    cursor官网逆向出免费的模型
6.（ cursor：  https://github.com/ibrahim317/cursor-chat-transfer）（https://github.com/lohasle/AI-Conversation-Viewer）
获取ai（例如cursor，trae等）本地对话记录，可以迁移至另一台电脑   
7.（https://github.com/junhoyeo/tokscale）（https://github.com/ramo-dev/tokwatch）可以获取ai本地token使用量

感谢项目作者的开源贡献！如果这些项目对你有帮助，也请给他们点个 ⭐ Star 支持一下！

---

---

## 免责声明

本项目仅供个人学习和研究使用。使用本项目即表示您同意：

- 不将本项目用于任何商业用途
- 承担使用本项目的所有风险和责任
- 遵守相关服务条款和法律法规

项目作者对因使用本项目而产生的任何直接或间接损失不承担责任。
