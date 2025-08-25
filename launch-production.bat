@echo off
title Paradigm Production Network Launcher
color 0A

echo.
echo =========================================
echo   PARADIGM PRODUCTION NETWORK LAUNCHER
echo =========================================
echo.

echo [INFO] Preparing production network configuration...

:: Check if production configuration exists
if not exist "production-config.toml" (
    echo [SETUP] Creating production configuration...
    copy "network-config-template.toml" "production-config.toml" >nul
    
    :: Modify for production settings
    powershell -Command "(gc production-config.toml) -replace 'bind_address = \"127.0.0.1:8080\"', 'bind_address = \"0.0.0.0:8080\"' | Out-File -encoding ASCII production-config.toml"
    powershell -Command "(gc production-config.toml) -replace 'enable_debug_mode = true', 'enable_debug_mode = false' | Out-File -encoding ASCII production-config.toml"
    powershell -Command "(gc production-config.toml) -replace 'max_peers = 10', 'max_peers = 100' | Out-File -encoding ASCII production-config.toml"
    
    echo [SUCCESS] Production configuration created
) else (
    echo [INFO] Using existing production configuration
)

:: Check if genesis configuration exists
if not exist "genesis-production.toml" (
    echo [SETUP] Creating production genesis configuration...
    copy "genesis-config.toml" "genesis-production.toml" >nul
    
    :: Set production parameters
    powershell -Command "(gc genesis-production.toml) -replace 'initial_supply = 2100000000000000', 'initial_supply = 2100000000000000' | Out-File -encoding ASCII genesis-production.toml"
    powershell -Command "(gc genesis-production.toml) -replace 'treasury_reserve = 1890000000000000', 'treasury_reserve = 1890000000000000' | Out-File -encoding ASCII genesis-production.toml"
    
    echo [SUCCESS] Production genesis configuration created
) else (
    echo [INFO] Using existing production genesis configuration
)

:: Create production data directory
if not exist "production-data" mkdir production-data

echo.
echo [NETWORK] Production Network Settings:
echo   - Bind Address: 0.0.0.0:8080 (accepts external connections)
echo   - Max Peers: 100 (supports larger network)
echo   - Debug Mode: Disabled (production logging)
echo   - Data Directory: ./production-data/
echo.

echo [WARNING] This will launch a PRODUCTION network that others can connect to
echo [WARNING] Make sure your firewall allows connections on port 8080
echo.

set /p confirm="Continue with production launch? (Y/N): "
if /i not "%confirm%"=="Y" (
    echo [CANCELLED] Production launch cancelled by user
    pause
    exit /b 1
)

echo.
echo [LAUNCH] Starting Paradigm Production Network...
echo [INFO] Network will be accessible at: %COMPUTERNAME%:8080
echo [INFO] External IP connections will be accepted
echo.

:: Test if executable exists and launch with production settings
if exist "target/release/paradigm-core.exe" (
    echo [LAUNCH] Using target/release/paradigm-core.exe
    ./target/release/paradigm-core.exe ^
        --config production-config.toml ^
        --genesis genesis-production.toml ^
        --data-dir ./production-data ^
        --enable-api ^
        --api-port 8080 ^
        --log-level info ^
        --enable-metrics ^
        --production-mode
) else if exist "target\release\paradigm-core.exe" (
    echo [LAUNCH] Using target\release\paradigm-core.exe
    ./target/release/paradigm-core.exe ^
        --config production-config.toml ^
        --genesis genesis-production.toml ^
        --data-dir ./production-data ^
        --enable-api ^
        --api-port 8080 ^
        --log-level info ^
        --enable-metrics ^
        --production-mode
) else (
    echo [ERROR] paradigm-core.exe not found in target/release/
    echo [INFO] Please run: cargo build --release
    pause
    exit /b 1
)

echo.
echo [SHUTDOWN] Production network stopped
pause