@echo off
REM Paradigm Cryptocurrency Optimized Build Script
REM Fast and efficient build without filesystem scanning issues

echo 🚀 Paradigm Build (Optimized)
echo ===============================
echo.

REM Check Rust
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust not found. Install from: https://rustup.rs/
    pause
    exit /b 1
)

for /f "tokens=2" %%a in ('rustc --version') do set RUST_VER=%%a
echo ✅ Rust %RUST_VER% found

REM Quick protoc check
where protoc >nul 2>nul
if %ERRORLEVEL% NEQ 0 (echo ⚠️  protoc not found - some features limited) else (echo ✅ protoc found)

echo.
echo 🧹 Cleaning...
cargo clean >nul 2>nul
echo ✅ Clean completed

echo.
echo 🔨 Building Paradigm Core...
echo    💡 Features: AI Governance • Quantum Security • ML Analytics
cargo build --release --package paradigm-core --quiet
if %ERRORLEVEL% NEQ 0 (echo ❌ Core build failed && pause && exit /b 1)
echo ✅ Core built successfully

echo.
echo 🔨 Building Wallet...
cargo build --release --package paradigm-wallet --quiet
if %ERRORLEVEL% NEQ 0 (echo ❌ Wallet build failed && pause && exit /b 1)  
echo ✅ Wallet built successfully

echo.
echo 🔨 Building Contributor...
cargo build --release --package paradigm-contributor --quiet
if %ERRORLEVEL% NEQ 0 (echo ❌ Contributor build failed && pause && exit /b 1)
echo ✅ Contributor built successfully

echo.
echo 🎉 Build completed successfully!
echo ===============================
echo.
echo 📁 Binaries: target\release\
echo ⚡ Build: Fast (no filesystem scanning)
echo.
echo 🚀 Quick Start:
echo   target\release\paradigm-core.exe     ^(blockchain node^)
echo   target\release\paradigm-wallet.exe   ^(wallet GUI^)  
echo   target\release\paradigm-contributor.exe ^(earn PAR^)
echo.
echo 💡 For advanced features: build-advanced.bat
echo.
pause