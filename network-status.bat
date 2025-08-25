@echo off
title Paradigm Network Status Monitor
color 0E

if "%1"=="" (
    set NETWORK_ADDRESS=localhost:8080
) else (
    set NETWORK_ADDRESS=%1
)

echo.
echo ======================================
echo   PARADIGM NETWORK STATUS MONITOR
echo ======================================
echo Target: %NETWORK_ADDRESS%
echo.

:monitor_loop
cls
echo ======================================
echo   PARADIGM NETWORK STATUS MONITOR
echo ======================================
echo Target: %NETWORK_ADDRESS%
echo Updated: %date% %time%
echo.

:: Get network health status
powershell -Command "try { $response = Invoke-RestMethod -Uri 'http://%NETWORK_ADDRESS%/health' -TimeoutSec 5; Write-Host '[NETWORK STATUS]'; Write-Host '  Status: ONLINE'; Write-Host '  Block Height:' $response.block_height; Write-Host '  Connected Peers:' $response.peers_count; Write-Host '  Network Health:' $response.network_status; Write-Host '  Uptime:' $response.uptime_seconds 'seconds' } catch { Write-Host '[NETWORK STATUS]'; Write-Host '  Status: OFFLINE'; Write-Host '  Error:' $_.Exception.Message }" 2>nul

echo.

:: Get available tasks
powershell -Command "try { $response = Invoke-RestMethod -Uri 'http://%NETWORK_ADDRESS%/api/tasks/available' -TimeoutSec 5; Write-Host '[TASK QUEUE]'; Write-Host '  Available Tasks:' $response.available_tasks.Length; Write-Host '  Queue Size:' $response.queue_size; Write-Host '  Estimated Reward:' ([math]::Round($response.estimated_reward / 100000000, 8)) 'PAR per task' } catch { Write-Host '[TASK QUEUE]'; Write-Host '  Status: UNAVAILABLE'; Write-Host '  Error:' $_.Exception.Message }" 2>nul

echo.
echo ======================================
echo Press Ctrl+C to stop monitoring
echo Press any key to refresh status...
echo ======================================

timeout /t 10 /nobreak >nul 2>&1
if not errorlevel 1 goto monitor_loop

goto monitor_loop