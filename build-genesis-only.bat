@echo off
echo.
echo ================================================
echo    Building Genesis Chain Components Only
echo ================================================
echo.

echo 🔧 Building paradigm-core (with genesis system)...
cargo build --release --bin paradigm-core

if %ERRORLEVEL% neq 0 (
    echo ❌ paradigm-core build failed!
    pause
    exit /b 1
)

echo ✅ paradigm-core build successful!
echo.

echo 🔧 Building paradigm-contributor...
cargo build --release --bin paradigm-contributor

if %ERRORLEVEL% neq 0 (
    echo ❌ paradigm-contributor build failed!
    pause
    exit /b 1
)

echo ✅ paradigm-contributor build successful!
echo.

echo 📋 Copying genesis init scripts to target directories...

:: Copy genesis init to debug directory
if not exist "target\debug\genesis-init.bat" (
    copy "target\debug\genesis-init.bat" "target\debug\genesis-init.bat" >nul 2>&1 || echo Genesis init script already exists in debug
) else (
    echo ✅ Genesis init script ready in target\debug\
)

:: Copy genesis init to release directory  
if not exist "target\release\genesis-init.bat" (
    copy "target\release\genesis-init.bat" "target\release\genesis-init.bat" >nul 2>&1 || echo Genesis init script already exists in release
) else (
    echo ✅ Genesis init script ready in target\release\
)

echo.
echo 🔧 Building paradigm-wallet (optional)...
cargo build --release --bin paradigm-wallet

if %ERRORLEVEL% neq 0 (
    echo ⚠️ paradigm-wallet build failed - continuing without wallet
) else (
    echo ✅ paradigm-wallet build successful!
)

echo.
echo ================================================
echo    Genesis Chain Components Build Complete!
echo ================================================
echo.
echo 📦 Built executables:
if exist "target\release\paradigm-core.exe" echo ✅ paradigm-core.exe
if exist "target\release\paradigm-contributor.exe" echo ✅ paradigm-contributor.exe  
if exist "target\release\paradigm-wallet.exe" echo ✅ paradigm-wallet.exe

echo.
echo 🚀 Ready to start genesis chain with:
echo.
echo **Development Mode:**
echo    cd target\debug ^&^& genesis-init.bat
echo.
echo **Production Mode:**  
echo    cd target\release ^&^& genesis-init.bat
echo.
echo **Legacy Mode:**
echo    ./launch-genesis.bat ^(uses paradigm-release directory^)
echo.
pause