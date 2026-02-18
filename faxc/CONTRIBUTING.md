# Contributing to Fax Compiler

## Minimum Supported Rust Version (MSRV)

This project supports **Rust 1.75 and later**. All contributions must be compatible with MSRV.

### MSRV Guidelines

- **Do not use** language features stabilized after Rust 1.75
- **Do not depend** on crate versions that require newer Rust
- **Test locally** with MSRV before submitting PRs:

```bash
# Install MSRV toolchain
rustup install 1.75

# Test with MSRV
rustup run 1.75 cargo check --workspace
rustup run 1.75 cargo test --workspace
rustup run 1.75 cargo clippy --workspace -- -D warnings
```

- CI automatically tests all PRs against Rust 1.75

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