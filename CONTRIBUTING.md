# Contributing to Fax Compiler

Thank you for your interest in contributing to the Fax programming language! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Commit Messages](#commit-messages)
- [Questions?](#questions)

---

## Code of Conduct

This project adheres to the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

---

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/faxc.git
   cd faxc
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/username/faxc.git
   ```
4. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

---

## Development Setup

### Prerequisites

- **Rust 1.75+** ([install via rustup](https://rustup.rs))
- **LLVM 14+** (for code generation)
- **Git**

### Build the Project

```bash
# Debug build
./scripts/build.sh

# Release build
./scripts/build.sh --release

# Run all tests
./scripts/test.sh

# Run specific test suite
cargo test -p fgc
```

### Verify Your Setup

```bash
# Check Rust version
rustc --version  # Must be >= 1.75.0

# Build and test
cargo build --all
cargo test --all
```

---

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug fixes** - Fix issues in the compiler, GC, or tooling
- **New features** - Implement new language features or compiler optimizations
- **Documentation** - Improve docs, add examples, fix typos
- **Tests** - Add test cases for existing or new functionality
- **Performance** - Optimize compiler or runtime performance
- **Code review** - Review pull requests and provide feedback

### Finding Issues

- Check [open issues](https://github.com/username/faxc/issues) for bugs and feature requests
- Look for issues labeled `good first issue` for beginner-friendly tasks
- Look for issues labeled `help wanted` for tasks needing assistance

---

## Pull Request Process

### Before Submitting

1. **Ensure your code compiles** without warnings
2. **Run all tests** and ensure they pass
3. **Update documentation** if you changed behavior
4. **Add tests** for new functionality
5. **Check code formatting**:
   ```bash
   cargo fmt --all -- --check
   ```

### Submitting a PR

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a pull request** on GitHub with:
   - Clear, descriptive title
   - Detailed description of changes
   - Reference to related issues (e.g., "Closes #123")
   - Screenshots or examples if applicable

3. **Wait for CI checks** to pass (automated tests, linting, etc.)

4. **Respond to feedback** from reviewers

### PR Requirements

- [ ] Code follows project style guidelines
- [ ] Tests pass locally and in CI
- [ ] Documentation is updated
- [ ] Commit messages follow Conventional Commits
- [ ] No new compiler warnings introduced

---

## Coding Standards

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` for linting:
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

### Fax Code (examples/tests)

- Follow the [Language Specification](SPEC.md)
- Use clear, descriptive variable names
- Include comments for complex logic

### File Organization

- Keep functions small and focused (single responsibility)
- Group related functionality in modules
- Use descriptive file and module names

---

## Testing

### Running Tests

```bash
# All tests
./scripts/test.sh

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific crate
cargo test -p fgc

# With coverage
./scripts/test.sh --coverage
```

### Writing Tests

- **Unit tests**: Test individual functions/modules
- **Integration tests**: Test complete workflows
- **Regression tests**: Add tests for fixed bugs

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

---

## Documentation

### Code Documentation

- Document all public APIs with rustdoc comments:
  ```rust
  /// Brief description of the function.
  ///
  /// Detailed explanation with examples if needed.
  ///
  /// # Arguments
  ///
  /// * `arg1` - Description of argument
  ///
  /// # Returns
  ///
  /// Description of return value
  ///
  /// # Example
  ///
  /// ```
  /// let result = function(arg1);
  /// ```
  pub fn function(arg1: Type) -> ReturnType {
      // ...
  }
  ```

### Documentation Files

- Update relevant `.md` files for significant changes
- Keep examples in sync with code changes
- Update the [SPEC.md](SPEC.md) for language changes

---

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(parser): add support for tuple patterns

Implemented pattern matching for tuple expressions in match arms.

Closes #42

fix(gc): prevent use-after-free in mark phase

Added proper root scanning to ensure all live objects are marked.

docs(spec): update function type syntax

Clarified function type grammar in section 5.2.5.
```

---

## Questions?

- **General questions**: Open a [Discussion](https://github.com/username/faxc/discussions)
- **Bug reports**: Open an [Issue](https://github.com/username/faxc/issues)
- **Code review**: Tag relevant maintainers in your PR

---

## License

By contributing to Fax, you agree that your contributions will be licensed under the project's dual license (MIT OR Apache-2.0). See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
