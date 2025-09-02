@echo off
echo 🚀 PARADIGM QUICK START

REM Build if needed
if not exist "target\release\paradigm-core.exe" (
    echo Building release binaries...
    cargo build --release
)

REM Create directories
if not exist "data" mkdir data
if not exist "data\node1" mkdir data\node1
if not exist "data\contributor1" mkdir data\contributor1
if not exist "data\contributor2" mkdir data\contributor2

echo Starting Genesis Node...
start "Genesis" cmd /c "target\release\paradigm-core.exe --config network-config-node1.toml --genesis genesis-config.toml --data-dir ./data/node1 --enable-api --api-port 8080"

echo Waiting for network...
timeout /t 8 /nobreak

echo Starting Contributors...
start "Contributor 1" cmd /c "target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --verbose"
start "Contributor 2" cmd /c "target\release\paradigm-contributor.exe --node-address 127.0.0.1:8080 --verbose"

echo Starting Web Dashboard...
cd snt-web
if not exist "node_modules" npm install
start "Dashboard" cmd /c "set PORT=3002 && npm start"
cd ..

echo ✅ Network launched! Dashboard: http://localhost:3002
pause