@echo off
title Test Local Paradigm Network
color 0C

echo.
echo ========================================
echo   LOCAL PARADIGM NETWORK TESTER  
echo ========================================
echo.

echo [INFO] Testing network connectivity...
echo.

echo [TEST] Checking if anything is listening on port 8080...
netstat -an | findstr :8080
echo.

echo [TEST] Testing local connection...
powershell -Command "try { $response = Invoke-WebRequest -Uri 'http://127.0.0.1:8080/health' -TimeoutSec 5 -UseBasicParsing; Write-Host '[SUCCESS] Local network reachable on 127.0.0.1:8080'; Write-Host 'Status:' $response.StatusCode } catch { Write-Host '[ERROR] Cannot reach 127.0.0.1:8080:' $_.Exception.Message }"
echo.

echo [TEST] Testing VM network address...
powershell -Command "try { $response = Invoke-WebRequest -Uri 'http://192.168.56.1:8080/health' -TimeoutSec 5 -UseBasicParsing; Write-Host '[SUCCESS] VM network reachable on 192.168.56.1:8080'; Write-Host 'Status:' $response.StatusCode } catch { Write-Host '[ERROR] Cannot reach 192.168.56.1:8080:' $_.Exception.Message }"
echo.

echo [SOLUTION] Use this command to connect contributor to correct network:
echo.
if exist target\debug\paradigm-contributor.exe (
    echo ./target/debug/paradigm-contributor.exe --node-address 127.0.0.1:8080 --enable-autopool
) else (
    echo paradigm-contributor --node-address 127.0.0.1:8080 --enable-autopool
)
echo.
echo Or if your network is running on VM address:
if exist target\debug\paradigm-contributor.exe (
    echo ./target/debug/paradigm-contributor.exe --node-address 192.168.56.1:8080 --enable-autopool
) else (
    echo paradigm-contributor --node-address 192.168.56.1:8080 --enable-autopool
)
echo.
pause