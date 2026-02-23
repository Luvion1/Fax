#!/bin/bash
# Fax Compiler - Run Examples Script
# Compiles and runs Fax example programs

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
EXAMPLES_DIR="$PROJECT_DIR/faxc/examples"
FAXC_BIN=""

# Find faxc binary
find_faxc() {
    # Check in target directories
    if [ -f "$PROJECT_DIR/target/debug/faxc" ]; then
        FAXC_BIN="$PROJECT_DIR/target/debug/faxc"
    elif [ -f "$PROJECT_DIR/target/release/faxc" ]; then
        FAXC_BIN="$PROJECT_DIR/target/release/faxc"
    elif [ -f "$PROJECT_DIR/target/debug/faxc-drv" ]; then
        FAXC_BIN="$PROJECT_DIR/target/debug/faxc-drv"
    elif [ -f "$PROJECT_DIR/target/release/faxc-drv" ]; then
        FAXC_BIN="$PROJECT_DIR/target/release/faxc-drv"
    else
        # Try to build first
        echo "Fax compiler not found. Building..."
        cd "$PROJECT_DIR"
        cargo build -p faxc-drv
        
        if [ -f "$PROJECT_DIR/target/debug/faxc-drv" ]; then
            FAXC_BIN="$PROJECT_DIR/target/debug/faxc-drv"
        elif [ -f "$PROJECT_DIR/target/release/faxc-drv" ]; then
            FAXC_BIN="$PROJECT_DIR/target/release/faxc-drv"
        fi
    fi
}

# Compile and run a single file
run_file() {
    local file="$1"
    local name=$(basename "$file" .fax)
    
    echo "=========================================="
    echo "Running: $name"
    echo "=========================================="
    echo ""
    echo "Source code:"
    echo "-------------------------------------------"
    cat "$file"
    echo "-------------------------------------------"
    echo ""
    
    # Compile
    local output="/tmp/fax_output_$$"
    echo "Compiling..."
    
    if [ -n "$FAXC_BIN" ]; then
        "$FAXC_BIN" "$file" -o "$output" 2>&1 || {
            echo "Compilation failed!"
            return 1
        }
    else
        echo "Error: faxc binary not found"
        return 1
    fi
    
    # Run
    if [ -f "$output" ]; then
        echo "Output:"
        echo "-------------------------------------------"
        "$output" 2>&1 || echo "(program exited with code: $?)"
        echo "-------------------------------------------"
        echo ""
        
        # Cleanup
        rm -f "$output"
    else
        echo "Error: output file not created"
        return 1
    fi
}

# List all examples
list_examples() {
    echo "Available examples:"
    echo ""
    local i=1
    for file in "$EXAMPLES_DIR"/*.fax; do
        if [ -f "$file" ]; then
            local name=$(basename "$file" .fax)
            printf "  %2d) %s\n" "$i" "$name"
            ((i++))
        fi
    done
    echo ""
}

# Show usage
usage() {
    echo "Fax Compiler - Run Examples"
    echo ""
    echo "Usage: $0 [OPTIONS] [EXAMPLE]"
    echo ""
    echo "Options:"
    echo "  --help, -h           Show this help message"
    echo "  --list, -l           List all available examples"
    echo "  --all, -a           Run all examples"
    echo "  --build, -b          Build before running"
    echo "  --release, -r       Use release build"
    echo "  --emit TYPE          Emit type: tokens, ast, hir, mir, lir, asm, llvm-ir"
    echo ""
    echo "Examples:"
   0                    # Show echo "  $ this help"
    echo "  $0 --list             # List all examples"
    echo "  $0 01                 # Run example 01 (hello)"
    echo "  $0 hello              # Run hello.fax"
    echo "  $0 --all              # Run all examples"
    echo "  $0 --emit asm hello   # Emit assembly for hello.fax"
    echo ""
}

# Main
main() {
    local run_all=false
    local emit_type=""
    local use_release=false
    local build_first=false
    local example_num=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)
                usage
                exit 0
                ;;
            --list|-l)
                find_faxc
                list_examples
                exit 0
                ;;
            --all|-a)
                run_all=true
                shift
                ;;
            --build|-b)
                build_first=true
                shift
                ;;
            --release|-r)
                use_release=true
                shift
                ;;
            --emit)
                emit_type="$2"
                shift 2
                ;;
            *)
                if [[ "$1" =~ ^[0-9]+$ ]]; then
                    example_num="$1"
                else
                    example_num="$1"
                fi
                shift
                ;;
        esac
    done
    
    # Find or build faxc
    find_faxc
    
    if [ -z "$FAXC_BIN" ]; then
        echo "Error: Could not find or build faxc compiler"
        exit 1
    fi
    
    echo "Using faxc: $FAXC_BIN"
    echo ""
    
    # Build if requested
    if [ "$build_first" = true ]; then
        echo "Building faxc..."
        cd "$PROJECT_DIR"
        if [ "$use_release" = true ]; then
            cargo build --release -p faxc-drv
            FAXC_BIN="$PROJECT_DIR/target/release/faxc-drv"
        else
            cargo build -p faxc-drv
            FAXC_BIN="$PROJECT_DIR/target/debug/faxc-drv"
        fi
    fi
    
    # Run all examples or specific one
    if [ "$run_all" = true ]; then
        for file in "$EXAMPLES_DIR"/*.fax; do
            if [ -f "$file" ]; then
                run_file "$file"
            fi
        done
    elif [ -n "$example_num" ]; then
        # Try to find the example by number or name
        local target_file=""
        
        # Check if it's a number
        if [[ "$example_num" =~ ^[0-9]+$ ]]; then
            local i=1
            for file in "$EXAMPLES_DIR"/*.fax; do
                if [ -f "$file" ] && [ $i -eq "$example_num" ]; then
                    target_file="$file"
                    break
                fi
                ((i++))
            done
        else
            # Check by filename
            if [ -f "$EXAMPLES_DIR/${example_num}.fax" ]; then
                target_file="$EXAMPLES_DIR/${example_num}.fax"
            elif [ -f "$EXAMPLES_DIR/${example_num}.fax" ]; then
                target_file="$EXAMPLES_DIR/${example_num}.fax"
            fi
        fi
        
        if [ -n "$target_file" ] && [ -f "$target_file" ]; then
            run_file "$target_file"
        else
            echo "Example not found: $example_num"
            list_examples
            exit 1
        fi
    else
        # No arguments - show help
        usage
        list_examples
    fi
}

main "$@"
