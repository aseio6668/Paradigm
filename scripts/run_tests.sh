#!/bin/bash

# Comprehensive test runner script for Paradigm SDK
# Runs all types of tests including unit, integration, property, security, and benchmarks

set -e  # Exit on any error

echo "Starting Paradigm SDK Test Suite..."
echo

# Set environment variables
export RUST_BACKTRACE=1
export RUST_LOG=info

# Change to project directory
cd "$(dirname "$0")/.."

# Check if Cargo is available
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Cargo not found. Please install Rust and Cargo."
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Track test results
declare -a test_results=()

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    print_status "Running $test_name"
    
    if eval "$test_command"; then
        print_success "$test_name: PASSED"
        test_results+=("✅ $test_name: PASSED")
        return 0
    else
        print_error "$test_name: FAILED"
        test_results+=("❌ $test_name: FAILED")
        return 1
    fi
}

# Run all test categories
echo "Running comprehensive test suite for Paradigm SDK..."
echo

# Unit Tests
run_test "Unit Tests" "cargo test --lib --features='testing,security'"

# Integration Tests  
run_test "Integration Tests" "cargo test --test integration_tests --features='testing,security'"

# Property-Based Tests
run_test "Property-Based Tests" "cargo test --test property_tests --features='testing,security'"

# Security Tests
run_test "Security Tests" "cargo test --test security_tests --features='testing,security'"

# Fuzz Tests
run_test "Fuzz Tests" "cargo test --test fuzzing --features='testing,security,fuzzing'"

# Documentation Tests
run_test "Documentation Tests" "cargo test --doc --features='testing,security'"

# Code Quality Checks
print_status "Running Clippy Lints"
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Clippy Lints: PASSED"
    test_results+=("✅ Clippy Lints: PASSED")
else
    print_error "Clippy Lints: FAILED" 
    test_results+=("❌ Clippy Lints: FAILED")
fi

echo

print_status "Checking Code Formatting"
if cargo fmt --all -- --check; then
    print_success "Code Formatting: PASSED"
    test_results+=("✅ Code Formatting: PASSED")
else
    print_error "Code Formatting: FAILED"
    print_warning "Run 'cargo fmt --all' to fix formatting issues"
    test_results+=("❌ Code Formatting: FAILED")
fi

echo

# Security Audit
print_status "Running Security Audit"
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        print_success "Security Audit: PASSED"
        test_results+=("✅ Security Audit: PASSED")
    else
        print_warning "Security Audit: FOUND ISSUES"
        print_warning "This may indicate vulnerable dependencies"
        test_results+=("⚠️  Security Audit: FOUND ISSUES")
    fi
else
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
    test_results+=("⚠️  Security Audit: SKIPPED (not installed)")
fi

echo

# Benchmarks (optional)
print_status "Running Benchmarks (Optional)"
if cargo bench --features="security" --bench crypto_benchmarks 2>/dev/null; then
    print_success "Benchmarks: COMPLETED"
    test_results+=("✅ Benchmarks: COMPLETED")
else
    print_warning "Benchmarks: SKIPPED (not available or failed)"
    test_results+=("⚠️  Benchmarks: SKIPPED")
fi

echo

# Coverage Report (optional)
print_status "Generating Test Coverage Report"
if command -v cargo-tarpaulin &> /dev/null; then
    if cargo tarpaulin --all-features --out Html --output-dir target/coverage; then
        print_success "Coverage report generated in target/coverage/tarpaulin-report.html"
        test_results+=("✅ Coverage Report: GENERATED")
    else
        print_warning "Coverage report generation failed"
        test_results+=("⚠️  Coverage Report: FAILED")
    fi
else
    print_warning "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
    test_results+=("⚠️  Coverage Report: SKIPPED (not installed)")
fi

echo

# Test Summary
print_status "Test Summary"
for result in "${test_results[@]}"; do
    echo -e "$result"
done

echo
echo -e "${BLUE}All core tests completed!${NC}"
echo

# Check if any critical tests failed
failed_tests=$(printf '%s\n' "${test_results[@]}" | grep -c "❌" || true)
if [ "$failed_tests" -gt 0 ]; then
    echo -e "${RED}$failed_tests critical test(s) failed. Please fix before proceeding.${NC}"
    exit 1
else
    echo -e "${GREEN}All critical tests passed successfully!${NC}"
fi

echo
echo "To run specific test categories:"
echo "  Unit tests:        cargo test --lib"
echo "  Integration tests: cargo test --test integration_tests" 
echo "  Security tests:    cargo test --test security_tests"
echo "  Property tests:    cargo test --test property_tests"
echo "  Fuzz tests:        cargo test --test fuzzing"
echo "  Benchmarks:        cargo bench"
echo
echo "For continuous testing during development:"
echo "  cargo watch -x test"
echo
echo "For release testing with optimizations:"
echo "  cargo test --release --all-features"
echo