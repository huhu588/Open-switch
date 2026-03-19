# Sub2api Go 二进制编译脚本
# 用于将 sub2api Go 后端编译为可打包的二进制文件

param(
    [string]$Target = "windows",
    [string]$Arch = "amd64",
    [switch]$EmbedFrontend
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Sub2apiDir = Join-Path $ProjectRoot "sub2api"
$BackendDir = Join-Path $Sub2apiDir "backend"
$FrontendDir = Join-Path $Sub2apiDir "frontend"
$OutputDir = Join-Path $ProjectRoot "src-tauri\binaries"

Write-Host "=== Sub2api 编译脚本 ===" -ForegroundColor Cyan
Write-Host "目标平台: $Target/$Arch"

# 确保输出目录存在
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# 编译前端（可选）
if ($EmbedFrontend) {
    Write-Host "`n[1/3] 编译 Vue 前端..." -ForegroundColor Yellow
    Push-Location $FrontendDir
    try {
        npm install --silent 2>&1 | Out-Null
        npm run build 2>&1 | Out-Null
        Write-Host "  前端编译完成" -ForegroundColor Green
    } finally {
        Pop-Location
    }
} else {
    Write-Host "`n[1/3] 跳过前端编译（使用 -EmbedFrontend 启用）" -ForegroundColor Gray
}

# 确定输出文件名
$targetTriple = switch ($Target) {
    "windows" { "x86_64-pc-windows-msvc" }
    "linux"   { "x86_64-unknown-linux-gnu" }
    "darwin"  { "x86_64-apple-darwin" }
    default   { "x86_64-pc-windows-msvc" }
}

$ext = if ($Target -eq "windows") { ".exe" } else { "" }
$outputFile = "sub2api-$targetTriple$ext"

# 编译 Go 后端
Write-Host "`n[2/3] 编译 Go 后端..." -ForegroundColor Yellow
Push-Location $BackendDir

$env:GOOS = switch ($Target) {
    "windows" { "windows" }
    "linux"   { "linux" }
    "darwin"  { "darwin" }
    default   { "windows" }
}
$env:GOARCH = $Arch
$env:CGO_ENABLED = "1"

$buildTags = "embed"
$ldflags = "-s -w"
$outputPath = Join-Path $OutputDir $outputFile

try {
    Write-Host "  go build -tags $buildTags -ldflags `"$ldflags`" -o $outputPath ./cmd/server"
    go build -tags $buildTags -ldflags $ldflags -o $outputPath ./cmd/server
    Write-Host "  Go 后端编译完成" -ForegroundColor Green
} catch {
    Write-Host "  Go 编译失败: $_" -ForegroundColor Red
    exit 1
} finally {
    Pop-Location
    Remove-Item Env:\GOOS -ErrorAction SilentlyContinue
    Remove-Item Env:\GOARCH -ErrorAction SilentlyContinue
}

# 验证输出
Write-Host "`n[3/3] 验证输出..." -ForegroundColor Yellow
if (Test-Path $outputPath) {
    $size = (Get-Item $outputPath).Length / 1MB
    Write-Host "  输出: $outputPath" -ForegroundColor Green
    Write-Host "  大小: $([math]::Round($size, 2)) MB" -ForegroundColor Green
} else {
    Write-Host "  错误: 输出文件不存在" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== 编译完成 ===" -ForegroundColor Cyan
