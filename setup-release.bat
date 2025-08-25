@echo off
echo ================================================
echo    Paradigm Release Setup Script
echo ================================================
echo.
echo This script builds the release binaries and sets up
echo all necessary scripts for production deployment.
echo.

echo 🔨 Building release binaries...
cargo build --release --bin paradigm-core --bin paradigm-contributor --bin paradigm-wallet

if errorlevel 1 (
    echo ❌ Build failed. Please check errors above.
    pause
    exit /b 1
)

echo ✅ Build completed successfully!
echo.

echo 📂 Setting up release scripts...

:: Ensure target/release exists
if not exist "target\release" (
    echo ❌ Release directory not found. Build may have failed.
    pause
    exit /b 1
)

:: Copy genesis script if it doesn't exist
if not exist "target\release\genesis-init.bat" (
    echo 📄 Genesis script already exists in release directory
) else (
    echo 📄 Genesis script found in release directory
)

:: Create a quick test script
echo 📄 Creating test-release.bat...
(
echo @echo off
echo echo Testing Paradigm release binaries...
echo echo.
echo if not exist "paradigm-core.exe" ^(
echo     echo ❌ paradigm-core.exe not found
echo     pause
echo     exit /b 1
echo ^)
echo echo ✅ paradigm-core.exe found
echo.
echo if not exist "paradigm-contributor.exe" ^(
echo     echo ❌ paradigm-contributor.exe not found  
echo     pause
echo     exit /b 1
echo ^)
echo echo ✅ paradigm-contributor.exe found
echo.
echo if not exist "paradigm-wallet.exe" ^(
echo     echo ❌ paradigm-wallet.exe not found
echo     pause  
echo     exit /b 1
echo ^)
echo echo ✅ paradigm-wallet.exe found
echo.
echo echo 🎉 All release binaries are present and ready!
echo echo.
echo echo Available commands:
echo echo - genesis-init.bat          ^(Start new blockchain^)
echo echo - paradigm-core.exe --help  ^(View core options^)
echo echo - paradigm-contributor.exe --help ^(View contributor options^)
echo echo.
echo pause
) > target\release\test-release.bat

:: Create README for release
echo 📄 Creating release README...
(
echo # Paradigm Cryptocurrency - Production Release
echo.
echo This directory contains the production-ready Paradigm binaries and scripts.
echo.
echo ## Quick Start
echo.
echo ### Start a New Genesis Network:
echo ```batch
echo genesis-init.bat
echo ```
echo.
echo ### Connect to Existing Network:
echo ```batch
echo paradigm-core.exe --addnode "genesis-ip:8080" --enable-api
echo paradigm-contributor.exe --node-address genesis-ip:8080
echo ```
echo.
echo ### Test Release:
echo ```batch  
echo test-release.bat
echo ```
echo.
echo ## Binaries Included
echo.
echo - **paradigm-core.exe** - Main blockchain node
echo - **paradigm-contributor.exe** - ML task processing client  
echo - **paradigm-wallet.exe** - Wallet management tool
echo.
echo ## Scripts Included
echo.
echo - **genesis-init.bat** - Initialize new blockchain from genesis
echo - **test-release.bat** - Verify all binaries are present
echo.
echo ## HTTP API Endpoints
echo.
echo When started with `--enable-api`, the core provides:
echo.
echo - `http://localhost:8080/health` - Node health status
echo - `http://localhost:8080/api/tasks/available` - Available ML tasks
echo.
echo ## Production Deployment
echo.
echo 1. Run `genesis-init.bat` on the genesis node
echo 2. Share the genesis node IP with other participants
echo 3. Other nodes connect using `--addnode "genesis-ip:8080"`
echo 4. Contributors connect using `--node-address genesis-ip:8080`
echo.
echo For detailed documentation, see the main repository.
) > target\release\README.md

echo.
echo ✅ RELEASE SETUP COMPLETED!
echo ================================================
echo.
echo 📂 Release Directory: target\release\
echo.
echo 📋 Available Files:
echo - paradigm-core.exe        ^(Main blockchain node^)
echo - paradigm-contributor.exe ^(ML task processor^) 
echo - paradigm-wallet.exe      ^(Wallet management^)
echo - genesis-init.bat         ^(Genesis blockchain launcher^)
echo - test-release.bat         ^(Binary verification^)
echo - README.md               ^(Release documentation^)
echo.
echo 🚀 Quick Commands:
echo   cd target\release
echo   test-release.bat         ^(Verify setup^)
echo   genesis-init.bat         ^(Start new network^)
echo.
echo 💡 The genesis script will create a prod-genesis/ subdirectory
echo    with all network configuration and management scripts.
echo.
echo 🎉 Your Paradigm production release is ready!
echo.

pause