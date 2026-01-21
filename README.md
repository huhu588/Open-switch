# Open Switch - OpenCode 配置管理工具

OpenCode 配置管理工具，支持多 Provider 和 Model 管理(该项目为二开，参考opcd和cc switch)

## 支持

- OpenCode

## 开发

```bash
npm run tauri dev
```

## 构建

```bash
npm run tauri build
```

## 功能

- 🔄 多 Provider 配置管理
- 🤖 模型列表管理与站点检测
- 💾 WebDAV 云同步备份
- 🎨 现代化桌面应用界面
- 📁 项目配置优先（可独立管理项目级配置）
- 🔧 MCP 服务器配置管理

## 配置说明

### 配置层级

Open Switch 支持两级配置管理：

1. **项目配置** - 存储在 `./.opencode/opencode.json`，仅作用于当前项目
2. **全局配置** - 存储在 `~/.Open Switch/`，作用于所有项目

### 应用配置

在 Provider 列表选择后确认，会弹出应用范围选择对话框：

- **当前项目** - 仅更新项目配置 `./.opencode/`
- **全局配置** - 仅更新全局配置 `~/.opencode/`
- **两者都应用** - 同时更新两级配置

## License

MIT
