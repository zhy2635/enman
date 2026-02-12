# PowerShell 脚本：构建发布版本的 enman
Write-Host "Building release version of enman..." -ForegroundColor Green

# 检查是否安装了 Rust
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Cargo (Rust) is not installed or not in PATH." -ForegroundColor Red
    Write-Host "Please install Rust from https://www.rust-lang.org/tools/install" -ForegroundColor Red
    exit 1
}

# 构建发布版本
Write-Host "Building release binaries..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# 创建发布目录
$releaseDir = "releases"
if (!(Test-Path $releaseDir)) {
    New-Item -ItemType Directory -Path $releaseDir | Out-Null
}

# 获取当前版本
$version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.*)"$').Matches[0].Groups[1].Value
$arch = $ENV:PROCESSOR_ARCHITECTURE
$platform = "windows"
$releaseName = "enman-v${version}-${platform}-${arch}"

$distDir = "${releaseDir}\${releaseName}"
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}
New-Item -ItemType Directory -Path $distDir | Out-Null

# 复制二进制文件 - 使用 Join-Path 确保路径正确
$srcDir = Join-Path -Path "." -ChildPath "target\release"
$enmanExe = Join-Path -Path $srcDir -ChildPath "enman.exe"
$emExe = Join-Path -Path $srcDir -ChildPath "em.exe"

Copy-Item $enmanExe -Destination $distDir
Copy-Item $emExe -Destination $distDir

# 复制文档
Copy-Item "README.md" -Destination $distDir
Copy-Item "LICENSE" -ErrorAction SilentlyContinue -Destination $distDir

Write-Host "Release built successfully in ${distDir}" -ForegroundColor Green
Write-Host "Files included:" -ForegroundColor Cyan
Get-ChildItem $distDir | ForEach-Object { Write-Host "  $($_.Name)" }

# 创建 ZIP 存档
$zipPath = "${releaseDir}\${releaseName}.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath
}

Compress-Archive -Path "${distDir}\*" -DestinationPath $zipPath
Write-Host "Archive created: $zipPath" -ForegroundColor Green