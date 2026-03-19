# 统一构建脚本 - AI Switch 融合版
# 按顺序编译 sub2api Go 二进制、前端、Rust 后端

param(
    [switch]$Release,
    [switch]$SkipSub2api
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "=== AI Switch 融合版构建 ===" -ForegroundColor Cyan

# Step 1: 编译 sub2api Go 二进制
if (-not $SkipSub2api) {
    Write-Host "`n[Step 1/3] 编译 Sub2api Go 后端..." -ForegroundColor Yellow
    & "$PSScriptRoot\build-sub2api.ps1" -Target windows -EmbedFrontend
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Sub2api 编译失败" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "`n[Step 1/3] 跳过 Sub2api 编译" -ForegroundColor Gray
}

# Step 2: 前端 + Rust 后端构建
Write-Host "`n[Step 2/3] 编译 React 前端和 Rust 后端..." -ForegroundColor Yellow
Push-Location $ProjectRoot

if ($Release) {
    Write-Host "  使用 Release 模式..."
    npx tauri build
} else {
    Write-Host "  使用 Dev 模式..."
    npx tauri build --debug
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri 构建失败" -ForegroundColor Red
    Pop-Location
    exit 1
}

Pop-Location

# Step 3: 验证
Write-Host "`n[Step 3/3] 构建完成" -ForegroundColor Green

$msiPath = Get-ChildItem -Path "$ProjectRoot\src-tauri\target\release\bundle\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
$nsisPath = Get-ChildItem -Path "$ProjectRoot\src-tauri\target\release\bundle\nsis\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

if ($msiPath) {
    Write-Host "  MSI: $($msiPath.FullName) ($([math]::Round($msiPath.Length / 1MB, 2)) MB)" -ForegroundColor Green
}
if ($nsisPath) {
    Write-Host "  NSIS: $($nsisPath.FullName) ($([math]::Round($nsisPath.Length / 1MB, 2)) MB)" -ForegroundColor Green
}

Write-Host "`n=== 构建完成 ===" -ForegroundColor Cyan
