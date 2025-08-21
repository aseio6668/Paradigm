@echo off
REM Paradigm Network Launcher (Windows)
REM Production-ready network deployment with monitoring and synchronization

setlocal enabledelayedexpansion

REM Configuration
set NETWORK_NAME=paradigm-mainnet
set LOG_DIR=.\logs
set PID_DIR=.\pids
set DATA_DIR=.\data
set CONFIG_DIR=.\config

REM Create directories
if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"
if not exist "%PID_DIR%" mkdir "%PID_DIR%"
if not exist "%DATA_DIR%" mkdir "%DATA_DIR%"
if not exist "%CONFIG_DIR%" mkdir "%CONFIG_DIR%"

REM Colors (Windows compatible)
set "GREEN=[92m"
set "YELLOW=[93m"
set "RED=[91m"
set "BLUE=[94m"
set "NC=[0m"

:print_header
echo %BLUE%╔══════════════════════════════════════════════════════════════╗%NC%
echo %BLUE%║                    Paradigm Network Launcher                ║%NC%
echo %BLUE%║                  Production Network Manager                  ║%NC%
echo %BLUE%╚══════════════════════════════════════════════════════════════╝%NC%
echo.
goto :eof

:print_status
echo %GREEN%[INFO]%NC% %~1
goto :eof

:print_warning
echo %YELLOW%[WARN]%NC% %~1
goto :eof

:print_error
echo %RED%[ERROR]%NC% %~1
goto :eof

:is_process_running
set "pid_file=%~1"
if exist "%pid_file%" (
    set /p pid=<"%pid_file%"
    tasklist /fi "pid eq !pid!" 2>nul | find "!pid!" >nul
    if !errorlevel! equ 0 (
        exit /b 0
    ) else (
        del "%pid_file%" 2>nul
        exit /b 1
    )
)
exit /b 1

:start_core_node
set "node_id=%~1"
set "port=%~2"
set "p2p_port=%~3"
if "%port%"=="" set port=8080
if "%p2p_port%"=="" set p2p_port=9000

set "pid_file=%PID_DIR%\core-node-%node_id%.pid"
set "log_file=%LOG_DIR%\core-node-%node_id%.log"

call :is_process_running "%pid_file%"
if !errorlevel! equ 0 (
    set /p existing_pid=<"%pid_file%"
    call :print_warning "Core node %node_id% already running (PID: !existing_pid!)"
    goto :eof
)

call :print_status "Starting core node %node_id% (HTTP: %port%, P2P: %p2p_port%)..."

REM Start core node in background
start /b "" cmd /c "set RUST_LOG=info && set PARADIGM_PORT=%port% && set PARADIGM_DATA_DIR=%DATA_DIR%\node-%node_id% && cargo run -p paradigm-core -- --port %port% > %log_file% 2>&1"

REM Get the PID (simplified for Windows)
timeout /t 2 /nobreak >nul
for /f "tokens=2" %%i in ('tasklist /fi "imagename eq cargo.exe" /fo csv ^| find "cargo"') do (
    echo %%~i > "%pid_file%"
    goto found_pid
)
:found_pid

timeout /t 3 /nobreak >nul

call :is_process_running "%pid_file%"
if !errorlevel! equ 0 (
    set /p new_pid=<"%pid_file%"
    call :print_status "Core node %node_id% started successfully (PID: !new_pid!)"
) else (
    call :print_error "Failed to start core node %node_id%"
)
goto :eof

:start_contributor
set "contrib_id=%~1"
set "threads=%~2"
set "node_port=%~3"
if "%threads%"=="" set threads=4
if "%node_port%"=="" set node_port=8080

set "pid_file=%PID_DIR%\contributor-%contrib_id%.pid"
set "log_file=%LOG_DIR%\contributor-%contrib_id%.log"

call :is_process_running "%pid_file%"
if !errorlevel! equ 0 (
    set /p existing_pid=<"%pid_file%"
    call :print_warning "Contributor %contrib_id% already running (PID: !existing_pid!)"
    goto :eof
)

call :print_status "Starting contributor %contrib_id% (%threads% threads, connecting to port %node_port%)..."

REM Start contributor in background
start /b "" cmd /c "set RUST_LOG=info && set PARADIGM_NODE_URL=http://127.0.0.1:%node_port% && cargo run -p paradigm-contributor -- --threads %threads% --verbose > %log_file% 2>&1"

REM Get the PID (simplified for Windows)
timeout /t 2 /nobreak >nul
for /f "tokens=2" %%i in ('tasklist /fi "imagename eq cargo.exe" /fo csv ^| find "cargo"') do (
    echo %%~i > "%pid_file%"
    goto found_contrib_pid
)
:found_contrib_pid

timeout /t 2 /nobreak >nul

call :is_process_running "%pid_file%"
if !errorlevel! equ 0 (
    set /p new_pid=<"%pid_file%"
    call :print_status "Contributor %contrib_id% started successfully (PID: !new_pid!)"
) else (
    call :print_error "Failed to start contributor %contrib_id%"
)
goto :eof

:stop_process
set "name=%~1"
set "pid_file=%~2"

call :is_process_running "%pid_file%"
if !errorlevel! equ 0 (
    set /p pid=<"%pid_file%"
    call :print_status "Stopping %name% (PID: !pid!)..."
    
    taskkill /pid !pid! /f >nul 2>&1
    del "%pid_file%" 2>nul
    call :print_status "%name% stopped"
) else (
    call :print_warning "%name% not running"
)
goto :eof

:show_status
echo %BLUE%╔══════════════════════════════════════════════════════════════╗%NC%
echo %BLUE%║                      Network Status                         ║%NC%
echo %BLUE%╚══════════════════════════════════════════════════════════════╝%NC%
echo.

echo %GREEN%Core Nodes:%NC%
for %%f in ("%PID_DIR%\core-node-*.pid") do (
    set "filename=%%~nf"
    set "node_id=!filename:core-node-=!"
    call :is_process_running "%%f"
    if !errorlevel! equ 0 (
        set /p pid=<"%%f"
        echo   ✅ Node !node_id! (PID: !pid!)
    ) else (
        echo   ❌ Node !node_id! (stopped)
    )
)

echo.
echo %GREEN%Contributors:%NC%
for %%f in ("%PID_DIR%\contributor-*.pid") do (
    set "filename=%%~nf"
    set "contrib_id=!filename:contributor-=!"
    call :is_process_running "%%f"
    if !errorlevel! equ 0 (
        set /p pid=<"%%f"
        echo   ✅ Contributor !contrib_id! (PID: !pid!)
    ) else (
        echo   ❌ Contributor !contrib_id! (stopped)
    )
)

echo.
echo %GREEN%Log Files:%NC%
if exist "%LOG_DIR%\*.log" (
    dir /b "%LOG_DIR%\*.log" 2>nul
) else (
    echo   No log files found
)
goto :eof

:cleanup
echo.
call :print_status "Shutting down network..."

REM Stop all contributors
for %%f in ("%PID_DIR%\contributor-*.pid") do (
    set "filename=%%~nf"
    set "contrib_id=!filename:contributor-=!"
    call :stop_process "Contributor !contrib_id!" "%%f"
)

REM Stop all core nodes
for %%f in ("%PID_DIR%\core-node-*.pid") do (
    set "filename=%%~nf"
    set "node_id=!filename:core-node-=!"
    call :stop_process "Core Node !node_id!" "%%f"
)

call :print_status "Network shutdown complete"
goto :eof

REM Main execution
if "%1"=="" set command=start
if not "%1"=="" set command=%1

if "%command%"=="start" goto start_network
if "%command%"=="stop" goto stop_network
if "%command%"=="status" goto status_network
if "%command%"=="monitor" goto monitor_network
if "%command%"=="logs" goto show_logs
if "%command%"=="restart" goto restart_network
goto show_help

:start_network
call :print_header
call :print_status "Building Paradigm network..."
cargo build --release

call :print_status "Starting production network..."

REM Start core nodes
call :start_core_node "1" "8080" "9000"
timeout /t 2 /nobreak >nul
call :start_core_node "2" "8081" "9001"
timeout /t 2 /nobreak >nul
call :start_core_node "3" "8082" "9002"

REM Wait for nodes to initialize
timeout /t 5 /nobreak >nul

REM Start contributors
call :start_contributor "1" "8" "8080"
call :start_contributor "2" "6" "8081"
call :start_contributor "3" "4" "8082"
call :start_contributor "4" "4" "8080"
call :start_contributor "5" "2" "8081"

call :print_status "Network started successfully!"
call :show_status

echo.
echo %GREEN%Network URLs:%NC%
echo   Node 1: http://127.0.0.1:8080
echo   Node 2: http://127.0.0.1:8081
echo   Node 3: http://127.0.0.1:8082
echo.
echo %BLUE%Commands:%NC%
echo   launch-network.bat monitor    # Monitor network health
echo   launch-network.bat status     # Show current status
echo   launch-network.bat stop       # Stop the network
echo   launch-network.bat logs       # Show recent logs
goto :eof

:stop_network
call :cleanup
goto :eof

:status_network
call :show_status
goto :eof

:monitor_network
call :print_header
call :print_status "Starting network monitoring... (Press Ctrl+C to stop)"
:monitor_loop
cls
call :show_status
echo.
echo %BLUE%Monitoring... Press Ctrl+C to stop%NC%
timeout /t 10 /nobreak >nul
goto monitor_loop

:show_logs
call :print_status "Recent network logs:"
echo.
for %%f in ("%LOG_DIR%\*.log") do (
    echo %GREEN%=== %%~nxf ===%NC%
    powershell "Get-Content '%%f' | Select-Object -Last 20"
    echo.
)
goto :eof

:restart_network
call :print_status "Restarting network..."
call :cleanup
timeout /t 3 /nobreak >nul
goto start_network

:show_help
echo Usage: %0 {start^|stop^|status^|monitor^|logs^|restart}
echo.
echo Commands:
echo   start    - Start the production network
echo   stop     - Stop all network components
echo   status   - Show current network status
echo   monitor  - Monitor and show network status
echo   logs     - Show recent logs from all components
echo   restart  - Restart the entire network
goto :eof
