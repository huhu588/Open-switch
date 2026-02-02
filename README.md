# Ai Switch

> OpenCode 配置管理工具 | v1.4.29

一个现代化的桌面应用，用于管理 OpenCode 的 Provider、Model、MCP 服务器、技能和规则配置。

## 功能特性

- **Provider 管理** - 多 Provider 配置，支持 OpenAI、Claude、Gemini 等
- **Model 管理** - 模型列表管理，支持从站点自动获取可用模型
- **MCP 服务器** - MCP 服务器配置与管理
- **技能管理** - 安装和管理 OpenCode 技能
- **规则管理** - 自定义规则配置
- **深链接** - 支持 `aiswitch://` 协议一键配置
- **备份恢复** - 配置导入导出功能
- **多语言** - 支持中文和英文界面
- **自动更新** - 全平台支持（Windows / macOS Intel / macOS Apple Silicon）

## 快速开始

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- npm / pnpm / yarn

### 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

#### 构建注意事项

- `npm run tauri build` 会先执行 `beforeBuildCommand`（`npm run build:tauri`），其中包含 `vue-tsc --noEmit`。
- `vue-tsc` 会在检测到**未使用变量/导入**时直接报错（如 TS6133），导致构建失败。请在提交/构建前清理未使用的变量、计算属性或导入。
- 建议先本地执行 `npm run build` 进行静态检查，避免在打包阶段才失败。

## 配置层级

| 层级 | 路径 | 作用范围 |
|------|------|----------|
| 项目配置 | `./.opencode/opencode.json` | 当前项目 |
| 全局配置 | `~/.config/opencode/opencode.json` | 所有项目 |

> 项目配置优先于全局配置

## 技术栈

- **前端**: Vue 3 + TypeScript + Tailwind CSS + Pinia
- **后端**: Rust + Tauri 2.0
- **构建**: Vite 5

## 发布注意事项

> **重要**: 本节包含所有可能影响软件自动更新功能的注意事项，请仔细阅读。

### 1. 版本号同步（最常见问题）

发布新版本时，**必须同步更新以下三个文件中的版本号**：

| 文件 | 位置 | 用途 |
|------|------|------|
| `src-tauri/tauri.conf.json` | `version` 字段 | Tauri updater 使用，决定自动更新检测 |
| `src-tauri/Cargo.toml` | `version` 字段 | Rust 编译时使用 |
| `package.json` | `version` 字段 | 前端依赖管理 |

**版本号不一致会导致的问题**：
- 界面显示的版本与实际版本不符（用户看到旧版本号）
- 自动更新检测失败（updater 使用 tauri.conf.json 的版本比较）
- 用户无法正常检测或安装新版本

**历史案例（v1.4.27）**：
- `tauri.conf.json` 更新到 1.4.27，但 `Cargo.toml` 仍是 1.4.23
- 用户界面显示 v1.4.23，但 updater 认为已是 v1.4.27
- 导致用户无法检测到新版本

### 2. GitHub Secrets 配置

确保 GitHub 仓库配置了以下 Secrets，否则构建的安装包无法进行签名验证：

| Secret 名称 | 用途 |
|-------------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri 签名私钥，用于签名更新包 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码 |

**签名缺失会导致的问题**：
- `latest.json` 中的 `signature` 字段为空
- 用户下载更新后签名验证失败
- 更新安装被拒绝

### 3. latest.json 文件格式

GitHub Actions 会自动生成 `latest.json`，其格式必须正确：

```json
{
  "version": "1.4.28",
  "notes": "Ai Switch v1.4.28 发布",
  "pub_date": "2026-02-02T03:57:16Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "签名内容（不能为空）",
      "url": "https://github.com/huhu588/Ai-Switch/releases/download/v1.4.29/Ai.Switch_1.4.29_x64-setup.exe"
    }
  }
}
```

**检查要点**：
- `signature` 字段不能为空
- `url` 中的文件名必须与实际上传的文件名完全一致
- 文件名中的空格会被 GitHub 转换为点号（`Ai Switch` → `Ai.Switch`）

### 4. 下载 URL 文件名格式

Tauri 生成的安装包文件名格式：`Ai Switch_X.Y.Z_x64-setup.exe`（带空格）

但 GitHub Release 会将空格转换为点号：`Open.Switch_X.Y.Z_x64-setup.exe`

**build.yml 中已处理此问题**：
```bash
EXENAME=$(echo "$EXENAME" | sed 's/ /./g')
```

如果下载 URL 不匹配，会导致 **404 Not Found** 错误。

### 5. Release 状态

- 只有 **非 draft** 状态的 Release 才会被视为 "latest"
- `releases/latest/download/latest.json` 端点只返回最新的正式 Release
- 如果 Release 是 draft 状态，用户将无法检测到更新

**检查方法**：
1. 访问 https://github.com/huhu588/Ai-Switch/releases
2. 确认最新版本不是 "Draft" 状态
3. 确认 `latest.json` 文件已包含在 Release assets 中

### 6. Updater 端点配置

`src-tauri/tauri.conf.json` 中的 updater 配置：

```json
"plugins": {
  "updater": {
    "pubkey": "公钥内容",
    "endpoints": [
      "https://github.com/huhu588/Ai-Switch/releases/latest/download/latest.json"
    ]
  }
}
```

**注意事项**：
- `pubkey` 必须与签名私钥匹配
- `endpoints` URL 必须正确指向 GitHub releases
- 仓库名称区分大小写（`Ai-Switch` 不是 `ai-switch`）

### 7. 构建产物检查清单

发布后，确认 GitHub Release 中包含以下文件：

**Windows（自动更新）**：
- [ ] `Open.Switch_X.Y.Z_x64-setup.exe` - Windows 安装包
- [ ] `Open.Switch_X.Y.Z_x64-setup.exe.sig` - Windows 签名文件

**macOS（自动更新）**：
- [ ] `Open.Switch.app.tar.gz` - macOS 更新包（Universal Binary）
- [ ] `Open.Switch.app.tar.gz.sig` - macOS 签名文件
- [ ] `Open.Switch_X.Y.Z_universal.dmg` - macOS 手动安装包（可选）

**更新元数据**：
- [ ] `latest.json` - 包含 Windows + macOS 平台配置

### 8. 发布流程（完整版）

```bash
# 1. 同步更新三个文件中的版本号
#    - src-tauri/tauri.conf.json
#    - src-tauri/Cargo.toml  
#    - package.json

# 2. 提交代码
git add .
git commit -m "vX.Y.Z: 更新说明"

# 3. 创建 tag（必须以 v 开头）
git tag vX.Y.Z

# 4. 推送代码
git push origin master

# 5. 推送 tag（触发 GitHub Actions 构建）
git push origin vX.Y.Z

# 6. 等待 GitHub Actions 完成（约 10-20 分钟）
# 7. 检查 Release 页面确认所有文件已上传
# 8. 验证 latest.json 内容正确
```

### 9. 构建配置注意事项

#### Cargo.toml 关键字段

```toml
[package]
name = "ai-switch"
version = "1.4.29"      # 必须与其他文件同步
edition = "2021"        # 只能是 2015/2018/2021/2024，不能用未来年份！
```

**常见构建错误**：

| 错误信息 | 原因 | 解决方案 |
|----------|------|----------|
| `this version of Cargo is older than the 'XXXX' edition` | edition 设置了不支持的年份 | 改为 `edition = "2021"` |
| `failed to parse manifest` | Cargo.toml 语法错误 | 检查 TOML 格式 |
| `unresolved import` | 缺少依赖 | 检查 dependencies |

**历史案例（v1.4.28 首次构建失败）**：
- `edition = "2026"` 导致构建失败
- Cargo 只支持 `2015`, `2018`, `2021`, `2024` editions
- 修复：改为 `edition = "2021"`

#### 各平台构建配置

| 平台 | 构建命令 | 输出格式 | 用于自动更新 |
|------|----------|----------|--------------|
| Windows | `--bundles nsis` | `.exe` + `.exe.sig` | ✅ |
| macOS | `--target universal-apple-darwin` | `.tar.gz` + `.tar.gz.sig` + `.dmg` | ✅ |

**macOS 构建说明**：
- 使用 **Universal Binary** 同时支持 Intel (x86_64) 和 Apple Silicon (aarch64)
- 需要添加两个 Rust targets：`aarch64-apple-darwin` 和 `x86_64-apple-darwin`
- `.tar.gz` 用于自动更新，`.dmg` 用于手动安装
- 未签名应用首次运行需要用户手动允许：`xattr -cr "/Applications/Ai Switch.app"`

**latest.json 平台标识**：
```json
{
  "platforms": {
    "windows-x86_64": { "url": "...exe", "signature": "..." },
    "darwin-aarch64": { "url": "...tar.gz", "signature": "..." },
    "darwin-x86_64": { "url": "...tar.gz", "signature": "..." }
  }
}
```

#### Node.js 和 Rust 版本要求

GitHub Actions 中使用的版本：
- **Node.js**: 20
- **Rust**: stable (最新稳定版)
- **macOS Runner**: macos-14 (支持 Universal Binary 构建)

本地开发环境要求：
- **Node.js**: >= 18
- **Rust**: >= 1.70

#### 依赖版本锁定

- `Cargo.lock` 和 `package-lock.json` 应提交到仓库
- 避免使用 `*` 通配符版本号
- 主要依赖使用固定大版本：`"2"` 而非 `"^2.0.0"`

### 10. 故障排查

#### 通用问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 检测不到新版本 | 版本号不一致 / Release 是 draft | 同步版本号 / 发布 Release |
| 下载失败 404 | URL 文件名不匹配 | 检查 latest.json 中的 url |
| 签名验证失败 | 签名为空或不匹配 | 检查 GitHub Secrets 配置 |
| 安装后版本未变 | 版本号未同步 | 确保三个文件版本一致 |
| GitHub Actions 失败 | Secrets 未配置 / 构建错误 | 检查 Secrets 和 Cargo.toml |
| `edition` 错误 | 使用了不支持的 Rust edition | 改为 2021 |
| npm ci 失败 | Node.js 版本不兼容 | 确保使用 Node.js 18+ |

#### macOS 特定问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| "应用已损坏" | 未签名应用被 Gatekeeper 阻止 | 执行 `xattr -cr "/Applications/Ai Switch.app"` |
| macOS 检测不到更新 | latest.json 缺少 darwin 平台 | 确保构建生成了 .tar.gz.sig 文件 |
| Universal Binary 构建失败 | 缺少 Rust targets | 执行 `rustup target add aarch64-apple-darwin x86_64-apple-darwin` |
| .tar.gz 文件未生成 | 使用了 `--bundles dmg` | 移除该参数，让 Tauri 生成默认 bundle |

### 11. 验证更新功能

发布后，使用以下命令验证 `latest.json`：

```powershell
# PowerShell
Invoke-RestMethod "https://github.com/huhu588/Ai-Switch/releases/latest/download/latest.json"

# 或检查下载 URL 是否有效（应返回 200）
(Invoke-WebRequest -Uri "下载URL" -Method Head).StatusCode
```

## 文档

- [架构设计](docs/ARCHITECTURE.md)
- [深链接配置](docs/DEEP_LINK.md)
- [自动推断 Provider](docs/AUTO_IMPORT_PROVIDER.md)

## License

MIT
