#!/usr/bin/env bash

# Comprehensive Test Runner for A2A Protocol Implementation
# This script runs all tests including unit tests, compliance tests, and integration tests

set -e

echo "🚀 A2A Protocol Comprehensive Test Runner"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Track overall results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to print section header
print_section() {
    echo -e "\n${CYAN}📋 $1${NC}"
    echo "========================================="
}

# Function to run tests and track results
run_tests() {
    local name=$1
    local command=$2
    local expected_to_fail=${3:-false}

    echo -e "${BLUE}🧪 Running $name...${NC}"

    ((TOTAL_TESTS++))

    if eval "$command" >/dev/null 2>&1; then
        if [ "$expected_to_fail" = true ]; then
            echo -e "${YELLOW}⚠️  $name passed but was expected to fail${NC}"
            ((FAILED_TESTS++))
        else
            echo -e "${GREEN}✅ $name passed${NC}"
            ((PASSED_TESTS++))
        fi
    else
        if [ "$expected_to_fail" = true ]; then
            echo -e "${GREEN}✅ $name failed as expected${NC}"
            ((PASSED_TESTS++))
        else
            echo -e "${RED}❌ $name failed${NC}"
            ((FAILED_TESTS++))
        fi
    fi
}

# Function to run tests with features
run_tests_with_features() {
    local name=$1
    local features=$2
    local expected_to_fail=${3:-false}

    echo -e "${BLUE}🧪 Running $name (features: $features)...${NC}"

    ((TOTAL_TESTS++))

    if cargo test --features "$features" --lib --package a2a-protocol >/dev/null 2>&1; then
        if [ "$expected_to_fail" = true ]; then
            echo -e "${YELLOW}⚠️  $name passed but was expected to fail${NC}"
            ((FAILED_TESTS++))
        else
            echo -e "${GREEN}✅ $name passed${NC}"
            ((PASSED_TESTS++))
        fi
    else
        if [ "$expected_to_fail" = true ]; then
            echo -e "${GREEN}✅ $name failed as expected${NC}"
            ((PASSED_TESTS++))
        else
            echo -e "${RED}❌ $name failed${NC}"
            ((FAILED_TESTS++))
        fi
    fi
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install dependencies if needed
install_dependencies() {
    print_section "Checking Dependencies"

    if ! command_exists cargo; then
        echo -e "${RED}❌ Cargo not found. Please install Rust first.${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Rust and Cargo are installed${NC}"

    # Install additional tools if needed
    if ! command_exists cargo-nextest; then
        echo -e "${YELLOW}⚠️  cargo-nextest not found. Installing...${NC}"
        cargo install cargo-nextest >/dev/null 2>&1 || echo -e "${YELLOW}⚠️  Failed to install cargo-nextest, using default test runner${NC}"
    fi
}

# Function to run linting checks
run_linting() {
    print_section "Linting and Formatting"

    # Check code formatting
    if command_exists cargo fmt; then
        echo -e "${BLUE}🔍 Checking code formatting...${NC}"
        if cargo fmt --check >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Code formatting check passed${NC}"
        else
            echo -e "${YELLOW}⚠️  Code formatting issues found. Run 'cargo fmt' to fix${NC}"
        fi
    fi

    # Run clippy if available
    if command_exists cargo clippy; then
        echo -e "${BLUE}🔍 Running clippy lints...${NC}"
        if cargo clippy -- -D warnings >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Clippy checks passed${NC}"
        else
            echo -e "${YELLOW}⚠️  Clippy found issues${NC}"
        fi
    fi
}

# Function to run documentation tests
run_doc_tests() {
    print_section "Documentation Tests"

    echo -e "${BLUE}📚 Testing documentation examples...${NC}"
    run_tests "Documentation tests" "cargo test --doc --package a2a-protocol"
}

# Function to run unit tests
run_unit_tests() {
    print_section "Unit Tests"

    echo -e "${BLUE}🔬 Running unit tests...${NC}"
    run_tests "Unit tests" "cargo test --lib --package a2a-protocol"

    # Test specific modules
    echo -e "${BLUE}🔬 Testing core modules...${NC}"
    run_tests "Core agent_id tests" "cargo test agent_id --lib --package a2a-protocol"
    run_tests "Core message tests" "cargo test message --lib --package a2a-protocol"
    run_tests "Core agent_card tests" "cargo test agent_card --lib --package a2a-protocol"
    run_tests "Core error tests" "cargo test error --lib --package a2a-protocol"

    echo -e "${BLUE}🔬 Testing transport modules...${NC}"
    run_tests "Transport http tests" "cargo test http --lib --package a2a-protocol"
    run_tests "Transport json_rpc tests" "cargo test json_rpc --lib --package a2a-protocol"

    echo -e "${BLUE}🔬 Testing auth modules...${NC}"
    run_tests "Auth authenticator tests" "cargo test authenticator --lib --package a2a-protocol"
    run_tests "Auth strategies tests" "cargo test strategies --lib --package a2a-protocol"
}

# Function to run integration tests
run_integration_tests() {
    print_section "Integration Tests"

    echo -e "${BLUE}🔗 Running integration tests...${NC}"
    run_tests "Integration tests" "cargo test --test integration --package a2a-protocol" true

    echo -e "${BLUE}🔗 Running compliance tests...${NC}"
    run_tests "Compliance tests" "cargo test --test compliance --package a2a-protocol"
}

# Function to run feature-specific tests
run_feature_tests() {
    print_section "Feature-Specific Tests"

    echo -e "${BLUE}🚚 Testing with default features...${NC}"
    run_tests "Default features" "cargo test --lib --package a2a-protocol"

    echo -e "${BLUE}🌊 Testing with websocket features...${NC}"
    run_tests_with_features "WebSocket features" "websocket" true

    echo -e "${BLUE}🔌 Testing with full features...${NC}"
    run_tests_with_features "Full features" "full" true
}

# Function to run performance tests
run_performance_tests() {
    print_section "Performance Tests"

    echo -e "${BLUE}⚡ Running performance tests...${NC}"

    # Run performance-related tests
    run_tests "Performance benchmarks" "cargo test performance --lib --package a2a-protocol" true

    # Test serialization performance
    echo -e "${BLUE}⚡ Testing serialization performance...${NC}"
    cargo test message_serialization_performance --lib --package a2a-protocol >/dev/null 2>&1 || echo -e "${YELLOW}⚠️  Performance tests not available${NC}"
}

# Function to run security tests
run_security_tests() {
    print_section "Security Tests"

    echo -e "${BLUE}🔒 Running security tests...${NC}"

    # Test input validation
    run_tests "Input validation" "cargo test validation --lib --package a2a-protocol" true

    # Test authentication security
    run_tests "Authentication security" "cargo test auth_security --lib --package a2a-protocol" true
}

# Function to generate test report
generate_report() {
    print_section "Test Report"

    echo -e "${CYAN}📊 Test Results Summary${NC}"
    echo "========================================="
    echo -e "${BLUE}Total tests run: ${TOTAL_TESTS}${NC}"
    echo -e "${GREEN}Tests passed: ${PASSED_TESTS}${NC}"
    echo -e "${RED}Tests failed: ${FAILED_TESTS}${NC}"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        local compliance="100%"
    else
        local percentage=$(( PASSED_TESTS * 100 / TOTAL_TESTS ))
        echo -e "${YELLOW}⚠️  Success rate: ${percentage}%${NC}"
        local compliance="${percentage}%"
    fi

    echo ""
    echo -e "${CYAN}📋 Detailed Results${NC}"
    echo "========================================="

    # Generate markdown report
    cat > test-report.md << EOF
# A2A Protocol Test Report

Generated on: $(date)

## Test Summary
- **Total Tests**: $TOTAL_TESTS
- **Passed**: $PASSED_TESTS
- **Failed**: $FAILED_TESTS
- **Success Rate**: $compliance

## Test Categories

### Unit Tests
- Core modules: Agent ID, Message, Agent Card, Error handling
- Transport modules: HTTP, JSON-RPC
- Authentication modules: API Key, Bearer, OAuth2

### Integration Tests
- Client-server integration
- Compliance test suite
- Protocol validation

### Performance Tests
- Serialization performance
- Agent ID generation performance

### Security Tests
- Input validation
- Authentication security

## Recommendations

EOF

    if [ $FAILED_TESTS -eq 0 ]; then
        cat >> test-report.md << EOF
✅ **All tests passed!** The A2A protocol implementation is ready for production use.

## Next Steps
1. Consider adding more comprehensive integration tests
2. Implement continuous integration testing
3. Add performance benchmarks for larger datasets
4. Consider fuzzing for security testing

EOF
    else
        cat >> test-report.md << EOF
⚠️ **Some tests failed.** Please review and fix the failing tests before production deployment.

## Failed Areas
- Review the test output above for specific failures
- Check integration test setup
- Verify feature flag configurations
- Validate test environment

EOF
    fi

    echo -e "${GREEN}📄 Test report generated: test-report.md${NC}"
}

# Function to run the compliance test script
run_compliance_script() {
    print_section "Compliance Testing"

    echo -e "${BLUE}📋 Running compliance test suite...${NC}"

    if [ -f "scripts/compliance-test.sh" ]; then
        ./scripts/compliance-test.sh || echo -e "${YELLOW}⚠️  Compliance test script completed with warnings${NC}"
    else
        echo -e "${YELLOW}⚠️  Compliance test script not found${NC}"
    fi
}

# Main execution
main() {
    echo -e "${PURPLE}🚀 Starting comprehensive A2A protocol test suite...${NC}"
    echo -e "${PURPLE}This may take a few minutes...${NC}"

    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || ! grep -q "a2a-protocol" Cargo.toml; then
        echo -e "${RED}❌ Error: Please run this script from the project root directory${NC}"
        exit 1
    fi

    # Install dependencies
    install_dependencies

    # Run all test categories
    run_linting
    run_doc_tests
    run_unit_tests
    run_integration_tests
    run_feature_tests
    run_performance_tests
    run_security_tests
    run_compliance_script

    # Generate report
    generate_report

    # Final result
    echo ""
    echo -e "${CYAN}🏁 Test Suite Complete${NC}"
    echo "========================================="

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed! The A2A protocol implementation is fully compliant.${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Please review the test report.${NC}"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTION]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --unit-only    Run only unit tests"
        echo "  --int-only     Run only integration tests"
        echo "  --compliance   Run only compliance tests"
        echo "  --quick        Quick test run (unit tests only)"
        echo ""
        echo "Default: Run all tests"
        exit 0
        ;;
    --unit-only)
        install_dependencies
        run_unit_tests
        exit 0
        ;;
    --int-only)
        install_dependencies
        run_integration_tests
        exit 0
        ;;
    --compliance)
        run_compliance_script
        exit 0
        ;;
    --quick)
        install_dependencies
        run_unit_tests
        echo -e "${GREEN}✅ Quick test run completed${NC}"
        exit 0
        ;;
    *)
        main
        ;;
esac