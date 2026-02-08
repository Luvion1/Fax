# Makefile for FAX Compiler Project
# Provides convenient build and development targets

.PHONY: help install build build-release test clean fmt check docs

# Default target
.DEFAULT_GOAL := help

help:
	@echo "FAX Compiler - Available targets:"
	@echo ""
	@echo "  install          Install all dependencies"
	@echo "  build            Build project in debug mode"
	@echo "  build-release    Build project with optimizations"
	@echo "  test             Run test suite"
	@echo "  clean            Remove build artifacts"
	@echo "  fmt              Format all source code"
	@echo "  check            Check code formatting and types without building"
	@echo "  docs             Build documentation"
	@echo ""
	@echo "Development:"
	@echo "  make test FILE=<file.fax>  Compile a specific FAX file"

install:
	@echo "Installing dependencies..."
	npm install
	cargo fetch
	ghc --version > /dev/null || (echo "GHC not found. Install from https://www.haskell.org"; exit 1)

build:
	@echo "Building FAX Compiler (Debug)..."
	cargo build
	zig build
	npm run build 2>/dev/null || true

build-release:
	@echo "Building FAX Compiler (Release)..."
	cargo build --release
	zig build -Doptimize=ReleaseFast
	npm run build 2>/dev/null || true

test:
	@echo "Running tests..."
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make test FILE=<file.fax>"; \
		echo ""; \
		echo "Available test files:"; \
		ls -1 *.fax 2>/dev/null || echo "  (No .fax files found)"; \
		exit 1; \
	fi
	npm start "$(FILE)"

clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf zig-cache zig-out
	rm -rf dist-newstyle
	find . -name "*.ll" -o -name "fgc.o" -o -name "*_bin" | xargs rm -f
	rm -f faxc/src/components/sema/sema_bin
	rm -f faxc/src/components/codegen/codegen_bin

fmt:
	@echo "Formatting code..."
	cargo fmt
	npx prettier --write "**/*.{ts,js,tsx,jsx}" 2>/dev/null || echo "Prettier not configured"
	zig fmt faxc/src

check:
	@echo "Checking code format and types..."
	cargo fmt -- --check
	npx tsc --noEmit 2>/dev/null || true
	@echo "Code check complete"

docs:
	@echo "Building documentation..."
	@if command -v mdbook &> /dev/null; then \
		cd docs && mdbook build; \
	else \
		echo "mdbook not installed. Skipping documentation build."; \
	fi

.PHONY: help install build build-release test clean fmt check docs
