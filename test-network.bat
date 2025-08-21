@echo off
REM Paradigm Network Test Launcher for Windows
REM This script demonstrates multiple clients connecting and synchronizing

echo 🚀 Starting Paradigm Network Test
echo ==================================

REM Create test data directory
if not exist "test-data" mkdir test-data

REM Function equivalent - start first node
echo 🔷 Starting Main Node (Port 8080)...
start "Paradigm-Node-Main" cmd /k "cd /d %~dp0 && cargo run -p paradigm-core -- --port 8080"

REM Wait a moment for node to initialize
timeout /t 5 /nobreak >nul

REM Start multiple contributors
echo 🔶 Starting Contributor 1 (Fast Worker - 8 threads)...
start "Paradigm-Contributor-1" cmd /k "cd /d %~dp0 && cargo run -p paradigm-contributor -- --node-address 127.0.0.1:8080 --threads 8 --verbose"

timeout /t 3 /nobreak >nul

echo 🔶 Starting Contributor 2 (Balanced Worker - 4 threads)...
start "Paradigm-Contributor-2" cmd /k "cd /d %~dp0 && cargo run -p paradigm-contributor -- --node-address 127.0.0.1:8080 --threads 4 --verbose"

timeout /t 3 /nobreak >nul

echo 🔶 Starting Contributor 3 (Efficient Worker - 2 threads)...
start "Paradigm-Contributor-3" cmd /k "cd /d %~dp0 && cargo run -p paradigm-contributor -- --node-address 127.0.0.1:8080 --threads 2 --verbose"

echo.
echo ✅ Network launched successfully!
echo.
echo 📊 You should now see multiple windows:
echo    - 1 Node window (Port 8080)
echo    - 3 Contributor windows (Different thread counts)
echo.
echo 🔍 Watch the contributor windows to see:
echo    - Task processing and PAR rewards
echo    - Performance metrics
echo    - Network synchronization
echo.
echo 🛑 To stop: Close all windows or press Ctrl+C in each window
echo.
pause
