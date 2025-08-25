@echo off
title Paradigm Contributor - Network Connection Tool
color 0B

echo.
echo ==========================================
echo   PARADIGM CONTRIBUTOR CONNECTION TOOL
echo ==========================================
echo.

if "%1"=="" (
    echo [USAGE] %0 ^<network-address^> [options]
    echo.
    echo Examples:
    echo   %0 localhost:8080                     ^(connect to local network^)
    echo   %0 192.168.1.100:8080                ^(connect to LAN network^)
    echo   %0 paradigm-network.example.com:8080 ^(connect to public network^)
    echo.
    echo Optional flags:
    echo   --enable-autopool     Enable work aggregation for small contributions
    echo   --wallet-file PATH    Use specific wallet file
    echo   --threads N           Number of worker threads ^(default: auto^)
    echo   --verbose             Enable detailed logging
    echo.
    pause
    exit /b 1
)

set NETWORK_ADDRESS=%1
shift

echo [INFO] Connecting to Paradigm Network: %NETWORK_ADDRESS%
echo [INFO] Preparing contributor client...

:: Test network connectivity first
echo [TEST] Testing network connectivity...
powershell -Command "try { $response = Invoke-WebRequest -Uri 'http://%NETWORK_ADDRESS%/health' -TimeoutSec 10 -UseBasicParsing; Write-Host '[SUCCESS] Network is reachable - Status:' $response.StatusCode } catch { Write-Host '[ERROR] Cannot reach network:' $_.Exception.Message; exit 1 }" 2>nul

if errorlevel 1 (
    echo [ERROR] Failed to connect to network: %NETWORK_ADDRESS%
    echo [HELP] Make sure:
    echo   1. The network address is correct
    echo   2. The Paradigm core is running on that address
    echo   3. Firewall allows connections on the specified port
    echo   4. You have internet/network connectivity
    echo.
    pause
    exit /b 1
)

echo [SUCCESS] Network connectivity verified
echo.

:: Collect additional arguments
set EXTRA_ARGS=
:parse_args
if "%1"=="" goto start_contributor
set EXTRA_ARGS=%EXTRA_ARGS% %1
shift
goto parse_args

:start_contributor
echo [LAUNCH] Starting Paradigm Contributor...
echo [TARGET] Network: %NETWORK_ADDRESS%
echo [CONFIG] Additional options: %EXTRA_ARGS%
echo.
echo [STATUS] Contributor will automatically:
echo   - Connect to the Paradigm network
echo   - Process available ML tasks
echo   - Receive PAR token rewards to your wallet
echo   - Handle network disconnections gracefully
echo.

:: Launch contributor with specified network
./target/debug/paradigm-contributor.exe --node-address %NETWORK_ADDRESS% %EXTRA_ARGS%

echo.
echo [DISCONNECTED] Contributor stopped
pause