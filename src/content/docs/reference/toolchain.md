---
title: Toolchain
---
# Toolchain

The `faxt` CLI provides comprehensive tooling for Fax development.

## Commands

### Project Management

| Command | Description | Example |
|---------|-------------|---------|
| `new` | Create new project | `faxt new myapp` |
| `init` | Initialize directory | `faxt init` |
| `add` | Add dependency | `faxt add <url>` |
| `list` | List dependencies | `faxt list` |
| `update` | Update dependencies | `faxt update` |

### Development

| Command | Description | Example |
|---------|-------------|---------|
| `run` | Compile and run | `faxt run file.fax` |
| `build` | Compile to target/ | `faxt build file.fax` |
| `check` | Type check | `faxt check file.fax` |
| `test` | Run tests | `faxt test` |
| `repl` | Interactive shell | `faxt repl` |
| `doc` | Generate docs | `faxt doc` |

### Maintenance

| Command | Description | Example |
|---------|-------------|---------|
| `stats` | Project analysis | `faxt stats` |
| `clean` | Remove artifacts | `faxt clean` |
| `doctor` | Check toolchain | `faxt doctor` |
| `bench` | Benchmark | `faxt bench file.fax` |

## Usage Examples

### Create New Project

```bash
faxt new myapp
cd myapp
```

### Run Program

```bash
faxt run src/main.fax
```

### Build Release

```bash
faxt build src/main.fax
```

### Check Types

```bash
faxt check src/main.fax
```

### Run Tests

```bash
faxt test
```

## Project Structure

A Fax project follows this structure:

```
myapp/
├── Fax.toml          # Project configuration
├── src/
│   └── main.fax      # Entry point
├── target/           # Build output
└── deps/            # Dependencies
```

## Configuration

### Fax.toml

```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2024"

[dependencies]
std = "path/to/std"
```

## Direct CLI Usage

You can also use the Python CLI directly:

```bash
python3 faxt/main.py run examples/hello.fax
python3 faxt/main.py build examples/hello.fax
```
