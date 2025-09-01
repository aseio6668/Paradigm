@echo off
echo 🔮 Paradigm SNT Network Demo
echo =============================
echo.

echo This demo showcases:
echo   🛡️  Keeper Identity SNTs - Automatically minted for network participants
echo   📦 Storage Contribution SNTs - Earned through data hosting
echo   📜 Memory Anchor SNTs - Created through fusion rituals
echo   ⚗️  Fusion Master SNTs - Unlocked via advanced rituals
echo   🌐 Living Network Visualization - Real-time network graph
echo.

echo Prerequisites:
echo   ✅ paradigm-core compiled successfully
echo   ✅ snt-demo working (CLI showcase)
echo   ✅ snt-web ready (Web dashboard)
echo.

echo 🚀 Starting Network Demo...
echo.

REM Check if paradigm-core is built
if not exist "target\release\paradigm-core.exe" (
    echo ❌ paradigm-core not found! Please build first:
    echo    cargo build --release
    echo.
    goto :end
)

REM Start the network
echo 1️⃣  Starting 3-node localhost network...
call start-network.bat
echo.

echo 2️⃣  Waiting for network to initialize...
timeout /t 10 /nobreak > nul

echo 3️⃣  Testing network connectivity...
curl -s http://localhost:8080/health > nul
if errorlevel 1 (
    echo ❌ Genesis node not responding
    goto :end
)

curl -s http://localhost:8081/health > nul
if errorlevel 1 (
    echo ⚠️  Node 2 not responding (may still be starting...)
) else (
    echo ✅ Node 2 online
)

curl -s http://localhost:8082/health > nul
if errorlevel 1 (
    echo ⚠️  Node 3 not responding (may still be starting...)
) else (
    echo ✅ Node 3 online  
)

echo.
echo 4️⃣  Network is ready! Available endpoints:
echo    Genesis Node API: http://localhost:8080
echo    Keeper Node 2 API: http://localhost:8081  
echo    Keeper Node 3 API: http://localhost:8082
echo.

echo 5️⃣  Available Demo Commands:
echo.
echo   📊 Network Stats:
echo      curl http://localhost:8080/api/network/stats
echo.
echo   🎯 SNT Overview:
echo      curl http://localhost:8080/api/snt/overview
echo.
echo   🛡️  List Keepers:
echo      curl http://localhost:8080/api/keepers/list
echo.
echo   📦 Storage Operations:
echo      curl -X POST http://localhost:8080/api/storage/store -d "{\"data\":\"Hello SNT World!\",\"filename\":\"demo.txt\"}"
echo.
echo   ⚗️  Fusion Rituals:
echo      curl -X POST http://localhost:8080/api/fusion/start -d "{\"sigils\":[\"sigil1\",\"sigil2\"],\"mode\":\"synthesis\"}"
echo.

echo 6️⃣  CLI Demo (Standalone SNT showcase):
echo      cd snt-demo
echo      cargo run showcase
echo.

echo 7️⃣  Web Dashboard (3D Network Visualization):
echo      Open: http://localhost:3000
echo.

echo.
echo 🔮 Paradigm SNT Network Demo is ready!
echo    This showcases living functional tokens that unlock capabilities
echo    instead of being dead commodities like traditional NFTs.
echo.
echo 🎯 Key Innovations Demonstrated:
echo    • SNTs evolve through network participation  
echo    • Tokens unlock access and permissions
echo    • Community roles and ritual participation
echo    • Gamified progression: Apprentice → Guardian → Archivist → Loremaster
echo    • Element-based classification: 🔥💧🌍💨⚡🌙🔮
echo.

:end
echo Press any key to continue...
pause > nul