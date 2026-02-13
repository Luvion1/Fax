#!/bin/bash

# Fax Automated Testing Framework
# Runs comprehensive test suite for the Fax programming language

# Configuration
TEST_DIR="./tests"
LOG_FILE="test_results.log"
SUMMARY_FILE="test_summary.txt"
FAILED_TESTS_FILE="failed_tests.txt"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Function to print colored output
print_color() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to run a single test
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .fax)
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    print_color $BLUE "Running test: $test_name"
    
    # Attempt to compile and run the test
    if ./run_pipeline.sh "$test_file" > "/tmp/${test_name}_out.txt" 2> "/tmp/${test_name}_err.txt"; then
        # Check if the output indicates success
        if [ -s "/tmp/${test_name}_out.txt" ] || [ ! -s "/tmp/${test_name}_err.txt" ]; then
            PASSED_TESTS=$((PASSED_TESTS + 1))
            print_color $GREEN "✓ PASSED: $test_name"
            echo "PASS: $test_name" >> "$LOG_FILE"
        else
            FAILED_TESTS=$((FAILED_TESTS + 1))
            print_color $RED "✗ FAILED: $test_name"
            echo "FAIL: $test_name - Error output: $(cat /tmp/${test_name}_err.txt)" >> "$LOG_FILE"
            echo "$test_file" >> "$FAILED_TESTS_FILE"
        fi
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        print_color $RED "✗ FAILED: $test_name"
        echo "FAIL: $test_name - Compilation error: $(cat /tmp/${test_name}_err.txt)" >> "$LOG_FILE"
        echo "$test_file" >> "$FAILED_TESTS_FILE"
    fi
    
    # Cleanup
    rm -f "/tmp/${test_name}_out.txt" "/tmp/${test_name}_err.txt" "./output" "./output.ll" "temp_"*
}

# Function to run tests by category
run_category_tests() {
    local category=$1
    local pattern=$2
    print_color $YELLOW "Running $category tests..."
    
    for test_file in "$TEST_DIR"/"$pattern".fax; do
        if [ -f "$test_file" ]; then
            run_test "$test_file"
        fi
    done
}

# Function to run all tests
run_all_tests() {
    print_color $YELLOW "Starting comprehensive test suite..."
    print_color $YELLOW "=================================="
    
    # Clear previous results
    > "$LOG_FILE"
    > "$FAILED_TESTS_FILE"
    
    # Run tests by category
    run_category_tests "Arithmetic" "arithmetic_*"
    run_category_tests "Logical" "logical_*"
    run_category_tests "Variable/Scope" "variable_*"
    run_category_tests "Control Flow" "control_*"
    run_category_tests "Data Structures" "data_*"
    run_category_tests "Strings/Characters" "string_*"
    run_category_tests "Memory/Pointers" "memory_*"
    run_category_tests "Concurrency" "concurrency_*"
    run_category_tests "Error Handling" "error_*"
    run_category_tests "Performance" "performance_*"
    
    # Also run any remaining tests that don't match the patterns
    for test_file in "$TEST_DIR"/*.fax; do
        if [ -f "$test_file" ]; then
            # Skip if already tested by pattern
            local basename=$(basename "$test_file")
            if [[ ! "$basename" =~ ^(arithmetic_|logical_|variable_|control_|data_|string_|memory_|concurrency_|error_|performance_) ]]; then
                run_test "$test_file"
            fi
        fi
    done
}

# Function to generate test summary
generate_summary() {
    local end_time=$(date)
    local duration=$(($(date +%s) - start_time))
    
    echo "" >> "$LOG_FILE"
    echo "Test Summary" >> "$LOG_FILE"
    echo "============" >> "$LOG_FILE"
    echo "Start time: $start_time" >> "$LOG_FILE"
    echo "End time: $end_time" >> "$LOG_FILE"
    echo "Duration: $duration seconds" >> "$LOG_FILE"
    echo "Total tests: $TOTAL_TESTS" >> "$LOG_FILE"
    echo "Passed: $PASSED_TESTS" >> "$LOG_FILE"
    echo "Failed: $FAILED_TESTS" >> "$LOG_FILE"
    echo "Skipped: $SKIPPED_TESTS" >> "$LOG_FILE"
    
    # Create summary file
    echo "Fax Language Test Suite Results" > "$SUMMARY_FILE"
    echo "===============================" >> "$SUMMARY_FILE"
    echo "Date: $(date)" >> "$SUMMARY_FILE"
    echo "Total Tests: $TOTAL_TESTS" >> "$SUMMARY_FILE"
    echo "Passed: $PASSED_TESTS" >> "$SUMMARY_FILE"
    echo "Failed: $FAILED_TESTS" >> "$SUMMARY_FILE"
    echo "Success Rate: $((PASSED_TESTS * 100 / TOTAL_TESTS))%" >> "$SUMMARY_FILE"
    
    if [ $FAILED_TESTS -gt 0 ]; then
        echo "" >> "$SUMMARY_FILE"
        echo "Failed Tests:" >> "$SUMMARY_FILE"
        cat "$FAILED_TESTS_FILE" >> "$SUMMARY_FILE"
    fi
    
    print_color $YELLOW "=================================="
    print_color $YELLOW "Test Suite Complete!"
    print_color $BLUE "Total Tests: $TOTAL_TESTS"
    print_color $GREEN "Passed: $PASSED_TESTS"
    print_color $RED "Failed: $FAILED_TESTS"
    print_color $YELLOW "Success Rate: $((PASSED_TESTS * 100 / TOTAL_TESTS))%"
    print_color $YELLOW "See $SUMMARY_FILE for details"
}

# Function to run tests with specific filters
run_filtered_tests() {
    local filter=$1
    print_color $YELLOW "Running tests matching pattern: $filter"
    
    for test_file in "$TEST_DIR"/*"$filter"*.fax; do
        if [ -f "$test_file" ]; then
            run_test "$test_file"
        fi
    done
}

# Function to show test statistics
show_stats() {
    echo "Test Statistics:"
    echo "---------------"
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "Skipped: $SKIPPED_TESTS"
    if [ $TOTAL_TESTS -gt 0 ]; then
        echo "Success Rate: $((PASSED_TESTS * 100 / TOTAL_TESTS))%"
    fi
}

# Main execution
start_time=$(date)
case "${1:-all}" in
    "all")
        run_all_tests
        ;;
    "arithmetic")
        run_category_tests "Arithmetic" "arithmetic_*"
        ;;
    "logical")
        run_category_tests "Logical" "logical_*"
        ;;
    "variables")
        run_category_tests "Variable/Scope" "variable_*"
        ;;
    "control")
        run_category_tests "Control Flow" "control_*"
        ;;
    "data")
        run_category_tests "Data Structures" "data_*"
        ;;
    "strings")
        run_category_tests "Strings/Characters" "string_*"
        ;;
    "memory")
        run_category_tests "Memory/Pointers" "memory_*"
        ;;
    "concurrency")
        run_category_tests "Concurrency" "concurrency_*"
        ;;
    "errors")
        run_category_tests "Error Handling" "error_*"
        ;;
    "performance")
        run_category_tests "Performance" "performance_*"
        ;;
    "filter")
        if [ -n "$2" ]; then
            run_filtered_tests "$2"
        else
            echo "Usage: $0 filter <pattern>"
            exit 1
        fi
        ;;
    "stats")
        show_stats
        exit 0
        ;;
    *)
        echo "Usage: $0 [all|arithmetic|logical|variables|control|data|strings|memory|concurrency|errors|performance|filter <pattern>|stats]"
        exit 1
        ;;
esac

generate_summary