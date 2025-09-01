@echo off
echo 🛑 Stopping Paradigm SNT Network...

REM Kill all paradigm-core processes
taskkill /f /im "paradigm-core.exe" 2>nul
if %errorlevel% == 0 (
    echo ✅ Stopped all paradigm-core nodes
) else (
    echo ℹ️  No paradigm-core processes found
)

REM Kill node processes if web frontend is running
taskkill /f /im "node.exe" 2>nul
if %errorlevel% == 0 (
    echo ✅ Stopped web frontend
)

echo.
echo 🔮 Network stopped successfully!
pause