# Contributing to Fax Compiler

## Getting Started

1. Install Rust: https://rustup.rs/
2. Clone repository
3. Run: `./scripts/build.sh`
4. Run tests: `./scripts/test.sh`

## Development Workflow

1. Create feature branch
2. Make changes
3. Add tests
4. Run: `cargo fmt && cargo clippy`
5. Submit PR

## Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Document public APIs
- Add tests for new features

## Testing

- Unit tests: In each crate's `tests/` directory
- Integration tests: In `tests/integ/`
- UI tests: In `tests/ui/`

## Commit Messages

Format: `<type>(<scope>): <subject>`

Types: feat, fix, docs, style, refactor, test, chore