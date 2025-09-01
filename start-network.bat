@echo off
echo 🌐 Starting Paradigm SNT Network...
echo.

REM Create data directories
if not exist "data" mkdir data
if not exist "data\node1" mkdir data\node1
if not exist "data\node2" mkdir data\node2
if not exist "data\node3" mkdir data\node3

echo 📁 Created data directories

REM Build paradigm-core if needed
if not exist "target\release\paradigm-core.exe" (
    echo 🔨 Building paradigm-core...
    cargo build --release
    if errorlevel 1 (
        echo ❌ Build failed! Please fix compilation errors first.
        pause
        exit /b 1
    )
)

echo ✅ paradigm-core is ready

REM Start genesis node (node 1)
echo 🚀 Starting Genesis Node (Port 8080)...
start "Genesis Node" cmd /k "target\release\paradigm-core.exe --config network-config-node1.toml"

REM Wait a bit for genesis to initialize
timeout /t 5 /nobreak > nul

REM Start keeper node 2
echo 🚀 Starting Keeper Node 2 (Port 8081)...
start "Keeper Node 2" cmd /k "target\release\paradigm-core.exe --config network-config-node2.toml"

REM Wait a bit
timeout /t 3 /nobreak > nul

REM Start keeper node 3
echo 🚀 Starting Keeper Node 3 (Port 8082)...
start "Keeper Node 3" cmd /k "target\release\paradigm-core.exe --config network-config-node3.toml"

echo.
echo 🎉 Network is starting up!
echo.
echo 🌐 Network Status:
echo   Genesis Node: http://localhost:8080
echo   Keeper Node 2: http://localhost:8081
echo   Keeper Node 3: http://localhost:8082
echo.
echo ✨ All nodes are running with SNT system enabled!
echo    - Keeper Identity SNTs will be auto-minted
echo    - Storage contribution rewards are active
echo    - Network visualization available via API
echo.
echo Press any key to open the web dashboard...
pause > nul

REM Start the web frontend
if exist "snt-web" (
    echo 🌐 Starting Web Frontend...
    cd snt-web
    start "SNT Web Dashboard" cmd /k "npm start"
    cd ..
    echo 🎯 Web Dashboard: http://localhost:3000
)

echo.
echo 🔮 Paradigm SNT Network is now running!
echo Press any key to exit...
pause > nul