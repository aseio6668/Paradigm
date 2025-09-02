@echo off
echo.
echo ╔══════════════════════════════════════════════════════════════╗
echo ║                                                              ║
echo ║  🔮 PARADIGM NETWORK LAUNCHER 🔮                           ║
echo ║                                                              ║
echo ║  Launching complete SNT ecosystem:                           ║
echo ║  • Multi-node blockchain network                             ║
echo ║  • Two contributor clients                                   ║
echo ║  • Web dashboard visualization                               ║
echo ║  • CLI demo interface                                        ║
echo ║                                                              ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

REM Check if binaries exist
if not exist "target\release\paradigm-core.exe" (
    echo ❌ paradigm-core.exe not found. Building release binaries...
    cargo build --release
    if errorlevel 1 (
        echo ❌ Build failed. Please fix compilation errors.
        pause
        exit /b 1
    )
)

if not exist "target\release\paradigm-contributor.exe" (
    echo ❌ paradigm-contributor.exe not found. Please build it first.
    pause
    exit /b 1
)

REM Create data directories
echo 📁 Creating data directories...
if not exist "data" mkdir data
if not exist "data\node1" mkdir data\node1
if not exist "data\node2" mkdir data\node2
if not exist "data\node3" mkdir data\node3
if not exist "data\contributor1" mkdir data\contributor1
if not exist "data\contributor2" mkdir data\contributor2

REM Start Genesis Node
echo.
echo 🚀 Starting Genesis Node (Port 8080)...
start "Paradigm Genesis Node" cmd /c "target\release\paradigm-core.exe --config network-config-node1.toml --genesis genesis-config.toml --data-dir ./data/node1 --enable-api --api-port 8080 & pause"

REM Wait for genesis node
echo ⏳ Waiting for Genesis Node to initialize...
timeout /t 10 /nobreak

REM Start Keeper Nodes
echo.
echo 🔗 Starting Keeper Node 2 (Port 8081)...
start "Paradigm Keeper Node 2" cmd /c "target\release\paradigm-core.exe --config network-config-node2.toml --genesis genesis-config.toml --data-dir ./data/node2 --enable-api --api-port 8081 & pause"

echo.
echo 🔗 Starting Keeper Node 3 (Port 8082)...
start "Paradigm Keeper Node 3" cmd /c "target\release\paradigm-core.exe --config network-config-node3.toml --genesis genesis-config.toml --data-dir ./data/node3 --enable-api --api-port 8082 & pause"

REM Wait for network to stabilize
echo ⏳ Waiting for network to stabilize...
timeout /t 15 /nobreak

REM Start Contributors
echo.
echo 👥 Starting Contributor 1...
start "Paradigm Contributor 1" cmd /c "target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --verbose & pause"

echo.
echo 👥 Starting Contributor 2...
start "Paradigm Contributor 2" cmd /c "target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --verbose & pause"

REM Check if web dependencies are installed
echo.
echo 🌐 Preparing Web Dashboard...
cd snt-web
if not exist "node_modules" (
    echo 📦 Installing web dependencies...
    npm install
    if errorlevel 1 (
        echo ❌ npm install failed
        cd ..
        pause
        exit /b 1
    )
)

REM Start Web Dashboard
echo.
echo 🖥️ Starting Web Dashboard (Port 3002)...
start "Paradigm Web Dashboard" cmd /c "set PORT=3002 && npm start & pause"

cd ..

REM Wait for everything to start
echo ⏳ Waiting for all services to start...
timeout /t 10 /nobreak

REM Health check
echo.
echo 🔍 Performing health checks...
curl -s http://localhost:8080/health >nul 2>&1
if errorlevel 1 (
    echo ⚠️ Genesis node health check failed
) else (
    echo ✅ Genesis node is healthy
)

curl -s http://localhost:8081/health >nul 2>&1
if errorlevel 1 (
    echo ⚠️ Keeper node 2 health check failed
) else (
    echo ✅ Keeper node 2 is healthy
)

curl -s http://localhost:8082/health >nul 2>&1
if errorlevel 1 (
    echo ⚠️ Keeper node 3 health check failed
) else (
    echo ✅ Keeper node 3 is healthy
)

REM Show CLI Demo option
echo.
echo 🎮 Want to see the CLI demo? Run:
echo    target\release\snt-demo.exe showcase
echo.

echo 🌟 PARADIGM NETWORK LAUNCHED SUCCESSFULLY!
echo.
echo 📊 Access Points:
echo    Web Dashboard: http://localhost:3002
echo    Genesis Node API: http://localhost:8080/api
echo    Keeper Node 2 API: http://localhost:8081/api  
echo    Keeper Node 3 API: http://localhost:8082/api
echo.
echo 🔧 Management:
echo    • All services run in separate windows
echo    • Close windows to stop individual services
echo    • Contributors will auto-connect to nodes
echo    • Web dashboard shows live network status
echo.

pause