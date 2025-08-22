@echo off
REM Paradigm Network Test Launcher - Updated for Release Binaries
REM This script demonstrates multiple clients connecting and synchronizing

echo 🚀 Starting Paradigm Network Test
echo ==================================

REM Check if binaries exist
if not exist "target\release\paradigm-core.exe" (
    echo ❌ Release binaries not found!
    echo Please run build.bat or build-advanced.bat first.
    pause
    exit /b 1
)

REM Create test data directory
if not exist "test-data" mkdir test-data

echo ✅ Release binaries found
echo 🔷 Starting Main Node (Port 8080)...
start "Paradigm-Node-Main" target\release\paradigm-core.exe --port 8080 --data-dir test-data\node-main

REM Wait for node to initialize
timeout /t 5 /nobreak >nul

REM Start multiple contributors
echo 🔶 Starting Contributor 1 (Fast Worker)...
start "Paradigm-Contributor-1" target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --data-dir test-data\contrib-1

timeout /t 3 /nobreak >nul

echo 🔶 Starting Contributor 2 (Balanced Worker)...
start "Paradigm-Contributor-2" target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --data-dir test-data\contrib-2

timeout /t 3 /nobreak >nul

echo 🔶 Starting Contributor 3 (Efficient Worker)...
start "Paradigm-Contributor-3" target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --data-dir test-data\contrib-3

echo.
echo ✅ Network test launched successfully!
echo.
echo 📊 You should now see multiple windows:
echo    - 1 Node window (Port 8080)
echo    - 3 Contributor windows (Earning PAR tokens)
echo.
echo 🔍 Watch the contributor windows to see:
echo    - ML task processing and PAR rewards
echo    - AI governance system in action
echo    - Network synchronization
echo.
echo 🛑 To stop: Close all windows or press Ctrl+C in each window
echo.
pause