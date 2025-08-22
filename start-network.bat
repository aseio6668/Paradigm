@echo off
title Paradigm Network Bootstrap Node
echo 🌟 Starting Paradigm Network Bootstrap Node
echo ==========================================
echo.
echo This will start a Paradigm network that others can join.
echo.

REM Check if binaries exist
if not exist "target\release\paradigm-core.exe" (
    echo ❌ Paradigm binaries not found!
    echo Please run build.bat or build-advanced.bat first.
    echo.
    pause
    exit /b 1
)

echo ✅ Paradigm binaries found
echo.

REM Get local IP address for display
for /f "tokens=2 delims=:" %%a in ('ipconfig ^| findstr /c:"IPv4 Address"') do set LOCAL_IP=%%a
set LOCAL_IP=%LOCAL_IP:~1%
if not defined LOCAL_IP set LOCAL_IP=127.0.0.1

echo 📡 Network Configuration:
echo    Port: 8080
echo    Local IP: %LOCAL_IP%
echo    Data Directory: ./network-data
echo.

echo ⚠️  IMPORTANT: For internet access, make sure:
echo    1. Port 8080 is forwarded in your router
echo    2. Windows Firewall allows paradigm-core.exe
echo.

echo 🚀 Starting bootstrap node...
echo    Your network address will be displayed in the logs below.
echo    Look for: "local_peer_id=12D3KooW..."
echo.

echo ✋ Press Ctrl+C to stop the network
echo.

REM Start the bootstrap node
cd target\release
paradigm-core.exe --port 8080 --data-dir ./network-data

echo.
echo 🔻 Bootstrap node stopped
pause