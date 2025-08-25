@echo off
echo.
echo ================================================
echo    Paradigm Genesis Chain Launcher
echo ================================================
echo.
echo This will launch the genesis chain from the release directory.
echo.

:: Check if release directory exists
if not exist "target\paradigm-release" (
    echo ❌ Error: target\paradigm-release directory not found
    echo Please run build-genesis-only.bat first
    pause
    exit /b 1
)

:: Navigate to release directory and launch
echo 🚀 Launching from target\paradigm-release directory...
echo.

cd target\paradigm-release
call genesis-chain.bat

echo.
echo 🔄 Returning to source directory...
cd ..\..
pause