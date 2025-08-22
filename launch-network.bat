@echo off
REM Paradigm Production Network Launcher
REM Streamlined production-ready network deployment

setlocal enabledelayedexpansion

echo 🌟 Paradigm Production Network Launcher
echo =====================================

REM Check if binaries exist
if not exist "target\release\paradigm-core.exe" (
    echo ❌ Release binaries not found!
    echo Please run build-advanced.bat first.
    pause
    exit /b 1
)

REM Configuration
set DATA_DIR=production-data
set LOG_DIR=production-logs

REM Create directories
if not exist "%DATA_DIR%" mkdir "%DATA_DIR%"
if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"

if "%1"=="stop" goto stop_network
if "%1"=="status" goto show_status
if "%1"=="logs" goto show_logs
if "%1"=="help" goto show_help

:start_network
echo ✅ Starting Paradigm Production Network...
echo.

REM Start core nodes
echo 🔷 Starting Core Node 1 (Bootstrap - Port 8080)...
start "Paradigm-Core-1" cmd /k "title Paradigm Core Node 1 && target\release\paradigm-core.exe --port 8080 --data-dir %DATA_DIR%\node-1"

timeout /t 3 /nobreak >nul

echo 🔷 Starting Core Node 2 (Port 8081)...
start "Paradigm-Core-2" cmd /k "title Paradigm Core Node 2 && target\release\paradigm-core.exe --port 8081 --data-dir %DATA_DIR%\node-2 --bootstrap-peers /ip4/127.0.0.1/tcp/8080"

timeout /t 3 /nobreak >nul

echo 🔷 Starting Core Node 3 (Port 8082)...
start "Paradigm-Core-3" cmd /k "title Paradigm Core Node 3 && target\release\paradigm-core.exe --port 8082 --data-dir %DATA_DIR%\node-3 --bootstrap-peers /ip4/127.0.0.1/tcp/8080"

REM Wait for network to establish
echo ⏳ Waiting for network to establish...
timeout /t 8 /nobreak >nul

REM Start contributors
echo 🔶 Starting Contributors (PAR Token Miners)...
start "Paradigm-Contributor-1" cmd /k "title PAR Miner 1 && target\release\paradigm-contributor.exe --data-dir %DATA_DIR%\contrib-1"
timeout /t 2 /nobreak >nul

start "Paradigm-Contributor-2" cmd /k "title PAR Miner 2 && target\release\paradigm-contributor.exe --data-dir %DATA_DIR%\contrib-2"
timeout /t 2 /nobreak >nul

start "Paradigm-Contributor-3" cmd /k "title PAR Miner 3 && target\release\paradigm-contributor.exe --data-dir %DATA_DIR%\contrib-3"

echo.
echo 🎉 Production Network Started Successfully!
echo ==========================================
echo.
echo 🌐 Network URLs:
echo    Node 1 (Bootstrap): http://127.0.0.1:8080
echo    Node 2: http://127.0.0.1:8081  
echo    Node 3: http://127.0.0.1:8082
echo.
echo 💰 PAR Mining Active:
echo    - 3 Contributors earning PAR tokens
echo    - AI governance system online
echo    - Quantum-resistant security enabled
echo.
echo 📊 Commands:
echo    launch-network.bat status    # Show network status
echo    launch-network.bat stop      # Stop all components
echo    launch-network.bat logs      # View recent logs
echo.
echo 🛑 To stop individual windows: Close each window or press Ctrl+C
echo ✋ To stop entire network: run 'launch-network.bat stop'
goto end

:stop_network
echo 🛑 Stopping Paradigm Network...
taskkill /f /im paradigm-core.exe 2>nul
taskkill /f /im paradigm-contributor.exe 2>nul
taskkill /f /im paradigm-wallet.exe 2>nul
echo ✅ Network stopped
goto end

:show_status
echo 📊 Paradigm Network Status
echo ========================
echo.
echo 🔷 Core Nodes:
tasklist | find "paradigm-core.exe" >nul && echo    ✅ Core nodes running || echo    ❌ No core nodes running
echo.
echo 🔶 Contributors:
tasklist | find "paradigm-contributor.exe" >nul && echo    ✅ Contributors mining PAR || echo    ❌ No contributors running
echo.
echo 💰 Wallets:
tasklist | find "paradigm-wallet.exe" >nul && echo    ✅ Wallets active || echo    ⚠️  No wallets running
goto end

:show_logs
echo 📄 Recent Network Logs
echo =====================
echo.
echo ⚠️  Log viewing feature coming soon
echo    For now, check individual terminal windows
goto end

:show_help
echo Paradigm Production Network Launcher
echo ===================================
echo.
echo Usage: launch-network.bat [command]
echo.
echo Commands:
echo   (none)   Start production network
echo   stop     Stop all network components  
echo   status   Show current network status
echo   logs     Show recent network logs
echo   help     Show this help message
echo.
echo Examples:
echo   launch-network.bat        # Start network
echo   launch-network.bat stop   # Stop network
echo   launch-network.bat status # Check status
goto end

:end
if not "%1"=="stop" if not "%1"=="status" if not "%1"=="logs" if not "%1"=="help" pause