@echo off
echo Resetting MySQL root password...
echo Please make sure MySQL service is not running before continuing.

set MYSQL_PATH=C:\Users\zhy\.enman\installs\mysql\8.0.27
set TEMP_SQL_FILE=%TEMP%\reset_root_password.sql

echo [mysql] > %TEMP%\mysql_no_pass.cnf
echo user=root >> %TEMP%\mysql_no_pass.cnf
echo host=localhost >> %TEMP%\mysql_no_pass.cnf
echo [client] >> %TEMP%\mysql_no_pass.cnf
echo user=root >> %TEMP%\mysql_no_pass.cnf
echo host=localhost >> %TEMP%\mysql_no_pass.cnf

echo ALTER USER 'root'@'localhost' IDENTIFIED BY 'root123'; > "%TEMP_SQL_FILE%"
echo FLUSH PRIVILEGES; >> "%TEMP_SQL_FILE%"

echo Starting MySQL with skip-grant-tables...
start /wait "" "%MYSQL_PATH%\bin\mysqld.exe" --defaults-file="%MYSQL_PATH%\my.ini" --skip-grant-tables --bootstrap

ping localhost -n 5 > nul

echo Executing password reset...
"%MYSQL_PATH%\bin\mysql.exe" -h localhost -P 3306 --defaults-file=%TEMP%\mysql_no_pass.cnf < "%TEMP_SQL_FILE%"

del "%TEMP_SQL_FILE%"
del %TEMP%\mysql_no_pass.cnf

echo.
echo Password reset completed. You can now connect with: mysql -u root -p
echo Your new password is 'root123'
pause