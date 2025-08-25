@echo off
echo.
echo ================================================
echo    Paradigm Genesis Chain Launcher
echo ================================================
echo.

echo Please choose your genesis mode:
echo.
echo 1. Development Mode  (target/debug - fast builds, verbose logging)
echo 2. Production Mode   (target/release - optimized, production-ready)
echo 3. Legacy Mode       (paradigm-release - pre-packaged release)
echo.

set /p choice="Enter your choice (1-3): "

if "%choice%"=="1" goto :dev_mode
if "%choice%"=="2" goto :prod_mode  
if "%choice%"=="3" goto :legacy_mode

echo Invalid choice. Exiting.
pause
exit /b 1

:dev_mode
echo.
echo 🔧 **Development Mode Selected**
echo.

if not exist "target\debug\paradigm-core.exe" (
    echo ❌ Error: Debug executables not found
    echo Please run: cargo build --bin paradigm-core --bin paradigm-contributor
    echo.
    set /p build_now="Build now? (y/n): "
    if /i "!build_now!"=="y" (
        echo Building debug executables...
        cargo build --bin paradigm-core --bin paradigm-contributor
        if !ERRORLEVEL! neq 0 (
            echo Build failed!
            pause
            exit /b 1
        )
    ) else (
        echo Please build first, then run this script again.
        pause
        exit /b 1
    )
)

echo Launching development genesis chain...
cd target\debug
call genesis-init.bat
goto :end

:prod_mode
echo.
echo 🚀 **Production Mode Selected**
echo.

if not exist "target\release\paradigm-core.exe" (
    echo ❌ Error: Release executables not found
    echo Please run: cargo build --release --bin paradigm-core --bin paradigm-contributor
    echo.
    set /p build_now="Build now? (y/n): "
    if /i "!build_now!"=="y" (
        echo Building release executables...
        cargo build --release --bin paradigm-core --bin paradigm-contributor
        if !ERRORLEVEL! neq 0 (
            echo Build failed!
            pause
            exit /b 1
        )
    ) else (
        echo Please build first, then run this script again.
        pause
        exit /b 1
    )
)

echo Launching production genesis chain...
cd target\release
call genesis-init.bat
goto :end

:legacy_mode
echo.
echo 📦 **Legacy Mode Selected**
echo.

if not exist "target\paradigm-release\genesis-chain.bat" (
    echo ❌ Error: paradigm-release directory not found
    echo Please run build-genesis-only.bat first
    pause
    exit /b 1
)

echo Launching from paradigm-release directory...
cd target\paradigm-release
call genesis-chain.bat
goto :end

:end
echo.
echo Returning to source directory...
cd ..