# Toolchain Guide (faxt)

`faxt` is the unified CLI for Fax development.

## Project Management

| Command | Description |
|---------|-------------|
| `new <name>` | Create a new project directory |
| `init` | Initialize Fax in current directory |
| `add <url>` | Add a git-based dependency |
| `list` | Show project dependencies |
| `update` | Sync dependencies |

## Development

- **`run [file]`**: Compile and execute immediately.
- **`build [file]`**: Generate binary in `target/`.
- **`check [file]`**: Fast type-checking without codegen.
- **`test`**: Execute all `test_*.fax` and `tests/*.fax`.
- **`repl`**: Start an interactive Fax shell.
- **`fmt`**: Format source files to standard style.

## Maintenance

- **`stats`**: Display LOC and compiler metrics.
- **`doctor`**: Verify toolchain requirements.
- **`bench [file]`**: Profile compile and run times.
- **`clean`**: Remove all build artifacts.
