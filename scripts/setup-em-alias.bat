@echo off
echo Setting up enman alias 'em' in Windows CMD...
echo.

REM 检查是否以管理员权限运行
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo This script requires administrator privileges.
    echo Please run as administrator.
    pause
    exit /b 1
)

echo Creating CMD alias for 'em' pointing to 'enman'...
doskey /macros > macros_backup.txt
echo Backed up existing macros to macros_backup.txt

REM 创建新的宏
doskey em=enman $*

echo.
echo Alias 'em' has been created for 'enman'.
echo You can now use 'em' instead of 'enman' in this CMD session.
echo.
echo Note: This alias is temporary and will only last for this CMD session.
echo To make it permanent, add the following line to your AutoRun registry key:
echo   doskey em=enman $*
echo.
echo To do this automatically, run regedit and navigate to:
echo   HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Command Processor\
echo Then add/modify the 'AutoRun' REG_SZ value with the doskey command above.
echo.

pause