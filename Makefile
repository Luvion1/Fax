# Fax Compiler Makefile
# Convenience commands for building and testing

.PHONY: all build test clean docker help

# Default target
all: build

# Build the project
build:
	@echo "Building Fax Compiler..."
	lake build

# Build in release mode
release:
	@echo "Building in release mode..."
	lake build --release

# Run all tests
test: test-unit test-integration test-e2e
	@echo "All tests completed!"

# Run unit tests
test-unit:
	@echo "Running unit tests..."
	lake exe test-unit-lexer
	lake exe test-unit-parser
	lake exe test-unit-codegen
	lake exe test-unit-semantic
	lake exe test-unit-gc

# Run integration tests
test-integration:
	@echo "Running integration tests..."
	lake exe test-integration

# Run E2E tests
test-e2e:
	@echo "Running E2E tests..."
	lake exe test-e2e

# Run benchmarks
benchmark:
	@echo "Running benchmarks..."
	lake exe benchmark-gc

# Run GC stress test
stress-test:
	@echo "Running GC stress test..."
	lake exe stress-test-gc -- --duration 60

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	lake clean
	rm -rf .lake/build
	rm -rf build

# Docker commands
docker-build:
	@echo "Building Docker image..."
	docker build -t fax:latest .

docker-run:
	@echo "Running Fax Compiler in Docker..."
	docker run --rm -v $(PWD)/examples:/workspace/examples fax:latest

docker-dev:
	@echo "Starting development environment..."
	docker-compose up -d dev
	docker-compose exec dev bash

docker-test:
	@echo "Running tests in Docker..."
	docker-compose run --rm test

# Example compilation
examples:
	@echo "Compiling example programs..."
	@for f in examples/*.fax; do \
		echo "Compiling $$f..."; \
		./faxc.sh $$f || true; \
	done

# Format code
format:
	@echo "Formatting code..."
	# Add formatting commands here

# Lint code
lint:
	@echo "Linting code..."
	# Add linting commands here

# Generate documentation
docs:
	@echo "Generating documentation..."
	lake exe doc-gen

# Watch mode for development
watch:
	@echo "Starting watch mode..."
	find . -name "*.lean" | entr -r make build

# Install locally
install: build
	@echo "Installing Fax Compiler..."
	cp .lake/build/bin/faxc /usr/local/bin/faxc
	chmod +x /usr/local/bin/faxc

# Uninstall
uninstall:
	@echo "Uninstalling Fax Compiler..."
	rm -f /usr/local/bin/faxc

# Help
help:
	@echo "Fax Compiler Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all              - Build the project (default)"
	@echo "  build            - Build the project"
	@echo "  release          - Build in release mode"
	@echo "  test             - Run all tests"
	@echo "  test-unit        - Run unit tests"
	@echo "  test-integration - Run integration tests"
	@echo "  test-e2e         - Run end-to-end tests"
	@echo "  benchmark        - Run benchmarks"
	@echo "  stress-test      - Run GC stress test"
	@echo "  clean            - Clean build artifacts"
	@echo "  docker-build     - Build Docker image"
	@echo "  docker-run       - Run in Docker"
	@echo "  docker-dev       - Start development environment"
	@echo "  examples         - Compile example programs"
	@echo "  format           - Format code"
	@echo "  lint             - Lint code"
	@echo "  docs             - Generate documentation"
	@echo "  install          - Install locally"
	@echo "  uninstall        - Uninstall"
	@echo "  help             - Show this help message"
