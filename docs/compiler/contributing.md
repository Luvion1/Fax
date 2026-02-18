# Contributing to the Fax Compiler

Guide for contributing to the Fax compiler codebase.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Code Guidelines](#code-guidelines)
4. [Testing](#testing)
5. [Submitting Changes](#submitting-changes)
6. [Code Review](#code-review)

---

## Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/Fax.git
cd Fax
```

### 2. Set Up Development Environment

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
sudo apt-get install llvm-dev clang libssl-dev pkg-config

# Verify setup
cd faxc
cargo check
```

### 3. Understand the Codebase

Read the [Architecture Documentation](architecture.md) to understand the compiler structure.

---

## Development Workflow

### 1. Create a Branch

```bash
# Update main branch
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feat/your-feature-name
```

### 2. Make Changes

```bash
# Edit files
# ...

# Check for errors
cargo check

# Run tests
cargo test
```

### 3. Format and Lint

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

### 4. Commit Changes

```bash
# Stage changes
git add <files>

# Commit with conventional commit message
git commit -m "feat(scope): description of change"
```

---

## Code Guidelines

### Rust Code Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// Use descriptive names
fn calculate_checksum(data: &[u8]) -> u32 {
    // Implementation
}

// Document public APIs
/// Parses a Fax source file and returns an AST.
///
/// # Arguments
///
/// * `source` - The source code to parse
///
/// # Returns
///
/// * `Result<AST, ParseError>` - The parsed AST or an error
pub fn parse(source: &str) -> Result<AST, ParseError> {
    // Implementation
}

// Use Result for error handling
fn process_input(input: &str) -> Result<Output, ProcessingError> {
    // Implementation
}
```

### Project-Specific Conventions

#### Error Handling

```rust
// Use thiserror for error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Type error: {0}")]
    Type(String),
}

// Use anyhow for application-level errors
use anyhow::{Result, Context};

fn compile_file(path: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;
    // Implementation
}
```

#### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test input";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_case() {
        let result = function_under_test("invalid input");
        assert!(result.is_err());
    }
}
```

---

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p faxc-lex

# Specific test
cargo test test_function_name

# With output
cargo test -- --nocapture

# Coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_integer_literal() {
        let mut lexer = Lexer::new("42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(42));
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("hello");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Ident("hello"));
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs

use faxc::compile;

#[test]
fn test_hello_world() {
    let source = r#"
        fn main() {
            println("Hello, World!")
        }
    "#;

    let result = compile(source);
    assert!(result.is_ok());
}
```

---

## Submitting Changes

### 1. Prepare Your PR

```bash
# Update your branch
git fetch upstream
git rebase upstream/main

# Run final checks
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test --all
```

### 2. Create Pull Request

1. Push your branch:
   ```bash
   git push origin feat/your-feature-name
   ```

2. Open PR on GitHub with:
   - Clear title following conventional commits
   - Detailed description
   - Linked issues (e.g., "Closes #123")
   - Checklist completed

### 3. PR Template

Fill out the [PR template](../../.github/PULL_REQUEST_TEMPLATE.md):

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Other

## Testing
- [ ] Tests pass
- [ ] New tests added (if applicable)

## Checklist
- [ ] Code formatted
- [ ] Clippy warnings fixed
- [ ] Documentation updated
```

---

## Code Review

### Reviewer Guidelines

When reviewing code:

1. **Correctness**: Does the code work as intended?
2. **Performance**: Are there obvious performance issues?
3. **Security**: Are there security concerns?
4. **Style**: Does it follow project conventions?
5. **Tests**: Are there adequate tests?

### Review Response Times

- **Bug fixes**: Within 48 hours
- **Features**: Within 1 week
- **Documentation**: Within 1 week

### Addressing Feedback

```bash
# Make requested changes
# ...

# Amend commit or add new commit
git commit --amend  # For small changes
# or
git commit -m "fix: address review comments"

# Push changes
git push origin feat/your-feature-name
```

---

## Areas Needing Contribution

### Good First Issues

Look for issues labeled `good first issue`:
- Documentation improvements
- Test additions
- Simple bug fixes
- Code cleanup

### Help Wanted

Issues labeled `help wanted`:
- Feature implementations
- Performance optimizations
- Refactoring tasks

### Core Development

For experienced contributors:
- Compiler optimizations
- New language features
- GC improvements

---

## Questions?

- **General questions**: [GitHub Discussions](https://github.com/Luvion1/Fax/discussions)
- **Bug reports**: [GitHub Issues](https://github.com/Luvion1/Fax/issues)
- **Code review**: Tag maintainers in your PR

---

<div align="center">

**Thank you for contributing to Fax!** 🙏

</div>
