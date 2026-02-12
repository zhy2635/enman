# enman 安装脚本 for Windows
#
# 用法:
#   在 PowerShell 中运行:
#     Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/yourname/enman/main/install.ps1')
#   或者下载脚本后运行:
#     .\install.ps1

param(
    [string]$InstallPath = "$env:USERPROFILE\.enman",
    [string]$ShimsPath = "$env:USERPROFILE\.enman\shims",
    [switch]$AddToPath = $true
)

Write-Host "正在安装 enman..." -ForegroundColor Green

# 检查是否安装了 Rust
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "未检测到 Cargo。请先安装 Rust 工具链：" -ForegroundColor Yellow
    Write-Host "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" -ForegroundColor Cyan
    Write-Host "然后重启终端以继续安装。" -ForegroundColor Yellow
    exit 1
}

# 克隆或更新 enman 仓库
$repoPath = Join-Path $env:TEMP "enman-repo"
if (Test-Path $repoPath) {
    Write-Host "更新 enman 仓库..." -ForegroundColor Cyan
    Set-Location $repoPath
    git pull origin main
} else {
    Write-Host "克隆 enman 仓库..." -ForegroundColor Cyan
    git clone "https://github.com/yourname/enman.git" $repoPath
    Set-Location $repoPath
}

# 构建 enman
Write-Host "正在构建 enman..." -ForegroundColor Cyan
$buildResult = cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "构建失败。退出。" -ForegroundColor Red
    exit $LASTEXITCODE
}

# 创建安装目录
Write-Host "创建安装目录..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $InstallPath -ErrorAction SilentlyContinue | Out-Null
$binPath = Join-Path $InstallPath "bin"
New-Item -ItemType Directory -Path $binPath -ErrorAction SilentlyContinue | Out-Null

# 复制二进制文件
Copy-Item (Join-Path $repoPath "target\release\enman.exe") $binPath
Copy-Item (Join-Path $repoPath "target\release\em.exe") $binPath

# 创建 shims 目录
New-Item -ItemType Directory -Path $ShimsPath -ErrorAction SilentlyContinue | Out-Null

# 初始化 enman
Write-Host "初始化 enman..." -ForegroundColor Cyan
& (Join-Path $binPath "enman.exe") init

# 添加到 PATH（如果需要）
if ($AddToPath) {
    Write-Host "添加到 PATH..." -ForegroundColor Cyan
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$ShimsPath*") {
        [Environment]::SetEnvironmentVariable("PATH", "$ShimsPath;$currentPath", "User")
        Write-Host "已将 $ShimsPath 添加到用户 PATH。" -ForegroundColor Green
        
        # 为当前会话也添加
        $env:PATH = "$ShimsPath;$env:PATH"
    } else {
        Write-Host "PATH 已包含 $ShimsPath。" -ForegroundColor Green
    }
}

Write-Host "enman 安装完成！" -ForegroundColor Green
Write-Host "请重启终端或运行: `$env:PATH = `"$ShimsPath;$env:PATH`"" -ForegroundColor Cyan
Write-Host ""
Write-Host "现在您可以使用以下命令开始:" -ForegroundColor Cyan
Write-Host "  enman install node@18.17.0  # 安装 Node.js"
Write-Host "  enman global node@18.17.0   # 设置全局版本"
Write-Host "  node --version              # 验证安装"