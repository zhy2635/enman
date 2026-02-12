# PowerShell 脚本：设置 enman 的 em 别名
Write-Host "Setting up enman alias 'em' in PowerShell profile..." -ForegroundColor Green

# 检查 PowerShell 配置文件是否存在
$profilePath = $PROFILE

if (-not (Test-Path $profilePath)) {
    Write-Host "Creating new PowerShell profile at: $profilePath" -ForegroundColor Yellow
    New-Item -ItemType File -Path $profilePath -Force
}

# 读取现有内容
$existingContent = Get-Content $profilePath -ErrorAction SilentlyContinue

# 检查是否已存在别名设置
$aliasExists = $existingContent | Select-String -Pattern "doskey em=enman" -Quiet

if ($aliasExists) {
    Write-Host "em alias already exists in profile." -ForegroundColor Yellow
} else {
    # 添加别名设置
    $aliasLine = "doskey em=enman `$*"
    Add-Content -Path $profilePath -Value $aliasLine
    
    Write-Host "Added 'em' alias to PowerShell profile." -ForegroundColor Green
    Write-Host "Alias definition: doskey em=enman `$*" -ForegroundColor Cyan
}

Write-Host "Please restart your PowerShell session or run: . $profilePath" -ForegroundColor Magenta
Write-Host "After restarting, you can use 'em' as an alias for 'enman'." -ForegroundColor Magenta