# OpenCode 配置验证测试脚本
# 用于验证 Open Switch 工具是否正确读取 OpenCode 官方配置

Write-Output "=== OpenCode 配置验证测试 ===`n"

# 1. 检查配置文件是否存在
Write-Output "1. 检查配置文件..."
$mainConfig = "$env:USERPROFILE\.config\opencode\opencode.json"
$backupConfig = "$env:USERPROFILE\.opencode\opencode.json"

if (Test-Path $mainConfig) {
    Write-Output "  ✅ 主配置文件存在: $mainConfig"
} else {
    Write-Output "  ❌ 主配置文件不存在: $mainConfig"
    exit 1
}

if (Test-Path $backupConfig) {
    Write-Output "  ✅ 备份配置文件存在: $backupConfig"
} else {
    Write-Output "  ⚠️  备份配置文件不存在: $backupConfig"
}

Write-Output ""

# 2. 读取并解析配置
Write-Output "2. 读取配置内容..."
try {
    $config = Get-Content $mainConfig -Raw | ConvertFrom-Json
    Write-Output "  ✅ 配置文件解析成功"
} catch {
    Write-Output "  ❌ 配置文件解析失败: $_"
    exit 1
}

Write-Output ""

# 3. 检查 provider 配置
Write-Output "3. 检查 Provider 配置..."
$providers = $config.provider.PSObject.Properties

if ($providers.Count -eq 0) {
    Write-Output "  ⚠️  未找到任何 provider"
} else {
    Write-Output "  ✅ 找到 $($providers.Count) 个 provider:`n"
    
    foreach ($prop in $providers) {
        $key = $prop.Name
        $prov = $prop.Value
        
        Write-Output "  [$key]"
        Write-Output "    名称: $($prov.name)"
        Write-Output "    协议: $($prov.npm)"
        
        if ($prov.options -and $prov.options.baseURL) {
            Write-Output "    URL: $($prov.options.baseURL)"
        }
        
        if ($prov.options -and $prov.options.apiKey) {
            $keyPrefix = $prov.options.apiKey.Substring(0, [Math]::Min(20, $prov.options.apiKey.Length))
            Write-Output "    API Key: $keyPrefix..."
        }
        
        if ($prov.models) {
            $modelCount = ($prov.models.PSObject.Properties | Measure-Object).Count
            Write-Output "    模型数: $modelCount"
        }
        
        Write-Output ""
    }
}

# 4. 验证 i7 relay provider 配置
Write-Output "4. 验证 i7 relay provider 配置..."
$i7Providers = @("i7 Claude", "i7 Gemini", "i7 Relay", "i7 code")
$expectedProtocol = "@ai-sdk/anthropic"
$expectedUrl = "https://i7dc.com/api/v1"

$allCorrect = $true

foreach ($provName in $i7Providers) {
    $prov = $config.provider.$provName
    
    if (-not $prov) {
        Write-Output "  ⚠️  未找到 provider: $provName"
        continue
    }
    
    $issues = @()
    
    if ($prov.npm -ne $expectedProtocol) {
        $issues += "协议不正确 (期望: $expectedProtocol, 实际: $($prov.npm))"
        $allCorrect = $false
    }
    
    if ($prov.options.baseURL -ne $expectedUrl) {
        $issues += "URL 不正确 (期望: $expectedUrl, 实际: $($prov.options.baseURL))"
        $allCorrect = $false
    }
    
    if ($issues.Count -eq 0) {
        Write-Output "  ✅ $provName 配置正确"
    } else {
        Write-Output "  ❌ $provName 配置有误:"
        foreach ($issue in $issues) {
            Write-Output "     - $issue"
        }
    }
}

Write-Output ""

# 5. 比较主配置和备份
Write-Output "5. 比较主配置和备份..."
if (Test-Path $backupConfig) {
    $mainHash = (Get-FileHash $mainConfig -Algorithm MD5).Hash
    $backupHash = (Get-FileHash $backupConfig -Algorithm MD5).Hash
    
    if ($mainHash -eq $backupHash) {
        Write-Output "  ✅ 主配置和备份文件一致"
    } else {
        Write-Output "  ⚠️  主配置和备份文件不一致（可能需要重新启动工具以同步）"
    }
} else {
    Write-Output "  ⚠️  备份文件不存在，跳过比较"
}

Write-Output ""

# 6. 总结
Write-Output "=== 测试总结 ==="
if ($allCorrect -and $providers.Count -gt 0) {
    Write-Output "✅ 所有测试通过！配置正确。"
    exit 0
} else {
    Write-Output "⚠️  部分测试未通过，请检查配置。"
    exit 1
}
