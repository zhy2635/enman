# MySQL初始化配置脚本

Write-Host "MySQL 8.0 初始化配置向导" -ForegroundColor Green

$mysqlDir = "$env:USERPROFILE\.enman\installs\mysql\8.0.27"
$configFile = "$mysqlDir\my.ini"

# 启动MySQL服务
Write-Host "`n正在启动MySQL服务器..." -ForegroundColor Yellow
Start-Process -FilePath "$mysqlDir\bin\mysqld.exe" -ArgumentList "--defaults-file=$configFile" -NoNewWindow

Write-Host "`n等待MySQL服务启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# 尝试连接并重置密码
Write-Host "`n连接到MySQL并设置新密码..." -ForegroundColor Yellow

# 获取初始密码
$errorLogFile = Get-ChildItem "$mysqlDir\data\*.err" | Select-Object -First 1
$initialPasswordMatch = Select-String -Path $errorLogFile -Pattern "temporary password" | Select-Object -Last 1
if ($initialPasswordMatch) {
    Write-Host "找到初始密码: $initialPasswordMatch" -ForegroundColor Cyan
} else {
    Write-Host "未能找到初始密码，请检查错误日志: $mysqlDir\data\*.err" -ForegroundColor Red
}

Write-Host "`n要更改root密码，请使用以下命令连接到MySQL:" -ForegroundColor Green
Write-Host "mysql -h 127.0.0.1 -P 3306 -u root -p" -ForegroundColor White
Write-Host "`n然后使用以下SQL命令更改密码:" -ForegroundColor Green
Write-Host "ALTER USER 'root'@'localhost' IDENTIFIED BY '你的新密码';" -ForegroundColor White
Write-Host "FLUSH PRIVILEGES;" -ForegroundColor White

Write-Host "`n要在my.ini中更改端口或其他配置，请编辑此文件:" -ForegroundColor Green
Write-Host $configFile -ForegroundColor White

Write-Host "`n常见配置选项:" -ForegroundColor Green
Write-Host "[mysqld]" -ForegroundColor White
Write-Host "port = 3306" -ForegroundColor White
Write-Host "max_connections = 200" -ForegroundColor White
Write-Host "innodb_buffer_pool_size = 1G" -ForegroundColor White

Write-Host "`n停止MySQL服务请使用:" -ForegroundColor Green
Write-Host "taskkill /f /im mysqld.exe" -ForegroundColor White