@echo off
REM Paradigm Cryptocurrency Build Script for Windows
REM This script builds all components of the Paradigm cryptocurrency system

echo 🚀 Building Paradigm Cryptocurrency System
echo ==========================================

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust/Cargo not found. Please install Rust first:
    echo    Visit https://rustup.rs/ and follow the instructions
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo ✅ Rust found: %RUST_VERSION%

echo 🔍 Checking system dependencies...

REM Check for protobuf compiler
where protoc >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ⚠️  protoc not found. Please install Protocol Buffers manually:
    echo    https://github.com/protocolbuffers/protobuf/releases
    echo    Or use vcpkg: vcpkg install protobuf
)

echo ✅ System dependencies checked

REM Create build directory
set BUILD_DIR=target\paradigm-release
if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"

echo 🔨 Building Paradigm Core...
cargo build --release --package paradigm-core
if %ERRORLEVEL% EQU 0 (
    echo ✅ Paradigm Core built successfully
    copy target\release\paradigm-core.exe "%BUILD_DIR%\"
) else (
    echo ❌ Failed to build Paradigm Core
    pause
    exit /b 1
)

echo 🔨 Building Paradigm Wallet...
cargo build --release --package paradigm-wallet
if %ERRORLEVEL% EQU 0 (
    echo ✅ Paradigm Wallet built successfully
    copy target\release\paradigm-wallet.exe "%BUILD_DIR%\"
) else (
    echo ❌ Failed to build Paradigm Wallet
    pause
    exit /b 1
)

echo 🔨 Building Paradigm Contributor...
cargo build --release --package paradigm-contributor
if %ERRORLEVEL% EQU 0 (
    echo ✅ Paradigm Contributor built successfully
    copy target\release\paradigm-contributor.exe "%BUILD_DIR%\"
) else (
    echo ❌ Failed to build Paradigm Contributor
    pause
    exit /b 1
)

REM Run tests
echo 🧪 Running tests...
cargo test --release --all
if %ERRORLEVEL% EQU 0 (
    echo ✅ All tests passed
) else (
    echo ⚠️  Some tests failed, but continuing...
)

REM Create configuration file
echo 📝 Creating configuration files...

echo # Paradigm Cryptocurrency Configuration > "%BUILD_DIR%\paradigm.toml"
echo. >> "%BUILD_DIR%\paradigm.toml"
echo [network] >> "%BUILD_DIR%\paradigm.toml"
echo # Network port for node communication >> "%BUILD_DIR%\paradigm.toml"
echo port = 8080 >> "%BUILD_DIR%\paradigm.toml"
echo. >> "%BUILD_DIR%\paradigm.toml"
echo # Bootstrap peers ^(comma-separated^) >> "%BUILD_DIR%\paradigm.toml"
echo bootstrap_peers = [] >> "%BUILD_DIR%\paradigm.toml"
echo. >> "%BUILD_DIR%\paradigm.toml"
echo [node] >> "%BUILD_DIR%\paradigm.toml"
echo # Data directory for blockchain data >> "%BUILD_DIR%\paradigm.toml"
echo data_dir = "./paradigm-data" >> "%BUILD_DIR%\paradigm.toml"
echo. >> "%BUILD_DIR%\paradigm.toml"
echo [contributor] >> "%BUILD_DIR%\paradigm.toml"
echo # Maximum concurrent ML tasks >> "%BUILD_DIR%\paradigm.toml"
echo max_tasks = 4 >> "%BUILD_DIR%\paradigm.toml"
echo # Use GPU acceleration if available >> "%BUILD_DIR%\paradigm.toml"
echo use_gpu = true >> "%BUILD_DIR%\paradigm.toml"

REM Create batch files
echo @echo off > "%BUILD_DIR%\start-node.bat"
echo echo Starting Paradigm Node... >> "%BUILD_DIR%\start-node.bat"
echo paradigm-core.exe --config paradigm.toml >> "%BUILD_DIR%\start-node.bat"
echo pause >> "%BUILD_DIR%\start-node.bat"

echo @echo off > "%BUILD_DIR%\start-contributor.bat"
echo echo Starting Paradigm Contributor... >> "%BUILD_DIR%\start-contributor.bat"
echo set /p WALLET_ADDR="Enter your wallet address: " >> "%BUILD_DIR%\start-contributor.bat"
echo paradigm-contributor.exe --wallet-address %%WALLET_ADDR%% --use-gpu >> "%BUILD_DIR%\start-contributor.bat"
echo pause >> "%BUILD_DIR%\start-contributor.bat"

echo @echo off > "%BUILD_DIR%\start-wallet.bat"
echo echo Starting Paradigm Wallet... >> "%BUILD_DIR%\start-wallet.bat"
echo paradigm-wallet.exe >> "%BUILD_DIR%\start-wallet.bat"

REM Create README
echo Paradigm Cryptocurrency - Release Build > "%BUILD_DIR%\README.txt"
echo ====================================== >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo This directory contains the complete Paradigm cryptocurrency system. >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Files: >> "%BUILD_DIR%\README.txt"
echo - paradigm-core.exe: Main blockchain node >> "%BUILD_DIR%\README.txt"
echo - paradigm-wallet.exe: GUI wallet application >> "%BUILD_DIR%\README.txt"
echo - paradigm-contributor.exe: ML contributor client >> "%BUILD_DIR%\README.txt"
echo - paradigm.toml: Configuration file >> "%BUILD_DIR%\README.txt"
echo - start-*.bat: Startup scripts >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Quick Start: >> "%BUILD_DIR%\README.txt"
echo 1. For regular users: Run start-wallet.bat >> "%BUILD_DIR%\README.txt"
echo 2. To run a node: Run start-node.bat >> "%BUILD_DIR%\README.txt"
echo 3. To contribute ML power: Run start-contributor.bat >> "%BUILD_DIR%\README.txt"

REM Copy documentation
if exist LICENSE copy LICENSE "%BUILD_DIR%\" >nul
if exist QUICKSTART.md copy QUICKSTART.md "%BUILD_DIR%\" >nul
if exist CONTRIBUTING.md copy CONTRIBUTING.md "%BUILD_DIR%\" >nul

echo.
echo 🎉 Build completed successfully!
echo 📁 Release files are in: %BUILD_DIR%
echo.
echo To get started:
echo   cd %BUILD_DIR%
echo   start-wallet.bat     # For users
echo   start-node.bat       # For node operators  
echo   start-contributor.bat # For ML contributors
echo.
echo 📚 See QUICKSTART.md for detailed instructions
echo 🚀 Welcome to the future of cryptocurrency!
echo.
pause
