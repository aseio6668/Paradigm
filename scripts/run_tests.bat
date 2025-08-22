@echo off
REM Comprehensive test runner script for Paradigm SDK
REM Runs all types of tests including unit, integration, property, security, and benchmarks

echo Starting Paradigm SDK Test Suite...
echo.

REM Set environment variables
set RUST_BACKTRACE=1
set RUST_LOG=info

REM Change to project directory
cd /d "%~dp0\.."

REM Check if Cargo is available
cargo --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: Cargo not found. Please install Rust and Cargo.
    exit /b 1
)

echo ===========================================
echo Running Unit Tests
echo ===========================================
cargo test --lib --features="testing,security"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Unit tests failed
    exit /b 1
)
echo.

echo ===========================================
echo Running Integration Tests
echo ===========================================
cargo test --test integration_tests --features="testing,security"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Integration tests failed
    exit /b 1
)
echo.

echo ===========================================
echo Running Property-Based Tests
echo ===========================================
cargo test --test property_tests --features="testing,security"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Property-based tests failed
    exit /b 1
)
echo.

echo ===========================================
echo Running Security Tests
echo ===========================================
cargo test --test security_tests --features="testing,security"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Security tests failed
    exit /b 1
)
echo.

echo ===========================================
echo Running Fuzz Tests
echo ===========================================
cargo test --test fuzzing --features="testing,security,fuzzing"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Fuzz tests failed
    exit /b 1
)
echo.

echo ===========================================
echo Running Documentation Tests
echo ===========================================
cargo test --doc --features="testing,security"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Documentation tests failed
    exit /b 1
)
echo.

echo ===========================================
echo Running Clippy Lints
echo ===========================================
cargo clippy --all-targets --all-features -- -D warnings
if %ERRORLEVEL% neq 0 (
    echo ERROR: Clippy lints failed
    exit /b 1
)
echo.

echo ===========================================
echo Checking Code Formatting
echo ===========================================
cargo fmt --all -- --check
if %ERRORLEVEL% neq 0 (
    echo ERROR: Code formatting check failed
    echo Run 'cargo fmt --all' to fix formatting issues
    exit /b 1
)
echo.

echo ===========================================
echo Running Security Audit
echo ===========================================
cargo audit
if %ERRORLEVEL% neq 0 (
    echo WARNING: Security audit found issues
    echo This may indicate vulnerable dependencies
)
echo.

echo ===========================================
echo Running Benchmarks (Optional)
echo ===========================================
echo Running performance benchmarks...
cargo bench --features="security" --bench crypto_benchmarks
if %ERRORLEVEL% neq 0 (
    echo WARNING: Benchmarks failed or not available
)
echo.

echo ===========================================
echo Generating Test Coverage Report
echo ===========================================
echo Attempting to generate coverage report...
cargo tarpaulin --all-features --out Html --output-dir target/coverage 2>nul
if %ERRORLEVEL% neq 0 (
    echo INFO: Coverage report generation skipped (tarpaulin not installed)
    echo Install with: cargo install cargo-tarpaulin
) else (
    echo Coverage report generated in target/coverage/tarpaulin-report.html
)
echo.

echo ===========================================
echo Test Summary
echo ===========================================
echo ✅ Unit Tests: PASSED
echo ✅ Integration Tests: PASSED  
echo ✅ Property-Based Tests: PASSED
echo ✅ Security Tests: PASSED
echo ✅ Fuzz Tests: PASSED
echo ✅ Documentation Tests: PASSED
echo ✅ Clippy Lints: PASSED
echo ✅ Code Formatting: PASSED
echo.
echo All tests completed successfully!
echo.
echo To run specific test categories:
echo   Unit tests:        cargo test --lib
echo   Integration tests: cargo test --test integration_tests
echo   Security tests:    cargo test --test security_tests  
echo   Property tests:    cargo test --test property_tests
echo   Fuzz tests:        cargo test --test fuzzing
echo   Benchmarks:        cargo bench
echo.
echo For continuous testing during development:
echo   cargo watch -x test
echo.

pause