@echo off
echo 🛑 STOPPING PARADIGM NETWORK

echo Stopping Paradigm processes...
taskkill /F /IM paradigm-core.exe 2>nul
taskkill /F /IM paradigm-contributor.exe 2>nul
taskkill /F /IM node.exe 2>nul

echo Cleaning up data directories...
if exist "data" (
    rmdir /s /q data
    echo ✅ Data directories cleaned
)

echo 🔄 Network stopped and cleaned up!
pause